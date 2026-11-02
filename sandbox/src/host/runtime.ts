// The host runtime (docs/running-components-in-the-sandbox.md, Q6, Q7): in the host's page, one per document. It
// turns the processor's components into what frames run (compiled in the document's compile frame, or built modules
// by their URL or as files the host holds, with the library packs they import), mounts trusted components inline and frames every other use at its level, gives `host` to permissive
// frames of packs given it, and stops a frame that stops answering.

import type { HostComponent, Trust } from "@compmd/processor";
import { createRegion, type ExpandedInput, type ExpandedUse, type Mount } from "@compmd/render";
import { serveHost, type HostApi } from "../host-api.ts";
import { codeFor, Mounts } from "../mounts.ts";
import { connectCompiler, type Compiler } from "../protocol/compiler.ts";
import { bindingOf, HELLO, SANDBOX_VERSION, type ComponentCode, type ErrorReason, type Level, type Libraries, type LibraryCode, type Scheme } from "../protocol/messages.ts";
import { frameUse } from "../use.ts";

type Place = NonNullable<HostComponent["module"]>;

/**
 * A built module: where it is served, with its file's hash, which the host checked when it got the pack; or its file
 * as a `Blob` the host has checked, as a resolver of registry packs gives it, which frames load as a `blob:` module.
 * A library pack's module is asked for the same way, and must be given as code.
 */
export type ModuleSource = (module: Place) => { url: string; integrity: string } | { code: Blob };

/** A built module's place in its pack, `pack@version/path`. */
const placeOf = ({ pack, version, path }: Place): string => `${pack}@${version}/${path}`;

export interface HostRuntimeOptions {
  /** Where the host serves the frame documents (`buildFrames`), ending in `/`: a full URL, or one relative to the page. */
  base: string;
  /** The host's own components, which run in its page: those the processor marks trusted. */
  trusted?: Readonly<Record<string, Mount>>;
  /** What permissive components of packs given `host` can call. */
  host?: HostApi;
  /** Where each built module is served, or its file; without it, a built module's uses show their fallbacks. */
  modules?: ModuleSource;
  /** The host's stylesheets, which every frame loads; served from the frames' origin. */
  stylesheets?: readonly string[];
  /** How long a frame may not answer before it is stopped, in ms (default 5000). */
  timeout?: number;
}

/** What a document's components do that the host hears of. */
export interface HostEvents {
  set(input: string, value: unknown): void;
  /** A link a reader clicked in a frame; the host shows its URL before it opens (spec §15 rule 4). */
  link(url: string): void;
  error(component: string, reason: ErrorReason, message: string): void;
}

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

/** The frame document `name` where the host serves the frame documents, `base`, full or relative to the page. */
export const frameUrl = (base: string, name: "strict.html" | "permissive.html"): string => new URL(name, new URL(base, location.href)).href;

/** A level as a frame has it; a trusted component never runs in one. */
const levelOf = (trust: Trust): Level | "trusted" => trust.level;

/**
 * The host runtime of one document.
 *
 * @example
 * const runtime = new HostRuntime({ base: "/compmd/", trusted: { stars: react(Stars) } }, events);
 * await runtime.set(processed.hostComponents);
 * createPage(container, { components: runtime.mounts }).render(processed.expanded);
 */
export class HostRuntime {
  readonly #options: HostRuntimeOptions;
  readonly #events: HostEvents;
  readonly #mounts: Mounts;
  #code: ComponentCode[] = [];
  #trust = new Map<string, Trust>();
  #compiler: { compiler: Compiler; frame: HTMLIFrameElement } | undefined;
  #styles: readonly string[];

