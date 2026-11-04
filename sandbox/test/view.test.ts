// A `View`, as a host uses it (docs/running-components-in-the-sandbox.md, Q1, Q7): each method and event, in the
// page modes where it differs, each a test. Its component is `label`, which shows its `label` param and sets the
// input bound to its `result` param to it, when it mounts and on each update.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedComponent, ExpandedRoot, Mount, Splice } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import type { HostClient } from "../src/host-api.ts";
import { createSandbox, type SandboxOptions, type View, type ViewEvents } from "../src/index.ts";
import { frames, pause, probeCode } from "./harness.ts";

const views: View[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
});

/**
 * `label`'s code, with `prefix` before what it shows. It sets its input after an update, not in it: a renderer refuses
 * `set` while it updates a component.
 */
const labelCode = (prefix = "") => `export default function mount(target, { params, set }) {
  const show = (next) => { target.textContent = ${JSON.stringify(prefix)} + next.label; queueMicrotask(() => set("result", JSON.stringify(${JSON.stringify(prefix)} + next.label))); };
  show(params);
  return { update: (next) => show(next) };
}`;

const label = (prefix = ""): HostComponent => ({ name: "label", language: "js", code: labelCode(prefix), trust: { level: "strict" } });

/** A use of `label` showing `text`, its result bound to the input `result`. */
const use = (text: string): ExpandedComponent => ({
  type: "componentLeaf",
  name: "label",
  params: [
    { key: "label", breakpoint: null, value: text },
    { key: "result", breakpoint: null, value: "", input: "result" },
  ],
  fallback: [{ type: "text", value: "label's fallback" }],
});

const RESULT = { result: { type: "string" as const, value: "" } };

/** A document of one use of `label` showing `text`. */
const page = (text: string): ExpandedRoot => ({ type: "root", children: [use(text)], inputs: RESULT });

/** A splice that makes the first block a use showing `text`. */
const spliceTo = (text: string): Splice => ({ from: 0, to: 1, nodes: [use(text)], lines: 0, offset: 0, inputs: RESULT });

/** A view of `components` in `mode`, with what it reports, and a way to wait for an event. */
function open(mode: "frame" | "host", components: HostComponent[] = [label()], options: Partial<SandboxOptions> = {}) {
  const area = document.body.appendChild(document.createElement("div"));
  area.style.height = "300px";
  areas.push(area);
  const view = createSandbox({ base: frames, page: mode, ...options }).open(area, { components });
  views.push(view);
  const events: { [T in keyof ViewEvents]: ViewEvents[T][] } = { set: [], link: [], error: [], recovered: [], rendered: [], loaded: [], pointed: [], layout: [], settled: [] };
  const waiting: (() => void)[] = [];
  for (const type of ["set", "link", "error", "recovered", "rendered", "loaded", "layout", "settled"] as const) {
    view.on(type, (event) => {
      (events[type] as unknown[]).push(event);
      for (const check of waiting) check();
    });
  }
  /** Resolves once `done` holds of the events so far. */
  const until = (done: () => boolean) =>
    new Promise<void>((resolve) => {
      const check = () => {
        if (done()) resolve();
      };
      waiting.push(check);
      check();
    });
  /** The values set so far, parsed. */
  const values = () => events.set.map(({ value }) => JSON.parse(value as string) as unknown);
  return { area, view, events, until, values };
}

const MODES = ["frame", "host"] as const;

describe("render and splice", () => {
  for (const mode of MODES) {
    it(`renders a tree, then updates a use where it is from a splice, as an object or as JSON text (${mode} mode)`, async () => {
      const { view, until, values } = open(mode);
      view.render(page("one"));
      await until(() => values().includes("one"));
      view.splice([spliceTo("two")]);
      await until(() => values().includes("two"));
      view.splice([JSON.stringify(spliceTo("three"))]);
      await until(() => values().includes("three"));
      expect(values()).toEqual(["one", "two", "three"]);
    });
  }

  it("says when the page is painted, and when its components are loaded", async () => {
    const { view, events, until } = open("frame");
    view.render(page("one"));
    await until(() => events.rendered.length > 0 && events.loaded.length > 0);
    expect(events.rendered[0]?.ms).toBeGreaterThanOrEqual(0);
  });
});

