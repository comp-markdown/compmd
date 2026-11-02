// @vitest-environment happy-dom
import type { Mount, MountContext } from "@compmd/render";
import { describe, expect, it } from "vitest";
import type { Runnable } from "../protocol/messages.ts";
import { Components, type Loaded } from "./components.ts";

/** A component whose code is the text it shows; `fail` in its code fails to load, `throw` throws when mounted. */
const component = (name: string, code: string): Runnable & { code: string } => ({ name, level: "strict", binding: "plain", code });

function setup() {
  const reports: [string, string, string][] = [];
  const adopted: [string, string | undefined][] = [];
  const log: string[] = [];
  const load = (loaded: Runnable): Promise<Loaded> => {
    const c = loaded as Runnable & { code: string };
    if (c.code === "fail") return Promise.reject(new Error("does not compile"));
    const mount: Mount = (target, context) => {
      if (c.code === "throw") throw new Error("throws");
      log.push(`mount ${c.code} ${String(context.params.n)}`);
      target.textContent = `${c.code} ${String(context.params.n)}`;
      return {
        update: (params) => {
          target.textContent = `${c.code} ${String(params.n)}`;
        },
        destroy: () => log.push(`destroy ${c.code}`),
      };
    };
    return Promise.resolve({ mount, ...(c.code.startsWith("styled") ? { css: "b {}" } : {}) });
  };
  const components = new Components({ load, report: (name, reason, message) => reports.push([name, reason, message]), adopt: (name, css) => adopted.push([name, css]) });
  return { components, reports, adopted, log };
}

function context(n: number): MountContext {
  const fallback = document.createElement("span");
  fallback.textContent = "fallback";
  return { params: { n }, slots: {}, set: () => undefined, fallback: { text: "fallback", element: fallback }, headingLevel: 1, scheme: "light", fail: () => undefined };
}

