use crate::cli::DEFAULT_STACK;
use crate::config::{
    add_root_indent, parse_config, print_banner,
    serialize_config,
};
use crate::docker::pull_config;
use anyhow::{bail, Context, Result};
use clap::Args;

use std::fs;

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

    //parse into config struct
    let mut config = parse_config(&raw_config)?;

    // ── Enable requested services ────────────────────────────────────────
    if !args.enable_services.is_empty() {
        let requested: Vec<&str> = args.enable_services.split(',').collect();

        //keep only the services requested by the user
        config
            .services
            .retain(|name, _| requested.contains(&name.as_str()));

        //enable de services requested by user
        for svc in config.services.values_mut() {
            svc.enabled = true
        }
    }

    // ── override platform-name if empty ────────────────────────────────────────────────
    if !args.platform_name.is_empty() {
        config.platys.platform_name = args.platform_name.clone();
    }

    // ── override structure ────────────────────────────────────────────────────
    if !args.structure.is_empty() {
        if args.structure != "flat" && args.structure != "subfolder" {
            bail!(
                "Invalid [structure] value '{}'. Accepted: flat | subfolder",
                args.structure
            );
        }
        config.platys.structure = args.structure.clone();
    }

    let yaml_str = serialize_config(&config)?;
    let indented = add_root_indent(&yaml_str, 6);
    fs::write(&args.config_file, indented)
        .with_context(|| format!("failed to write {}", args.config_file))?;
    print_banner(&args.config_file);
    Ok(())
}
