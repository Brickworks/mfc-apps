use std::fmt;
use std::thread;
use std::time::Duration;

// -- CONSTANTS --
const CTRL_ALTITUDE_FLOOR: f32 = 15000.0; // minimum allowed altitude in meters
const DEFAULT_TARGET_ALTITUDE: f32 = 99999.9; // default target alt in meters

// -- DATA STRUCTURES --
#[derive(Copy, Clone, PartialEq)]
enum ControlMode {
    // State machine control mode
    Init,  // startup, POST, FSW initialization, HW initialization
    Safe,  // not allowed to actuate valves
    Idle,  // allowed to actuate valves but currently nothing to do
    Vent,  // only open descent valve
    Dump,  // only open ascent valve
    Abort, // panic! dump all ballast and lock balloon valve closed
}

struct Valve {
    // Altitude control mass flow control valve
    id: u8,       // integer identifier for this valve
    name: String, // name for this valve - be descriptive but concise!
    pwm: f32,     // instantaneous PWM setting for valve open/close duty cycle
    locked: bool, // whether valve is allowed to change PWM
}

struct ControlMngr {
    // Master altitude control state machine
    state: ControlMode,
    valve_vent: Valve,    // valve to actuate in order to lower altitude
    valve_dump: Valve,    // valve to actuate in order to raise altitude
    target_altitude: f32, // target altitude hold in meters
}

// -- FORMATTING FOR DISPLAY --
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

impl fmt::Display for Valve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:} [valve_id: {:}] @ PWM {:}",
            self.name, self.id, self.pwm
        )
    }
}

fn display_pwm(ctrl_manager: &ControlMngr) {
    println!(
        "balloon pwm {:} ({:}) | ballast pwm {:} ({:})",
        ctrl_manager.valve_vent.get_pwm(), 
        ctrl_manager.valve_vent.print_lock_status(),
        ctrl_manager.valve_dump.get_pwm(),
        ctrl_manager.valve_dump.print_lock_status()
    );
}

// -- METHODS --
impl Valve {
    fn init(valve_id: u8, valve_name: String) -> Self {
        println!("Initializing valve: {:} (id: {:})", valve_name, valve_id);
        Valve {
            id: valve_id,     // integer device identifier
            name: valve_name, // device nickname
            pwm: 0.0,         // PWM setting for open/close duty cycle
            locked: true,     // whether changes to PWM are allowed
        }
    }
    fn is_locked(&self) -> bool {
        // report if the valve is locked or not
        return self.locked;
    }
    fn set_pwm(&mut self, pwm_value: f32) {
        // set valve open/close PWM
        if self.is_locked() == false {
            self.pwm = pwm_value;
        } else {
            println!("Not allowed to set PWM when {:} is locked!", self.name);
        }
    }
    fn get_pwm(&self) -> f32 {
        // report the valve's current PWM setting
        return self.pwm;
    }
    fn lock(&mut self) {
        // lock at the current PWM, no changes allowed
        self.locked = true;
        // println!("Locking {:} [{:}]", &self.name, &self.id);
    }
    fn unlock(&mut self) {
        // unlock, changes to PWM allowed
        self.locked = false;
        // println!("Unlocking {:} [{:}]", &self.name, &self.id);
    }
    fn print_lock_status(&self) -> &str {
        // return a string "locked"/"unlocked" depending on valve's lock status
        if self.is_locked() {
            return "locked"
        } else {
            return "unlocked"
        }
    }
}

