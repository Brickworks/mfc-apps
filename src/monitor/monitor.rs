extern crate rmp_serde as rmps;

use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use rmps::{Deserializer, Serializer};
use rmps::encode::{ExtFieldSerializer, ExtSerializer};
use nng::{Protocol, Socket, Message};
use nng::options::protocol::pubsub::Subscribe;
use nng::options::Options;

use serde_json::{Value};

use nucleus_common::tlm_def::av_sensor_card;



fn main() {
    let s = match Socket::new(Protocol::Sub0) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            panic!();
        }
    };

    match s.dial("ipc:///tmp/test0.ipc") {
        Ok(v) => (),
        Err(e) => {
            println!("Error: {}", e);
            panic!();
        }
    };

    let topic = String::from("avionics");
    match s.set_opt::<Subscribe>(topic.into_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            panic!();
        }
    };

    loop {
        let msg = match s.recv() {
            Ok(v) => v,
            Err(e) => {
                println!("Error: {}", e);
                panic!();
            }
        };

        let raw_nng_body = match std::str::from_utf8(msg.as_slice()) {
            Ok(v) =>  v,
            Err(e) => {
                println!("Error decoding: {}", e);
                "ERROR"
            }
        };

        let (_,body) = raw_nng_body.split_at(raw_nng_body.find(':').unwrap_or_default());
        println!{"body: {}", body}

        let v: Value =  match serde_json::from_str(body) {
            Ok(r) => r,
            Err(e) => {
                println!("Error decoding: {}", e);
                panic!();
            }
        };
    }
    /*
    println!("Hello, world!");
    let a = av_sensor_card::TlmPacket{
        gps: av_sensor_card::Gps {
            lat: 0.0,
            lon: 0.0,
            alt: 0.0,
        }
    };


    let mut buf = Vec::new();
    //a.serialize(&mut Serializer::new(&mut buf)).unwrap();

    //let mut se = Serializer::new(Vec::new()).with_struct_map();
    //a.serialize(&mut se).unwrap();


    let mut se = Serializer::new(Vec::new());
    let xs = ExtSerializer::new(&mut se);

    buf.insert(0, 'a' as u8);
    buf.insert(1, ':' as u8);

    println!("{}", buf.len());

    let sock = Socket::new(Protocol::Pub0).unwrap();
    match sock.listen("tcp://localhost:3001") {
        Ok(_) => println!("Listen OK"),
        Err(e) => {
            println!("Listen FAILED: {}", e);
            panic!()
        }

    }

    loop {
        thread::sleep(Duration::from_secs(1));
        let mut msg = Message::from_slice(&buf).unwrap();
        //let msg = Message::from_slice("a:rust".as_bytes()).unwrap();

        println!("Sending {:?}", msg.len());
        match sock.try_send(msg) {
            Ok(t) => println!("OK"),
            Err(e) => {
                println!("NOT OK");
                panic!()
            }
        }
    }
    */
}