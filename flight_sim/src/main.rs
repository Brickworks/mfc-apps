extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate libm;

mod gas;
mod force;

fn main() {
    pretty_env_logger::init(); // initialize pretty print

    let mut altitude = 10000.0;
    let mut atmo = gas::Atmosphere::new(altitude);
    let mut lift_gas = gas::GasVolume::new(gas::GasSpecies::He, 0.5);
    lift_gas.update_from_ambient(atmo);
    info!("force: {:} N", force::net_force(altitude, 5.0, atmo, lift_gas, 0.25, 2.0));
    altitude = 25000.0;
    atmo.set_altitude(altitude);
    lift_gas.update_from_ambient(atmo);
    info!("force: {:} N", force::net_force(altitude, 5.0, atmo, lift_gas, 0.25, 2.0));
    altitude = 35000.0;
    atmo.set_altitude(altitude);
    lift_gas.update_from_ambient(atmo);
    info!("force: {:} N", force::net_force(altitude, 5.0, atmo, lift_gas, 0.25, 2.0));
}
