use anyhow::Result;
use clap::Args;

use crate::cli::DEFAULT_STACK;
use crate::config::parse_config;
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
    let cfg = parse_config(&raw)?;

    let banner = "*".repeat(94);
    println!("{banner}");
    println!(
        "* The following services are available in [ {} : {} ]  *",
        args.stack, args.stack_version
    );
    println!("{banner}");

    for service_name in cfg.services.keys() {
        println!("{service_name}");
    }

    Ok(())
}
