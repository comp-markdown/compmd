// The site's search index, made at build time from the pages as they are rendered: each section of each page, from
// one heading to the next, with its title, where it is, and its text. The browser searches it (`src/search.ts`).
import { Window } from "happy-dom";

/**
 * The site's pages to index: the landing page's HTML, the spec's page (the version at `spec/`, its companion in it),
 * and the guides' pages, as `guidePages` gives each for the index.
 */
export function sitePages(landing: string, spec: string, guides: readonly { search: IndexedPage }[]): IndexedPage[] {
  return [{ url: "", page: "CompMD", html: landing }, { url: "spec/", page: "The spec", html: spec }, ...guides.map(({ search }) => search)];
}

/** A page to index: its address from the site's root, what it is called in results, and its HTML. */
export interface IndexedPage {
  url: string;
  page: string;
  html: string;
}

/** A section of a page, as the browser searches it. */
export interface Section {
  /** Its place in the index. */
  id: number;
  /** Its address from the site's root: the page's, with the heading's id. */
  url: string;
  /** The page it is in: `Spec`, `Author's Guide: 6. Data from files`. */
  page: string;
  /** Its heading's text; the page's name for the text before the first heading. */
  title: string;
  /** Its text, whitespace collapsed. */
  text: string;
}

/** What isn't content: the site's header and footer, navigation, and a heading's own links. */
const SKIPPED = "header.site, footer, nav, script, style, template, .anchor, .source, .toc, .turn, .status";
const HEADING = /^H[1-4]$/;

/**
 * Every section of `pages`, in order: a section starts at each heading (`h1` to `h4`) and runs to the next. Text
 * before a page's first heading is a section titled with the page's name.
 *
 * @example
 * sectionsOf([{ url: "spec/", page: "Spec", html: "<h2 id=\"a\">A</h2><p>Text.</p>" }]);
 * // [{ id: 0, url: "spec/#a", page: "Spec", title: "A", text: "Text." }]
 */
export function sectionsOf(pages: readonly IndexedPage[]): Section[] {
  const window = new Window();
  const sections: Section[] = [];
  for (const { url, page, html } of pages) {
    const root = window.document.createElement("div");
    root.innerHTML = html;
    for (const skipped of root.querySelectorAll(SKIPPED)) skipped.remove();
    let current = { url, title: page, text: [] as string[] };
    const close = () => {
      const text = current.text.join(" ").replace(/\s+/g, " ").trim();
      if (text || current.title !== page) sections.push({ id: sections.length, url: current.url, page, title: current.title, text });
    };
    const walker = window.document.createTreeWalker(root, 1 | 4 /* elements and text */);
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      if (node.nodeType === 1 && HEADING.test((node as unknown as Element).tagName)) {
        const heading = node as unknown as Element;
        close();
        const id = heading.getAttribute("id");
        current = { url: id ? `${url}#${id}` : url, title: heading.textContent.replace(/\s+/g, " ").trim(), text: [] };
        continue;
      }
      if (node.nodeType === 3 && !node.parentElement?.closest("h1, h2, h3, h4")) current.text.push(node.textContent);
    }
    close();
  }
  window.close();
  return sections;
}
