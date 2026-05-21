use anyhow::{bail, Context, Result};
use bollard::container::{
    Config, CreateContainerOptions, LogsOptions, RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions
};
use bollard::models::HostConfig;
use bollard::models::{Mount, MountTypeEnum};
use clap::Args;
use futures_util::StreamExt;
use std::fs;
use std::path::PathBuf;


use crate::config::{download_remote_file, node_limit, validate_platys, YamlFile};
use crate::docker::{init_client, wait_for_container};

#[derive(Args, Debug)]
pub struct GenArgs {
    /// Remove empty lines from the generated docker-compose.yml
    #[arg(short = 'l', long = "del-empty-lines", default_value_t = true)]
    pub del_empty_lines: bool,

    /// URL to a remote config file (overrides --config-file)
    #[arg(short = 'u', long = "config-url", default_value = "")]
    pub config_url: String,

    /// The name of the local config file
    #[arg(short = 'c', long = "config-file", default_value = "config.yml")]
    pub config_file: String,
}

pub async fn run(mut args: GenArgs) -> Result<()> {
    // ── Resolve config file ──────────────────────────────────────────────
    if !args.config_url.is_empty() {
        log::info!(
            "[configUrl] was defined with value [{}]; overwriting config file",
            args.config_url
        );
        let path = download_remote_file(&args.config_url)?;
        args.config_file = path.to_string_lossy().into_owned();
    }

    if args.config_file.is_empty() {
        bail!("Unable to run: config file is not set");
    }

    let yml_content = fs::read_to_string(&args.config_file)
        .with_context(|| format!("Cannot read config file [{}]", args.config_file))?;

    // ── Parse config ─────────────────────────────────────────────────────
    let platys: YamlFile =
        serde_yaml::from_str(&yml_content).context("Failed to parse config YAML")?;
    let services: serde_yaml::Value =
        serde_yaml::from_str(&yml_content).context("Failed to parse config YAML (services)")?;

    validate_platys(&platys, &yml_content)?;

    // ── Check node limits ────────────────────────────────────────────────
    if let Some(mapping) = services.as_mapping() {
        for (key, value) in mapping {
            let key_str = key.as_str().unwrap_or("");
            if let Some(max) = node_limit(key_str) {
                if let Some(val_str) = value.as_str() {
                    let val: u32 = val_str.parse().unwrap_or(0);
                    if val > max {
                        bail!(
                            "Number of nodes for [{}] = {} exceeds maximum = {}",
                            key_str,
                            val,
                            max
                        );
                    }
                }
            }
        }
    }

    log::debug!("Using config [{}]: platform-name={}, stack={}, stack-version={}, structure={}",
            args.config_file,
            platys.platys.platform_name,
            platys.platys.platform_stack,
            platys.platys.platform_stack_version,
            platys.platys.structure);


    // ── Determine output destination ─────────────────────────────────────
    let current_dir = std::env::current_dir().context("Cannot determine current directory")?;
    let mut destination = current_dir.clone();

    if platys.platys.structure == "subfolder" {
        destination = destination.join(&platys.platys.platform_name);
        fs::create_dir_all(&destination)
            .with_context(|| format!("Failed to create destination {:?}", destination))?;
        eprintln!("Generating stack on [{:?}]", destination);
    }

    //when verbose is passed to the main class it set logging level to debug  @see main.rs
    let verbose = log::max_level() >= log::LevelFilter::Debug;


    // ── Build environment for the container ──────────────────────────────
    let mut env: Vec<String> = Vec::new();
    env.push(format!("VERBOSE={}", if verbose { 1 } else { 0 }));
    //casting bool to u8 will give either 1 or 0
    env.push(format!("DEL_EMPTY_LINES={}", args.del_empty_lines as u8));

    // ── Run the generator container ──────────────────────────────────────
    let stack = &platys.platys.platform_stack;
    let version = &platys.platys.platform_stack_version;

    let docker = init_client(stack, version).await?;

    let full_config_path = PathBuf::from(&args.config_file)
        .canonicalize()
        .context("Failed to resolve config file path")?;

    // On Windows we skip setting the User field
    let user = if cfg!(target_os = "windows") {
        None
    } else {
        let uid = users_current_uid();
        let gid = users_current_gid();
        Some(format!("{uid}:{gid}"))
    };

    let container_config = Config {
        image: Some(format!("{stack}:{version}")),
        tty: Some(true),
        attach_stdout: Some(!cfg!(target_os = "windows")),
        attach_stderr: Some(!cfg!(target_os = "windows")),
        env: Some(env.clone()),
        user: user.clone(),
        ..Default::default()
    };

    let host_config = HostConfig {
        mounts: Some(vec![
            Mount {
                target: Some("/tmp/config.yml".to_string()),
                source: Some(full_config_path.to_string_lossy().into_owned()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            },
            Mount {
                target: Some("/opt/mdps-gen/destination".to_string()),
                source: Some(destination.to_string_lossy().into_owned()),
                typ: Some(MountTypeEnum::BIND),
                read_only: Some(false),
                ..Default::default()
            },
        ]),
        ..Default::default()
    };

    let resp = docker
        .create_container(
            Some(CreateContainerOptions {
                name: "platys",
                platform: None,
            }),
            Config {
                host_config: Some(host_config),
                ..container_config
            },
        )
        .await
        .context("Failed to create generator container")?;

    docker
        .start_container(&resp.id, None::<StartContainerOptions<String>>)
        .await
        .context("Failed to start generator container")?;

    // Wait for completion
    wait_for_container(&docker, &resp.id,).await?;

    // Stream logs to stdout
    let mut log_stream = docker.logs(
        &resp.id,
        Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            ..Default::default()
        }),
    );
    while let Some(item) = log_stream.next().await {
        match item {
            Ok(chunk) => print!("{chunk}"),
            Err(e) => eprintln!("Log error: {e}"),
        }
    }

    // Cleanup
    docker
        .stop_container(&resp.id, Some(StopContainerOptions { t: 0 }))
        .await
        .context("Failed to stop container")?;
    docker
        .remove_container(
            &resp.id,
            Some(RemoveContainerOptions {
                force: false,
                ..Default::default()
            }),
        )
        .await
        .context("Failed to remove container")?;

    Ok(())
}

// ── UID/GID helpers (Unix only) ───────────────────────────────────────────────

#[cfg(unix)]
fn users_current_uid() -> u32 {
    unsafe { libc::getuid() }
}

#[cfg(unix)]
fn users_current_gid() -> u32 {
    unsafe { libc::getgid() }
}

#[cfg(not(unix))]
fn users_current_uid() -> u32 { 0 }

#[cfg(not(unix))]
fn users_current_gid() -> u32 { 0 }
