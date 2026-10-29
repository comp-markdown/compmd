// The editor's budgets, measured in Chrome (the installed one) with real key presses: for each document, from a key
// press to the frame painted, and the text area's input latency by the browser's Event Timing. `pnpm bench`; fails
// when a budget is exceeded, or when a document shows a diagnostic or a component problem it shouldn't.
//
// A key press is typed into a paragraph in the middle of the document, then deleted, in turn: 20 to warm up, then
// 200, one every 80 ms. Presses made while a change is processed are merged into the next, so the paint times count
// renders. It runs in a window of its own; `BENCH_HEADLESS=1` runs it headless.

import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { extname, join, normalize } from "node:path";
import { chromium, type Page } from "playwright-core";

const DIST = join(import.meta.dirname, "../dist");
const WARM_UP = 20;
const PRESSES = 200;
const EVERY = 80;
/** The menu's index of each document, and its budget from key press to paint, p95 in ms. */
const DOCUMENTS = [
  { name: "The spec's complete example", doc: 0, budget: 16 },
  { name: "The changelog", doc: 3, budget: 50 },
  { name: "Field notes (52 KB)", doc: 5, budget: 50 },
];
/** The text area's input latency, p95 in ms. */
const INPUT_BUDGET = 50;
const TYPES: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".wasm": "application/wasm",
};

const p95 = (values: readonly number[]): number => [...values].sort((a, b) => a - b)[Math.ceil(values.length * 0.95) - 1] ?? 0;

// The frame has an opaque origin, so everything it loads is cross-origin.
const server = createServer((request, response) => {
  const path = normalize(decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname));
  readFile(join(DIST, path === "/" ? "index.html" : path)).then(
    (body) => response.writeHead(200, { "access-control-allow-origin": "*", "content-type": TYPES[extname(path)] ?? "text/plain" }).end(body),
    () => response.writeHead(404).end(),
  );
});
await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
const { port } = server.address() as AddressInfo;

interface Measured {
  painted: number[];
  problems: string[];
  diagnostics: string[];
  interactions: number[];
}

/** What the page measured so far. */
async function measured(page: Page): Promise<Measured> {
  return page.evaluate(() => {
    type Editor = { state: { painted: { values: number[] }; problems: string[]; diagnostics: { code: string; message: string }[] } };
    const editor = (window as unknown as { compmdEditor: Editor }).compmdEditor;
    const latencies = (window as unknown as { latencies: Map<number, number> }).latencies;
    return {
      painted: [...editor.state.painted.values],
      problems: editor.state.problems,
      diagnostics: editor.state.diagnostics.map((d) => `${d.code} ${d.message}`),
      interactions: [...latencies.values()],
    };
  });
}

const over: string[] = [];
const browser = await chromium.launch({ channel: "chrome", headless: process.env.BENCH_HEADLESS === "1" });
for (const { name, doc, budget } of DOCUMENTS) {
  const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });
  // Event Timing reports interactions of 16 ms and more; each one's longest event is its latency.
  await page.addInitScript(() => {
    const latencies = new Map<number, number>();
    (window as unknown as { latencies: Map<number, number> }).latencies = latencies;
    new PerformanceObserver((list) => {
      for (const entry of list.getEntries() as PerformanceEventTiming[]) {
        if (entry.interactionId > 0) latencies.set(entry.interactionId, Math.max(latencies.get(entry.interactionId) ?? 0, entry.duration));
      }
    }).observe({ type: "event", durationThreshold: 16, buffered: true } as PerformanceObserverInit);
  });
  await page.goto(`http://127.0.0.1:${String(port)}/index.html?doc=${String(doc)}`);
  await page.waitForFunction(() => ((window as { compmdEditor?: { state: { painted: { count: number } } } }).compmdEditor?.state.painted.count ?? 0) > 0, null, { timeout: 60_000 });
  // The caret before the end of the sentence the middle of the text is in.
  await page.evaluate(() => {
    const textarea = document.querySelector("textarea");
    if (!textarea) return;
    const at = textarea.value.indexOf(".", textarea.value.length / 2);
    textarea.focus();
    textarea.setSelectionRange(at, at);
  });
  /** How many renders the warm-up made, which the result leaves out. */
  let warm = 0;
  for (let press = 0; press < WARM_UP + PRESSES; press++) {
    if (press === WARM_UP) {
      await page.waitForTimeout(500);
      warm = (await measured(page)).painted.length;
    }
    await page.keyboard.press(press % 2 === 0 ? "x" : "Backspace");
    await page.waitForTimeout(EVERY);
  }
  await page.waitForTimeout(500);
  const result = await measured(page);
  const painted = result.painted.slice(warm);
  const latency = p95([...result.interactions, ...Array<number>(Math.max(0, PRESSES - result.interactions.length)).fill(0)]);
  const paintedOk = p95(painted) <= budget;
  const inputOk = latency <= INPUT_BUDGET;
  if (!paintedOk) over.push(`${name}: key press to paint`);
  if (!inputOk) over.push(`${name}: input latency`);
  const clean = result.problems.length === 0 && result.diagnostics.length === 0;
  if (!clean) over.push(`${name}: ${[...result.diagnostics, ...result.problems].join("; ")}`);
  const input = latency === 0 ? "under 16 ms" : `${latency.toFixed(0)} ms`;
  console.log(
    `${name}: key press to paint p95 ${p95(painted).toFixed(1)} ms (n ${String(painted.length)}), budget ${String(budget)} ms: ${paintedOk ? "ok" : "OVER BUDGET"}; input latency p95 ${input}, budget ${String(INPUT_BUDGET)} ms: ${inputOk ? "ok" : "OVER BUDGET"}${clean ? "" : "; PROBLEMS"}`,
  );
  await page.close();
}
await browser.close();
server.close();
console.log(`Chrome ${browser.version()}`);
if (over.length > 0) {
  console.log(`\nFailed: ${over.join("; ")}`);
  process.exitCode = 1;
}
