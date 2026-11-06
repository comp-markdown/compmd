// The views of Markdown nodes (CommonMark and GFM, with alerts and math; spec §2.3): text, elements, code blocks that
// show as code, images whose files the caller gives, and code or math the caller's renderer draws.
import type { ImageSource } from "./images.ts";
import { safeUrl } from "./html.ts";
import { destroyAll, patch, pendingTag, shapeOf, type Pass, type Surface, type View } from "./render.ts";
import { textOf, type ExpandedNode, type MarkdownNode } from "./tree.ts";

/** A node made by the renderer: an element with a tag, attributes, and text or children. */
interface ElementNode extends MarkdownNode {
  type: "compmdElement";
  tag: string;
  attributes?: Record<string, string>;
  text?: string;
}

const element = (tag: string, content: Omit<ElementNode, "type" | "tag"> = {}): ElementNode => ({ type: "compmdElement", tag, ...content });

export function textView(surface: Surface, node: MarkdownNode, key: string | undefined): View {
  const dom = surface.document.createTextNode(textOf(node.value));
  return {
    shape: "text",
    key,
    dom,
    update: (next) => {
      const value = textOf((next as MarkdownNode).value);
      if (dom.data !== value) dom.data = value;
    },
    destroy: () => undefined,
  };
}

/** How a Markdown node renders: its element, and either its text or the nodes inside it. */
interface Rendering {
  tag: string;
  attributes: Record<string, string>;
  text?: string;
  children?: readonly ExpandedNode[];
}

const ALERT_TITLES: Record<string, string> = { note: "Note", tip: "Tip", important: "Important", warning: "Warning", caution: "Caution" };

function linkOrText(url: string, title: unknown, children: readonly ExpandedNode[]): Rendering {
  const href = safeUrl(url);
  if (href === undefined) return { tag: "span", attributes: {}, children };
  return { tag: "a", attributes: { href, ...(typeof title === "string" && title ? { title } : {}) }, children };
}

function image(url: string, alt: unknown, title: unknown): Rendering {
  const src = safeUrl(url);
  return { tag: "img", attributes: { ...(src === undefined ? {} : { src }), alt: textOf(alt), ...(typeof title === "string" && title ? { title } : {}) } };
}

function table(node: MarkdownNode): Rendering {
  const align = (node.align as (string | null)[] | undefined) ?? [];
  const rows = node.children ?? [];
  const row = (tr: ExpandedNode, cell: string) =>
    element("tr", {
      children: (tr.children ?? []).map((td, i) => element(cell, { ...(align[i] ? { attributes: { class: `compmd-align-${align[i] ?? ""}` } } : {}), children: td.children ?? [] })),
    });
  const [head, ...body] = rows;
  return { tag: "table", attributes: {}, children: [element("thead", { children: head ? [row(head, "th")] : [] }), element("tbody", { children: body.map((tr) => row(tr, "td")) })] };
}

