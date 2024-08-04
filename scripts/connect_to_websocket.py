import asyncio
import websockets
import json


def main():
    asyncio.run(connect())


async def connect():
    uri = "wss://6wgg3w1q3k.execute-api.eu-west-2.amazonaws.com/production"
    async with websockets.connect(uri) as websocket:
        await websocket.send(
            json.dumps(
                {
                    "action": "setSession",
                    "data": {
                        "sessionId": "sessionId",
                    },
                }
            )
        )
        while True:
            try:
                response = await websocket.recv()
                print(f"Received: {response}")
            except websockets.ConnectionClosed:
                print("Connection closed")
                break


if __name__ == "__main__":
    main()