describe("the page's colour scheme", () => {
  /** `label`, but setting the scheme it is shown in and the one its frame's document has, on each update. */
  const scheme: HostComponent = {
    name: "label",
    language: "js",
    code: `export default function mount(target, context) {
  const show = () => queueMicrotask(() => context.set("result", JSON.stringify(context.scheme + " " + getComputedStyle(document.documentElement).colorScheme)));
  show();
  return { update: show };
}`,
    trust: { level: "strict" },
  };

  for (const mode of MODES) {
    it(`gives the frames and their components the scheme of the page around them, and the new one when it changes (${mode} mode)`, async () => {
      const { area, view, until, values } = open(mode, [scheme]);
      area.style.colorScheme = "dark";
      view.render(page("one"));
      await until(() => values().includes("dark dark"));
      area.style.colorScheme = "light";
      view.render(page("two"));
      await until(() => values().includes("light light"));
      expect(values().at(-1)).toBe("light light");
    });

    it(`follows a theme the host switches itself, without a render, when it says so (${mode} mode)`, async () => {
      const { area, view, until, values } = open(mode, [scheme]);
      view.render(page("one"));
      await until(() => values().includes("light light"));
      area.classList.add("dark");
      area.style.colorScheme = "dark";
      view.scheme();
      await until(() => values().includes("dark dark"));
      expect(values().at(-1)).toBe("dark dark");
    });
  }
});

describe("a component that fails", () => {
  /** `label`, but failing on the label `bad`, after it returned, as an asynchronous parse does. */
  const failing: HostComponent = {
    name: "label",
    language: "js",
    code: `export default function mount(target, { params, fail, set }) {
  const show = (next) => {
    target.textContent = next.label;
    queueMicrotask(() => {
      if (next.label === "bad") fail("can't read bad");
      else set("result", JSON.stringify(target.textContent));
    });
  };
  show(params);
  return { update: show };
}`,
    trust: { level: "strict" },
  };

  for (const mode of MODES) {
    it(`shows its fallback with W118 and its message, until the next update draws it and says so, by its use (${mode} mode)`, async () => {
      const { view, events, until, values } = open(mode, [failing]);
      view.render(page("bad"));
      await until(() => events.error.length > 0);
      expect(events.error[0]).toMatchObject({ component: "label", reason: "threw", code: "W118", message: "can't read bad" });
      const use = events.error[0]?.use;
      expect(typeof use).toBe("string");
      view.render(page("good"));
      await until(() => values().includes("good") && events.recovered.length > 0);
      expect(events.error).toHaveLength(1);
      expect(events.recovered).toEqual([{ component: "label", use }]);
    });
  }
});

describe("a use's frame size (host mode)", () => {
  /** `label`, but drawing a 120px block 300ms after it mounts, as a component that loads or parses first does. */
  const late: HostComponent = {
    name: "label",
    language: "js",
    code: `export default function mount(target, { set }) {
  setTimeout(() => {
    target.append(Object.assign(document.createElement("div"), { style: "height: 120px" }));
    set("result", JSON.stringify("drawn"));
  }, 300);
}`,
    trust: { level: "strict" },
  };

  it("takes the height of what its component draws later, below the fold, where the browser doesn't render it", async () => {
    const { area, view, until, values } = open("host", [late]);
    area.style.marginTop = "3000px";
    view.render(page("one"));
    await until(() => values().includes("drawn"));
    const frame = area.querySelector("iframe");
    await expect.poll(() => frame?.getBoundingClientRect().height, { timeout: 5000 }).toBe(120);
  });
});