/** The rendering of a Markdown node (CommonMark and GFM, with alerts and math; spec §2.3). */
function renderingOf(surface: Surface, node: MarkdownNode, pass: Pass): Rendering {
  const children = node.children ?? [];
  const plain = (tag: string, attributes: Record<string, string> = {}): Rendering => ({ tag, attributes, children });
  switch (node.type) {
    case "paragraph":
      return plain("p");
    case "heading":
      return plain(`h${textOf(node.depth)}`);
    case "emphasis":
      return plain("em");
    case "strong":
      return plain("strong");
    case "delete":
      return plain("del");
    case "inlineCode":
      return { tag: "code", attributes: {}, text: textOf(node.value) };
    case "break":
      return { tag: "br", attributes: {} };
    case "thematicBreak":
      return { tag: "hr", attributes: {} };
    case "blockquote": {
      const type = typeof node.alert === "string" ? node.alert : undefined;
      if (type === undefined) return plain("blockquote", { class: "compmd-quote" });
      const title = element("p", { attributes: { class: "compmd-alert-title" }, text: ALERT_TITLES[type] ?? type });
      return { tag: "div", attributes: { class: `compmd-alert compmd-alert-${type}`, role: "note" }, children: [title, ...children] };
    }
    case "list": {
      const start = typeof node.start === "number" && node.start !== 1 && node.ordered === true ? { start: String(node.start) } : {};
      // A tight list's items hold their paragraphs without margins, as HTML renders them without `<p>` (CommonMark §5.3).
      const tight = node.spread === false ? { class: "compmd-tight" } : {};
      return plain(node.ordered === true ? "ol" : "ul", { ...start, ...tight });
    }
    case "listItem": {
      if (typeof node.checked !== "boolean") return plain("li");
      const box = element("input", { attributes: { type: "checkbox", disabled: "", ...(node.checked ? { checked: "" } : {}) } });
      return { tag: "li", attributes: { class: "compmd-task" }, children: [box, ...children] };
    }
    case "link":
      return linkOrText(textOf(node.url), node.title, children);
    case "image":
      return image(textOf(node.url), node.alt, node.title);
    case "linkReference": {
      const definition = pass.definitions.get(textOf(node.identifier));
      return definition ? linkOrText(definition.url, definition.title, children) : plain("span");
    }
    case "imageReference": {
      const definition = pass.definitions.get(textOf(node.identifier));
      return definition ? image(definition.url, node.alt, definition.title) : { tag: "span", attributes: {}, text: textOf(node.alt) };
    }
    case "table":
      return table(node);
    case "math":
      return { tag: "pre", attributes: {}, children: [element("code", { attributes: { class: "language-math math-display" }, text: textOf(node.value) })] };
    case "inlineMath":
      return { tag: "code", attributes: { class: "language-math math-inline" }, text: textOf(node.value) };
    case "footnoteReference": {
      const number = String(pass.footnotes.get(textOf(node.identifier)) ?? "?");
      return { tag: "sup", attributes: {}, children: [element("a", { attributes: { href: `#compmd-fn-${number}`, id: `compmd-fnref-${number}` }, text: number })] };
    }
    case "wikiLink": {
      const label = textOf(node.label ?? node.target);
      const href = surface.wikiLink?.(textOf(node.target));
      const safe = href === undefined ? undefined : safeUrl(href);
      return safe === undefined ? { tag: "span", attributes: {}, text: label } : { tag: "a", attributes: { href: safe }, text: label };
    }
    case "compmdElement":
    case "compmdHtml": {
      const own = node as unknown as ElementNode;
      return { tag: own.tag, attributes: own.attributes ?? {}, ...(own.text === undefined ? { children: own.children ?? [] } : { text: own.text }) };
    }
    case "componentPending":
      // A use of a component whose pack is on its way (spec §4.5): its fallback, busy, until the pack arrives.
      return { tag: pendingTag(children), attributes: { class: "compmd-pending", "aria-busy": "true" }, children };
    case "definition":
    case "footnoteDefinition":
      // A definition shows where it is used; the page lists footnotes at the end.
      return { tag: "template", attributes: {} };
    default:
      return plain("span");
  }
}

