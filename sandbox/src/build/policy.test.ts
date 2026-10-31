import { createHash } from "node:crypto";
import { describe, expect, it } from "vitest";
import { permissivePolicy, strictPolicy } from "./policy.ts";

const map = '{"imports":{"react":"./libs/react@19.3.0/index.js"}}';
const hash = createHash("sha256").update(map).digest("base64");

describe("the frame documents' policies", () => {
  it("lets a strict frame run the frames' scripts, compiled components and its import map, and reach nothing", () => {
    expect(strictPolicy(map).split("; ")).toEqual([
      "default-src 'none'",
      `script-src 'self' blob: 'sha256-${hash}'`,
      "style-src 'self' 'unsafe-inline'",
      "object-src 'none'",
      "base-uri 'none'",
      "form-action 'none'",
      "worker-src 'none'",
      "img-src data: blob:",
      "font-src 'self'",
      "frame-src 'none'",
    ]);
  });

  it("leaves a permissive frame's network and nested frames to its second policy", () => {
    const policy = permissivePolicy(map).split("; ");
    expect(policy).toEqual([
      `script-src 'self' blob: 'sha256-${hash}'`,
      "style-src 'self' 'unsafe-inline'",
      "object-src 'none'",
      "base-uri 'none'",
      "form-action 'none'",
      "worker-src 'none'",
    ]);
    for (const network of ["default-src", "connect-src", "img-src", "font-src", "media-src", "frame-src"]) {
      expect(policy.some((directive) => directive.startsWith(network))).toBe(false);
    }
  });
});
