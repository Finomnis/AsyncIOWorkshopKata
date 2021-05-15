import logging
import asyncio
import websockets
import json
from aioconsole import ainput

from heartbeats import send_heartbeat, BeatingHearts

logger = logging.getLogger(__name__)

receivers = []


async def heartbeat_sender():
    while True:
        send_heartbeat()
        await asyncio.sleep(1)


async def get_nodes():
    beating_hearts = BeatingHearts()
    await beating_hearts.connect()
    while True:
        nodes = await beating_hearts.receive()
        logger.info(f"Nodes: {nodes}")
        global receivers
        receivers = nodes


async def connection_handler(socket, _path):
    try:
        while True:
            msg = await socket.recv()
            parsed = json.loads(msg)
            sender = parsed["sender"]
            message = parsed["message"]
            print(f"{sender}: {message}")
    except websockets.exceptions.ConnectionClosedOK:
        pass
    except Exception as e:
        logger.warn(f"Disconnected: {e}")


async def receive():
    await websockets.serve(connection_handler, port=9002)


async def send_from_command_line():
    while True:
        msg = await ainput()
        msg_json = json.dumps({"sender": "me", "message": msg})
        for node in receivers:
            async with websockets.connect(f'ws://192.168.133.202:9002') as websocket:
                await websocket.send(msg_json)


async def main():
    await asyncio.gather(
        heartbeat_sender(),
        get_nodes(),
        receive(),
        send_from_command_line(),
    )

if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(main())
