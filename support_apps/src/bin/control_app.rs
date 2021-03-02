use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use nng;
use nng::options::protocol::pubsub::Subscribe;
use nng::options::{Options, RecvTimeout};
use nng::{Message, PipeEvent, Protocol, Socket};

use rmp_serde::Deserializer;
use serde::Deserialize;

use mfc::common::mfc_msgs;
use mfc::common::mfc_msgs::{AltitudeBoardTlm, MessageCache};
use control_apps::control_mngr;

const CYCLE_RATE_HZ: f32 = 1.0;
const BASE_SLEEP_DURATION_US: Duration =
    Duration::from_micros((1_000_000.0 * CYCLE_RATE_HZ) as u64);

fn tlm_listen(most_recent_msg: Arc<Mutex<MessageCache<AltitudeBoardTlm>>>) {
    let s = nng::Socket::new(nng::Protocol::Sub0).unwrap();
    s.dial("ipc:///tmp/nucleus").unwrap();
    s.set_opt::<Subscribe>(String::from(mfc_msgs::ALT_CTRL_TOPIC).into_bytes())
        .unwrap();

    let topic_len: usize = mfc_msgs::ALT_CTRL_TOPIC.chars().count() + 1;

    loop {
        let msg = match s.recv() {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Try and deserialize into a AltitudeBoardTlm msg
        let mut de = Deserializer::new(&msg.as_slice()[topic_len..]);
        let deser_msg = match Deserialize::deserialize(&mut de) as Result<AltitudeBoardTlm, _> {
            Ok(v) => v,
            Err(e) => continue,
        };

        most_recent_msg.lock().unwrap().update(deser_msg);
    }
}

fn updater(most_recent_msg: Arc<Mutex<MessageCache<AltitudeBoardTlm>>>) {
    let mut start = Instant::now();
    let mut diff = BASE_SLEEP_DURATION_US - start.elapsed();

    loop {

        diff = BASE_SLEEP_DURATION_US - start.elapsed();
        sleep(BASE_SLEEP_DURATION_US - start.elapsed());
        start = Instant::now();
    }
}

fn main() {
    let most_recent_msg = Arc::new(Mutex::new(MessageCache::<AltitudeBoardTlm>::default()));

    //println!("{}", most_recent_msg.lock().unwrap().timestamp);

    let listener_msg_copy = most_recent_msg.clone();
    let listener_thread = std::thread::spawn(move || tlm_listen(listener_msg_copy));
    let update_thread = std::thread::spawn(move || updater(most_recent_msg));

    listener_thread.join().unwrap();
    update_thread.join().unwrap();
}