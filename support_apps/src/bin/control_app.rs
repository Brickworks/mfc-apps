use std::{env, thread};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

use nng;
use nng::options::protocol::pubsub::Subscribe;
use nng::options::Options;
use toml::Value;

use rmp_serde::{Deserializer, Serializer};
use serde::Deserialize;

use control_apps::control_mngr::ControlMngr;
use mfc::common::ipc::{self, NNG_PWM_ADDR};
use mfc::common::mfc_msgs;
use mfc::common::mfc_msgs::{AltitudeBoardTlm, MessageCache};

const CYCLE_RATE_HZ: f32 = 1.0;
const BASE_SLEEP_DURATION_US: Duration =
    Duration::from_micros((1_000_000.0 * CYCLE_RATE_HZ) as u64);

use serde::Serialize;

fn tlm_listen(most_recent_msg: Arc<Mutex<MessageCache<AltitudeBoardTlm>>>) {
    let s = nng::Socket::new(nng::Protocol::Sub0).unwrap();
    s.dial(ipc::NNG_TX_ADDR).unwrap();
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
            Err(_) => continue,
        };

        most_recent_msg.lock().unwrap().update(deser_msg);
    }
}

fn cmd_send(thread_rx: Receiver<(f32, f32)>) {
    let s = nng::Socket::new(nng::Protocol::Pub0).unwrap();
    s.listen(ipc::NNG_PWM_ADDR).unwrap();

    loop {
        let pwms = match thread_rx.recv() {
            Ok(v) => v,
            Err(e) => {
                println!("Channel disconnected {:?}", e);
                break;
            }
        };

        let mut buffer = Vec::new();
        pwms.serialize(&mut Serializer::new(&mut buffer)).unwrap();

        let msg_content = ipc::fmt_nng_msg("pwms", buffer.as_slice());

        nng::Message::from_slice(msg_content.as_slice()).unwrap();
        match s.send(nng::Message::from_slice(msg_content.as_slice()).unwrap()) {
            Ok(_) => (),
            Err(e) => println!("Failed to send ipc msg; {:?}", e),
        }
    }
}

fn updater(
    most_recent_msg: Arc<Mutex<MessageCache<AltitudeBoardTlm>>>,
    mngr: &mut ControlMngr,
    thread_tx: Sender<(f32, f32)>,
) {
    let mut start = Instant::now();
    loop {
        let incoming_msg_guard = most_recent_msg.lock().unwrap();
        let incoming_msg = &incoming_msg_guard.msg;

        let pwms = mngr.update(incoming_msg.altitude, 0.0, incoming_msg.ballast_mass);
        std::mem::drop(incoming_msg_guard); // release the lock

        thread_tx.send(pwms).unwrap();

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
    let (thread_tx, thread_rx) = std::sync::mpsc::channel();

    let listener_msg_copy = most_recent_msg.clone();
    let listener_thread = std::thread::spawn(move || tlm_listen(listener_msg_copy));

    let mut mngr = ControlMngr::new(
        config["target_altitude_m"].as_float().unwrap() as f32,
        config["vent_kp"].as_float().unwrap() as f32,
        config["vent_ki"].as_float().unwrap() as f32,
        config["vent_kd"].as_float().unwrap() as f32,
        config["dump_kp"].as_float().unwrap() as f32,
        config["dump_ki"].as_float().unwrap() as f32,
        config["dump_kd"].as_float().unwrap() as f32,
    );
    let update_thread = std::thread::spawn(move || updater(most_recent_msg, &mut mngr, thread_tx));

    let commander_thread = std::thread::spawn(move || cmd_send(thread_rx));

    listener_thread.join().unwrap();
    update_thread.join().unwrap();
    commander_thread.join().unwrap();
}
