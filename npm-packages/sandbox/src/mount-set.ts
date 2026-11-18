// Mounts as a renderer takes them: the document's by name, and each example's by its number (spec §7.8), since an
// example's `card` is not the document's. The objects stay the same as names come and go: a page reads them at each
// mount.

import type { Mount } from "@compmd/render";
import type { Named } from "./protocol/messages.ts";

type Place = Pick<Named, "name" | "example">;

export class MountSet {
  /** The document's mounts, by name: a renderer's `components`. */
  readonly mounts: Record<string, Mount> = {};
  /** Each example's mounts, by its number, then by name: a renderer's `examples`. */
  readonly examples: Record<number, Record<string, Mount>> = {};

  /** The document's `mounts`, by name, and no example's: a host's own trusted components. */
  static of(mounts: Readonly<Record<string, Mount>>): MountSet {
    const set = new MountSet();
    Object.assign(set.mounts, mounts);
    return set;
  }

  /** The mounts of `component`'s document or example. */
  #of({ example }: Place): Record<string, Mount> {
    if (example === undefined) return this.mounts;
    return (this.examples[example] ??= {});
  }

  get(component: Place): Mount | undefined {
    return this.#of(component)[component.name];
  }

  put(component: Place, mount: Mount): void {
    this.#of(component)[component.name] = mount;
  }

  remove(component: Place): void {
    Reflect.deleteProperty(this.#of(component), component.name);
  }

  /** Every mount gone; the objects stay. */
  clear(): void {
    for (const name of Object.keys(this.mounts)) Reflect.deleteProperty(this.mounts, name);
    for (const mounts of Object.values(this.examples)) for (const name of Object.keys(mounts)) Reflect.deleteProperty(mounts, name);
  }
}
