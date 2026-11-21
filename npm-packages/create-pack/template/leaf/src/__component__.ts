// `__component__`: draws its `value` in its box. Replace the drawing with yours; what it imports is bundled in.
import { component, fitBox } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  value: string;
}

const SVG = "http://www.w3.org/2000/svg";

/** An SVG element with `attributes`. */
function svg(name: string, attributes: Record<string, string>): SVGElement {
  const element = document.createElementNS(SVG, name);
  for (const [key, value] of Object.entries(attributes)) element.setAttribute(key, value);
  return element;
}

export default component<Params>((target, context) => {
  // A drawing with an aspect ratio of its own, 3 to 1. Its size is its box's, which the document sizes (`width`,
  // `height`, `aspect-ratio`): the component has no size params. `index.comp.md` gives the box its default width.
  const drawing = svg("svg", { viewBox: "0 0 120 40", role: "img" });
  // The fallback is what a reader without the component sees: its text is what you draw, as text.
  drawing.setAttribute("aria-label", context.fallback.text);
  const text = svg("text", { x: "60", y: "20", "text-anchor": "middle", "dominant-baseline": "central", "font-family": "system-ui, sans-serif", "font-size": "16", "font-weight": "600", fill: "currentColor" });
  drawing.append(svg("rect", { x: "1", y: "1", width: "118", height: "38", rx: "19", fill: "none", stroke: "currentColor", "stroke-width": "2" }), text);
  target.append(drawing);
  const draw = ({ value }: Params) => {
    text.textContent = value;
    // How the drawing sits in a box of another shape (`object-fit`, `object-position`); a change comes as an update.
    fitBox(drawing, context.fit);
  };
  draw(context.params);
  return {
    update: draw,
    destroy: () => {
      drawing.remove();
    },
  };
});
