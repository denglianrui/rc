# RC (remote control)

This is remote control application, which goal is remote management the computers.

## architecture

It uses the  python language to write the server and client.
The server is on the public network, the client is on the private network.
The server can send commands to the client, and the client can execute the commands and send the results back to the server.
client can update the system information to the server.

## dependencies

- websockets asyncio psutil

