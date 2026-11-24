// Raw HTML (spec §2.3): rendered through an allowlist like GitHub's. CommonMark splits HTML into `html` nodes around
// the Markdown between them (`<kbd>`, text, `</kbd>`; or `<details>`, paragraphs, `</details>`), so tags are paired
// across a list of siblings. A tag outside the allowlist, or one that doesn't pair, shows as text; comments are dropped.
import { scanHtml, type Scanned, type ScannedAttribute } from "./html-scanner.ts";
import type { ExpandedNode, MarkdownNode } from "./tree.ts";

/** An allowed element, built from paired tags, holding the nodes between them. */
export interface HtmlElementNode extends MarkdownNode {
  type: "compmdHtml";
  tag: string;
  attributes: Record<string, string>;
  children: ExpandedNode[];
}

/** The allowed tags and their attributes; a `Map`, so a tag a document writes (`<constructor>`) is never a property. */
const ALLOWED = new Map<string, readonly string[]>([
  ["details", ["open"]],
  ["summary", []],
  ["sub", []],
  ["sup", []],
  ["kbd", []],
  ["br", []],
  ["img", ["src", "alt"]],
  ["a", ["href"]],
]);
const VOID = new Set(["br", "img"]);
const URL_ATTRIBUTES = new Set(["href", "src"]);

/** The schemes every link and image may use (spec §15). */
const SAFE_SCHEMES: readonly string[] = ["http", "https", "mailto"];

/** Schemes a host can never name as its own: they run code, or carry content of their own (spec §15). */
const NEVER: readonly string[] = ["javascript", "vbscript", "data", "blob", "file"];

/**
 * The scheme of `url`, in lower case and without its `:`, or `undefined` when it has none (a relative URL).
 *
 * @example
 * urlScheme("App:item/7"); // "app"
 * urlScheme("./logo.png"); // undefined
 */
export function urlScheme(url: string): string | undefined {
  // Browsers ignore control characters and whitespace in a scheme, so they don't hide one here either.
  // eslint-disable-next-line no-control-regex
  return /^([a-z][a-z0-9+.-]*):/i.exec(url.replace(/[\u0000- ]/g, ""))?.[1]?.toLowerCase();
}

/**
 * `url` if a link may follow it or an image show it: relative, a fragment, `http:`, `https:` or `mailto:`, or one of
 * `schemes`, the host's own, which {@link ownSchemes} has checked.
 *
 * @example
 * safeUrl("https://example.com"); // "https://example.com"
 * safeUrl("javascript:alert(1)"); // undefined
 * safeUrl("app:page/1", ["app"]); // "app:page/1"
 */
export function safeUrl(url: string, schemes: readonly string[] = []): string | undefined {
  const scheme = urlScheme(url);
  return scheme === undefined || SAFE_SCHEMES.includes(scheme) || schemes.includes(scheme) ? url : undefined;
}

/**
 * The host's own URL schemes, as {@link safeUrl} takes them: names without their `:`, in lower case.
 *
 * @throws Error when one isn't a scheme, or is one a host can never name: `javascript`, `vbscript`, `data`, `blob`
 * or `file`.
 * @example
 * ownSchemes(["App", "fig"]); // ["app", "fig"]
 */
export function ownSchemes(schemes: readonly string[]): readonly string[] {
  return schemes.map((given) => {
    const scheme = given.replace(/:$/, "").toLowerCase();
    if (!/^[a-z][a-z0-9+.-]*$/.test(scheme)) throw new Error(`\`${given}\` isn't a URL scheme`);
    if (NEVER.includes(scheme)) throw new Error(`\`${scheme}:\` can't be a host's own scheme: it runs code or carries content of its own`);
    return scheme;
  });
}

type Token =
  | { kind: "open"; tag: string; attributes: Record<string, string> }
  | { kind: "close"; tag: string; source: string }
  | { kind: "text"; value: string }
  | { kind: "node"; node: ExpandedNode };

/** The allowed attributes of `tag` among those written, a link's or an image's URL kept only when {@link safeUrl} keeps it. */
function attributesOf(written: readonly ScannedAttribute[], tag: string, schemes: readonly string[]): Record<string, string> {
  const allowed = ALLOWED.get(tag) ?? [];
  const attributes: Record<string, string> = {};
  for (const { name, value } of written) {
    if (!allowed.includes(name)) continue;
    if (URL_ATTRIBUTES.has(name) && safeUrl(value, schemes) === undefined) continue;
    attributes[name] = value;
  }
  return attributes;
}

