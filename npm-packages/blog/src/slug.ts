// A name as an address's part: a post's file from its title, a tag's page from the tag.

/**
 * `text` as a slug: lowercase letters and digits, words joined by `-`, accents dropped.
 *
 * @example
 * slugOf("Why CompMD?"); // "why-compmd"
 * slugOf("Café & Crème"); // "cafe-creme"
 */
export function slugOf(text: string): string {
  return text
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, "-")
    .replace(/^-+|-+$/g, "");
}
