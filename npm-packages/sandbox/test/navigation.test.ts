// A frame that navigates itself, or whose element moves (docs/running-components-in-the-sandbox.md, decision 10, T11):
// only a load its parent started is greeted, so what loaded otherwise gets no port, code or inputs.

import type { HostComponent } from "@compmd/processor";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox } from "../src/index.ts";
import type { UseFrame } from "../src/use.ts";
import { frames, hits, leaf, pause, probe, probeCode, tagged } from "./harness.ts";

let made: UseFrame[] = [];
afterEach(() => {
  for (const frame of made) frame.destroy();
  made = [];
});

describe("the page frame's loads", () => {
  it("fails the page frame when its code navigates it, with an error", async () => {
    const area = document.body.appendChild(document.createElement("div"));
    const leaving: HostComponent = { name: "leaving", language: "js", code: probeCode(`async () => { location.href = ${JSON.stringify(`${frames}strict.html?${tagged()}`)}; }`), trust: { level: "strict" } };
    const view = createSandbox({ base: frames, page: "frame" }).open(area, { components: [leaving] });
    const errors: string[] = [];
    view.on("error", ({ message }) => errors.push(message));
    view.render({ type: "root", children: [leaf("leaving")], inputs: { result: { type: "string", value: "" } } });
    await expect.poll(() => errors, { timeout: 10_000 }).toEqual([expect.stringContaining("loaded documents it wasn't given")]);
    view.destroy();
    area.remove();
  });
});

describe("a use frame's loads", () => {
  it("fails a frame whose code keeps navigating it, and greets none of what it loaded", async () => {
    const tag = tagged();
    const failures: string[] = [];
    // Each load of the frame's own document runs the probe again, which navigates away again, to a URL the frames'
    // server records. Greeting what loaded, as each load once was, would run it for good: the frame must fail instead.
    const away = `${frames}strict.html?${tag}`;
    const { frame } = probe("strict", `async () => { location.href = ${JSON.stringify(away)}; return new Promise(() => undefined); }`, {
      on: { failed: ({ reason }) => failures.push(reason) },
    });
    made.push(frame);
    await expect.poll(() => failures, { timeout: 15_000 }).toEqual(["navigated"]);
    // No more loads of the page it went to once the frame failed.
    await pause(500);
    const loads = (await hits(frames, tag)).length;
    await pause(1000);
    expect((await hits(frames, tag)).length).toBe(loads);
  });

  it("boots a frame again whose element moved, and it draws", async () => {
    const box = document.body.appendChild(document.createElement("div"));
    const other = document.body.appendChild(document.createElement("div"));
    let results = 0;
    const { frame } = probe("strict", `async () => "drawn"`, {
      target: box,
      on: {
        set: ({ input }) => {
          if (input === "result") results += 1;
        },
      },
    });
    made.push(frame);
    await expect.poll(() => results, { timeout: 10_000 }).toBe(1);
    // Moving the frame's element loads its document again, a load its parent didn't start.
    other.append(frame.iframe.parentElement ?? frame.iframe);
    await expect.poll(() => results, { timeout: 10_000 }).toBe(2);
    box.remove();
    other.remove();
  });
});
