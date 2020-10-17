use mfc::common::mfc_msgs::*;
use mfc::manager::ipc_receiver::*;

mod manager_state {
    pub struct CutdownStateTracker {
        cutdown_armed: bool,
        // state of safety lock, changes from ground
        cutdown_ground: bool,
        // has the ground commanded cutdown? latched
        cutdown_ctrl: bool,   // has the control app commanded cutdown? latched
    }

    impl CutdownStateTracker {
        pub fn new() -> CutdownStateTracker {
            CutdownStateTracker {
                cutdown_armed: false,
                cutdown_ground: false,
                cutdown_ctrl: false,
            }
        }
        /// Move the cutdown arm state into ARMED
        pub fn arm(&mut self) {
            self.cutdown_armed = true
        }
        /// Move the cutdown arm state into DISARMED
        pub fn disarm(&mut self) {
            self.cutdown_armed = false
        }
        /// Set the latch that the ground has commanded for a cutdown
        pub fn set_cutdown_ground(&mut self) {
            self.cutdown_ground = true;
        }
        /// Set the latch that the ctrl app has commanded for a cutdown
        pub fn set_cutdown_ctrl(&mut self) {
            self.cutdown_ctrl = true;
        }
        /// Determine if we should cutdown based on latches and the given arm state
        pub fn should_we_cutdown(&self) -> bool {
            if self.cutdown_armed && (self.cutdown_ground || self.cutdown_ctrl) {
                true
            } else {
                false
            }
        }
    }
}

fn main() {
    let messages = ManagerIPCReceiver::new();

    let mut cutdown_state_tracker = manager_state::CutdownStateTracker::new();

    let alt_ctrl_status_msg = messages.get_alt_ctrl_status();
    let alt_ctrl_arm_msg = messages.get_alt_ctrl_arm();
    let ground_cmd_msg = messages.get_ground_cmd();

    loop {
        // Update received IPC messages
        // TODO: maybe block until we get a message type we want?
        messages.update();

        //// UPDATE STATE ////
        // Update cutdown state
        if ground_cmd_msg.msg.arm_cutdown == true {
            cutdown_state_tracker.arm();
        }

        if ground_cmd_msg.msg.cutdown == true {
            cutdown_state_tracker.set_cutdown_ground();
        }

        if alt_ctrl_arm_msg.msg.cutdown == true {
            cutdown_state_tracker.set_cutdown_ctrl();
        }

        //// COMMAND /////
        if cutdown_state_tracker.should_we_cutdown() == true {
            // SEND CUTDOWN COMMAND
        }

        // tmp for dev
        println!("Finished loop");
        break;
    }

    // 1) recv msgs (cache messages that have been received, may need to add timestamps)

    // 2) analyze receied msgs and update state as needed

    // 3) Look at state and determine if commands need to be sent back.
    // Should we handle responding to commands with mutable flags? Where updating state may set them, and the sending step resets em?
}
