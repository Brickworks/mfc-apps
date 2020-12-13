use std::{fmt, thread, time};

enum ControlMode {
    Initializing,
    Safe,
    Idle,
    Raising,
    Lowering,
    Dumping,
}

impl fmt::Display for ControlMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       match *self {
           ControlMode::Initializing => write!(f, "Initializing"),
           ControlMode::Safe => write!(f, "Safe"),
           ControlMode::Idle => write!(f, "Idle"),
           ControlMode::Raising => write!(f, "Raising"),
           ControlMode::Lowering => write!(f, "Lowering"),
           ControlMode::Dumping => write!(f, "Dumping"),
       }
    }
}

struct Controller {
    state: ControlMode,
}

struct Valve {
    pwm: f32,
    locked: bool,
}

impl Controller {
    fn init() -> Self {
        Controller { 
            state: ControlMode::Initializing,
        }
    }
    fn transition(&mut self, target_state:ControlMode) {
        println!("Control Mode: {:} --> {:}", self.state, target_state);
        self.state = target_state;
    }
    fn safe(&mut self) {
        self.transition(ControlMode::Safe);
    }
    fn abort(&mut self) {
        self.transition(ControlMode::Dumping);
        println!("(waiting 5 seconds to dump ballast...)");
        thread::sleep(time::Duration::from_secs(5)); // wait 5 seconds
        self.transition(ControlMode::Safe);
    }
}

fn main() {
    let mut controller = Controller::init();
    // test state transitions here
    controller.safe();
    controller.abort();
}
