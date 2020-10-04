pub trait MFCMessage: Default {}

/// Cache of a message, storing the timestamp received along with the body
pub struct MessageCache <T: MFCMessage> {
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
    fn default() -> Self {AltCtrlStatus{cutdown:false}}
}


//// Altitude Control Arm ////
pub struct AltCtrlArm {}

impl MFCMessage for AltCtrlArm {}

impl Default for AltCtrlArm {
    fn default() -> Self {AltCtrlArm{}}
}


//// Ground Command ////
pub struct GroundCmd {}

impl MFCMessage for GroundCmd {}

impl Default for GroundCmd {
    fn default() -> Self {GroundCmd{}}
}