describe("which inputs a page may set (frame mode)", () => {
  it("passes on a set of an input the tree has", async () => {
    const { view, events, until } = open("frame");
    view.render(page("one"));
    await until(() => events.set.length > 0);
    expect(events.set[0]?.input).toBe("result");
  });

  it("drops a set of an input the tree doesn't have: the page's code may have written it", async () => {
    // A use whose param is bound to `admin`, which the tree's inputs don't have.
    const sneaky: ExpandedComponent = { ...use("one"), params: [{ key: "label", breakpoint: null, value: "one" }, { key: "result", breakpoint: null, value: "", input: "admin" }] };
    const { view, events, until } = open("frame");
    view.render({ type: "root", children: [sneaky, use("two")], inputs: RESULT });
    await until(() => events.set.length > 0);
    await pause(300);
    expect(events.set).toEqual([{ input: "result", value: JSON.stringify("two") }]);
  });

  it("follows the inputs a splice gives: one it drops can't be set any more", async () => {
    const { view, events, until, values } = open("frame");
    view.render(page("one"));
    await until(() => values().includes("one"));
    view.splice([spliceTo("two")], undefined, { inputs: {} });
    await pause(400);
    expect(values()).toEqual(["one"]);
    expect(events.set).toHaveLength(1);
  });
});

describe("components", () => {
  for (const mode of MODES) {
    it(`swaps new code in where its component is mounted, with its last params, and resolves once it is in (${mode} mode)`, async () => {
      const { view, until, values } = open(mode);
      view.render(page("one"));
      await until(() => values().includes("one"));
      await view.components([label("new ")]);
      await until(() => values().includes("new one"));
      expect(values()).toEqual(["one", "new one"]);
    });
  }

  it("makes the next render wait for a component it hadn't, so its use mounts with its code", async () => {
    const { view, events, until, values } = open("host", []);
    void view.components([label()]);
    view.render(page("one"));
    await until(() => values().includes("one"));
    expect(events.error).toEqual([]);
  });

  it("shows a built module's use its fallback when the host gives no `modules`, with W121", async () => {
    const built: HostComponent = { name: "label", language: "js", code: "", trust: { level: "strict" }, module: { pack: "@kit/labels", version: "1.0.0", path: "dist/label.js" } };
    const { view, events, until } = open("host", [built]);
    view.render(page("one"));
    await until(() => events.error.length > 0);
    expect(events.error[0]).toEqual({ component: "label", reason: "failed", code: "W121", message: "no URL for `@kit/labels@1.0.0/dist/label.js`" });
  });

  it("shows a trusted component's use its fallback, as missing, when the host gives no mount for it (host mode)", async () => {
    const stars: HostComponent = { name: "label", language: "js", code: "", trust: { level: "trusted" } };
    const { area, view, events, until } = open("host", [stars], { trusted: {} });
    view.render(page("one"));
    await until(() => events.error.length > 0);
    expect(events.error[0]).toMatchObject({ component: "label", reason: "missing" });
    expect(area.textContent).toContain("label's fallback");
  });
});

describe("`host` and its events (host mode)", () => {
  /** `label` as the host's own, trusted: it calls the host, then sets what each of its events brings. */
  const own: Mount = (target, { host, set }) => {
    const client = host as HostClient;
    client.on("changed", (data) => {
      set("result", JSON.stringify(data));
    });
    void client.call("greeting").then((greeting) => {
      set("result", JSON.stringify(greeting));
    });
  };

  it("gives a trusted component in the page `host`, and `emit` sends it the host's events", async () => {
    const trusted: HostComponent = { name: "label", language: "js", code: "", trust: { level: "trusted" } };
    const { view, until, values } = open("host", [trusted], { trusted: { label: own }, host: { greeting: () => "hello" } });
    view.render(page("one"));
    await until(() => values().includes("hello"));
    view.emit("changed", "news");
    await until(() => values().includes("news"));
    expect(values()).toEqual(["hello", "news"]);
  });
});

