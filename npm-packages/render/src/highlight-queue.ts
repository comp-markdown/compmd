// The code blocks of a document waiting to be highlighted, one at a time: the highlighter is asked for a block's
// tokens, which are drawn, and the next block is asked for a task later, so a render of many blocks never tokenizes
// them all in one frame. Blocks on screen go first, as an `IntersectionObserver` reports them, which reads no layout.
// A block whose text changed is asked for again with its new text, and one that is gone is dropped.
import { drawTokens, type CodeHighlighter } from "./highlight.ts";
import { nextTask } from "./next-task.ts";

/** A block waiting: what to ask for, and what to tell once it is drawn or dropped. */
interface Job {
  target: HTMLElement;
  code: string;
  language: string;
  highlight: CodeHighlighter;
  done: () => void;
}

/** One document's queue. */
class HighlightQueue {
  static readonly #all = new WeakMap<Document, HighlightQueue>();
  /** The blocks waiting, by their `<code>`, in the order they were asked for. */
  readonly #waiting = new Map<HTMLElement, Job>();
  /** The blocks on screen, as the observer last said. */
  readonly #shown = new WeakSet<Element>();
  readonly #observer: IntersectionObserver | undefined;
  #running = false;

  static of(document: Document): HighlightQueue {
    let queue = HighlightQueue.#all.get(document);
    if (!queue) HighlightQueue.#all.set(document, (queue = new HighlightQueue(document)));
    return queue;
  }

  private constructor(document: Document) {
    const Observer = document.defaultView?.IntersectionObserver;
    this.#observer = Observer
      ? new Observer((entries) => {
          for (const { target, isIntersecting } of entries) {
            if (isIntersecting) this.#shown.add(target);
            else this.#shown.delete(target);
          }
        })
      : undefined;
  }

  /** Asks for `job`'s block: in place of what it waited for before, which is told it is done. */
  add(job: Job): void {
    const before = this.#waiting.get(job.target);
    if (before) {
      this.#waiting.delete(job.target);
      before.done();
    } else {
      this.#observer?.observe(job.target);
    }
    this.#waiting.set(job.target, job);
    if (!this.#running) void this.#run();
  }

  /** The block is gone: it isn't highlighted. */
  drop(target: HTMLElement): void {
    const job = this.#waiting.get(target);
    if (!job) return;
    this.#forget(job);
    job.done();
  }

  #forget(job: Job): void {
    this.#waiting.delete(job.target);
    this.#observer?.unobserve(job.target);
  }

  /** The next block: the first on screen, else the first asked for. */
  #next(): Job | undefined {
    for (const job of this.#waiting.values()) if (this.#shown.has(job.target)) return job;
    return this.#waiting.values().next().value;
  }

  async #run(): Promise<void> {
    this.#running = true;
    try {
      for (;;) {
        // A task first: the render that asked is placed, and the observer has had its say.
        await nextTask();
        const job = this.#next();
        if (!job) break;
        this.#forget(job);
        await this.#highlight(job);
        job.done();
      }
    } finally {
      this.#running = false;
    }
  }

  /** Asks for `job`'s tokens and draws them, unless its block has been asked for again meanwhile. */
  async #highlight({ target, code, language, highlight }: Job): Promise<void> {
    try {
      const lines = await highlight(code, language);
      if (lines && !this.#waiting.has(target)) drawTokens(target, code, lines);
    } catch {
      // The block keeps its text.
    }
  }
}

/**
 * Highlights `target`, a block's `<code>` holding `code` as its lines, with `highlight`, when its turn comes in its
 * document's queue: its tokens drawn unless the block has changed or gone since. Resolves once they are drawn, or the
 * block is dropped, asked for again, or keeps its text.
 */
export function highlightCode(target: HTMLElement, code: string, language: string, highlight: CodeHighlighter): Promise<void> {
  return new Promise((done) => {
    HighlightQueue.of(target.ownerDocument).add({ target, code, language, highlight, done });
  });
}

/** `target`'s block is gone: whatever it waited for isn't highlighted. */
export function dropHighlight(target: HTMLElement): void {
  HighlightQueue.of(target.ownerDocument).drop(target);
}
