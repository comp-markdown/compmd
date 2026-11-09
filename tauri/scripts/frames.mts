// The frames the crate embeds (docs/tauri-plugin.md, decision 2): `@compmd/sandbox`'s built frames, as
// `buildHashedFrames` writes them for the scheme `compmd`, into `tauri/frames/`. Run by `cargo make tauri-frames`,
// after the sandbox's build, and before the crate is built or published.

import { fileURLToPath } from "node:url";
import { buildHashedFrames } from "../../sandbox/src/build/index.ts";

const out = fileURLToPath(new URL("../frames/", import.meta.url));
const from = fileURLToPath(new URL("../../sandbox/dist/frames/", import.meta.url));
const { hash } = await buildHashedFrames({ out, from, source: "compmd://localhost/frames/" });
console.log(`tauri-plugin-compmd: the frames in tauri/frames/${hash}/`);
