// Printing a page of the site as a PDF, as the build does (`scripts/pdf.ts`) and the dev server does when a PDF is
// asked for: what each PDF is printed from, and how it is typeset as a book and printed.
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import type { ConsoleMessage, Page } from "playwright-core";
import { guidePdf, specPdf } from "../src/site.ts";
import { BOOKS } from "./guide-pages.ts";
import { addPreviews } from "./previews.ts";
import type { SpecPage } from "./versions.ts";

/** A book's own page, which is printed into its PDF, in the book's folder. */
export const PRINT = "print/";

/** What to print: a page of the site, and the PDF it becomes, by their paths from the site's root. */
export interface PrintJob {
  page: string;
  pdf: string;
}

/** Every PDF of the site: each book's, from its own page, and each version of the spec's, from its page. */
export function printJobs(specs: readonly SpecPage[]): PrintJob[] {
  return [
    ...BOOKS.map((book) => ({ page: `guides/${book}/${PRINT}`, pdf: `guides/${book}/${guidePdf(book)}` })),
    ...specs.map(({ folder, version }) => ({ page: folder, pdf: `${folder}${specPdf(version)}` })),
  ];
}

/** A book's typesetting: its page size, margins, title page, contents, chapter openers and running heads. */
const BOOK_CSS = readFileSync(join(import.meta.dirname, "book.css"), "utf8");

/** Paged.js, the CSS Paged Media polyfill, which lays a page out in pages by {@link BOOK_CSS}. */
// The package exports only its entry, `src/index.js`, so its built polyfill is found from there.
const PAGED = join(dirname(createRequire(join(import.meta.dirname, "print.ts")).resolve("pagedjs")), "../dist/paged.polyfill.js");

/** What Paged.js gives the page, run by hand rather than as the page loads. */
interface Paged {
  PagedConfig?: { auto: boolean };
  PagedPolyfill: { preview: (content: undefined, stylesheets: Record<string, string>[]) => Promise<unknown> };
}

/**
 * Gives each entry of the page's contents (a book's chapter list, the spec's contents) the place of its target as
 * `data-page-of`, which {@link BOOK_CSS} numbers it by: a marker at the start of the target, with an id that is a
 * selector as it stands. Paged.js finds a target with `querySelector`, which the spec's ids (`1-status-…`) aren't.
 * Run in the page.
 */
function markContents(): void {
  for (const link of document.querySelectorAll<HTMLAnchorElement>('.front ol > li > a[href^="#"], .toc li > a[href^="#"]')) {
    const id = decodeURIComponent(link.hash.slice(1));
    const target = document.getElementById(id);
    if (!target) continue;
    const marker = document.createElement("span");
    marker.id = `page-of-${String(document.querySelectorAll("[data-page-of]").length)}`;
    target.prepend(marker);
    link.dataset.pageOf = `#${marker.id}`;
  }
}

/**
 * Gives each section's heading its title alone as `data-title`, the running head {@link BOOK_CSS} takes from it: its
 * text holds its anchor's and its source link's too, which print hides. Run in the page.
 */
function titleHeadings(): void {
  for (const heading of document.querySelectorAll<HTMLElement>("h2")) {
    const title = [...heading.childNodes].filter((node) => !(node instanceof Element && node.matches(".anchor, .source"))).map((node) => node.textContent).join("");
    heading.dataset.title = title.replace(/\s+/g, " ").trim();
  }
}

/**
 * Numbers the laid-out pages by increments alone. Paged.js restarts the numbering (a book's first chapter at 1) with
 * `counter-reset` on that page, which Chrome scopes to the page itself: the pages after it went on from the old count.
 * An increment by the difference counts on in the pages after, as every other page's does. Run in the page.
 */
function numberPages(): void {
  let number = 0;
  for (const page of document.querySelectorAll<HTMLElement>(".pagedjs_page")) {
    const reset = /\bpage (-?\d+)/.exec(getComputedStyle(page).counterReset);
    const next = reset ? Number(reset[1]) : number + 1;
    page.style.counterReset = "none";
    page.style.counterIncrement = `page ${String(next - number)}`;
    number = next;
  }
}

/**
 * Prints the page at `url` in `page` (light) as a book: typeset by {@link BOOK_CSS} with Paged.js, at the size of its
 * pages, with an outline from its headings. With `previews`, a book's examples get their pictures (`previews.ts`),
 * from the playground served beside the page.
 *
 * Paged.js is given the book's stylesheet alone: the site's stay in the page as they are, under print media from the
 * start so its pages are laid out as they print. (Its CSS parser, from 2022, can't read all of the site's.)
 */
export async function printPdf(page: Page, url: string, { previews }: { previews: boolean }): Promise<Buffer> {
  // Paged.js reports a layout it gives up on (a block it can't place anywhere) as a console error, and prints the pages
  // it laid out so far: the rest of the book would be missing.
  const problems: string[] = [];
  const onConsole = (message: ConsoleMessage) => {
    if (message.type() === "error") problems.push(message.text());
  };
  const onError = (error: Error) => problems.push(error.message);
  page.on("console", onConsole);
  page.on("pageerror", onError);
  try {
    return await typeset(page, url, previews, problems);
  } finally {
    page.off("console", onConsole);
    page.off("pageerror", onError);
  }
}

/** {@link printPdf}'s work: `problems` collects the page's errors as it goes. */
async function typeset(page: Page, url: string, previews: boolean, problems: readonly string[]): Promise<Buffer> {
  await page.emulateMedia({ media: "print" });
  await page.goto(url, { waitUntil: "load" });
  if (previews) await addPreviews(page, new URL(url).origin);
  await page.evaluate(() => {
    (window as unknown as Paged).PagedConfig = { auto: false };
  });
  // The site's header, status line and footer, which print hides: Paged.js would still give them a page of their own,
  // the title page's verso with a running head, or one at the end.
  await page.evaluate(() => {
    for (const element of document.querySelectorAll("header.site, .status, footer.site")) element.remove();
  });
  // A code block's line breaks between Shiki's lines, which the book sets as blocks instead (`book.css`).
  await page.evaluate(() => {
    for (const code of document.querySelectorAll("pre > code")) {
      for (const node of [...code.childNodes]) if (node.nodeType === Node.TEXT_NODE && node.textContent?.trim() === "") node.remove();
    }
  });
  await page.evaluate(markContents);
  await page.evaluate(titleHeadings);
  await page.addScriptTag({ path: PAGED });
  await page.evaluate(async (css) => {
    await (window as unknown as Paged).PagedPolyfill.preview(undefined, [{ [location.href]: css }]);
  }, BOOK_CSS);
  if (problems.length > 0) throw new Error(`${url} couldn't be typeset: ${problems.join("; ")}`);
  await page.evaluate(numberPages);
  return page.pdf({ preferCSSPageSize: true, printBackground: true, outline: true, tagged: true });
}
