// A frame's host-language components, by name, for the renderer. Each name's mount goes through one of these
// entries, so when its code changes, the instances on the page are mounted again with the new code and their last
// params and slots, and the rest of the page keeps its state. Code that doesn't load shows each instance's fallback.

import type { Mount, Mounted, MountContext, Params, Slots } from "@compmd/render";
import type { Runnable } from "../protocol/messages.ts";

/** A component's code, compiled and loaded: its mount, and a Svelte component's CSS. */
export interface Loaded {
  mount: Mount;
  css?: string;
}

export interface ComponentsOptions {
  load: (component: Runnable) => Promise<Loaded>;
  /** A component couldn't be loaded or mounted again. */
  report: (component: string, message: string) => void;
  /** A component's CSS, or none, in place of what it had. */
  adopt: (component: string, css: string | undefined) => void;
}

/** One instance on the page. */
interface Instance {
  target: Element;
  context: MountContext;
  mounted?: Mounted | undefined;
}

interface Entry {
  /** What the code is, so that the same code isn't loaded again. */
  key: string;
  mount?: Mount;
  failed?: string;
  instances: Set<Instance>;
}

const keyOf = (component: Runnable): string =>
  "module" in component ? `${component.module.url}\n${component.module.integrity}` : `${component.binding}\n${component.css ?? ""}\n${component.code}`;
const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

export class Components {
  /** The mounts by name, for the renderer; the object stays the same as names come and go. */
  readonly mounts: Record<string, Mount> = {};
  readonly #entries = new Map<string, Entry>();
  readonly #options: ComponentsOptions;

  constructor(options: ComponentsOptions) {
    this.#options = options;
  }

  /** The components are now `components`; resolves once each changed one is loaded, or has failed. */
  async set(components: readonly Runnable[]): Promise<void> {
    const names = new Set(components.map((component) => component.name));
    for (const name of this.#entries.keys()) {
      if (names.has(name)) continue;
      this.#entries.delete(name);
      Reflect.deleteProperty(this.mounts, name);
      this.#options.adopt(name, undefined);
    }
    await Promise.all(components.map((component) => this.#update(component)));
  }

  async #update(component: Runnable): Promise<void> {
    const key = keyOf(component);
    let entry = this.#entries.get(component.name);
    if (entry?.key === key) return;
    if (!entry) {
      entry = { key, instances: new Set() };
      this.#entries.set(component.name, entry);
      this.mounts[component.name] = this.#indirect(entry);
    }
    entry.key = key;
    try {
      const loaded = await this.#options.load(component);
      if (entry.key !== key) return;
      this.#options.adopt(component.name, loaded.css);
      entry.mount = loaded.mount;
      delete entry.failed;
    } catch (error) {
      if (entry.key !== key) return;
      delete entry.mount;
      entry.failed = messageOf(error);
      this.#options.report(component.name, entry.failed);
    }
    for (const instance of entry.instances) this.#remount(component.name, entry, instance);
  }

  /** The mount the renderer calls for a name: it runs the entry's code, now and whenever it changes. */
  #indirect(entry: Entry): Mount {
    return (target, context) => {
      if (entry.failed !== undefined) throw new Error(entry.failed);
      const instance: Instance = { target, context, mounted: entry.mount?.(target, context) ?? undefined };
      entry.instances.add(instance);
      return {
        update(params: Params, slots: Slots) {
          instance.context = { ...instance.context, params, slots };
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
      instance.mounted = entry.mount(instance.target, instance.context) ?? undefined;
    } catch (error) {
      instance.target.replaceChildren(instance.context.fallback.element);
      if (entry.mount) this.#options.report(name, messageOf(error));
    }
  }
}
