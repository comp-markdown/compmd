import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

const CLI = join(import.meta.dirname, "cli.ts");
let dir: string;

/** Runs the command with `args` in the test's folder: what it printed, and how it exited. */
function run(...args: string[]): { out: string; code: number } {
  try {
    return { out: execFileSync("node", [CLI, ...args], { cwd: dir, encoding: "utf8", stdio: "pipe" }), code: 0 };
  } catch (error) {
    const failed = error as { stdout: string; stderr: string; status: number };
    return { out: failed.stdout + failed.stderr, code: failed.status };
  }
}

beforeEach(async () => {
  dir = await mkdtemp(join(tmpdir(), "compmd-create-pack-cli-"));
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

describe("create-pack", () => {
  it("writes a pack, and says what to run next", () => {
    const { out, code } = run("Mermaid", "--name", "@kit/mermaid", "--renders", "mermaid");
    expect(code).toBe(0);
    expect(out).toBe("Wrote @kit/mermaid in Mermaid: 13 files, with the component mermaid, a renderer of ```mermaid blocks.\n\n  cd Mermaid\n  npm install\n  npm run dev\n");
    expect(existsSync(join(dir, "Mermaid/src/mermaid.ts"))).toBe(true);
  });

  it("takes the component's name", () => {
    expect(run("charts", "--name", "@kit/charts", "--component", "bar-chart").out).toContain("with the component bar-chart.");
    expect(existsSync(join(dir, "charts/src/bar-chart.ts"))).toBe(true);
  });

  it("writes a library pack with --library, and says to build it", () => {
    expect(run("three", "--name", "@kit/three", "--library", "three")).toEqual({ out: "Wrote @kit/three in three: 7 files, a library pack of three.\n\n  cd three\n  npm install\n  npm run build\n", code: 0 });
    expect(existsSync(join(dir, "three/src/index.ts"))).toBe(true);
    expect(run("d3", "--name", "@kit/d3", "--library", "d3", "--component", "chart")).toEqual({ out: "a library pack has no components: leave out --component\n", code: 1 });
  });

  it("shows its usage, and stops at what can't make a pack", () => {
    expect(run("--help")).toEqual({ out: expect.stringContaining("npm create @compmd/pack <folder> [-- options]") as string, code: 0 });
    expect(run().code).toBe(1);
    expect(run("x", "--name", "kit")).toEqual({ out: "kit isn't a scoped npm name, such as @kit/mermaid: packs are always scoped\n", code: 1 });
  });
});
