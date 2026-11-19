// The test page: hundreds of short TypeScript runs, each a page of its own (`createPage`), mounted when it comes
// within a screen of view, as a diff viewer's list is. The variant is the URL's query (`highlighters.ts`). The page
// records every animation frame, so the runner can read how long frames blocked in each phase: opening, a skim to the
// end, and a revisit, every run mounted again, as a reader jumping back.

import { createPage, type CodeHighlighter, type Page } from "@compmd/render";
import { shikiHighlight } from "@compmd/sandbox";
import type { FrameReport, TokensAnswer, TokensRequest } from "./frame.ts";
import { highlighters, stats, type Config } from "./highlighters.ts";
import { runsOf } from "./runs.ts";

const query = new URLSearchParams(location.search);
const config: Config = {
  shiki: query.get("shiki") === "shared" ? "shared" : "per-view",
  engine: (query.get("engine") ?? "js") as Config["engine"],
  where: query.get("where") === "worker" ? "worker" : "main",
  schedule: query.get("schedule") === "sliced" ? "sliced" : "eager",
  cache: query.get("cache") === "1",
};
const COUNT = Number(query.get("runs") ?? 440);

/** Every animation frame's time, from the page's start. */
const frames: number[] = [];
const tick = (at: number) => {
  frames.push(at);
  requestAnimationFrame(tick);
};
requestAnimationFrame(tick);

const scroller = document.querySelector<HTMLElement>("#scroller") ?? document.body;
const runs = runsOf(COUNT);

interface Run {
  slot: HTMLElement;
  page?: Page;
  mounted?: number;
  highlighted?: number;
}
const slots: Run[] = runs.map((code) => {
  const slot = scroller.appendChild(document.createElement("div"));
  slot.className = "run";
  // About the run's height, so the list is as long before its runs mount.
  slot.style.minHeight = `${String(code.split("\n").length * 18 + 8)}px`;
  return { slot };
});
const bySlot = new Map(slots.map((run) => [run.slot, run]));

const highlighted = (target: Element) => {
  const run = bySlot.get(target.closest<HTMLElement>(".run") ?? (target as HTMLElement));
  if (run && run.highlighted === undefined) run.highlighted = performance.now();
};
const made = highlighters(config, highlighted);

/**
 * `?sandbox=per-view` or `?sandbox=shared`: the sandbox's own highlighter, as the page's `highlight` (CMD-256), one
 * for each page, as a view made before, or one for all, as `framesHighlight` gives them now. A run is highlighted
 * when its tokens are drawn, which the renderer does once the highlighter's promise has settled.
 */
const sandbox = query.get("sandbox");
/** `&wasm=1`: with Oniguruma where the page compiles WebAssembly, as a host-mode page since CMD-257. */
const shikiOptions = { webAssembly: query.get("wasm") === "1" };
const sandboxHighlight = (() => {
  if (sandbox !== "per-view" && sandbox !== "shared") return undefined;
  const one = shikiHighlight(shikiOptions);
  return (slot: HTMLElement): CodeHighlighter => {
    const highlight = sandbox === "shared" ? one : shikiHighlight(shikiOptions);
    return async (code, language) => {
      const lines = await highlight(code, language);
      setTimeout(() => {
        highlighted(slot);
      }, 0);
      return lines;
    };
  };
})();

/** A run's page, rendered: one code block, with its origin, as a document of one fenced block. */
function mount(index: number): void {
  const run = slots[index];
  const code = runs[index];
  if (!run || code === undefined || run.page) return;
  run.mounted = performance.now();
  delete run.highlighted;
  run.page = createPage(run.slot, sandboxHighlight ? { highlight: sandboxHighlight(run.slot) } : {});
  const lines = code.split("\n").length;
  run.page.render({ type: "root", children: [{ type: "code", lang: "ts", meta: null, value: code, origin: { start: { line: 1, column: 1 }, end: { line: lines + 2, column: 4 }, occurrence: 0 } }] });
  if (sandboxHighlight) return;
  // The variant's highlighter draws the block itself, as the page's `highlight` did when the spike was run.
  const target = run.slot.querySelector<HTMLElement>("pre > code");
  if (query.get("highlight") === "off" || !target) run.highlighted = performance.now();
  else made()(code, "ts", target);
}

