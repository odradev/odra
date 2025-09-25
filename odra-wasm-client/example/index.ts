import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';
import { createProxyMiddleware } from 'http-proxy-middleware';
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const port = 3000;
const app = express();

app.use(
  '/rpc',
  createProxyMiddleware({
    target: 'http://95.165.150.165:7777/rpc',
    changeOrigin: true,
  })
);
app.use(
  '/speculative/rpc',
  createProxyMiddleware({
    target: 'http://34.220.83.153:7778/rpc',
    changeOrigin: true,
  })
);

app.use(express.static(path.join(__dirname, '..')));
app.listen(port, () => {
  console.log(`Server is running at http://localhost:${port}`);
});
