// ----------------------------------------------------------------------------
// ControlMngr
// -----------
// Top level control application state machine and operations coordinator.
// ----------------------------------------------------------------------------

use std::fmt;
use std::thread::sleep; // only used to emulate some sort of POST
use std::time::Duration;

use crate::valve::Valve;
use pid::Pid;

const POST_DELAY_MS: u64 = 1000; // bogus delay to emulate POST operations
const CTRL_ALTITUDE_FLOOR: f32 = 15000.0; // minimum allowed altitude in meters
const CTRL_ERROR_DEADZONE: f32 = 100.0; // magnitude of margin to allow
const CTRL_ERROR_READY_THRESHOLD: f32 = 500.0; // basically opposite of deadzone

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Copy, Clone, PartialEq)]
pub enum ControlSubState {
    // Substates to dictate valve PWM constraints
    Safe, // not allowed to actuate valves
    Idle, // not worth actuating, sit tight to save control resources
    Vent, // only open descent valve
    Dump, // only open ascent valve
}

impl fmt::Display for ControlSubState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ControlSubState::Safe => write!(f, "Safe"),
            ControlSubState::Idle => write!(f, "Idle"),
            ControlSubState::Vent => write!(f, "Vent"),
            ControlSubState::Dump => write!(f, "Dump"),
        }
    }
}

pub struct ControlMngr {
    // Master altitude control state machine
    state: ControlState,
    substate: ControlSubState,
    valve_vent: Valve,    // valve to actuate in order to lower altitude
    valve_dump: Valve,    // valve to actuate in order to raise altitude
    target_altitude: f32, // target altitude hold in meters
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
        // define PID error and output limits (-limit <= term <= limit)
        let p_limit = 100.0;
        let i_limit = 100.0;
        let d_limit = 100.0;
        let output_limit = 1.0;
        // initialize PID controllers for each valve
        let vent_controller = Pid::new(
            vent_kp,
            vent_ki,
            vent_kd,
            p_limit,
            i_limit,
            d_limit,
            output_limit,
            target_altitude,
        );
        let dump_controller = Pid::new(
            dump_kp,
            dump_ki,
            dump_kd,
            p_limit,
            i_limit,
            d_limit,
            output_limit,
            target_altitude,
        );
        // initialize each valve with its corresponding controller
        let valve_vent = Valve::new(vent_controller);
        let valve_dump = Valve::new(dump_controller);
        // return a configured control manager
        return ControlMngr {
            state: ControlState::Init,
            substate: ControlSubState::Safe,
            valve_vent: valve_vent,
            valve_dump: valve_dump,
            target_altitude,
        };
    }

    pub fn get_mode(&self) -> ControlState {
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

    pub fn start_control(&mut self) {
        // override enable altitude control+transition to idle
        self.state = ControlState::Stabilize
    }

    pub fn power_on_self_test(&mut self) {
        // turn on and test devices to look for errors
        info!("Starting Power-On Self Test...");
        // hard-coded wait to check
        sleep(Duration::from_millis(POST_DELAY_MS));
        info!("POST complete!");
        // when POST is complete, transition from idle to safe
        self.state = ControlState::Ready
    }

    fn abort_if_out_of_ballast(&mut self, ballast_mass: f32) {
        if ballast_mass <= 0.0 {
            // abort if there is no ballast left
            warn!("Not enough ballast mass! Ballast mass: {:}kg", ballast_mass);
            self.state = ControlState::Abort
        } // otherwise carry on
    }

    pub fn update(
        &mut self,
        altitude: f32,     // instantaneous altitude in meters
        ballast_mass: f32, // ballast mass remining in kg
    ) {
        self.abort_if_out_of_ballast(ballast_mass);
        let error = altitude - self.target_altitude;
        debug!("Altitude: {:} m ({:} m error) | State: {:}[{:}] | {:} kg ballast left",
            altitude, error, self.state, self.substate, ballast_mass
        );
        match self.state {
            ControlState::Init => {
                // initialize the hardware and software
                info!("Inititalizing Control Manager...");
                self.power_on_self_test()
            },
            ControlState::Ready => {
                // lets do this!
                if !(altitude <= CTRL_ALTITUDE_FLOOR) && error.abs() <= CTRL_ERROR_READY_THRESHOLD
                {
                    info!(
                        "{:}m is close enough to target {:}m -- Starting control!",
                        altitude, self.target_altitude
                    );
                    self.state = ControlState::Stabilize;
                    self.substate = ControlSubState::Idle;
                }
                // transition from Ready --> Stabilize within this many meters
                // of the set point
            },
            ControlState::Stabilize => {
                // not sure yet
                if altitude <= CTRL_ALTITUDE_FLOOR {
                    self.state = ControlState::Abort;
                } else {
                    // check which substate to be in
                    if error.abs() >= CTRL_ERROR_DEADZONE {
                        if error > 0.0 {
                            // lower altitude for error to converge to zero
                            debug!(
                                "Altitude {:}m is higher than target {:}m, vent gas",
                                altitude, self.target_altitude
                            );
                            self.substate = ControlSubState::Vent
                        } else {
                            // raise altitude for error to converge to zero
                            debug!(
                                "Altitude {:}m is lower than target {:}m, drop ballast",
                                altitude, self.target_altitude
                            );
                            self.substate = ControlSubState::Dump
                        }
                    } else {
                        // if in dead zone, do nothing
                        debug!(
                            "Altitude error {:}m less than deadzone threshold {:}m, do nothing",
                            error, CTRL_ERROR_DEADZONE
                        );
                        self.substate = ControlSubState::Idle
                    }
                    // do something based on substate
                    match self.substate {
                        ControlSubState::Idle => { // nothing to do, sit tight
                            // close the vent valve
                            self.valve_vent.set_pwm(0.0);
                            // close the dump valve
                            self.valve_dump.set_pwm(0.0);
                        },
                        ControlSubState::Vent => { // get low
                            // close the dump valve
                            self.valve_dump.set_pwm(0.0);
                            // set the vent PWM to whatever the controller says
                            self.valve_vent.update_pwm(altitude)
                        },
                        ControlSubState::Dump => { // get high
                            // close the vent valve
                            self.valve_vent.set_pwm(0.0);
                            // set the vent PWM to whatever the controller says
                            self.valve_dump.update_pwm(altitude)
                        },
                        _ => {},// do nothing
                    }
                }
            },
            ControlState::Safe => {
                // not sure yet
                self.substate = ControlSubState::Safe;
                // close the vent valve
                self.valve_vent.set_pwm(0.0);
                // close the dump valve
                self.valve_dump.set_pwm(0.0);
            },
            ControlState::Abort => {
                // not sure yet
                if ballast_mass <= 0.0 {
                    warn!("Out of ballast mass!");
                    self.state = ControlState::Safe;
                    self.substate = ControlSubState::Safe;
                } else {
                    self.substate = ControlSubState::Dump;
                    // close the vent valve
                    self.valve_vent.set_pwm(0.0);
                    // open the dump valve
                    self.valve_dump.set_pwm(1.0);
                }
            },
        }
    }
}
