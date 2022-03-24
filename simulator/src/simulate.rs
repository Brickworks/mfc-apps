// ----------------------------------------------------------------------------
// Simulate
// --------
// Coordinate execution of a discrete-time simulation.
// ----------------------------------------------------------------------------
use crate::balloon::{Balloon, BalloonType};
use crate::force::{sphere_area_from_volume, net_force, gross_lift, free_lift};
use crate::gas::{Atmosphere, GasSpecies, GasVolume};

use toml::Value;

pub struct SimInstant {
    pub time: f32,
    pub altitude: f32,
    pub ascent_rate: f32,
    pub acceleration: f32,
    pub atmosphere: Atmosphere,
    pub balloon: Balloon,
    pub ballast_mass: f32,
    pub vent_pwm: f32,
    pub dump_pwm: f32,
    pub gross_lift: f32,
    pub free_lift: f32,
    pub atmo_temp: f32,
    pub atmo_pres: f32,
}

pub struct SimConfig {
    pub delta_t: f32,
    pub dry_mass: f32,
    pub lift_gas_species: GasSpecies,
    pub box_area: f32,
    pub box_drag_coeff: f32,
    pub balloon_part_id: BalloonType,
    pub parachute_area: f32,
    pub parachute_open_alt: f32,
    pub parachute_drag_coeff: f32,
    pub vent_mass_flow_rate: f32,
    pub dump_mass_flow_rate: f32,
}

pub fn init(config: &Value) -> (SimInstant, SimConfig) {
    // create an initial time step based on the config
    let balloon_part_id = BalloonType::Hab2000;
    let altitude = config["initial_altitude_m"].as_float().unwrap() as f32;
    let atmo = Atmosphere::new(altitude);
    let gas = GasVolume::new(
        GasSpecies::Helium,
        config["lift_gas_mass_kg"].as_float().unwrap() as f32,
    );
    let balloon = Balloon::new(balloon_part_id, gas);
    let dry_mass = config["dry_mass_kg"].as_float().unwrap() as f32;
    let initial_ballast_mass = config["ballast_mass_kg"].as_float().unwrap() as f32;
    let total_dry_mass = dry_mass + initial_ballast_mass;
    (
        SimInstant {
            time: 0.0,
            altitude: config["initial_altitude_m"].as_float().unwrap() as f32,
            ascent_rate: config["initial_velocity_m_s"].as_float().unwrap() as f32,
            acceleration: 0.0,
            atmosphere: atmo,
            balloon,
            ballast_mass: initial_ballast_mass,
            vent_pwm: 0.0,
            dump_pwm: 0.0,
            gross_lift: gross_lift(atmo, gas),
            free_lift: free_lift(atmo, gas, total_dry_mass),
            atmo_temp: atmo.temperature(),
            atmo_pres: atmo.pressure(),
        },
        SimConfig {
            delta_t: 1.0/config["physics_rate_hz"].as_float().unwrap() as f32,
            dry_mass: dry_mass,
            lift_gas_species: GasSpecies::Helium,
            box_area: config["box_area_m2"].as_float().unwrap() as f32,
            box_drag_coeff: config["box_drag_coeff"].as_float().unwrap() as f32,
            balloon_part_id,
            parachute_area: config["parachute_area_m2"].as_float().unwrap() as f32,
            parachute_open_alt: config["parachute_open_altitude_m"].as_float().unwrap() as f32,
            parachute_drag_coeff: config["parachute_drag_coeff"].as_float().unwrap() as f32,
            vent_mass_flow_rate: config["vent_valve_mass_flow_kg_s"].as_float().unwrap() as f32,
            dump_mass_flow_rate: config["dump_valve_mass_flow_kg_s"].as_float().unwrap() as f32,
        },
    )
}

pub fn step(input: SimInstant, config: &SimConfig) -> SimInstant {
    // propagate the closed loop simulation forward by one time step
    let time = input.time + config.delta_t;
    let mut atmosphere = input.atmosphere;
    let mut balloon = input.balloon;
    balloon.lift_gas.update_from_ambient(atmosphere);

    // mass properties -- pretend to open valves as continuous control
    let ballast_mass =
        (input.ballast_mass - (input.dump_pwm * config.dump_mass_flow_rate)).max(0.0);
    balloon
        .lift_gas
        .set_mass((balloon.lift_gas.mass() - input.vent_pwm * config.vent_mass_flow_rate).max(0.0));
    let total_dry_mass = config.dry_mass + ballast_mass;

    // switch drag conditions
    let projected_area: f32;
    let drag_coeff: f32;
    balloon.check_burst_condition(); // has the balloon popped?
    if balloon.intact {
        // balloon is intact
        projected_area = sphere_area_from_volume(balloon.lift_gas.volume());
        drag_coeff = balloon.drag_coeff;
    } else {
        // balloon has popped
        if input.altitude <= config.parachute_open_alt {
            // parachute open
            projected_area = config.parachute_area;
            drag_coeff = config.parachute_drag_coeff;
        } else {
            // free fall, parachute not open
            projected_area = config.box_area;
            drag_coeff = config.box_drag_coeff;
        }
    }

    // calculate the net force
    let net_force = net_force(
        input.altitude,
        input.ascent_rate,
        atmosphere,
        balloon.lift_gas,
        projected_area,
        drag_coeff,
        total_dry_mass,
    );

    let acceleration = net_force / total_dry_mass;
    let ascent_rate = input.ascent_rate + acceleration * config.delta_t;
    let altitude = input.altitude + ascent_rate * config.delta_t;

    atmosphere.set_altitude(altitude);

    // pass through pwms
    let vent_pwm = input.vent_pwm;
    let dump_pwm = input.dump_pwm;

    // derived outputs
    let gross_lift = gross_lift(atmosphere, balloon.lift_gas);
    let free_lift = free_lift(atmosphere, balloon.lift_gas, total_dry_mass);

    // atmosphere stats
    let atmo_temp = atmosphere.temperature();
    let atmo_pres = atmosphere.pressure();

    SimInstant {
        time,
        altitude,
        ascent_rate,
        acceleration,
        atmosphere,
        balloon,
        ballast_mass,
        vent_pwm,
        dump_pwm,
        gross_lift,
        free_lift,
        atmo_temp,
        atmo_pres,
    }
}
