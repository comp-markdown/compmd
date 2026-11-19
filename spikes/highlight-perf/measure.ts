// Runs the test page in Playwright's WebKit and Chromium, once per variant, and writes what each measured to
// `results.json`, with a table on the console. Build the page first (`pnpm measure` does both).
//
//   node measure.ts [repeats] [variant…]     # each variant `repeats` times (1 by default), all variants without names
//   node measure.ts 1 today --runs=40        # a smaller page, to try the runner
//   node measure.ts 3 churn --out=x.json     # written to `x.json` here, not `results.json`
//   node measure.ts 3 framepage-host-10 --site-per-process   # Chromium's frames each in a process of their own

import { readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join } from "node:path";
import { chromium, webkit, type BrowserType, type Page } from "playwright";

const DIST = join(import.meta.dirname, "dist");
const TYPES: Record<string, string> = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".wasm": "application/wasm", ".json": "application/json" };

/** Each variant, by name: its query. The single choices, then the combinations they suggest. */
const VARIANTS: Record<string, string> = {
  // Every run kept mounted, a page each in one document: what that costs with and without highlighting.
  "off-kept": "highlight=off&keep=1",
  "shared-kept": "shiki=shared&engine=js&where=main&schedule=eager&keep=1",
  off: "highlight=off",
  today: "shiki=per-view&engine=js&where=main&schedule=eager",
  shared: "shiki=shared&engine=js&where=main&schedule=eager",
  sliced: "shiki=per-view&engine=js&where=main&schedule=sliced",
  precompiled: "shiki=per-view&engine=precompiled&where=main&schedule=eager",
  worker: "shiki=shared&engine=js&where=worker&schedule=eager",
  oniguruma: "shiki=per-view&engine=oniguruma&where=main&schedule=eager",
  cache: "shiki=shared&engine=js&where=main&schedule=eager&cache=1",
  "shared+precompiled": "shiki=shared&engine=precompiled&where=main&schedule=eager",
  "shared+oniguruma": "shiki=shared&engine=oniguruma&where=main&schedule=eager",
  "worker+oniguruma": "shiki=shared&engine=oniguruma&where=worker&schedule=eager",
  "shared+sliced": "shiki=shared&engine=js&where=main&schedule=sliced",
  "shared+sliced+precompiled": "shiki=shared&engine=precompiled&where=main&schedule=sliced",
  "shared+sliced+precompiled+cache": "shiki=shared&engine=precompiled&where=main&schedule=sliced&cache=1",
  "shared+sliced+oniguruma+cache": "shiki=shared&engine=oniguruma&where=main&schedule=sliced&cache=1",
  "worker+sliced+precompiled+cache": "shiki=shared&engine=precompiled&where=worker&schedule=sliced&cache=1",
  // The sandbox's own highlighter, a Shiki for each page as before CMD-256, and one for all as since.
  "sandbox-per-view": "sandbox=per-view",
  "sandbox-shared": "sandbox=shared",
  // And with Oniguruma where the page compiles WebAssembly, as a host-mode page since CMD-257.
  "sandbox-shared-wasm": "sandbox=shared&wasm=1",
  // A large host document, and 100 pages opened and closed in it, without highlighting (CMD-253): code only, then
  // each in a box with rules of its own.
  churn: "highlight=off&churn=1",
  "churn-styled": "highlight=off&churn=1&styled=1",
  // The same document, one sheet, a block's rules written as a whole sheet, rule by rule, or not at all (CMD-254).
  "sheet-replace": "highlight=off&churn=1&sheet=replace",
  "sheet-insert": "highlight=off&churn=1&sheet=insert",
  "sheet-none": "highlight=off&churn=1&sheet=none",
  "churn-styled-nowrite": "highlight=off&churn=1&styled=1&nowrite=1",
  // Twelve frames, each highlighting its first block with a highlighter of its own (CMD-260).
  "frames-js": "highlight=off&churn=1&frames=12",
  "frames-wasm": "highlight=off&churn=1&frames=12&wasm=1",
  // A page of 1, 10 and 40 documents in frame mode, each frame from another origin highlighting five blocks: with a
  // highlighter of its own, as today, or asking the page, which highlights for all of them with one (CMD-267).
  "framepage-frame-1": "highlight=off&churn=1&framepage=1&where=frame",
  "framepage-frame-10": "highlight=off&churn=1&framepage=10&where=frame",
  "framepage-frame-40": "highlight=off&churn=1&framepage=40&where=frame",
  "framepage-host-1": "highlight=off&churn=1&framepage=1&where=host",
  "framepage-host-10": "highlight=off&churn=1&framepage=10&where=host",
  "framepage-host-40": "highlight=off&churn=1&framepage=40&where=host",
};

/**
 * The document each frame of `?framepage=N` loads, from `127.0.0.1` while the page is `localhost`'s, so it is
 * cross-origin, as the sandbox's frames are: it imports `frame.js` and highlights the blocks the page sends it.
 */
