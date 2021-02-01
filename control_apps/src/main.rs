extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod control_mngr;
mod pid;
mod valve;

fn test_control_mngr() {
    // set bogus values for testing and debugging
    let dt = 0.01; // seconds between datapoints
    let target_altitude = 25000.0; // meters
    let vent_controller = pid::PIDcontroller::init(0.00001, 0.00001, 0.00000, 1.00000);
    let dump_controller = pid::PIDcontroller::init(0.00001, 0.00001, 0.00000, 1.00000);
    let balloon_valve = valve::Valve::init(0, String::from("BalloonValve"), vent_controller);
    let ballast_valve = valve::Valve::init(1, String::from("BallastValve"), dump_controller);
    let mut ctrl_manager = control_mngr::ControlMngr::init(balloon_valve, ballast_valve);

    // test mode transitions and other functions for debugging
    ctrl_manager.power_on_self_test();
    ctrl_manager.set_target(target_altitude);
    ctrl_manager.start_control();
    ctrl_manager.update(target_altitude - 500.0, 1.0, dt);
    ctrl_manager.update(target_altitude, 0.8, dt);
    ctrl_manager.update(target_altitude + 100.0, 0.5, dt);
    ctrl_manager.update(target_altitude, 0.0, dt);
    ctrl_manager.update(14999.9, 1.0, dt);
}

fn main() {
    pretty_env_logger::init(); // initialize pretty print
    test_control_mngr();
}
