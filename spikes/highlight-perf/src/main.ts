// The test page: hundreds of short TypeScript runs, each a page of its own (`createPage`), mounted when it comes
// within a screen of view, as a diff viewer's list is. The variant is the URL's query (`highlighters.ts`). The page
// records every animation frame, so the runner can read how long frames blocked in each phase: opening, a skim to the
// end, and a revisit, every run mounted again, as a reader jumping back.

import { createPage, type CodeHighlighter, type Page } from "@compmd/render";
import { shikiHighlight } from "@compmd/sandbox";
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
const sandboxHighlight = (() => {
  if (sandbox !== "per-view" && sandbox !== "shared") return undefined;
  const one = shikiHighlight();
  return (slot: HTMLElement): CodeHighlighter => {
    const highlight = sandbox === "shared" ? one : shikiHighlight();
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
  churn,
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
