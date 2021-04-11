use csv;

use std::{fs::File, time::Instant};
use std::path::Path;
use toml::Value;

use control_apps::{control_mngr::{ControlCommand, ControlMngr}, measurement::Measurement};

const TIME_INDEX: usize = 0;
const ALTITUDE_INDEX: usize = 5; 
const ASCENT_RATE_INDEX: usize = 18;
const BALLAST_MASS_INDEX: usize = 19;

#[derive(Debug)]
struct StepInput {
    time: f32,
    ascent_rate: f32,
    ballast_mass: f32,
    altitude: f32,
}

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
        inputs.push(
            StepInput{
                time: time,
                ascent_rate: ascent_rate,
                ballast_mass: ballast_mass,
                altitude: altitude,
            }
        );
    }

    inputs
}

#[test]
fn test_open_loop() {
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
    writer.write_record(&["t", "alt", "ar", "b", "vent", "dump"]).unwrap();
    for input in inputs {
        let now = Instant::now();

        let cmd = mngr.update(
            Measurement{value: input.altitude, timestamp: now},
            Measurement{value: input.ascent_rate, timestamp: now},
            Measurement{value: input.ballast_mass, timestamp: now},
        );

        writer.write_record(&[
            input.time.to_string(),
            input.altitude.to_string(),
            input.ascent_rate.to_string(),
            input.ballast_mass.to_string(),
            cmd.vent_pwm.to_string(),
            cmd.dump_pwm.to_string(),
        ]).unwrap();
        writer.flush().unwrap();
    }

}
