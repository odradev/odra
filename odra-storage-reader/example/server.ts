// Serves the storage explorer (index.html) and proxies the node RPC, which sends no CORS headers.
//
//   npm run serve                      # http://localhost:3000
//
// NODE_URL        the node RPC (default: NCTL)
// CONTRACTS_FILE  the contracts deployed with odra-cli (default: <repo>/resources/casper-net-1-contracts.toml)
// PORT            the port of the page (default: 3000)
import express from "express";
import { createProxyMiddleware } from "http-proxy-middleware";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const PORT = Number(process.env.PORT ?? 3000);
const NODE_URL = new URL(process.env.NODE_URL ?? "http://localhost:11101/rpc");
const CONTRACTS_FILE =
  process.env.CONTRACTS_FILE ?? path.join(here, "../../resources/casper-net-1-contracts.toml");
// Written by `just export-layouts`.
const LAYOUTS_DIR = path.join(here, "layouts");

interface Contract {
  name: string;
  packageHash: string;
  hasLayout: boolean;
}

// Reads the `[[contracts]]` tables odra-cli writes; they only hold string values.
function readContracts(): Contract[] {
  const tables: Record<string, string>[] = [];
  for (const line of readFileSync(CONTRACTS_FILE, "utf8").split("\n")) {
    const trimmed = line.trim();
    if (trimmed === "[[contracts]]") {
      tables.push({});
      continue;
    }
    const entry = /^(\w+)\s*=\s*"(.*)"$/.exec(trimmed);
    if (entry && tables.length > 0) {
      tables[tables.length - 1][entry[1]] = entry[2];
    }
  }
  return tables.map((table) => ({
    name: table.name,
    packageHash: table.package_hash,
    hasLayout: existsSync(path.join(LAYOUTS_DIR, `${table.name}.json`))
  }));
}

const app = express();

app.get("/api/contracts", (_req, res) => {
  try {
    res.json(readContracts());
  } catch (e) {
    res.status(500).json({ error: `Cannot read ${CONTRACTS_FILE}: ${(e as Error).message}` });
  }
});

app.use(
  createProxyMiddleware({
    target: NODE_URL.origin,
    pathFilter: "/rpc",
    pathRewrite: { "^/rpc": NODE_URL.pathname },
    changeOrigin: true
  })
);

app.use(express.static(here));

app.listen(PORT, () => {
  console.log(`Storage explorer: http://localhost:${PORT}`);
  console.log(`Node:             ${NODE_URL}`);
  console.log(`Contracts:        ${CONTRACTS_FILE}`);
});