impl ControlMngr {
    fn init(valve_vent: Valve, valve_dump: Valve) -> Self {
        println!(
            "Initializing Control Manager with \
             \n> vent valve: {:} \
             \n> dump valve: {:} \
             \n> target alt: {:}",
            valve_vent, valve_dump, DEFAULT_TARGET_ALTITUDE
        );
        return ControlMngr {
            state: ControlMode::Init,
            valve_vent,                               // valve used to lower altitude
            valve_dump,                               // valve used to raise altitude
            target_altitude: DEFAULT_TARGET_ALTITUDE, // bogus target alt
        };
    }
    fn safe(&mut self) {
        // running but not allowed to actuate valves
        let previous_state = self.state;
        // unlock the valves if they are locked to force the next step!
        self.valve_vent.unlock();
        self.valve_dump.unlock();
        // execute state transition
        self.state = ControlMode::Safe;
        println!(
            "CtrlMode transition: {:} --> {:}",
            previous_state, self.state
        );
        // lock the vent valve closed
        self.valve_vent.set_pwm(0.0);
        self.valve_vent.lock();
        // lock the dump valve closed
        self.valve_dump.set_pwm(0.0);
        self.valve_dump.lock();
    }
    fn abort(&mut self) {
        // dump all ballast and lock balloon valve closed
        let previous_state = self.state;
        // unlock the valves if they are locked to force the next step!
        self.valve_vent.unlock();
        self.valve_dump.unlock();
        // execute state transition
        self.state = ControlMode::Abort;
        println!(
            "CtrlMode transition: {:} --> {:}",
            previous_state, self.state
        );
        // lock the vent valve closed
        self.valve_vent.set_pwm(0.0);
        self.valve_vent.lock();
        // lock the dump valve open
        self.valve_dump.set_pwm(1.0);
        self.valve_dump.lock();
    }
    fn idle(&mut self) {
        // enter state for holding altitude, nothing to do
        let previous_state = self.state;
        if previous_state != ControlMode::Safe
            && previous_state != ControlMode::Abort
            && previous_state != ControlMode::Init
        {
            // unlock the valves
            self.valve_vent.unlock();
            self.valve_dump.unlock();
            self.state = ControlMode::Idle;
            println!(
                "CtrlMode transition: {:} --> {:}",
                previous_state, self.state
            );
        } else {
            println!(
                "CtrlMode transition ({:} --> {:}) not allowed! Ignoring...",
                previous_state,
                ControlMode::Idle
            );
        }
    }
    fn vent(&mut self) {
        // enter state used for lowering altitude
        let previous_state = self.state;
        if previous_state == ControlMode::Idle || previous_state == ControlMode::Dump {
            // stop dumping -- can only do one at a time!
            self.valve_dump.set_pwm(0.0);
            self.valve_dump.lock();
            // enable venting
            self.valve_vent.unlock();
            self.state = ControlMode::Vent;
            println!(
                "CtrlMode transition: {:} --> {:}",
                previous_state, self.state
            );
        } else {
            println!(
                "CtrlMode transition ({:} --> {:}) not allowed! Ignoring...",
                previous_state,
                ControlMode::Idle
            );
        }
    }
    fn dump(&mut self) {
        // enter state used for raising altitude
        let previous_state = self.state;
        if previous_state == ControlMode::Idle || previous_state == ControlMode::Vent {
            // stop vemtomg -- can only do one at a time!
            self.valve_vent.set_pwm(0.0);
            self.valve_vent.lock();
            // enable dumping
            self.valve_dump.unlock();
            self.state = ControlMode::Dump;
            println!(
                "CtrlMode transition: {:} --> {:}",
                previous_state, self.state
            );
        } else {
            println!(
                "CtrlMode transition ({:} --> {:}) not allowed! Ignoring...",
                previous_state,
                ControlMode::Idle
            );
        }
    }
    fn set_target(&mut self, target_altitude: f32) {
        // set new target altitude
        // Set a new target altitude to converge toward (in meters)
        if target_altitude > CTRL_ALTITUDE_FLOOR {
            // target must be above the minimum allowed altitude
            self.target_altitude = target_altitude;
            println!("New target altitude: {:}", self.target_altitude);
        } else {
            println!(
                "Not allowed to set target altitude below {:}! Ignoring...",
                CTRL_ALTITUDE_FLOOR
            );
        }
    }
    fn start_control(&mut self) {
        // enable altitude control+transition to idle
        let previous_state = self.state;
        if previous_state == ControlMode::Safe {
            println!("Enabling altitude control system!");
            // unlock the valves if they are locked to force the next step!
            self.valve_vent.unlock();
            self.valve_dump.unlock();
            // execute state transition
            self.state = ControlMode::Idle;
            println!(
                "CtrlMode transition: {:} --> {:}",
                previous_state, self.state
            );
        } else {
            println!(
                "Not allowed to enable control from {:} mode! Ignoring...",
                previous_state
            );
        }
    }
    fn update_pwm(
        &mut self,
        gps_altitude: f32, // instantaneous altitude in meters from GPS
        gps_velocity: f32, // instantaneous vertical velocity in m/s from GPS
        ballast_mass: f32, // ballast mass remining in kg
    ) {
        // execute control algorithm and update vent and dump valve PWM values
        println!("Tick!");
        // check abort criteria
        abort_if_too_low(self, gps_altitude);
        abort_if_out_of_ballast(self, ballast_mass);
        
    }
    fn power_on_self_test(&mut self) {
        // turn on and test devices to look for errors
        println!("Power On Self Test\n> starting POST...");
        thread::sleep(Duration::from_millis(4000)); // hard-coded wait to check
                                                    // when POST is complete, transition from idle to safe
        println!("> POST complete!");
        self.safe()
    }
}

