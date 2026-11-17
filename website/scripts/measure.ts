// How long the landing page's live parts take to show, on the built site served as in production: for each browser,
// cold (a new context) and warm (the same page loaded again), the time from navigation until the hero's page shows with
// its components, and the requests on the way there, by when each finished.
// `node scripts/measure.ts [chromium|webkit] [--site <url>] [--throttle]`: `--site` measures a deployed site instead,
// `--throttle` (Chrome only) a 10 Mbit/s connection with 40 ms of latency, as a home connection.
import { createServer } from "node:http";
import { join } from "node:path";
import { chromium, webkit, type Browser, type Page } from "playwright-core";
import { serveFolder } from "../build/static.ts";

const DIST = join(import.meta.dirname, "../dist");
const PORT = 4192;
const site = process.argv.indexOf("--site");
const ROOT = site > 0 ? (process.argv[site + 1] ?? "") : `http://localhost:${String(PORT)}/`;
const THROTTLE = process.argv.includes("--throttle");
const RUNS = 5;

interface Run {
  /** Until the hero's page shows with its components, in ms since navigation. */
  hero: number;
  /** The requests until then: what, how big, when it finished. */
  requests: { name: string; kb: number; end: number }[];
}

/** Loads the landing page and waits for the hero's live page. */
async function run(page: Page, reload: boolean): Promise<Run> {
  if (reload) await page.reload();
  else await page.goto(ROOT);
  await page.locator(".hero-pair .proof.ready").waitFor({ timeout: 60_000 });
  const hero = await page.evaluate(() => performance.now());
  const requests: Run["requests"] = [];
  // The frames' own requests, on the page's clock: a frame's `timeOrigin` is its own.
  const origin = await page.evaluate(() => performance.timeOrigin);
  for (const frame of page.frames()) {
    const listed = await frame
      .evaluate(
        (pageOrigin) =>
          performance.getEntriesByType("resource").map((entry) => {
            const resource = entry as PerformanceResourceTiming;
            return { name: resource.name, kb: resource.encodedBodySize / 1024, end: resource.responseEnd + performance.timeOrigin - pageOrigin };
          }),
        origin,
      )
      .catch(() => []);
    const where = frame === page.mainFrame() ? "" : "[frame] ";
    requests.push(...listed.map((request) => ({ ...request, name: where + request.name })));
  }
  return { hero, requests };
}

const median = (values: number[]): number => values.sort((a, b) => a - b)[Math.floor(values.length / 2)] ?? Number.NaN;

async function measure(browser: Browser, label: string): Promise<void> {
  const cold: Run[] = [];
  const warm: Run[] = [];
  for (let i = 0; i < RUNS; i++) {
    const context = await browser.newContext({ viewport: { width: 1280, height: 860 } });
    const page = await context.newPage();
    if (THROTTLE) {
      const cdp = await context.newCDPSession(page);
      await cdp.send("Network.enable");
      await cdp.send("Network.emulateNetworkConditions", { offline: false, latency: 40, downloadThroughput: (10 * 1024 * 1024) / 8, uploadThroughput: (5 * 1024 * 1024) / 8 });
    }
    cold.push(await run(page, false));
    warm.push(await run(page, true));
    await context.close();
  }
  console.log(`${label}: the hero's live page, median of ${String(RUNS)}: cold ${median(cold.map((r) => r.hero)).toFixed(0)} ms, warm ${median(warm.map((r) => r.hero)).toFixed(0)} ms`);
  const last = cold.at(-1);
  console.log(`  the last cold run: the hero at ${(last?.hero ?? 0).toFixed(0)} ms, after`);
  for (const request of (last?.requests ?? []).sort((a, b) => a.end - b.end)) {
    console.log(`  ${request.end.toFixed(0).padStart(6)} ms  ${request.kb.toFixed(1).padStart(8)} KB  ${request.name}`);
  }
}

const serve = serveFolder("/", DIST);
const server = createServer((request, response) => {
  serve(request, response, () => {
    response.statusCode = 404;
    response.end();
  });
});
await new Promise<void>((resolve) => server.listen(PORT, resolve));
const which = process.argv[2] === "webkit" ? "webkit" : "chromium";
const browser = which === "webkit" ? await webkit.launch() : await chromium.launch({ channel: "chrome" });
try {
  await measure(browser, which);
} finally {
  await browser.close();
  server.close();
}
