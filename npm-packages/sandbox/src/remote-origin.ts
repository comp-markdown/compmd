// Which origin an image is from (spec §15, audit S15): the URL the page loads it from, its `src` read against the
// page's base as the browser reads it, so `//tracker.example/p.png` and `\\tracker.example/p.png` are the tracker's,
// not the page's own.

/**
 * The origin the reader is asked about for the image `src`: that of `new URL(src, base)`, the URL the page loads, when
 * it is an `http:` or `https:` URL on another origin than `base`'s; `undefined` for one on the page's own origin (the
 * document's files, a pack's), one of another scheme (the renderer lets only the host's own through), and text that
 * isn't a URL.
 *
 * @example
 * remoteOrigin("./logo.png", "https://docs.example.com/notes/"); // undefined
 * remoteOrigin("//tracker.example/p.png", "https://docs.example.com/notes/"); // "https://tracker.example"
 */
export function remoteOrigin(src: string, base: string): string | undefined {
  let url: URL;
  try {
    url = new URL(src, base);
  } catch {
    return undefined;
  }
  if (url.protocol !== "http:" && url.protocol !== "https:") return undefined;
  return url.origin === new URL(base).origin ? undefined : url.origin;
}
