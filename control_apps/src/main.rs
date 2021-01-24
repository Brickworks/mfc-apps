use std::fmt;

#[derive(Copy, Clone)]
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
            println!("{:} must be unlocked to set PWM!", self.name);
        }
    }

    fn lock(&mut self) {
        self.pwm = 0.0;
        self.locked = true;
        println!("Locked {:}", &self.name);
    }

    fn unlock(&mut self) {
        self.locked = false;
        println!("Unlocked {:}", &self.name);
    }
}

struct Controller {
    state: ControlMode,
    pwm_vent: usize,
    pwm_ballast: usize,
}

impl Controller {
    fn init() -> Self {
        Controller {
            state: ControlMode::Init,
            pwm_vent: 0,
            pwm_ballast: 0,
        }
    }
    fn cycle(&mut self) {
        let next = match self.state {
            ControlMode::Init => {
                ControlMode::Safe
            }
            ControlMode::Safe => {
                ControlMode::Idle
            }
            ControlMode::Idle => {
                ControlMode::Dump
            }
            ControlMode::Dump => {
                ControlMode::Vent
            }
            ControlMode::Vent => {
                ControlMode::Idle
            }
            ControlMode::Abort => {
                ControlMode::Safe
            }
        };
        self.state = next;
        println!("{:} \t| pwm vent: {} \t| pwm ballast: {}", self.state, self.pwm_vent, self.pwm_ballast);
    }
    fn safe(&mut self) {
        self.state = ControlMode::Safe;
        self.pwm_vent = 0;
        self.pwm_ballast = 0;
        println!("{:} \t| pwm vent: {} \t| pwm ballast: {}", self.state, self.pwm_vent, self.pwm_ballast);
    }
    fn abort(&mut self) {
        self.state = ControlMode::Abort;
        self.pwm_vent = 0;
        self.pwm_ballast = 1;
        println!("{:} \t| pwm vent: {} \t| pwm ballast: {}", self.state, self.pwm_vent, self.pwm_ballast);
    }
    fn idle(&mut self) {
        let new_state = match self.state {
            ControlMode::Safe => {
                ControlMode::Idle
            },
            ControlMode::Vent => {
                ControlMode::Idle
            },
            ControlMode::Dump => {
                ControlMode::Idle
            },
            _ => self.state,
        };
        self.state = new_state;
        self.pwm_vent = 0;
        self.pwm_ballast = 0;
        println!("{:} \t| pwm vent: {} \t| pwm ballast: {}", self.state, self.pwm_vent, self.pwm_ballast);
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
    controller.idle();
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.cycle();
    controller.abort();
    controller.cycle();
    controller.cycle();
    controller.safe();
}
