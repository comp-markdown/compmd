// What a parent sent a frame, as the keys a frame may ask about: an image's `src`, a code block's text and language.
// Code in the frame could ask too, so the parent answers only for what it sent (docs/remote-images.md,
// docs/host-highlighter.md, decision 4).

import type { ExpandedNode, Splice } from "@compmd/render";

/**
 * The keys of the trees and splices a parent sent its frame, by `keysOf`.
 *
 * @example
 * const sources = new SentNodes((nodes) => imageSources(nodes));
 * sources.rendered(tree.children);
 * sources.has("./logo.png"); // true
 */
export class SentNodes<K> {
  readonly #keysOf: (nodes: readonly ExpandedNode[]) => Iterable<K>;
  #keys = new Set<K>();
  /** Splices as a processor's JSON text, read only once the frame asks about something they may have. */
  #unread: string[] = [];

  constructor(keysOf: (nodes: readonly ExpandedNode[]) => Iterable<K>) {
    this.#keysOf = keysOf;
  }

  /** The frame renders `nodes`, in place of all it had. */
  rendered(nodes: readonly ExpandedNode[]): void {
    this.#keys = new Set(this.#keysOf(nodes));
    this.#unread = [];
  }

  /** The frame applies `splices`, beside what it had. */
  spliced(splices: readonly (Splice | string)[]): void {
    for (const splice of splices) {
      if (typeof splice === "string") this.#unread.push(splice);
      else this.#add(splice.nodes);
    }
  }

  /** Whether `key` is in what was sent. */
  has(key: K): boolean {
    if (this.#keys.has(key)) return true;
    this.#readSplices();
    return this.#keys.has(key);
  }

  /** Whether a key of what was sent is one `matches` takes. */
  some(matches: (key: K) => boolean): boolean {
    this.#readSplices();
    return [...this.#keys].some(matches);
  }

  #readSplices(): void {
    for (const splice of this.#unread) this.#add((JSON.parse(splice) as Splice).nodes);
    this.#unread = [];
  }

  #add(nodes: readonly ExpandedNode[]): void {
    for (const key of this.#keysOf(nodes)) this.#keys.add(key);
  }
}
