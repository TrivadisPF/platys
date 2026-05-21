use anyhow::{bail, Context, Result};
use clap::Args;
use serde_yaml::Value;
use std::fs;
use crate::cli::DEFAULT_STACK;
use crate::config::{add_root_indent, is_service_key, is_service_property, print_banner};
use crate::docker::pull_config;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Comma-separated list of services to enable in the config file
    #[arg(short = 'y', long = "enable-services", default_value = "")]
    pub enable_services: String,

    /// Overwrite an existing config file
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Hardware architecture for the platform
    #[arg(short = 'x', long = "hw-arch", default_value = "x86-64")]
    pub hw_arch: String,

    /// The name of a predefined stack to base this new platform on
    #[arg(short = 'e', long = "seed-config", default_value = "")]
    pub seed_config: String,

    /// The name of the local config file
    #[arg(short = 'c', long = "config-file", default_value = "config.yml")]
    pub config_file: String,

    /// Structure of the generated platform: flat | subfolder
    #[arg(short = 'b', long = "structure", default_value = "")]
    pub structure: String,

    /// The name of the platform to generate
    #[arg(short = 'n', long = "platform-name", default_value = "")]
    pub platform_name: String,

    /// Stack image to use
    #[arg(short = 's', long = "stack", default_value = DEFAULT_STACK)]
    pub stack: String,

    /// Version of the stack to use
    #[arg(short = 'w', long = "stack-version", default_value = "latest")]
    pub stack_version: String,
}

pub async fn run(args: InitArgs) -> Result<()> {
    log::info!("Running using config file [{}]", args.config_file);

    // Guard against overwriting existing config
    if fs::metadata(&args.config_file).is_ok() && !args.force {
        bail!(
            "[{}] already exists. Use --force to overwrite.",
            args.config_file
        );
    }

    // Pull the template config.yml from the Docker image
    let raw_config = pull_config(&args.stack, &args.stack_version).await?;

    // Parse into a generic YAML value so we can surgically edit it
    let mut root: Value = serde_yaml::from_str(&raw_config)
        .context("Failed to parse config from Docker image")?;

    // ── Enable requested services ────────────────────────────────────────
    if !args.enable_services.is_empty() {
        let services: Vec<&str> = args.enable_services.split(',').collect();
        filter_and_enable_services(&mut root, &services)?;
    }

    // ── Set platform-name ────────────────────────────────────────────────
    if !args.platform_name.is_empty() {
        update_platys_key(&mut root, "platform-name", &args.platform_name);
    }

    // ── Set structure ────────────────────────────────────────────────────
    if !args.structure.is_empty() {
        if args.structure != "flat" && args.structure != "subfolder" {
            bail!(
                "Invalid [structure] value '{}'. Accepted: flat | subfolder",
                args.structure
            );
        }
        update_platys_key(&mut root, "structure", &args.structure);
    }

    // ── Serialize and write ──────────────────────────────────────────────
    let yaml_str = serde_yaml::to_string(&root).context("Failed to serialise config")?;
    let indented = add_root_indent(&yaml_str, 6);

    fs::write(&args.config_file, indented)
        .with_context(|| format!("Failed to write {}", args.config_file))?;

    print_banner(&args.config_file);
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Walks the top-level YAML mapping and keeps only:
///  - keys containing "platys"
///  - keys containing "use_timezone" or "private_docker_repository_name"
///  - the `_enable` key + all property keys for any requested service
fn filter_and_enable_services(root: &mut Value, services: &[&str]) -> Result<()> {
    let mapping = root
        .as_mapping_mut()
        .context("Expected top-level YAML mapping")?;

    let mut current_service = String::new();
    let mut to_keep: Vec<(Value, Value)> = Vec::new();

    let pairs: Vec<(Value, Value)> = mapping
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    for (key, mut value) in pairs {
        let key_str = key.as_str().unwrap_or("");

        // Track which service we're currently inside
        let (is_svc_enable, svc_name) = is_service_key(key_str);
        if is_svc_enable {
            current_service = svc_name.clone();
        }

        let keep = if key_str.contains("platys")
            || key_str.contains("use_timezone")
            || key_str.contains("private_docker_repository_name")
        {
            true
        } else if services.contains(&current_service.as_str()) {
            if !is_service_property(&current_service, key_str) {
                // This is the `_enable` key — flip it on
                log::info!("Enabling service [{}]", current_service);
                value = Value::Bool(true);
                true
            } else {
                log::info!(
                    "Grabbing service property [{}] for service [{}]",
                    key_str, current_service
                );
                true
            }
        } else {
            false
        };

        if keep {
            to_keep.push((key, value));
        }
    }

    // Replace the mapping content with only the kept pairs
    *mapping = serde_yaml::Mapping::new();
    for (k, v) in to_keep {
        mapping.insert(k, v);
    }

    Ok(())
}

/// Updates a key inside the nested `platys:` section of the config.
fn update_platys_key(root: &mut Value, name: &str, value: &str) {
    if let Some(mapping) = root.as_mapping_mut() {
        if let Some(platys_val) = mapping.get_mut("platys") {
            if let Some(inner) = platys_val.as_mapping_mut() {
                inner.insert(
                    Value::String(name.to_string()),
                    Value::String(value.to_string()),
                );
            }
        }
    }
}
