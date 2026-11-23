// A post's relative URLs, for a page that is folders below the post's own place (docs/blog-tool.md, decision 5): a
// post's page is a folder below the posts', so `photos/a.png` there is `../photos/a.png`. The build rebases the HTML it
// writes; a live post's script rebases the expanded tree it renders, which the sandbox resolves against the page.

import type { ExpandedRoot } from "@compmd/render";

/** Whether `url` is relative to the document: not absolute, not from the site's root, not a fragment. */
export const isRelative = (url: string): boolean => url !== "" && !/^([a-z][a-z0-9+.-]*:|\/|#|\?)/i.test(url);

/** `url` with `up` before it when it is relative to the document. */
export const rebased = (url: string, up: string): string => (isRelative(url) ? `${up}${url.replace(/^\.\//, "")}` : url);

/** The nodes whose `url` is the document's: an image, a link, a link definition. */
const HOLDS_URL = new Set(["image", "link", "definition"]);

/** Whether `record` is a font's file, as a root's `fonts` list them (spec §12.3). */
const isFontFile = (record: Record<string, unknown>): boolean => "weight" in record && "style" in record;

/**
 * `tree` with `up` before each relative URL of its images, links, link definitions and fonts' files, in its examples
 * too; a pack's file (`pack: true`) is the pack's, not the document's, and stays, and so do params, which are data.
 */
export function rebasedTree(tree: ExpandedRoot, up: string): ExpandedRoot {
  if (up === "") return tree;
  const walk = (value: unknown): unknown => {
    if (Array.isArray(value)) return value.map(walk);
    if (value === null || typeof value !== "object") return value;
    const record = value as Record<string, unknown>;
    const copy = Object.fromEntries(Object.entries(record).map(([key, field]) => [key, key === "params" ? field : walk(field)]));
    const own = typeof record.url === "string" && record.pack !== true && (HOLDS_URL.has(String(record.type)) || isFontFile(record));
    if (own) copy.url = rebased(record.url as string, up);
    return copy;
  };
  return walk(tree) as ExpandedRoot;
}
