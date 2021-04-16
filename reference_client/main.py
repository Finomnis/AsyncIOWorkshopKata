import logging
import asyncio
import websockets
import json
from aioconsole import ainput

logger = logging.getLogger(__name__)

receivers = []


async def heartbeat_sender():
    while True:
        reader, writer = await asyncio.open_connection('192.168.133.22', 9000)

        writer.write(b"Heartbeat!")
        await writer.drain()

        writer.close()
        await writer.wait_closed()

        await asyncio.sleep(1)


async def get_nodes():
    async with websockets.connect('ws://192.168.133.22:9001') as websocket:
        while True:
            nodes = json.loads(await websocket.recv())
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
            async with websockets.connect(f'ws://{node}:9002') as websocket:
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
