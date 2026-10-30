// Checks the built site in Chrome (the installed one, through Playwright), as GitHub Pages will serve it: `pnpm check`
// after `pnpm build`. It fails on the first problem it finds, saying what it was.
//
// - Every page, light and dark, at a desktop's and a phone's width: no page errors, no failed requests, nothing wider
//   than the window.
// - The landing page: the hero goes live and follows an edit; the plain toggle switches; the input example follows its
//   input; the JSX example renders in the playground's sandbox; a section chosen in the contents wheel scrolls into
//   view, and the wheel marks it.
// - Every link within the site resolves.
// - The spec: its contents, the section being read marked, the menu on a phone.
// - The playground opens every document of its menu.
// - The budgets (website/docs/design.md): the page before its live parts at most 100 KB gzipped, and painted within one
//   second on a mid-range phone over fast 3G. `--no-timing` leaves the paint out, for a shared runner whose timings
//   vary too much to hold a budget (docs/ci.md).
import { readFileSync } from "node:fs";
import { createServer } from "node:http";
import { join } from "node:path";
import { gzipSync } from "node:zlib";
import { chromium, type Browser, type Page } from "playwright-core";
import { serveFolder } from "../build/static.ts";

const DIST = join(import.meta.dirname, "../dist");
const PORT = 4191;
const ROOT = `http://localhost:${String(PORT)}/`;
const PAGES = ["", "spec/"];
const VIEWS = [
  { name: "desktop", width: 1280, height: 860 },
  { name: "phone", width: 390, height: 844 },
] as const;
const BUDGET_KB = 100;
const BUDGET_PAINT_MS = 1000;

