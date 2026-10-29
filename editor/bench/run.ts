// The editor's budgets, measured in Chrome (the installed one) with real key presses: for each document, from a key
// press to the frame painted, and the text area's input latency by the browser's Event Timing. `pnpm bench`; fails
// when a budget is exceeded, or when a document shows a diagnostic or a component problem it shouldn't.
//
// A key press is typed at each edit point, then deleted, in turn: 20 to warm up, then 200, one every 80 ms. Presses
// made while a change is processed are merged into the next, so the paint times count renders. It runs in a window
// of its own; `BENCH_HEADLESS=1` runs it headless.

import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { extname, join, normalize } from "node:path";
import { chromium, type Page } from "playwright-core";

const DIST = join(import.meta.dirname, "../dist");
const WARM_UP = 20;
const PRESSES = 200;
const EVERY = 80;
/**
 * Where each document is typed into: its id in the menu, the text the caret goes into (after its first four
 * characters), and the budget from key press to paint, p95 in ms. Typing in a paragraph processes that block; typing
 * in a definition processes the whole document again, and in a host-language one, compiles it again too.
 */
const EDITS = [
  { name: "The spec's complete example, in a paragraph", doc: "complete", at: "This page renders live data", budget: 16 },
  { name: "The spec's complete example, in the sparkline's JSX", doc: "complete", at: "compmd.fallback.text", budget: 16 },
  { name: "The changelog, in its heading", doc: "changelog", at: "# RayPlay changelog", budget: 50 },
  { name: "The changelog, in a definition it uses 384 times", doc: "changelog", at: "c.release == release && c.type == kind)) as=c", budget: 50 },
  { name: "Field notes (52 KB), in a paragraph", doc: "notes", at: "Seeds order the early light", budget: 50 },
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
  processing: number[];
  rendering: number[];
  problems: string[];
  diagnostics: string[];
  interactions: number[];
}

/** What the page measured so far. */
async function measured(page: Page): Promise<Measured> {
  return page.evaluate(() => {
    type Series = { values: number[] };
    type Editor = { state: { painted: Series; processing: Series; rendering: Series; problems: string[]; diagnostics: { code: string; message: string }[] } };
    const editor = (window as unknown as { compmdEditor: Editor }).compmdEditor;
    const latencies = (window as unknown as { latencies: Map<number, number> }).latencies;
    return {
      painted: [...editor.state.painted.values],
      processing: [...editor.state.processing.values],
      rendering: [...editor.state.rendering.values],
      problems: editor.state.problems,
      diagnostics: editor.state.diagnostics.map((d) => `${d.code} ${d.message}`),
      interactions: [...latencies.values()],
    };
  });
}

const over: string[] = [];
const browser = await chromium.launch({ channel: "chrome", headless: process.env.BENCH_HEADLESS === "1" });
for (const { name, doc, at, budget } of EDITS) {
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
  await page.goto(`http://127.0.0.1:${String(port)}/index.html?doc=${doc}`);
  await page.waitForFunction(() => ((window as { compmdEditor?: { state: { painted: { count: number } } } }).compmdEditor?.state.painted.count ?? 0) > 0, null, { timeout: 60_000 });
  const placed = await page.evaluate((text) => {
    const textarea = document.querySelector("textarea");
    const found = textarea?.value.indexOf(text) ?? -1;
    if (!textarea || found < 0) return false;
    textarea.focus();
    textarea.setSelectionRange(found + 4, found + 4);
    return true;
  }, at);
  if (!placed) throw new Error(`${name}: no \`${at}\` in the text`);
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
  // The processing of opening comes first, so the changes' are one further on.
  const processing = result.processing.slice(warm + 1);
  const rendering = result.rendering.slice(warm);
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
  console.log(`  of which, p95: processing ${p95(processing).toFixed(2)} ms; the frame, from the splices' arrival to painted, ${p95(rendering).toFixed(1)} ms`);
  await page.close();
}
await browser.close();
server.close();
console.log(`Chrome ${browser.version()}`);
if (over.length > 0) {
  console.log(`\nFailed: ${over.join("; ")}`);
  process.exitCode = 1;
}
