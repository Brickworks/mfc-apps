use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use nng;
use nng::options::protocol::pubsub::Subscribe;
use nng::options::{Options, RecvTimeout};
use nng::{Message, PipeEvent, Protocol, Socket};


use serde::Deserialize;
use rmp_serde::Deserializer;

use mfc::common::mfc_msgs::{AltitudeBoardTlm, MessageCache};

const CYCLE_RATE_HZ: f32 = 1.0;
const BASE_SLEEP_DURATION_US: Duration =
    Duration::from_micros((1_000_000.0 * CYCLE_RATE_HZ) as u64);

fn tlm_listen(most_recent_msg: Arc<Mutex<MessageCache<AltitudeBoardTlm>>>) {
    let s = nng::Socket::new(nng::Protocol::Sub0).unwrap();
    s.dial("ipc:///tmp/nucleus").unwrap();
    let all_topics = vec![];
    s.set_opt::<Subscribe>(all_topics).unwrap();

    loop {
        println!("Waiting...");
        let msg = match s.recv() {
            Ok(v) => v,
            Err(_) => continue,
        };
        println!("Received: {}", std::str::from_utf8(&msg.as_slice()[..8]).unwrap());

        //let cur = std::io::Cursor::new(msg.as_slice());
        let mut de = Deserializer::new(&msg.as_slice()[8..]);
        let x: Result<AltitudeBoardTlm, _> = Deserialize::deserialize(&mut de);
        match x {
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!("{:?}", msg.as_slice())
            },
            Ok(v) => println!("Value: {:?}", v.altitude),
        };

        most_recent_msg
            .lock()
            .unwrap()
            .update(AltitudeBoardTlm::default());
    }
}

fn update_thread() {}

fn main() {
    let most_recent_msg = Arc::new(Mutex::new(MessageCache::<AltitudeBoardTlm>::default()));

    //println!("{}", most_recent_msg.lock().unwrap().timestamp);

    let listener_thread = std::thread::spawn(move || {
        tlm_listen(most_recent_msg)
    });

    listener_thread.join();
}

/*
fn main() {
    // Subscribe to topics
    // Update some local state in the callback
    let s = nng::Socket::new(nng::Protocol::Sub0).unwrap();
    s.dial("ipc:///tmp/nucleus").unwrap();
    let all_topics = vec![];
    s.set_opt::<Subscribe>(all_topics).unwrap();
    s.set_opt::<RecvTimeout>(Some(Duration::from_millis(500))).unwrap();

    let mut start = Instant::now();
    let mut diff = BASE_SLEEP_DURATION_US - start.elapsed();
    loop {
        println!("Ello? {:?}", diff);
        //update();

        match s.recv() {
            Err(_) => (),
            Ok(v) => {
                let subs = v.as_slice();
                println!("Msg: {:?}", subs);
            }
        };


        diff = BASE_SLEEP_DURATION_US - start.elapsed();
        sleep(BASE_SLEEP_DURATION_US - start.elapsed());
        start = Instant::now();
    }

}
*/
