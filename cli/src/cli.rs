use std::path::PathBuf;

use clap::{Parser, Subcommand};
use log::error;

use crate::sim;
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
            parse(from_os_str),
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
            parse(from_os_str),
            value_name = "TOML",
            default_value = "../support_apps/config/control_config.toml"
        )]
        config: PathBuf,
    },

    /// Physics simulation app
    ///
    /// Configure an asynchronous physics simulation in the background. This
    /// simulation runs on the MFC with flight software code running in the
    /// loop and logs the simulation output to a CSV file.
    Sim {
        /// Simulator app commands
        #[clap(subcommand)]
        cmd: SimCmds,
    },
}

#[derive(Subcommand, Debug)]
enum SimCmds {
    /// Start a new simulation process
    Start {
        /// Sets a custom simulation config file
        #[clap(
            short,
            long,
            parse(from_os_str),
            value_name = "TOML",
            default_value = "../simulator/config/sim_config.toml"
        )]
        sim_config: PathBuf,

        /// Sets a custom output file
        #[clap(
            short,
            long,
            parse(from_os_str),
            value_name = "CSV",
            default_value = "./out.csv"
        )]
        outfile: PathBuf,
    },

    /// Inspect a physics parameter in an existing simulation
    Get {
        /// Parameter to be inspect
        param: String,
    },

    /// Modify a physics parameter in an existing simulation
    Set {
        /// Parameter to be modified
        param: String,
        /// New value to set
        value: String,
    },
}

pub fn parse_inputs() {
    // parse CLI input args and options
    let cli = Cli::parse();
    match &cli.command {
        Commands::Status {} => status::full_report(),
        Commands::Sim { cmd } => match cmd {
            SimCmds::Start {
                sim_config,
                outfile,
            } => {
                sim::start_sim(sim_config, outfile);
            }
            _ => {
                error!("Command not implemented yet!")
            }
        },
        Commands::AltCtrl { config } => {
            let control_mngr = sys::init_altctrl(config);
        }
        _ => {
            error!("Command not implemented yet!")
        }
    }
}
