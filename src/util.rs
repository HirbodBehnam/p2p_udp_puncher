use std::{fmt, net::{SocketAddr, UdpSocket, SocketAddrV4}};

use crate::messages::UDPMessage;

// Size of buffer of network sockets for connecting to STUN server
pub const STUN_BUFFER_SIZE: usize = 128;

pub fn die<E: fmt::Debug>(error: E) -> ! {
    log::error!("{:?}", error);
    std::process::exit(1);
}

/// Sends an UDP packet from a socket into an
pub fn send_udp_packet(msg: &UDPMessage, socket: &UdpSocket, addr: &SocketAddrV4) {
    if let Ok(write_buffer) = postcard::to_vec::<&UDPMessage, STUN_BUFFER_SIZE>(&msg) {
        // Send it
        let _ = socket.send_to(&write_buffer, addr);
    }
}