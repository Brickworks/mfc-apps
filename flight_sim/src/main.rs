extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate libm;

mod atmosphere;
mod gas;
mod utils;

fn main() {
    pretty_env_logger::init(); // initialize pretty print

    let mut lift_gas = gas::GasVolume::new(gas::GasSpecies::He, 1.0);
    lift_gas.set_temperature(utils::temperature_c2k(25.0));
    info!("{:}", lift_gas);
    lift_gas.set_temperature(utils::temperature_c2k(400.0));
    info!("{:}", lift_gas);
    let altitude = 10.0; // [m]
    let ambient_temp = atmosphere::temperature(altitude);
    let ambient_pres = atmosphere::pressure(altitude);
    info!(
        "Ambient conditions at {:} m: {:} K | {:} Pa",
        altitude, ambient_temp, ambient_pres
    );
    let altitude = 10000.0; // [m]
    let ambient_temp = atmosphere::temperature(altitude);
    let ambient_pres = atmosphere::pressure(altitude);
    info!(
        "Ambient conditions at {:} m: {:} K | {:} Pa",
        altitude, ambient_temp, ambient_pres
    );
    let altitude = 50000.0; // [m]
    let ambient_temp = atmosphere::temperature(altitude);
    let ambient_pres = atmosphere::pressure(altitude);
    info!(
        "Ambient conditions at {:} m: {:} K | {:} Pa",
        altitude, ambient_temp, ambient_pres
    );

    let altitude = 100000.0; // [m]
    let ambient_temp = atmosphere::temperature(altitude);
    let ambient_pres = atmosphere::pressure(altitude);
    info!(
        "Ambient conditions at {:} m: {:} K | {:} Pa",
        altitude, ambient_temp, ambient_pres
    );
}
