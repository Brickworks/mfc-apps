use crate::common::mfc_msgs::{AltCtrlCmd, AltCtrlStatus, GroundCmd, MessageCache};
use crate::manager::ipc_receiver::IPCReceiverError::Generic;

/// A structure to store the last received deserialized message for each inbound topic
/// In the future there will be some methods that assist with setup and receiving.
pub struct ManagerIPCReceiver {
    alt_ctrl_status: MessageCache<AltCtrlStatus>,
    alt_ctrl_arm: MessageCache<AltCtrlCmd>,
    ground_cmd: MessageCache<GroundCmd>,
}

pub enum IPCReceiverError {
    Generic,
}

impl ManagerIPCReceiver {
    pub fn new() -> ManagerIPCReceiver {
        ManagerIPCReceiver {
            alt_ctrl_status: MessageCache::<AltCtrlStatus>::default(),
            alt_ctrl_arm: MessageCache::<AltCtrlCmd>::default(),
            ground_cmd: MessageCache::<GroundCmd>::default(),
        }
    }

    pub fn get_alt_ctrl_status(&self) -> &MessageCache<AltCtrlStatus> {
        &self.alt_ctrl_status
    }

    pub fn get_alt_ctrl_arm(&self) -> &MessageCache<AltCtrlCmd> {
        &self.alt_ctrl_arm
    }

    pub fn get_ground_cmd(&self) -> &MessageCache<GroundCmd> {
        &self.ground_cmd
    }

    pub fn update(&self) -> Result<(), IPCReceiverError> {
        Err(Generic)
    }
}
