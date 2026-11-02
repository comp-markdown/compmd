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

/** What follows the scheme: a component's handle, or anything else that takes it. */
export interface SchemeFollower {
  scheme(scheme: Scheme): void;
}

/**
 * The scheme of what a page or region renders into, and who follows it. It reads the scheme again on `read()` (at
 * each render, and when the host switches its own theme), when the reader's preference changes, and when printing
 * starts and ends, since print styles may change it.
 */
export class Schemes {
  #current: Scheme;
  readonly #root: Element;
  readonly #followers = new Set<SchemeFollower>();
  readonly #stop: () => void;

  constructor(root: Element) {
    this.#root = root;
    this.#current = schemeOf(root);
    const view = root.ownerDocument.defaultView;
    const preference = view?.matchMedia("(prefers-color-scheme: dark)");
    const read = () => {
      this.read();
    };
    preference?.addEventListener("change", read);
    view?.addEventListener("beforeprint", read);
    view?.addEventListener("afterprint", read);
    this.#stop = () => {
      preference?.removeEventListener("change", read);
      view?.removeEventListener("beforeprint", read);
      view?.removeEventListener("afterprint", read);
    };
  }

  get current(): Scheme {
    return this.#current;
  }

  /** Reads the scheme again; when it changed, every follower gets it. */
  read(): void {
    const next = schemeOf(this.#root);
    if (next === this.#current) return;
    this.#current = next;
    for (const follower of this.#followers) follower.scheme(next);
  }

  follow(follower: SchemeFollower): void {
    this.#followers.add(follower);
  }

  unfollow(follower: SchemeFollower): void {
    this.#followers.delete(follower);
  }

  destroy(): void {
    this.#stop();
    this.#followers.clear();
  }
}
