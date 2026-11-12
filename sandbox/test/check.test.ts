// The hosting check (docs/hosting.md, decision 11), in Chrome and WebKit: a host that serves the bundle as the
// contract says passes it, and a host that breaks one rule fails it, saying which.

import { afterEach, describe, expect, it } from "vitest";
import { inject } from "vitest";
import { runCheck, type CheckOptions, type CheckReport } from "../src/check/index.ts";
import { CHECK_HASH } from "./fixtures/hosts.ts";

const hosts = inject("hosts");
const FRAMES = `frames/${CHECK_HASH}/`;
const drawn = { pack: "@kit/drawn", version: "1.0.0", path: "dist/drawn.js" };

/** The file at `path` on `host`'s origin, as an integrity hash. */
async function integrityOf(host: string, path: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-384", await (await fetch(`${hosts}${host}/${path}`)).arrayBuffer());
  return `sha384-${btoa(String.fromCharCode(...new Uint8Array(digest)))}`;
}

const containers: HTMLElement[] = [];
afterEach(() => {
  for (const container of containers.splice(0)) container.remove();
  delete (globalThis as { __COMPMD__?: unknown }).__COMPMD__;
});

/** The check on the host `host`, its origin given by the adapter, with a pack served there and `more` options. */
async function checked(host: string, more: Partial<CheckOptions> = {}): Promise<CheckReport> {
  (globalThis as { __COMPMD__?: unknown }).__COMPMD__ = { origin: `${hosts}${host}` };
  const container = document.body.appendChild(document.createElement("div"));
  containers.push(container);
  const integrity = await integrityOf("good", "packs/drawn.js");
  return runCheck({ container, frames: FRAMES, modules: () => ({ path: "packs/drawn.js", integrity }), packs: [{ place: drawn }], hostApi: ["__TAURI_INTERNALS__"], timeout: 8000, ...more });
}

describe("the hosting check", () => {
  it("passes a host that serves the bundle as the contract says", async () => {
    const report = await checked("good");
    expect(report.failed).toEqual([]);
    expect(report.ok).toBe(true);
    expect(report.passed).toEqual(
      expect.arrayContaining([
        "a strict frame loads from the origin",
        "a strict frame's origin is opaque",
        "a strict frame's `fetch` is refused",
        "a strict frame has no WebKit `ipc` message handler",
        "a strict frame has no `__TAURI_INTERNALS__`",
        "a code block is highlighted",
        "`@kit/drawn@1.0.0/dist/drawn.js` loads by its URL and draws",
      ]),
    );
    expect(report.message).toMatch(/^passed: /);
  }, 30_000);

  it("fails a host that leaves out the contract's headers", async () => {
    const report = await checked("loose");
    expect(report.ok).toBe(false);
    expect(report.failed.some((failure) => failure.includes("strict.html` is served with the contract's headers: Content-Security-Policy is null"))).toBe(true);
  }, 30_000);

  it("fails a host that serves what is outside the contract", async () => {
    const report = await checked("lax");
    expect(report.failed.filter((failure) => failure.includes("outside the contract, is a 404: it answered 200"))).toHaveLength(2);
  }, 30_000);

  it("fails a host whose page can't load the frames' highlighter", async () => {
    const report = await checked("plain");
    expect(report.failed.some((failure) => failure.startsWith("a code block is highlighted:"))).toBe(true);
  }, 30_000);

  it("fails frames that can fetch", async () => {
    const report = await checked("open");
    expect(report.failed).toContain("a strict frame's `fetch` is refused: it fetched the frames' own document");
  }, 30_000);

  it("fails a frame that sees the host's API", async () => {
    // A global every frame has stands in for an API an adapter leaked into its frames.
    const report = await checked("good", { hostApi: ["postMessage"] });
    expect(report.failed).toContain("a strict frame has no `postMessage`: `postMessage` is there: a frame's code could reach the host");
  }, 30_000);

  it("leaves WebKit's `ipc` handler to a host that guards it", async () => {
    const report = await checked("good", { webkitIpc: "guarded" });
    expect(report.ok).toBe(true);
    expect(report.passed).toContain("a strict frame's WebKit `ipc` message handler, if any, is the host's to guard");
  }, 30_000);

  it("fails a pack the host gives as code, or not at all", async () => {
    const asCode = await checked("good", { modules: () => ({ code: new Blob(['export default function mount(target) { target.textContent = "drawn"; }\n']) }) });
    expect(asCode.failed).toContain("`@kit/drawn@1.0.0/dist/drawn.js` loads by its URL and draws: the host has no URL for it");
    const missing = await checked("good", { modules: () => undefined });
    expect(missing.failed).toContain("`@kit/drawn@1.0.0/dist/drawn.js` loads by its URL and draws: the host has no URL for it");
  }, 30_000);

  it("fails at once without an origin", async () => {
    const container = document.body.appendChild(document.createElement("div"));
    containers.push(container);
    expect(await runCheck({ container, frames: FRAMES })).toEqual({
      ok: false,
      passed: [],
      failed: ["the bundle has an origin: neither the options nor the host's adapter (`__COMPMD__`) give one"],
      message: "failed: the bundle has an origin: neither the options nor the host's adapter (`__COMPMD__`) give one",
    });
  });
});