function check(condition: boolean, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

/** A page that records its errors and failed requests. */
async function open(browser: Browser, path: string, options: { width: number; height: number; scheme?: "light" | "dark" }): Promise<{ page: Page; problems: string[] }> {
  const page = await browser.newPage({ viewport: { width: options.width, height: options.height }, colorScheme: options.scheme ?? "light" });
  const problems: string[] = [];
  page.on("pageerror", (error) => problems.push(`page error: ${error.message}`));
  page.on("console", (message) => {
    if (message.type() === "error") problems.push(`console error: ${message.text()}`);
  });
  page.on("response", (response) => {
    if (response.status() >= 400) problems.push(`${String(response.status())} for ${response.url()}`);
  });
  await page.goto(ROOT + path);
  return { page, problems };
}

async function everyPage(browser: Browser): Promise<void> {
  for (const path of PAGES) {
    for (const view of VIEWS) {
      for (const scheme of ["light", "dark"] as const) {
        const { page, problems } = await open(browser, path, { ...view, scheme });
        await page.waitForLoadState("networkidle");
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
        check(problems.length === 0, `/${path} (${view.name}, ${scheme}): ${problems.join("; ")}`);
        check(overflow <= 0, `/${path} (${view.name}, ${scheme}) is ${String(overflow)}px wider than the window`);
        await page.close();
      }
    }
  }
}

async function landingPage(browser: Browser): Promise<void> {
  const { page, problems } = await open(browser, "", VIEWS[0]);
  await page.waitForSelector(".hero .proof.ready", { timeout: 15000 });
  const text = page.locator(".hero textarea");
  await text.fill((await text.inputValue()).replace("title: Launch week", "title: Ship it"));
  await page.waitForFunction(() => document.querySelector(".hero .proof h1")?.textContent === "Ship it");

  await page.getByRole("button", { name: "A plain Markdown renderer" }).click();
  check((await page.locator("#plain .proof").innerText()).includes(":value[14 bugs fixed]"), "the plain view doesn't show the document's syntax as text");

  const since = page.locator("#what input[type=number]");
  await since.scrollIntoViewIfNeeded();
  await page.waitForSelector("#what .proof.live.ready");
  check((await since.inputValue()) === "1970", `the input example starts at ${await since.inputValue()}, not 1970`);
  await since.fill("2000");
  await page.waitForFunction(() => !document.querySelector("#what .feature:nth-of-type(3) .proof")?.textContent.includes("Dispossessed"));

  await page.locator(".proof.framing").scrollIntoViewIfNeeded();
  await page.waitForSelector(".proof.framed", { timeout: 15000 });
  const meter = await page.frameLocator(".proof.framed iframe").locator("progress").getAttribute("value");
  check(meter === "72", `the JSX example rendered ${String(meter)} in the sandbox, not 72`);

  // The page scrolls smoothly, so the section arrives, and the wheel marks it, a moment after the click.
  await page.locator(".wheel a[href='#safe']").click();
  await page.waitForFunction(() => {
    const top = document.getElementById("safe")?.getBoundingClientRect().top ?? -1;
    return top >= 0 && top < 200 && document.querySelector(".wheel a[aria-current]")?.getAttribute("href") === "#safe";
  });
  check(problems.length === 0, `the landing page: ${problems.join("; ")}`);
  await page.close();
}

async function links(browser: Browser): Promise<void> {
  const seen = new Set<string>();
  for (const path of [...PAGES, "playground/"]) {
    const { page } = await open(browser, path, VIEWS[0]);
    const hrefs = await page.evaluate(() => [...document.querySelectorAll<HTMLAnchorElement>("a[href]")].map((link) => link.href));
    await page.close();
    for (const href of hrefs) {
      const url = new URL(href);
      url.hash = "";
      if (!url.href.startsWith(ROOT) || seen.has(url.href)) continue;
      seen.add(url.href);
      const response = await fetch(url);
      check(response.ok, `/${path} links to ${url.href}, which is ${String(response.status)}`);
    }
  }
  for (const place of ["", "spec/", "playground/"]) check(seen.has(ROOT + place), `nothing links to /${place}`);
}

async function spec(browser: Browser): Promise<void> {
  const { page } = await open(browser, "spec/", VIEWS[0]);
  const entries = await page.locator(".toc li").count();
  check(entries >= 60, `the spec's contents have ${String(entries)} entries`);
  await page.locator(".toc a[href='#31-grammar']").click();
  await page.waitForFunction(() => document.querySelector(".toc a[aria-current]")?.getAttribute("href") === "#31-grammar");
  await page.close();

  const phone = await open(browser, "spec/", VIEWS[1]);
  const menu = phone.page.locator(".toc-toggle");
  check(await menu.isVisible(), "the spec has no contents menu on a phone");
  check(!(await phone.page.locator("#toc-list").isVisible()), "the spec's contents aren't folded on a phone");
  await menu.click();
  await phone.page.locator(".toc a[href='#15-security']").click();
  await phone.page.waitForFunction(() => location.hash === "#15-security" && document.querySelector(".toc")?.classList.contains("folded"));
  await phone.page.close();
}

async function playground(browser: Browser): Promise<void> {
  const { page, problems } = await open(browser, "playground/", VIEWS[0]);
  const documents = await page.locator("select").first().locator("option").count();
  for (let index = 0; index < documents; index++) {
    await page.locator("select").first().selectOption({ index });
    const frame = page.frameLocator("iframe[title='The rendered page']");
    await frame.locator("#page > *").first().waitFor({ timeout: 15000 });
  }
  check(documents >= 5, `the playground's menu has ${String(documents)} documents`);
  check(problems.length === 0, `the playground: ${problems.join("; ")}`);
  await page.close();
}

/** The page's bytes before the live parts, gzipped: the HTML and what it loads itself. */
function weight(): number {
  const html = readFileSync(join(DIST, "index.html"), "utf8");
  const assets = [...html.matchAll(/(?:src|href)="\.\/(assets\/[^"]+\.(?:js|css))"/g)].map(([, path]) => readFileSync(join(DIST, path ?? "")));
  return [Buffer.from(html), ...assets].reduce((sum, bytes) => sum + gzipSync(bytes).length, 0) / 1024;
}

async function paint(browser: Browser): Promise<number> {
  const runs: number[] = [];
  for (let run = 0; run < 5; run++) {
    const context = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 3 });
    const page = await context.newPage();
    const cdp = await context.newCDPSession(page);
    await cdp.send("Network.enable");
    await cdp.send("Network.emulateNetworkConditions", { offline: false, latency: 562.5, downloadThroughput: (1474 * 1024) / 8, uploadThroughput: (675 * 1024) / 8 });
    await cdp.send("Emulation.setCPUThrottlingRate", { rate: 4 });
    await page.goto(ROOT, { waitUntil: "load" });
    runs.push(
      await page.evaluate(
        () =>
          new Promise<number>((resolve) => {
            new PerformanceObserver((list) => {
              resolve(list.getEntries().at(-1)?.startTime ?? Number.POSITIVE_INFINITY);
            }).observe({ type: "largest-contentful-paint", buffered: true });
          }),
      ),
    );
    await context.close();
  }
  return runs.sort((a, b) => a - b)[Math.floor(runs.length / 2)] ?? Number.POSITIVE_INFINITY;
}

const serve = serveFolder("/", DIST);
const server = createServer((request, response) => {
  serve(request, response, () => {
    response.statusCode = 404;
    response.end();
  });
});
await new Promise<void>((resolve) => server.listen(PORT, resolve));
const browser = await chromium.launch({ channel: "chrome" });
try {
  for (const [name, step] of [
    ["every page", everyPage],
    ["the landing page", landingPage],
    ["the links", links],
    ["the spec", spec],
    ["the playground", playground],
  ] as const) {
    await step(browser);
    console.log(`ok: ${name}`);
  }
  const kb = weight();
  check(kb <= BUDGET_KB, `the page before its live parts is ${kb.toFixed(1)} KB gzipped, over ${String(BUDGET_KB)} KB`);
  console.log(`ok: the page before its live parts is ${kb.toFixed(1)} KB gzipped (budget ${String(BUDGET_KB)} KB)`);
  if (!process.argv.includes("--no-timing")) {
    const ms = await paint(browser);
    check(ms <= BUDGET_PAINT_MS, `the largest paint is ${ms.toFixed(0)} ms on a phone over fast 3G, over ${String(BUDGET_PAINT_MS)} ms`);
    console.log(`ok: the largest paint is ${ms.toFixed(0)} ms on a phone over fast 3G (budget ${String(BUDGET_PAINT_MS)} ms)`);
  }
} finally {
  await browser.close();
  server.close();
}
