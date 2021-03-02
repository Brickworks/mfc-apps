use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::env;
use std::fs::File;
use std::path::Path;

use toml::Value;
use nng;
use nng::options::protocol::pubsub::Subscribe;
use nng::options::{Options, RecvTimeout};

use rmp_serde::Deserializer;
use serde::Deserialize;

use mfc::common::mfc_msgs;
use mfc::common::mfc_msgs::{AltitudeBoardTlm, MessageCache};
use control_apps::control_mngr::ControlMngr;

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

fn updater(most_recent_msg: Arc<Mutex<MessageCache<AltitudeBoardTlm>>>, mngr: ControlMngr) {
    let mut start = Instant::now();
    loop {
        let data = most_recent_msg.lock().unwrap();

        sleep(BASE_SLEEP_DURATION_US - start.elapsed());
        start = Instant::now();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = std::fs::read_to_string(&args[1])
        .unwrap()
        .as_str()
        .parse::<Value>()
        .unwrap();

    let most_recent_msg = Arc::new(Mutex::new(MessageCache::<AltitudeBoardTlm>::default()));


    let listener_msg_copy = most_recent_msg.clone();
    let listener_thread = std::thread::spawn(move || tlm_listen(listener_msg_copy));

    let mngr = ControlMngr::new(
        config["target_altitude_m"].as_float().unwrap() as f32,
        config["vent_kp"].as_float().unwrap() as f32,
        config["vent_ki"].as_float().unwrap() as f32,
        config["vent_kd"].as_float().unwrap() as f32,
        config["dump_kp"].as_float().unwrap() as f32,
        config["dump_ki"].as_float().unwrap() as f32,
        config["dump_kd"].as_float().unwrap() as f32,
    );
    let update_thread = std::thread::spawn(move || updater(most_recent_msg, mngr));

    listener_thread.join().unwrap();
    update_thread.join().unwrap();
}
