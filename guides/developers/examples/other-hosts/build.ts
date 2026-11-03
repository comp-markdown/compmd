// Builds the sandbox's frame documents into the folder the server serves them from: `node build.ts public/compmd`.
import { buildFrames } from "@compmd/sandbox/build";

const [out = "public/compmd"] = process.argv.slice(2);
const { runtimes } = await buildFrames({ out });
console.log(`The frames are in ${out}, with ${runtimes.map(({ name, version }) => `${name} ${version}`).join(", ")}.`);
