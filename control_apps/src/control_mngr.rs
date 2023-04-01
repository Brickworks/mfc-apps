// ----------------------------------------------------------------------------
// ControlMngr
// -----------
// Top level control application state machine and operations coordinator.
// ----------------------------------------------------------------------------

use std::fmt;
use std::time::Duration;
use toml::Value;

use bitflags::bitflags;
use log::{debug, info, warn};

use crate::controller::Controller;
use crate::controller::Valve;
use crate::measurement::Measurement;
use pid::Pid;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ControlMode {
    // States to dictate overall control modes
    Init,      // startup, POST, FSW initialization, HW initialization
    Ready,     // not allowed to actuate valves, but waiting for go-ahead
    Stabilize, // actively actuate valves
    Safe,      // not allowed to actuate valves, sit tight
    Abort,     // panic! dump all ballast and lock balloon valve closed
}

impl fmt::Display for ControlMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ControlMode::Init => write!(f, "Init"),
            ControlMode::Ready => write!(f, "Ready"),
            ControlMode::Stabilize => write!(f, "Stabilize"),
            ControlMode::Safe => write!(f, "Safe"),
            ControlMode::Abort => write!(f, "Abort"),
        }
    }
}

bitflags! {
    #[derive(Default)]
    struct ControlStatus: u32 {
        // registers to indicate when conditions are true
        const INACTIVE          = 0b00000000;
        const ACTIVE            = 0b00000001;
        const VENT              = 0b00000010;
        const DUMP              = 0b00000100;
        const STALE_TELEMETRY   = 0b00001000;
        const ALTITUDE_DEADZONE = 0b00010000;
        const SPEED_DEADZONE    = 0b00100000;
        const PROBLEM           = 0b10000000;
    }
}

#[derive(Debug)]
pub struct ControlCommand {
    // Commands to be distributed to the rest of the system as a result from
    // a control input.
    pub vent_pwm: f32,
    pub dump_pwm: f32,
}

pub struct ControlMngr {
    // Master altitude control state machine
    mode: ControlMode,
    status: ControlStatus,
    vent_valve: Valve,      // vent valve object
    dump_valve: Valve,      // dump valve object
    target_altitude: f32,   // target altitude hold in meters
    controller: Controller, // PID controller object
    altitude_floor: f32,    // minimum allowed altitude in meters
    error_deadzone: f32,    // magnitude of margin to allow without actuation
    error_ready: f32,       // basically opposite of deadzone
    speed_deadzone: f32,    // magnitude of margin to allow without actuation
    tlm_max_age: Duration,  // maximum age of telemetry to act on
    min_ballast: f32,       // abort if ballast is less than this in kg
}

impl ControlMngr {
    pub fn new(config: Value) -> Self {
        info!(
            "Setting up altitude controller with following config: \n{}",
            config
        );
        let target_altitude = config["target_altitude_m"]
            .as_float().unwrap() as f32; // desired flight altitude
        let vent_kp = config["vent_kp"]
            .as_float().unwrap() as f32; // vent valve controller proportional gain
        let vent_ki = config["vent_ki"]
            .as_float().unwrap() as f32; // vent valve controller integral gain
        let vent_kd = config["vent_kd"]
            .as_float().unwrap() as f32; // vent valve controller derivatitve gain
        let dump_kp = config["dump_kp"]
            .as_float().unwrap() as f32; // dump valve controller proportional gain
        let dump_ki = config["dump_ki"]
            .as_float().unwrap() as f32; // dump valve controller integral gain
        let dump_kd = config["dump_kd"]
            .as_float().unwrap() as f32; // dump valve controller derivatitve gain
        let altitude_floor = config["altitude_floor_m"]
            .as_float().unwrap() as f32; // minimum allowed altitude in meters
        let error_deadzone = config["error_deadzone_m"]
            .as_float().unwrap() as f32; // magnitude of margin to allow without actuation
        let error_ready = config["error_ready_threshold_m"]
            .as_float().unwrap() as f32; // basically opposite of deadzone
        let speed_deadzone = config["speed_deadzone_m_s"]
            .as_float().unwrap() as f32; // magnitude of margin to allow without actuation
        let tlm_max_age = Duration::from_secs(config["tlm_max_age_s"]
            .as_float().unwrap() as u64); // maximum age of telemetry to act on
        let min_ballast = config["min_ballast_kg"]
            .as_float().unwrap() as f32; // abort if ballast is less than this in kg

        // initialize valve objects
        let vent_valve = Valve::new(-1.0, 0.0, vent_kp, vent_ki, vent_kd, String::from("VENTER"));
        let dump_valve = Valve::new(0.0, 1.0, dump_kp, dump_ki, dump_kd, String::from("DUMPER"));

        // define PID error and output limits (-limit <= term <= limit)
        let p_limit = 1.0;
        let i_limit = 1.0;
        let d_limit = 1.0;
        let output_limit = 1.0;
        // initialize PID controllers beginning with vent gains
        let pid_controller = Pid::new(
            vent_kp,
            vent_ki,
            vent_kd,
            p_limit,
            i_limit,
            d_limit,
            output_limit,
            target_altitude,
        );
        // initialize each valve with its corresponding controller
        let controller = Controller::new(pid_controller);
        // return a configured control manager
        return ControlMngr {
            mode: ControlMode::Init,
            status: ControlStatus::INACTIVE,
            vent_valve,
            dump_valve,
            target_altitude,
            controller,
            altitude_floor,
            error_deadzone,
            error_ready,
            speed_deadzone,
            tlm_max_age,
            min_ballast,
        };
    }

