// The origins a blog's posts reach without asking a reader (spec §15, a host's own documents): those its
// `compmd.config.json` names, or every origin; and always the blog's own, where its readers already are.

import type { BlogConfig } from "./config.ts";

/**
 * The origins the blog `config`'s posts reach without a question: `configured` (the `origins` of its
 * `compmd.config.json`) and the blog's own origin, or `"any"`.
 *
 * @example
 * blogOrigins({ url: "https://example.com/blog/", … }, ["https://cdn.example.com"]);
 * // ["https://cdn.example.com", "https://example.com"]
 */
export function blogOrigins(config: BlogConfig, configured: readonly string[] | "any" = []): string[] | "any" {
  if (configured === "any") return "any";
  const own = new URL(config.url).origin;
  return configured.includes(own) ? [...configured] : [...configured, own];
}
