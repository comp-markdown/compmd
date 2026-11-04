// Checks the built site in Chrome (the installed one, through Playwright), as GitHub Pages will serve it: `pnpm check`
// after `pnpm build`. It fails on the first problem it finds, saying what it was.
//
// - Every page, light and dark, at a desktop's and a phone's width: no page errors, no failed requests, nothing wider
//   than the window; and each painted styled before any script runs.
// - The PDFs: each book's has every chapter in its outline, in order, and in its text; each spec page's, its sections.
// - The landing page: the hero goes live and follows an edit; the plain toggle switches; the input example follows its
//   input; the JSX and code renderer examples render in the site's sandbox; the site's own stars mount in the page
//   and set their input; a section chosen in the contents wheel scrolls into
//   view, and the wheel marks it.
// - Every link within the site resolves.
// - The spec: its contents, the section being read marked, the menu on a phone, and its companion, a section of the
//   page, linked from the spec and back.
// - Search finds sections of the spec and the guides, and opens them with the words found marked; on a phone, its
//   dialog fits.
// - The playground opens every document of its menu.
// - Shiki and Temml once in each app, the site's and the playground's: in the frames' files, not in its own bundle.
// - The budgets (website/docs/design.md): the page before its live parts at most 100 KB gzipped, and painted within one
//   second on a mid-range phone over fast 3G. `--no-timing` leaves the paint out, for a shared runner whose timings
//   vary too much to hold a budget (docs/ci.md).
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { createServer } from "node:http";
import { join } from "node:path";
import { gzipSync } from "node:zlib";
import { getDocument } from "pdfjs-dist/legacy/build/pdf.mjs";
import { chromium, type Browser, type Page } from "playwright-core";
import { bookOf, writtenOf } from "../build/guide.ts";
import { serveFolder } from "../build/static.ts";
import { specPages } from "../build/versions.ts";
import { guidePdf } from "../src/site.ts";

const DIST = join(import.meta.dirname, "../dist");
const PORT = 4191;
const ROOT = `http://localhost:${String(PORT)}/`;
/** A guide's pages: its contents, then each chapter it lists, as the site builds them. */
const bookPages = (book: string): string[] => {
  const chapters = writtenOf(bookOf(readFileSync(join(import.meta.dirname, `../../guides/${book}/README.md`), "utf8")));
  return [`guides/${book}/`, ...chapters.map(({ slug }) => `guides/${book}/${slug}/`)];
};
const PAGES = ["", "spec/", "guides/", ...bookPages("authors"), ...bookPages("developers")];
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

/** Every page paints styled from its HTML alone, its stylesheets inlined, so a link followed shows no unstyled page. */
async function firstPaint(browser: Browser): Promise<void> {
  for (const path of PAGES) {
    const page = await browser.newPage({ javaScriptEnabled: false, colorScheme: "light" });
    await page.goto(ROOT + path);
    const background = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    check(background === "rgb(233, 237, 241)", `/${path} paints unstyled before its script runs (its background is ${background})`);
    await page.close();
  }
}

/** A PDF's outline, every entry at every depth in order, its text with its white space as single spaces, and its pages. */
async function readPdf(path: string): Promise<{ outline: string[]; text: string; pages: number }> {
  const loading = getDocument({ data: new Uint8Array(readFileSync(path)) });
  const pdf = await loading.promise;
  const outline: string[] = [];
  const walk = (items: { title: string; items: unknown[] }[] | null) => {
    for (const item of items ?? []) {
      outline.push(item.title.replace(/\s+/g, " ").trim());
      walk(item.items as typeof items);
    }
  };
  walk(await pdf.getOutline());
  const texts: string[] = [];
  for (let number = 1; number <= pdf.numPages; number++) {
    const content = await (await pdf.getPage(number)).getTextContent();
    texts.push(content.items.map((item) => ("str" in item ? item.str : "")).join(" "));
  }
  const pages = pdf.numPages;
  await loading.destroy();
  return { outline, text: texts.join(" ").replace(/\s+/g, " "), pages };
}

