import asyncio
import websockets
import json


def main():
    CLOUD_URL = "wss://bgxmeyu48f.execute-api.eu-west-2.amazonaws.com/production"
    # LOCAL_URL = "ws://127.0.0.1:8080/ws/"
    asyncio.run(run_tests(CLOUD_URL))


async def run_tests(url: str):
    session_id = await connect(url)
    await play_single_game(url, session_id)
    print("done")


async def connect(url: str) -> str:
    async with websockets.connect(url) as websocket:
        await websocket.send(json.dumps({"action": "getSession", "data": {}}))
        response = await websocket.recv()
        print(response)
        response_json = json.loads(response)
        session_id = response_json["data"]
    return session_id


async def play_single_game(url: str, session_id: str):
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
        response = await websocket.recv()
        print(response)
        response = await websocket.recv()
        print(response)
        await websocket.send(
            json.dumps(
                {
                    "action": "rollDice",
                    "data": {
                        "sessionId": session_id,
                    },
                }
            )
        )
        response = await websocket.recv()
        print(response)
        await websocket.send(
            json.dumps(
                {
                    "action": "newRound",
                    "data": {
                        "sessionId": session_id,
                    },
                }
            )
        )
        response = await websocket.recv()
        print(response)

        while True:
            try:
                response = await websocket.recv()
                print(f"Received: {response}")
            except websockets.ConnectionClosed:
                print("Connection closed")
                break


if __name__ == "__main__":
    main()
