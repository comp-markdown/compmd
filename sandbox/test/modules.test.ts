// Built modules (docs/running-components-in-the-sandbox.md, T8, decision 7; docs/packs-from-a-registry.md, R2): a
// frame loads a pack's module by its URL, and runs it only if its file matches the hash the host gives; or it gets the
// module's file as a `Blob` the host's page has checked, and loads it as a `blob:` module.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedRoot } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type View } from "../src/index.ts";
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
