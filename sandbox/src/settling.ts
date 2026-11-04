// When a window's page has settled (docs/sandbox-protocol.md, Settled): what it renders drawn, the drawings its
// components gave `busy` settled, its own frames settled and sized, its images, stylesheets and fonts loaded, then an
// animation frame in which nothing moved. Each change starts it again, so it settles after the latest.

/** What a window waits for before it is settled. */
export interface SettlingParts {
  /** What renders in the window has drawn: its page's or region's `drawn()`, and its code loaded. */
  drawn(): Promise<unknown>;
  /** The window's own frames, each settled after what it was sent. */
  frames(): Iterable<{ settled(): Promise<void> }>;
  /** Where its images are. */
  root(): ParentNode | undefined;
}

/** Resolves once every image under `root` has loaded or failed. */
function imagesLoaded(root: ParentNode | undefined): Promise<unknown> {
  const loading = [...(root?.querySelectorAll("img") ?? [])].filter((image) => !image.complete);
  return Promise.all(loading.map((image) => new Promise((resolve) => {
    image.addEventListener("load", resolve, { once: true });
    image.addEventListener("error", resolve, { once: true });
  })));
}

/** Resolves once every stylesheet the window links has loaded or failed: a host's stylesheet moves its page too. */
function stylesheetsLoaded(): Promise<unknown> {
  const loading = [...document.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]')].filter((link) => link.sheet === null);
  return Promise.all(loading.map((link) => new Promise((resolve) => {
    link.addEventListener("load", resolve, { once: true });
    link.addEventListener("error", resolve, { once: true });
  })));
}

/** The next animation frame, once the browser has laid out what came before it. */
const nextFrame = () => new Promise<void>((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));

/** A window's settling, which `changed` starts again, and `onSettled` hears each time it is done. */
export class Settling {
  readonly #parts: SettlingParts;
  readonly #onSettled: () => void;
  #version = 0;
  /** The version the window last settled at; -1 before it ever did. */
  #settledAt = -1;
  #running = false;
  #waiting: (() => void)[] = [];

  constructor(parts: SettlingParts, onSettled: () => void) {
    this.#parts = parts;
    this.#onSettled = onSettled;
  }

  /** Whether the window is settled now. */
  get settled(): boolean {
    return !this.#running && this.#settledAt === this.#version;
  }

  /** What the window shows changed, or moved: it settles again. */
  changed(): void {
    this.#version += 1;
    void this.#run();
  }

  /** Resolves the next time the window is settled: at once when it is. */
  whenSettled(): Promise<void> {
    if (this.settled) return Promise.resolve();
    return new Promise((resolve) => this.#waiting.push(resolve));
  }

  async #run(): Promise<void> {
    if (this.#running) return;
    this.#running = true;
    try {
      for (;;) {
        const version = this.#version;
        // A task first: the listeners of the message that changed it, a renderer's among them, run before.
        await new Promise((resolve) => setTimeout(resolve, 0));
        await this.#parts.drawn();
        await Promise.all([...this.#parts.frames()].map((frame) => frame.settled()));
        await imagesLoaded(this.#parts.root());
        await stylesheetsLoaded();
        await document.fonts.ready;
        await nextFrame();
        if (version === this.#version) break;
      }
    } finally {
      this.#running = false;
    }
    this.#settledAt = this.#version;
    const waiting = this.#waiting;
    this.#waiting = [];
    for (const resolve of waiting) resolve();
    this.#onSettled();
  }
}
