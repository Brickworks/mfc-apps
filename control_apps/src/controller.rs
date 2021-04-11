// ----------------------------------------------------------------------------
// Controller
// -----
// Objects used to define and constrain interaction between the ControlMngr
// and the commands that regulate control reaction mass.
// ----------------------------------------------------------------------------

use log::{debug, warn};

use pid::Pid;

pub struct Controller {
    // Altitude control algorithm handler
    controller: Pid<f32>, // PID used to determine PWM
}

impl Controller {
    pub fn new(controller: Pid<f32>) -> Self {
        Controller {
            controller, // controller used to update PWM
        }
    }

    pub fn set_controller(&mut self, controller: Pid<f32>) {
        // set a new controller used for updating PWM
        self.controller = controller
    }

    pub fn set_target(&mut self, new_target: f32) {
        // set the controller setpoint by initializing a new PID with target
        self.controller.setpoint = new_target
    }

    pub fn set_gains(&mut self, kp: f32, ki: f32, kd: f32) {
        // update the controller gains
        self.controller.kp = kp;
        self.controller.ki = ki;
        self.controller.kd = kd;
    }

    pub fn update_control(&mut self, measurement: f32) -> f32 {
        // execute control algorithm to get control effort
        return self.controller.next_control_output(measurement).output;
    }

    pub fn reset_integral(&mut self) {
        // reset the integral term of the controller
        // use with caution!
        self.controller.reset_integral_term();
    }
}

pub struct Valve {
    // Altitude control mass flow control valve
    pwm: f32,      // instantaneous PWM setting [0, 1]
    min_ctrl: f32, // control effort upper limit
    max_ctrl: f32, // control effort upper limit
    pub kp: f32,   // valve controller proportional gain
    pub ki: f32,   // valve controller integral gain
    pub kd: f32,   // valve controller derivatitve gain
}

impl Valve {
    pub fn new(min_ctrl: f32, max_ctrl: f32, kp: f32, ki: f32, kd: f32) -> Self {
        Valve {
            pwm: 0.0, // PWM setting for open/close duty cycle
            min_ctrl, // control effort upper limit
            max_ctrl, // control effort upper limit
            kp,       // valve controller proportional gain
            ki,       // valve controller integral gain
            kd,       // valve controller derivatitve gain
        }
    }

    pub fn set_pwm(&mut self, pwm_value: f32) {
        // set valve open/close PWM
        if pwm_value >= 0.0 && pwm_value <= 1.0 {
            self.pwm = pwm_value
        } else {
            warn!(
                "Clamping PWM {:} to {:}",
                pwm_value,
                clamp(pwm_value, 0.0, 1.0)
            );
            self.pwm = clamp(pwm_value, 0.0, 1.0);
        }
    }

    pub fn get_pwm(&self) -> f32 {
        // report the valve's current PWM setting
        return self.pwm;
    }

    pub fn ctrl2pwm(&self, control_effort: f32) -> f32 {
        // translate control effort to PWM
        let new_pwm = clamp(control_effort, self.min_ctrl, self.max_ctrl).abs();
        debug!("PID effort: {:} | PWM {:}", control_effort, new_pwm);
        return new_pwm;
    }
}

fn clamp(val: f32, min: f32, max: f32) -> f32 {
    let clamped_val: f32;
    if val > max {
        clamped_val = max // clamp to max
    } else if val < min {
        clamped_val = min // clamp to min
    } else {
        clamped_val = val; // otherwise pass through
    }
    return clamped_val;
}
