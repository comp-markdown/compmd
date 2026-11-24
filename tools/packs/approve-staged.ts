// Approving what npm has staged of a scope's packages, for `cargo make approve-packs` (the packs) and `cargo make
// approve-release` (a release's `@compmd/` packages): checkboxes, every version waiting for the author chosen; the
// chosen ones approved in the order they were staged, with one one-time code, which npm asks for again once it has
// expired. With `dryRun` it lists them and approves nothing.

import { createInterface } from "node:readline/promises";
import checkbox from "@inquirer/checkbox";
import { readNpm, runNpm, type RunNpm } from "./npm.ts";
import { describe, stagedIn, waiting, type Staged } from "./staged.ts";

/** What `approveStaged` does through: each a stub in its tests. */
export interface Steps {
  /** `npm stage list --json`'s output. */
  list(): string;
  /** The ones the author chooses among `items`, those with `chosen` checked at first. */
  choose(items: readonly Staged[], chosen: (item: Staged) => boolean): Promise<Staged[]>;
  /** The author's one-time code, or `undefined` to let npm ask (a security key, a browser). */
  code(): Promise<string | undefined>;
  npm: RunNpm;
  log(line: string): void;
}

/** What to approve: the packages of `scope` (`@compmd`), named `what` in messages ("the packs"). */
export interface Approving {
  scope: string;
  what: string;
  dryRun: boolean;
}

/**
 * npm's arguments that approve a staged version, with the author's one-time code when there is one. npm uses the
 * code while it is valid, and asks for a new one itself once it has expired.
 */
export const approveArgs = (item: Staged, code?: string): string[] => ["stage", "approve", item.id, ...(code ? ["--otp", code] : [])];

/**
 * Approves the staged versions of `scope` the author chooses, oldest first.
 *
 * @returns The exit code: 0, also when nothing is staged or nothing chosen.
 * @throws Error when npm fails, after the versions before it are approved.
 */
export async function approveStaged({ scope, what, dryRun }: Approving, steps: Steps): Promise<number> {
  const staged = stagedIn(steps.list(), scope);
  if (staged.length === 0) {
    steps.log(`npm has nothing staged for ${what}`);
    return 0;
  }
  if (dryRun) {
    steps.log(`staged, ${String(staged.length)}; would offer them all, these chosen:`);
    for (const item of staged) steps.log(`  [${waiting(item) ? "x" : " "}] ${describe(item)}`);
    return 0;
  }
  const chosen = await steps.choose(staged, waiting);
  // In the order they were staged, whatever order the list gave them back in.
  const approving = staged.filter((item) => chosen.includes(item));
  if (approving.length === 0) {
    steps.log("nothing chosen: nothing approved");
    return 0;
  }
  const code = await steps.code();
  for (const item of approving) {
    steps.npm(approveArgs(item, code));
    steps.log(`approved ${describe(item)}`);
  }
  return 0;
}

/** The steps as they run: the pinned npm, and the checkboxes in the terminal. */
export const terminalSteps: Steps = {
  list: () => readNpm(["stage", "list", "--json"]),
  choose: (items, chosen) =>
    checkbox<Staged>({
      message: "Approve, in the order they were staged (space toggles, a toggles all, Enter approves):",
      choices: items.map((item) => ({ name: describe(item), value: item, checked: chosen(item) })),
      pageSize: 20,
    }),
  async code() {
    const terminal = createInterface({ input: process.stdin, output: process.stdout });
    const answer = (await terminal.question("One-time code from your authenticator (Enter to let npm ask, for a security key): ")).trim();
    terminal.close();
    return answer === "" ? undefined : answer;
  },
  npm: runNpm,
  log: (line) => {
    console.log(line);
  },
};

/** Runs `approveStaged` for `approving` with the terminal's steps, as a command does: its exit code, errors said. */
export async function runApproving(approving: Omit<Approving, "dryRun">): Promise<void> {
  try {
    process.exitCode = await approveStaged({ ...approving, dryRun: process.argv.includes("--dry-run") }, terminalSteps);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