describe("math", () => {
  /** A paragraph with inline math, display math, and TeX that doesn't typeset. */
  const math: ExpandedRoot = {
    type: "root",
    children: [
      { type: "paragraph", children: [{ type: "text", value: "Square: " }, { type: "inlineMath", value: "x^2" }] },
      { type: "math", value: "\\frac{1}{2}" },
      { type: "paragraph", children: [{ type: "inlineMath", value: "\\frac{1}{" }] },
    ],
  };

  /** What a page shows of its math: how many `<math>` elements, and the source of the math that didn't typeset. */
  const shown = (root: ParentNode) => ({
    typeset: root.querySelectorAll(".compmd-math math").length,
    source: [...root.querySelectorAll(".compmd-math")].filter((math) => !math.querySelector("math")).map((math) => math.textContent),
  });

  /** A strict component in the page's frame, which reports what the frame's page shows once Temml has typeset. */
  const looking: HostComponent = {
    name: "label",
    language: "js",
    code: probeCode(`async () => {
      const shown = ${shown.toString()};
      for (let tries = 0; tries < 100 && shown(document).typeset < 2; tries++) await new Promise((done) => setTimeout(done, 20));
      return shown(document);
    }`),
    trust: { level: "strict" },
  };

  const expected = { typeset: 2, source: ["\\frac{1}{"] };

  it("typesets the page's math as MathML in frame mode", async () => {
    const { view, until, values } = open("frame", [looking]);
    view.render({ ...math, children: [...math.children, use("one")], inputs: RESULT });
    await until(() => values().length > 0);
    expect(values()[0]).toEqual(expected);
  });

  it("typesets the page's math as MathML in host mode, as in frame mode", async () => {
    const { area, view } = open("host");
    view.render(math);
    for (let tries = 0; tries < 100 && shown(area).typeset < 2; tries++) await pause(20);
    expect(shown(area)).toEqual(expected);
  });
});

describe("code blocks", () => {
  /** A JavaScript block, and a Mermaid block whose code renderer isn't here, so its code block shows as its fallback. */
  const blocks: ExpandedRoot["children"] = [
    { type: "code", lang: "js", meta: null, value: "const a = 1;" },
    { type: "code", lang: "mermaid", meta: null, value: "graph LR; A --> B", component: { name: "mermaid", params: [{ key: "code", breakpoint: null, value: "graph LR; A --> B" }] } },
  ];

  /** Each code block's language, how many tokens it shows, and its first token's colour, once all are highlighted. */
  const shown = async (root: ParentNode) => {
    const codes = () => [...root.querySelectorAll("pre code")];
    for (let tries = 0; tries < 200 && (codes().length < 2 || codes().some((code) => !code.querySelector("span"))); tries++) await new Promise((done) => setTimeout(done, 20));
    return codes().map((code) => ({ language: code.className, tokens: code.querySelectorAll("span").length, text: code.textContent, color: getComputedStyle(code.querySelector("span") ?? code).color }));
  };

  /** A strict component in the page's frame, which reports what the frame's page shows of its code blocks. */
  const looking: HostComponent = { name: "label", language: "js", code: probeCode(`async () => (${shown.toString()})(document)`), trust: { level: "strict" } };

  // GitHub's colours for `const`, a keyword: #D73A49 in the light theme, #F97583 in the dark one.
  const keyword = { light: "rgb(215, 58, 73)", dark: "rgb(249, 117, 131)" };

  for (const scheme of ["light", "dark"] as const) {
    it(`highlights the page's code blocks and a code renderer's fallback, in the ${scheme} theme (frame mode)`, async () => {
      const { area, view, until, values } = open("frame", [looking]);
      area.style.colorScheme = scheme;
      view.render({ type: "root", children: [...blocks, use("one")], inputs: RESULT });
      await until(() => values().length > 0);
      const [js, mermaid] = values()[0] as Awaited<ReturnType<typeof shown>>;
      expect(js).toMatchObject({ language: "language-js", text: "const a = 1;", color: keyword[scheme] });
      expect(js?.tokens).toBeGreaterThan(1);
      expect(mermaid).toMatchObject({ language: "language-mermaid", text: "graph LR; A --> B" });
      expect(mermaid?.tokens).toBeGreaterThan(1);
    });

    it(`highlights the page's code blocks and a code renderer's fallback, in the ${scheme} theme (host mode)`, async () => {
      const { area, view } = open("host", []);
      area.style.colorScheme = scheme;
      view.render({ type: "root", children: blocks });
      const [js, mermaid] = await shown(area);
      expect(js).toMatchObject({ language: "language-js", text: "const a = 1;", color: keyword[scheme] });
      expect(mermaid?.tokens).toBeGreaterThan(1);
    });
  }
});

