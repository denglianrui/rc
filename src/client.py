import asyncio
import websockets
import psutil
import json
from websockets import client
import websockets.client


class RemoteControlClient:
    def __init__(self, server_url="ws://localhost:8765"):
        self.server_url = server_url

    async def send_system_info(self, websocket):
        while True:
            system_info = {
                "cpu_percent": psutil.cpu_percent(),
                "memory_percent": psutil.virtual_memory().percent,
                "disk_usage": psutil.disk_usage("/").percent,
            }
            await websocket.send(json.dumps(system_info))
            await asyncio.sleep(5)

    async def start(self):
        async with websockets.client.connect(self.server_url) as websocket:
            print(f"Connected to {self.server_url}")
            await self.send_system_info(websocket)


if __name__ == "__main__":
    client = RemoteControlClient()
    asyncio.run(client.start())
