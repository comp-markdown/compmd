// The files of the document's own fonts a use's box's type names (spec §9.4), for its frame, which has no network and
// isn't reached by the page's `@font-face` rules: in the host's page, those of the faces `@compmd/render` declared,
// fetched once each; in a frame, those its own parent sent it.

import { fontFacesIn } from "@compmd/render";
import { heldFontsIn } from "./frame/frame-fonts.ts";
import type { FrameFont } from "./protocol/messages.ts";

/** Each face file's contents, by its URL, fetched once. */
const fetched = new Map<string, Promise<Blob | undefined>>();

function fileAt(url: string): Promise<Blob | undefined> {
  let file = fetched.get(url);
  if (!file) {
    file = fetch(url, { credentials: "same-origin" }).then(
      (response) => (response.ok ? response.blob() : undefined),
      () => undefined,
    );
    fetched.set(url, file);
  }
  return file;
}

/** Whether `family`, a computed `font-family`, names a font of the document's own here. */
export function namesFonts(family: string): boolean {
  return fontFacesIn(family).length > 0 || heldFontsIn(family).length > 0;
}

/**
 * The files of the faces `family` names: those this window's frame was sent, and those the page declared, fetched; a
 * file that can't be had is left out, so its face's fallback shows.
 *
 * @example
 * await fontFilesOf(getComputedStyle(box).fontFamily); // [{ name: "compmd-font-0-serif", weight: "normal", style: "normal", file: Blob }]
 */
export async function fontFilesOf(family: string): Promise<FrameFont[]> {
  const faces = await Promise.all(
    fontFacesIn(family).map(async ({ name, weight, style, url }) => {
      const file = await fileAt(url);
      return file ? [{ name, weight, style, file }] : [];
    }),
  );
  return [...heldFontsIn(family), ...faces.flat()];
}
