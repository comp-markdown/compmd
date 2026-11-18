// The serving contract's headers (docs/hosting.md, decision 8), which every file of the bundle is served with, in every
// host: `@compmd/vite`'s servers and `_headers`, `compmd-host`'s `respond`, and the check that a host sends them.

/**
 * The headers of each file of the bundle: frames have an opaque origin, so their files load cross-origin and a host's
 * check reads the headers; a frame document is sandboxed even when opened on its own; a path's file never changes.
 */
export const CONTRACT_HEADERS: Readonly<Record<string, string>> = {
  "Content-Security-Policy": "sandbox allow-scripts",
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Expose-Headers": "*",
  "X-Content-Type-Options": "nosniff",
  "Cache-Control": "max-age=31536000, immutable",
};
