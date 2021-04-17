use csv;

use std::time::Instant;
use toml::Value;

use control_apps::{
    control_mngr::{ControlCommand, ControlMngr},
    measurement::Measurement,
};
use simulator::simulate;

extern crate pretty_env_logger;

#[test]
fn test_closed_loop() {
    pretty_env_logger::init(); // initialize pretty print

    // configure simulation
    let sim_config = std::fs::read_to_string("./config/sim_config.toml")
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();
    let (mut input, sim_config) = simulate::init(sim_config);

    // configure controller
    let ctrl_config = std::fs::read_to_string("./config/control_config.toml")
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();
    let mut mngr = ControlMngr::new(
        ctrl_config["target_altitude_m"].as_float().unwrap() as f32,
        ctrl_config["vent_kp"].as_float().unwrap() as f32,
        ctrl_config["vent_ki"].as_float().unwrap() as f32,
        ctrl_config["vent_kd"].as_float().unwrap() as f32,
        ctrl_config["dump_kp"].as_float().unwrap() as f32,
        ctrl_config["dump_ki"].as_float().unwrap() as f32,
        ctrl_config["dump_kd"].as_float().unwrap() as f32,
    );

    // set up data logger
    let mut writer = csv::Writer::from_path("./out.csv").unwrap();
    writer
        .write_record(&["t", "alt", "ar", "ac", "b", "vent", "dump", ])
        .unwrap();

    // log the initial condition
    let now = Instant::now();
    let cmd = mngr.update(
        Measurement {
            value: input.altitude,
            timestamp: now,
        },
        Measurement {
            value: input.ascent_rate,
            timestamp: now,
        },
        Measurement {
            value: input.ballast_mass,
            timestamp: now,
        },
    );
    writer
        .write_record(&[
            input.time.to_string(),
            input.altitude.to_string(),
            input.ascent_rate.to_string(),
            input.acceleration.to_string(),
            input.ballast_mass.to_string(),
            cmd.vent_pwm.to_string(),
            cmd.dump_pwm.to_string(),
        ])
        .unwrap();
    writer.flush().unwrap();

    // now iterate until the altitude hits zero or time is too long
    while (input.time < 30_000.0) & (input.altitude > 0.1) {
        // propagate the simulation forward by one timestep
        input = simulate::step(input, sim_config);

        // create commands and telemetry for the current timestep
        let now = Instant::now();
        let cmd = mngr.update(
            Measurement {
                value: input.altitude,
                timestamp: now,
            },
            Measurement {
                value: input.ascent_rate,
                timestamp: now,
            },
            Measurement {
                value: input.ballast_mass,
                timestamp: now,
            },
        );

        input.vent_pwm = cmd.vent_pwm;
        input.dump_pwm = cmd.dump_pwm;

        writer
            .write_record(&[
                input.time.to_string(),
                input.altitude.to_string(),
                input.ascent_rate.to_string(),
                input.acceleration.to_string(),
                input.ballast_mass.to_string(),
                cmd.vent_pwm.to_string(),
                cmd.dump_pwm.to_string(),
            ])
            .unwrap();
        writer.flush().unwrap();
    }
}
