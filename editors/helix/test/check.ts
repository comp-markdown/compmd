// The Helix drafts checked in a real Helix, against local builds: Helix runs in a tmux session of its own, with
// languages.toml and the queries as its config and runtime, and the check reads its screen. `cargo make helix-check`
// builds what it needs and runs it:
//
//   node editors/helix/test/check.ts <built grammars' folder> <compmd-lsp>
//
// It exits 0 when everything holds, else 1, saying what didn't.

import { execFileSync } from "node:child_process";
import { chmodSync, existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { helixSetup } from "./setup.ts";

const [grammars = "", server = ""] = process.argv.slice(2).map((path) => resolve(path));
const failures: string[] = [];

function check(holds: boolean, what: string): void {
  console.log(`${holds ? "ok  " : "FAIL"} ${what}`);
  if (!holds) failures.push(what);
}

// A browser that writes down the address it is given; diagnostics at the end of their line, so the check reads them.
const recorder = mkdtempSync(join(tmpdir(), "compmd-helix-browser-"));
const opened = join(recorder, "opened");
const browser = join(recorder, "browser");
writeFileSync(browser, `#!/bin/sh\necho "$1" > '${opened}'\n`);
chmodSync(browser, 0o755);
const { root, work, environment } = helixSetup(grammars, server, browser, '[editor]\nend-of-line-diagnostics = "hint"\n');
// A component its project doesn't have (F002), an interpolation, an expression and a definition.
writeFileSync(
  join(work, "notes.comp.md"),
  [
    "---",
    "title: Harbour notes",
    "---",
    "",
    "# {{md.doc.title}}",
    "",
    "::: nowhere {wide=(1 + 2)}",
    "The tide is :mark[in].",
    "::: /nowhere",
    "",
    "```compmd component=tide",
    "params:",
    "  level: number",
    "---",
    "The tide is {{level}} metres.",
    "```",
    "",
  ].join("\n"),
);


const health = (language: string): string =>
  execFileSync("hx", ["--health", language], { cwd: work, env: environment, encoding: "utf8" });
check(health("compmd").includes(server), "the language server is the entry's compmd-lsp");
check(health("markdown").includes(server), "Markdown files have compmd-lsp too, after Helix's own servers");
for (const language of ["compmd", "compmd-inline"]) {
  const report = health(language);
  for (const part of ["Tree-sitter parser", "Highlight queries", "Textobject queries"]) {
    check(new RegExp(`${part}: \\S*✓`).test(report), `${language}: ${part.toLowerCase()} found`);
  }
}

const tmux = (...args: string[]): string =>
  execFileSync("tmux", ["-L", "compmd-helix-check", "-f", "/dev/null", ...args], { encoding: "utf8", stdio: "pipe" });
const screen = (): string => tmux("capture-pane", "-p", "-t", "helix");
const keys = (...keys: string[]): string => tmux("send-keys", "-t", "helix", ...keys);

async function until(holds: (text: string) => boolean): Promise<boolean> {
  for (let tries = 0; tries < 100; tries += 1) {
    if (holds(screen())) return true;
    await sleep(200);
  }
  return false;
}

// The highlights at a place, as `:tree-sitter-highlight-name` shows them, in a popup on the line below. An Escape sent
// with the keys after it would reach Helix as Alt and the next key, so it goes on its own.
async function highlightAt(line: number, column: number): Promise<string> {
  keys("Escape");
  await sleep(200);
  keys(`:goto ${line}`, "Enter");
  if (column > 0) keys(`${column}l`);
  keys(":tree-sitter-highlight-name", "Enter");
  await sleep(300);
  return screen().split("\n")[line + 1] ?? "";
}

tmux(
  "new-session", "-d", "-s", "helix", "-x", "160", "-y", "40", "-c", work,
  "-e", `HELIX_RUNTIME=${environment["HELIX_RUNTIME"] ?? ""}`, "-e", `XDG_CONFIG_HOME=${environment["XDG_CONFIG_HOME"] ?? ""}`,
  "hx", "-vv", "--log", join(root, "helix.log"), "notes.comp.md",
);
try {
  check(await until((text) => text.includes("notes.comp.md")), "Helix opens the document");
  for (const [line, column, name, what] of [
    [7, 4, "tag", "a container's name"],
    [7, 13, "attribute", "a param's name"],
    [5, 4, "variable", "an interpolation's path (compmd-inline, injected)"],
    [8, 13, "tag", "an inline component's name (compmd-inline, injected)"],
    [7, 19, "constant.numeric", "an expression's number (cel, injected)"],
    [11, 20, "tag", "a definition's name"],
  ] as const) {
    const status = await highlightAt(line, column);
    check(status.includes(name), `${what} is ${name}${status.includes(name) ? "" : `; the popup's line: ${status.trim()}`}`);
  }

  check(await until((text) => text.includes("unknown component `nowhere`")), "compmd-lsp diagnoses the unknown component");

  keys("Escape");
  await sleep(200);
  keys(":goto 1", "Enter", "Space", "a");
  check(await until((text) => text.includes("Open preview")), "Open preview is a code action");
  // The menu opens below the first line, its first action chosen.
  const below = screen().split("\n").findIndex((row) => row.includes("Open preview")) - 1;
  for (let step = 0; step < below; step += 1) keys("Down");
  keys("Enter");
  for (let tries = 0; tries < 100 && !existsSync(opened); tries += 1) await sleep(200);
  const url = existsSync(opened) ? readFileSync(opened, "utf8").trim() : "";
  check(/^http:\/\/127\.0\.0\.1:\d+\/[0-9a-f]+\/$/.test(url), "Open preview opens the browser at the page");
  const page = url ? await fetch(url) : undefined;
  check(page?.status === 200, "the page is served");
} finally {
  keys("Escape");
  await sleep(200);
  keys(":quit!", "Enter");
  await sleep(200);
  // Helix quitting ends the session, and tmux's server with its last one; the server is killed only if it is still up.
  try {
    tmux("kill-server");
  } catch {
    // Gone already.
  }
}

// `--health` finds the query files but doesn't compile them; Helix says in its log which it couldn't.
const unreadable = readFileSync(join(root, "helix.log"), "utf8").split("\n").filter((line) => line.includes("Failed to compile"));
check(unreadable.length === 0, `Helix compiles every query${unreadable.length ? `: ${unreadable.join("; ")}` : ""}`);

if (failures.length > 0) {
  console.log(`${failures.length} failed (Helix's log: ${join(root, "helix.log")})`);
  process.exit(1);
}
console.log("all hold");
