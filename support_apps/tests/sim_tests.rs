use csv;

use std::path::Path;
use std::{fs::File, time::Instant};
use toml::Value;

use control_apps::{
    control_mngr::{ControlCommand, ControlMngr},
    measurement::Measurement,
};
use simulator::simulate;
use simulator::simulate::StepInput;

extern crate pretty_env_logger;

const TIME_INDEX: usize = 0;
const ALTITUDE_INDEX: usize = 5;
const ASCENT_RATE_INDEX: usize = 18;
const BALLAST_MASS_INDEX: usize = 19;
const PRESSURE_INDEX: usize = 9;
const TEMPERATURE_INDEX: usize = 10;

fn read_in_data(path: &std::path::Path) -> Vec<StepInput> {
    println!("{:?}", path);

    let data_str = std::fs::read_to_string(path).unwrap();
    let mut csv_reader = csv::Reader::from_reader(data_str.as_bytes());

    let mut inputs = Vec::new();
    for record in csv_reader.records() {
        let record = record.unwrap();
        let time: f32 = record[TIME_INDEX].parse().unwrap();
        let altitude: f32 = record[ALTITUDE_INDEX].parse().unwrap();
        let ascent_rate: f32 = record[ASCENT_RATE_INDEX].parse().unwrap();
        let ballast_mass: f32 = record[BALLAST_MASS_INDEX].parse().unwrap();
        let pressure: f32 = record[PRESSURE_INDEX].parse().unwrap();
        let temperature: f32 = record[TEMPERATURE_INDEX].parse().unwrap();
        inputs.push(StepInput {
            time: time,
            altitude: altitude,
            ascent_rate: ascent_rate,
            acceleration: 0.0,
            ballast_mass: ballast_mass,
            pressure: pressure,
            temperature: temperature,
            lift_gas_mass: 0.0, // not used
        });
    }

    inputs
}

#[test]
fn test_open_loop() {
    pretty_env_logger::init(); // initialize pretty print
    let csv_path = Path::new("./tests/data/run_ctrl-target-24000-no-mass-flow.csv");
    let inputs = read_in_data(csv_path);

    let config = std::fs::read_to_string("./config/control_config.toml")
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();
    let mut mngr = ControlMngr::new(
        config["target_altitude_m"].as_float().unwrap() as f32,
        config["vent_kp"].as_float().unwrap() as f32,
        config["vent_ki"].as_float().unwrap() as f32,
        config["vent_kd"].as_float().unwrap() as f32,
        config["dump_kp"].as_float().unwrap() as f32,
        config["dump_ki"].as_float().unwrap() as f32,
        config["dump_kd"].as_float().unwrap() as f32,
    );

    let mut writer = csv::Writer::from_path("./out.csv").unwrap();
    writer
        .write_record(&["t", "alt", "ar", "b", "vent", "dump"])
        .unwrap();
    for input in inputs {
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
                input.ballast_mass.to_string(),
                cmd.vent_pwm.to_string(),
                cmd.dump_pwm.to_string(),
            ])
            .unwrap();
        writer.flush().unwrap();
    }
}

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
        .write_record(&["t", "alt", "ar", "ac", "b", "vent", "dump"])
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
    
    // conduct the first input manually to kick things off
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