/**
 * A run's page gone, once it leaves the window around the screen, as a virtual list does: its slot keeps its height.
 * Without `?keep=1`: kept mounted, hundreds of pages are each a stylesheet in the one document (see the README).
 */
function unmount(index: number): void {
  const run = slots[index];
  if (!run?.page || query.get("keep") === "1") return;
  run.slot.style.minHeight = `${String(run.slot.offsetHeight)}px`;
  run.page.destroy();
  delete run.page;
  run.slot.replaceChildren();
}

const observer = new IntersectionObserver(
  (entries) => {
    for (const entry of entries) {
      const index = slots.findIndex((run) => run.slot === entry.target);
      if (entry.isIntersecting) mount(index);
      else unmount(index);
    }
  },
  { root: scroller, rootMargin: "100% 0px" },
);
for (const run of slots) observer.observe(run.slot);

const pause = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/** Resolves once every mounted run is highlighted, or after `ms`. */
async function idle(ms = 60_000): Promise<void> {
  const until = performance.now() + ms;
  while (performance.now() < until && slots.some((run) => run.page && run.highlighted === undefined)) await pause(20);
}

/** The frames of `[from, to]`: the time in frames over 50 ms past 50 ms, the longest, and how many were long. */
function blocked(from: number, to: number) {
  let time = 0;
  let longest = 0;
  let long = 0;
  for (let i = 1; i < frames.length; i++) {
    const [before, at] = [frames[i - 1] ?? 0, frames[i] ?? 0];
    if (at < from || before > to) continue;
    const gap = at - before;
    longest = Math.max(longest, gap);
    if (gap > 50) {
      time += gap - 50;
      long += 1;
    }
  }
  return { blockedMs: Math.round(time), longestMs: Math.round(longest), longFrames: long };
}

const percentile = (values: number[], p: number) => {
  const sorted = [...values].sort((a, b) => a - b);
  return Math.round(sorted[Math.min(sorted.length - 1, Math.floor((sorted.length - 1) * p))] ?? 0);
};

/** From mounting to highlighted, of the runs mounted in `[from, to]`. */
function waits(from: number, to: number) {
  const all = slots.filter((run) => run.mounted !== undefined && run.mounted >= from && run.mounted <= to && run.highlighted !== undefined).map((run) => (run.highlighted ?? 0) - (run.mounted ?? 0));
  return { runs: all.length, medianWaitMs: percentile(all, 0.5), p95WaitMs: percentile(all, 0.95) };
}

/** Scrolls to the end as a reader skims: a screen and a half every 100 ms. */
async function skim(): Promise<void> {
  while (scroller.scrollTop + scroller.clientHeight < scroller.scrollHeight - 1) {
    scroller.scrollTop += scroller.clientHeight * 1.5;
    await pause(100);
  }
}

const sum = (values: number[]) => Math.round(values.reduce((total, value) => total + value, 0));

/** The steps' costs so far, and since `mark`. */
const costs = () => ({
  setups: stats.setups.map(Math.round),
  firstTokenizeMs: Math.round(stats.tokenize[0] ?? 0),
  tokenizeMs: sum(stats.tokenize),
  tokenized: stats.tokenize.length,
  medianTokenizeMs: Number((stats.tokenize.length ? [...stats.tokenize].sort((a, b) => a - b)[Math.floor(stats.tokenize.length / 2)] ?? 0 : 0).toFixed(2)),
  drawMs: sum(stats.draw),
  cacheHits: stats.cacheHits,
});

/** What makes the browser style the whole document again, counted from here on. */
function counted() {
  const counts = { adoptions: 0, styleReads: 0, sheetWrites: 0 };
  const sheets = Object.getOwnPropertyDescriptor(Document.prototype, "adoptedStyleSheets");
  Object.defineProperty(document, "adoptedStyleSheets", {
    configurable: true,
    get: () => sheets?.get?.call(document) as CSSStyleSheet[],
    set: (next: CSSStyleSheet[]) => {
      counts.adoptions += 1;
      sheets?.set?.call(document, next);
    },
  });
  const read = window.getComputedStyle.bind(window);
  window.getComputedStyle = (...given: Parameters<typeof read>) => {
    counts.styleReads += 1;
    return read(...given);
  };
  const write = CSSStyleSheet.prototype.replaceSync;
  CSSStyleSheet.prototype.replaceSync = function (this: CSSStyleSheet, text: string) {
    counts.sheetWrites += 1;
    write.call(this, text);
  };
  return counts;
}

