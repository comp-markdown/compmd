// The spec's companion, *CompMD packs on a registry*, as a section of the spec's page: where it is, and where the links
// between the two go, within the page, since a reader of one is a click from the other.

import { renderMarkdown, type RenderedSpec } from "./spec.ts";

/** The spec's file, and its companion's, in the repository. */
export const SPEC = "compmd-spec-v1.md";
export const COMPANION = "compmd-registry-v1.md";

/** The id of the companion's section on the spec's page; its own ids are under it (`registry-4-who-fetches`). */
export const COMPANION_SECTION = "registry";

/**
 * Where a link written in the spec or its companion goes on the page that holds both: a link to the other, or to a
 * section of it, within the page; `undefined` for any other link, which stays as it is.
 *
 * @example
 * betweenThem("compmd-registry-v1.md#4-who-fetches"); // "#registry-4-who-fetches"
 * betweenThem("compmd-spec-v1.md#64-packages-and-the-registry"); // "#64-packages-and-the-registry"
 */
export function betweenThem(href: string): string | undefined {
  const [file = "", fragment] = href.split("#");
  const path = file.replace(/^\.\//, "");
  if (path === COMPANION) return fragment ? `#${COMPANION_SECTION}-${fragment}` : `#${COMPANION_SECTION}`;
  if (path === SPEC) return `#${fragment ?? ""}`;
  return undefined;
}

/**
 * The spec's page: the spec, then its companion as a section of it, each from `read` (a file of the repository by its
 * path, at one version: `undefined` when that version has none), with their contents together. `blob` is the
 * repository's file URL at that version, ending in `/`, which each heading's source link starts with.
 */
export async function renderSpecPage(read: (path: string) => string | undefined, blob: string): Promise<RenderedSpec> {
  const spec = await renderMarkdown(read(SPEC) ?? "", { blob, path: SPEC }, { rules: true, link: betweenThem });
  const written = read(COMPANION);
  if (written === undefined) return spec;
  const companion = await renderMarkdown(written, { blob, path: COMPANION }, { rules: true, link: betweenThem, section: COMPANION_SECTION });
  return {
    html: `${spec.html}<section class="companion" aria-labelledby="${COMPANION_SECTION}">${companion.html}</section>`,
    toc: [...spec.toc, ...companion.toc],
  };
}
