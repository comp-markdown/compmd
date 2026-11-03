// The site's long documents as pages, made at build time: the spec, and the chapters of the guide. Their Markdown as
// HTML (the processor's plain GFM rendering), with the code highlighted by Shiki, ids on the headings as GitHub gives
// them so links keep working, each heading linked to itself and to its source, the spec's requirement words (MUST,
// SHOULD, …) marked, relative links pointed at the repository or at the site, and a table of contents of the sections
// and subsections.
import { readFileSync } from "node:fs";
import { initSync, plainHtml } from "@compmd/processor";
import { Window } from "happy-dom";
import { highlighter, type Highlight } from "./highlight.ts";

/** A section or subsection, for the table of contents. */
export interface Heading {
  level: 2 | 3;
  id: string;
  /** Its text, as written. */
  text: string;
}

/** The spec as a page: its HTML, and its table of contents. */
export interface RenderedSpec {
  html: string;
  toc: Heading[];
}

/** The requirement words of RFC 2119 and RFC 8174, longest first. */
const RULE = /\b(MUST NOT|SHALL NOT|SHOULD NOT|NOT RECOMMENDED|MUST|SHALL|SHOULD|RECOMMENDED|REQUIRED|MAY|OPTIONAL)\b/g;

let highlight: Promise<Highlight> | undefined;
let loaded = false;

/** Loads the processor's module and the highlighter, once. */
function load(): Promise<Highlight> {
  if (!loaded) {
    initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
    loaded = true;
  }
  highlight ??= highlighter();
  return highlight;
}

/** The id GitHub gives a heading: lower case, without punctuation, spaces as hyphens; a repeated one numbered. */
export function slugger(): (text: string) => string {
  const seen = new Map<string, number>();
  return (text) => {
    const slug = text
      .toLowerCase()
      .replace(/[^\p{L}\p{N}\s_-]/gu, "")
      .replace(/\s/g, "-");
    const count = seen.get(slug) ?? 0;
    seen.set(slug, count + 1);
    return count === 0 ? slug : `${slug}-${String(count)}`;
  };
}

/** The 1-based line of each ATX heading of `markdown`, in order, outside fenced code. */
export function headingLines(markdown: string): number[] {
  const lines: number[] = [];
  let fence: string | undefined;
  for (const [index, line] of markdown.split("\n").entries()) {
    const opener = /^ {0,3}(`{3,}|~{3,})/.exec(line)?.[1];
    if (fence) {
      if (opener && opener[0] === fence[0] && opener.length >= fence.length && line.trim() === opener) fence = undefined;
    } else if (opener) {
      fence = opener;
    } else if (/^ {0,3}#{1,6}(\s|$)/.test(line)) {
      lines.push(index + 1);
    }
  }
  return lines;
}

/** Where the spec's files are: `blob` is the repository's file URL at the spec's version, ending in `/`. */
export interface Source {
  blob: string;
  /** The spec's file in the repository. */
  path: string;
}

/** How a document is rendered beyond what every long page gets. */
export interface RenderOptions {
  /** Whether the requirement words (MUST, SHOULD, …) are marked: the spec's are rules. */
  rules?: boolean;
  /** Whether each heading links to its line on GitHub (`Source`), as the spec's do. */
  sources?: boolean;
  /**
   * Where a link goes on the site, or `undefined` to keep it: an address as written, a relative link to its file on
   * GitHub. Links within the page (`#…`) never reach it.
   */
  link?: (href: string) => string | undefined;
  /**
   * The document is a section of another page, with this id: its title is the section's heading, with the id, every
   * heading is a level lower, and every other id, and every link within the document, is under it (`<id>-<slug>`), so
   * its ids can't meet the page's own.
   */
  section?: string;
}

/**
 * Renders the long document `markdown`, from `source`, as a page: the spec, a chapter of the guide.
 *
 * @throws Error when its headings and the lines they come from don't match, which would link sections to the wrong
 * place.
 */
export async function renderMarkdown(markdown: string, source: Source, options: RenderOptions = {}): Promise<RenderedSpec> {
  const highlighted = await load();
  const window = new Window();
  const document = window.document;
  const root = document.createElement("article");
  // The logo the file opens with, raw HTML, which the plain rendering would show as text; the site's header has it.
  root.innerHTML = plainHtml(markdown.replace(/^<picture>[\s\S]*?<\/picture>\n+/, ""));

  for (const code of root.querySelectorAll("pre > code")) {
    const html = highlighted(code.textContent.replace(/\n$/, ""), /\blanguage-(\S+)/.exec(code.className)?.[1] ?? "");
    if (html !== undefined) code.parentElement?.replaceWith(document.createRange().createContextualFragment(html));
  }

  const lines = headingLines(markdown);
  const headings = [...root.querySelectorAll("h1, h2, h3, h4, h5, h6")];
  if (headings.length !== lines.length) throw new Error(`${source.path} has ${String(lines.length)} headings in its text and ${String(headings.length)} rendered`);
  const slug = slugger();
  const { section } = options;
  const idOf = (text: string, level: number) => (section === undefined ? slug(text) : level === 1 ? section : `${section}-${slug(text)}`);
  const toc: Heading[] = [];
  for (const [index, written] of headings.entries()) {
    const text = written.textContent;
    const writtenLevel = Number(written.tagName.slice(1));
    const id = idOf(text, writtenLevel);
    const level = section === undefined ? writtenLevel : Math.min(6, writtenLevel + 1);
    const heading = level === writtenLevel ? written : document.createElement(`h${String(level)}`);
    if (heading !== written) {
      heading.append(...written.childNodes);
      written.replaceWith(heading);
    }
    heading.id = id;
    if (level === 2 || level === 3) toc.push({ level, id, text });
    if (level === 1) continue;
    const self = document.createElement("a");
    self.className = "anchor";
    self.href = `#${id}`;
    self.setAttribute("aria-label", `Link to “${text}”`);
    self.textContent = "#";
    heading.append(self);
    if (options.sources === false) continue;
    const original = document.createElement("a");
    original.className = "source";
    original.href = `${source.blob}${source.path}#L${String(lines[index] ?? 1)}`;
    original.textContent = "Source";
    heading.append(original);
  }

  for (const link of root.querySelectorAll("a[href]")) {
    const href = link.getAttribute("href") ?? "";
    if (href.startsWith("#")) {
      if (section !== undefined && !link.classList.contains("anchor")) link.setAttribute("href", `#${section}-${href.slice(1)}`);
      continue;
    }
    const kept = /^([a-z][a-z+.-]*:|\/)/i.test(href) ? href : new URL(href, `${source.blob}${source.path}`).href;
    link.setAttribute("href", options.link?.(href) ?? kept);
  }

  if (options.rules ?? true) markRules(root);
  const html = root.innerHTML;
  window.close();
  return { html, toc };
}

/** Marks the requirement words in `root`'s text, outside code and links. */
function markRules(root: ReturnType<Window["document"]["createElement"]>): void {
  const document = root.ownerDocument;
  const walker = document.createTreeWalker(root, 4 /* text */);
  const texts: Text[] = [];
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    if (!(node.parentElement?.closest("code, pre, a"))) texts.push(node as unknown as Text);
  }
  for (const text of texts) {
    const value = text.data;
    if (!value.match(RULE)) continue;
    const span = document.createElement("span");
    span.innerHTML = value.replace(/[&<>]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" })[c] ?? c).replace(RULE, '<em class="rule">$1</em>');
    (text as unknown as { replaceWith(...nodes: unknown[]): void }).replaceWith(...span.childNodes);
  }
}
