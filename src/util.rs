use std::{
    fmt,
    net::{SocketAddrV4, SocketAddr},
};

use crate::messages::UDPMessage;

/// Size of buffer of network sockets for connecting to STUN server
pub const STUN_BUFFER_SIZE: usize = 128;

/// Local address on which we should bind in order to open the UDP socket as client
pub const LOCAL_UDP_BIND_ADDRESS: SocketAddrV4 = SocketAddrV4::new(std::net::Ipv4Addr::new(0, 0, 0, 0), 0);

/// The buffer size which is used to copy two UDP sockets
const FORWARD_BUFFER_SIZE: usize = 4 * 1024;

/// Kill the program with an error message
pub fn die<E: fmt::Debug>(error: E) -> ! {
    log::error!("{:?}", error);
    std::process::exit(1);
}

/// Sends an UDP packet from a socket to address
pub fn send_udp_packet(msg: &UDPMessage, socket: &std::net::UdpSocket, addr: &SocketAddrV4) {
    if let Ok(write_buffer) = postcard::to_vec::<&UDPMessage, STUN_BUFFER_SIZE>(&msg) {
        // Send it
        let _ = socket.send_to(&write_buffer, addr);
    }
}

// Copy UDP diagrams from one socket to another bidirectionally
pub async fn forward_udp(remote_socket: tokio::net::UdpSocket, local_socket: tokio::net::UdpSocket, remote_address: SocketAddrV4, local_address: SocketAddr) -> anyhow::Result<()> {
    // Connect to hosts from each socket
    remote_socket.connect(remote_address).await?;
    local_socket.connect(local_address).await?;
    // One thread to copy from remote to local
    let task1 = copy_udp(&remote_socket, &local_socket);
    // One thread to copy from local to remote
    let task2 = copy_udp(&local_socket, &remote_socket);
    // Wait for them
    tokio::try_join!(task1, task2)?;
    Ok(())
}

async fn copy_udp(a: &tokio::net::UdpSocket, b: &tokio::net::UdpSocket) -> anyhow::Result<()> {
    let mut buffer = [0; FORWARD_BUFFER_SIZE];
    loop {
        let read = a.recv(&mut buffer).await?;
        b.send(&buffer[..read]).await?;
        tokio::task::yield_now().await;
    }
}