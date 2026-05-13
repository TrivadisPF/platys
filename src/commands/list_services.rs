use anyhow::{Context, Result};
use clap::Args;

use crate::cli::DEFAULT_STACK;
use crate::config::service_regex;
use crate::docker::pull_config;

#[derive(Args, Debug)]
pub struct ListServicesArgs {
    /// Stack image to query
    #[arg(short = 's', long = "stack", default_value = DEFAULT_STACK)]
    pub stack: String,

    /// Version of the stack
    #[arg(short = 'w', long = "stack-version", default_value = "latest")]
    pub stack_version: String,
}

pub async fn run(args: ListServicesArgs) -> Result<()> {
    let raw = pull_config(&args.stack, &args.stack_version).await?;

    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&raw).context("Failed to parse config YAML")?;

    let re = service_regex();

    println!("{}",
        "*".repeat(94)
    );
    println!(
        "* The following services are available in [ {} : {} ]  *",
        args.stack, args.stack_version
    );
    println!("{}", "*".repeat(94));

    if let Some(mapping) = parsed.as_mapping() {
        for (key, _) in mapping {
            let key_str = key.as_str().unwrap_or("");
            if let Some(caps) = re.captures(key_str) {
                println!("{}", &caps[1]);
            }
        }
    }

    Ok(())
}
