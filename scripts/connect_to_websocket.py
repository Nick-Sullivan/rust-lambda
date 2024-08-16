import asyncio
import websockets
import json


def main():
    asyncio.run(connect())


async def connect():
    uri = "wss://k6lgst3usl.execute-api.eu-west-2.amazonaws.com/production"
    async with websockets.connect(uri) as websocket:
        await websocket.send(json.dumps({"action": "getSession", "data": {}}))
        response = await websocket.recv()
        print(response)
        response_json = json.loads(response)
        session_id = response_json["data"]

    async with websockets.connect(uri) as websocket:
        await websocket.send(
            json.dumps({"action": "setSession", "data": {"sessionId": session_id}})
        )
        await websocket.send(
            json.dumps(
                {
                    "action": "setNickname",
                    "data": {
                        "sessionId": session_id,
                        "nickname": "nick",
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