const nextFrame = () => new Promise<number>((resolve) => requestAnimationFrame(resolve));

/**
 * A large host document, then pages opened and closed in it, one a frame, eight live at a time, as a list that mounts
 * what scrolls into view: what each open costs the page, apart from highlighting. `?host=N` is the host's elements,
 * `?styled=1` gives each page a box with rules of its own.
 */
async function churn(): Promise<unknown> {
  observer.disconnect();
  for (const run of slots) unmountNow(run);
  const host = scroller.appendChild(document.createElement("div"));
  host.className = "host";
  const elements = Number(query.get("host") ?? 20_000);
  for (let i = 0; i < elements / 2; i++) {
    const row = host.appendChild(document.createElement("div"));
    row.className = `row r${String(i % 7)}`;
    row.appendChild(document.createElement("span")).textContent = `Row ${String(i)}`;
  }
  await nextFrame();
  await pause(500);
  const counts = counted();
  // `?nowrite=1`: the renderer's sheet writes counted but not made, to see what they cost apart from the rest.
  if (query.get("nowrite") === "1") CSSStyleSheet.prototype.replaceSync = () => (counts.sheetWrites += 1, undefined);
  const styled = query.get("styled") === "1";
  const live: Run[] = [];
  const opens: number[] = [];
  const start = performance.now();
  for (let i = 0; i < 100; i++) {
    const run = slots[i];
    const code = runs[i];
    if (!run || code === undefined) break;
    const before = performance.now();
    run.page = createPage(run.slot);
    const block = { type: "code" as const, lang: "ts", meta: null, value: code };
    run.page.render({ type: "root", children: styled ? [{ type: "componentContainer", name: "box", params: [{ key: "width", breakpoint: null, value: `${String(50 + (i % 40))}%` }], children: [block], slots: [] }] : [block] });
    live.push(run);
    if (live.length > 8) unmountNow(live.shift());
    // Until the frame after: the open's own work and the styling it left the browser to do.
    opens.push((await nextFrame()) - before);
  }
  const end = performance.now();
  return {
    config: { churn: true, hostElements: elements, styled },
    counts,
    totalMs: Math.round(end - start),
    ...blocked(start, end),
    firstTenOpenMs: percentile(opens.slice(0, 10), 0.5),
    lastTenOpenMs: percentile(opens.slice(-10), 0.5),
    longestOpenMs: Math.round(Math.max(...opens)),
  };
}

/**
 * What writing a block's rules costs in a large host document, apart from the renderer (CMD-254): one adopted sheet,
 * the renderer's base rules in it, then 100 blocks added and removed, eight live at a time, one a frame, each with a
 * rule of its own and one in a container query, as a styled block's. `?sheet=replace` writes the whole sheet's text
 * each time (`replaceSync`, as `Stylesheet.flush` does), `?sheet=insert` inserts and deletes the block's rules
 * (`insertRule`, `deleteRule`), `?sheet=none` writes no rule, for the blocks' own cost.
 */
