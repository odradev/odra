import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const port = 3000;
const app = express();

app.use(express.static(path.join(__dirname, '..')));
app.listen(port, () => {
  console.log(`Server is running at http://localhost:${port}`);
});
