// Checks a CompMD document as a build would: what it needs fetched, every diagnostic with its fix, and whether any is
// an error. Run it with Node: `node check.ts <folder> <document>`, such as `node check.ts . quarter.comp.md`.
import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join, relative, sep } from "node:path";
import { argv, exit } from "node:process";
import { initSync, process as processDocument, sources, type Diagnostic, type Request } from "@compmd/processor";

// In Node the module is loaded from its file; in a browser, `await init()` fetches it.
initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });

/** The text files under `folder` a document may read, by their `/`-separated paths in it. */
export function filesOf(folder: string): Record<string, string> {
  return Object.fromEntries(
    readdirSync(folder, { recursive: true, withFileTypes: true })
      .filter((entry) => entry.isFile() && /\.(comp\.md|csv|tsv|json|ya?ml)$/.test(entry.name))
      .map((entry) => {
        const path = join(entry.parentPath, entry.name);
        return [relative(folder, path).split(sep).join("/"), readFileSync(path, "utf8")];
      }),
  );
}

/** A diagnostic as a line: where it is, its code and message, and the edit that fixes it, when it has one. */
export function lineOf(document: string, { file, range, code, message, fix }: Diagnostic): string {
  const at = `${file ?? document}:${String(range.start.line)}:${String(range.start.column)}`;
  const fixed = fix ? `\n  fix: write \`${fix.text}\` at ${String(fix.range.start.line)}:${String(fix.range.start.column)}-${String(fix.range.end.column)}` : "";
  return `${at} ${code} ${message}${fixed}`;
}

/** Checks `document`, one of `folder`'s files, rendered at `now`: the lines to print, and whether it has an error. */
export function check(folder: string, document: string, now: string): { lines: string[]; failed: boolean } {
  const request: Request = { path: document, files: filesOf(folder), now };
  const { remote, host } = sources(request);
  const needed = [...remote, ...host.map((source) => `host:${source}`)].map((source) => `${document}: needs ${source}, which a check doesn't fetch`);
  const { diagnostics } = processDocument(request);
  return { lines: [...needed, ...diagnostics.map((diagnostic) => lineOf(document, diagnostic))], failed: diagnostics.some(({ severity }) => severity === "error") };
}

if (import.meta.main) {
  const [folder = ".", document = "index.comp.md"] = argv.slice(2);
  const { lines, failed } = check(folder, document, new Date().toISOString());
  console.log(lines.length === 0 ? `${document}: no diagnostics` : lines.join("\n"));
  if (failed) exit(1);
}
