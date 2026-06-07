import websocket

ws = websocket.WebSocket()
ws.connect("ws://127.0.0.1:10000")

print("Connected to ws://127.0.0.1:10000")

while True:
    msg = input("> ")
    ws.send(msg)
    reply = ws.recv()
    print(reply)
