// ----------------------------------------------------------------------------
// ControlMngr
// -----------
// Top level control application state machine and operations coordinator.
// ----------------------------------------------------------------------------

use std::fmt;
use std::thread;
use std::time::Duration;

use crate::valve::Valve;

const POST_DELAY_MS: u64 = 1000; // bogus delay to emulate POST operations
const CTRL_ALTITUDE_FLOOR: f32 = 15000.0; // minimum allowed altitude in meters
const DEFAULT_TARGET_ALTITUDE: f32 = 99999.9; // default target alt in meters
const CTRL_ERROR_DEADZONE: f32 = 100.0; // magnitude of margin to allow

#[derive(Copy, Clone, PartialEq)]
pub enum ControlMode {
    // State machine control mode
    Init,  // startup, POST, FSW initialization, HW initialization
    Safe,  // not allowed to actuate valves
    Idle,  // allowed to actuate valves but currently nothing to do
    Vent,  // only open descent valve
    Dump,  // only open ascent valve
    Abort, // panic! dump all ballast and lock balloon valve closed
}

impl fmt::Display for ControlMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ControlMode::Init => write!(f, "Init"),
            ControlMode::Safe => write!(f, "Safe"),
            ControlMode::Idle => write!(f, "Idle"),
            ControlMode::Dump => write!(f, "Dump"),
            ControlMode::Vent => write!(f, "Vent"),
            ControlMode::Abort => write!(f, "Abort"),
        }
    }
}

pub struct ControlMngr {
    // Master altitude control state machine
    mode: ControlMode,
    valve_vent: Valve,    // valve to actuate in order to lower altitude
    valve_dump: Valve,    // valve to actuate in order to raise altitude
    target_altitude: f32, // target altitude hold in meters
    altitude_error: f32,  // latest known altitude error
    altitude_error_prev: f32, // previous known altitude error
}

impl ControlMngr {
    pub fn new(valve_vent: Valve, valve_dump: Valve) -> Self {
        info!("Initializing Control Manager...");
        info!("\tvent valve {:}", valve_vent);
        info!("\tdump valve {:}", valve_dump);
        info!("\ttarget alt {:}", DEFAULT_TARGET_ALTITUDE);
        return ControlMngr {
            mode: ControlMode::Init,
            valve_vent,
            valve_dump,
            target_altitude: DEFAULT_TARGET_ALTITUDE, // bogus target altitude
            altitude_error: 0.0,                      // bogus altitude error
            altitude_error_prev: 0.0,                 // bogus prev error
        }
    }

    fn safe(&mut self) {
        // running but not allowed to actuate valves
        debug!("Attempting transition to {:}", ControlMode::Safe);
        let previous_mode = self.mode;
        // unlock the valves if they are locked to force the next step!
        self.valve_vent.unlock();
        self.valve_dump.unlock();
        // execute mode transition
        self.mode = ControlMode::Safe;
        warn!("CtrlMode transition: {:} --> {:}", previous_mode, self.mode);
        // lock the vent valve closed
        self.valve_vent.set_pwm(0.0);
        self.valve_vent.lock();
        // lock the dump valve closed
        self.valve_dump.set_pwm(0.0);
        self.valve_dump.lock();
    }

    fn idle(&mut self) {
        // enter mode for holding altitude, nothing to do
        debug!("Attempting transition to {:}", ControlMode::Idle);
        let previous_mode = self.mode;
        if previous_mode != ControlMode::Safe
            && previous_mode != ControlMode::Abort
            && previous_mode != ControlMode::Init
        {
            // unlock the valves
            self.valve_vent.unlock();
            self.valve_dump.unlock();
            self.mode = ControlMode::Idle;
            info!("CtrlMode transition: {:} --> {:}", previous_mode, self.mode);
            self.valve_vent.set_pwm(0.0);
            self.valve_dump.set_pwm(0.0);
        } else {
            warn!(
                "CtrlMode transition ({:} --> {:}) not allowed! Ignoring...",
                previous_mode,
                ControlMode::Idle
            );
        }
    }

    fn vent(&mut self, delta_t: f32) {
        // enter mode used for lowering altitude
        debug!("Attempting transition to {:}", ControlMode::Vent);
        let previous_mode = self.mode;
        if previous_mode == ControlMode::Idle || previous_mode == ControlMode::Dump {
            // stop dumping -- can only do one at a time!
            self.valve_dump.set_pwm(0.0);
            self.valve_dump.lock();
            // enable venting
            self.valve_vent.unlock();
            self.mode = ControlMode::Vent;
            info!("CtrlMode transition: {:} --> {:}", previous_mode, self.mode);
            self.valve_vent.update_pwm(self.altitude_error, self.altitude_error_prev, delta_t);
        } else {
            warn!(
                "CtrlMode transition ({:} --> {:}) not allowed! Ignoring...",
                previous_mode,
                ControlMode::Idle
            );
        }
    }

