// Builds the bundle the server serves, from the app's folder, the one with its `package.json` and `node_modules`:
// `node build.ts bundle compmd.config.json`.
import { readFile } from "node:fs/promises";
import { buildBundle } from "@compmd/vite";

const [out = "bundle", configFile = "compmd.config.json"] = process.argv.slice(2);
const config: unknown = JSON.parse(await readFile(configFile, "utf8"));
const { manifest, warnings } = await buildBundle({ out, config });
for (const warning of warnings) console.warn(warning);
console.log(`The bundle is in ${out}: the frames at frames/${manifest.frames.hash}/, and ${manifest.packs.map(({ pack, version }) => `${pack}@${version}`).join(", ")}.`);
