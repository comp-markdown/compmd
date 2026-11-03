// The worker's part: it keeps the open document and answers each message with what it did, each splice as the
// processor's JSON text, which goes to the frame unread. Packs the document imports that the playground doesn't
// have come from the registry its configuration names (docs/packs-from-a-registry.md, R4): the document renders at
// once with what it has, and when they arrive the editor opens it again, with them.

import type { ChangesJson, HostComponent, NeededPack, OpenDocumentJson, PackRelease, Request } from "@compmd/processor";
import type { FromWorker, ToWorker } from "./protocol.ts";

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

/** Where a built module is in its pack. */
type ModulePlace = NonNullable<HostComponent["module"]>;

/** What resolving a request's missing packs gave, as `@compmd/registry`'s `resolvePacks` does. */
export interface Resolution {
  request: Request;
  modules(place: ModulePlace): { code: Blob } | undefined;
  problems: { pack: string; message: string }[];
}

/**
 * What fetching packs gives the editor: whether to open the document again (packs arrived, or began or stopped being
 * pending), and what couldn't be had.
 */
export interface Fetched {
  again: boolean;
  problems: string[];
}

export interface ProcessingOptions {
  /** Resolves a request's missing packs from the registry; without it, a pack the request doesn't hold stays F026. */
  resolve?: (request: Request) => Promise<Resolution>;
  /** Packs were fetched, or couldn't be. */
  onFetched?: (fetched: Fetched) => void;
  /** Remote URLs the document needs that weren't reported before, for the page to ask about and fetch. */
  onRemote?: (urls: string[]) => void;
  /** How long typing must pause before missing packs are asked for, so a name typed letter by letter isn't (ms). */
  pause?: number;
}

/** One import as the processor lists it. */
const keyOf = ({ pack, range, integrity }: NeededPack): string => `${pack}@${range}#${integrity ?? ""}`;

/** A built module's place as one key, `pack@version/path`. */
export const placeKey = ({ pack, version, path }: ModulePlace): string => `${pack}@${version}/${path}`;

export class Processing {
  readonly #open: (request: Request) => OpenDocumentJson;
  readonly #options: ProcessingOptions;
  #document: OpenDocumentJson | undefined;
  /** The request opened last, as the editor sent it, and its document's text after the edits since. */
  #request: Request | undefined;
  #text = "";
  /** The imports asked for already, so each is fetched once. */
  readonly #asked = new Set<string>();
  /** The remote URLs reported already, so each is reported once. */
  readonly #reported = new Set<string>();
  /** The releases fetched, and the packs the registry's size cap refused, which every request opened gets. */
  #fetched: Record<string, PackRelease[]> = {};
  #refused: Record<string, string> = {};
  /** The packs being fetched, which the document is opened with as pending (spec §6.4): loading, not F026. */
  readonly #pending = new Set<string>();
  readonly #resolutions: Resolution[] = [];
  #waiting: ReturnType<typeof setTimeout> | undefined;

  /** Processes with documents `open` makes: `new OpenDocumentJson(request)` in the worker. */
  constructor(open: (request: Request) => OpenDocumentJson, options: ProcessingOptions = {}) {
    this.#open = open;
    this.#options = { ...options };
  }

  /** Resolves missing packs with `resolve` from now on, as the page gives the worker its registries. */
  useRegistry(resolve: NonNullable<ProcessingOptions["resolve"]>): void {
    this.#options.resolve = resolve;
  }

  /**
   * What `message` gives: a splice per edit, the diagnostics after them, the components when they changed, with the
   * built modules fetched for them, and the hint at the caret; or only the hint, when only the caret moved.
   */
  handle(message: ToWorker): FromWorker {
    const started = performance.now();
    try {
      if (message.type === "hint") {
        const hint = this.#document?.hintAt(message.caret);
        return { type: "hint", ...(hint ? { hint } : {}) };
      }
      const changes = message.type === "open" ? [this.#opened(message.request)] : this.#edited(message.edits);
      const document = this.#document;
      if (!document) return { type: "failed", message: "no document is open" };
      const components = changes.findLast((change) => change.components)?.components;
      const last = changes.at(-1) ?? document.changes();
      const hint = document.hintAt(message.caret);
      const modules = components && this.#modulesOf(components);
      this.#askLater();
      return {
        type: "processed",
        splices: changes.map((change) => change.splice),
        opened: message.type === "open",
        diagnostics: last.diagnostics,
        inputs: last.inputs,
        ...(components ? { components } : {}),
        ...(modules && Object.keys(modules).length > 0 ? { modules } : {}),
        ...(hint ? { hint } : {}),
        ms: performance.now() - started,
      };
    } catch (error) {
      return { type: "failed", message: messageOf(error) };
    }
  }

  #opened(request: Request): ChangesJson {
    this.#document?.free();
    this.#document = undefined;
    this.#request = request;
    this.#text = request.files?.[request.path] ?? "";
    this.#document = this.#open(this.#withFetched(request));
    // Packs it imports that will be fetched are pending from the first render, so it shows them loading at once.
    const fresh = this.#options.resolve ? this.#document.sources().packs.filter((needed) => !this.#asked.has(keyOf(needed))) : [];
    if (fresh.length > 0) {
      for (const { pack } of fresh) this.#pending.add(pack);
      this.#document.free();
      this.#document = this.#open(this.#withFetched(request));
      void this.#ask();
    }
    return this.#document.changes();
  }

