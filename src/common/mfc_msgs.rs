pub trait MFCMessage: Default {}

/// Cache of a message, storing the timestamp received along with the body
pub struct MessageCache<T: MFCMessage> {
    pub timestamp: u64,
    pub msg: T,
}

impl<T: MFCMessage> Default for MessageCache<T> {
    /// provide a default message cache with a default of the message type
    fn default() -> Self {
        MessageCache::<T> {
            timestamp: 0,
            msg: T::default(),
        }
    }
}

//// Altitude Control Status ////
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
