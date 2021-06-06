use crate::simulate;
use crate::simulate::step;
use crate::{SimCommands, SimOutput};
use std::ops::{Deref, Sub};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SendError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use toml;

const PHYSICS_RATE_HZ: f32 = 1000.0;

struct Rate {
    cycle_time: Duration,
    end_of_last_sleep: Option<Instant>,
}

impl Rate {
    fn new(rate_hz: f32) -> Self {
        Self {
            cycle_time: Duration::from_secs_f32(0.0 / rate_hz),
            end_of_last_sleep: None,
        }
    }

    fn sleep(&mut self) {
        let now = Instant::now();

        let sleep_duration = match self.end_of_last_sleep {
            Some(v) => self
                .cycle_time
                .checked_sub(now.checked_duration_since(v).expect(
                    "Rate sleep experienced a last sleep with time ahead of the current time",
                ))
                .expect("Rate sleep detected a blown cycle"),
            None => self.cycle_time,
        };

        std::thread::sleep(sleep_duration);

        self.end_of_last_sleep = Some(Instant::now());
    }
}

pub struct AsyncSim {
    config: toml::Value,
    sim_output: Arc<Mutex<SimOutput>>,
    command_sender: Option<Sender<SimCommands>>,
    /// keep track of
    run_handle: Option<JoinHandle<()>>,
}

impl AsyncSim {
    pub fn new(config: toml::Value) -> Self {
        Self {
            config,
            sim_output: Arc::new(Mutex::new(SimOutput::default())),
            command_sender: None,
            run_handle: None,
        }
    }

    pub fn get_sim_output(&self) -> SimOutput {
        *self.sim_output.lock().unwrap()
    }

    pub fn send_commands(&self, command: SimCommands) {
        self.command_sender.as_ref().unwrap().send(command).unwrap()
    }

    /// Start a thread to run the sim
    pub fn start(&mut self) {
        if self.run_handle.is_some() {
            panic!("Can't start again, sim already ran. Need to stop.")
        }

        let config = self.config.clone();
        let output = self.sim_output.clone();

        let (s, command_receiver) = mpsc::channel();
        self.command_sender = Some(s);

        self.run_handle = Some(std::thread::spawn(move || {
            AsyncSim::run_sim(config, command_receiver, output)
        }));
    }

    fn run_sim(
        config: toml::Value,
        command_channel: Receiver<SimCommands>,
        sim_output: Arc<Mutex<SimOutput>>,
    ) {
        let (mut step_input, step_config) = simulate::init(&config);

        let mut current_vent_flow_percentage = 0.0;
        let mut current_dump_flow_percentage = 0.0;

        let mut rate_sleeper = Rate::new(PHYSICS_RATE_HZ);

        loop {
            rate_sleeper.sleep();

            if let Ok(new_flow_percentages) = command_channel.try_recv() {
                current_vent_flow_percentage = new_flow_percentages.vent_flow_percentage;
                current_dump_flow_percentage = new_flow_percentages.dump_flow_percentage;
            }

            step_input.vent_pwm = current_vent_flow_percentage;
            step_input.dump_pwm = current_dump_flow_percentage;
            step_input = simulate::step(step_input, &step_config);

            // Sync update all the fields
            {
                let mut output = sim_output.lock().unwrap();
                output.altitude = step_input.altitude;
                output.ballast_mass = step_input.ballast_mass;
                output.ascent_rate = step_input.ascent_rate;
                output.time_s = step_input.time;
            }
        }
    }
}
