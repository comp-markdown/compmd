// Frames whose runtime is missing, for `--check-broken`: the sandbox's frames, built as an app builds its own, then
// without `runtime.js`, so each frame never loads it and the sandbox must say so.

import { rm } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { buildHashedFrames } from "@compmd/sandbox/build";

const out = fileURLToPath(new URL("../src-tauri/broken/", import.meta.url));
const { folder } = await buildHashedFrames({ out, source: "compmd://localhost/frames/" });
await rm(join(folder, "runtime.js"));
console.log(`broken frames: ${folder}`);
