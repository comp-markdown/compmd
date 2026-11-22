// Mounts the built `mermaid` component in the page itself, as an app that runs CompMD's components in its own page
// does: no frame, the page's own policy. `window.drawn` says when it has drawn, and what failed.
import mount from "/mermaid.js";

const box = document.getElementById("box");
const style = getComputedStyle(box);
const type = {
  family: style.fontFamily,
  size: parseFloat(style.fontSize),
  weight: style.fontWeight,
  style: style.fontStyle,
  color: style.color,
  lineHeight: style.lineHeight,
  letterSpacing: style.letterSpacing,
};
const failed = [];
const mounted = mount(box, {
  params: { code: "flowchart LR\n  A[Draft a long headline] --> B[Review it twice] --> C[Published]", theme: "default" },
  slots: {},
  scheme: "light",
  fit: { mode: "contain", x: "center", y: "center" },
  type,
  fallback: { text: "Draft, then review, then published" },
  fail: (message) => failed.push(message),
  busy: () => undefined,
  set: () => undefined,
});
await mounted[Symbol.for("compmd.pack.settled")]();
window.drawn = { failed };