    fn dump(&mut self, delta_t: f32) {
        // enter mode used for raising altitude
        debug!("Attempting transition to {:}", ControlMode::Dump);
        let previous_mode = self.mode;
        if previous_mode == ControlMode::Idle || previous_mode == ControlMode::Vent {
            // stop vemtomg -- can only do one at a time!
            self.valve_vent.set_pwm(0.0);
            self.valve_vent.lock();
            // enable dumping
            self.valve_dump.unlock();
            self.mode = ControlMode::Dump;
            info!("CtrlMode transition: {:} --> {:}", previous_mode, self.mode);
            self.valve_dump.update_pwm(self.altitude_error, self.altitude_error_prev, delta_t);
        } else {
            warn!(
                "CtrlMode transition ({:} --> {:}) not allowed! Ignoring...",
                previous_mode,
                ControlMode::Idle
            );
        }
    }

    pub fn abort(&mut self) {
        // dump all ballast and lock balloon valve closed
        debug!("Attempting transition to {:}", ControlMode::Abort);
        let previous_mode = self.mode;
        // unlock the valves if they are locked to force the next step!
        self.valve_vent.unlock();
        self.valve_dump.unlock();
        // execute mode transition
        self.mode = ControlMode::Abort;
        warn!("CtrlMode transition: {:} --> {:}", previous_mode, self.mode);
        // lock the vent valve closed
        self.valve_vent.set_pwm(0.0);
        self.valve_vent.lock();
        // lock the dump valve open
        self.valve_dump.set_pwm(1.0);
        self.valve_dump.lock();
    }

    pub fn get_mode(&self) -> ControlMode {
        return self.mode;
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
        // enable altitude control+transition to idle
        let previous_mode = self.get_mode();
        if previous_mode == ControlMode::Safe || previous_mode == ControlMode::Idle {
            info!("Enabling altitude control system!");
            // unlock the valves if they are locked to force the next step!
            self.valve_vent.unlock();
            self.valve_dump.unlock();
            // execute mode transition
            self.mode = ControlMode::Idle;
            info!("CtrlMode transition: {:} --> {:}", previous_mode, self.mode);
        } else {
            warn!(
                "Not allowed to enable control from {:} mode! Ignoring...",
                previous_mode
            );
        }
    }

    pub fn power_on_self_test(&mut self) {
        // turn on and test devices to look for errors
        info!("Starting Power-On Self Test...");
        // hard-coded wait to check
        thread::sleep(Duration::from_millis(POST_DELAY_MS));
        info!("POST complete!");
        // when POST is complete, transition from idle to safe
        self.safe()
    }

    pub fn update(
        &mut self,
        gps_altitude: f32, // instantaneous altitude in meters from GPS
        ballast_mass: f32, // ballast mass remining in kg
        delta_t: f32,      // time since last update
    ) {
        // execute control algorithm and update vent and dump valve PWM values
        self.altitude_error_prev = self.altitude_error;
        self.altitude_error = gps_altitude - self.target_altitude;
        // check abort criteria
        if !self.abort_if_too_low(gps_altitude) && !self.abort_if_out_of_ballast(ballast_mass) {
            // change mode if applicable, run control with gain switching
            if self.altitude_error.abs() >= CTRL_ERROR_DEADZONE {
                debug!(
                    "Altitude error {:}m greater than deadzone threshold {:}m",
                    self.altitude_error, CTRL_ERROR_DEADZONE
                );
                if self.altitude_error > 0.0 {
                    // lower altitude for error to converge to zero
                    debug!(
                        "Altitude {:}m is higher than target {:}m, vent gas",
                        gps_altitude, self.target_altitude
                    );
                    self.vent(delta_t);
                } else {
                    // raise altitude for error to converge to zero
                    debug!(
                        "Altitude {:}m is lower than target {:}m, drop ballast",
                        gps_altitude, self.target_altitude
                    );
                    self.dump(delta_t);
                }
            } else {
                // if in dead zone, do nothing
                debug!(
                    "Altitude error {:}m less than deadzone threshold {:}m, do nothing",
                    self.altitude_error, CTRL_ERROR_DEADZONE
                );
                self.idle();
            }
        }
        // otherwise don't update
    }

    fn abort_if_too_low(&mut self, altitude: f32) -> bool {
        if altitude <= CTRL_ALTITUDE_FLOOR {
            // abort if the altitude is at or below the allowable limit
            warn!(
                "Altitude is too low! Current Alt: {:}m / Min allowed: {:}m",
                altitude, CTRL_ALTITUDE_FLOOR
            );
            self.abort();
            return true
        } else {
            // otherwise carry on
            return false
        }
    }

    fn abort_if_out_of_ballast(&mut self, ballast_mass: f32) -> bool {
        if ballast_mass <= 0.0 {
            // abort if there is no ballast left
            warn!("Not enough ballast mass! Ballast mass: {:}kg", ballast_mass);
            self.abort();
            return true
        } else {
            // otherwise carry on
            return false
        }
    }
}