/**
 * The first of `wanted` that `found` lacks in that order, or `undefined` when it has them all in order. White space
 * doesn't count: Chrome joins a heading's lines without a space when it wraps, in the outline it writes.
 */
function missingInOrder(wanted: readonly string[], found: readonly string[]): string | undefined {
  const bare = (title: string | undefined) => title?.replace(/\s/g, "");
  let next = 0;
  for (const title of found) if (bare(title) === bare(wanted[next])) next++;
  return wanted[next];
}

/** Each book's PDF has every chapter in its outline, in the contents' order, and in its text; each spec page's PDF has the page's sections. */
async function pdfs(browser: Browser): Promise<void> {
  for (const book of ["authors", "developers"]) {
    const chapters = writtenOf(bookOf(readFileSync(join(import.meta.dirname, `../../guides/${book}/README.md`), "utf8"))).map(({ number, title }) => `${String(number)}. ${title}`);
    const { outline, text, pages } = await readPdf(join(DIST, "guides", book, guidePdf(book)));
    const missing = missingInOrder(chapters, outline);
    check(missing === undefined, `the ${book} book's PDF lacks "${String(missing)}" in its outline, or has it out of order; its outline starts ${outline.slice(0, 8).join(" | ")}`);
    // Without white space, as for the outline: pdf.js gives the text in runs, with or without the spaces between them.
    const bare = text.replace(/\s/g, "");
    for (const chapter of chapters) check(bare.includes(chapter.replace(/\s/g, "")), `the ${book} book's PDF lacks "${chapter}" in its text`);
    console.log(`ok: the ${book} book's PDF, ${String(pages)} pages, its ${String(chapters.length)} chapters in its outline`);
  }
  const tags = execFileSync("git", ["tag", "--list", "spec-v*"], { cwd: join(import.meta.dirname, "../.."), encoding: "utf8" }).split("\n").filter(Boolean);
  for (const { folder } of specPages(tags)) {
    const { page } = await open(browser, folder, VIEWS[0]);
    const sections = await page.locator("article.spec h2").evaluateAll((headings) =>
      headings.map((heading) => [...heading.childNodes].filter((node) => !(node instanceof Element && node.matches(".anchor, .source"))).map((node) => node.textContent).join("").replace(/\s+/g, " ").trim()),
    );
    const pdf = (await page.locator(".status a[download]").getAttribute("href")) ?? "";
    await page.close();
    const { outline, pages } = await readPdf(join(DIST, folder, pdf));
    const missing = missingInOrder(sections, outline);
    check(missing === undefined, `/${folder}'s PDF lacks "${String(missing)}" in its outline, or has it out of order`);
    console.log(`ok: /${folder}'s PDF, ${String(pages)} pages, its ${String(sections.length)} sections in its outline`);
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

  const framed = async (title: string) => {
    const proof = page.locator(".feature", { has: page.getByRole("heading", { name: title }) }).locator(".proof");
    await proof.scrollIntoViewIfNeeded();
    await proof.and(page.locator(".framed")).waitFor({ timeout: 15000 });
    return proof.frameLocator("iframe");
  };
  const meter = await (await framed("Components of your own")).locator("progress").getAttribute("value");
  check(meter === "72", `the JSX example rendered ${String(meter)} in the sandbox, not 72`);
  const steps = await (await framed("Code renderers")).locator('svg[aria-label="Draft, then review, then published"] text').allTextContents();
  check(steps.join(" ") === "Draft Review Published", `the code renderer example drew "${steps.join(", ")}" in the sandbox`);

  // The site's own stars, trusted, in this page: a click sets the document's input.
  const rating = page.locator(".feature", { has: page.getByRole("heading", { name: "Your own components, live" }) }).locator(".proof");
  await rating.scrollIntoViewIfNeeded();
  await rating.and(page.locator(".framed")).waitFor({ timeout: 15000 });
  await rating.getByRole("radio", { name: "5 of 5" }).click();
  await page.waitForFunction(() => [...document.querySelectorAll(".proof.hosted")].some((proof) => proof.textContent.includes("5 of 5.")));

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
  // The companion, a section of the page: in the contents, and reached from the spec's §6.4 within the page.
  check((await page.locator(".toc a[href='#registry']").count()) === 1, "the spec's contents don't list its companion");
  const toCompanion = page.locator("[id='64-packages-and-the-registry'] ~ ul a[href='#registry']").first();
  check((await toCompanion.count()) === 1, "the spec's §6.4 doesn't link to its companion within the page");
  await toCompanion.click();
  await page.waitForFunction(() => location.hash === "#registry" && Math.abs(document.getElementById("registry")?.getBoundingClientRect().top ?? 999) < 200);
  await page.locator(".toc a[href='#registry-4-who-fetches']").click();
  await page.waitForFunction(() => document.querySelector(".toc a[aria-current]")?.getAttribute("href") === "#registry-4-who-fetches");
  const toSpec = page.locator("#registry ~ p a[href='#']").first();
  check((await toSpec.count()) === 1, "the companion doesn't link back to the spec within the page");
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

/** Search: `/` opens it on a guide's page, results come from the spec and the guides, Enter opens the first with its words marked for a moment; on a phone, the header's button opens it, within the window. */
async function search(browser: Browser): Promise<void> {
  const { page, problems } = await open(browser, "guides/developers/quick-start/", VIEWS[0]);
  await page.keyboard.press("/");
  const dialog = page.locator("dialog.search");
  await dialog.waitFor();
  await dialog.locator("input").fill("remote data");
  await dialog.locator(".search-results a").first().waitFor({ timeout: 15000 });
  const places = await dialog.locator(".search-where").allTextContents();
  check(places.includes("The spec"), `searching "remote data" finds nothing in the spec: ${places.join(", ")}`);
  check(places.some((place) => place.includes("Guide")), `searching "remote data" finds nothing in the guides: ${places.join(", ")}`);
  const first = new URL((await dialog.locator(".search-results a").first().getAttribute("href")) ?? "", page.url()).href;
  await dialog.locator("input").press("Enter");
  await page.waitForURL(first);
  // The words found, marked where the result opens, then no more.
  await page.waitForFunction(() => (CSS.highlights.get("compmd-search")?.size ?? 0) > 0);
  await page.waitForFunction(() => !CSS.highlights.has("compmd-search"), undefined, { timeout: 10000 });
  check(problems.length === 0, `search: ${problems.join("; ")}`);
  await page.close();

  const phone = await open(browser, "spec/", VIEWS[1]);
  await phone.page.locator("header.site button.search").click();
  const box = await phone.page.locator("dialog.search").boundingBox();
  check(box !== null && box.x >= 0 && box.x + box.width <= VIEWS[1].width, `search's dialog on a phone isn't within the window: ${JSON.stringify(box)}`);
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

/**
 * Each app of the site, the site and the playground, has Shiki's grammars and Temml once, in the frames' files: a
 * host-mode page loads them from there (`compmd/highlight.js`, `compmd/math.js`), so its own bundle has none.
 */
function oneCopy(): void {
  for (const app of ["", "playground/"]) {
    check(existsSync(join(DIST, app, "compmd/highlight.js")) && existsSync(join(DIST, app, "compmd/math.js")), `/${app}compmd/ has no highlight.js or math.js`);
    const bundled = readdirSync(join(DIST, app, "assets")).filter((name) => /^(cpp|javascript|typescript|temml)-/.test(name));
    check(bundled.length === 0, `/${app}assets/ has its own copy of Shiki or Temml: ${bundled.join(", ")}`);
  }
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
    ["the first paint", firstPaint],
    ["the PDFs", pdfs],
    ["the landing page", landingPage],
    ["the links", links],
    ["the spec", spec],
    ["search", search],
    ["the playground", playground],
  ] as const) {
    await step(browser);
    console.log(`ok: ${name}`);
  }
  oneCopy();
  console.log("ok: Shiki and Temml once each, in the frames' files");
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