  #edited(edits: readonly { start: number; end: number; text: string }[]): ChangesJson[] {
    const document = this.#document;
    if (!document) return [];
    return edits.map(({ start, end, text }) => {
      this.#text = this.#text.slice(0, start) + text + this.#text.slice(end);
      return document.edit(start, end, text);
    });
  }

  /** `request` with the packs fetched so far, its own over them, and those still being fetched as pending. */
  #withFetched(request: Request): Request {
    const pending = this.#pending.size > 0 ? { pending: { ...request.pending, packs: [...(request.pending?.packs ?? []), ...this.#pending] } } : {};
    return { ...request, packs: { ...this.#fetched, ...request.packs }, policy: { ...request.policy, refused: { ...this.#refused, ...request.policy?.refused } }, ...pending };
  }

  /** The fetched text of each built module `components` load, by its place. */
  #modulesOf(components: readonly HostComponent[]): Record<string, Blob> {
    const modules: Record<string, Blob> = {};
    for (const { module } of components) {
      if (!module) continue;
      const code = this.#resolutions.map((resolution) => resolution.modules(module)).find((found) => found !== undefined)?.code;
      if (code) modules[placeKey(module)] = code;
    }
    return modules;
  }

  /** Asks for the document's missing packs, and reports the remote URLs it needs, once typing pauses. */
  #askLater(): void {
    if (!this.#options.resolve && !this.#options.onRemote) return;
    clearTimeout(this.#waiting);
    this.#waiting = setTimeout(() => {
      this.#report();
      void this.#ask();
    }, this.#options.pause ?? 600);
  }

  /** Reports the remote URLs the document needs that weren't reported before: of allowed origins, and of others. */
  #report(): void {
    const onRemote = this.#options.onRemote;
    const sources = this.#document?.sources();
    const urls = sources ? [...sources.remote, ...sources.disallowed].filter((url) => !this.#reported.has(url)) : [];
    if (!onRemote || urls.length === 0) return;
    for (const url of urls) this.#reported.add(url);
    onRemote(urls);
  }

  async #ask(): Promise<void> {
    const resolve = this.#options.resolve;
    const request = this.#request;
    const missing = this.#document?.sources().packs.filter((needed) => !this.#asked.has(keyOf(needed))) ?? [];
    if (!resolve || !request || missing.length === 0) return;
    for (const needed of missing) this.#asked.add(keyOf(needed));
    // An import typed since the document opened: pending now, so the editor opens it again showing it loading.
    const fresh = missing.filter(({ pack }) => !this.#pending.has(pack));
    for (const { pack } of missing) this.#pending.add(pack);
    if (fresh.length > 0) this.#options.onFetched?.({ again: true, problems: [] });
    try {
      const resolution = await resolve(this.#withFetched({ ...request, files: { ...request.files, [request.path]: this.#text } }));
      this.#resolutions.push(resolution);
      for (const [pack, releases] of Object.entries(resolution.request.packs ?? {})) if (!request.packs?.[pack]) this.#fetched[pack] = releases;
      this.#refused = { ...this.#refused, ...resolution.request.policy?.refused };
      const problems = resolution.problems.map(({ pack, message }) => `${pack}: ${message}`);
      for (const { pack } of missing) this.#pending.delete(pack);
      // Arrived, or given up on: either way the document opens again, without them pending.
      this.#options.onFetched?.({ again: true, problems });
    } catch (error) {
      for (const { pack } of missing) this.#pending.delete(pack);
      this.#options.onFetched?.({ again: true, problems: [messageOf(error)] });
    }
  }
}