async function churnSheet(): Promise<unknown> {
  observer.disconnect();
  for (const run of slots) unmountNow(run);
  const mode = query.get("sheet") ?? "replace";
  const host = scroller.appendChild(document.createElement("div"));
  host.className = "host";
  const elements = Number(query.get("host") ?? 20_000);
  for (let i = 0; i < elements / 2; i++) {
    const row = host.appendChild(document.createElement("div"));
    row.className = `row r${String(i % 7)}`;
    row.appendChild(document.createElement("span")).textContent = `Row ${String(i)}`;
  }
  // The base rules as the renderer has them: what a page's own sheet held before CMD-253.
  const probe = document.createElement("div");
  const page = createPage(probe);
  const base = page.stylesheet.text;
  page.destroy();
  const sheet = new CSSStyleSheet();
  sheet.replaceSync(base);
  document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet];
  await nextFrame();
  await pause(500);
  const counts = counted();
  const live: { element: HTMLElement; rules: readonly string[] }[] = [];
  const opens: number[] = [];
  const rulesOf = (i: number) => [`.blk${String(i)} { width: ${String(50 + (i % 40))}%; }`, `@container (min-width: 480px) { .blk${String(i)} { padding: ${String(i % 9)}px; } }`];
  const write = () => {
    sheet.replaceSync([base, ...live.flatMap(({ rules }) => rules)].join("\n"));
  };
  const start = performance.now();
  for (let i = 0; i < 100; i++) {
    const before = performance.now();
    const element = scroller.appendChild(document.createElement("div"));
    element.className = `compmd-page blk${String(i)}`;
    element.textContent = runs[i] ?? "";
    const rules = rulesOf(i);
    live.push({ element, rules });
    if (mode === "insert") for (const rule of rules) sheet.insertRule(rule, sheet.cssRules.length);
    const gone = live.length > 8 ? live.shift() : undefined;
    gone?.element.remove();
    if (gone && mode === "insert") {
      const selector = `.${gone.element.classList[1] ?? ""} {`;
      for (let at = sheet.cssRules.length - 1; at >= 0; at--) if (sheet.cssRules[at]?.cssText.includes(selector)) sheet.deleteRule(at);
    }
    if (mode === "replace") write();
    opens.push((await nextFrame()) - before);
  }
  const end = performance.now();
  return {
    config: { churnSheet: true, hostElements: elements, sheet: mode },
    counts,
    rules: sheet.cssRules.length,
    totalMs: Math.round(end - start),
    ...blocked(start, end),
    firstTenOpenMs: percentile(opens.slice(0, 10), 0.5),
    lastTenOpenMs: percentile(opens.slice(-10), 0.5),
    longestOpenMs: Math.round(Math.max(...opens)),
  };
}

/**
 * What each frame of a page pays to highlight its first block (CMD-260): `?frames=N` frames, one after the other, each
 * a document of its own that makes the sandbox's highlighter and asks for a TypeScript block's tokens, with Oniguruma
 * (`&wasm=1`) or the JavaScript engine. The files come from the browser's cache after the first frame; the compiling
 * is each frame's own. In WebKit the frames run in the page's thread, so the page's frames show what they block.
 */
async function perFrame(): Promise<unknown> {
  observer.disconnect();
  const count = Number(query.get("frames") ?? 12);
  const wasm = query.get("wasm") === "1";
  const module = new URL("frame.js", location.href).href;
  const ms: number[] = [];
  const start = performance.now();
  for (let i = 0; i < count; i++) {
    const frame = document.body.appendChild(document.createElement("iframe"));
    const heard = new Promise<number>((resolve) => {
      const listener = ({ data, source }: MessageEvent<number>) => {
        if (source !== frame.contentWindow) return;
        removeEventListener("message", listener);
        resolve(data);
      };
      addEventListener("message", listener);
    });
    // A frame that fails says so with -1, so the run goes on and the result shows it.
    frame.srcdoc = `<script type="module">
      try {
        const { firstTokens } = await import(${JSON.stringify(module)});
        parent.postMessage(await firstTokens(${JSON.stringify(runs[i] ?? "")}, ${String(wasm)}), "*");
      } catch (error) {
        console.error(String(error));
        parent.postMessage(-1, "*");
      }
    </script>`;
    ms.push(await heard);
    frame.remove();
  }
  const end = performance.now();
  const rest = ms.slice(1);
  return {
    config: { frames: count, wasm },
    firstFrameMs: Math.round(ms[0] ?? 0),
    medianFrameMs: percentile(rest, 0.5),
    longestFrameMs: Math.round(Math.max(...rest)),
    ...blocked(start, end),
  };
}

/** How many blocks each frame of `?framepage=N` highlights: a short post's. */
const BLOCKS_PER_FRAME = Number(query.get("blocks") ?? 5);

/**
 * A page of documents in frame mode (CMD-267): `?framepage=N` frames live at once, each from another origin, as the
 * sandbox's are (so Chromium gives each a process of its own), each highlighting `BLOCKS_PER_FRAME` TypeScript runs.
 * `&where=frame`, as today: each frame with a highlighter of its own. `&where=host`: each frame asks the page over a
 * port, and the page highlights every frame's blocks with one. Both with `webAssembly`, so the engine is the
 * sandbox's choice: Oniguruma in WebKit, the JavaScript engine in Chromium. The frames stay, for the runner to read
 * their memory.
 */
