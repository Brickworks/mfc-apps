// ----------------------------------------------------------------------------
// PID
// ---
// Objects used to define and constrain access to PID controllers.
// ----------------------------------------------------------------------------

use std::fmt;

#[derive(Copy, Clone)]
pub struct PIDcontroller {
    // Data structure for PID control.
    k_p: f32, // proportional error gain
    k_i: f32, // integral error gain
    k_d: f32, // derivative error gain
    k_n: f32, // derivative error filter coefficient
}

impl fmt::Display for PIDcontroller {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Kp {:} | Ki {:} | Kd {:} | N {:}",
            self.k_p, self.k_i, self.k_d, self.k_n
        )
    }
}

impl PIDcontroller {
    pub fn init(k_p: f32, k_i: f32, k_d: f32, k_n: f32) -> Self {
        info!(
            "Initializing PID controller with gains: \
             \n\tKp \t{:} \n\tKi \t{:} \n\tKd \t{:} \n\tN \t{:}",
            k_p, k_i, k_d, k_n
        );
        return PIDcontroller { k_p, k_i, k_d, k_n };
    }

    pub fn set_all_gains(&mut self, k_p: f32, k_i: f32, k_d: f32, k_n: f32) {
        self.k_p = k_p;
        self.k_i = k_i;
        self.k_d = k_d;
        self.k_n = k_n;
    }

    pub fn set_kp(&mut self, kp: f32) {
        // set proportional gain
        self.k_p = kp;
    }

    pub fn get_kp(&mut self) -> f32 {
        // get proportional gain
        return self.k_p;
    }

    pub fn set_ki(&mut self, ki: f32) {
        // set integral gain
        self.k_i = ki;
    }

    pub fn get_ki(&mut self) -> f32 {
        // get integral gain
        return self.k_i;
    }

    pub fn set_kd(&mut self, kd: f32) {
        // set derivative gain
        self.k_d = kd;
    }

    pub fn get_kd(&mut self) -> f32 {
        // get derivative gain
        return self.k_d;
    }

    pub fn set_kn(&mut self, kn: f32) {
        // set derivative filter coefficient
        self.k_n = kn;
    }

    pub fn get_kn(&mut self) -> f32 {
        // get derivative filter coefficient
        return self.k_n;
    }

    pub fn get_control_effort(&mut self, error: f32, last_error: f32, elapsed_time: f32) -> f32 {
        let control_effort = self.get_kp() * error
            + self.get_ki() * error * elapsed_time
            + self.get_kd() * (error - last_error) / elapsed_time;
        debug!("Control effort: {:}", control_effort);
        return control_effort
    }
}
