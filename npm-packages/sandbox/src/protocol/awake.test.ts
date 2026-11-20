import { afterEach, describe, expect, it, vi } from "vitest";
import { afterAwake, AWAKE_TICK_MS } from "./awake.ts";

afterEach(() => {
  vi.useRealTimers();
});

/** Keeps this thread busy for `ms`, as a long task does. */
function busyFor(ms: number): void {
  const until = Date.now() + ms;
  while (Date.now() < until);
}

const pause = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

describe("afterAwake", () => {
  it("is done once the thread has been awake that long, and not a tick before", () => {
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
    const done = vi.fn();
    afterAwake(1000, done);
    vi.advanceTimersByTime(1000 - AWAKE_TICK_MS);
    expect(done).not.toHaveBeenCalled();
    vi.advanceTimersByTime(AWAKE_TICK_MS);
    expect(done).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(5000);
    expect(done).toHaveBeenCalledTimes(1);
  });

  it("is never done once cancelled", () => {
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval"] });
    const done = vi.fn();
    afterAwake(1000, done)();
    vi.advanceTimersByTime(5000);
    expect(done).not.toHaveBeenCalled();
  });

  it("counts a stretch the thread was busy for as one tick, however long", async () => {
    const done = vi.fn();
    afterAwake(3 * AWAKE_TICK_MS, done);
    // Twice the time it is given, by the clock: a timer of that time would be done on the thread's next turn.
    busyFor(6 * AWAKE_TICK_MS);
    await pause(AWAKE_TICK_MS / 5);
    expect(done).not.toHaveBeenCalled();
    await pause(4 * AWAKE_TICK_MS);
    expect(done).toHaveBeenCalledTimes(1);
  });
});
