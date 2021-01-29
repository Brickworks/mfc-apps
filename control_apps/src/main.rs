use std::fmt;

#[derive(Copy, Clone, PartialEq)]
enum ControlMode {
    Init,
    Safe,
    Idle,
    Vent,
    Dump,
    Abort,
}

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

struct Valve {
    id: u8,
    name: String,
    pwm: f32,
    locked: bool,
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

impl Valve {
    fn init(valve_id: u8, valve_name: String) -> Self {
        println!("Initializing valve: {:} (id: {:})", valve_name, valve_id);
        Valve {
            id: valve_id,
            name: valve_name,
            pwm: 0.0,
            locked: true,
        }
    }
    fn is_locked(&self) -> bool {
        return self.locked;
    }
    fn set_pwm(&mut self, pwm_value: f32) {
        if self.is_locked() == false {
            self.pwm = pwm_value;
        } else {
            println!("Cannot set PWM because {:} is locked!", self.name);
        }
    }
    fn get_pwm(&self) -> f32 {
        return self.pwm;
    }
    fn lock(&mut self) {
        self.locked = true;
        // println!("Locking {:} [{:}]", &self.name, &self.id);
    }
    fn unlock(&mut self) {
        self.locked = false;
        // println!("Unlocking {:} [{:}]", &self.name, &self.id);
    }
}

struct ControlMngr {
    state: ControlMode,
    valve_vent: Valve,
    valve_dump: Valve,
}

impl ControlMngr {
    fn init(valve_vent: Valve, valve_dump: Valve) -> Self {
        ControlMngr {
            state: ControlMode::Init,
            valve_vent,
            valve_dump,
        }
    }
    fn safe(&mut self) {
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
        let previous_state = self.state;
        if previous_state != ControlMode::Safe && previous_state != ControlMode::Abort {
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
    fn start_control(&mut self) {
        let previous_state = self.state;
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
    }
    fn update_pwm(&mut self) {
        
    }
}

fn main() {
    let balloon_valve = Valve::init(0, String::from("BalloonValve"));
    let ballast_valve = Valve::init(1, String::from("BallastValve"));
    let mut ctrl_manager = ControlMngr::init(balloon_valve, ballast_valve);

    // test state transitions here
    ctrl_manager.idle();
    ctrl_manager.safe();
    ctrl_manager.idle();
    ctrl_manager.start_control();
    println!(
        "balloon pwm {:} | ballast pwm {:}",
        ctrl_manager.valve_vent.get_pwm(),
        ctrl_manager.valve_dump.get_pwm()
    );
    ctrl_manager.vent();
    println!(
        "balloon pwm {:} | ballast pwm {:}",
        ctrl_manager.valve_vent.get_pwm(),
        ctrl_manager.valve_dump.get_pwm()
    );
    ctrl_manager.valve_vent.set_pwm(0.5);
    println!(
        "balloon pwm {:} | ballast pwm {:}",
        ctrl_manager.valve_vent.get_pwm(),
        ctrl_manager.valve_dump.get_pwm()
    );
    ctrl_manager.valve_dump.set_pwm(0.5);
    println!(
        "balloon pwm {:} | ballast pwm {:}",
        ctrl_manager.valve_vent.get_pwm(),
        ctrl_manager.valve_dump.get_pwm()
    );
    ctrl_manager.dump();
    println!(
        "balloon pwm {:} | ballast pwm {:}",
        ctrl_manager.valve_vent.get_pwm(),
        ctrl_manager.valve_dump.get_pwm()
    );
    ctrl_manager.valve_dump.set_pwm(0.5);
    println!(
        "balloon pwm {:} | ballast pwm {:}",
        ctrl_manager.valve_vent.get_pwm(),
        ctrl_manager.valve_dump.get_pwm()
    );
    ctrl_manager.abort();
    println!(
        "balloon pwm {:} | ballast pwm {:}",
        ctrl_manager.valve_vent.get_pwm(),
        ctrl_manager.valve_dump.get_pwm()
    );
    ctrl_manager.vent();
}
