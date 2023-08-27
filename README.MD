# P2P UDP Puncher
Connect two UDP applications behind NAT without the need of additional configurations.

## Requirements
* Both parties behind NAT
* No Symmetric NAT
* A server reachable from both clients. This works as TURN server

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
6. Server then starts another socket and registers it in TURN server in order to accept other clients as well.

## Usage

### TURN

### Server

### Client
