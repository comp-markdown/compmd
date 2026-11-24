// A post rendered at build time (docs/blog-tool.md, decision 5): processed by the processor in strict mode, so an
// error fails the build, and rendered to static HTML by `@compmd/render` in happy-dom's DOM, with its code highlighted.
// A host-language component shows its fallback here; the page's script mounts it live. Its images are those the blog
// reaches without asking a reader (`post-images.ts`).

import { OpenDocument, process, type Diagnostic, type Request } from "@compmd/processor";
import { createPage } from "@compmd/render";
import { Window } from "happy-dom";
import { BlogError } from "./config.ts";
import type { Highlight } from "./highlight.ts";
import { PostImages } from "./post-images.ts";
import { POSTS, type Post } from "./posts.ts";
import { isRelative, rebased } from "./rebase.ts";
import { loadProcessor } from "./processor.ts";

/** A post, rendered. */
export interface Rendered {
  /** The post's HTML, the content of its page's article. */
  html: string;
  /** The same HTML for the feed, which a reader reads elsewhere: its relative links and images absolute (audit B18). */
  feedHtml: string;
  /** Its stylesheet, the renderer's. */
  css: string;
  /** Its warnings, as lines of the build's output. */
  warnings: string[];
  /** Whether it has host-language components, which the page's script mounts live. */
  live: boolean;
}

/**
 * What the blog gives its posts besides their files: its packs, as `@compmd/vite` reads them, and the origins its
 * posts' data and images come from without asking a reader (`origins.ts`).
 */
export type Given = Pick<Request, "packs" | "ambient" | "policy" | "allowedOrigins">;

/** A diagnostic as a line of the build's output. */
const lineOf = (post: Post, { code, message, file, range }: Diagnostic): string => `${POSTS}/${file ?? post.path}:${String(range.start.line)}:${String(range.start.column)}: ${code} ${message}`;

/** The fence languages a post's text names: its code blocks', for the highlighter to load first. */
const languagesOf = (text: string): Set<string> => new Set([...text.matchAll(/^ {0,3}(?:`{3,}|~{3,})\s*([\w+-]+)/gm)].map(([, language]) => language ?? ""));

/** Each relative link and image source in `root` given to `change`, and set to what it gives. */
function relativeUrls(root: HTMLElement, change: (url: string) => string): void {
  for (const [selector, attribute] of [["a[href]", "href"], ["img[src]", "src"], ["source[src]", "src"]] as const) {
    for (const element of root.querySelectorAll(selector)) {
      const url = element.getAttribute(attribute) ?? "";
      if (isRelative(url)) element.setAttribute(attribute, change(url));
    }
  }
}

/** Where a post's HTML is read: its page's folder below the posts' (`up`, `../`), and the posts' address (`base`). */
export interface Placing {
  /** What goes before its relative links and images on its page. */
  up?: string;
  /** The address its relative links and images are read against in the feed; left relative without it. */
  base?: string;
}

/** `root`'s HTML: the element the renderer set a post's language or direction on (spec §12.3) kept, when it has one. */
const htmlOf = (root: HTMLElement): string => (root.hasAttribute("lang") || root.hasAttribute("dir") ? root.outerHTML : root.innerHTML);

/**
 * `post` rendered at `now`, with what the blog gives it, for its page and for the feed (`placing`).
 *
 * @throws BlogError with its errors, when it has one.
 */
export async function renderPost(post: Post, now: string, highlight: Highlight, given: Given = {}, { up = "", base }: Placing = {}): Promise<Rendered> {
  loadProcessor();
  const request: Request = { ...given, path: post.path, files: post.files, now, mode: "strict" };
  const { expanded, diagnostics, failed } = process(request);
  const errors = diagnostics.filter(({ severity }) => severity === "error");
  if (failed || errors.length > 0) throw new BlogError(`${POSTS}/${post.path} has errors:\n${errors.map((error) => `- ${lineOf(post, error)}`).join("\n")}`);
  const open = new OpenDocument(request);
  const live = open.components.length > 0;
  open.free();
  await highlight.load(languagesOf(post.files[post.path] ?? ""));
  const window = new Window();
  try {
    // happy-dom's elements are the DOM's, as the renderer takes them.
    const root = window.document.body.appendChild(window.document.createElement("div")) as unknown as HTMLElement;
    const images = new PostImages(given.allowedOrigins ?? []);
    const page = createPage(root, { images: images.source });
    page.render(expanded);
    await images.settled();
    highlight.highlight(root);
    // An input's state is in its properties, which HTML doesn't hold; its attributes give it to the page.
    for (const input of root.querySelectorAll("input")) {
      if (input.type === "checkbox") input.toggleAttribute("checked", input.checked);
      else input.setAttribute("value", input.value);
    }
    const feed = root.cloneNode(true) as HTMLElement;
    if (base !== undefined) relativeUrls(feed, (url) => new URL(url, base).href);
    if (up !== "") relativeUrls(root, (url) => rebased(url, up));
    const refused = [...images.refused].map((origin) => `${POSTS}/${post.path}: its images from ${origin} show their alt text: compmd.config.json's \`origins\` doesn't name ${origin}`);
    return { html: htmlOf(root), feedHtml: htmlOf(feed), css: page.stylesheet.text, warnings: [...diagnostics.map((diagnostic) => lineOf(post, diagnostic)), ...refused], live };
  } finally {
    window.close();
  }
}
