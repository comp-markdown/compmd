import { createHash } from "node:crypto";
import { describe, expect, it } from "vitest";
import { framePolicy } from "../../frame-policy.ts";

describe("framePolicy", () => {
  it("lets the frame run the site's scripts, its compiled components and its import map, and reach no network", () => {
    const map = '{"imports":{"react":"./libs/react@19.3.0/index.js"}}';
    const policy = framePolicy(map, false);
    const hash = createHash("sha256").update(map).digest("base64");
    expect(policy.split("; ")).toEqual([
      "default-src 'none'",
      `script-src 'self' blob: 'sha256-${hash}'`,
      "style-src 'self' 'unsafe-inline'",
      "img-src data: blob:",
      "font-src 'self'",
    ]);
  });

  it("lets the dev server's live reload connect", () => {
    expect(framePolicy("{}", true)).toContain("connect-src 'self' ws: wss:");
  });
});
