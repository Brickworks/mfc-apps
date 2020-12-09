use std::fmt;

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

impl Controller {
    fn init() -> Self {
        Controller { 
            state: ControlMode::Initializing
        }
    }
    fn cycle(&mut self) {
        let next = match self.state {
            ControlMode::Initializing => {
                ControlMode::Safe
            }
            ControlMode::Safe => {
                ControlMode::Idle
            }
            ControlMode::Idle => {
                ControlMode::Raising
            }
            ControlMode::Raising => {
                ControlMode::Lowering
            }
            ControlMode::Lowering => {
                ControlMode::Idle
            }
            ControlMode::Dumping => {
                ControlMode::Safe
            }
        };
        self.state = next;
        println!("{:}", self.state);
    }
    fn safe(&mut self) {
        self.state = ControlMode::Safe;
        println!("{:}", self.state);
    }
    fn abort(&mut self) {
        self.state = ControlMode::Dumping;
        println!("{:}", self.state);
    }
}

fn main() {
    let mut controller = Controller::init();
    // test state transitions here
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.abort();
    controller.cycle();
    controller.cycle();
    controller.safe();
    
}
