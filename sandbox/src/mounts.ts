// The renderer's components where a part renders: a component of this level runs here, one of a lower level gets a
// frame of its own where it is used, and one of a higher level shows its fallback, since the frames only go down in
// trust (docs/sandbox.md, decisions 1 and 4, Q2).

import { rendersSlots, type ExpandedComponent, type ExpandedInput, type Mount, type NodesMounted } from "@compmd/render";
import type { ComponentCode, ErrorReason, Level } from "./protocol/messages.ts";

/** Where a part renders: in a frame of a level, or in the host's page, where trusted code runs. */
export type Here = Level | "host";

const RANK: Record<ComponentCode["level"] | Here, number> = { strict: 0, permissive: 1, trusted: 2, host: 2 };

export interface MountsOptions {
  here: Here;
  /** The mounts of components of this level: their loaded code, or the host's trusted components. */
  inline: Readonly<Record<string, Mount>>;
  /** Frames a use of a lower level in `target`. */
  frame(target: Element, node: ExpandedComponent, inputs: Readonly<Record<string, ExpandedInput>>, component: ComponentCode): NodesMounted & { failed(listener: (message: string) => void): void };
  /** A component shows its fallback: it can't run here (`not-here`), or its frame stopped answering (`threw`). */
  report(component: string, reason: ErrorReason, message: string): void;
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

  constructor(options: MountsOptions) {
    this.#options = options;
  }

  set(components: readonly ComponentCode[]): void {
    for (const name of Object.keys(this.mounts)) Reflect.deleteProperty(this.mounts, name);
    for (const component of components) {
      const mount = this.#mountOf(component);
      if (mount) this.mounts[component.name] = mount;
    }
  }

  #mountOf(component: ComponentCode): Mount | undefined {
    const { here, inline } = this.#options;
    const rank = RANK[component.level];
    if (rank === RANK[here]) return inline[component.name];
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
      frame.failed((message) => {
        frame.destroy?.();
        target.replaceChildren(fallback.element);
        this.#options.report(component.name, "threw", message);
      });
      return frame;
    });
  }
}
