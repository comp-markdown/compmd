// The frames' own source, for a host that serves them on a scheme of its own. A frame is sandboxed, so its origin is
// opaque, and WebKit reads `'self'` in an opaque origin's policy as the response URL's origin only for http and https:
// on any other scheme (`app://`, a Tauri app's custom scheme) `'self'` matches nothing, and the frame can't load its
// own runtime. The host names where it serves the frames, and each `'self'` of their policies gets that source too.

/** A source the frames may name for themselves: a scheme and a host, with a path, and nothing a policy reads as more. */
const SOURCE = /^[a-z][a-z0-9+.-]*:\/\/[a-z0-9.-]+(:\d+)?(\/[^\s;,'"*]*)?$/i;

/**
 * `source` as the frames' policies name it, checked: a scheme, a host and a path, such as
 * `app://localhost/frames/`.
 *
 * @throws Error for a value that would widen a policy: a wildcard, a bare scheme, quotes or another directive.
 */
export function frameSource(source: string): string {
  if (!SOURCE.test(source)) throw new Error(`\`${source}\` isn't a source the frames can be served from`);
  return source;
}

/**
 * `policy` with `source` after each `'self'`, so the frames' own files load wherever they are served: one source, or
 * several (the frames' and the packs', which a frame loads built modules from); `policy` as it is without any.
 *
 * @example
 * withSource("script-src 'self' blob:", "app://localhost/frames/"); // "script-src 'self' app://localhost/frames/ blob:"
 */
export function withSource(policy: string, source: string | readonly string[] | undefined): string {
  const sources = source === undefined ? [] : typeof source === "string" ? [source] : source;
  if (sources.length === 0) return policy;
  return policy.replaceAll("'self'", ["'self'", ...sources.map(frameSource)].join(" "));
}
