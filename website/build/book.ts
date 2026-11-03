// A book of the guides as one page, for its PDF (`scripts/pdf.ts`): its contents page, then every chapter written, in
// order, each a section of the page. Links between its chapters go within the page, and the rest where they go on
// the site or GitHub, in full, since a PDF is read anywhere.
import { Window } from "happy-dom";
import { SITE } from "../src/site.ts";
import { bookOf, guideLink, slugOf, writtenOf } from "./guide.ts";
import { BOOKS } from "./guide-pages.ts";
import { PRINT } from "./print.ts";
import { renderMarkdown } from "./spec.ts";

/** What a book's page shows (`src/Book.svelte`). */
export interface BookProps {
  /** The book's title. */
  title: string;
  /** Its contents page, as HTML. */
  contents: string;
  /** Its chapters, in order, as HTML. */
  chapters: string[];
  /** Its address on the site. */
  online: string;
}

/** A book's page: its folder in the site, its path to the site's root, and what its component is given. */
export interface BookPage {
  book: string;
  folder: string;
  root: string;
  props: BookProps;
}

/**
 * Where a link of `book`'s contents or chapters goes in the book's page: a chapter of the book to its section
 * (`#<slug>`, a heading of it `#<slug>-<id>`), its contents to the top, and anything on the site, another book among
 * it, to its address in full; `undefined` for its file on GitHub.
 */
export function bookLink(book: string, href: string): string | undefined {
  if (href.startsWith(SITE)) return href;
  const [path = "", hash] = href.replace(/^\.\//, "").split("#");
  if (/^\d+-[^/]+\.md$/.test(path)) return `#${slugOf(path)}${hash === undefined ? "" : `-${hash}`}`;
  if (path === "README.md") return "#book";
  const onSite = guideLink(href, "");
  return onSite === undefined ? undefined : new URL(onSite, `${SITE}guides/${book}/`).href;
}

/** The elements' ids in `html`, each once. @throws Error naming one given twice, which would link to the wrong place. */
function checkIds(book: string, html: string): void {
  const window = new Window();
  window.document.body.innerHTML = html;
  const seen = new Set<string>();
  for (const { id } of window.document.querySelectorAll("[id]")) {
    if (seen.has(id)) throw new Error(`the ${book} book's page has the id "${id}" twice`);
    seen.add(id);
  }
  window.close();
}

/** Each book's page, from the guides' files: `read` gives a file of `guides/` by its path there. */
export async function bookPages(read: (path: string) => string, blob: string): Promise<BookPage[]> {
  const pages: BookPage[] = [];
  for (const book of BOOKS) {
    const markdown = read(`${book}/README.md`);
    const link = (href: string) => bookLink(book, href);
    const contents = await renderMarkdown(markdown, { blob: `${blob}guides/`, path: `${book}/README.md` }, { rules: false, link });
    const chapters = [];
    for (const chapter of writtenOf(bookOf(markdown))) {
      const path = `${book}/${chapter.file}`;
      chapters.push((await renderMarkdown(read(path), { blob: `${blob}guides/`, path }, { rules: false, link, section: chapter.slug })).html);
    }
    checkIds(book, [contents.html, ...chapters].join("\n"));
    const title = /^# (.+)$/m.exec(markdown)?.[1] ?? "";
    pages.push({ book, folder: `guides/${book}/${PRINT}`, root: "../../../", props: { title, contents: contents.html, chapters, online: `${SITE}guides/${book}/` } });
  }
  return pages;
}
