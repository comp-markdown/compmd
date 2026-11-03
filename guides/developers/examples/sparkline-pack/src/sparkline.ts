// `sparkline`: a line through a list of numbers, drawn with `draw`, which follows the page's colour scheme, lets the
// newest params win, and fails with the message of what `render` throws.
import { draw } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. An `array` may hold anything a document gives. */
export interface Params {
  values: unknown[];
}

const WIDTH = 120;
const HEIGHT = 32;

export default draw<Params>(
  ({ values }, { scheme }) => {
    const numbers = values.filter((value): value is number => typeof value === "number" && Number.isFinite(value));
    if (numbers.length < 2 || numbers.length !== values.length) throw new Error("a sparkline needs two or more numbers, and nothing else");
    const low = Math.min(...numbers);
    const span = Math.max(...numbers) - low || 1;
    const points = numbers.map((n, i) => `${String((i * WIDTH) / (numbers.length - 1))},${String(HEIGHT - ((n - low) * HEIGHT) / span)}`);
    // Only numbers go into the markup, so a document can't write any of its own into it.
    const stroke = scheme === "dark" ? "#7ee2b8" : "#0b7a52";
    return `<svg viewBox="-2 -2 ${String(WIDTH + 4)} ${String(HEIGHT + 4)}" width="${String(WIDTH)}" height="${String(HEIGHT)}"><polyline fill="none" stroke="${stroke}" stroke-width="2" points="${points.join(" ")}"/></svg>`;
  },
  { role: "img" },
);
