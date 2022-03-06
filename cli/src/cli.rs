use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::{trace, debug, info, warn, error};

use crate::status;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// generate a status report
    Status {},

    /// configure and run a simulation
    ///
    /// Configure an asynchronous physics simulation in the background. This
    /// simulation runs on the MFC with flight software code running in the
    /// loop and logs the simulation output to a CSV file.
    Sim {
        /// Sets a custom config file
        #[clap(short, long, parse(from_os_str), value_name = "FILE")]
        config: Option<PathBuf>,
    },
}

pub fn parse_inputs() {
    // parse CLI input args and options
    let cli = Cli::parse();
    match &cli.command {
        Commands::Status {} => {
            status::full_report()
        },
        _ => {
            error!("Command not implemented yet!")
        }
    }
}