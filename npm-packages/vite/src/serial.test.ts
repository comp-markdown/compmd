import { describe, expect, it, vi } from "vitest";
import { serialized } from "./serial.ts";

/** A job that records each run, taking `ms` each, and fails on the runs `failing` names (from 1). */
function job(ms: number, failing: number[] = []) {
  const runs: { started: number; ended?: number }[] = [];
  let inFlight = 0;
  let overlapped = false;
  const run = async () => {
    const each = { started: Date.now() };
    runs.push(each);
    inFlight += 1;
    overlapped ||= inFlight > 1;
    await new Promise((resolve) => setTimeout(resolve, ms));
    inFlight -= 1;
    if (failing.includes(runs.length)) throw new Error(`run ${String(runs.length)} failed`);
  };
  return { run, runs, overlapped: () => overlapped };
}

describe("serialized", () => {
  it("runs a burst of calls once, never two runs at once, and once more after a call made while it ran", async () => {
    const { run, runs, overlapped } = job(30);
    const rebuild = serialized(run, { wait: 5, onError: () => undefined });
    rebuild();
    rebuild();
    rebuild();
    await vi.waitFor(() => {
      expect(runs).toHaveLength(1);
    });
    rebuild();
    rebuild();
    await vi.waitFor(() => {
      expect(runs).toHaveLength(2);
    });
    await new Promise((resolve) => setTimeout(resolve, 80));
    expect(runs).toHaveLength(2);
    expect(overlapped()).toBe(false);
  });

  it("logs a run that fails, as a save with a syntax error does, and runs again at the next call", async () => {
    const { run, runs } = job(1, [1]);
    const errors: string[] = [];
    const rebuild = serialized(run, { wait: 1, onError: (error) => errors.push(error instanceof Error ? error.message : String(error)) });
    rebuild();
    await vi.waitFor(() => {
      expect(errors).toEqual(["run 1 failed"]);
    });
    rebuild();
    await vi.waitFor(() => {
      expect(runs).toHaveLength(2);
    });
    expect(errors).toHaveLength(1);
  });
});
