use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::error;

use crate::status;
use crate::sys;

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

    /// Start necessary systems for flight
    FlightReady {
        /// Altitude controller configuration to use
        #[clap(
            short,
            long,
            value_name = "TOML",
            default_value = "../support_apps/config/control_config.toml"
        )]
        altctrl_config: PathBuf,
    },

    /// Start altitude control app
    AltCtrl {
        /// Controller configuration to use
        #[clap(
            short,
            long,
            value_name = "TOML",
            default_value = "../support_apps/config/control_config.toml"
        )]
        config: PathBuf,
    },
}

pub fn parse_inputs() {
    // parse CLI input args and options
    let cli = Cli::parse();
    match &cli.command {
        Commands::Status {} => status::full_report(),
        Commands::AltCtrl { config } => {
            sys::init_altctrl(config);
        }
        _ => {
            error!("Command not implemented yet!")
        }
    }
}
