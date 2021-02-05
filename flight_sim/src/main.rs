extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod gas;

fn main() {
    let x = gas::GasVolume::new(gas::GasSpecies::He, 1.0);
    println!("{:}", &x.density());
}