    pub fn get_mode(&self) -> ControlMode {
        return self.mode;
    }

    pub fn set_target(&mut self, target_altitude: f32) {
        // set new target altitude
        // Set a new target altitude to converge toward (in meters)
        if target_altitude > self.altitude_floor {
            // target must be above the minimum allowed altitude
            self.target_altitude = target_altitude;
            info!("New target altitude: {:}m", self.target_altitude);
        } else {
            warn!(
                "Not allowed to set target altitude below {:}! Ignoring...",
                self.altitude_floor
            );
        }
    }

    pub fn power_on_self_test(&mut self) {
        // turn on and test devices to look for errors
        info!("Starting Power-On Self Test...");
        // PLACEHOLDER: hard-coded wait to check.
        // In the future this will wait for real hardware and software checks
        debug!("(the POST is a placeholder for now)");
        info!("POST complete!");
        // when POST is complete, transition from idle to safe
        self.mode = ControlMode::Ready;
    }

    fn abort_if_out_of_ballast(&mut self, ballast_mass: f32) {
        if ballast_mass <= self.min_ballast {
            // abort if there is no ballast left
            warn!(
                "Ballast mass is {:} kg which is below the minimum {} kg --> Abort!",
                ballast_mass, self.min_ballast
            );
            self.mode = ControlMode::Abort;
            self.status.set(ControlStatus::PROBLEM, true);
        } // otherwise carry on
    }

