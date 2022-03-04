use std::path::PathBuf;

use clap::{Parser, Subcommand};

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

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    name: Option<String>,

    /// Turn debugging information on
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[clap(short, long)]
        list: bool,

        /// Sets a custom config file
        #[clap(short, long, parse(from_os_str), value_name = "FILE")]
        config: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {}", name);
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.verbose {
        0 => println!("quiet mode"),
        1 => println!("verbose mode"),
        2 => println!("very verbose mode"),
        _ => println!("can't get any more verbose!"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Test { list, config }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        None => {}
    }

    // Continued program logic goes here...
}