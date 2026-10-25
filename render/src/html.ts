// Raw HTML (spec §2.3): rendered through an allowlist like GitHub's. CommonMark splits HTML into `html` nodes around
// the Markdown between them (`<kbd>`, text, `</kbd>`; or `<details>`, paragraphs, `</details>`), so tags are paired
// across a list of siblings. A tag outside the allowlist, or one that doesn't pair, shows as text; comments are dropped.
import type { ExpandedNode, MarkdownNode } from "./tree.ts";

/** An allowed element, built from paired tags, holding the nodes between them. */
export interface HtmlElementNode extends MarkdownNode {
  type: "compmdHtml";
  tag: string;
  attributes: Record<string, string>;
  children: ExpandedNode[];
}

const ALLOWED: Record<string, readonly string[]> = {
  details: ["open"],
  summary: [],
  sub: [],
  sup: [],
  kbd: [],
  br: [],
  img: ["src", "alt"],
  a: ["href"],
};
const VOID = new Set(["br", "img"]);
const URL_ATTRIBUTES = new Set(["href", "src"]);

/**
 * `url` if it is safe to follow: relative, a fragment, or `http:`, `https:` or `mailto:`.
 *
 * @example
 * safeUrl("https://example.com"); // "https://example.com"
 * safeUrl("javascript:alert(1)"); // undefined
 */
export function safeUrl(url: string): string | undefined {
  // Browsers ignore control characters and whitespace in a scheme, so they don't hide one here either.
  // eslint-disable-next-line no-control-regex
  const scheme = /^([a-z][a-z0-9+.-]*):/i.exec(url.replace(/[\u0000- ]/g, ""))?.[1]?.toLowerCase();
  return scheme === undefined || ["http", "https", "mailto"].includes(scheme) ? url : undefined;
}

type Token =
  | { kind: "open"; tag: string; attributes: Record<string, string> }
  | { kind: "close"; tag: string; source: string }
  | { kind: "text"; value: string }
  | { kind: "node"; node: ExpandedNode };

const TAG = /<!--[\s\S]*?-->|<(\/?)([a-zA-Z][a-zA-Z0-9-]*)((?:\s+[^\s"'>/=]+(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s"'=<>`]+))?)*)\s*\/?>/g;
const ATTRIBUTE = /([^\s"'>/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g;

function attributesOf(source: string, tag: string): Record<string, string> {
  const allowed = ALLOWED[tag] ?? [];
  const attributes: Record<string, string> = {};
  for (const match of source.matchAll(ATTRIBUTE)) {
    const name = (match[1] ?? "").toLowerCase();
    const value = match[2] ?? match[3] ?? match[4] ?? "";
    if (!allowed.includes(name)) continue;
    if (URL_ATTRIBUTES.has(name) && safeUrl(value) === undefined) continue;
    attributes[name] = value;
  }
  return attributes;
}

function tokensOf(html: string): Token[] {
  const tokens: Token[] = [];
  let at = 0;
  for (const match of html.matchAll(TAG)) {
    if (match.index > at) tokens.push({ kind: "text", value: html.slice(at, match.index) });
    at = match.index + match[0].length;
    const tag = match[2]?.toLowerCase();
    if (tag === undefined) continue; // a comment
    if (!(tag in ALLOWED)) tokens.push({ kind: "text", value: match[0] });
    else if (match[1] === "/") tokens.push({ kind: "close", tag, source: match[0] });
    else tokens.push({ kind: "open", tag, attributes: attributesOf(match[3] ?? "", tag) });
  }
  if (at < html.length) tokens.push({ kind: "text", value: html.slice(at) });
  return tokens;
}

/**
 * `nodes` with their raw HTML made safe: allowed tags become elements holding what's between them, everything else
 * text. A list without `html` nodes comes back as it is.
 *
 * @example
 * withHtml([html("<kbd>"), text("Ctrl"), html("</kbd>")]); // [{ type: "compmdHtml", tag: "kbd", children: [text("Ctrl")] }]
 */
export function withHtml(nodes: readonly ExpandedNode[]): readonly ExpandedNode[] {
  if (!nodes.some((node) => node.type === "html")) return nodes;
  const tokens = nodes.flatMap((node): Token[] => (node.type === "html" ? tokensOf(String(node.value)) : [{ kind: "node", node }]));
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
