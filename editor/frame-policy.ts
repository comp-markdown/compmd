// The frame's content security policy, spec §15's "no network" for host-language components.

import { createHash } from "node:crypto";

/**
 * The policy of the frame whose import map is `importMap`: scripts from the site (its own, the shared libraries, the
 * compilers), the components compiled into `blob:` modules, and the import map by its hash; styles from the site and
 * inline; images only as `data:` and `blob:`; no requests, sockets, frames or workers. The dev server's live reload
 * needs its socket and requests, so there (`serving`) the frame may connect to the site.
 */
export function framePolicy(importMap: string, serving: boolean): string {
  const hash = createHash("sha256").update(importMap).digest("base64");
  return [
    "default-src 'none'",
    `script-src 'self' blob: 'sha256-${hash}'`,
    "style-src 'self' 'unsafe-inline'",
    "img-src data: blob:",
    "font-src 'self'",
    ...(serving ? ["connect-src 'self' ws: wss:"] : []),
  ].join("; ");
}
