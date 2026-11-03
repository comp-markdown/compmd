// The renderer's components where a part renders: a component of this level runs here, one of a lower level gets a
// frame of its own where it is used, and one of a higher level shows its fallback, since the frames only go down in
// trust (docs/running-components-in-the-sandbox.md, decisions 1 and 4, Q2).

import { rendersSlots, type ExpandedInput, type ExpandedUse, type Mount, type NodesMounted, type Params, type Slots } from "@compmd/render";
import type { Observer } from "./observer.ts";
import type { ComponentCode, ErrorReason, Level, Scheme } from "./protocol/messages.ts";

/** Where a part renders: in a frame of a level, or in the host's page, where trusted code runs. */
export type Here = Level | "host";

const RANK: Record<ComponentCode["level"] | Here, number> = { strict: 0, permissive: 1, trusted: 2, host: 2 };

/** A use in its frame, as the renderer updates it, with the document's code when it changes. */
export interface Framed extends NodesMounted {
  code(components: readonly ComponentCode[]): void;
  /** The page's theme tokens (spec §9.4), which a frame's document doesn't inherit. */
  theme(tokens: Readonly<Record<string, string>>): void;
  /** The host's stylesheets, in place of those the frame had. */
  styles(urls: readonly string[]): void;
  /** The page's colour scheme, which a frame's document doesn't inherit either. */
  scheme(scheme: Scheme): void;
  /** Calls `listener` if the frame stops answering, or is of another protocol version. */
  failed(listener: (message: string) => void): void;
}

export interface MountsOptions {
  here: Here;
  /** The mounts of components of this level: their loaded code, or the host's trusted components. */
  inline: Readonly<Record<string, Mount>>;
  /** Frames a use of a lower level in `target`. */
  frame(target: Element, node: ExpandedUse, inputs: Readonly<Record<string, ExpandedInput>>, component: ComponentCode): Framed;
  /** A component shows its fallback: it can't run here (`not-here`), or its frame stopped answering (`threw`). */
  report(component: string, reason: ErrorReason, message: string): void;
  /** The window's observer, which records each component mounted here, with how long it took to mount and update. */
  observer?: Observer;
}

/**
 * `mount` with each mount and update timed by `observer`. A mount that carries more than a function (`rendersSlots`
 * marks one) is left as it is: the renderer reads what it carries.
 */
function timed(name: string, mount: Mount, observer: Observer): Mount {
  if (Object.getOwnPropertySymbols(mount).length > 0) return mount;
  return (target, context) => {
    const started = performance.now();
    const mounted = mount(target, context);
    const use = observer.mounted(name, performance.now() - started);
    return {
      ...(mounted?.update
        ? {
            update: (params: Params, slots: Slots) => {
              const updating = performance.now();
              mounted.update?.(params, slots);
              observer.updated(use, performance.now() - updating);
            },
          }
        : {}),
      destroy: () => {
        observer.unmounted(use);
        mounted?.destroy?.();
      },
    };
  };
}

/**
 * What a frame of `level` gets of `components`: the code of those it runs or frames below it, and only the name and
 * level of the rest, so it never holds code it can't run.
 */
export function codeFor(components: readonly ComponentCode[], level: Level): ComponentCode[] {
  return components.map((component) => (RANK[component.level] <= RANK[level] ? component : { name: component.name, level: component.level }));
}

/** Why a component of `level` can't be in a part rendered `here`. */
function notHereMessage(name: string, level: ComponentCode["level"], here: Here): string {
  const where = here === "host" ? "the host's page" : `a ${here} frame`;
  const fix = level === "trusted" ? "only the host's page runs trusted code" : "only the host's page holds permissive frames";
  return `\`${name}\` is ${level} and can't be in ${where}: ${fix}`;
}

/**
 * The components for a renderer in a part rendered `options.here`, as one object that stays the same as `set` changes
 * them, since a page reads it at each mount.
 *
 * @example
 * const mounts = new Mounts({ here: "permissive", inline: loaded.mounts, frame, report });
 * mounts.set(components);
 * createRegion(container, { components: mounts.mounts });
 */
export class Mounts {
  readonly mounts: Record<string, Mount> = {};
  readonly #options: MountsOptions;
  /** The frames on the page, which get the document's code and theme again when they change. */
  readonly #frames = new Set<Framed>();
  #tokens: Readonly<Record<string, string>> = {};
  #scheme: Scheme | undefined;

  constructor(options: MountsOptions) {
    this.#options = options;
  }

  set(components: readonly ComponentCode[]): void {
    for (const name of Object.keys(this.mounts)) Reflect.deleteProperty(this.mounts, name);
    for (const component of components) {
      const mount = this.#mountOf(component);
      if (mount) this.mounts[component.name] = mount;
    }
    for (const frame of this.#frames) frame.code(components);
  }

  /** Gives every frame, and those made later, the page's theme tokens. */
  theme(tokens: Readonly<Record<string, string>>): void {
    this.#tokens = tokens;
    for (const frame of this.#frames) frame.theme(tokens);
  }

  /** Gives every frame, and those made later, the page's colour scheme. */
  scheme(scheme: Scheme): void {
    this.#scheme = scheme;
    for (const frame of this.#frames) frame.scheme(scheme);
  }

  /** Gives every frame on the page the host's stylesheets; those made later get them from `frame`. */
  styles(urls: readonly string[]): void {
    for (const frame of this.#frames) frame.styles(urls);
  }

  #mountOf(component: ComponentCode): Mount | undefined {
    const { here, inline, observer } = this.#options;
    const rank = RANK[component.level];
    if (rank === RANK[here]) {
      const mount = inline[component.name];
      return mount && observer ? timed(component.name, mount, observer) : mount;
    }
    if (rank > RANK[here]) return this.#notHere(component);
    return this.#framed(component);
  }

  #notHere({ name, level }: ComponentCode): Mount {
    const message = notHereMessage(name, level, this.#options.here);
    return (target, { fallback }) => {
      target.replaceChildren(fallback.element);
      this.#options.report(name, "not-here", message);
    };
  }

  #framed(component: ComponentCode): Mount {
    return rendersSlots((target, { node, inputs, fallback }) => {
      const frame = this.#options.frame(target, node, inputs, component);
      this.#frames.add(frame);
      if (Object.keys(this.#tokens).length > 0) frame.theme(this.#tokens);
      if (this.#scheme) frame.scheme(this.#scheme);
      frame.failed((message) => {
        this.#frames.delete(frame);
        frame.destroy?.();
        target.replaceChildren(fallback.element);
        this.#options.report(component.name, "threw", message);
      });
      return {
        update: (next, nextInputs) => frame.update?.(next, nextInputs),
        destroy: () => {
          this.#frames.delete(frame);
          frame.destroy?.();
        },
      };
    });
  }
}
