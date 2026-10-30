// The spec as a page of the site, made at build time: its Markdown as HTML (the processor's plain GFM rendering),
// with the code highlighted by Shiki, ids on the headings as GitHub gives them so links keep working, each heading
// linked to itself and to its source, the requirement words (MUST, SHOULD, …) marked, relative links pointed at the
// repository, and a table of contents of its sections and subsections.
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

/**
 * Renders the spec `markdown`, from `source`, as a page.
 *
 * @throws Error when its headings and the lines they come from don't match, which would link sections to the wrong
 * place.
 */
export async function renderSpec(markdown: string, source: Source): Promise<RenderedSpec> {
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
  if (headings.length !== lines.length) throw new Error(`the spec has ${String(lines.length)} headings in its text and ${String(headings.length)} rendered`);
  const slug = slugger();
  const toc: Heading[] = [];
  for (const [index, heading] of headings.entries()) {
    const text = heading.textContent;
    const id = slug(text);
    heading.id = id;
    const level = Number(heading.tagName.slice(1));
    if (level === 2 || level === 3) toc.push({ level, id, text });
    if (level === 1) continue;
    const self = document.createElement("a");
    self.className = "anchor";
    self.href = `#${id}`;
    self.setAttribute("aria-label", `Link to “${text}”`);
    self.textContent = "#";
    const original = document.createElement("a");
    original.className = "source";
    original.href = `${source.blob}${source.path}#L${String(lines[index] ?? 1)}`;
    original.textContent = "Source";
    heading.append(self, original);
  }

  for (const link of root.querySelectorAll("a[href]")) {
    const href = link.getAttribute("href") ?? "";
    if (!/^([a-z][a-z+.-]*:|#|\/)/i.test(href)) link.setAttribute("href", source.blob + href.replace(/^\.\//, ""));
  }

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

  const html = root.innerHTML;
  window.close();
  return { html, toc };
}
