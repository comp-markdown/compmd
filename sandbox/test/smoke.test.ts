import { expect, it } from "vitest";

it("runs in a browser, where a sandboxed frame has an opaque origin", async () => {
  const frame = document.createElement("iframe");
  frame.sandbox.add("allow-scripts");
  frame.srcdoc = "<script>parent.postMessage(self.origin, '*')</script>";
  const origin = new Promise((resolve) => {
    addEventListener("message", ({ data }) => {
      resolve(data);
    }, { once: true });
  });
  document.body.append(frame);
  expect(await origin).toBe("null");
  frame.remove();
});
