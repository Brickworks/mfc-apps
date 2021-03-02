use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

pub trait MFCMessage: Default + Serialize {}

/// Cache of a message, storing the timestamp received along with the body
pub struct MessageCache<T: MFCMessage> {
    timestamp: Instant,
    updated: bool,
    pub msg: T,
}

impl<T: MFCMessage> Default for MessageCache<T> {
    /// provide a default message cache with a default of the message type
    fn default() -> Self {
        MessageCache::<T> {
            timestamp: Instant::now(),
            updated: false,
            msg: T::default(),
        }
    }
}

impl<T: MFCMessage> MessageCache<T> {
    pub fn update(&mut self, new_msg: T) {
        self.timestamp = Instant::now();
        self.msg = new_msg;
    }

    /// Returns a duration from now until the msg was last updated.
    /// MAX duration will be returned if it has not yet been updated.
    pub fn get_age(&self) -> Duration {

        if !self.updated {
            return Duration::new(u64::MAX, 0)
        }

        return Instant::now() - self.timestamp;
    }
}

//// Altitude Board Tlm ////
pub const ALT_CTRL_TOPIC: &str = "altitude";

#[derive(Serialize, Deserialize)]
pub struct AltitudeBoardTlm {
    pub altitude: f32,
    pub ballast_mass: f32,
}

impl MFCMessage for AltitudeBoardTlm {}

impl Default for AltitudeBoardTlm {
    fn default() -> Self {
        AltitudeBoardTlm {
            altitude: 0.0,
            ballast_mass: 0.0,
        }
    }
}

//// Altitude Control Status ////
#[derive(Serialize, Deserialize)]
pub struct AltCtrlStatus {
    pub cutdown: bool,
}

impl MFCMessage for AltCtrlStatus {}

impl Default for AltCtrlStatus {
    fn default() -> Self {
        AltCtrlStatus { cutdown: false }
    }
}

//// Altitude Control Arm ////
#[derive(Serialize, Deserialize)]
pub struct AltCtrlCmd {
    /// True: request actuator control to be armed, false to disarm
    pub arm_actuator: bool,
    /// True: request the HAB performs a cutdown, false to maintain
    pub cutdown: bool,
}

impl MFCMessage for AltCtrlCmd {}

impl Default for AltCtrlCmd {
    fn default() -> Self {
        AltCtrlCmd {
            arm_actuator: false,
            cutdown: false,
        }
    }
}

//// Ground Command ////
#[derive(Serialize, Deserialize)]
pub struct GroundCmd {
    pub arm_alt_ctrl: bool,
    pub arm_cutdown: bool,
    pub cutdown: bool,
}

impl MFCMessage for GroundCmd {}

impl Default for GroundCmd {
    fn default() -> Self {
        GroundCmd {
            arm_alt_ctrl: false,
            arm_cutdown: false,
            cutdown: false,
        }
    }
}
