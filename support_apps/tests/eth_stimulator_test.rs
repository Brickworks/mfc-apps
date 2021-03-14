use std::net::{UdpSocket};

use rmp;
use rmp_serde as rmps;
use serde::Serialize;
use rmps::Serializer;

use mfc::common::mfc_msgs::AltitudeBoardTlm;

#[derive(Serialize)]
struct HabState {
    altitude: u32,
    ballas_mass: u32,
}


#[test]
fn test_eth_rx_relay() {
    let socket = UdpSocket::bind("127.0.0.1:5555").expect("couldn't bind to address");
    socket.connect("127.0.0.1:6666").expect("connect function failed");


    let tlm = AltitudeBoardTlm {
        altitude: 80000.0,
        ballast_mass: 0.0,
    };

    let mut buf = Vec::new();
    buf.push(0x01);
    let mut se = Serializer::new(buf);

    tlm.serialize(&mut se).unwrap();

    let value = se.into_inner();

    println!("Sent: {:x?}", value);
    socket.send(&value).expect("error sending");
    assert!(true);
}
