// A host's page that is itself in a sandboxed frame, as an editor that runs in a sandbox of its own
// (docs/running-components-in-the-sandbox.md, decision 2): it opens a document in either page mode and tells the test
// what its components did, each through the input its param is bound to. One is compiled source; the other a built
// module the page holds, as a resolver of registry packs gives it (docs/packs-from-a-registry.md, R2).

import { createSandbox } from "../../src/index.ts";

/** A component's code that sets its param `result` to where it runs. */
const reporting = 'export default function mount(target, { set }) { set("result", JSON.stringify({ origin: self.origin, nested: window.parent !== window.top })); }';

// Relative to the page, as a host that serves the frame documents next to it gives it.
const base = "../";
const page = new URLSearchParams(location.search).get("page") === "frame" ? "frame" : "host";
const modules = () => ({ code: new Blob([reporting]) });
const view = createSandbox({ base, page, modules }).open(document.body, {
  components: [
    { name: "chart", language: "js", code: reporting, trust: { level: "strict" } },
    { name: "diagram", language: "js", code: "", trust: { level: "strict" }, module: { pack: "@kit/diagrams", version: "1.0.0", path: "dist/diagram.js" } },
  ],
});
view.on("set", ({ input, value }) => {
  parent.postMessage({ from: "sandboxed-host", input, value }, "*");
});
view.on("error", (error) => {
  parent.postMessage({ from: "sandboxed-host", error }, "*");
});
view.render({
  type: "root",
  children: [
    { type: "componentLeaf", name: "chart", params: [{ key: "result", breakpoint: null, value: "", input: "result" }] },
    { type: "componentLeaf", name: "diagram", params: [{ key: "result", breakpoint: null, value: "", input: "diagram" }] },
  ],
  inputs: { result: { type: "string", value: "" }, diagram: { type: "string", value: "" } },
});
