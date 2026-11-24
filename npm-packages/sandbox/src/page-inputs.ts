// Which inputs the page's frame may set, in frame mode (docs/sandbox-protocol.md, decision 7): an input of the
// document, by its own key, that a use of the trees and splices sent binds, as a use frame's `set` is checked against
// its use. Strict code in the page may have named any.

import type { ExpandedInput, ExpandedRoot, Splice } from "@compmd/render";
import { boundInputs } from "./protocol/inputs.ts";
import { SentNodes } from "./sent-nodes.ts";

/**
 * The document's inputs and those its uses bind, from what was sent the page's frame.
 *
 * @example
 * const inputs = new PageInputs();
 * inputs.rendered(tree);
 * inputs.binds("region"); // true when an input `region` is declared and a use binds it
 */
export class PageInputs {
  /** The document's inputs, from the last tree or splice that gave them. */
  #declared: Readonly<Record<string, ExpandedInput>> = {};
  readonly #bound = new SentNodes(boundInputs);

  /** The page's frame renders `tree`, in place of all it had. */
  rendered(tree: ExpandedRoot): void {
    this.#declared = tree.inputs ?? {};
    this.#bound.rendered(tree.children);
  }

  /** The page's frame applies `splices`, beside what it had; `inputs`, when given, are the document's now. */
  spliced(splices: readonly (Splice | string)[], inputs?: Readonly<Record<string, ExpandedInput>>): void {
    for (const splice of splices) if (typeof splice !== "string" && splice.inputs) this.#declared = splice.inputs;
    if (inputs) this.#declared = inputs;
    this.#bound.spliced(splices);
  }

  /** Whether the page may set `input`: the document declares it, as its own key, and a use sent binds it. */
  binds(input: string): boolean {
    return Object.hasOwn(this.#declared, input) && this.#bound.has(input);
  }
}
