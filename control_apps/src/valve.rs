// ----------------------------------------------------------------------------
// Valve
// -----
// Objects used to define and constrain interaction between the ControlMngr
// and the valves that regulate control reaction mass.
// ----------------------------------------------------------------------------

use std::fmt;

use crate::pid::PIDcontroller;

pub struct Valve {
    // Altitude control mass flow control valve
    id: u8,                        // integer identifier for this valve
    name: String,                  // name for this valve - be descriptive but concise!
    pwm: f32,                      // instantaneous PWM setting [0, 1]
    locked: bool,                  // whether valve is allowed to change PWM
    pub controller: PIDcontroller, // PID used to determine PWM
}

impl fmt::Display for Valve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:} [valve_id: {:}]", self.name, self.id)
    }
}

impl Valve {
    pub fn init(valve_id: u8, valve_name: String, controller: PIDcontroller) -> Self {
        info!("Initializing valve: {:} (id: {:})", valve_name, valve_id);
        Valve {
            id: valve_id,     // integer device identifier
            name: valve_name, // device nickname
            pwm: 0.0,         // PWM setting for open/close duty cycle
            locked: true,     // whether changes to PWM are allowed
            controller,       // controller used to update PWM
        }
    }

    pub fn set_pwm(&mut self, pwm_value: f32) {
        // set valve open/close PWM
        if self.is_locked() == false {
            self.pwm = pwm_value;
        } else {
            warn!("Not allowed to set PWM when {:} is locked!", self.name);
        }
        debug!("{:} PWM set to {:}", self, self.get_pwm());
    }

    pub fn get_pwm(&self) -> f32 {
        // report the valve's current PWM setting
        return self.pwm;
    }

    pub fn set_controller(&mut self, controller: PIDcontroller) {
        // set a new controller used for updating PWM
        self.controller = controller;
    }

    pub fn update_pwm(&mut self, error: f32, last_error: f32, elapsed_time: f32) {
        // execute control algorithm to get control effort
        let control_effort = self
            .controller
            .get_control_effort(error, last_error, elapsed_time);
        // map control effort to PWM values if applicable
        let new_pwm = clamp(control_effort.abs(), 0.0, 1.0);
        // then set PWM
        self.set_pwm(new_pwm);
    }

    pub fn lock(&mut self) {
        // lock at the current PWM, no changes allowed
        self.locked = true;
        debug!("{:} {:}", self, self.print_lock_status());
    }

    pub fn unlock(&mut self) {
        // unlock, changes to PWM allowed
        self.locked = false;
        debug!("{:} {:}", self, self.print_lock_status());
    }

    pub fn is_locked(&self) -> bool {
        // report if the valve is locked or not
        return self.locked;
    }

    pub fn print_lock_status(&self) -> &str {
        // return a string "locked"/"unlocked" depending on valve's lock status
        if self.is_locked() {
            return "locked";
        } else {
            return "unlocked";
        }
    }
}

pub fn clamp(input: f32, min: f32, max: f32) -> f32 {
    assert!(max >= min);
    let mut x = input;
    if x < min {
        x = min;
        debug!("clamping {:} to {:}", input, x);
    }
    if x > max {
        x = max;
        debug!("clamping {:} to {:}", input, x);
    }
    x
}
