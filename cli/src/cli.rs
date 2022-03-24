use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::error;

use crate::sim;
use crate::status;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

/*
    start physics sim   start an asychronous physics simulation
                        and start writing the results to a csv
    
    modify sim param    "god mode" modify a parameter of the
                        simulation on the fly
                        - dry mass, ballast mass, gas mass
                        - position, speed, acceleration
                        - air temperature, pressure, density

    boot up             start the main execution loop, initialize
                        the logger and critical apps
    
    toggle subsystem    turn on or off an app or hardware system
                        - control app
                        - radio
                        - sensor, GPS, microcontroller
*/

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a system status report
    Status {},

    /// Configure and run a simulation
    ///
    /// Configure an asynchronous physics simulation in the background. This
    /// simulation runs on the MFC with flight software code running in the
    /// loop and logs the simulation output to a CSV file.
    Sim {
        /// Sets a custom altitude controller config file
        #[clap(
            short,
            long,
            parse(from_os_str),
            value_name = "FILE",
            default_value = "../support_apps/config/control_config.toml"
        )]
        ctrl_config: PathBuf,

        /// Sets a custom simulation config file
        #[clap(
            short,
            long,
            parse(from_os_str),
            value_name = "FILE",
            default_value = "../support_apps/config/sim_config.toml"
        )]
        sim_config: PathBuf,

        /// Sets a custom output file
        #[clap(
            short,
            long,
            parse(from_os_str),
            value_name = "FILE",
            default_value = "./out.csv"
        )]
        outfile: PathBuf,
    },

    /// Initiate flight sequence
    Boot {
        /// Sets a custom altitude controller config file
        #[clap(
            short,
            long,
            parse(from_os_str),
            value_name = "FILE",
            default_value = "../support_apps/config/control_config.toml"
        )]
        config: PathBuf,
    },
}

pub fn parse_inputs() {
    // parse CLI input args and options
    let cli = Cli::parse();
    match &cli.command {
        Commands::Status {} => {
            // print system information
            status::full_report()
        }
        Commands::Sim {
            ctrl_config,
            sim_config,
            outfile,
        } => {
            sim::start_sim(ctrl_config, sim_config, outfile);
        }
        _ => {
            error!("Command not implemented yet!")
        }
    }
}