describe("theme and styles", () => {
  /** A component that sets `result` to the page's `--compmd-accent` and the host's stylesheets its frame has. */
  const looking: HostComponent = {
    name: "label",
    language: "js",
    code: probeCode(`async (target) => ({
      accent: getComputedStyle(target).getPropertyValue("--compmd-accent").trim(),
      styles: [...document.querySelectorAll("link[data-compmd-style]")].map((link) => link.href.split("/").pop()),
    })`),
    trust: { level: "strict" },
  };

  for (const mode of MODES) {
    it(`gives the page's theme tokens and the host's stylesheets to the frames (${mode} mode)`, async () => {
      const { view, until, values } = open(mode, [looking], { stylesheets: [`${frames}docs.css`] });
      view.theme({ "--compmd-accent": "rgb(0, 128, 128)" });
      view.render(page("one"));
      await until(() => values().length > 0);
      expect(values()[0]).toEqual({ accent: "rgb(0, 128, 128)", styles: ["docs.css"] });
    });
  }

  it("gives the theme to a use mounted once its code has loaded, in the page's frame (frame mode)", async () => {
    const { view, until, values } = open("frame", [looking]);
    view.theme({ "--compmd-accent": "rgb(0, 128, 128)" });
    view.render(page("one"));
    await until(() => values().length > 0);
    // The code is loaded now, so the second use mounts within the render that adds it.
    view.splice([{ from: 1, to: 1, nodes: [use("two")], lines: 0, offset: 0, inputs: RESULT }]);
    await until(() => values().length > 1);
    expect(values()[1]).toEqual({ accent: "rgb(0, 128, 128)", styles: [] });
  });

  it("replaces the frames' stylesheets with those `styles` gives", async () => {
    const watching: HostComponent = {
      ...looking,
      code: probeCode(`async () => {
        const hrefs = () => [...document.querySelectorAll("link[data-compmd-style]")].map((link) => link.href.split("/").pop());
        while (!hrefs().includes("print.css")) await new Promise((resolve) => setTimeout(resolve, 20));
        return hrefs();
      }`),
    };
    const { view, until, values } = open("host", [watching], { stylesheets: [`${frames}docs.css`] });
    view.render(page("one"));
    await pause(300);
    view.styles([`${frames}print.css`]);
    await until(() => values().length > 0);
    expect(values()[0]).toEqual(["print.css"]);
  });
});

