extern crate pretty_env_logger;
mod cli;
mod status;
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

fn main() {
    // initialize pretty print logger
    pretty_env_logger::init();
    // parse the commands, arguments, and options
    cli::parse_inputs();
}