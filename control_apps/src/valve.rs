// ----------------------------------------------------------------------------
// Valve
// -----
// Objects used to define and constrain interaction between the ControlMngr
// and the valves that regulate control reaction mass.
// ----------------------------------------------------------------------------

use pid::Pid;

pub struct Valve {
    // Altitude control mass flow control valve
    pwm: f32,             // instantaneous PWM setting [0, 1]
    controller: Pid<f32>, // PID used to determine PWM
}

impl Valve {
    pub fn new(controller: Pid<f32>) -> Self {
        Valve {
            pwm: 0.0,         // PWM setting for open/close duty cycle
            controller,       // controller used to update PWM
        }
    }

    pub fn set_pwm(&mut self, pwm_value: f32) {
        // set valve open/close PWM
        if pwm_value >= 0.0 && pwm_value <= 1.0 {
            self.pwm = pwm_value
        } else {
            error!("PWM {:} is not allowed! Must be [0, 1]", pwm_value)
        }
    }

    pub fn get_pwm(&self) -> f32 {
        // report the valve's current PWM setting
        return self.pwm
    }

    pub fn set_controller(&mut self, controller: Pid<f32>) {
        // set a new controller used for updating PWM
        self.controller = controller
    }

    pub fn set_target(&mut self, new_target: f32) {
        // set the controller setpoint by initializing a new PID with target
        self.controller.setpoint = new_target
    }

    pub fn update_pwm(&mut self, measurement: f32) {
        // execute control algorithm to get control effort as PWM
        let control_effort = self.controller.next_control_output(measurement);
        // translate control effort to PWM
        let mut new_pwm = control_effort.output.abs(); // WIP
        if new_pwm > 1.0 {
            new_pwm = 1.0 // clamp max to 1
        } else if new_pwm < 0.0 {
            new_pwm = 0.0 // clamp min to 0
        }
        debug!("PID effort: {:} | PWM {:}", control_effort.output, new_pwm);
        self.set_pwm(new_pwm)
    }
}
