// The editor's snippet the page shows, run as it is written: it renders into the document's body. The other tabs show
// the guide's quick start, which the guide's tests build and run, and the site's own pack, which the page runs live.
import { readFileSync } from "node:fs";
import { initSync } from "@compmd/processor";
import { beforeAll, beforeEach, expect, it } from "vitest";

beforeAll(() => {
  // In a browser `init` fetches the module; here it is loaded first, and `init` finds it loaded.
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
});

beforeEach(() => {
  document.body.replaceChildren();
});

it("keeps a document open, and renders an edit", async () => {
  await import("./editor.ts");
  expect(document.querySelector("h1")?.textContent).toBe("Hello there");
});
