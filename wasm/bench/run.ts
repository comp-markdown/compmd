// The processor's budgets, measured: the module's gzipped size, and the time of `process` in a worker in Chrome (the
// installed one) and WebKit (Playwright's) for three documents. `pnpm bench`; fails when a budget is exceeded.
// WebKit needs `pnpm exec playwright-core install webkit` once.

import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { extname, join, normalize } from "node:path";
import { gzipSync } from "node:zlib";
import { chromium, webkit, type Browser } from "playwright-core";
import { changelog, completeExample, longBody } from "./requests.ts";

const PACKAGE = join(import.meta.dirname, "..");
/** 1 MB, gzipped. */
const SIZE_BUDGET = 1024 * 1024;
/** p95 in ms, from one frame (16 ms) and from 50 ms between key press and paint, less the renderer's 2 ms. */
const DOCUMENTS = [
  { name: "complete example", budget: 14, request: completeExample() },
  { name: "changelog", budget: 48, request: changelog() },
  { name: "long body", budget: 48, request: longBody() },
];
const BROWSERS: [string, () => Promise<Browser>][] = [
  ["Chrome", () => chromium.launch({ channel: "chrome" })],
  ["WebKit", () => webkit.launch()],
];
const TYPES: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".wasm": "application/wasm",
  ".json": "application/json",
};

interface Results {
  error?: string;
  userAgent: string;
  isolated: boolean;
  startup: number;
  results: { name: string; p50: number; p95: number }[];
}

/** What went over its budget. */
const over: string[] = [];
const verdict = (what: string, within: boolean) => {
  if (!within) over.push(what);
  return within ? "ok" : "OVER BUDGET";
};

const size = gzipSync(readFileSync(join(PACKAGE, "pkg/compmd_wasm_bg.wasm")), { level: 9 }).length;
console.log(`Size: ${(size / 1024).toFixed(0)} KB gzipped, budget ${String(SIZE_BUDGET / 1024)} KB: ${verdict("size", size <= SIZE_BUDGET)}`);

const documents = JSON.stringify(DOCUMENTS.map(({ name, request }) => ({ name, request: { ...request, origins: true } })));

// Cross-origin isolated, so the worker's timers are fine-grained.
const server = createServer((request, response) => {
  const path = normalize(decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname));
  const headers = { "cross-origin-opener-policy": "same-origin", "cross-origin-embedder-policy": "require-corp" };
  if (path === "/documents.json") {
    response.writeHead(200, { ...headers, "content-type": "application/json" }).end(documents);
    return;
  }
  readFile(join(PACKAGE, path)).then(
    (body) => response.writeHead(200, { ...headers, "content-type": TYPES[extname(path)] ?? "text/plain" }).end(body),
    () => response.writeHead(404).end(),
  );
});
await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
const { port } = server.address() as AddressInfo;

for (const [name, launch] of BROWSERS) {
  const browser = await launch();
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${String(port)}/bench/index.html`);
  const handle = await page.waitForFunction(() => (window as { results?: Results }).results, null, { timeout: 600_000 });
  const measured = (await handle.jsonValue()) as Results;
  await browser.close();
  if (measured.error) throw new Error(`${name}: ${measured.error}`);
  console.log(`\n${name} (${measured.userAgent}), isolated: ${String(measured.isolated)}, start-up ${measured.startup.toFixed(1)} ms`);
  for (const { name: document, p50, p95 } of measured.results) {
    const budget = DOCUMENTS.find((d) => d.name === document)?.budget ?? 0;
    console.log(`  ${document}: p50 ${p50.toFixed(2)} ms, p95 ${p95.toFixed(2)} ms, budget ${String(budget)} ms: ${verdict(`${name}, ${document}`, p95 <= budget)}`);
  }
}
server.close();
if (over.length > 0) {
  console.log(`\nOver budget: ${over.join("; ")}`);
  process.exitCode = 1;
}
