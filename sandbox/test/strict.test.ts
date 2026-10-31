// A strict frame (docs/sandbox.md, threat model T1, T2, T3, T5): its code reaches nothing outside its contract.

import { afterEach, describe, expect, it } from "vitest";
import { ATTEMPT, denied, frames, hits, pause, probe, tagged, type Probe } from "./harness.ts";

let probes: Probe[] = [];
const run = (body: string): Promise<unknown> => {
  const started = probe("strict", body);
  probes.push(started);
  return started.result;
};

afterEach(() => {
  for (const { frame } of probes) frame.destroy();
  probes = [];
});

describe("a strict frame", () => {
  it("runs its component with its contract", async () => {
    expect(await run(`async (target, { params, fallback }) => ({ origin: self.origin, params, fallback: fallback.text })`)).toEqual({
      origin: "null",
      params: { result: "" },
      fallback: "probe's fallback",
    });
  });

  it("sends nothing anywhere: requests, sockets, images, fonts, styles, beacons, prefetches (T1)", async () => {
    const tag = tagged();
    const targets = [denied, frames];
    const result = await run(`async () => {
      ${ATTEMPT}
      const out = {};
      for (const base of ${JSON.stringify(targets)}) {
        const at = (kind) => base + "attempt?${tag}-" + kind;
        out[base] = {
          fetch: await attempt(() => fetch(at("fetch"))),
          xhr: await attempt(() => new Promise((resolve, reject) => { const x = new XMLHttpRequest(); x.open("GET", at("xhr")); x.onload = resolve; x.onerror = reject; x.send(); })),
          websocket: await attempt(() => new Promise((resolve, reject) => { const s = new WebSocket(at("ws").replace("http", "ws")); s.onopen = resolve; s.onerror = reject; })),
          eventSource: await attempt(() => new Promise((resolve, reject) => { const s = new EventSource(at("sse")); s.onopen = resolve; s.onerror = () => { s.close(); reject(); }; })),
          image: await attempt(() => loads(Object.assign(new Image(), { src: at("image") }))),
          font: await attempt(() => new FontFace("probe", "url(" + at("font") + ")").load()),
          style: await attempt(() => loads(Object.assign(document.createElement("link"), { rel: "stylesheet", href: at("style") }))),
        };
        try { navigator.sendBeacon(at("beacon"), "x"); } catch {}
        document.head.append(Object.assign(document.createElement("link"), { rel: "prefetch", href: at("prefetch") }));
      }
      await new Promise((resolve) => setTimeout(resolve, 300));
      return out;
    }`);
    const refused = { fetch: "refused", xhr: "refused", websocket: "refused", eventSource: "refused", image: "refused", font: "refused", style: "refused" };
    expect(result).toEqual({ [denied]: refused, [frames]: refused });
    await pause(200);
    expect(await hits(denied, tag)).toEqual([]);
    // What a browser can't stop (docs/sandbox.md): styles, fonts and prefetches the policy allows from the frames'
    // own origin, the host's, carry a query string there, and nothing else does.
    const atHost = (await hits(frames, tag)).map((hit) => hit.slice(hit.indexOf(`?${tag}-`) + `?${tag}-`.length));
    expect(atHost.filter((kind) => !["font", "style", "prefetch"].includes(kind))).toEqual([]);
  });

  it("can't reach the host's session or page (T2)", async () => {
    expect(
      await run(`async () => {
        const tries = (read) => { try { read(); return "read"; } catch { return "refused"; } };
        return {
          cookie: tries(() => document.cookie),
          localStorage: tries(() => localStorage.length),
          sessionStorage: tries(() => sessionStorage.length),
          indexedDB: await new Promise((resolve) => { try { const r = indexedDB.open("x"); r.onsuccess = () => resolve("read"); r.onerror = () => resolve("refused"); } catch { resolve("refused"); } }),
          parent: tries(() => parent.document.title),
          top: tries(() => top.location.href),
        };
      }`),
    ).toEqual({ cookie: "refused", localStorage: "refused", sessionStorage: "refused", indexedDB: "refused", parent: "refused", top: "refused" });
  });

  it("can't navigate, open windows or submit forms (T3)", async () => {
    const tag = tagged();
    const result = await run(`async () => {
      const tries = (act) => { try { return act(); } catch { return "refused"; } };
      const opened = tries(() => window.open(${JSON.stringify(denied)} + "attempt?${tag}-open") === null ? "refused" : "opened");
      tries(() => { top.location.href = ${JSON.stringify(denied)} + "attempt?${tag}-top"; });
      const form = Object.assign(document.createElement("form"), { action: ${JSON.stringify(denied)} + "attempt?${tag}-form", method: "post" });
      document.body.append(form);
      tries(() => form.submit());
      await new Promise((resolve) => setTimeout(resolve, 300));
      return { opened, here: location.pathname };
    }`);
    expect(result).toEqual({ opened: "refused", here: "/sandbox/strict.html" });
    await pause(200);
    expect(await hits(denied, tag)).toEqual([]);
  });

  it("can't make frames, and a srcdoc frame it makes has no network (T5)", async () => {
    const tag = tagged();
    const result = await run(`async () => {
      const framed = (frame) => new Promise((resolve) => { frame.onload = () => resolve("loaded"); document.body.append(frame); setTimeout(() => resolve("timed out"), 500); });
      for (const src of [${JSON.stringify(frames)} + "permissive.html?${tag}", ${JSON.stringify(frames)} + "strict.html?${tag}", ${JSON.stringify(denied)} + "attempt?${tag}-frame"]) {
        await framed(Object.assign(document.createElement("iframe"), { src }));
      }
      const fetching = (kind) => "<script>fetch(" + JSON.stringify(${JSON.stringify(denied)} + "attempt?${tag}-" + kind) + ").catch(() => {})<\\/script>";
      await framed(Object.assign(document.createElement("iframe"), { src: URL.createObjectURL(new Blob([fetching("blob")], { type: "text/html" })) }));
      await framed(Object.assign(document.createElement("iframe"), { srcdoc: fetching("srcdoc") }));
      await new Promise((resolve) => setTimeout(resolve, 300));
      return "done";
    }`);
    expect(result).toBe("done");
    await pause(200);
    expect(await hits(denied, tag)).toEqual([]);
    expect(await hits(frames, tag)).toEqual([]);
  });
});
