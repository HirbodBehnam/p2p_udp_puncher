use std::{
    collections::HashMap,
    net::{SocketAddr, SocketAddrV4, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};

use tokio::net::UdpSocket;

use crate::{
    messages::{PunchMessage, UDPMessage},
    util::{die, FORWARD_BUFFER_SIZE, LOCAL_UDP_BIND_ADDRESS, STUN_BUFFER_SIZE},
};

/// Spawn a client which connects to a server which is punched via a STUN server
pub async fn spawn_client(listen: &str, stun: &str, service: &str) -> ! {
    // Parse socket addresses
    let stun_address = stun
        .to_socket_addrs()
        .expect("cannot parse STUN address")
        .next()
        .expect("cannot parse STUN address");
    let stun_address = match stun_address {
        SocketAddr::V4(v4) => v4,
        SocketAddr::V6(_) => die("stun address cannot be IPv6"),
    };
    // Listen for incoming connections. We leak this socket because its open until the end of program
    let listener_socket: &'static UdpSocket = Box::leak(Box::new(
        UdpSocket::bind(listen)
            .await
            .expect("cannot bind to local address"),
    ));
    log::info!("Listening on {}", listener_socket.local_addr().unwrap());
    let mut buffer = [0; FORWARD_BUFFER_SIZE];
    // A map from remote address to outbound sockets
    let mut connection_map: HashMap<SocketAddr, Arc<UdpSocket>> = HashMap::new();
    // In a loop wait for connections and forward them
    loop {
        // Wait for packets...
        let (read_bytes, addr) = listener_socket
            .recv_from(&mut buffer)
            .await
            .expect("cannot read data from socket");
        // Check if this address exists in our map or not
        if let Some(udp_socket) = connection_map.get(&addr) {
            if let Err(err) = udp_socket.send(&buffer[..read_bytes]).await {
                // Delete this entry from map
                log::warn!(
                    "cannot send udp packet to {}: {}",
                    udp_socket.peer_addr().unwrap(),
                    err
                );
                connection_map.remove(&addr);
            }
            continue;
        }
        // Otherwise, we need to punch!
        log::info!("New connection from {}", addr);
        let server_socket = Arc::new(punch(&stun_address, service).await);
        log::info!(
            "{} now is sending packets to {}",
            addr,
            server_socket.peer_addr().unwrap()
        );
        // Send the first packet we just got
        let _ = server_socket.send(&buffer[..read_bytes]).await; // fuck errors
                                                                 // Register the socket
        connection_map.insert(addr, server_socket.clone());
        // Create a thread to watch incoming packets
        tokio::task::spawn(async move {
            let mut buffer = [0; FORWARD_BUFFER_SIZE];
            loop {
                let read = server_socket.recv(&mut buffer).await?;
                listener_socket.send_to(&buffer[..read], addr).await?;
                tokio::task::yield_now().await;
            }
            // Read here: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
            #[allow(unreachable_code)]
            tokio::io::Result::Ok(())
        });
    }
}

async fn punch(stun: &SocketAddrV4, service: &str) -> UdpSocket {
    let mut buffer = [0; STUN_BUFFER_SIZE];
    // At first create a socket
    let socket = UdpSocket::bind(LOCAL_UDP_BIND_ADDRESS)
        .await
        .expect("Cannot bind new socket");
    log::debug!("Bound local socket on {}", socket.local_addr().unwrap());
    // Now send the STUN hello to server.
    // Server might not be ready. In this case we implement a retry mechanism.
    let server_address: SocketAddrV4;
    let mut retry_counter = 0;
    loop {
        let write_buffer = postcard::to_slice(
            &UDPMessage::Client {
                service_name: service,
            },
            &mut buffer,
        )
        .unwrap();
        socket
            .send_to(write_buffer, stun)
            .await
            .expect("Cannot send STUN hello to STUN server");
        // This should send back either server address or a error which server does exists (yet)
        let (read_bytes, _) = socket
            .recv_from(&mut buffer)
            .await
            .expect("Cannot get STUN hello answer");
        let stun_punch = match postcard::from_bytes::<UDPMessage<'_>>(&buffer[..read_bytes]) {
            Err(err) => die(format!("Got invalid packet from STUN server: {}", err)),
            Ok(pkt) => pkt,
        };
        // Check status
        if let UDPMessage::Punch(p) = stun_punch {
            if let PunchMessage::STUN(peer) = p {
                log::info!("Got {} as server address", peer);
                server_address = peer;
                break;
            }
        }
        // Fuck up. Retry
        log::warn!(
            "Cannot get the server address from STUN server. Got {:?}",
            stun_punch
        );
        if retry_counter == 5 {
            die("Out of reties. RIP");
        }
        retry_counter += 1;
        tokio::time::sleep(Duration::from_secs(retry_counter)).await;
        log::warn!("Retrying...");
    }
    // Before punching, wait one second in order to let the server punch its NAT
    tokio::time::sleep(Duration::from_secs(1)).await;
    // Now punch! (handshake step 2)
    socket
        .connect(server_address)
        .await
        .expect("cannot do the connect to server ip");
    let write_buffer = postcard::to_slice(
        &UDPMessage::Punch {
            0: PunchMessage::PeerHandshake2,
        },
        &mut buffer,
    )
    .unwrap();
    socket.send(write_buffer).await.expect("cannot punch");
    log::debug!("Punched own NAT");
    // Wait for server]
    loop {
        let read_bytes = socket
            .recv(&mut buffer)
            .await
            .expect("cannot do third stage of punch");
        let server_punch = match postcard::from_bytes::<UDPMessage<'_>>(&buffer[..read_bytes]) {
            Err(err) => die(format!("Got invalid packet from server: {}", err)),
            Ok(pkt) => pkt,
        };
        if matches!(
            server_punch,
            UDPMessage::Punch(PunchMessage::PeerHandshake1)
        ) {
            // NAT already punched
            // ... but we need to wait for last packet from server as well
            log::trace!("First handshake packet went through the NAT! A full-cone nat or no nat");
            continue;
        }
        if matches!(
            server_punch,
            UDPMessage::Punch(PunchMessage::PeerHandshake3)
        ) {
            // Last packet
            break;
        }
        die(format!("Server response is not ok: {:?}", server_punch));
    }
    // Done!
    return socket;
}