// -- FUNCTIONS --
fn abort_if_too_low(ctrl_manager: &mut ControlMngr, altitude: f32) {
    if altitude <= CTRL_ALTITUDE_FLOOR {
        // abort if the altitude is at or below the allowable limit
        println!(
            "Altitude is too low!\n> Current Alt: {:}m\n> Min allowed: {:}m",
            altitude, CTRL_ALTITUDE_FLOOR
        );
        ctrl_manager.abort()
    }
    // otherwise carry on
}

fn abort_if_out_of_ballast(ctrl_manager: &mut ControlMngr, ballast_mass: f32) {
    if ballast_mass <= 0.0 {
        // abort if there is no ballast left
        println!(
            "Not enough ballast mass!\n> Ballast mass: {:}kg",
            ballast_mass
        );
        ctrl_manager.abort()
    }
    // otherwise carry on
}

// -- MAIN --
fn main() {
    let target_altitude = 25000.0; // meters
    let balloon_valve = Valve::init(0, String::from("BalloonValve"));
    let ballast_valve = Valve::init(1, String::from("BallastValve"));
    let mut ctrl_manager = ControlMngr::init(balloon_valve, ballast_valve);
    // test state transitions here
    ctrl_manager.idle();
    ctrl_manager.power_on_self_test();
    ctrl_manager.set_target(13000.0);
    ctrl_manager.set_target(15000.1);
    ctrl_manager.set_target(target_altitude);
    ctrl_manager.idle();
    ctrl_manager.start_control();
    display_pwm(&ctrl_manager);
    ctrl_manager.vent();
    display_pwm(&ctrl_manager);
    ctrl_manager.valve_vent.set_pwm(0.5);
    display_pwm(&ctrl_manager);
    ctrl_manager.valve_dump.set_pwm(0.5);
    display_pwm(&ctrl_manager);
    ctrl_manager.dump();
    display_pwm(&ctrl_manager);
    ctrl_manager.valve_dump.set_pwm(0.5);
    display_pwm(&ctrl_manager);
    ctrl_manager.update_pwm(14999.9, 0.0, 1.0);
    display_pwm(&ctrl_manager);
    ctrl_manager.vent();
    display_pwm(&ctrl_manager);
    ctrl_manager.safe();
    ctrl_manager.start_control();
    ctrl_manager.update_pwm(15000.1, 0.0, 0.0);
    display_pwm(&ctrl_manager);
}
