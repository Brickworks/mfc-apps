use std::time::Instant;
use std::path::PathBuf;
use toml::Value;
use log::info;

use control_apps::{
    control_mngr::{ControlCommand, ControlMngr},
    measurement::Measurement,
};
use simulator::{async_sim, SimCommands, SimOutput};

pub fn start_sim(ctrl_config: &PathBuf, sim_config: &PathBuf, outfile: &PathBuf) {
    let sim_config_toml = std::fs::read_to_string(sim_config)
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();
    let ctrl_config_toml = std::fs::read_to_string(ctrl_config)
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();

    info!(
        "Setting up sim with the following config: \n{}",
        sim_config_toml
    );
    info!(
        "Setting up altitude controller with following config: \n{}",
        ctrl_config_toml
    );

    // configure simulation
    let mut sim = async_sim::AsyncSim::new(sim_config_toml, outfile.clone());

    // configure controller
    let mut mngr = ControlMngr::new(
        ctrl_config_toml["target_altitude_m"].as_float().unwrap() as f32,
        ctrl_config_toml["vent_kp"].as_float().unwrap() as f32,
        ctrl_config_toml["vent_ki"].as_float().unwrap() as f32,
        ctrl_config_toml["vent_kd"].as_float().unwrap() as f32,
        ctrl_config_toml["dump_kp"].as_float().unwrap() as f32,
        ctrl_config_toml["dump_ki"].as_float().unwrap() as f32,
        ctrl_config_toml["dump_kd"].as_float().unwrap() as f32,
    );
    let mut ctrl_sleeper =
        async_sim::Rate::new(ctrl_config_toml["ctrl_rate_hz"].as_float().unwrap() as f32);
    sim.start();

    
    const MAX_SIM_TIME: f32 = 300.0; // max number of seconds for a simulation
    // now iterate until the altitude hits zero or time is too long
    loop {
        let sim_output = sim.get_sim_output();
        // Run for a certain amount of sim time or to a certain altitude
        if (sim_output.time_s >= MAX_SIM_TIME)
            || (sim_output.altitude <= 0.0 && sim_output.ascent_rate < 0.0)
        {
            break;
        }

        // get commands and telemetry for the current timestep
        let cmd = update_control(&mut mngr, &sim_output);

        sim.send_commands(SimCommands {
            vent_flow_percentage: cmd.vent_pwm,
            dump_flow_percentage: cmd.dump_pwm,
        });

        ctrl_sleeper.sleep();
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