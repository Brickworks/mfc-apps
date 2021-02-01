// ----------------------------------------------------------------------------
// Valve
//
// ----------------------------------------------------------------------------

use std::fmt;

pub struct Valve {
    // Altitude control mass flow control valve
    id: u8,       // integer identifier for this valve
    name: String, // name for this valve - be descriptive but concise!
    pwm: f32,     // instantaneous PWM setting for valve open/close duty cycle
    locked: bool, // whether valve is allowed to change PWM
}

impl fmt::Display for Valve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:} [valve_id: {:}]", self.name, self.id)
    }
}

impl Valve {
    pub fn init(valve_id: u8, valve_name: String) -> Self {
        info!("Initializing valve: {:} (id: {:})", valve_name, valve_id);
        Valve {
            id: valve_id,     // integer device identifier
            name: valve_name, // device nickname
            pwm: 0.0,         // PWM setting for open/close duty cycle
            locked: true,     // whether changes to PWM are allowed
        }
    }

    pub fn is_locked(&self) -> bool {
        // report if the valve is locked or not
        return self.locked;
    }

    pub fn set_pwm(&mut self, pwm_value: f32) {
        // set valve open/close PWM
        if self.is_locked() == false {
            self.pwm = pwm_value;
        } else {
            warn!("Not allowed to set PWM when {:} is locked!", self.name);
        }
    }

    pub fn get_pwm(&self) -> f32 {
        // report the valve's current PWM setting
        return self.pwm;
    }

    pub fn lock(&mut self) {
        // lock at the current PWM, no changes allowed
        self.locked = true;
        // println!("Locking {:} [{:}]", &self.name, &self.id);
    }

    pub fn unlock(&mut self) {
        // unlock, changes to PWM allowed
        self.locked = false;
        // println!("Unlocking {:} [{:}]", &self.name, &self.id);
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
