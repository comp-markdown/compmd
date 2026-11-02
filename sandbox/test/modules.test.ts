// Built modules (docs/running-components-in-the-sandbox.md, T8, decision 7; docs/packs-from-a-registry.md, R2): a
// frame loads a pack's module by its URL, and runs it only if its file matches the hash the host gives; or it gets the
// module's file as a `Blob` the host's page has checked, and loads it as a `blob:` module, with the library packs it
// imports (docs/sharing-library-packs.md).

import type { HostComponent } from "@compmd/processor";
import type { ExpandedRoot } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type SandboxOptions, type View } from "../src/index.ts";
import { frames, leaf, probe, type Probe } from "./harness.ts";

let probes: Probe[] = [];
const views: View[] = [];
afterEach(() => {
  for (const { frame } of probes) frame.destroy();
  probes = [];
  for (const view of views.splice(0)) view.destroy();
});

/** A pack's built module, as `setup.ts` serves it: it sets the input `result` when it mounts. */
const BUILT = 'export default function mount(target, { set }) { target.textContent = "built"; set("result", JSON.stringify("built module")); }\n';

/** The file's SHA-384, as an integrity hash. */
async function integrityOf(url: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-384", await (await fetch(url)).arrayBuffer());
  return `sha384-${btoa(String.fromCharCode(...new Uint8Array(digest)))}`;
}

describe("a built module", () => {
  const url = () => `${frames}packs/built.js`;

  it("loads by its URL and runs when its file matches its hash", async () => {
    const started = probe("strict", "", { code: [{ name: "probe", level: "strict", module: { url: url(), integrity: await integrityOf(url()) } }] });
    probes.push(started);
    expect(await started.result).toBe("built module");
  });

  it("doesn't run when its file doesn't match, and its use shows its fallback", async () => {
    const errors: unknown[] = [];
    let failed: () => void = () => undefined;
    const reported = new Promise<void>((resolve) => (failed = resolve));
    const started = probe("strict", "", {
      code: [{ name: "probe", level: "strict", module: { url: url(), integrity: "sha384-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" } }],
      node: leaf("probe"),
      on: {
        error: (error) => {
          errors.push(error);
          failed();
        },
        set: () => {
          errors.push("ran");
        },
      },
    });
    probes.push(started);
    await reported;
    expect(errors).toEqual([expect.objectContaining({ reason: "failed", component: "probe" })]);
  });
});

describe("a built module the host's page holds", () => {
  const place = "@kit/diagrams@1.0.0/dist/diagram.js";

  it("loads in a strict frame as a `blob:` module", async () => {
    const started = probe("strict", "", { code: [{ name: "probe", level: "strict", binding: "plain", code: new Blob([BUILT]), place }] });
    probes.push(started);
    expect(await started.result).toBe("built module");
  });

  it("is what `modules` gives for a module's place, in frame mode and in host mode", async () => {
    const component: HostComponent = { name: "diagram", language: "js", code: "", trust: { level: "strict" }, module: { pack: "@kit/diagrams", version: "1.0.0", path: "dist/diagram.js" } };
    const tree: ExpandedRoot = {
      type: "root",
      children: [{ type: "componentLeaf", name: "diagram", params: [{ key: "result", breakpoint: null, value: "", input: "result" }], fallback: [{ type: "text", value: "diagram's fallback" }] }],
      inputs: { result: { type: "string", value: "" } },
    };
    for (const page of ["frame", "host"] as const) {
      const asked: string[] = [];
      const sandbox = createSandbox({
        base: frames,
        page,
        modules: (module) => {
          asked.push(`${module.pack}@${module.version}/${module.path}`);
          return { code: new Blob([BUILT]) };
        },
      });
      const area = document.body.appendChild(document.createElement("div"));
      const view = sandbox.open(area, { components: [component] });
      views.push(view);
      const set = new Promise((resolve) => {
        view.on("set", ({ value }) => {
          resolve(JSON.parse(value as string));
        });
      });
      view.render(tree);
      expect(await set, page).toBe("built module");
      expect(asked, page).toEqual([place]);
      expect(area.querySelectorAll("iframe"), page).toHaveLength(1);
      area.remove();
    }
  });
});

