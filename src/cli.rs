use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{clean, generate, init, list_services, stacks, version};

pub const DEFAULT_STACK: &str = "trivadis/platys-modern-data-platform";

pub const VERSION_INFO: &str = concat!(
    "Platys - Trivadis Platform in a Box - v ",
    env!("CARGO_PKG_VERSION"),
    "\nhttps://github.com/trivadispf/platys\n",
    "Copyright (c) 2018-2026, Trivadis AG",
);

/// Platys platform generator
#[derive(Parser)]
#[command(
    name = "platys",
    about = "Platys platform generator",
    long_about = VERSION_INFO,
    version,
    arg_required_else_help = true,
)]
pub struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print the version number of platys
    Version,

    /// Initializes the current directory to be the root for the Modern (Data) Platform
    /// by creating an initial config file, if one does not already exist.
    Init(init::InitArgs),

    /// Generates all the needed artifacts for the docker-based modern (data) platform
    Gen(generate::GenArgs),

    /// Cleans the contents in the $PATH/container-volume folder
    Clean(clean::CleanArgs),

    /// List the services contained in the given version of the platys tool
    ListServices(list_services::ListServicesArgs),

    /// Lists the predefined stacks available for the init command
    Stacks(stacks::StacksArgs),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Version => {
                version::run();
                Ok(())
            }
            Commands::Init(args) => init::run(args).await,
            Commands::Gen(args) => generate::run(args).await,
            Commands::Clean(args) => clean::run(args).await,
            Commands::ListServices(args) => list_services::run(args).await,
            Commands::Stacks(args) => stacks::run(args).await,
        }
    }
}
