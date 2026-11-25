// Checks the built site in Chrome (the installed one, through Playwright), as GitHub Pages will serve it: `pnpm check`
// after `pnpm build`. It fails on the first problem it finds, saying what it was.
//
// - Every page, light and dark, at a desktop's and a phone's width: no page errors, no failed requests, nothing wider
//   than the window; and each painted styled before any script runs.
// - The PDFs: each book's has every chapter in its outline, in order, and in its text, its title page no number, and
//   its contents each chapter's page, the one it opens on, and a picture for each example; each spec page's, its
//   sections.
// - The landing page: the hero goes live, its shelf picks the book clicked, again and again, with every title fitting
//   its cover, and the page under it follows, stars from the data among it; the hero follows an edit; the plain toggle
//   switches; the input example follows its input; the JSX, code renderer and table cell examples render in the site's
//   sandbox; the site's own stars mount in the page and set their input; a section chosen in the contents wheel
//   scrolls into view, and the wheel marks it.
// - Every link within the site resolves.
// - The spec: its contents and "On this page", the section and the subsection being read marked, the menu on a phone,
//   and its companion, a section of the page, linked from the spec and back.
// - Search finds sections of the spec and the guides, and opens them with the words found marked; on a phone, its
//   dialog fits.
// - The blog: every page names its feed, the feed and search have every post, the first post's book club picks the
//   cover clicked, and a post's comments load only once scrolled to.
// - The playground opens every document of its menu.
// - Shiki and Temml once in each app, the site's and the playground's: in the frames' files, not in its own bundle.
// - The budgets (website/docs/design.md): the page before its live parts at most 100 KB gzipped, and so is a post's,
//   and painted within one second on a mid-range phone over fast 3G. `--no-timing` leaves the paint out, for a shared runner whose timings
//   vary too much to hold a budget (docs/ci.md).
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { createServer } from "node:http";
import { join, sep } from "node:path";
import { gzipSync } from "node:zlib";
import { getDocument, OPS } from "pdfjs-dist/legacy/build/pdf.mjs";
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
/** The blog's pages, as the build wrote them: each folder of `dist/blog/` with an `index.html`. */
const blogPages = (): string[] =>
  readdirSync(join(DIST, "blog"), { recursive: true, encoding: "utf8" })
    .filter((path) => path === "index.html" || path.endsWith(`${sep}index.html`))
    .map((path) => `blog/${path.slice(0, -"index.html".length).split(sep).join("/")}`)
    .sort();
const BLOG_PAGES = blogPages();
/** The blog's posts: its pages that aren't its list or its tags'. */
const BLOG_POSTS =BLOG_PAGES.filter((path) => path !== "blog/" && !path.startsWith("blog/tags/"));
const PAGES = ["", "spec/", "guides/", ...bookPages("authors"), ...bookPages("developers"), ...BLOG_PAGES];
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
  // The blog's comments are giscus's, a service of its own: its script is answered here, empty, so the check sees it
  // asked for and never depends on giscus, nor on a post's discussion existing yet.
  await page.route("https://giscus.app/**", (route) => route.fulfill({ status: 200, contentType: "text/javascript", body: "" }));
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

/** A run of a PDF page's text, and how high on the page it is (from the foot, in points). */
interface PdfRun {
  str: string;
  y: number;
}

/**
 * A PDF's outline, every entry at every depth in order, its text with its white space as single spaces, its pages'
 * count, and each page's runs of text.
 */
async function readPdf(path: string): Promise<{ outline: string[]; text: string; pages: number; runs: PdfRun[][]; images: number }> {
  const loading = getDocument({ data: new Uint8Array(readFileSync(path)) });
  const pdf = await loading.promise;
  const outline: string[] = [];
  const walk = (items: { title: string; items: unknown[] }[] | null) => {
    for (const item of items ?? []) {
      outline.push(item.title.normalize("NFKC").replace(/\s+/g, " ").trim());
      walk(item.items as typeof items);
    }
  };
  walk(await pdf.getOutline());
  const runs: PdfRun[][] = [];
  let images = 0;
  for (let number = 1; number <= pdf.numPages; number++) {
    const page = await pdf.getPage(number);
    const content = await page.getTextContent();
    // NFKC: a ligature the font sets (the "fi" of "first") is its letters again.
    runs.push(content.items.flatMap((item) => ("str" in item && item.str.trim() !== "" ? [{ str: item.str.normalize("NFKC").trim(), y: item.transform[5] as number }] : [])));
    images += (await page.getOperatorList()).fnArray.filter((op) => op === OPS.paintImageXObject).length;
  }
  const pages = pdf.numPages;
  await loading.destroy();
  const text = runs.map((page) => page.map(({ str }) => str).join(" ")).join(" ").replace(/\s+/g, " ");
  return { outline, text, pages, runs, images };
}

