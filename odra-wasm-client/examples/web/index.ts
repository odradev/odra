import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';
import { createProxyMiddleware } from 'http-proxy-middleware';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const port = 3000;

app.use(express.static(path.join(__dirname, '..')));

app.listen(port, () => {
  console.log(`Server is running at http://localhost:${port}`);
});


// ----- RPC proxy server -----
// This proxy must bind to the exact host:port your wasm client uses (95.165.150.165:7777)
// Set RPC_TARGET to the real node you want to talk to (can be an SSH tunnel endpoint)
const RPC_BIND_HOST = process.env.RPC_BIND_HOST || '95.165.150.165';
const RPC_BIND_PORT = Number(process.env.RPC_BIND_PORT || 7777);
const RPC_TARGET = process.env.RPC_TARGET || 'http://127.0.0.1:8888'; // replace with real node or tunnel

const proxyApp = express();

proxyApp.use(
  '/',
  createProxyMiddleware({
    target: RPC_TARGET,
    changeOrigin: true,
    ws: true,
  })
);

proxyApp.listen(RPC_BIND_PORT, RPC_BIND_HOST, () => {
  console.log(`RPC proxy listening on http://${RPC_BIND_HOST}:${RPC_BIND_PORT} -> ${RPC_TARGET}`);
});