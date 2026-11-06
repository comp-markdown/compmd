// The guides' pages, rendered: the shelf at `guides/`, each book's contents at `guides/<book>/` and each chapter written
// at `guides/<book>/<slug>/`, each with the books beside it, its own open to its chapters, and the chapters before and
// after it. The prerender writes them; the dev server renders one per request.
import type { ComponentProps } from "svelte";
import type Guide from "../src/Guide.svelte";
import { bookOf, guideLink, writtenOf } from "./guide.ts";
import type { IndexedPage } from "./search.ts";
import { renderMarkdown } from "./spec.ts";
import { SITE } from "../src/site.ts";
import processor from "../../wasm/package.json" with { type: "json" };

/** The version of the `@compmd` packages, released together: the guides say which they are written for. */
const VERSION = processor.version;

/** The books, in the order the site lists them: their folders in `guides/`. */
export const BOOKS = ["authors", "developers"] as const;

/**
 * A page of the guides: its folder in the site, its path to the site's root, what its component is given, and what
 * the search index takes of it, named by its book and chapter.
 */
export interface GuidePage {
  folder: string;
  root: string;
  props: ComponentProps<typeof Guide>;
  search: IndexedPage;
}

/** A document's title: its first heading. */
const titleOf = (markdown: string): string => /^# (.+)$/m.exec(markdown)?.[1] ?? "";

/** The longest description, about what a search result shows. */
const DESCRIBED = 200;

/**
 * A page's description, for its head: its first paragraph of prose, as plain text, cut at a word to about
 * {@link DESCRIBED} characters. Headings, lists, tables, quotes, code and HTML are passed over.
 *
 * @example
 * descriptionOf("# Quick start\n\nRender a **document** in [a page](./page.md).\n");
 * // "Render a document in a page."
 */
export function descriptionOf(markdown: string): string {
  const paragraph = markdown
    .split(/\n\s*\n/)
    .map((block) => block.trim())
    .find((block) => block !== "" && !/^(#|[-*+] |\d+\. |\||>|```|~~~|<|:::)/.test(block));
  const text = (paragraph ?? "")
    .replace(/!?\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/[*_`]/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (text.length <= DESCRIBED) return text;
  const cut = text.lastIndexOf(" ", DESCRIBED);
  return `${text.slice(0, cut > 0 ? cut : DESCRIBED)}…`;
}

/**
 * The guides' pages, from their files: `read` gives a file of `guides/` by its path there (`developers/README.md`).
 * `blob` is the repository's file URL, ending in `/`.
 */
export async function guidePages(read: (path: string) => string, blob: string): Promise<GuidePage[]> {
  const books = BOOKS.map((name) => {
    const contents = read(`${name}/README.md`);
    return { name, title: titleOf(contents), parts: bookOf(contents) };
  });
  const shelf = books.map(({ name, title }) => ({ name, title }));
  /**
   * The file `path` of `guides/` as a page `root` from the site's root, its links resolved from `up` (see `guideLink`).
   * A link to the site, which a chapter needs on GitHub, stays on the site it is read on: a local build's, or a preview.
   */
  const page = async (path: string, up: string | undefined, root: string) => {
    const markdown = read(path);
    const link = (href: string) => (href.startsWith(SITE) ? `${root}${href.slice(SITE.length)}` : guideLink(href, up));
    const { html, toc } = await renderMarkdown(markdown, { blob: `${blob}guides/`, path }, { rules: false, sources: false, link });
    return { html, toc, root, books: shelf, title: titleOf(markdown), description: descriptionOf(markdown), version: VERSION };
  };
  const shelfPage = await page("README.md", undefined, "../");
  const pages: GuidePage[] = [{ folder: "guides/", root: "../", props: { ...shelfPage, parts: [] }, search: { url: "guides/", page: "The guides", html: shelfPage.html } }];
  for (const book of books) {
    const chapters = writtenOf(book.parts);
    const link = (chapter: (typeof chapters)[number] | undefined, up: string) =>
      chapter && { title: `${String(chapter.number)}. ${chapter.title}`, href: `${up}${chapter.slug}/` };
    const contents = await page(`${book.name}/README.md`, "", "../../");
    pages.push({
      folder: `guides/${book.name}/`,
      root: "../../",
      props: { ...contents, book: book.name, parts: book.parts, next: link(chapters[0], "") },
      search: { url: `guides/${book.name}/`, page: book.title, html: contents.html },
    });
    for (const [index, chapter] of chapters.entries()) {
      const title = `${String(chapter.number)}. ${chapter.title}`;
      const written = await page(`${book.name}/${chapter.file}`, "../", "../../../");
      pages.push({
        folder: `guides/${book.name}/${chapter.slug}/`,
        root: "../../../",
        search: { url: `guides/${book.name}/${chapter.slug}/`, page: `${book.title}: ${title}`, html: written.html },
        props: {
          ...written,
          title,
          book: book.name,
          parts: book.parts,
          current: chapter.slug,
          previous: index === 0 ? { title: "Contents", href: "../" } : link(chapters[index - 1], "../"),
          next: link(chapters[index + 1], "../"),
        },
      });
    }
  }
  return pages;
}
