extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod valve;
use valve::Valve;

mod control_mngr;
use control_mngr::ControlMngr;

// -- TEST --
fn test_control_mngr(ctrl_manager: &mut ControlMngr, target_altitude: f32) {
    // test mode transitions here
    ctrl_manager.power_on_self_test();
    ctrl_manager.set_target(13000.0);
    ctrl_manager.set_target(15000.1);
    ctrl_manager.set_target(target_altitude);
    ctrl_manager.start_control();
    ctrl_manager.print_pwm();
    ctrl_manager.update_pwm(14999.9, 1.0);
    ctrl_manager.print_pwm();
    ctrl_manager.start_control();
    ctrl_manager.update_pwm(target_altitude + 500.0, 1.0);
    ctrl_manager.print_pwm();
    ctrl_manager.update_pwm(target_altitude, 1.0);
    ctrl_manager.print_pwm();
    ctrl_manager.update_pwm(target_altitude - 500.0, 0.5);
    ctrl_manager.print_pwm();
    ctrl_manager.update_pwm(target_altitude, 0.0);
    ctrl_manager.print_pwm();
}

// -- MAIN --
fn main() {
    pretty_env_logger::init(); // initialize pretty print
    let target_altitude = 25000.0; // meters
    let balloon_valve = Valve::init(0, String::from("BalloonValve"));
    let ballast_valve = Valve::init(1, String::from("BallastValve"));
    let vent_gains = control_mngr::PIDgains {
        k_p: 0.00001,
        k_i: 0.00001,
        k_d: 0.00000,
        k_n: 1.00000,
    };
    let dump_gains = control_mngr::PIDgains {
        k_p: 0.00001,
        k_i: 0.00001,
        k_d: 0.00000,
        k_n: 1.00000,
    };
    let mut ctrl_manager = ControlMngr::init(
        balloon_valve, vent_gains, ballast_valve, dump_gains);
    test_control_mngr(&mut ctrl_manager, target_altitude);
}
