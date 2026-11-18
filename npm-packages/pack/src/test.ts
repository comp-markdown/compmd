// `compmd-pack test` (docs/writing-a-pack.md, decision 2): the pack's tests, `src/**/*.test.ts`, in happy-dom, as
// `@compmd/pack/testing` runs components there. The pack needs no Vitest configuration of its own.

import { startVitest } from "vitest/node";
import type { Pack } from "./pack.ts";

/**
 * Runs `pack`'s tests once.
 *
 * @returns Whether they all passed.
 * @example
 * process.exitCode = (await testPack(await readPack("."))) ? 0 : 1;
 */
export async function testPack(pack: Pack, options: { filters?: string[] } = {}): Promise<boolean> {
  const vitest = await startVitest(options.filters ?? [], {
    root: pack.dir,
    config: false,
    run: true,
    watch: false,
    environment: "happy-dom",
    include: ["src/**/*.test.{ts,tsx,js,jsx}"],
    passWithNoTests: true,
  });
  await vitest.close();
  return vitest.state.getCountOfFailedTests() === 0 && vitest.state.getUnhandledErrors().length === 0;
}
