// A pack's image's media type, by its extension, for the `Blob` a host shows it from: an image loads only with its
// type (an SVG as `image/svg+xml`). Only images are given as blobs: a link opened from a `blob:` URL would run the
// pack's page or SVG with the host's page's origin, so a pack's other files never get one.

const IMAGE_TYPES: Readonly<Record<string, string>> = {
  svg: "image/svg+xml",
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  gif: "image/gif",
  webp: "image/webp",
  avif: "image/avif",
  ico: "image/x-icon",
};

/**
 * The image type of the file at `path`, by its extension; `undefined` for a file that isn't an image.
 *
 * @example
 * imageTypeOf("icons/star.svg"); // "image/svg+xml"
 * imageTypeOf("guide.comp.md"); // undefined
 */
export function imageTypeOf(path: string): string | undefined {
  const extension = /\.([^./]+)$/.exec(path)?.[1]?.toLowerCase() ?? "";
  return IMAGE_TYPES[extension];
}
