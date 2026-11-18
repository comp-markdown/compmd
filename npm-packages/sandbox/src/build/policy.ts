// The frame documents' content security policies (docs/running-components-in-the-sandbox.md, Q3). A strict frame's
// is fixed: no network, no frames. A permissive frame's first policy leaves out the network directives and nested
// frames, and the frame adds a second one with its pack's origins and `strict.html` next to it (`narrowed`) before any
// component loads; a browser enforces both, so the second only narrows.

import { createHash } from "node:crypto";
import { withSource } from "../frame/source.ts";

/** The directives both documents start with: scripts from the frames' origin and the import map, styles, no objects. */
function common(importMap: string): string[] {
  const hash = createHash("sha256").update(importMap).digest("base64");
  return [`script-src 'self' blob: 'sha256-${hash}'`, "style-src 'self' 'unsafe-inline'", "object-src 'none'", "base-uri 'none'", "form-action 'none'", "worker-src 'none'"];
}

/**
 * `strict.html`'s policy, with `importMap` allowed by its hash: no requests anywhere, images only as `data:` and
 * `blob:`, fonts from the frames' origin, and no nested frames. With `source`, where the host serves the frames
 * and the packs (`frameSource`), each is named beside each `'self'`.
 */
export function strictPolicy(importMap: string, source?: string | readonly string[]): string {
  const policy = ["default-src 'none'", ...common(importMap), "img-src data: blob:", "font-src 'self'", "frame-src 'none'"].join("; ");
  return withSource(policy, source);
}

/**
 * `permissive.html`'s first policy: as a strict frame's, without the network directives and nested frames, which the
 * frame's second policy gives.
 */
export function permissivePolicy(importMap: string, source?: string | readonly string[]): string {
  return withSource(common(importMap).join("; "), source);
}
