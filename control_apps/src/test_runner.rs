extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate pid;

mod control_mngr;
mod valve;

fn test_control_mngr() {
    // set bogus values for testing and debugging
    let target_altitude = 30000.0; // meters
    let mut pwms: (f32, f32);
    let mut ctrl_manager = control_mngr::ControlMngr::new(
        24000.0, // some random target altitude
        1.0e-5_f32, // vent valve controller proportional gain
        0.0e+1_f32, // vent valve controller integral gain
        1.0e-3_f32, // vent valve controller derivatitve gain
        1.0e-8_f32, // dump valve controller proportional gain
        1.0e-5_f32, // dump valve controller integral gain
        1.0e-3_f32, // dump valve controller derivatitve gain
    );
    ctrl_manager.set_target(target_altitude); // test setting target
    // test mode transitions and other functions for debugging
    ctrl_manager.power_on_self_test();
    pwms = ctrl_manager.update(target_altitude - 500.0, 2.0, 1.0);
    pwms = ctrl_manager.update(target_altitude - 450.0, 2.0, 0.8);
    pwms = ctrl_manager.update(target_altitude + 100.0, 2.0, 0.5);
    pwms = ctrl_manager.update(target_altitude, 2.0, 0.2);
    pwms = ctrl_manager.update(target_altitude, 2.0, 0.0);
    pwms = ctrl_manager.update(14999.9, 2.0, 1.0);
}

fn main() {
    pretty_env_logger::init(); // initialize pretty print
    test_control_mngr();
}
