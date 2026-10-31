// The frame documents' content security policies (docs/sandbox.md, Q3). A strict frame's is fixed: no network, no
// frames. A permissive frame's first policy leaves out the network directives and nested frames, and the frame adds a
// second one with its pack's origins and `strict.html` next to it (`narrowed`) before any component loads; a browser
// enforces both, so the second only narrows.

import { createHash } from "node:crypto";

/** The directives both documents start with: scripts from the frames' origin and the import map, styles, no objects. */
function common(importMap: string): string[] {
  const hash = createHash("sha256").update(importMap).digest("base64");
  return [`script-src 'self' blob: 'sha256-${hash}'`, "style-src 'self' 'unsafe-inline'", "object-src 'none'", "base-uri 'none'", "form-action 'none'", "worker-src 'none'"];
}

/**
 * `strict.html`'s policy, with `importMap` allowed by its hash: no requests anywhere, images only as `data:` and
 * `blob:`, fonts from the frames' origin, and no nested frames.
 */
export function strictPolicy(importMap: string): string {
  return ["default-src 'none'", ...common(importMap), "img-src data: blob:", "font-src 'self'", "frame-src 'none'"].join("; ");
}

/**
 * `permissive.html`'s first policy: as a strict frame's, without the network directives and nested frames, which the
 * frame's second policy gives.
 */
export function permissivePolicy(importMap: string): string {
  return common(importMap).join("; ");
}