    pub fn update(
        &mut self,
        altitude: Measurement<f32>,     // instantaneous altitude in meters
        ascent_rate: Measurement<f32>,  // instantaneous ascent rate in m/s
        ballast_mass: Measurement<f32>, // ballast mass remining in kg
    ) -> ControlCommand {
        // calculate altitude difference from the target aka altitude error
        let error = altitude.value - self.target_altitude;

        // decide what to do based on what mode the controller is in
        match self.mode {
            ControlMode::Init => {
                // initialize the hardware and software
                info!("Inititalizing Control Manager...");
                self.power_on_self_test();
            }
            ControlMode::Ready => {
                // log status information
                info!(
                    "{}:[{:#?}] Distance from target altitude: {} m",
                    self.mode,
                    self.status,
                    error.abs()
                );
                // lets do this!
                if !(altitude.value <= self.altitude_floor) && error.abs() <= self.error_ready {
                    info!(
                        "{} m is close enough to target {} m --> Stabilize!",
                        altitude.value, self.target_altitude
                    );
                    self.mode = ControlMode::Stabilize;
                    self.status.set(ControlStatus::ACTIVE, true);
                    // reset the integral to avoid accumulated error
                    self.controller.reset_integral();
                }
            }
            ControlMode::Stabilize => {
                // abort if there's no ballast left, doesn't matter if tlm is stale
                self.abort_if_out_of_ballast(ballast_mass.value);
                // abort if there's a problem
                if self.status.intersects(ControlStatus::PROBLEM) {
                    self.mode = ControlMode::Abort;
                    // run mission abort procedure immediately
                    return self.update(altitude, ascent_rate, ballast_mass);
                }
                // switch gains depending on ascent rate
                if ascent_rate.value > 0.0 {
                    // if rising, vent
                    self.controller.set_gains(
                        self.vent_valve.kp,
                        self.vent_valve.ki,
                        self.vent_valve.kd,
                    );
                    self.status.set(ControlStatus::VENT, true);
                    self.status.set(ControlStatus::DUMP, false);
                } else {
                    // otherwise, dump
                    self.controller.set_gains(
                        self.dump_valve.kp,
                        self.dump_valve.ki,
                        self.dump_valve.kd,
                    );
                    self.status.set(ControlStatus::VENT, false);
                    self.status.set(ControlStatus::DUMP, true);
                };
                // always update the controller even if no action is taken
                let control_effort = self.controller.update_control(altitude.value);
                // decide what to do in order to converge toward the target
                if altitude.value > self.altitude_floor {
                    // determine what the PWM should be for each valve
                    let vent_pwm = self.vent_valve.ctrl2pwm(control_effort);
                    let dump_pwm = self.dump_valve.ctrl2pwm(control_effort);

                    // configure registers for reasons to not actuate
                    if is_stale(&altitude, self.tlm_max_age)
                        | is_stale(&ascent_rate, self.tlm_max_age)
                    {
                        // altitude telemetry is stale
                        warn!(
                            "Altitude telemetry is stale! ({:#?} s old)",
                            &altitude.timestamp.elapsed()
                        );
                        self.status.set(ControlStatus::STALE_TELEMETRY, true)
                    } else {
                        self.status.set(ControlStatus::STALE_TELEMETRY, false)
                    }

                    if error.abs() < self.error_deadzone {
                        // altitude error is within the deadzone
                        debug!(
                            "Altitude error is {}, which is within the deadzone of {}",
                            error.abs(),
                            self.error_deadzone
                        );
                        // reset the integral to avoid accumulated error
                        self.controller.reset_integral();
                        self.status.set(ControlStatus::ALTITUDE_DEADZONE, true)
                    } else {
                        self.status.set(ControlStatus::ALTITUDE_DEADZONE, false)
                    }

                    if ascent_rate.value.abs() < self.speed_deadzone {
                        // ascent rate is within the deadzone
                        debug!(
                            "Ascent Rate is {}, which is within the deadzone of {}",
                            ascent_rate.value.abs(),
                            self.speed_deadzone
                        );
                        self.status.set(ControlStatus::SPEED_DEADZONE, true)
                    } else {
                        self.status.set(ControlStatus::SPEED_DEADZONE, false)
                    }

                    // decide if/how to actuate valves
                    let telem_ok = !self.status.intersects(ControlStatus::STALE_TELEMETRY);
                    let control_ok = telem_ok
                        & !(self.status.intersects(ControlStatus::ALTITUDE_DEADZONE)
                            & self.status.intersects(ControlStatus::SPEED_DEADZONE));

                    if control_ok & (ascent_rate.value > 0.0) {
                        // lower altitude for error to converge to zero
                        // close the dump valve, set the vent PWM
                        self.status.set(ControlStatus::VENT, true);
                        self.status.set(ControlStatus::DUMP, false);
                    } else if control_ok & (ascent_rate.value < 0.0) {
                        // raise altitude for error to converge to zero
                        // close the vent valve, set the dump PWM
                        self.status.set(ControlStatus::VENT, false);
                        self.status.set(ControlStatus::DUMP, true);
                    } else {
                        // if in dead zone or telemetry is stale, do nothing
                        // close the vent valve and close the dump valve
                        self.status.set(ControlStatus::VENT, false);
                        self.status.set(ControlStatus::DUMP, false);
                    };
                    // actuate the valves
                    if self.status.intersects(ControlStatus::VENT) {
                        self.vent_valve.set_pwm(vent_pwm)
                    } else {
                        self.vent_valve.set_pwm(0.0)
                    }
                    if self.status.intersects(ControlStatus::DUMP) {
                        self.dump_valve.set_pwm(dump_pwm)
                    } else {
                        self.dump_valve.set_pwm(0.0)
                    }
                    info!("{}:[{:#?}]", self.mode, self.status,);
                } else {
                    // abort if altitude is lower than the lowest allowed value
                    warn!(
                        "{} m lower than minimum {} m --> Abort!",
                        altitude.value, self.altitude_floor
                    );
                    self.mode = ControlMode::Abort;
                }
            }
            ControlMode::Safe => {
                // close the valves and sit tight
                // close the vent valve
                self.vent_valve.set_pwm(0.0);
                // close the dump valve
                self.dump_valve.set_pwm(0.0);
                self.status.set(ControlStatus::ACTIVE, false);
                self.status.set(ControlStatus::VENT, false);
                self.status.set(ControlStatus::DUMP, false);
            }
            ControlMode::Abort => {
                self.status.set(ControlStatus::PROBLEM, true);
                // keep the balloon valve closed and dump all ballast
                if ballast_mass.value <= 0.0 {
                    warn!("Out of ballast mass!");
                    self.mode = ControlMode::Safe;
                } else {
                    // close the vent valve
                    self.vent_valve.set_pwm(0.0);
                    // open the dump valve
                    self.dump_valve.set_pwm(1.0);
                    self.status.set(ControlStatus::VENT, false);
                    self.status.set(ControlStatus::DUMP, true);
                }
            }
        }

        return ControlCommand {
            vent_pwm: self.vent_valve.get_pwm(),
            dump_pwm: self.dump_valve.get_pwm(),
        };
    }
}

fn is_stale(telemetry: &Measurement<f32>, max_age: Duration) -> bool {
    return telemetry.timestamp.elapsed() > max_age;
}
