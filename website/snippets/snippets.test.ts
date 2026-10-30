// The snippets the page shows, run as they are written: each renders into the document's body.
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

it("renders a document in a page", async () => {
  await import("./page.ts");
  expect(document.querySelector("h1")?.textContent).toBe("Hello");
  expect(document.querySelector(".compmd-box")?.textContent).toBe("A box, written in CompMD.");
});

it("keeps a document open, and renders an edit", async () => {
  await import("./editor.ts");
  expect(document.querySelector("h1")?.textContent).toBe("Hello there");
});
