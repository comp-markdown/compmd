// `cargo make approve-packs` (docs/releases.md, decision 8): what npm has staged of the packs, as checkboxes, every
// version waiting for you chosen; Enter approves the chosen ones, in the order they were staged, with one one-time
// code, which npm asks for again once it has expired. With `--dry-run` it lists them and approves nothing.
//
//   node tools/packs/approve-packs.ts [--dry-run]

import { createInterface } from "node:readline/promises";
import checkbox from "@inquirer/checkbox";
import { readNpm, runNpm, type RunNpm } from "./npm.ts";
import { describe, stagedPacks, waiting, type Staged } from "./staged.ts";

/** What `approvePacks` does through: each a stub in its tests. */
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

/**
 * npm's arguments that approve a staged version, with the author's one-time code when there is one. npm uses the
 * code while it is valid, and asks for a new one itself once it has expired.
 */
export const approveArgs = (item: Staged, code?: string): string[] => ["stage", "approve", item.id, ...(code ? ["--otp", code] : [])];

/**
 * Approves the staged versions the author chooses, oldest first.
 *
 * @returns The exit code: 0, also when nothing is staged or nothing chosen.
 * @throws Error when npm fails, after the versions before it are approved.
 */
export async function approvePacks(options: { dryRun: boolean }, steps: Steps): Promise<number> {
  const staged = stagedPacks(steps.list());
  if (staged.length === 0) {
    steps.log("npm has nothing staged for the packs");
    return 0;
  }
  if (options.dryRun) {
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
const realSteps: Steps = {
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

if (import.meta.main) {
  try {
    process.exitCode = await approvePacks({ dryRun: process.argv.includes("--dry-run") }, realSteps);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