  constructor(options: HostRuntimeOptions, events: HostEvents) {
    this.#options = options;
    this.#events = events;
    this.#styles = options.stylesheets ?? [];
    this.#mounts = new Mounts({
      here: "host",
      inline: options.trusted ?? {},
      frame: (target, node, inputs, component) => this.#frame(target, node, inputs, component),
      report: (component, reason, message) => {
        events.error(component, reason, message);
      },
    });
  }

  /** The components for a renderer in the host's page, the same object as they change. */
  get mounts(): Record<string, Mount> {
    return this.#mounts.mounts;
  }

  /** What each frame of the document runs, as `set` last made it. */
  get code(): readonly ComponentCode[] {
    return this.#code;
  }

  /** The document's components are now `components`: resolves once each is compiled, or has failed. */
  async set(components: readonly HostComponent[]): Promise<void> {
    this.#trust = new Map(components.map((component) => [component.name, component.trust]));
    const code = await Promise.all(components.map((component) => this.#codeOf(component)));
    // A component whose code didn't compile or has no URL is left out: its uses show their fallbacks, as missing.
    this.#code = code.filter((component) => component !== undefined);
    this.#mounts.set(this.#code);
  }

  /**
   * Mounts one use in `target`, inline or in a frame of its level, as an editor's widget does; the handle updates and
   * removes it.
   */
  mountUse(target: HTMLElement, node: ExpandedUse, inputs: Readonly<Record<string, ExpandedInput>>): { update(node: ExpandedUse, inputs: Readonly<Record<string, ExpandedInput>>): void; destroy(): void } {
    const region = createRegion(target, {
      components: this.mounts,
      onSet: (input, value) => {
        this.#events.set(input, value);
      },
      onError: (error) => {
        if ("component" in error) this.#events.error(error.component, error.reason, messageOf(error.error ?? ""));
      },
      ...(this.#options.host ? { host: this.#options.host } : {}),
    });
    region.render([node], { inputs: { ...inputs } });
    return {
      update: (next, nextInputs) => {
        region.render([next], { inputs: { ...nextInputs } });
      },
      destroy: () => {
        region.destroy();
      },
    };
  }

  /** The host's stylesheets for the document's frames, in place of `stylesheets`. */
  styles(urls: readonly string[]): void {
    this.#styles = urls;
    this.#mounts.styles(urls);
  }

  /** Gives the document's frames the page's theme tokens (spec §9.4), which their documents don't inherit. */
  theme(tokens: Readonly<Record<string, string>>): void {
    this.#mounts.theme(tokens);
  }

  /** Gives the document's frames the page's colour scheme, which their documents don't inherit either. */
  scheme(scheme: Scheme): void {
    this.#mounts.scheme(scheme);
  }

  destroy(): void {
    this.#compiler?.compiler.close();
    this.#compiler?.frame.remove();
    this.#compiler = undefined;
  }

  async #codeOf(component: HostComponent): Promise<ComponentCode | undefined> {
    const level = levelOf(component.trust);
    if (level === "trusted") return { name: component.name, level };
    const given = this.#given(component);
    if (typeof given === "string") {
      this.#events.error(component.name, "failed", given);
      return undefined;
    }
    const withLibraries = Object.keys(given.libraries).length > 0 ? { libraries: given.libraries } : {};
    if ("module" in given) {
      const { module, place } = given;
      return "code" in module ? { name: component.name, level, binding: "plain", code: module.code, place, ...withLibraries } : { name: component.name, level, module };
    }
    try {
      const { code, css } = await this.#compilerFor().compile({ name: component.name, language: component.language, code: component.code, ...(component.runtime ? { runtime: component.runtime } : {}) });
      return { name: component.name, level, binding: bindingOf(component.language), code, ...(css === undefined ? {} : { css }), ...withLibraries };
    } catch (error) {
      this.#events.error(component.name, "failed", messageOf(error));
      return undefined;
    }
  }

  /**
   * What the host gives for `component`: the modules of the library packs it imports, and a built module's file or
   * URL, with its place; why not, when one of them isn't given as a frame needs it.
   */
  #given(component: HostComponent): { libraries: Libraries } | { libraries: Libraries; module: ReturnType<ModuleSource>; place: string } | string {
    const libraries = this.#librariesOf(component.libraries);
    if (typeof libraries === "string") return libraries;
    if (!component.module) return { libraries };
    const place = placeOf(component.module);
    const module = this.#options.modules?.(component.module);
    if (!module) return `no URL for \`${place}\``;
    const names = Object.keys(libraries).map((name) => `\`${name}\``);
    if ("url" in module && names.length > 0) return `\`${place}\` loads by its URL, so its imports of library packs (${names.join(", ")}) can't be written: give it as code`;
    return { libraries, module, place };
  }

  /**
   * The library packs `libraries` names, as frames get them, each module's file from `modules`; why not, when one
   * isn't given as code, which a frame needs to write the imports of it (docs/sharing-library-packs.md, decision 8).
   */
  #librariesOf(libraries: HostComponent["libraries"]): Libraries | string {
    const found: Record<string, LibraryCode> = {};
    for (const [name, module] of Object.entries(libraries ?? {})) {
      const place = placeOf(module);
      const source = this.#options.modules?.(module);
      if (!source || !("code" in source)) return `the library pack \`${place}\` isn't given as code, which a frame needs to load it`;
      const own = this.#librariesOf(module.libraries);
      if (typeof own === "string") return own;
      found[name] = { place, code: source.code, ...(Object.keys(own).length > 0 ? { libraries: own } : {}) };
    }
    return found;
  }

  /** The document's compile frame, a hidden strict frame that runs no components, made the first time it's needed. */
  #compilerFor(): Compiler {
    if (this.#compiler) return this.#compiler.compiler;
    const frame = document.createElement("iframe");
    frame.setAttribute("sandbox", "allow-scripts");
    frame.setAttribute("allow", "");
    frame.hidden = true;
    frame.title = "compile";
    const { port1, port2 } = new MessageChannel();
    frame.addEventListener("load", () => frame.contentWindow?.postMessage({ type: HELLO, version: SANDBOX_VERSION }, "*", [port2]), { once: true });
    frame.src = frameUrl(this.#options.base, "strict.html");
    document.body.append(frame);
    const compiler = connectCompiler(port1);
    this.#compiler = { compiler, frame };
    return compiler;
  }

  #frame(target: Element, node: ExpandedUse, inputs: Readonly<Record<string, ExpandedInput>>, component: ComponentCode) {
    const level = component.level === "permissive" ? "permissive" : "strict";
    const trust = this.#trust.get(component.name);
    const permissive = trust?.level === "permissive" ? trust : undefined;
    const api = this.#options.host;
    let failed: (message: string) => void = () => undefined;
    const frame = frameUse(target, {
      url: frameUrl(this.#options.base, `${level}.html`),
      level,
      code: codeFor(this.#code, level),
      node,
      inputs,
      ...(permissive ? { network: permissive.network } : {}),
      ...(permissive?.host && api
        ? {
            host: () => {
              const { port1, port2 } = new MessageChannel();
              serveHost(port1, api);
              return port2;
            },
          }
        : {}),
      styles: this.#styles,
      ping: { every: 1000, timeout: this.#options.timeout ?? 5000 },
      on: {
        set: ({ input, value }) => {
          this.#events.set(input, value);
        },
        link: ({ url }) => {
          this.#events.link(url);
        },
        error: ({ reason, component: name, message }) => {
          this.#events.error(name ?? component.name, reason, message);
        },
        failed: (failure) => {
          failed(failure.reason === "hung" ? "its frame stopped answering" : `its frame is of protocol version ${String(failure.version)}`);
        },
      },
    });
    return {
      update: (next: ExpandedUse, nextInputs: Readonly<Record<string, ExpandedInput>>) => {
        frame.update(next, nextInputs);
      },
      destroy: () => {
        frame.destroy();
      },
      code: (next: readonly ComponentCode[]) => {
        frame.code(codeFor(next, level));
      },
      theme: (tokens: Readonly<Record<string, string>>) => {
        frame.theme(tokens);
      },
      styles: (urls: readonly string[]) => {
        frame.styles(urls);
      },
      scheme: (scheme: Scheme) => {
        frame.scheme(scheme);
      },
      failed: (listener: (message: string) => void) => {
        failed = listener;
      },
    };
  }
}
