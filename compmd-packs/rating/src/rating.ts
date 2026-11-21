// `rating`: `value` of `of` stars, half stars included, read-only, drawn as one SVG that fits its box. The filled stars
// are clipped to the value's width over the empty ones, so 3.5 shows three and a half. Its text alternative is the
// use's fallback, "Rated 4 of 5" when it has none; it prints as drawn.
import { component, fitBox } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  value: number;
  of: number;
}

const SVG = "http://www.w3.org/2000/svg";
/** A star, 24 units across. */
const STAR = "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01z";
const SIZE = 24;
/** The most stars a rating draws: a larger `of` is drawn as this many. */
export const MOST = 20;

/** An SVG element with `attributes`. */
function svg(name: string, attributes: Record<string, string>): SVGElement {
  const element = document.createElementNS(SVG, name);
  for (const [key, value] of Object.entries(attributes)) element.setAttribute(key, value);
  return element;
}

/** `of`, a whole number of stars from 1 to `MOST`, and `value`, between 0 and it, to the nearest half. */
export function starsOf({ value, of }: Params): { value: number; of: number } {
  const stars = Math.min(MOST, Math.max(1, Math.round(Number.isFinite(of) ? of : 5)));
  const shown = Math.round(Math.min(stars, Math.max(0, Number.isFinite(value) ? value : 0)) * 2) / 2;
  return { value: shown, of: stars };
}

/** The number as a reader says it: `4`, `3.5`. */
const said = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(1));

/** Each mount's own clip id, as several ratings share a document. */
let mounted = 0;

export default component<Params>((target, context) => {
  const clip = `compmd-rating-${String((mounted += 1))}`;
  const drawing = svg("svg", { role: "img" });
  const shape = svg("clipPath", { id: clip });
  const width = svg("rect", { x: "0", y: "0", height: String(SIZE) });
  shape.append(width);
  const empty = svg("g", { fill: "currentColor", "fill-opacity": "0.25" });
  const filled = svg("g", { fill: "var(--compmd-accent, #e0a100)", "clip-path": `url(#${clip})` });
  drawing.append(svg("defs", {}), empty, filled);
  drawing.firstElementChild?.append(shape);
  target.append(drawing);
  const draw = (params: Params) => {
    const { value, of } = starsOf(params);
    drawing.setAttribute("viewBox", `0 0 ${String(of * SIZE)} ${String(SIZE)}`);
    drawing.setAttribute("aria-label", context.fallback.text || `Rated ${said(value)} of ${String(of)}`);
    width.setAttribute("width", String(value * SIZE));
    const stars = Array.from({ length: of }, (_, i) => svg("path", { d: STAR, transform: `translate(${String(i * SIZE)} 0)` }));
    empty.replaceChildren(...stars.map((star) => star.cloneNode()));
    filled.replaceChildren(...stars);
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
