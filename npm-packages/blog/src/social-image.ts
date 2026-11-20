// The image a link to a page shows on social sites and in chats, its Open Graph image (docs/blog-tool.md, decisions 2
// and 4): a post's own, `socialImage` in its frontmatter, for its page; the blog's default, `defaultSocialImage` in
// `blog.config.json`, for every other page and for a post without one.

import { escape } from "./html.ts";

/** A page's social image. */
export interface SocialImage {
  /** Where it is: an http(s) address, or one relative to the blog's address (`why-compmd/cover.png`). */
  url: string;
  /** What it shows, for a reader who can't see it. */
  alt: string;
}

const isText = (value: unknown): value is string => typeof value === "string" && value.trim() !== "";

/** `url`, an image's, as an absolute address, against the blog's address `base`; `undefined` when it isn't http(s). */
function addressIn(base: string, url: string): string | undefined {
  try {
    const address = new URL(url, base);
    return address.protocol === "https:" || address.protocol === "http:" ? address.href : undefined;
  } catch {
    return undefined;
  }
}

/**
 * `given`, the field `field`, read as a social image, its address made absolute against the blog's address `base`; or
 * what is wrong with it.
 *
 * @example
 * socialImageOf({ url: "cover.png", alt: "A cover" }, "socialImage", "https://example.com/blog/");
 * // { image: { url: "https://example.com/blog/cover.png", alt: "A cover" } }
 */
export function socialImageOf(given: unknown, field: string, base: string): { image: SocialImage } | { problem: string } {
  const fields = (typeof given === "object" && given !== null ? given : {}) as Record<string, unknown>;
  const url = isText(fields.url) ? addressIn(base, fields.url) : undefined;
  if (url === undefined || !isText(fields.alt)) return { problem: `\`${field}\` has a \`url\` (an http(s) address, or one in the blog) and an \`alt\`` };
  return { image: { url, alt: fields.alt } };
}

/** A page's head tags for its social image: the Open Graph image and its alt, and the large card; none without one. */
export function socialImageTags(image: SocialImage | undefined): string[] {
  if (image === undefined) return [];
  return [
    `<meta property="og:image" content="${escape(image.url)}">`,
    `<meta property="og:image:alt" content="${escape(image.alt)}">`,
    `<meta name="twitter:card" content="summary_large_image">`,
  ];
}