/** Each `html` node's text, scanned once however often it renders. */
const scans = new WeakMap<ExpandedNode, Scanned[]>();

/** The scan of an `html` node's text. */
function scanOf(node: ExpandedNode): Scanned[] {
  let scanned = scans.get(node);
  if (scanned === undefined) scans.set(node, (scanned = scanHtml(String((node as MarkdownNode).value))));
  return scanned;
}

/** The tokens of scanned HTML: a tag outside the allowlist as text, a comment as nothing, with `schemes` for URLs. */
function tokensOf(scanned: readonly Scanned[], schemes: readonly string[]): Token[] {
  return scanned.flatMap((part): Token[] => {
    if (part.kind === "comment") return [];
    if (part.kind === "text") return [part];
    if (!ALLOWED.has(part.tag)) return [{ kind: "text", value: part.source }];
    if (part.closing) return [{ kind: "close", tag: part.tag, source: part.source }];
    return [{ kind: "open", tag: part.tag, attributes: attributesOf(part.attributes, part.tag, schemes) }];
  });
}

/** The `src` of each image of raw HTML that renders, as {@link withHtml} keeps it with the host's own `schemes`. */
export function htmlImageSources(html: string, schemes: readonly string[] = []): string[] {
  return tokensOf(scanHtml(html), schemes).flatMap((token) => (token.kind === "open" && token.tag === "img" && token.attributes.src !== undefined ? [token.attributes.src] : []));
}

/** An allowed tag of raw HTML that can pair across siblings: an opener of an element that isn't void, or a closer. */
export interface Tag {
  open: boolean;
  tag: string;
}

/** The tags of `nodes`' raw HTML that can pair across them, in order. */
export function tagsOf(nodes: readonly ExpandedNode[]): Tag[] {
  return nodes
    .filter((node) => node.type === "html")
    .flatMap((node) => tokensOf(scanOf(node), []))
    .flatMap((token): Tag[] => (token.kind === "close" ? [{ open: false, tag: token.tag }] : token.kind === "open" && !VOID.has(token.tag) ? [{ open: true, tag: token.tag }] : []));
}

/** The elements still open after `tags`, with `open` open before them, paired as {@link withHtml} pairs them. */
export function stillOpen(open: readonly string[], tags: readonly Tag[]): string[] {
  const stack = [...open];
  for (const { open: opens, tag } of tags) {
    if (opens) stack.push(tag);
    else if (stack.at(-1) === tag) stack.pop();
  }
  return stack;
}

/**
 * `nodes` with their raw HTML made safe: allowed tags become elements holding what's between them, everything else
 * text. Links and images keep their URL when {@link safeUrl} does, with the host's own `schemes`. A list without
 * `html` nodes comes back as it is.
 *
 * @example
 * withHtml([html("<kbd>"), text("Ctrl"), html("</kbd>")]); // [{ type: "compmdHtml", tag: "kbd", children: [text("Ctrl")] }]
 */
export function withHtml(nodes: readonly ExpandedNode[], schemes: readonly string[] = []): readonly ExpandedNode[] {
  if (!nodes.some((node) => node.type === "html")) return nodes;
  const tokens = nodes.flatMap((node): Token[] => (node.type === "html" ? tokensOf(scanOf(node), schemes) : [{ kind: "node", node }]));
  const root: ExpandedNode[] = [];
  const open: HtmlElementNode[] = [];
  const append = (node: ExpandedNode) => (open.at(-1)?.children ?? root).push(node);
  for (const token of tokens) {
    if (token.kind === "node") append(token.node);
    else if (token.kind === "text") append({ type: "text", value: token.value });
    else if (token.kind === "open") {
      const element: HtmlElementNode = { type: "compmdHtml", tag: token.tag, attributes: token.attributes, children: [] };
      append(element);
      if (!VOID.has(token.tag)) open.push(element);
    } else if (open.at(-1)?.tag === token.tag) open.pop();
    else append({ type: "text", value: token.source });
  }
  return root;
}
