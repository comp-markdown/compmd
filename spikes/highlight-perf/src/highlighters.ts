// The variants' highlighters, `@compmd/render`'s `CodeHighlighter`, each made of the choices the spike tries: where
// Shiki lives (one per view, as the sandbox's host mode has it today, or one for the page), its engine, where it
// tokenizes (the page's thread or a worker), when blocks are highlighted (each as it renders, or one per task with
// those on screen first), and a cache of tokens. Every step is counted in `stats`.

import type { CodeHighlighter } from "@compmd/render";
import type { HighlighterCore } from "@shikijs/core";
import { draw } from "./draw.ts";
import { loadShiki, tokensOf, type Engine, type Lines } from "./shiki.ts";

export interface Config {
  shiki: "per-view" | "shared";
  engine: Engine;
  where: "main" | "worker";
  schedule: "eager" | "sliced";
  cache: boolean;
}

/** What the highlighting cost, on the page's thread unless named. */
export const stats = {
  /** Shiki made, and how long each took (in the worker, for `where: "worker"`). */
  setups: [] as number[],
  /** Each tokenizing, in order: the first ones carry the regexes' compiling. */
  tokenize: [] as number[],
  /** Drawing the tokens into the lines. */
  draw: [] as number[],
  cacheHits: 0,
};

type Tokenize = (code: string) => Promise<Lines>;

/** Tokenizing on the page's thread, with one Shiki made by `shiki` the first time. */
function onPage(engine: Engine): Tokenize {
  let shiki: Promise<HighlighterCore> | undefined;
  return async (code) => {
    shiki ??= loadShiki(engine).then(({ shiki: made, ms }) => {
      stats.setups.push(ms);
      return made;
    });
    const loaded = await shiki;
    const started = performance.now();
    const lines = tokensOf(loaded, code);
    stats.tokenize.push(performance.now() - started);
    return lines;
  };
}

/** Tokenizing in one worker for the page. */
function inWorker(engine: Engine): Tokenize {
  const worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
  const waiting = new Map<number, (lines: Lines) => void>();
  let last = 0;
  let setupCounted = false;
  worker.addEventListener("message", (event: MessageEvent<{ id: number; lines: Lines; ms: number; setup: number }>) => {
    const { id, lines, ms, setup } = event.data;
    if (!setupCounted) {
      stats.setups.push(setup);
      setupCounted = true;
    }
    stats.tokenize.push(ms);
    waiting.get(id)?.(lines);
    waiting.delete(id);
  });
  return (code) =>
    new Promise((resolve) => {
      const id = ++last;
      waiting.set(id, resolve);
      worker.postMessage({ id, code, engine });
    });
}

/** `tokenize` with its tokens kept by their text, for a block seen again. */
function cached(tokenize: Tokenize): Tokenize {
  const kept = new Map<string, Lines>();
  return async (code) => {
    const found = kept.get(code);
    if (found) {
      stats.cacheHits += 1;
      return found;
    }
    const lines = await tokenize(code);
    kept.set(code, lines);
    return lines;
  };
}

/** The next task, after the browser has had its turn: a message, which no timer clamps. */
const nextTask = () =>
  new Promise<void>((resolve) => {
    const { port1, port2 } = new MessageChannel();
    port1.onmessage = () => {
      resolve();
    };
    port2.postMessage(null);
  });

const shows = (element: HTMLElement) => {
  const { top, bottom } = element.getBoundingClientRect();
  return bottom > 0 && top < innerHeight;
};

/**
 * Makes each view's highlighter, as `config` says; `drawn` is told when a block's tokens are drawn. With `shiki:
 * "per-view"` each call makes a Shiki of its own (a worker is the page's either way).
 */
export function highlighters(config: Config, drawn: (target: HTMLElement) => void): () => CodeHighlighter {
  const make = (): Tokenize => {
    const tokenize = config.where === "worker" ? inWorker(config.engine) : onPage(config.engine);
    return config.cache ? cached(tokenize) : tokenize;
  };
  const shared = config.shiki === "shared" || config.where === "worker" ? make() : undefined;

  const show = (lines: Lines, code: string, target: HTMLElement) => {
    const started = performance.now();
    if (draw(lines, code, target)) drawn(target);
    stats.draw.push(performance.now() - started);
  };

  // Sliced: one block per task, those on screen first, across every view.
  const queue: { code: string; target: HTMLElement; tokenize: Tokenize }[] = [];
  let pumping = false;
  const pump = async () => {
    pumping = true;
    while (queue.length > 0) {
      await nextTask();
      const at = Math.max(
        0,
        queue.findIndex((job) => shows(job.target)),
      );
      const [job] = queue.splice(at, 1);
      if (!job) continue;
      show(await job.tokenize(job.code), job.code, job.target);
    }
    pumping = false;
  };

  return () => {
    const tokenize = shared ?? make();
    return (code, _language, target) => {
      if (config.schedule === "sliced") {
        queue.push({ code, target, tokenize });
        if (!pumping) void pump();
        return;
      }
      void tokenize(code).then((lines) => {
        show(lines, code, target);
      });
    };
  };
}