const FRAME_PAGE = `<!doctype html>
<script type="module">
  import { highlightBlocks } from "/frame.js";
  addEventListener("message", async ({ data, ports }) => {
    parent.postMessage(await highlightBlocks(data.blocks, data.askThePage ? ports[0] : undefined), "*");
  }, { once: true });
  parent.postMessage("ready", "*");
</script>
`;

/**
 * In Chromium, the JavaScript heap the page and its frames use, in MB, after a collection: each cross-origin frame is
 * a process of its own, with a heap of its own, and the page's session has the page's. WebKit has no such measure.
 */
async function heapMb(page: Page): Promise<{ heapMb: number; heapsRead: number }> {
  let bytes = 0;
  let heapsRead = 0;
  for (const frame of page.frames()) {
    // A frame in the page's process has no session of its own: its heap is the page's, counted with it.
    const session = await page
      .context()
      .newCDPSession(frame === page.mainFrame() ? page : frame)
      .catch(() => undefined);
    if (!session) continue;
    await session.send("HeapProfiler.collectGarbage");
    bytes += (await session.send("Runtime.getHeapUsage")).usedSize;
    heapsRead++;
    await session.detach();
  }
  return { heapMb: Math.round(bytes / 1024 / 1024), heapsRead };
}

const options = process.argv.slice(2).filter((argument) => argument.startsWith("--"));
const [repeatsArgument, ...named] = process.argv.slice(2).filter((argument) => !argument.startsWith("--"));
const repeats = Number(repeatsArgument ?? 1) || 1;
const chosen = named.length > 0 ? named : Object.keys(VARIANTS);
const runsOption = options.find((option) => option.startsWith("--runs="));
const runsQuery = runsOption ? `&runs=${runsOption.slice("--runs=".length)}` : "";

const server = createServer((request, response) => {
  const path = new URL(request.url ?? "/", "http://localhost").pathname;
  if (path === "/frame-page.html") {
    response.writeHead(200, { "content-type": "text/html" }).end(FRAME_PAGE);
    return;
  }
  try {
    const file = join(DIST, path === "/" ? "index.html" : path);
    response.writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream" }).end(readFileSync(file));
  } catch {
    response.writeHead(404).end();
  }
});
await new Promise<void>((resolve) => server.listen(4290, resolve));

const BROWSERS: [string, BrowserType][] = [
  ["webkit", webkit],
  ["chromium", chromium],
];

const results: unknown[] = [];
for (const [browserName, type] of BROWSERS) {
  // `--site-per-process`: Chromium gives each cross-site frame a process of its own, as desktop Chrome does.
  const isolated = type === chromium && options.includes("--site-per-process");
  const browser = await type.launch({ headless: true, ...(isolated ? { args: ["--site-per-process"] } : {}) });
  for (const name of chosen) {
    const query = VARIANTS[name];
    if (query === undefined) throw new Error(`no variant ${name}`);
    for (let repeat = 0; repeat < repeats; repeat++) {
      const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
      page.on("console", (message) => {
        console.log(`  [${browserName} ${name}] ${message.text()}`);
      });
      page.on("pageerror", (error) => {
        console.log(`  [${browserName} ${name}] error: ${error.message}`);
      });
      await page.goto(`http://localhost:4290/?${query}${runsQuery}`);
      const churn = query.includes("churn=1");
      const result = (await page.evaluate((churns) => (churns ? window.spike.churn() : window.spike.run()), churn)) as Record<string, unknown>;
      if (query.includes("framepage=") && browserName === "chromium") Object.assign(result, await heapMb(page));
      await page.close();
      results.push({ browser: browserName, variant: name, repeat, ...result });
      if (churn) {
        console.log([browserName, name.padEnd(32), JSON.stringify(result)].join("  "));
        continue;
      }
      const { open, skim, revisit, costs } = result as Record<string, Record<string, unknown>>;
      console.log(
        [browserName, name.padEnd(32), `open ${String(open?.blockedMs)}ms/${String(open?.longestMs)}ms`, `skim ${String(skim?.blockedMs)}ms/${String(skim?.longestMs)}ms (${String(skim?.longFrames)} long)`, `revisit ${String(revisit?.blockedMs)}ms`, `wait ${String(open?.medianWaitMs)}/${String(skim?.medianWaitMs)}ms`, `tok ${String((costs?.total as Record<string, unknown>).medianTokenizeMs)}ms first ${String((costs?.total as Record<string, unknown>).firstTokenizeMs)}ms`].join("  "),
      );
    }
  }
  await browser.close();
}
server.close();
const outOption = options.find((option) => option.startsWith("--out="));
writeFileSync(join(import.meta.dirname, outOption ? outOption.slice("--out=".length) : "results.json"), `${JSON.stringify(results, null, 2)}\n`);
