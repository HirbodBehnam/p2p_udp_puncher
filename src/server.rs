use std::net::{SocketAddr, SocketAddrV4, ToSocketAddrs};

use anyhow::bail;
use tokio::{net::UdpSocket, task, time, select};

use crate::{
    messages::{PunchMessage, UDPMessage},
    util::{die, forward_udp, LOCAL_UDP_BIND_ADDRESS, STUN_BUFFER_SIZE},
};

/// Spawn a webserver which gets incoming connections from STUN server
pub async fn spawn_server(forward: &str, stun: &str, service: &str) -> ! {
    // Parse socket addresses
    let forward_address = forward
        .to_socket_addrs()
        .expect("cannot parse forward address")
        .next()
        .expect("cannot parse forward address");
    let stun_address = stun
        .to_socket_addrs()
        .expect("cannot parse STUN address")
        .next()
        .expect("cannot parse STUN address");
    let stun_address = match stun_address {
        SocketAddr::V4(v4) => v4,
        SocketAddr::V6(_) => die("stun address cannot be IPv6"),
    };
    // In a loop, we must connect to STUN server and advertise ourselves
    loop {
        // Spawn a client
        let socket = match UdpSocket::bind(LOCAL_UDP_BIND_ADDRESS).await {
            Ok(socket) => socket,
            Err(err) => die(err),
        };
        log::debug!("Started a socket on {}", socket.local_addr().unwrap());
        // Connect to stun server and get the client address
        let client_addr = stun_handshake(&socket, &stun_address, service).await;
        // Now punch!
        tokio::task::spawn(async move {
            if let Err(err) = punch(socket, client_addr, forward_address).await {
                log::error!("Cannot punch: {}", err);
            }
        });
    }
}

async fn stun_handshake(socket: &UdpSocket, stun: &SocketAddrV4, service: &str) -> SocketAddrV4 {
    let mut buf = [0; STUN_BUFFER_SIZE];
    // Send server hello
    log::debug!("Sending server hello");
    let write_buffer = postcard::to_slice(
        &UDPMessage::Server {
            service_name: service,
        },
        &mut buf,
    )
    .unwrap();
    socket.send_to(&write_buffer, stun).await.unwrap();
    // Get the answer
    log::debug!("Waiting for STUN ack");
    let (read_len, _) = socket.recv_from(&mut buf).await.unwrap();
    let stun_ack = match postcard::from_bytes::<UDPMessage<'_>>(&buf[..read_len]) {
        Err(err) => die(format!("Got invalid packet from STUN server: {}", err)),
        Ok(pkt) => pkt,
    };
    // Check status
    if !matches!(stun_ack, UDPMessage::Ok) {
        die(format!(
            "Got non successful ack packet from STUN server: {:?}",
            stun_ack
        ));
    }
    log::info!("Server registered {}", socket.local_addr().unwrap());
    // Keep alive to tell the NAT to keep the state.
    // This method should never return
    let keep_alive = async {
        let keep_alive_buffer = postcard::to_vec::<UDPMessage<'_>, STUN_BUFFER_SIZE>(&UDPMessage::KeepAlive).unwrap();
        loop {
            time::sleep(time::Duration::from_secs(1)).await;
            log::trace!("Sending keep alive from {}", socket.local_addr().unwrap());
            if socket.send_to(&keep_alive_buffer, stun).await.is_err() {
                break;
            }
        }
    };
    // Wait for punch and poll the keep alive
    let stun_punch;
    select! {
        () = keep_alive => unreachable!(),
        recv_result = socket.recv_from(&mut buf) => {
            let (read_len, _) = recv_result.unwrap();
            stun_punch = match postcard::from_bytes::<UDPMessage<'_>>(&buf[..read_len]) {
                Err(err) => die(format!("Got invalid packet from STUN server: {}", err)),
                Ok(pkt) => pkt,
            };
        },
    };
    // Parse packet
    if let UDPMessage::Punch(p) = &stun_punch {
        if let PunchMessage::STUN(other) = &p {
            log::info!("Client peer is {}", other);
            return *other;
        }
    }
    // Something went south
    die(format!(
        "Got invalid packet from STUN server while waiting for client: {:?}",
        stun_punch
    ));
}

async fn punch(
    socket: UdpSocket,
    other_peer: SocketAddrV4,
    forward_address: SocketAddr,
) -> anyhow::Result<()> {
    let mut punch_buffer = [0; 4]; // very very small buffer. The packet is 2 bytes only
    log::info!(
        "Punching {} from {}",
        other_peer,
        socket.local_addr().unwrap()
    );
    // Step 1: Punch the NAT
    let to_write_punch_buffer = postcard::to_slice(
        &UDPMessage::Punch {
            0: PunchMessage::Peer,
        },
        &mut punch_buffer,
    )
    .unwrap();
    socket.send_to(to_write_punch_buffer, other_peer).await?;
    // Step 2: Wait for client to send something back
    log::debug!("Waiting for client step 2 handshake");
    let (packet_length, _) = socket.recv_from(&mut punch_buffer).await?;
    let client_punch = postcard::from_bytes::<UDPMessage<'_>>(&punch_buffer[..packet_length])?;
    if !matches!(client_punch, UDPMessage::Punch(PunchMessage::Peer)) {
        bail!(
            "Invalid packet received from client peer: {:?}",
            client_punch
        );
    }
    // Send back a packet
    log::debug!("Sending handshake step 3");
    let to_write_punch_buffer = postcard::to_slice(
        &UDPMessage::Punch {
            0: PunchMessage::Peer,
        },
        &mut punch_buffer,
    )
    .unwrap();
    socket.send_to(to_write_punch_buffer, other_peer).await?;
    // Now dial the destination
    let local_socket = UdpSocket::bind(LOCAL_UDP_BIND_ADDRESS).await?;
    // Now proxy data
    task::spawn(async move {
        if let Err(err) = forward_udp(socket, local_socket, other_peer, forward_address).await {
            log::error!("Cannot forward: {}", err);
        }
    });
    // Done
    return Ok(());
}
