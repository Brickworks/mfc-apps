// ----------------------------------------------------------------------------
// ControlMngr
// -----------
// Top level control application state machine and operations coordinator.
// ----------------------------------------------------------------------------

use std::fmt;
use std::thread::sleep; // only used to emulate some sort of POST
use std::time::Duration;

use log::{debug, info, warn};

use crate::measurement::Measurement;
use crate::controller::Controller;
use crate::controller::Valve;
use pid::Pid;

const POST_DELAY_MS: u64 = 1000; // bogus delay to emulate POST operations
const CTRL_ALTITUDE_FLOOR: f32 = 15000.0; // minimum allowed altitude in meters
const CTRL_ERROR_DEADZONE: f32 = 100.0; // magnitude of margin to allow without actuation
const CTRL_ERROR_READY_THRESHOLD: f32 = 1000.0; // basically opposite of deadzone
const CTRL_SPEED_DEADZONE: f32 = 0.2; // magnitude of margin to allow without actuation
const CTRL_TLM_MAX_AGE: Duration = Duration::from_secs(2); // maximum age of telemetry to act on
const CTRL_MIN_BALLAST: f32 = 0.01; // abort if ballast is less than this in kg

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ControlState {
    // States to dictate overall control states
    Init,      // startup, POST, FSW initialization, HW initialization
    Ready,     // not allowed to actuate valves, but waiting for go-ahead
    Stabilize, // actively actuate valves
    Safe,      // not allowed to actuate valves, sit tight
    Abort,     // panic! dump all ballast and lock balloon valve closed
}

impl fmt::Display for ControlState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ControlState::Init => write!(f, "Init"),
            ControlState::Ready => write!(f, "Ready"),
            ControlState::Stabilize => write!(f, "Stabilize"),
            ControlState::Safe => write!(f, "Safe"),
            ControlState::Abort => write!(f, "Abort"),
        }
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
    state: ControlState,
    vent_valve: Valve,      // vent valve object
    dump_valve: Valve,      // dump valve object
    target_altitude: f32,   // target altitude hold in meters
    controller: Controller, // PID controller object
}

