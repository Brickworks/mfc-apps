// ----------------------------------------------------------------------------
// Simulate
// --------
// Coordinate execution of a discrete-time simulation.
// ----------------------------------------------------------------------------

use crate::balloon::{Balloon, BalloonType};
use crate::force;
use crate::gas::{Atmosphere, GasSpecies, GasVolume};

use toml::Value;

#[derive(Debug)]
pub struct StepInput {
    time: f32,
    altitude: f32,
    ascent_rate: f32,
    pressure: f32,
    temperature: f32,
    ballast_mass: f32,
    lift_gas_mass: f32,
}

pub struct SimConfig {
    delta_t: f32,
    dry_mass: f32,
    lift_gas_species: GasSpecies,
    box_area: f32,
    box_drag_coeff: f32,
    balloon_part_id: BalloonType,
    parachute_area: f32,
    parachute_open_alt: f32,
    parachute_drag_coeff: f32,
}

pub fn init(config_path: String) -> (StepInput, SimConfig) {
    // create an initial time step based on the config
    let sim_config = std::fs::read_to_string(config_path)
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();
    let altitude = sim_config["initial_altitude_m"].as_float().unwrap() as f32;
    let atmo = Atmosphere::new(altitude);
    return (
        StepInput {
            time: 0.0,
            altitude: sim_config["initial_altitude_m"].as_float().unwrap() as f32,
            ascent_rate: sim_config["initial_velocity_m_s"].as_float().unwrap() as f32,
            pressure: atmo.pressure(),
            temperature: atmo.temperature(),
            ballast_mass: sim_config["ballast_mass_kg"].as_float().unwrap() as f32,
            lift_gas_mass: sim_config["lift_gas_mass_kg"].as_float().unwrap() as f32,
        },
        SimConfig {
            delta_t: sim_config["time_step_s"].as_float().unwrap() as f32,
            dry_mass: sim_config["dry_mass_kg"].as_float().unwrap() as f32,
            lift_gas_species: GasSpecies::Helium,
            box_area: sim_config["box_area_m2"].as_float().unwrap() as f32,
            box_drag_coeff: sim_config["box_drag_coeff"].as_float().unwrap() as f32,
            balloon_part_id: BalloonType::HAB_2000,
            parachute_area: sim_config["parachute_area_m2"].as_float().unwrap() as f32,
            parachute_open_alt: sim_config["parachute_open_altitude_m"].as_float().unwrap() as f32,
            parachute_drag_coeff: sim_config["parachute_drag_coeff"].as_float().unwrap() as f32,
        },
    );
}

pub fn step(input: StepInput, config: SimConfig) -> StepInput {
    // propagate the closed loop simulation forward by one time step
    let time = input.time + config.delta_t;
    let mut atmo = Atmosphere::new(input.altitude);
    let mut gas = GasVolume::new(config.lift_gas_species, input.lift_gas_mass);
    gas.update_from_ambient(atmo);
    let mut balloon = Balloon::new(config.balloon_part_id, gas);
    let total_dry_mass = config.dry_mass + input.ballast_mass;

    balloon.check_burst_condition();
    if balloon.intact {
        let projected_area = force::sphere_area_from_volume(balloon.lift_gas.volume());
        let c_d = balloon.c_d;
    } else {
        if input.altitude > config.parachute_open_alt {
            let projected_area = config.parachute_drag_coeff;
            let c_d = config.parachute_drag_coeff;
        } else {
            let projected_area = config.box_area;
            let c_d = config.box_drag_coeff;
        }
    }

    let net_force = force::net_force(
        input.altitude,
        input.ascent_rate,
        atmo,
        gas,
        projected_area,
        c_d,
        total_dry_mass,
    );

    let acceleration = net_force / total_dry_mass;
    let ascent_rate = acceleration * config.delta_t;
    let altitude = ascent_rate * config.delta_t;

    atmo.set_altitude(altitude);
    let pressure = atmo.pressure();
    let temperature = atmo.temperature();

    // TODO: implement mass flow
    let ballast_mass = input.ballast_mass;
    let lift_gas_mass = input.lift_gas_mass;

    return StepInput {
        time,
        altitude,
        ascent_rate,
        ballast_mass,
        lift_gas_mass,
        pressure,
        temperature,
    };
}