/** The number printed at a page's foot: its lowest run, when that is a number (arabic or roman); else `undefined`. */
function folio(page: readonly PdfRun[]): string | undefined {
  const lowest = page.reduce<PdfRun | undefined>((low, run) => (low === undefined || run.y < low.y ? run : low), undefined);
  return lowest && /^(\d+|[ivxlc]+)$/.test(lowest.str) ? lowest.str : undefined;
}

/**
 * A book's PDF is typeset as a book: its title page has no number, and each chapter's page in the contents (the number
 * after its title there) is the number printed on the page it opens on (the first with its numbered title, which the
 * contents and the running heads after it don't have first).
 */
function checkBookPages(book: string, chapters: readonly { number: number; title: string }[], runs: readonly PdfRun[][]): void {
  const bare = (text: string) => text.replace(/\s/g, "");
  check(folio(runs[0] ?? []) === undefined, `the ${book} book's title page has a page number`);
  for (const { number, title } of chapters) {
    const numbered = `${String(number)}. ${title}`;
    const opening = runs.findIndex((page) => page.some(({ str }) => bare(str) === bare(numbered)));
    const contents = runs.findIndex((page) => page.some(({ str }) => bare(str) === bare(title)));
    check(opening !== -1 && contents !== -1 && contents < opening, `the ${book} book's PDF lacks "${numbered}" in its contents or as a chapter's title`);
    const page = runs[contents] ?? [];
    const entry = page.findIndex(({ str }) => bare(str) === bare(title));
    const listed = page.slice(entry + 1).find(({ str }) => /^\d+$/.test(str))?.str;
    const printed = folio(runs[opening] ?? []);
    check(listed !== undefined && listed === printed, `the ${book} book's contents put "${numbered}" on page ${String(listed)}, but it opens on the page numbered ${String(printed)}`);
  }
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
    const written = writtenOf(bookOf(readFileSync(join(import.meta.dirname, `../../guides/${book}/README.md`), "utf8")));
    const chapters = written.map(({ number, title }) => `${String(number)}. ${title}`);
    const { outline, text, pages, runs, images } = await readPdf(join(DIST, "guides", book, guidePdf(book)));
    const missing = missingInOrder(chapters, outline);
    check(missing === undefined, `the ${book} book's PDF lacks "${String(missing)}" in its outline, or has it out of order; its outline starts ${outline.slice(0, 8).join(" | ")}`);
    // Without white space, as for the outline: pdf.js gives the text in runs, with or without the spaces between them.
    const bare = text.replace(/\s/g, "");
    for (const chapter of chapters) check(bare.includes(chapter.replace(/\s/g, "")), `the ${book} book's PDF lacks "${chapter}" in its text`);
    checkBookPages(book, written, runs);
    // Every example a chapter links to the playground has its picture (`build/previews.ts`).
    const examples = written.reduce((count, { file }) => count + (readFileSync(join(import.meta.dirname, `../../guides/${book}/${file}`), "utf8").match(/\]\(https:\/\/compmd\.dev\/playground\/\?doc=/g)?.length ?? 0), 0);
    check(images >= examples, `the ${book} book's PDF has ${String(images)} pictures for its ${String(examples)} examples`);
    console.log(`ok: the ${book} book's PDF, ${String(pages)} pages, its ${String(chapters.length)} chapters in its outline and its contents, at the pages they open on, and a picture for each of its ${String(examples)} examples`);
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

  // The book club's shelf, `content/shelf.js` in its frame: a click on a cover picks its book, again and again, and the
  // page under it follows.
  const shelf = page.frameLocator(".hero iframe");
  const rendered = page.locator(".hero .sandboxed");
  const picked = shelf.locator("[role=radio][aria-checked=true]");
  await picked.waitFor({ timeout: 15000 });
  check((await picked.getAttribute("aria-label"))?.startsWith("Klara and the Sun") === true, `the shelf starts at ${String(await picked.getAttribute("aria-label"))}, not Klara and the Sun`);
  // Every line of every title fits its cover: 92 units wide, the title starting at 14, with 8 to spare at the end;
  // within half a unit, since a title shrunk to fit exactly measures a hair over by rounding.
  const widest = await shelf.locator("svg text tspan").evaluateAll((spans) => Math.max(...spans.map((span) => (span as SVGTextContentElement).getComputedTextLength())));
  check(widest <= 70.5, `a title's line on the shelf is ${widest.toFixed(1)} units wide, more than its cover has room for`);
  // Still being read: no rating yet.
  await rendered.getByText("Not rated yet.").waitFor();
  const hailMary = shelf.getByRole("radio", { name: /^Project Hail Mary/ });
  // Low on the cover, where a neighbour's pattern could reach if it weren't kept to its own cover.
  const box = await hailMary.boundingBox();
  await hailMary.click({ position: { x: 8, y: (box?.height ?? 100) - 30 } });
  await shelf.locator("[role=radio][aria-checked=true][aria-label^='Project Hail Mary']").waitFor();
  await rendered.getByText("Andy Weir").waitFor();
  await hailMary.click();
  // Given time to go wrong: a second click would have been answered by now.
  await page.waitForTimeout(1500);
  check((await picked.getAttribute("aria-label"))?.startsWith("Project Hail Mary") === true, "a second click on the picked cover picked another book");
  await shelf.getByRole("radio", { name: /^Piranesi/ }).click();
  // Read, and rated: its stars, from the club's data.
  const stars = rendered.getByRole("img", { name: "Rated 5 of 5" });
  await stars.waitFor();
  check((await stars.textContent()) === "★★★★★", `Piranesi's rating shows "${String(await stars.textContent())}", not five stars`);

  const text = page.locator(".hero textarea");
  await text.fill((await text.inputValue()).replace("# Thursday Book Club", "# Friday Book Club"));
  await page.waitForFunction(() => document.querySelector(".hero .sandboxed h1")?.textContent === "Friday Book Club");

  await page.getByRole("button", { name: "A plain Markdown renderer" }).click();
  check((await page.locator("#plain .proof").innerText()).includes("::: grid {min=12rem gap=m}"), "the plain view doesn't show the document's syntax as text");

  const since = page.locator("#what input[type=number]");
  await since.scrollIntoViewIfNeeded();
  await page.waitForSelector("#what .proof.live.ready");
  check((await since.inputValue()) === "1970", `the input example starts at ${await since.inputValue()}, not 1970`);
  await since.fill("2020");
  await page.waitForFunction(() => !document.querySelector("#what .feature:nth-of-type(3) .proof")?.textContent.includes("Gentleman"));

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
  const weeks = await (await framed("Components in a table cell")).locator("svg[role=img] path").count();
  check(weeks === 3, `the table drew ${String(weeks)} cities' weeks in the sandbox, not 3`);

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
  for (const place of ["", "spec/", "blog/", "playground/"]) check(seen.has(ROOT + place), `nothing links to /${place}`);
}

