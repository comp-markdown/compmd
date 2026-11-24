// Two permissive packs in one document (docs/running-components-in-the-sandbox.md, decision 10, T12): each pack's
// frame runs its own pack's code only, so the network and `host` one pack is given reach no other code (spec §15).

import type { HostComponent } from "@compmd/processor";
import type { ExpandedComponent } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type View } from "../src/index.ts";
import { allowed, frames, tagged } from "./harness.ts";

const views: View[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
});

/**
 * A permissive pack's component whose module records, as it loads, that it ran in this frame; its use then sets its
 * param `seen`, bound to an input, to everything that ran there.
 */
function recording(name: string, pack: string): HostComponent {
  const code = `globalThis.__ran = [...(globalThis.__ran ?? []), ${JSON.stringify(name)}];
export default function mount(target, context) {
  setTimeout(() => context.set("seen", JSON.stringify(globalThis.__ran)), 50);
}`;
  return { name, language: "js", code, trust: { level: "permissive", network: [], host: false }, pack };
}

/** A use of `name` that binds `input`. */
const use = (name: string, input: string, children: ExpandedComponent[] = []): ExpandedComponent =>
  children.length === 0
    ? { type: "componentLeaf", name, params: [{ key: "seen", breakpoint: null, value: "", input }], fallback: [{ type: "text", value: `${name}'s fallback` }] }
    : { type: "componentContainer", name, params: [{ key: "seen", breakpoint: null, value: "", input }], children };

describe("two permissive packs", () => {
  it("run each in their own frames, and one's use in the other's slot shows its fallback (S2)", async () => {
    const area = document.body.appendChild(document.createElement("div"));
    areas.push(area);
    const components = [recording("alpha", "@kit/alpha"), recording("beta", "@kit/beta")];
    const view = createSandbox({ base: frames, page: "host" }).open(area, { components });
    views.push(view);
    const seen: Record<string, unknown> = {};
    const errors: [string, string][] = [];
    view.on("set", ({ input, value }) => {
      seen[input] = JSON.parse(value as string) as unknown;
    });
    view.on("error", ({ component, reason }) => errors.push([component, reason]));
    view.render({
      type: "root",
      children: [use("alpha", "alphaSaw", [use("beta", "inAlpha")]), use("beta", "betaSaw")],
      inputs: { alphaSaw: { type: "string", value: "" }, betaSaw: { type: "string", value: "" }, inAlpha: { type: "string", value: "" } },
    });
    await expect.poll(() => ({ seen, errors }), { timeout: 10_000 }).toEqual({ seen: { alphaSaw: ["alpha"], betaSaw: ["beta"] }, errors: [["beta", "not-here"]] });
  });
});

describe("a document's component and an example's of the same name", () => {
  it("keep each its own trust (B7)", async () => {
    const area = document.body.appendChild(document.createElement("div"));
    areas.push(area);
    // Each tries the allowed origin, which only a permissive frame given it reaches.
    const code = `export default function mount(target, context) {
  fetch(${JSON.stringify(`${allowed}attempt?${tagged()}`)}, { mode: "no-cors" }).then(() => "reached", () => "refused").then((r) => context.set("seen", JSON.stringify(r)));
}`;
    const example: HostComponent = { name: "chart", language: "js", code, trust: { level: "permissive", network: [new URL(allowed).origin], host: false }, pack: "@kit/charts", example: 1 };
    // The document's last, as a map by name kept only the last.
    const own: HostComponent = { name: "chart", language: "js", code, trust: { level: "strict" } };
    const view = createSandbox({ base: frames, page: "host" }).open(area, { components: [example, own] });
    views.push(view);
    const seen: Record<string, unknown> = {};
    view.on("set", ({ value }) => {
      seen.document = JSON.parse(value as string) as unknown;
    });
    view.on("exampleSet", ({ value }) => {
      seen.example = JSON.parse(value as string) as unknown;
    });
    const inputs = { saw: { type: "string" as const, value: "" } };
    view.render({
      type: "root",
      children: [use("chart", "saw"), { type: "code", lang: "compmd", meta: null, value: ":: chart[x]", render: "output", document: { type: "root", children: [use("chart", "saw")], inputs, example: 1 } }],
      inputs,
    });
    await expect.poll(() => seen, { timeout: 10_000 }).toEqual({ document: "refused", example: "reached" });
  });
});
