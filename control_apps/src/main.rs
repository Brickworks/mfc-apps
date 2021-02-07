extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate pid;

mod control_mngr;
mod valve;

fn test_control_mngr() {
    // set bogus values for testing and debugging
    let dt = 0.01; // seconds between datapoints
    let target_altitude = 25000.0; // meters
    let mut ctrl_manager = control_mngr::ControlMngr::new(
        target_altitude,
        1.0e-5_f32, // vent valve controller proportional gain
        1.0e-5_f32, // vent valve controller integral gain
        1.0e-5_f32, // vent valve controller derivatitve gain
        1.0e-5_f32, // dump valve controller proportional gain
        1.0e-5_f32, // dump valve controller integral gain
        1.0e-5_f32, // dump valve controller derivatitve gain
    );

    // test mode transitions and other functions for debugging
    ctrl_manager.power_on_self_test();
    ctrl_manager.update(target_altitude - 500.0, 1.0);
    ctrl_manager.update(target_altitude - 450.0, 0.8);
    ctrl_manager.update(target_altitude + 100.0, 0.5);
    ctrl_manager.update(target_altitude, 0.2);
    ctrl_manager.update(target_altitude, 0.0);
    ctrl_manager.update(14999.9, 1.0);
}

fn main() {
    pretty_env_logger::init(); // initialize pretty print
    test_control_mngr();
}
