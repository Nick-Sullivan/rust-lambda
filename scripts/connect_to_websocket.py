import asyncio
import websockets


def main():
    asyncio.run(connect())


async def connect():
    uri = "wss://gg7rnclr3f.execute-api.eu-west-2.amazonaws.com/production"
    async with websockets.connect(uri) as websocket:
        await websocket.send("Hello, WebSocket!")
        while True:
            try:
                response = await websocket.recv()
                print(f"Received: {response}")
            except websockets.ConnectionClosed:
                print("Connection closed")
                break


if __name__ == "__main__":
    main()
