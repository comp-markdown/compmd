// The media type of a pack's file a frame may load, by its extension, for the `Blob` a host gives it from: an image
// (an SVG as `image/svg+xml`) or a font's file (spec §9.4). Only these are given as blobs: a link opened from a
// `blob:` URL would run the pack's page or script with the host's page's origin, so a pack's other files never get one.

const FILE_TYPES: Readonly<Record<string, string>> = {
  svg: "image/svg+xml",
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  gif: "image/gif",
  webp: "image/webp",
  avif: "image/avif",
  ico: "image/x-icon",
  woff2: "font/woff2",
  woff: "font/woff",
  ttf: "font/ttf",
  otf: "font/otf",
};

/**
 * The media type of the file at `path` when a frame may load it, an image or a font's file, by its extension;
 * `undefined` for any other file.
 *
 * @example
 * fileTypeOf("icons/star.svg"); // "image/svg+xml"
 * fileTypeOf("fonts/harbour.woff2"); // "font/woff2"
 * fileTypeOf("guide.comp.md"); // undefined
 */
export function fileTypeOf(path: string): string | undefined {
  const extension = /\.([^./]+)$/.exec(path)?.[1]?.toLowerCase() ?? "";
  return FILE_TYPES[extension];
}
