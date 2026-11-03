// What the host's page keeps of what it and its frames observe (docs/devtools.md, O2): a bound per source, so a frame
// that floods pushes out only its own.

import { describe, expect, it } from "vitest";
import { Log, reportedBy } from "./observation.ts";
import type { Observed } from "./protocol/observed.ts";

const event = (at: number): Observed => ({ type: "dropped", frame: "", count: 0, at });

describe("Log", () => {
  it("keeps events in the order they came, each source's marked", () => {
    const log = new Log();
    log.add(reportedBy([event(1)]));
    log.add(reportedBy([event(2)], "1"));
    log.add(reportedBy([event(3)]));
    expect(log.events.map(({ at, by }) => [at, by])).toEqual([
      [1, undefined],
      [2, "1"],
      [3, undefined],
    ]);
  });

  it("keeps the last of each source past its limit, and counts what it dropped", () => {
    const log = new Log(3);
    log.add(reportedBy([event(1), event(2)]));
    log.add(reportedBy([4, 5, 6, 7, 8].map(event), "2"));
    expect(log.events.map(({ at }) => at)).toEqual([1, 2, 6, 7, 8]);
    expect(log.dropped).toEqual({ "2": 2 });
  });
});
