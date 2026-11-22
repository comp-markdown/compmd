// A drawing given as markup, made into nodes a page whose `style-src` refuses inline styles still styles: the markup
// is parsed inert, its `<style>` elements give a stylesheet the page adopts, and its `style` attributes are set through
// the CSSOM, which no `style-src` refuses.

/** Markup's nodes, ready to insert, and the CSS of its `<style>` elements. */
export interface Parsed {
  nodes: DocumentFragment;
  css: string;
}

/**
 * `markup` (SVG or HTML) as nodes of `document`, without its `<style>` elements, whose CSS it gives apart, and with
 * each `style` attribute set as the element's `style` instead.
 *
 * @example
 * const { nodes, css } = parseMarkup('<svg><style>.a{fill:red}</style><rect style="width:10px"/></svg>', document);
 * // css is ".a{fill:red}"; the rect's style.width is "10px", with no style attribute ever parsed in the page.
 */
export function parseMarkup(markup: string, document: Document): Parsed {
  const inert = new DOMParser().parseFromString(markup, "text/html");
  const css: string[] = [];
  // By local name, an SVG `<style>` as well as an HTML one.
  for (const style of Array.from(inert.querySelectorAll("*")).filter((element) => element.localName === "style")) {
    css.push(style.textContent);
    style.remove();
  }
  const styled: [Element, string][] = [];
  for (const element of Array.from(inert.querySelectorAll("[style]"))) {
    styled.push([element, element.getAttribute("style") ?? ""]);
    element.removeAttribute("style");
  }
  const nodes = document.createDocumentFragment();
  for (const node of Array.from(inert.body.childNodes)) nodes.append(document.adoptNode(node));
  for (const [element, cssText] of styled) (element as HTMLElement | SVGElement).style.cssText = cssText;
  return { nodes, css: css.join("\n") };
}

/** Where a page's adopted stylesheets are: a document, or a shadow root. */
type Root = Document | ShadowRoot;

/**
 * The stylesheet a drawing brings, adopted where `target` is shown, until it is replaced or dropped. Without
 * constructed stylesheets (an older browser), the CSS goes in a `<style>` element in `fallback`, as markup has it.
 */
export class DrawingSheet {
  private adopted: { root: Root; sheet: CSSStyleSheet } | undefined;
  private readonly target: Element;

  constructor(target: Element) {
    this.target = target;
  }

  /** Adopts `css` in place of the last, or drops the last when `css` is empty; `fallback` holds it otherwise. */
  replace(css: string, fallback: Element): void {
    this.drop();
    if (css.trim() === "") return;
    const root = this.target.getRootNode();
    const where = root instanceof Document || root instanceof ShadowRoot ? root : this.target.ownerDocument;
    const Sheet = this.target.ownerDocument.defaultView?.CSSStyleSheet;
    if (Sheet === undefined || !("adoptedStyleSheets" in where)) {
      const style = this.target.ownerDocument.createElement("style");
      style.textContent = css;
      fallback.prepend(style);
      return;
    }
    const sheet = new Sheet();
    sheet.replaceSync(css);
    where.adoptedStyleSheets = [...where.adoptedStyleSheets, sheet];
    this.adopted = { root: where, sheet };
  }

  /** Drops the adopted stylesheet. */
  drop(): void {
    if (this.adopted === undefined) return;
    const { root, sheet } = this.adopted;
    root.adoptedStyleSheets = root.adoptedStyleSheets.filter((known) => known !== sheet);
    this.adopted = undefined;
  }
}
