// A bundle whose frames' runtime is missing, for `--check-broken`: the bundle the page's build wrote, copied, then
// without `runtime.js`, so each frame never loads it and the sandbox must say so. The app reads it at run time.

import { cp, readFile, rm } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const bundle = fileURLToPath(new URL("../src-tauri/compmd/", import.meta.url));
const broken = fileURLToPath(new URL("../src-tauri/broken/", import.meta.url));
const { frames } = JSON.parse(await readFile(join(bundle, "manifest.json"), "utf8")) as { frames: { hash: string } };
await rm(broken, { recursive: true, force: true });
await cp(bundle, broken, { recursive: true });
await rm(join(broken, "frames", frames.hash, "runtime.js"));
console.log(`broken bundle: ${broken}`);
