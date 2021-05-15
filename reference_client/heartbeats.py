import websockets
import json


async def send_heartbeat():
    reader, writer = await asyncio.open_connection('192.168.133.22', 9000)

    writer.write(b"Heartbeat!")
    await writer.drain()

    writer.close()
    await writer.wait_closed()


class BeatingHearts():
    def __init__(self):
        self.websocket = None

    async def connect(self):
        self.websocket = await websockets.connect('ws://192.168.133.22:9001')

    async def receive(self):
        return json.loads(await self.websocket.recv())
