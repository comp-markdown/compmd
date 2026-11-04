// @vitest-environment happy-dom
// What a part renders where (docs/running-components-in-the-sandbox.md, decisions 1 and 4, Q2): a component of the
// part's level runs there, one of a lower level gets a frame of its own, one of a higher level shows its fallback.
import { createRegion, type ExpandedComponent, type ExpandedInput, type Mount } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { codeFor, Mounts, type Framed, type Here } from "./mounts.ts";
import type { ComponentCode } from "./protocol/messages.ts";

const strict: ComponentCode = { name: "chart", level: "strict", binding: "plain", code: "export default () => {}" };
const permissive: ComponentCode = { name: "map", level: "permissive", binding: "plain", code: "export default () => {}" };
const trusted: ComponentCode = { name: "stars", level: "trusted" };

describe("codeFor: what a frame of a level is sent", () => {
  it("a strict frame gets the code of strict components only, and the rest by name and level", () => {
    expect(codeFor([strict, permissive, trusted], "strict")).toEqual([strict, { name: "map", level: "permissive" }, { name: "stars", level: "trusted" }]);
  });

  it("a permissive frame gets its own and its strict children's code, never trusted code", () => {
    expect(codeFor([strict, permissive, trusted], "permissive")).toEqual([strict, permissive, { name: "stars", level: "trusted" }]);
  });
});

/**
 * Mounts rendering `here`, with `inline` running in place. Each frame it makes is a stand-in that records what it is
 * sent, as `<use> <what>`, and keeps the listener for its failure, to call as a frame that stops answering would.
 */
function setup(here: Here, inline: Record<string, Mount> = {}) {
  const framed: string[] = [];
  const sent: string[] = [];
  const reports: [string, string, string][] = [];
  const failures = new Map<string, (message: string) => void>();
  const mounts = new Mounts({
    here,
    inline,
    frame: (_target: Element, node: ExpandedComponent, _inputs: Readonly<Record<string, ExpandedInput>>, component: ComponentCode): Framed => {
      framed.push(`${node.name} in a ${component.level} frame`);
      return {
        code: (code) => sent.push(`${node.name} code ${code.map(({ name }) => name).join()}`),
        theme: (tokens) => sent.push(`${node.name} theme ${Object.keys(tokens).join()}`),
        styles: (urls) => sent.push(`${node.name} styles ${urls.join()}`),
        scheme: (scheme) => sent.push(`${node.name} scheme ${scheme}`),
        failed: (listener) => failures.set(node.name, listener),
        settled: () => Promise.resolve(),
      };
    },
    report: (component, reason, message) => reports.push([component, reason, message]),
  });
  return { mounts, framed, sent, reports, failures };
}

/** Renders a leaf use of `name` with `mounts`, as a page does, in a new element, and gives the element. */
function use(mounts: Mounts, name: string): HTMLElement {
  const target = document.body.appendChild(document.createElement("div"));
  const node: ExpandedComponent = { type: "componentLeaf", name, params: [], fallback: [{ type: "text", value: `${name}'s fallback` }] };
  createRegion(target, { components: mounts.mounts }).render([node], { inputs: {} });
  return target;
}

describe("Mounts: where each use runs", () => {
  it("runs a component of the part's level in place, with the mount it is given", () => {
    const chart: Mount = (target) => {
      target.textContent = "chart, here";
    };
    const { mounts, framed } = setup("strict", { chart });
    mounts.set([strict]);
    expect(use(mounts, "chart").textContent).toBe("chart, here");
    expect(framed).toEqual([]);
  });

  it("frames a component of a lower level where it is used: the host's page frames strict and permissive uses", () => {
    const { mounts, framed } = setup("host");
    mounts.set([strict, permissive, trusted]);
    use(mounts, "chart");
    use(mounts, "map");
    expect(framed).toEqual(["chart in a strict frame", "map in a permissive frame"]);
  });

  it("frames a strict use in a permissive part", () => {
    const { mounts, framed } = setup("permissive");
    mounts.set([strict]);
    use(mounts, "chart");
    expect(framed).toEqual(["chart in a strict frame"]);
  });

  it("shows the fallback of a use above the part's level, and says why (not-here)", () => {
    const { mounts, reports } = setup("strict");
    mounts.set([permissive, trusted]);
    expect(use(mounts, "map").textContent).toBe("map's fallback");
    expect(use(mounts, "stars").textContent).toBe("stars's fallback");
    expect(reports).toEqual([
      ["map", "not-here", "`map` is permissive and can't be in a strict frame: only the host's page holds permissive frames"],
      ["stars", "not-here", "`stars` is trusted and can't be in a strict frame: only the host's page runs trusted code"],
    ]);
  });

  it("shows the fallback of a trusted use in a permissive frame: only the host's page runs trusted code", () => {
    const { mounts, reports } = setup("permissive");
    mounts.set([trusted]);
    use(mounts, "stars");
    expect(reports).toEqual([["stars", "not-here", "`stars` is trusted and can't be in a permissive frame: only the host's page runs trusted code"]]);
  });

  it("has no mount for a component of the part's level it wasn't given: the renderer shows it as missing", () => {
    const { mounts } = setup("host", {});
    mounts.set([trusted]);
    expect(mounts.mounts.stars).toBeUndefined();
  });

  it("is the same object as the components change, since a page reads it at each mount", () => {
    const { mounts } = setup("host");
    const before = mounts.mounts;
    mounts.set([strict]);
    mounts.set([permissive]);
    expect(mounts.mounts).toBe(before);
    expect(Object.keys(mounts.mounts)).toEqual(["map"]);
  });
});

describe("Mounts: its frames", () => {
  it("gives every frame the new code, the page's theme and the host's stylesheets", () => {
    const { mounts, sent } = setup("host");
    mounts.set([strict, permissive]);
    use(mounts, "chart");
    mounts.set([strict, permissive, trusted]);
    mounts.theme({ "--compmd-accent": "teal" });
    mounts.styles(["/docs.css"]);
    expect(sent).toEqual(["chart code chart,map,stars", "chart theme --compmd-accent", "chart styles /docs.css"]);
  });

  it("gives a frame made after the theme was set the theme at once", () => {
    const { mounts, sent } = setup("host");
    mounts.set([strict]);
    mounts.theme({ "--compmd-accent": "teal" });
    use(mounts, "chart");
    expect(sent).toEqual(["chart theme --compmd-accent"]);
  });

  it("gives every frame the page's colour scheme, those made later too", () => {
    const { mounts, sent } = setup("host");
    mounts.set([strict, permissive]);
    use(mounts, "chart");
    mounts.scheme("dark");
    use(mounts, "map");
    expect(sent).toEqual(["chart scheme dark", "map scheme dark"]);
  });

  it("shows a use's fallback when its frame fails (it stopped answering), and says it threw", () => {
    const { mounts, reports, failures } = setup("host");
    mounts.set([strict]);
    const target = use(mounts, "chart");
    failures.get("chart")?.("its frame stopped answering");
    expect(target.textContent).toBe("chart's fallback");
    expect(reports).toEqual([["chart", "threw", "its frame stopped answering"]]);
  });
});
