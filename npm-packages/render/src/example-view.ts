// An example (spec §7.8): a ```compmd block whose text the processor rendered as a document of its own. Its box holds
// its code first with `render=both`, folded on screen, then what the example renders, as a page renders a document: its components
// are the example's own (by its number), and a change of its inputs is the example's.
import { Blocks } from "./blocks.ts";
import { applyBox } from "./box.ts";
import { FoldedCode } from "./folded-code.ts";
import { FontFaces } from "./fonts.ts";
import { setLanguage } from "./language.ts";
import { CodeBlockView } from "./markdown-views.ts";
import { shapeOf, type Pass, type Surface, type View } from "./render.ts";
import { textOf, type ExpandedCode, type ExpandedNode, type ExpandedRoot, type MarkdownNode } from "./tree.ts";

/** The surface an example's content renders on: its own components and inputs, the rest the page's. */
function exampleSurface(surface: Surface, example: number): Surface {
  return {
    ...surface,
    components: surface.examples[example] ?? {},
    examples: {},
    onSet: (input, value) => {
      surface.onSetExample(example, input, value);
    },
  };
}

/** An example's block: its box, its code when it shows both, and what it renders. */
export class ExampleView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly dom: HTMLElement;
  readonly #surface: Surface;
  readonly #code: View | undefined;
  /** The bar and the tag around the code, with `render=both`. */
  readonly #folded: FoldedCode | undefined;
  readonly #content: HTMLElement;
  readonly #blocks: Blocks;
  /** The example's own fonts, which its root lists. */
  readonly #fonts: FontFaces;

  constructor(surface: Surface, node: MarkdownNode, pass: Pass, key: string | undefined) {
    const example = node as ExpandedCode;
    this.#surface = surface;
    this.shape = shapeOf(node);
    this.key = key;
    this.dom = surface.document.createElement("div");
    this.dom.classList.add("compmd-component", "compmd-example");
    if (example.render === "both") {
      this.dom.classList.add("compmd-both");
      this.#code = new CodeBlockView(surface, { type: "code", lang: example.lang, meta: example.meta, value: example.value }, undefined);
      this.dom.appendChild(this.#code.dom);
      this.#folded = new FoldedCode(this.dom, this.#code.dom as HTMLElement);
    }
    this.#content = this.dom.appendChild(surface.document.createElement("div"));
    this.#content.classList.add("compmd-example-document");
    this.#blocks = new Blocks(this.#content, exampleSurface(surface, example.document?.example ?? 0));
    this.#fonts = new FontFaces(this.#content, surface.packFile, surface.fonts);
    this.update(node, pass);
  }

  update(node: ExpandedNode, pass: Pass): void {
    const example = node as ExpandedCode;
    applyBox(this.dom, example.params ?? [], pass.parent, this.#surface.stylesheet);
    this.#code?.update({ type: "code", lang: example.lang, meta: example.meta, value: example.value }, pass);
    this.#folded?.show(textOf(example.lang), textOf(example.value));
    const root: ExpandedRoot = example.document ?? { type: "root", children: [] };
    setLanguage(this.#content, root);
    this.#surface.busy.add(this.#fonts.set(root.fonts ?? []));
    this.#blocks.apply({ from: 0, to: this.#blocks.length, nodes: root.children, lines: 0, offset: 0, ...(root.inputs ? { inputs: root.inputs } : {}) }, undefined);
    this.#blocks.render([], undefined);
  }

  destroy(): void {
    this.#code?.destroy();
    this.#blocks.destroy();
    this.#fonts.remove();
    this.#surface.stylesheet.forget(this.dom);
    this.dom.remove();
  }
}
