import asyncio
from websockets.server import serve


class RemoteControlServer:
    def __init__(self, host="0.0.0.0", port=8765):
        self.host = host
        self.port = port
        self.clients = set()

    async def handle_client(self, websocket, path):
        self.clients.add(websocket)
        try:
            async for message in websocket:
                print(f"Received message: {message}")
                # Echo back for now
                await websocket.send(f"Server received: {message}")
        finally:
            self.clients.remove(websocket)

    async def start(self):
        server = await serve(self.handle_client, self.host, self.port)
        print(f"Server running on ws://{self.host}:{self.port}")
        await server.wait_closed()


if __name__ == "__main__":
    server = RemoteControlServer()
    asyncio.run(server.start())
