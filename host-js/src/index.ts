// @compmd/host: the serving contract of a CompMD host over the Fetch API (docs/hosting.md, decision 8). A bundle in a
// folder, for Node, is `@compmd/host/node`'s, which also loads the WebAssembly from its file.
export { respond, type Bundle, type Place, type Store } from "./respond.ts";
/** Loads the WebAssembly `respond` decides with: `respond` does it once by itself, from its file next to this one. */
export { default as init, initSync } from "../pkg/compmd_host_wasm.js";