async function framePage(): Promise<unknown> {
  observer.disconnect();
  const count = Number(query.get("framepage"));
  const askThePage = query.get("where") === "host";
  const pageHighlight = askThePage ? shikiHighlight({ webAssembly: true }) : undefined;
  const frameOrigin = `http://127.0.0.1:${location.port}`;
  const answerOn = (port: MessagePort) => {
    port.onmessage = async ({ data }: MessageEvent<TokensRequest>) => {
      port.postMessage({ id: data.id, lines: await pageHighlight?.(data.code, data.language) } satisfies TokensAnswer);
    };
  };
  const openFrame = (index: number) =>
    new Promise<FrameReport>((resolve) => {
      const frame = document.body.appendChild(document.createElement("iframe"));
      const blocks = Array.from({ length: BLOCKS_PER_FRAME }, (_, i) => runs[(index * BLOCKS_PER_FRAME + i) % runs.length] ?? "");
      const listener = ({ data, source }: MessageEvent<"ready" | FrameReport>) => {
        if (source !== frame.contentWindow) return;
        if (data !== "ready") {
          removeEventListener("message", listener);
          resolve(data);
          return;
        }
        const channel = new MessageChannel();
        if (askThePage) answerOn(channel.port1);
        frame.contentWindow?.postMessage({ blocks, askThePage }, frameOrigin, askThePage ? [channel.port2] : []);
      };
      addEventListener("message", listener);
      frame.src = `${frameOrigin}/frame-page.html`;
    });
  const start = performance.now();
  const reports = await Promise.all(Array.from({ length: count }, (_, index) => openFrame(index)));
  const end = performance.now();
  return {
    config: { framepage: count, blocks: BLOCKS_PER_FRAME, where: askThePage ? "host" : "frame" },
    pageMs: Math.round(end - start),
    medianFirstBlockMs: percentile(reports.map(({ firstMs }) => firstMs), 0.5),
    longestFirstBlockMs: Math.round(Math.max(...reports.map(({ firstMs }) => firstMs))),
    medianFrameMs: percentile(reports.map(({ allMs }) => allMs), 0.5),
    ...blocked(start, end),
  };
}

/** A run's page gone now, whatever `?keep` says. */
function unmountNow(run: Run | undefined): void {
  run?.page?.destroy();
  delete run?.page;
  run?.slot.replaceChildren();
}

declare global {
  interface Window {
    spike: { run(): Promise<unknown>; churn(): Promise<unknown> };
  }
}

window.spike = {
  churn: () => (query.has("framepage") ? framePage() : query.has("frames") ? perFrame() : query.has("sheet") ? churnSheet() : churn()),
  async run() {
    const opened = performance.now();
    // The first runs mount when the observer first reports, a frame or so after the page loads.
    for (let tries = 0; tries < 250 && !slots.some((run) => run.page); tries++) await pause(20);
    console.log(`opening: ${String(slots.filter((run) => run.page).length)} runs mounted`);
    await idle();
    console.log(`opened: ${String(slots.filter((run) => run.highlighted !== undefined).length)} highlighted`);
    await pause(500);
    const open = { ...blocked(0, performance.now()), ...waits(0, performance.now()), loadToHighlightedMs: Math.round(performance.now() - 500) };
    const afterOpen = costs();

    const skimStart = performance.now();
    await skim();
    console.log(`skimmed: ${String(slots.filter((run) => run.page).length)} mounted`);
    await idle();
    console.log(`skim idle: ${String(slots.filter((run) => run.highlighted !== undefined).length)} highlighted`);
    await pause(500);
    const skimEnd = performance.now();
    const skimmed = { ...blocked(skimStart, skimEnd), ...waits(skimStart, skimEnd), toAllHighlightedMs: Math.round(skimEnd - 500 - skimStart) };
    const afterSkim = costs();

    // A reader jumping back: every run's page gone, then each mounted again as the list is skimmed once more.
    for (const run of slots) {
      run.page?.destroy();
      delete run.page;
      run.slot.replaceChildren();
    }
    scroller.scrollTop = 0;
    await pause(300);
    const revisitStart = performance.now();
    await idle();
    await skim();
    await idle();
    await pause(500);
    const revisitEnd = performance.now();
    const revisit = { ...blocked(revisitStart, revisitEnd), ...waits(revisitStart, revisitEnd) };

    return { config, runs: runs.length, openedAtMs: Math.round(opened), open, skim: skimmed, revisit, costs: { afterOpen, afterSkim, total: costs() } };
  },
};
