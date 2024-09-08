import asyncio
import websockets
import json


def main():
    CLOUD_URL = "wss://q56zk3w0df.execute-api.eu-west-2.amazonaws.com/production"
    LOCAL_URL = "ws://127.0.0.1:8080/ws/"
    asyncio.run(connect(CLOUD_URL))


async def connect(url: str):
    # async with websockets.connect(url) as websocket:
    #     await websocket.send("Hello")
    #     response = await websocket.recv()
    #     print(response)

    async with websockets.connect(url) as websocket:
        await websocket.send(json.dumps({"action": "getSession", "data": {}}))
        response = await websocket.recv()
        print(response)
        response_json = json.loads(response)
        session_id = response_json["data"]

    async with websockets.connect(url) as websocket:
        await websocket.send(
            json.dumps({"action": "setSession", "data": {"sessionId": session_id}})
        )
        response = await websocket.recv()
        print(response)
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
        response = await websocket.recv()
        print(response)
        await websocket.send(
            json.dumps(
                {
                    "action": "createGame",
                    "data": {
                        "sessionId": session_id,
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
