extern crate pretty_env_logger;
mod cli;
mod status;
mod sys;

fn main() {
    // initialize pretty print logger
    pretty_env_logger::init();
    // parse the commands, arguments, and options
    cli::parse_inputs();
}