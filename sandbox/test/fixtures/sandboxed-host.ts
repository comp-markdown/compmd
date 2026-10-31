// A host's page that is itself in a sandboxed frame, as an editor that runs in a sandbox of its own (docs/sandbox.md,
// decision 2): it opens a document in host mode and tells the test what its components did.

import { createSandbox } from "../../src/index.ts";

const base = new URL("../", location.href).href;
const page = new URLSearchParams(location.search).get("page") === "frame" ? "frame" : "host";
const view = createSandbox({ base, page }).open(document.body, {
  components: [
    {
      name: "chart",
      language: "js",
      code: 'export default function mount(target, { set }) { set("result", JSON.stringify({ origin: self.origin, nested: window.parent !== window.top })); }',
      trust: { level: "strict" },
    },
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
  children: [{ type: "componentLeaf", name: "chart", params: [{ key: "result", breakpoint: null, value: "", input: "result" }] }],
  inputs: { result: { type: "string", value: "" } },
});
