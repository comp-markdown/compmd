#!/usr/bin/env node
// `compmd-lsp`: runs the language server's binary for this platform, from its package (`@compmd/lsp-<os>-<cpu>`, an
// optional dependency npm installs only where it runs), with the same arguments and standard input and output.

import { spawnSync } from "node:child_process";
import { binaryPath } from "./binary.ts";

const binary = binaryPath(process.platform, process.arch, (specifier) => import.meta.resolve(specifier));
if (binary === undefined) {
  process.stderr.write(
    `compmd-lsp has no binary for ${process.platform} ${process.arch}: it is released for macOS, Linux and Windows on arm64 and x64. Install it with cargo (cargo install compmd-lsp) instead.\n`,
  );
  process.exit(1);
}
const run = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
if (run.error) {
  process.stderr.write(`compmd-lsp couldn't start ${binary}: ${run.error.message}\n`);
  process.exit(1);
}
process.exit(run.status ?? 1);