describe("Components", () => {
  it("mounts a component once its code is loaded, and again with the same params when its code changes", async () => {
    const { components, log } = setup();
    const target = document.createElement("div");
    const loading = components.set([component("card", "one")]);
    const handle = components.mounts.card?.(target, context(1));
    expect(target.textContent).toBe("");
    await loading;
    expect(target.textContent).toBe("one 1");
    handle?.update?.({ n: 2 }, {});
    expect(target.textContent).toBe("one 2");
    await components.set([component("card", "two")]);
    expect(target.textContent).toBe("two 2");
    expect(log).toEqual(["mount one 1", "destroy one", "mount two 2"]);
    await components.set([component("card", "two")]);
    expect(log).toHaveLength(3);
    handle?.destroy?.();
    expect(log.at(-1)).toBe("destroy two");
  });

  it("gives a component mounted again the renderer's scheme, as the renderer changes it", async () => {
    const schemes: string[] = [];
    const load = (): Promise<Loaded> =>
      Promise.resolve({
        mount: (_, mounted) => {
          schemes.push(`mount ${mounted.scheme}`);
          return { update: () => schemes.push(`update ${mounted.scheme}`) };
        },
      });
    const components = new Components({ load, report: () => undefined, adopt: () => undefined });
    await components.set([component("card", "one")]);
    const live = context(1);
    const handle = components.mounts.card?.(document.createElement("div"), live);
    await components.set([component("card", "two")]);
    live.scheme = "dark";
    handle?.update?.({ n: 1 }, {});
    expect(schemes).toEqual(["mount light", "mount light", "update dark"]);
  });

  it("shows the fallback and reports code that doesn't load, and mounts it once it does", async () => {
    const { components, reports } = setup();
    await components.set([component("card", "fail")]);
    const target = document.createElement("div");
    components.mounts.card?.(target, context(1));
    expect(target.textContent, "mounted after its code failed: no second report").toBe("fallback");
    const live = document.createElement("div");
    await components.set([component("card", "one")]);
    expect(target.textContent).toBe("one 1");
    components.mounts.card?.(live, context(1));
    await components.set([component("card", "fail")]);
    expect(live.textContent).toBe("fallback");
    await components.set([component("card", "one")]);
    expect(live.textContent).toBe("one 1");
    expect(reports).toEqual([
      ["card", "failed", "does not compile"],
      ["card", "failed", "does not compile"],
    ]);
  });

  it("shows the fallback when new code throws as it mounts", async () => {
    const { components, reports } = setup();
    await components.set([component("card", "one")]);
    const target = document.createElement("div");
    components.mounts.card?.(target, context(1));
    await components.set([component("card", "throw")]);
    expect(target.textContent).toBe("fallback");
    expect(reports).toEqual([["card", "threw", "throws"]]);
  });

  it("drops names no longer given, with their styles, and adopts each component's CSS", async () => {
    const { components, adopted } = setup();
    await components.set([component("a", "styled"), component("b", "plain")]);
    expect(Object.keys(components.mounts)).toEqual(["a", "b"]);
    await components.set([component("b", "plain")]);
    expect(Object.keys(components.mounts)).toEqual(["b"]);
    expect(adopted).toEqual([
      ["a", "b {}"],
      ["b", undefined],
      ["a", undefined],
    ]);
  });

  it("loads a built module again only when its URL or hash changes", async () => {
    const loads: string[] = [];
    const components = new Components({
      load: (c) => {
        loads.push("module" in c ? c.module.integrity : "");
        return Promise.resolve({ mount: () => undefined });
      },
      report: () => undefined,
      adopt: () => undefined,
    });
    const mermaid = (integrity: string): Runnable => ({ name: "mermaid", level: "strict", module: { url: "/packs/mermaid.js", integrity } });
    await components.set([mermaid("sha384-a")]);
    await components.set([mermaid("sha384-a")]);
    await components.set([mermaid("sha384-b")]);
    expect(loads).toEqual(["sha384-a", "sha384-b"]);
  });

  it("loads a built module given as a Blob again only when its place changes", async () => {
    const loads: string[] = [];
    const components = new Components({
      load: (c) => {
        loads.push("place" in c ? c.place : "");
        return Promise.resolve({ mount: () => undefined });
      },
      report: () => undefined,
      adopt: () => undefined,
    });
    // Each message brings a copy of the Blob, so the same module comes as another Blob each time.
    const mermaid = (place: string): Runnable => ({ name: "mermaid", level: "strict", binding: "plain", code: new Blob(["export default () => undefined"]), place });
    await components.set([mermaid("@kit/mermaid@1.0.0/dist/mermaid.js")]);
    await components.set([mermaid("@kit/mermaid@1.0.0/dist/mermaid.js")]);
    await components.set([mermaid("@kit/mermaid@1.1.0/dist/mermaid.js")]);
    expect(loads).toEqual(["@kit/mermaid@1.0.0/dist/mermaid.js", "@kit/mermaid@1.1.0/dist/mermaid.js"]);
  });

  it("loads code again when a library it imports, or that library's own, is another version", async () => {
    let loads = 0;
    const components = new Components({
      load: () => {
        loads += 1;
        return Promise.resolve({ mount: () => undefined });
      },
      report: () => undefined,
      adopt: () => undefined,
    });
    const library = (place: string, math = "@kit/math@1.0.0/dist/math.js") => ({ place, code: new Blob(["export {}"]), libraries: { "@kit/math": { place: math, code: new Blob(["export {}"]) } } });
    const chart = (three: ReturnType<typeof library>): Runnable => ({ name: "chart", level: "strict", binding: "plain", code: "export default () => undefined", libraries: { "@kit/three": three } });
    await components.set([chart(library("@kit/three@0.170.0/dist/three.js"))]);
    await components.set([chart(library("@kit/three@0.170.0/dist/three.js"))]);
    expect(loads).toBe(1);
    await components.set([chart(library("@kit/three@0.171.0/dist/three.js"))]);
    await components.set([chart(library("@kit/three@0.171.0/dist/three.js", "@kit/math@1.1.0/dist/math.js"))]);
    expect(loads).toBe(3);
  });

  it("keeps the newest code when loads finish out of order", async () => {
    const { components } = setup();
    const target = document.createElement("div");
    const first = components.set([component("card", "one")]);
    components.mounts.card?.(target, context(1));
    const second = components.set([component("card", "two")]);
    await Promise.all([second, first]);
    expect(target.textContent).toBe("two 1");
  });
});
