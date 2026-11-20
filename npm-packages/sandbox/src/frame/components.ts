// A frame's host-language components, by name (an example's by its number too, spec §7.8), for the renderer. Each name's mount goes through one of these
// entries, so when its code changes, the instances on the page are mounted again with the new code and their last
// params and slots, and the rest of the page keeps its state. Code that doesn't load shows each instance's fallback.

import type { Mount, Mounted, MountContext, Params, Slots } from "@compmd/render";
import { MountSet } from "../mount-set.ts";
import type { Observer } from "../observer.ts";
import { entryOf, type Libraries, type Runnable } from "../protocol/messages.ts";
import { timed } from "../timed.ts";

/** A component's code, compiled and loaded: its mount, and a Svelte component's CSS. */
export interface Loaded {
  mount: Mount;
  css?: string;
}

export interface ComponentsOptions {
  load: (component: Runnable) => Promise<Loaded>;
  /** A component's code couldn't be loaded (`failed`), or threw when mounted again (`threw`). */
  report: (component: string, reason: "failed" | "threw", message: string) => void;
  /** A component's CSS, or none, in place of what it had; the component as `entryOf` names it. */
  adopt: (component: string, css: string | undefined) => void;
  /** The frame's observer, which records each instance as its code mounts, and how long that took (`timed`). */
  observer?: Observer;
}

/** One instance on the page. */
interface Instance {
  target: Element;
  /** The renderer's context, whose `scheme` the renderer keeps up to date. */
  context: MountContext;
  /** The last params and slots, which a remount gets. */
  latest: { params: Params; slots: Slots };
  mounted?: Mounted | undefined;
}

/** The context a remount gets: the last params and slots, and the renderer's scheme as it is now and later. */
function remountContext({ context, latest }: Instance): MountContext {
  return {
    ...context,
    ...latest,
    get scheme() {
      return context.scheme;
    },
  };
}

interface Entry {
  /** What the code is, so that the same code isn't loaded again. */
  key: string;
  mount?: Mount;
  failed?: string;
  instances: Set<Instance>;
}

function keyOf(component: Runnable): string {
  if ("module" in component) return `${component.module.url}\n${component.module.integrity}`;
  const libraries = librariesKey(component.libraries);
  if ("place" in component) return `${component.place}\n${libraries}`;
  return `${component.binding}\n${component.css ?? ""}\n${libraries}\n${component.code}`;
}

/** Which library module each import names, nested: what changes the code a component runs. */
function librariesKey(libraries: Libraries | undefined): string {
  return Object.entries(libraries ?? {})
    .sort(([one], [other]) => one.localeCompare(other))
    .map(([name, library]) => `${name}=${library.place}(${librariesKey(library.libraries)})`)
    .join(",");
}
const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

export class Components {
  /** The mounts by name, and each example's, for the renderer; the objects stay the same as names come and go. */
  readonly mounts = new MountSet();
  /** By where each component is kept (`entryOf`): its name, with its example's number for an example's. */
  readonly #entries = new Map<string, Entry & { component: Runnable }>();
  readonly #options: ComponentsOptions;

  constructor(options: ComponentsOptions) {
    this.#options = options;
  }

  /** The components are now `components`; resolves once each changed one is loaded, or has failed. */
  async set(components: readonly Runnable[]): Promise<void> {
    const kept = new Set(components.map(entryOf));
    for (const [at, { component }] of this.#entries) {
      if (kept.has(at)) continue;
      this.#entries.delete(at);
      this.mounts.remove(component);
      this.#options.adopt(at, undefined);
    }
    await Promise.all(components.map((component) => this.#update(component)));
  }

  async #update(component: Runnable): Promise<void> {
    const key = keyOf(component);
    const at = entryOf(component);
    let entry = this.#entries.get(at);
    if (entry?.key === key) return;
    if (!entry) {
      entry = { key, instances: new Set(), component };
      this.#entries.set(at, entry);
      this.mounts.put(component, this.#indirect(entry));
    }
    entry.key = key;
    try {
      const loaded = await this.#options.load(component);
      if (entry.key !== key) return;
      this.#options.adopt(at, loaded.css);
      const { observer } = this.#options;
      entry.mount = observer ? timed(component.name, loaded.mount, observer) : loaded.mount;
      delete entry.failed;
    } catch (error) {
      if (entry.key !== key) return;
      delete entry.mount;
      entry.failed = messageOf(error);
      this.#options.report(component.name, "failed", entry.failed);
    }
    for (const instance of entry.instances) this.#remount(component.name, entry, instance);
  }

  /**
   * The mount the renderer calls for a name: it runs the entry's code, now and whenever it changes. While its code
   * doesn't load, which was reported as `failed`, it shows its fallback, without throwing: the renderer would report
   * it again, as code that threw.
   */
  #indirect(entry: Entry): Mount {
    return (target, context) => {
      const instance: Instance = { target, context, latest: { params: context.params, slots: context.slots } };
      if (entry.failed === undefined) instance.mounted = entry.mount?.(target, context) ?? undefined;
      else target.replaceChildren(context.fallback.element);
      entry.instances.add(instance);
      return {
        update(params: Params, slots: Slots) {
          instance.latest = { params, slots };
          instance.mounted?.update?.(params, slots);
        },
        destroy() {
          instance.mounted?.destroy?.();
          entry.instances.delete(instance);
        },
      };
    };
  }

  #remount(name: string, entry: Entry, instance: Instance): void {
    instance.mounted?.destroy?.();
    instance.mounted = undefined;
    instance.target.replaceChildren();
    try {
      if (!entry.mount) throw new Error(entry.failed);
      instance.mounted = entry.mount(instance.target, remountContext(instance)) ?? undefined;
    } catch (error) {
      instance.target.replaceChildren(instance.context.fallback.element);
      if (entry.mount) this.#options.report(name, "threw", messageOf(error));
    }
  }
}
