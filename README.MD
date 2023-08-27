# P2P UDP Puncher
Connect two UDP applications behind NAT without the need of additional configurations.

## Requirements
* Both parties behind NAT
* No Symmetric NAT
* A server reachable from both clients. This works as TURN server

## Features
* **Very Light**: Single threaded when ran as TURN server and written in Rust with NO HEAP ALLOCATIONS!
* **Small Overhead**: Most of the packets are 2 bytes each. Keep alive packets are 1 byte.
* **Works on Top of Other Programs**: You don't need to change the code of other programs to use this program. Just change the destination address in them.

## How it works

There are three components in each established connections:
1. A client which initiates the connection. Must be behind NAT.
2. Server which accepts the connections from clients. Must be behind NAT.
3. A TURN server which connects clients with servers. This server must be reachable from all client and servers.

And the flow of a client connecting to a server follows:
1. A TURN server is set up.
2. Server creates a socket to TURN server. This will create a entry in its NAT. A keep alive packet is sent to TURN in order to keep the entry in NAT.
3. A client connects to TURN server and requests the address of the server based on a key. TURN will give the address of the server to the client.
4. TURN notifies the server with the address of the client.
5. A three step handshake is then initiated:
    1. Server sends a packet to client. This punches the NAT of the server.
    2. Client does not receive the first packet of the server unless it is behind a Full-Cone NAT or no NAT at all. Thus, after one second, a packet is sent to server in order to punch the client's NAT.
    3. The packet from the client is received in server because of the punched NAT. Server finally responds with a last packet and the handshake is done.
6. Server and client both proxy the connection of their socket to each other.
7. Server then starts another socket and registers it in TURN server in order to accept other clients as well.

## Usage

This program uses [env_logger](https://docs.rs/env_logger/latest/env_logger/) to log. You can configure the log level by environment variables like this before running the program: `export RUST_LOG="p2p_udp_puncher=trace"`.

You can also run each subcommand without any arguments to get a help message.

### TURN

Running the TURN server is easy as hell. You just need to specify the listening address.

```bash
./p2p_udp_puncher turn 0.0.0.0:12345
```

Above command runs a TURN server on `0.0.0.0:12345`.

### Server

For running a server you need a key (which clients need to supply as well), the address of TURN server and the address to forward the packets of incoming clients to.

```bash
./p2p_udp_puncher server 127.0.0.1:1984 1.1.1.1:12345 test
```

Above commands runs a server. The packets of clients are forwarded to `127.0.0.1:1984`, TURN server used is located at `1.1.1.1:12345` and the key that clients use in order to connect is `test`.

### Client

For running a client you need the address of the TURN server, the key which server gave you and a local address to listen for incoming packets in order to forward them to server.

```bash
./p2p_udp_puncher client 127.0.0.1:54321 1.1.1.1:12345 test
```

Above commands runs a server. Incoming packets are expected to be sent to `127.0.0.1:54321`, TURN server used is located at `1.1.1.1:12345` and the key that server gave you is `test`.