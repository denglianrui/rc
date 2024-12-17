import asyncio
import websockets
import psutil
import json
import subprocess
from websockets import client
import websockets.client


class RemoteControlClient:
    def __init__(self, server_url="ws://localhost:8765"):
        self.server_url = server_url

    async def execute_command(self, command):
        try:
            process = await asyncio.create_subprocess_shell(
                command,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE
            )
            stdout, stderr = await process.communicate()
            return {
                'stdout': stdout.decode() if stdout else '',
                'stderr': stderr.decode() if stderr else '',
                'return_code': process.returncode
            }
        except Exception as e:
            return {'error': str(e)}

    async def send_system_info(self, websocket):
        while True:
            system_info = {
                'cpu_percent': psutil.cpu_percent(),
                'memory_percent': psutil.virtual_memory().percent,
                'disk_usage': psutil.disk_usage('/').percent
            }
            await websocket.send(json.dumps({
                'type': 'system_info',
                'data': system_info
            }))
            await asyncio.sleep(5)

    async def start(self):
        async with websockets.client.connect(self.server_url) as websocket:
            print(f"Connected to {self.server_url}")
            system_info_task = asyncio.create_task(self.send_system_info(websocket))
            
            try:
                async for message in websocket:
                    data = json.loads(message)
                    if data.get('type') == 'command':
                        result = await self.execute_command(data['data'])
                        await websocket.send(json.dumps({
                            'type': 'command_result',
                            'data': result
                        }))
            finally:
                system_info_task.cancel()


if __name__ == "__main__":
    client = RemoteControlClient()
    asyncio.run(client.start())
