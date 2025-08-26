import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';
import { createProxyMiddleware } from 'http-proxy-middleware';
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const port = 3000;
const app = express();
app.use('/rpc', createProxyMiddleware({
  target: "http://34.220.83.153:7778/rpc",
  // target: "http://95.165.150.165:7777/rpc",
  changeOrigin: true,
  secure: false, // set to true if remote has valid TLS cert
  pathRewrite: { '^/rpc': '/' }, // strip /rpc prefix if needed by node
  onProxyReq: (proxyReq: any, req: Request, res: Response) => {
      // proxyReq.setHeader('Origin', 'http://95.165.150.165:7777');
      proxyReq.setHeader('Origin', 'http://34.220.83.153:7778');
    },
  } as any)
);

app.use(express.static(path.join(__dirname, '..')));

app.listen(port, () => {
  console.log(`Server is running at http://localhost:${port}`);
});