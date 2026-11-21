// The colour scheme components are shown in (spec §7.4): read from the element a page or region renders into, and
// given to every component in it when it changes.
import type { Scheme } from "./contract.ts";

/**
 * The colour scheme `element` is shown in: its `color-scheme`, and the reader's preference when it allows both. An
 * element whose page sets none is light, as browsers show it.
 *
 * @example
 * schemeOf(document.body); // "dark" on a page with `color-scheme: dark`
 */
export function schemeOf(element: Element): Scheme {
  const view = element.ownerDocument.defaultView;
  const words = (view?.getComputedStyle(element).colorScheme ?? "").split(/\s+/);
  if (!words.includes("dark")) return "light";
  if (!words.includes("light")) return "dark";
  return view?.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

/** What follows the scheme: a component's handle, or anything else that takes it; a handle also its box's type. */
export interface SchemeFollower {
  scheme(scheme: Scheme): void;
  restyle?(): void;
}

/**
 * The scheme of what a page or region renders into, and who follows it. While something follows it, it reads the
 * scheme again on `read()` (at each render, and when the host switches its own theme), when the reader's preference
 * changes, and when printing starts and ends, since print styles may change it.
 *
 * Reading it asks the browser for the element's style, which makes it style the whole document first if anything in
 * it changed. So nothing is read, and nothing listened to, for a page with no component to show in a scheme.
 */
export class Schemes {
  /** The scheme as last read; `undefined` when nothing has asked since the last follower left. */
  #current: Scheme | undefined;
  readonly #root: Element;
  readonly #followers = new Set<SchemeFollower>();
  #stop: (() => void) | undefined;

  constructor(root: Element) {
    this.#root = root;
  }

  /** The scheme now: read when nothing has it yet. */
  get current(): Scheme {
    return (this.#current ??= schemeOf(this.#root));
  }

  /** Reads the scheme again; when it changed, every follower gets it. With no follower, the next `current` reads it. */
  read(): void {
    if (this.#followers.size === 0) {
      this.#current = undefined;
      return;
    }
    const next = schemeOf(this.#root);
    if (next === this.#current) return;
    this.#current = next;
    for (const follower of this.#followers) follower.scheme(next);
  }

  /** Something around them changed their type (a frame's root, a theme): every follower reads its box's again. */
  restyle(): void {
    for (const follower of this.#followers) follower.restyle?.();
  }

  follow(follower: SchemeFollower): void {
    this.#followers.add(follower);
    this.#stop ??= this.#listen();
  }

  unfollow(follower: SchemeFollower): void {
    this.#followers.delete(follower);
    if (this.#followers.size === 0) this.#unlisten();
  }

  destroy(): void {
    this.#unlisten();
    this.#followers.clear();
  }

  /** Reads again at what changes the scheme without a render; returns what stops it. */
  #listen(): () => void {
    const view = this.#root.ownerDocument.defaultView;
    const preference = view?.matchMedia("(prefers-color-scheme: dark)");
    const read = () => {
      this.read();
    };
    preference?.addEventListener("change", read);
    view?.addEventListener("beforeprint", read);
    view?.addEventListener("afterprint", read);
    return () => {
      preference?.removeEventListener("change", read);
      view?.removeEventListener("beforeprint", read);
      view?.removeEventListener("afterprint", read);
    };
  }

  #unlisten(): void {
    this.#stop?.();
    this.#stop = undefined;
  }
}
