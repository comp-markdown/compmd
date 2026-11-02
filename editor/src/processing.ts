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

/** What fetched packs give the editor: whether to open the document again with them, and what couldn't be had. */
export interface Fetched {
  arrived: boolean;
  problems: string[];
}

export interface ProcessingOptions {
  /** Resolves a request's missing packs from the registry; without it, a pack the request doesn't hold stays F026. */
  resolve?: (request: Request) => Promise<Resolution>;
  /** Packs were fetched, or couldn't be. */
  onFetched?: (fetched: Fetched) => void;
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
  /** The releases fetched, and the packs the registry's size cap refused, which every request opened gets. */
  #fetched: Record<string, PackRelease[]> = {};
  #refused: Record<string, string> = {};
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

  /** `request` with the packs fetched so far, its own over them. */
  #withFetched(request: Request): Request {
    return { ...request, packs: { ...this.#fetched, ...request.packs }, policy: { ...request.policy, refused: { ...this.#refused, ...request.policy?.refused } } };
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

  /** Asks for the document's missing packs once typing pauses. */
  #askLater(): void {
    if (!this.#options.resolve) return;
    clearTimeout(this.#waiting);
    this.#waiting = setTimeout(() => void this.#ask(), this.#options.pause ?? 600);
  }

  async #ask(): Promise<void> {
    const resolve = this.#options.resolve;
    const request = this.#request;
    const missing = this.#document?.sources().packs.filter((needed) => !this.#asked.has(keyOf(needed))) ?? [];
    if (!resolve || !request || missing.length === 0) return;
    for (const needed of missing) this.#asked.add(keyOf(needed));
    try {
      const resolution = await resolve(this.#withFetched({ ...request, files: { ...request.files, [request.path]: this.#text } }));
      this.#resolutions.push(resolution);
      const before = Object.keys(this.#fetched).length;
      for (const [pack, releases] of Object.entries(resolution.request.packs ?? {})) if (!request.packs?.[pack]) this.#fetched[pack] = releases;
      this.#refused = { ...this.#refused, ...resolution.request.policy?.refused };
      const problems = resolution.problems.map(({ pack, message }) => `${pack}: ${message}`);
      this.#options.onFetched?.({ arrived: Object.keys(this.#fetched).length > before, problems });
    } catch (error) {
      this.#options.onFetched?.({ arrived: false, problems: [messageOf(error)] });
    }
  }
}
