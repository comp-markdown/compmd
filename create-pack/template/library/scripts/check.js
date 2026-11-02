// Before publishing: the pack's version is the library's, and its module is there to publish.
import { existsSync, readFileSync } from "node:fs";

const manifest = JSON.parse(readFileSync("package.json", "utf8"));
const library = JSON.parse(readFileSync("node_modules/__library__/package.json", "utf8"));
const problems = [];
if (manifest.version !== library.version) problems.push(`package.json's version is ${manifest.version}, __library__'s is ${library.version}: a library pack's version is its library's, which npm run build writes`);
if (!existsSync(manifest.compmd.library)) problems.push(`${manifest.compmd.library} is missing: run npm run build`);
if (problems.length > 0) {
  console.error(problems.join("\n"));
  process.exit(1);
}
