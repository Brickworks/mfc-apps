use std::net::{UdpSocket};

use rmp;
use rmp_serde as rmps;
use serde::Serialize;

use rmps::Serializer;

#[derive(Serialize)]
struct Dog {
    number: u32,
    age: u32,
}


#[test]
fn test_eth_rx_relay() {
    let socket = UdpSocket::bind("127.0.0.1:5555").expect("couldn't bind to address");
    socket.connect("127.0.0.1:6666").expect("connect function failed");


    let dog = Dog {
        number: 0xdeadbeef,
        age: 0xcafecafe,
    };

    let mut ser_buf = Vec::new();
    ser_buf.push(0x01);
    let mut se = Serializer::new(ser_buf)
        .with_struct_map();

    dog.serialize(&mut se).unwrap();
    println!("!!!!!!!!!!!!!!!!!!");

    let value = se.into_inner();

    println!("Sent: {:x?}", value);
    socket.send(&value).expect("error sending");
    assert!(false);
}
