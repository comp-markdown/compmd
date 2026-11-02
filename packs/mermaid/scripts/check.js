// Before publishing: the pack's document says the version package.json does (F022 otherwise), and the built module is
// there to publish, with the licences of what it bundles.
import { existsSync, readFileSync } from "node:fs";

const manifest = JSON.parse(readFileSync("package.json", "utf8"));
const document = readFileSync("index.comp.md", "utf8");
const written = /^\s*version:\s*"?([^"\s]+)"?\s*$/m.exec(document)?.[1];
const problems = [];
if (written !== manifest.version) problems.push(`index.comp.md's export.version is ${written ?? "missing"}, package.json's version is ${manifest.version}: write the same in both`);
if (!existsSync("dist/mermaid.js")) problems.push("dist/mermaid.js is missing: run npm run build");
if (!existsSync("dist/LICENSES.md")) problems.push("dist/LICENSES.md is missing: run npm run build");
if (problems.length > 0) {
  console.error(problems.join("\n"));
  process.exit(1);
}
