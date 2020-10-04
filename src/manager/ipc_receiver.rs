use crate::common::mfc_msgs::{MessageCache, AltCtrlStatus, AltCtrlArm, GroundCmd};

/// A structure to store the last received deserialized message for each inbound topic
/// In the future there will be some methods that assist with setup and receiving.
pub struct ManagerMessages {
    alt_ctrl_status: MessageCache<AltCtrlStatus>,
    alt_ctrl_arm: MessageCache<AltCtrlArm>,
    ground_cmd: MessageCache<GroundCmd>,
}

impl ManagerMessages {
    pub fn new() -> ManagerMessages {
        ManagerMessages {
            alt_ctrl_status: MessageCache::<AltCtrlStatus>::default(),
            alt_ctrl_arm: MessageCache::<AltCtrlArm>::default(),
            ground_cmd: MessageCache::<GroundCmd>::default(),
        }
    }

    fn get_alt_ctrl_status(&self) -> &MessageCache<AltCtrlStatus> {
        &self.alt_ctrl_status
    }
}