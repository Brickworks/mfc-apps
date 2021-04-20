

use std::{fs::File, time::Instant};
use toml::Value;

use control_apps::{
    control_mngr::{ControlCommand, ControlMngr},
    measurement::Measurement,
};
use simulator::{simulate, simulate::StepInput};

extern crate pretty_env_logger;

const MAX_SIM_TIME: f32 = 30_000.0; // max number of seconds for a simulation

#[test]
fn test_closed_loop() {
    pretty_env_logger::init(); // initialize pretty print

    // configure simulation
    let sim_config_toml = std::fs::read_to_string("./config/sim_config.toml")
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();
    let (mut input, sim_config) = simulate::init(sim_config_toml);

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
    let mut writer = init_log_file();

    // now iterate until the altitude hits zero or time is too long
    let mut first_run: bool = true;
    while (input.time < MAX_SIM_TIME) & (input.altitude > 0.0) {
        if first_run {
            // just log the initial condition
            first_run = false;
        } else {
            // propagate the simulation forward by one timestep
            input = simulate::step(input, &sim_config);
        }

        // get commands and telemetry for the current timestep
        let cmd = update_control(&mut mngr, &input);
        input.vent_pwm = cmd.vent_pwm;
        input.dump_pwm = cmd.dump_pwm;
        log_to_file(&input, &mut writer);
    }
}

fn update_control(mngr: &mut ControlMngr, input: &StepInput) -> ControlCommand {
    // pass simulation data to controller as sensor measurements
    let now = Instant::now();
    mngr.update(
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
    )
}

fn init_log_file() -> csv::Writer<File> {
    let mut writer = csv::Writer::from_path("./out.csv").unwrap();
    writer
        .write_record(&[
            "time",
            "altitude_m",
            "ascent_rate_m_s",
            "acceleration_m_s2",
            "lift_gas_mass_kg",
            "ballast_mass_kg",
            "vent_pwm",
            "dump_pwm",
        ])
        .unwrap();
    writer
}

fn log_to_file(input: &StepInput, writer: &mut csv::Writer<File>) {
    writer
        .write_record(&[
            input.time.to_string(),
            input.altitude.to_string(),
            input.ascent_rate.to_string(),
            input.acceleration.to_string(),
            input.balloon.lift_gas.mass().to_string(),
            input.ballast_mass.to_string(),
            input.vent_pwm.to_string(),
            input.dump_pwm.to_string(),
        ])
        .unwrap();
    writer.flush().unwrap();
}