describe("library packs (docs/sharing-library-packs.md)", () => {
  /** A component that sets `result` to what its library gives: the library's version and how many times it was asked. */
  const USES = 'import { next, version } from "@kit/count"; export default function mount(target, { set }) { set("result", JSON.stringify(`${version}:${next()}`)); }\n';
  /** A library pack's module, `@kit/count` at `version`, which counts in one instance per frame. */
  const count = (version: string) => `let n = 0; export const next = () => ++n; export const version = "${version}";\n`;
  const counter = (version: string) => ({ pack: "@kit/count", version, path: "dist/count.js" });

  it("loads in a strict frame, its imports and its own library's written as their `blob:` modules", async () => {
    const base = { place: "@kit/base@1.0.0/dist/base.js", code: new Blob(['export const base = "base";\n']) };
    const label = { place: "@kit/label@1.0.0/dist/label.js", code: new Blob(['import { base } from "@kit/base"; export const label = `${base} and label`;\n']), libraries: { "@kit/base": base } };
    const code = 'import { label } from "@kit/label"; export default function mount(target, { set }) { set("result", JSON.stringify(label)); }\n';
    const started = probe("strict", "", { code: [{ name: "probe", level: "strict", binding: "plain", code: new Blob([code]), place: "@kit/charts@1.0.0/dist/chart.js", libraries: { "@kit/label": label } }] });
    probes.push(started);
    expect(await started.result).toBe("base and label");
  });

  it("is one instance for the packs that import a version of it in a frame, and two versions are two", async () => {
    const component = (name: string, version: string): HostComponent => ({
      name,
      language: "js",
      code: "",
      trust: { level: "strict" },
      module: { pack: `@kit/${name}`, version: "1.0.0", path: "dist/index.js" },
      libraries: { "@kit/count": counter(version) },
    });
    const use = (name: string) => ({ type: "componentLeaf" as const, name, params: [{ key: "result", breakpoint: null, value: "", input: name }], fallback: [{ type: "text" as const, value: `${name}'s fallback` }] });
    const tree: ExpandedRoot = {
      type: "root",
      children: [use("bars"), use("lines"), use("pie")],
      inputs: { bars: { type: "string", value: "" }, lines: { type: "string", value: "" }, pie: { type: "string", value: "" } },
    };
    const sandbox = createSandbox({
      base: frames,
      modules: ({ pack, version }) => ({ code: new Blob([pack === "@kit/count" ? count(version) : USES]) }),
    });
    const area = document.body.appendChild(document.createElement("div"));
    const view = sandbox.open(area, { components: [component("bars", "1.0.0"), component("lines", "1.0.0"), component("pie", "2.0.0")] });
    views.push(view);
    const results: Record<string, unknown> = {};
    const all = new Promise<void>((resolve) => {
      view.on("set", ({ input, value }) => {
        results[input] = JSON.parse(value as string);
        if (Object.keys(results).length === 3) resolve();
      });
    });
    view.render(tree);
    await all;
    expect([results.bars, results.lines].sort()).toEqual(["1.0.0:1", "1.0.0:2"]);
    expect(results.pie).toBe("2.0.0:1");
    area.remove();
  });

  it("is imported by compiled source too: a component of a pack written in JavaScript, not built", async () => {
    const source: HostComponent = { name: "probe", language: "js", code: USES, trust: { level: "strict" }, libraries: { "@kit/count": counter("1.0.0") } };
    const sandbox = createSandbox({ base: frames, page: "host", modules: ({ version }) => ({ code: new Blob([count(version)]) }) });
    const area = document.body.appendChild(document.createElement("div"));
    const view = sandbox.open(area, { components: [source] });
    views.push(view);
    const set = new Promise((resolve) => {
      view.on("set", ({ value }) => {
        resolve(JSON.parse(value as string));
      });
    });
    view.render({ type: "root", children: [leaf("probe")], inputs: { result: { type: "string", value: "" } } });
    expect(await set).toBe("1.0.0:1");
    area.remove();
  });

  /** The errors a host-mode view reports for `component`, with `modules` as given. */
  async function errorsOf(component: HostComponent, modules: SandboxOptions["modules"]): Promise<string[]> {
    const area = document.body.appendChild(document.createElement("div"));
    const view = createSandbox({ base: frames, page: "host", ...(modules ? { modules } : {}) }).open(area, { components: [component] });
    views.push(view);
    const errors: string[] = [];
    view.on("error", ({ code, message }) => errors.push(`${String(code)} ${message}`));
    await view.components([component]);
    area.remove();
    return [...new Set(errors)];
  }

  const byUrl = { url: `${frames}packs/built.js`, integrity: "sha384-x" };
  const built = (libraries: NonNullable<HostComponent["libraries"]>): HostComponent => ({ name: "chart", language: "js", code: "", trust: { level: "strict" }, module: { pack: "@kit/chart", version: "1.0.0", path: "dist/index.js" }, libraries });

  it("can't be imported by a built module given by its URL: its file can't be written (W121)", async () => {
    const errors = await errorsOf(built({ "@kit/count": counter("1.0.0") }), ({ pack }) => (pack === "@kit/chart" ? byUrl : { code: new Blob([count("1.0.0")]) }));
    expect(errors).toEqual(["W121 `@kit/chart@1.0.0/dist/index.js` loads by its URL, so its imports of library packs (`@kit/count`) can't be written: give it as code"]);
  });

  it("can't be given by its URL itself, nor can a library it imports (W121)", async () => {
    const math = { pack: "@kit/math", version: "1.0.0", path: "dist/math.js" };
    const nested = built({ "@kit/count": { ...counter("1.0.0"), libraries: { "@kit/math": math } } });
    const errors = await errorsOf(nested, ({ pack }) => (pack === "@kit/math" ? byUrl : { code: new Blob([USES]) }));
    expect(errors).toEqual(["W121 the library pack `@kit/math@1.0.0/dist/math.js` isn't given as code, which a frame needs to load it"]);
  });
});