export class ElementView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly dom: HTMLElement;
  readonly #surface: Surface;
  #attributes: Record<string, string> = {};
  #children: View[] = [];

  constructor(surface: Surface, node: MarkdownNode, pass: Pass, key: string | undefined) {
    this.#surface = surface;
    this.shape = shapeOf(node);
    this.key = key;
    this.dom = surface.document.createElement(renderingOf(surface, node, pass).tag);
    this.update(node, pass);
  }

  update(node: ExpandedNode, pass: Pass): void {
    const rendering = renderingOf(this.#surface, node as MarkdownNode, pass);
    for (const name of Object.keys(this.#attributes)) if (!(name in rendering.attributes)) this.dom.removeAttribute(name);
    for (const [name, value] of Object.entries(rendering.attributes)) if (this.dom.getAttribute(name) !== value) this.dom.setAttribute(name, value);
    this.#attributes = rendering.attributes;
    if (node.type === "heading") pass.heading.level = Number(node.depth);
    if (rendering.text !== undefined) {
      if (this.dom.textContent !== rendering.text) this.dom.textContent = rendering.text;
    } else if (rendering.children) {
      this.#children = patch(this.#surface, this.dom, this.#children, rendering.children, { ...pass, parent: "block" });
    }
  }

  destroy(): void {
    destroyAll(this.#children);
  }
}

/**
 * A code block that shows as code, `<pre><code class="language-…">`: one no code renderer draws, or a code renderer's
 * fallback. The caller's highlighter, if any, highlights it, again only when its text or language changes.
 */
export class CodeBlockView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly dom: HTMLElement;
  readonly #surface: Surface;
  readonly #code: HTMLElement;
  /** The language and text shown, as one string. */
  #shown: string | undefined;

  constructor(surface: Surface, node: MarkdownNode, key: string | undefined) {
    this.#surface = surface;
    this.shape = shapeOf(node);
    this.key = key;
    this.dom = surface.document.createElement("pre");
    this.#code = this.dom.appendChild(surface.document.createElement("code"));
    this.update(node);
  }

  update(next: ExpandedNode): void {
    const language = textOf((next as MarkdownNode).lang);
    const code = textOf((next as MarkdownNode).value);
    const shown = `${language}\u0000${code}`;
    if (shown === this.#shown) return;
    this.#shown = shown;
    if (language) this.#code.className = `language-${language}`;
    else this.#code.removeAttribute("class");
    this.#code.textContent = code;
    this.#surface.highlight?.(code, language, this.#code);
  }

  destroy(): void {
    return undefined;
  }
}

/** Whether `node` is an image: in Markdown, by reference, or in raw HTML. */
export function isImage(node: MarkdownNode): boolean {
  return node.type === "image" || node.type === "imageReference" || (node.type === "compmdHtml" && node.tag === "img");
}

/**
 * An image whose file the caller gives (`SurfaceOptions.images`): its alt text shows until the file comes, and stays
 * when it can't be had.
 */
export class ImageView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly #surface: Surface;
  readonly #images: ImageSource;
  #dom: HTMLElement;
  #src: string | undefined;
  #alt = "";
  #title: string | undefined;
  #destroyed = false;

  constructor(surface: Surface, images: ImageSource, node: MarkdownNode, pass: Pass, key: string | undefined) {
    this.#surface = surface;
    this.#images = images;
    this.shape = shapeOf(node);
    this.key = key;
    this.#dom = this.#altText();
    this.update(node, pass);
  }

  get dom(): Node {
    return this.#dom;
  }

  update(node: ExpandedNode, pass: Pass): void {
    const { attributes, text } = renderingOf(this.#surface, node as MarkdownNode, pass);
    this.#alt = attributes.alt ?? text ?? "";
    this.#title = attributes.title;
    if (attributes.src !== this.#src) {
      this.#src = attributes.src;
      this.#show(this.#altText());
      if (attributes.src !== undefined) this.#load(attributes.src);
    }
    this.#describe();
  }

  destroy(): void {
    this.#destroyed = true;
  }

  #altText(): HTMLElement {
    const span = this.#surface.document.createElement("span");
    span.className = "compmd-image-alt";
    return span;
  }

  #load(src: string): void {
    this.#images(src).then(
      (shown) => {
        if (this.#destroyed || this.#src !== src || shown === undefined) return;
        const image = this.#surface.document.createElement("img");
        image.src = shown;
        this.#show(image);
        this.#describe();
      },
      () => undefined,
    );
  }

  #show(element: HTMLElement): void {
    this.#dom.replaceWith(element);
    this.#dom = element;
  }

  #describe(): void {
    const dom = this.#dom;
    if (dom.tagName === "IMG") dom.setAttribute("alt", this.#alt);
    else if (dom.textContent !== this.#alt) dom.textContent = this.#alt;
    if (this.#title) dom.title = this.#title;
    else dom.removeAttribute("title");
  }
}

/** A code block or math that the caller's renderer draws; if it throws, the source shows instead. */
export class CallbackView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly dom: HTMLElement;
  readonly #surface: Surface;
  #source: string | undefined;

  constructor(surface: Surface, node: MarkdownNode, key: string | undefined) {
    this.#surface = surface;
    this.shape = shapeOf(node);
    this.key = key;
    const inline = node.type === "inlineMath";
    this.dom = surface.document.createElement(inline ? "span" : "div");
    this.dom.className = node.type === "code" ? `compmd-code language-${textOf(node.lang)}` : `compmd-math ${inline ? "math-inline" : "math-display"}`;
    this.update(node);
  }

  update(next: ExpandedNode): void {
    const node = next as MarkdownNode;
    const source = textOf(node.value);
    if (source === this.#source) return;
    this.#source = source;
    this.dom.replaceChildren();
    try {
      if (node.type === "code") this.#surface.code[textOf(node.lang)]?.(source, this.dom);
      else this.#surface.math?.(source, { display: node.type === "math" }, this.dom);
    } catch {
      this.dom.replaceChildren(this.#surface.document.createTextNode(source));
    }
  }

  destroy(): void {
    return undefined;
  }
}