describe("where the page came from", () => {
  /** "Intro" at 0-5 on line 1. */
  const origin = { start: { line: 1, column: 1, offset: 0 }, end: { line: 1, column: 6, offset: 5 }, occurrence: 0 };
  const intro: ExpandedRoot = { type: "root", children: [{ type: "paragraph", origin, children: [{ type: "text", value: "Intro", origin }] }] };
  /** A strict component in the page, which clicks the page's paragraph once it mounts: the frame's reader, as it were. */
  const clicker: HostComponent = { name: "clicker", language: "js", code: "export default function mount() { setTimeout(() => document.querySelector('p').click(), 100); }", trust: { level: "strict" } };

  for (const mode of MODES) {
    it(`gives the boxes of what a source offset drew, in the host's viewport, and none for an offset nothing came from (${mode} mode)`, async () => {
      const { area, view, events, until } = open(mode, []);
      view.render(intro);
      await until(() => events.rendered.length > 0);
      const [box, ...more] = await view.rectsAt(2);
      expect(more).toEqual([]);
      const around = (mode === "host" ? area.querySelector("p") : area.querySelector("iframe"))?.getBoundingClientRect();
      expect(box && around && box.top >= around.top && box.left >= around.left && box.width > 0).toBe(true);
      if (mode === "host") expect(box?.toJSON()).toEqual(around?.toJSON());
      expect(await view.rectsAt(40)).toEqual([]);
    });
  }

  it("reveals what a source offset drew: scrolls to it and marks it a while (host mode)", async () => {
    const { area, view, events, until } = open("host", []);
    view.render(intro);
    await until(() => events.rendered.length > 0);
    view.reveal(2);
    await expect.poll(() => area.querySelector("p")?.classList.contains("compmd-revealed")).toBe(true);
  });

  it("reports the source of what the reader clicks once the host listens, and not before (host mode)", async () => {
    const { area, view, events, until } = open("host", []);
    view.render(intro);
    await until(() => events.rendered.length > 0);
    const pointed: ViewEvents["pointed"][] = [];
    area.querySelector("p")?.click();
    view.on("pointed", (event) => pointed.push(event));
    area.querySelector("p")?.click();
    expect(pointed).toEqual([{ source: { start: origin.start, end: origin.end } }]);
  });

  it("reports the source of what is clicked in the page's frame, once the host listens (frame mode)", async () => {
    const { view } = open("frame", [clicker]);
    const pointed: ViewEvents["pointed"][] = [];
    view.on("pointed", (event) => pointed.push(event));
    view.render({ type: "root", children: [...intro.children, { type: "componentLeaf", name: "clicker", params: [], fallback: [] }] });
    await expect.poll(() => pointed, { timeout: 5000 }).toEqual([{ source: { start: origin.start, end: origin.end } }]);
  });
});

describe("where each line is", () => {
  /** `"# Title\n\nOne\ntwo\n\n```js\nlet a;\nlet b;\n```"`: the heading on line 1, the paragraph on 3 and 4, the code on 7 and 8. */
  const point = (line: number, offset: number) => ({ line, column: 1, offset });
  const range = (from: [number, number], to: [number, number]) => ({ start: point(...from), end: point(...to), occurrence: 0 });
  const lined: ExpandedRoot = {
    type: "root",
    children: [
      { type: "heading", depth: 1, origin: range([1, 0], [1, 7]), children: [{ type: "text", value: "Title", origin: range([1, 2], [1, 7]) }] },
      { type: "paragraph", origin: range([3, 9], [4, 16]), children: [{ type: "text", value: "One\ntwo", origin: range([3, 9], [4, 16]) }] },
      { type: "code", lang: "js", meta: null, value: "let a;\nlet b;", origin: range([6, 18], [9, 42]) },
    ],
  };

  for (const mode of MODES) {
    it(`gives each line that draws something its boxes in the container, in line order, once the page says it moved (${mode} mode)`, async () => {
      const { area, view, events, until } = open(mode, []);
      view.render(lined);
      await until(() => events.layout.length > 0);
      const lines = [];
      for await (const line of view.lines()) lines.push(line);
      expect(lines.map(({ line }) => line)).toEqual([1, 3, 4, 7, 8]);
      const tops = lines.map(({ rects: [first] }) => first?.y ?? -1);
      // Lines 3 and 4 may share a visual line: a soft line break shows as a space.
      expect(tops.every((top, i) => top >= 0 && top <= area.clientHeight && (i === 0 || top >= (tops[i - 1] ?? 0)))).toBe(true);
      expect(tops[3]).toBeGreaterThan(tops[2] ?? 0);
      expect(lines.every(({ rects }) => rects.every(({ width, height }) => width > 0 && height > 0))).toBe(true);
    });
  }
});

