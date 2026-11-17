// The processor's WebAssembly, loaded once for everything that reads posts.

import { readFileSync } from "node:fs";
import { initSync } from "@compmd/processor";

let loaded = false;

/** Loads the processor's module, once. */
export function loadProcessor(): void {
  if (loaded) return;
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
  loaded = true;
}
