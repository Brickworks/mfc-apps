use log::info;
use std::path::PathBuf;
use std::time::Instant;
use toml::Value;

use control_apps::{
    control_mngr::{ControlCommand, ControlMngr},
    measurement::Measurement,
};
use simulator::{
    async_sim::{self, AsyncSim},
    SimCommands, SimOutput,
};

pub fn start_sim(sim_config: &PathBuf, outfile: &PathBuf) {
    let config = std::fs::read_to_string(sim_config)
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();

    info!("Setting up sim with the following config: \n{}", config);

    // initialize the simulation
    let mut sim = async_sim::AsyncSim::new(config, outfile.clone());
    let mut rate_sleeper = async_sim::Rate::new(1.0);

    // start the sim
    sim.start();
    loop {
        sim.get_sim_output();
        rate_sleeper.sleep();
    }
}

fn start_control(ctrl_config: &PathBuf, sim: &AsyncSim) {
    let config = std::fs::read_to_string(ctrl_config)
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();

    info!(
        "Setting up altitude controller with following config: \n{}",
        config
    );

    // configure controller
    let mut ctrl_sleeper = async_sim::Rate::new(config["ctrl_rate_hz"].as_float().unwrap() as f32);
    let mut mngr = ControlMngr::new(config);

    // now iterate until the altitude hits zero or time is too long
    loop {
        let sim_output = sim.get_sim_output();
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
