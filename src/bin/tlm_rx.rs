extern crate rmp_serde as rmps;

use std::thread;
use std::io::Cursor;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

use rmps::Deserializer;
use serde;

static RX_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 6666);

#[derive(Debug, PartialEq, serde::Deserialize)]
struct LatLon {
    lat: f32,
    lon: f32,
}

fn eth_rx_loop(_thread_tx: SyncSender<u8>) {
    let mut buf = [0; 256];
    assert!(buf[255] == 0);

    let mut cur = Cursor::new(&buf[..]);
    let mut deserializer = Deserializer::new(cur);

    let socket = match UdpSocket::bind(SocketAddr::from(RX_ADDR)) {
        Ok(v) => v,
        Err(e) => {
            println!("Creating socket error: {:?}", e);
            return ();
        }
    };

    //socket.set_read_timeout(Some(Duration::from_millis(1000))).unwrap();

    loop {
        let (_size, _sender_addr) = match socket.recv_from(&mut buf[..]) {
            Ok(v) => v,
            Err(e) => {
                println! {"Error receiving message: {:?}", e};
                continue;
            }
        };
        //let filled_buf = &buf[..size];

        let actual: LatLon = serde::Deserialize::deserialize(&mut deserializer).unwrap();
        println!("Received: {:?}", actual)
    }
}

fn can_rx_loop(thread_tx: SyncSender<u8>) {
    for _ in 0..255 {
        thread::sleep(Duration::from_millis(5));
        thread_tx.send(0x55);
    }
}

fn ipc_tx_loop(thread_rx: Receiver<u8>) {
    loop {
        match thread_rx.recv() {
            Ok(v) => println!("Received: {}", v),
            Err(_) => {
                println!("Channel disconnected");
                break;
            }
        }
    }
}

fn main() {
    let (eth_thread_sender, thread_rx) = std::sync::mpsc::sync_channel(1);
    let can_thread_sender = eth_thread_sender.clone();

    let eth_rx_handler = thread::spawn(move || { eth_rx_loop(eth_thread_sender) });
    let can_rx_hander = thread::spawn(|| { can_rx_loop(can_thread_sender) });
    let ipc_tx_handler = thread::spawn(|| { ipc_tx_loop(thread_rx) });

    match eth_rx_handler.join() {
        Ok(_) => println!("eth_rx Shit worked out"),
        Err(e) => println!("eth_rx Error: {:?}", e)
    }
    can_rx_hander.join().unwrap();

    ipc_tx_handler.join().unwrap();
}