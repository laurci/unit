import { WebSocket } from "ws";

const URL = "ws://127.0.0.1:6447/ws?app=hello-world";

async function openWebSocket(onMessage: (msg: Buffer | string) => any) {
  return new Promise<WebSocket>((resolve, reject) => {
    const ws = new WebSocket(URL);
    ws.binaryType = "nodebuffer";

    ws.onopen = () => {
      ws.onmessage = (msg) => {
        onMessage(msg.data as string | Buffer);
      };

      ws.onclose = () => {
        console.log("websocket closed");
      };

      resolve(ws);
    };

    ws.onerror = (err) => {
      reject(err);
    };
  });
}

function isBinaryMessage(msg: Buffer | string): msg is Buffer {
  return msg instanceof Buffer;
}

async function main() {
  console.time("openWebSocket");

  const sock = await openWebSocket((msg) => {
    if (isBinaryMessage(msg)) {
      console.log("recv binary message:", msg);
    } else {
      console.log("recv text message:", msg);
    }
  });

  setInterval(() => {
    const message = "hello!";
    console.log("sending:", message);
    sock.send(message);
  }, 2000);

  console.timeEnd("openWebSocket");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