describe("when the page has settled", () => {
  /** `label`, but drawing a 120px block `ms` after it mounts, as a diagram drawn asynchronously does, and saying so with `busy`. */
  const late = (ms: number): HostComponent => ({
    name: "label",
    language: "js",
    code: `export default function mount(target, { busy }) {
  busy(new Promise((done) => setTimeout(() => {
    target.append(Object.assign(document.createElement("div"), { className: "late", style: "height: 120px" }));
    done();
  }, ${String(ms)})));
}`,
    trust: { level: "strict" },
  });

  for (const mode of MODES) {
    it(`settles only once a component that draws late has drawn and its frame is sized, with the container marked (${mode} mode)`, async () => {
      const { area, view } = open(mode, [late(400)]);
      const started = performance.now();
      view.render(page("one"));
      await view.settled();
      expect(performance.now() - started).toBeGreaterThanOrEqual(400);
      expect(area.hasAttribute("data-compmd-settled")).toBe(true);
      // In frame mode the use is in the page's frame, another origin's, which the test can't look into.
      if (mode === "host") expect(area.querySelector("iframe")?.getBoundingClientRect().height).toBe(120);
    });
  }

  it("settles again after a render, and not before what it drew has drawn (host mode)", async () => {
    const { area, view, events } = open("host", [late(300)]);
    view.render(page("one"));
    await view.settled();
    const settledBefore = events.settled.length;
    view.render({ type: "root", children: [use("one"), use("two")], inputs: RESULT });
    expect(area.hasAttribute("data-compmd-settled"), "a render unsettles it").toBe(false);
    await view.settled();
    expect(events.settled.length).toBeGreaterThan(settledBefore);
    expect([...area.querySelectorAll("iframe")].map((frame) => frame.getBoundingClientRect().height)).toEqual([120, 120]);
  });
});

describe("the compile frame", () => {
  it("stays hidden whatever the host's page styles iframes with", async () => {
    const style = document.head.appendChild(Object.assign(document.createElement("style"), { textContent: "iframe { display: block; height: 100%; }" }));
    try {
      const { view, until, values } = open("host");
      view.render(page("one"));
      await until(() => values().includes("one"));
      const compiler = document.querySelector<HTMLIFrameElement>('iframe[title="compile"]');
      expect(compiler).not.toBeNull();
      expect(compiler && getComputedStyle(compiler).display).toBe("none");
    } finally {
      style.remove();
    }
  });
});

describe("destroy", () => {
  it("removes the page, its frames and the compile frame, and reports nothing after", async () => {
    const { area, view, events, until, values } = open("host");
    view.render(page("one"));
    await until(() => values().includes("one"));
    view.destroy();
    views.splice(views.indexOf(view), 1);
    expect(area.querySelectorAll("iframe")).toHaveLength(0);
    expect(document.querySelectorAll('iframe[title="compile"]')).toHaveLength(0);
    const reported = events.set.length + events.error.length;
    await pause(300);
    expect(events.set.length + events.error.length).toBe(reported);
  });
});

// WebKit runs these frames in the page's own process, so a frame that spins freezes the test page too.
const webkit = !navigator.userAgent.includes("Chrome");

describe("a page frame that hangs", () => {
  it.skipIf(webkit)("is reported as code that threw (W118), once it hasn't answered within `timeout` (frame mode)", async () => {
    const spinner: HostComponent = { name: "label", language: "js", code: probeCode("async () => { setTimeout(() => { for (const end = Date.now() + 4000; Date.now() < end; ) {} }, 200); return 'started'; }"), trust: { level: "strict" } };
    const { view, events, until } = open("frame", [spinner], { timeout: 1500 });
    view.render(page("one"));
    await until(() => events.error.some(({ message }) => message === "the page's frame stopped answering"));
    expect(events.error.find(({ message }) => message === "the page's frame stopped answering")).toEqual({ component: "", reason: "threw", code: "W118", message: "the page's frame stopped answering" });
  });

  it.skipIf(webkit)("doesn't keep the page from settling: stopped, it counts as settled (frame mode)", async () => {
    const spinner: HostComponent = { name: "label", language: "js", code: "export default function mount() { setTimeout(() => { for (const end = Date.now() + 4000; Date.now() < end; ) {} }, 0); }", trust: { level: "strict" } };
    const { view, events, until } = open("frame", [spinner], { timeout: 1500 });
    view.render(page("one"));
    await until(() => events.error.some(({ message }) => message === "the page's frame stopped answering"));
    await view.settled();
    expect(events.settled.length).toBeGreaterThan(0);
  });
});