async function spec(browser: Browser): Promise<void> {
  const { page } = await open(browser, "spec/", VIEWS[0]);
  const entries = await page.locator(".toc li").count();
  check(entries >= 60, `the spec's contents have ${String(entries)} entries`);
  // On a desktop's width, the contents list the sections and mark the one read; "On this page", its subsections and the
  // one read.
  const marked = (where: string, href: string) => page.waitForFunction(({ list, link }) => document.querySelector(`${list} a[aria-current]`)?.getAttribute("href") === link, { list: where, link: href });
  await page.locator(".toc li.level-2 a[href^='#3-']").click();
  await page.locator(".onpage a[href='#31-grammar']").click();
  await marked(".onpage", "#31-grammar");
  check((await page.locator(".toc a[aria-current]").getAttribute("href"))?.startsWith("#3-") === true, "the spec's contents don't mark §3 while 3.1 is read");
  // The companion, a section of the page: in the contents, and reached from the spec's §6.4 within the page.
  check((await page.locator(".toc a[href='#registry']").count()) === 1, "the spec's contents don't list its companion");
  const toCompanion = page.locator("[id='64-packages-and-the-registry'] ~ ul a[href='#registry']").first();
  check((await toCompanion.count()) === 1, "the spec's §6.4 doesn't link to its companion within the page");
  await toCompanion.click();
  await page.waitForFunction(() => location.hash === "#registry" && Math.abs(document.getElementById("registry")?.getBoundingClientRect().top ?? 999) < 200);
  await page.locator(".onpage a[href='#registry-4-who-fetches']").click();
  await marked(".onpage", "#registry-4-who-fetches");
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

/** The blog: every page names its feed; the feed and search have every post; a post's comments load only once a reader scrolls to them. */
async function blog(browser: Browser): Promise<void> {
  const feed = await fetch(`${ROOT}blog/feed.xml`);
  check(feed.ok && (feed.headers.get("content-type") ?? "").includes("xml"), `the blog's feed is ${String(feed.status)}, ${feed.headers.get("content-type") ?? "no type"}`);
  const entries = await feed.text();
  for (const post of BLOG_POSTS) check(entries.includes(`https://compmd.dev/${post}`), `the blog's feed lacks /${post}`);
  const searched = JSON.parse(readFileSync(join(DIST, "search.json"), "utf8")) as { url: string }[];
  for (const post of BLOG_POSTS) check(searched.some(({ url }) => url.startsWith(post)), `search's index has nothing of /${post}`);
  for (const path of BLOG_PAGES) {
    const { page } = await open(browser, path, VIEWS[0]);
    const named = await page.locator("link[rel='alternate'][type='application/atom+xml']").getAttribute("href");
    check(named === "/blog/feed.xml", `/${path} names ${named ?? "no feed"}, not the blog's feed`);
    await page.close();
  }
  // The first post's book club, the landing page's hero: its shelf draws the covers in its frame, and a cover clicked
  // shows what the club made of that book.
  const hello = await open(browser, "blog/hello-world/", VIEWS[0]);
  await hello.page.frameLocator(".compmd-example iframe").getByRole("radio", { name: /^Piranesi/ }).click();
  await hello.page.getByText("A strange, gentle book").waitFor();
  check(hello.problems.length === 0, `/blog/hello-world/'s book club: ${hello.problems.join("; ")}`);
  await hello.page.close();
  for (const path of BLOG_POSTS) {
    // Its HTML doesn't load giscus: its script, scrolled near, does.
    const html = readFileSync(join(DIST, path, "index.html"), "utf8");
    check(!/<script[^>]*\ssrc="https:\/\/giscus\.app/.test(html), `/${path} loads its comments with the page, before they are scrolled to`);
    const { page } = await open(browser, path, VIEWS[0]);
    await page.locator("#comments").scrollIntoViewIfNeeded();
    await page.waitForFunction(() => document.querySelector("#comments script[src='https://giscus.app/client.js']") !== null);
    await page.close();
  }
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

/** The page at `path`'s bytes before its live parts, in KB gzipped: its HTML and the scripts and stylesheets it loads itself. */
function weightOf(path: string): number {
  const html = readFileSync(join(DIST, path, "index.html"), "utf8");
  const loaded = [...html.matchAll(/(?:src|href)="([^"]+\.(?:js|css))"/g)].map(([, href]) => new URL(href ?? "", ROOT + path)).filter((url) => url.href.startsWith(ROOT));
  const assets = loaded.map((url) => readFileSync(join(DIST, decodeURIComponent(url.pathname))));
  return [Buffer.from(html), ...assets].reduce((sum, bytes) => sum + gzipSync(bytes).length, 0) / 1024;
}

/**
 * The site, its playground's page included, has Shiki's grammars and Temml once, in the frames' files: a host-mode
 * page loads them from there (`compmd/frames/<hash>/highlight.js` and `math.js`), so its own bundle has none. Its
 * pages' configurations offer the same libraries, so they share one build of the frames
 * (docs/vite-per-page-config.md, decision 6).
 */
function oneCopy(): void {
  const builds = existsSync(join(DIST, "compmd/frames")) ? readdirSync(join(DIST, "compmd/frames")) : [];
  check(builds.length === 1, `/compmd/frames/ has ${String(builds.length)} builds of the frames, not one`);
  const frames = join(DIST, "compmd/frames", builds[0] ?? "");
  check(existsSync(join(frames, "highlight.js")) && existsSync(join(frames, "math.js")), `/compmd/frames/${builds[0] ?? ""}/ has no highlight.js or math.js`);
  const bundled = readdirSync(join(DIST, "assets")).filter((name) => /^(cpp|javascript|typescript|temml)-/.test(name));
  check(bundled.length === 0, `/assets/ has its own copy of Shiki or Temml: ${bundled.join(", ")}`);
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
    ["the blog", blog],
    ["the playground", playground],
  ] as const) {
    await step(browser);
    console.log(`ok: ${name}`);
  }
  oneCopy();
  console.log("ok: Shiki and Temml once each, in the frames' files");
  const kb = weightOf("");
  check(kb <= BUDGET_KB, `the page before its live parts is ${kb.toFixed(1)} KB gzipped, over ${String(BUDGET_KB)} KB`);
  console.log(`ok: the page before its live parts is ${kb.toFixed(1)} KB gzipped (budget ${String(BUDGET_KB)} KB)`);
  for (const post of BLOG_POSTS) {
    const postKb = weightOf(post);
    check(postKb <= BUDGET_KB, `/${post} is ${postKb.toFixed(1)} KB gzipped, over ${String(BUDGET_KB)} KB`);
    console.log(`ok: /${post} is ${postKb.toFixed(1)} KB gzipped (budget ${String(BUDGET_KB)} KB)`);
  }
  if (!process.argv.includes("--no-timing")) {
    const ms = await paint(browser);
    check(ms <= BUDGET_PAINT_MS, `the largest paint is ${ms.toFixed(0)} ms on a phone over fast 3G, over ${String(BUDGET_PAINT_MS)} ms`);
    console.log(`ok: the largest paint is ${ms.toFixed(0)} ms on a phone over fast 3G (budget ${String(BUDGET_PAINT_MS)} ms)`);
  }
} finally {
  await browser.close();
  server.close();
}
