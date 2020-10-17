extern crate rmp_serde as rmps;

use std::io::{Cursor, Write};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::{Receiver, SyncSender};
use std::thread;

use nng;
use rmp;
use serde;

static UDP_RX_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 6666);
static NNG_TX_ADDR: &str = "ipc:///tmp/nucleus";
//static NNG_TX_ADDR: &str = "tcp://localhost:3008";

#[derive(Debug, PartialEq, serde::Deserialize)]
struct LatLon {
    lat: f32,
    lon: f32,
}

/// Relays UDP msgpack messages to the IPC sender
/// Not expected to terminate
fn eth_rx_loop(thread_tx: SyncSender<Vec<u8>>) {
    let mut buf = [0; 256];

    let socket = match UdpSocket::bind(SocketAddr::from(UDP_RX_ADDR)) {
        Ok(v) => v,
        Err(e) => {
            println!("Creating socket error: {:?}", e);
            return ();
        }
    };

    // Loop on listening for UDP packets, and relaying them over to the IPC sender
    loop {
        let (size, _sender_addr) = match socket.recv_from(&mut buf[..]) {
            Ok(v) => v,
            Err(e) => {
                println! {"Error receiving message: {:?}", e};
                continue;
            }
        };

        if let Err(e) = thread_tx.send(buf[..size].to_vec()) {
            println! {"Error sending eth rx intrapc: {:?}", e};
        }
    }
}

/// Relays msgpack CAN messages to IPC sender thread
/// Not expected to terminate
fn can_rx_loop(_thread_tx: SyncSender<Vec<u8>>) {
    // TODO implement receiving CAN messages
    // TODO implement sending these messages via intra thread comms
    loop {}
}

/// Construct a NNG message with the given topic
/// TODO: Move to common code
fn fmt_nng_msg(topic: &str, body: &[u8]) -> Vec<u8> {
    [topic.as_bytes(), ":".as_bytes(), body].concat()
}

/// Receives intra-thread messages to publish to a NNG socket for IPC
/// Not expected to terminate
fn ipc_tx_loop(thread_rx: Receiver<Vec<u8>>) {
    // TODO move initialization of sockets to init function, moving into thread
    let s = nng::Socket::new(nng::Protocol::Pub0).unwrap();
    s.listen(NNG_TX_ADDR).unwrap();
    // TODO use contexts and spawn threads

    loop {
        // listen for messages from other threads
        let buf = match thread_rx.recv() {
            Ok(v) => v,
            Err(e) => {
                println!("Channel disconnected: {:?}", e);
                break;
            }
        };

        // grab extension code and size of data
        let ext_meta = rmp::decode::read_ext_meta(&mut Cursor::new(&buf)).unwrap();

        // chop off extension, this might be janky? TODO find a better way
        let data = &buf[(buf.len() - (ext_meta.size as usize))..];

        let mut nng_msg = nng::Message::new().unwrap();

        // TODO add ext meta to topic lookup table
        let msg_content = fmt_nng_msg("testing", data);

        // publish nng_msg on nng
        nng_msg.write_all(msg_content.as_slice()).unwrap();
        match s.send(nng_msg) {
            Ok(_) => (),
            Err(e) => println!("Failed to send ipc msg: {:?}", e),
        };
    }
}

fn main() {
    // TODO move setup of sockets and all to an init function, moving the objects into the threads
    let (eth_thread_sender, thread_rx) = std::sync::mpsc::sync_channel(1);
    let can_thread_sender = eth_thread_sender.clone();

    let eth_rx_handler = thread::spawn(move || eth_rx_loop(eth_thread_sender));
    let can_rx_hander = thread::spawn(|| can_rx_loop(can_thread_sender));
    let ipc_tx_handler = thread::spawn(|| ipc_tx_loop(thread_rx));

    eth_rx_handler.join().unwrap();
    can_rx_hander.join().unwrap();
    ipc_tx_handler.join().unwrap();
}
