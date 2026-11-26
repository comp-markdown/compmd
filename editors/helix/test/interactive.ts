// An interactive Helix with the drafts against local builds, in place of the writer's own config.
// `cargo make helix-interactive [file]` builds what it needs and starts it:
//
//   node editors/helix/test/interactive.ts <built grammars' folder> <compmd-lsp> [file]
//
// Without a file, it opens a copy of the Author's Guide's containers example, in a folder of its own, to edit freely.

import { execFileSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { drafts, helixSetup } from "./setup.ts";

const [grammars = "", server = "", file] = process.argv.slice(2).map((path) => resolve(path));
const { work, environment } = helixSetup(grammars, server);
const opened = file ?? join(work, "garden.comp.md");
if (!file) copyFileSync(join(drafts, "..", "..", "guides", "authors", "examples", "containers", "garden.comp.md"), opened);
execFileSync("hx", [opened], { cwd: work, env: environment, stdio: "inherit" });
