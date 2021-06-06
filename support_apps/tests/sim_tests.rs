use std::{fs::File, time::Instant};
use toml::Value;

use control_apps::{
    control_mngr::{ControlCommand, ControlMngr},
    measurement::Measurement,
};
use simulator::{async_sim, simulate, simulate::StepInput, SimOutput, SimCommands};

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
    let mut sim = async_sim::AsyncSim::new(sim_config_toml);

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

    sim.start();

    // now iterate until the altitude hits zero or time is too long
    loop {
        let sim_output = sim.get_sim_output();

        // Run for a certain amount of sim time or to a certain altitude
        if (sim_output.time_s >= MAX_SIM_TIME) || (sim_output.altitude <= 0.0) {
            break
        }

        // get commands and telemetry for the current timestep
        let cmd = update_control(&mut mngr, &sim_output);

        sim.send_commands(SimCommands{
            vent_flow_percentage: cmd.vent_pwm,
            dump_flow_percentage: cmd.dump_pwm,
        });

        log_to_file(&sim_output, &mut writer);
    }
}

fn update_control(mngr: &mut ControlMngr, input: &SimOutput) -> ControlCommand {
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
            "ballast_mass_kg",
            // "acceleration_m_s2",
            // "lift_gas_mass_kg",
            // "vent_pwm",
            // "dump_pwm",
        ])
        .unwrap();
    writer
}

fn log_to_file(sim_output: &SimOutput, writer: &mut csv::Writer<File>) {
    writer
        .write_record(&[
            sim_output.time_s.to_string(),
            sim_output.altitude.to_string(),
            sim_output.ascent_rate.to_string(),
            sim_output.ballast_mass.to_string(),
            //input.acceleration.to_string(),
            //input.balloon.lift_gas.mass().to_string(),
            //input.vent_pwm.to_string(),
            //input.dump_pwm.to_string(),
        ])
        .unwrap();
    writer.flush().unwrap();
}