impl ControlMngr {
    pub fn new(
        target_altitude: f32,
        vent_kp: f32, // vent valve controller proportional gain
        vent_ki: f32, // vent valve controller integral gain
        vent_kd: f32, // vent valve controller derivatitve gain
        dump_kp: f32, // dump valve controller proportional gain
        dump_ki: f32, // dump valve controller integral gain
        dump_kd: f32, // dump valve controller derivatitve gain
    ) -> Self {
        // initialize valve objects
        let vent_valve = Valve::new(-1.0, 0.0, vent_kp, vent_ki, vent_kd);
        let dump_valve = Valve::new(0.0, 1.0, dump_kp, dump_ki, dump_kd);

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
            state: ControlState::Init,
            vent_valve,
            dump_valve,
            target_altitude,
            controller,
        };
    }

    pub fn get_state(&self) -> ControlState {
        return self.state;
    }

    pub fn set_target(&mut self, target_altitude: f32) {
        // set new target altitude
        // Set a new target altitude to converge toward (in meters)
        if target_altitude > CTRL_ALTITUDE_FLOOR {
            // target must be above the minimum allowed altitude
            self.target_altitude = target_altitude;
            info!("New target altitude: {:}m", self.target_altitude);
        } else {
            warn!(
                "Not allowed to set target altitude below {:}! Ignoring...",
                CTRL_ALTITUDE_FLOOR
            );
        }
    }

    pub fn power_on_self_test(&mut self) {
        // turn on and test devices to look for errors
        info!("Starting Power-On Self Test...");
        // PLACEHOLDER: hard-coded wait to check.
        // In the future this will wait for real hardware and software checks
        sleep(Duration::from_millis(POST_DELAY_MS));
        info!("POST complete!");
        // when POST is complete, transition from idle to safe
        self.state = ControlState::Ready
    }

    fn abort_if_out_of_ballast(&mut self, ballast_mass: f32) {
        if ballast_mass <= CTRL_MIN_BALLAST {
            // abort if there is no ballast left
            warn!("Not enough ballast mass! Ballast mass: {:}kg", ballast_mass);
            self.state = ControlState::Abort
        } // otherwise carry on
    }

    fn allow_control(&self, ctrl_inhibitor_mask: u8) -> bool {
        // check bitmask of control inhibitor conditions
        // - not allowed to control if in both deadzones
        // - of if telemetry is stale
        // bitmask key:
        // 1    ascent rate telemetry is stale
        // 2    altitude telemetry is stale
        // 3    within ascent rate deadzone
        // 4    within altitude deadzone
        // allow: 0000, 1000, 0100
        // TODO: refactor this to use bitflags
        let control_allowed: bool = match ctrl_inhibitor_mask {
            0b0000_0000 => true,
            0b0000_1000 => true,
            0b0000_0100 => true,
            _ => false,
        };
        return control_allowed
    }

    pub fn update(
        &mut self,
        altitude: Measurement<f32>,     // instantaneous altitude in meters
        ascent_rate: Measurement<f32>,  // instantaneous ascent rate in m/s
        ballast_mass: Measurement<f32>, // ballast mass remining in kg
    ) -> ControlCommand {
        // log status information
        info!(
            "[{:}] Altitude {:} m ({:}), Ballast Remaining {:} kg ({:})",
            self.state,
            altitude.value,
            altitude.timestamp.elapsed().as_secs(),
            ballast_mass.value,
            ballast_mass.timestamp.elapsed().as_secs()
        );

        // calculate altitude difference from the target aka altitude error
        let error = altitude.value - self.target_altitude;

        // decide what to do based on what state the controller is in
        match self.state {
            ControlState::Init => {
                // initialize the hardware and software
                info!("Inititalizing Control Manager...");
                self.power_on_self_test()
            }
            ControlState::Ready => {
                // lets do this!
                if !(altitude.value <= CTRL_ALTITUDE_FLOOR)
                    && error.abs() <= CTRL_ERROR_READY_THRESHOLD
                {
                    info!(
                        "{:}m is close enough to target {:}m --> Stabilize!",
                        altitude.value, self.target_altitude
                    );
                    self.state = ControlState::Stabilize;

                    // reset the integral to avoid accumulated error
                    self.controller.reset_integral();
                }
            }
            ControlState::Stabilize => {
                
                // abort if there's no ballast left, doesn't matter if tlm is stale
                self.abort_if_out_of_ballast(ballast_mass.value);
                
                // switch gains depending on ascent rate
                if ascent_rate.value > 0.0 {
                    // switch to vent gains
                    self.controller.set_gains(
                        self.vent_valve.kp,
                        self.vent_valve.ki,
                        self.vent_valve.kd
                    );
                } else {
                    // switch to dump gains
                    self.controller.set_gains(
                        self.dump_valve.kp,
                        self.dump_valve.ki,
                        self.dump_valve.kd
                    );
                };
                // always update the controller even if no action is taken
                let control_effort = self.controller.update_control(
                    altitude.value);
                // decide what to do in order to converge toward the target
                if altitude.value > CTRL_ALTITUDE_FLOOR {
                    // configure registers for reasons to not actuate
                    let mut ctrl_inhibitor_mask: u8 = 0b0000_0000;
                    if error.abs() < CTRL_ERROR_DEADZONE {
                        // altitude error is within the deadzone
                        ctrl_inhibitor_mask ^= 0b0000_1000;
                    }
                    if ascent_rate.value.abs() < CTRL_SPEED_DEADZONE {
                        // ascent rate is within the deadzone
                        ctrl_inhibitor_mask ^= 0b0000_0100;
                    }
                    if is_stale(&altitude, CTRL_TLM_MAX_AGE) {
                        // altitude telemetry is stale
                        ctrl_inhibitor_mask ^= 0b0000_0010;
                    }
                    if is_stale(&ascent_rate, CTRL_TLM_MAX_AGE) {
                        // ascent rate telemetry is stale
                        ctrl_inhibitor_mask ^= 0b0000_0001;
                    }

                    // decide if/how to actuate valves
                    if self.allow_control(ctrl_inhibitor_mask) {
                        if ascent_rate.value > 0.0 {
                            // lower altitude for error to converge to zero
                            // set the vent PWM to whatever the controller says
                            self.vent_valve.set_pwm(
                                self.vent_valve.ctrl2pwm(control_effort)
                            );
                            // close the dump valve
                            self.dump_valve.set_pwm(0.0);
                            debug!(
                                "[{:}] Venting at {:}%",
                                self.state,
                                self.vent_valve.get_pwm() * 100.0
                            );
                        } else {
                            // raise altitude for error to converge to zero
                            // close the vent valve
                            self.vent_valve.set_pwm(0.0);
                            // set the vent PWM to whatever the controller says
                            self.dump_valve.set_pwm(
                                self.dump_valve.ctrl2pwm(control_effort)
                            );
                            debug!(
                                "[{:}] Dumping at {:}%",
                                self.state,
                                self.dump_valve.get_pwm() * 100.0
                            );
                        };
                    } else {
                        // if in dead zone or telemetry is stale, do nothing
                        // close the vent valve
                        self.vent_valve.set_pwm(0.0);
                        // close the dump valve
                        self.dump_valve.set_pwm(0.0);
                        debug!(
                            "[{:}] Controller idle. Reason: {:#04b}",
                            self.state,
                            ctrl_inhibitor_mask
                        );
                        // reset the controller to avoid accumulated error
                        self.controller.reset_integral();
                    };
                } else {
                    // abort if altitude is lower than the lowest allowed value
                    warn!(
                        "{:}m lower than minimum {:}m --> Abort!",
                        altitude.value, CTRL_ALTITUDE_FLOOR
                    );
                    self.state = ControlState::Abort;
                }
            }
            ControlState::Safe => {
                // close the valves and sit tight
                // close the vent valve
                self.vent_valve.set_pwm(0.0);
                // close the dump valve
                self.dump_valve.set_pwm(0.0);
            }
            ControlState::Abort => {
                // keep the balloon valve closed and dump all ballast
                if ballast_mass.value <= 0.0 {
                    warn!("Out of ballast mass!");
                    self.state = ControlState::Safe;
                } else {
                    // close the vent valve
                    self.vent_valve.set_pwm(0.0);
                    // open the dump valve
                    self.dump_valve.set_pwm(1.0);
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
