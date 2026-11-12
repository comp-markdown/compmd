// The serving contract's headers (docs/hosting.md, decision 8), which every file of the bundle is served with: the
// dev and preview servers set them, and a `_headers` file gives them to a static host that reads one (H2). No Node.

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

/** The bundle's folders a host serves, under its origin: `manifest.json` isn't one, being the host's. */
export const SERVED = ["frames/", "packs/"] as const;

/**
 * A `_headers` file (as Netlify and Cloudflare Pages read it) giving the bundle's files under `prefix` the contract's
 * headers.
 *
 * @param prefix The bundle's URL path on the site, ending in `/`: `/compmd/` for an app's `compmd/` folder.
 * @example
 * headersFile("/compmd/"); // "/compmd/frames/*\n  Content-Security-Policy: sandbox allow-scripts\n…"
 */
export function headersFile(prefix: string): string {
  const lines = Object.entries(CONTRACT_HEADERS).map(([name, value]) => `  ${name}: ${value}`);
  return SERVED.map((folder) => [`${prefix}${folder}*`, ...lines].join("\n")).join("\n\n") + "\n";
}
