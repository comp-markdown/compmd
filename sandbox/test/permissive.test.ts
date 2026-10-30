// A permissive frame (docs/sandbox.md, T1, T6, T7, decision 1): it reaches its pack's origins and nothing else, holds
// strict frames only, and gets `host` only when its parent gives it.

import type { ExpandedComponent } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { serveHost } from "../src/host-api.ts";
import { ATTEMPT, allowed, denied, frames, hits, leaf, pause, probe, probeCode, tagged, type Probe } from "./harness.ts";

let probes: Probe[] = [];
const started = (value: Probe) => {
  probes.push(value);
  return value.result;
};
/** The allowed probe as a policy names it: its origin, without the path. */
const origin = () => allowed.slice(0, -1);

afterEach(() => {
  for (const { frame } of probes) frame.destroy();
  probes = [];
});

describe("a permissive frame", () => {
  it("reaches its pack's origins, and no other (T1)", async () => {
    const tag = tagged();
    const result = await started(
      probe(
        "permissive",
        `async () => {
          ${ATTEMPT}
          return {
            allowed: await attempt(() => fetch(${JSON.stringify(allowed)} + "attempt?${tag}-fetch")),
            denied: await attempt(() => fetch(${JSON.stringify(denied)} + "attempt?${tag}-fetch")),
            frames: await attempt(() => fetch(${JSON.stringify(frames)} + "attempt?${tag}-fetch")),
          };
        }`,
        { network: [origin()] },
      ),
    );
    expect(result).toEqual({ allowed: "reached", denied: "refused", frames: "refused" });
    await pause(200);
    expect(await hits(allowed, tag)).toEqual([`/attempt?${tag}-fetch`]);
    expect(await hits(denied, tag)).toEqual([]);
    expect(await hits(frames, tag)).toEqual([]);
  });

  it("is strict without origins", async () => {
    const tag = tagged();
    expect(await started(probe("permissive", `async () => { ${ATTEMPT} return attempt(() => fetch(${JSON.stringify(allowed)} + "attempt?${tag}")); }`))).toBe("refused");
    expect(await hits(allowed, tag)).toEqual([]);
  });

  it("can't nest a permissive frame, so it can't widen its own network (T6)", async () => {
    const tag = tagged();
    const result = await started(
      probe(
        "permissive",
        `async () => {
          document.body.append(Object.assign(document.createElement("iframe"), { src: ${JSON.stringify(frames)} + "permissive.html?${tag}" }));
          document.body.append(Object.assign(document.createElement("iframe"), { src: ${JSON.stringify(allowed)} + "attempt?${tag}-frame" }));
          await new Promise((resolve) => setTimeout(resolve, 500));
          return "done";
        }`,
        { network: [origin()] },
      ),
    );
    expect(result).toBe("done");
    expect(await hits(frames, tag)).toEqual([]);
    expect(await hits(allowed, tag)).toEqual([]);
  });

  it("frames a strict component in its slot, which reaches nothing (decision 1)", async () => {
    const tag = tagged();
    const inner: ExpandedComponent = { ...leaf("inner"), params: [{ key: "result", breakpoint: null, value: "", input: "inner" }] };
    const container: ExpandedComponent = { type: "componentContainer", name: "probe", params: [], children: [inner] };
    let resolve: (value: unknown) => void = () => undefined;
    const innerResult = new Promise((done) => (resolve = done));
    void started(
      probe("permissive", `async (target, { slots }) => { for (const item of slots.default) target.append(item.element); }`, {
        network: [origin()],
        node: container,
        inputs: { inner: { type: "string", value: "" } },
        extra: [
          {
            name: "inner",
            level: "strict",
            binding: "plain",
            code: probeCode(`async () => { ${ATTEMPT} return { origin: self.origin, top: window.parent !== window.top, fetch: await attempt(() => fetch(${JSON.stringify(allowed)} + "attempt?${tag}")) }; }`),
          },
        ],
        on: {
          set: ({ input, value }) => {
            if (input === "inner") resolve(JSON.parse(value as string));
          },
        },
      }),
    );
    // Nested in the permissive frame, which is nested in the test's page.
    expect(await innerResult).toEqual({ origin: "null", top: true, fetch: "refused" });
    expect(await hits(allowed, tag)).toEqual([]);
  });

  it("calls `host` when its parent gives it, and has none otherwise", async () => {
    const withHost = await started(
      probe("permissive", `async (target, { host }) => host.call("rating", "p-1")`, {
        host: () => {
          const { port1, port2 } = new MessageChannel();
          serveHost(port1, { rating: (product) => ({ product, stars: 4 }) });
          return port2;
        },
      }),
    );
    expect(withHost).toEqual({ product: "p-1", stars: 4 });
    expect(await started(probe("permissive", `async (target, { host }) => host === undefined`))).toBe(true);
  });

  it("ignores a hello, a level and code from a window that isn't its parent (T7)", async () => {
    const tag = tagged();
    const bare = Object.assign(document.createElement("iframe"), { src: `${frames}permissive.html` });
    bare.setAttribute("sandbox", "allow-scripts");
    document.body.prepend(bare);
    await new Promise((resolve) => {
      bare.addEventListener("load", resolve, { once: true });
    });
    const index = [...document.querySelectorAll("iframe")].indexOf(bare);
    const result = await started(
      probe(
        "strict",
        `async () => {
          const { port1, port2 } = new MessageChannel();
          parent.frames[${String(index)}].postMessage({ type: "compmd-sandbox", version: 1 }, "*", [port2]);
          port1.postMessage({ type: "level", level: "permissive", policy: "connect-src *" });
          port1.postMessage({ type: "code", components: [{ name: "x", level: "permissive", binding: "plain", code: "fetch(" + JSON.stringify(${JSON.stringify(denied)} + "attempt?${tag}") + "); export default () => {}" }] });
          await new Promise((resolve) => setTimeout(resolve, 500));
          return "posed";
        }`,
      ),
    );
    expect(result).toBe("posed");
    expect(await hits(denied, tag)).toEqual([]);
    bare.remove();
  });
});
