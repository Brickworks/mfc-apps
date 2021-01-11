use std::{fmt, thread, time};

<<<<<<< HEAD
#[derive(Eq, PartialEq)]
=======
#[derive(Copy, Clone)]
>>>>>>> CTRL: state machine first try
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
<<<<<<< HEAD
        Controller { 
            state: ControlMode::Initializing,
        }
    }
    fn transition(&mut self, target_state:ControlMode) {
        if self.state == target_state {
            println!("Control Mode: (still) {:}", self.state);
            self.state = target_state; // technically a no-op
        } else {
            println!("Control Mode: {:} --> {:}", self.state, target_state);
            self.state = target_state;
        }
    }
    fn safe(&mut self) {
        self.transition(ControlMode::Safe);
    }
    fn abort(&mut self) {
        let mut wait_ticker = 0;
        loop {
            if wait_ticker >= 5 {
                break;
            } else {
                self.transition(ControlMode::Dumping);
                println!("(waiting {:} seconds for ballast to finish dumping...)", 5-wait_ticker);
                thread::sleep(time::Duration::from_secs(1)); // wait 5 seconds
                wait_ticker = wait_ticker + 1;
            }
        }
        self.transition(ControlMode::Safe);
    }
    fn idle(&mut self) {
        self.transition(ControlMode::Idle);
    }
    fn stabilize(&mut self) {
        // run control algorithm every N ticks to refresh control effort
        // change mode to Raising if control effort wants to raise altitude
        // change mode to Lowering if control effort wants to lower altitude
        // change mode to Idle if altitude is within controller "Dead zone"
        // TODO: allow loop to be interrupted by other reinforced commands
        println!("The code for the Stabilize command hasn't been written yet!");
=======
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
>>>>>>> CTRL: state machine first try
    }
}

fn main() {
    // test state transitions here
<<<<<<< HEAD
    let mut controller = Controller::init(); // initialize the controller
    controller.safe(); // start in safe mode
    controller.idle(); // transition to idle
    controller.stabilize(); // ups and downs
    controller.idle(); // reassert idle
    controller.abort(); // abort and end in safe

    // test valves here
    let mut bleed_valve = Valve::init(0, "BleedValve".to_string());
    let mut ballast_valve = Valve::init(99, "BallastValve".to_string());
    bleed_valve.lock();
    ballast_valve.lock();
    ballast_valve.unlock();
    if bleed_valve.is_locked() == true {
        println!("{:} is locked!", bleed_valve.name);
    } else {
        println!("{:} is unlocked!", bleed_valve.name);
    }
    bleed_valve.unlock();
    println!("{:} PWM is {:}", ballast_valve.name, ballast_valve.pwm);
    ballast_valve.set_pwm(0.5);
    println!("{:} PWM is {:}", ballast_valve.name, ballast_valve.pwm);
    ballast_valve.lock();
    ballast_valve.set_pwm(0.5);
    println!("{:} PWM is {:}", ballast_valve.name, ballast_valve.pwm);
=======
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
>>>>>>> CTRL: state machine first try
    
}
