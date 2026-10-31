// The sandbox's frame documents for the playground, in `public/sandbox/`: the runtimes `@compmd/sandbox` ships, and d3,
// which the examples' charts import.

import { buildFrames, buildLibraries, RUNTIMES } from "@compmd/sandbox/build";
import { rm } from "node:fs/promises";
import { join } from "node:path";

const ROOT = join(import.meta.dirname, "..");
const OUT = join(ROOT, "public/sandbox");

await rm(OUT, { recursive: true, force: true });
// The playground's own libraries first: the frame documents' files go over them, with the runtimes' import map.
const imports = await buildLibraries([{ name: "d3", imports: ["d3"] }], { root: ROOT, out: join(OUT, "libs"), shared: RUNTIMES });
await buildFrames({ out: OUT, imports });
console.log("Frame documents in public/sandbox/, with d3");
