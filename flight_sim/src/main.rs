extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod gas;

fn main() {
    let x = gas::GasVolume::new("helium", 1.0);
    warn!("{:}", &x.density());
}
