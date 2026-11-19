// Runs the test page in Playwright's WebKit and Chromium, once per variant, and writes what each measured to
// `results.json`, with a table on the console. Build the page first (`pnpm measure` does both).
//
//   node measure.ts [repeats] [variant…]     # each variant `repeats` times (1 by default), all variants without names
//   node measure.ts 1 today --runs=40        # a smaller page, to try the runner
//   node measure.ts 3 churn --out=x.json     # written to `x.json` here, not `results.json`

import { readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { extname, join } from "node:path";
import { chromium, webkit, type BrowserType } from "playwright";

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
  // A large host document, and 100 pages opened and closed in it, without highlighting (CMD-253): code only, then
  // each in a box with rules of its own.
  churn: "highlight=off&churn=1",
  "churn-styled": "highlight=off&churn=1&styled=1",
  // The same document, one sheet, a block's rules written as a whole sheet, rule by rule, or not at all (CMD-254).
  "sheet-replace": "highlight=off&churn=1&sheet=replace",
  "sheet-insert": "highlight=off&churn=1&sheet=insert",
  "sheet-none": "highlight=off&churn=1&sheet=none",
  "churn-styled-nowrite": "highlight=off&churn=1&styled=1&nowrite=1",
};

const options = process.argv.slice(2).filter((argument) => argument.startsWith("--"));
const [repeatsArgument, ...named] = process.argv.slice(2).filter((argument) => !argument.startsWith("--"));
const repeats = Number(repeatsArgument ?? 1) || 1;
const chosen = named.length > 0 ? named : Object.keys(VARIANTS);
const runsOption = options.find((option) => option.startsWith("--runs="));
const runsQuery = runsOption ? `&runs=${runsOption.slice("--runs=".length)}` : "";

const server = createServer((request, response) => {
  const path = new URL(request.url ?? "/", "http://localhost").pathname;
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
  const browser = await type.launch({ headless: true });
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
