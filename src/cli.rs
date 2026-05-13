use clap::{Parser, Subcommand};
use anyhow::Result;

use crate::commands::{init, generate, clean, version, list_services, stacks};

pub const VERSION: &str = "3.0.0";
pub const DEFAULT_STACK: &str = "trivadis/platys-modern-data-platform";

pub fn version_info() -> String {
    format!(
        "Platys - Trivadis Platform in a Box - v {VERSION}\n\
         https://github.com/trivadispf/platys\n\
         Copyright (c) 2018-2020, Trivadis AG"
    )
}

/// Platys platform generator
#[derive(Parser)]
#[command(
    name = "platys",
    about = "Platys platform generator",
    long_about = version_info(),
    version = VERSION,
)]
pub struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
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
    #[command(name = "list_services")]
    ListServices(list_services::ListServicesArgs),

    /// Lists the predefined stacks available for the init command
    Stacks(stacks::StacksArgs),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            None => {
                // Print help when no subcommand given (mirrors Go behaviour)
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
            Some(Commands::Version) => {
                version::run();
                Ok(())
            }
            Some(Commands::Init(args)) => init::run(args, self.verbose).await,
            Some(Commands::Gen(args)) => generate::run(args, self.verbose).await,
            Some(Commands::Clean(args)) => clean::run(args, self.verbose).await,
            Some(Commands::ListServices(args)) => list_services::run(args).await,
            Some(Commands::Stacks(args)) => stacks::run(args).await,
        }
    }
}
