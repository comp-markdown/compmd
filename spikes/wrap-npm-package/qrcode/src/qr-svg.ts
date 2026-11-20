// `qr-svg`: a QR code drawn by the library qrcode as SVG markup. The command writes a `draw()` that imports the library
// (docs/writing-a-pack.md, decision 6); the author wrote the call and the params, since a library has no props.
//
// Plain `component`, not `draw()`: `draw()` puts the drawing in a `div` of its own with no height, so a drawing can't
// take its box's height through it (finding 12 in the README). Once the helper's `div` fills its box, this is `draw()`.
import { toString } from "qrcode";
import { component } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them, written by the author. */
export interface Params {
  value: string;
  level: "L" | "M" | "Q" | "H";
  margin: number;
}

/**
 * The drawing fits its box (docs/writing-a-pack.md, decision 8): as large as the box's width and height allow, its
 * aspect ratio kept (SVG's default `meet`) and centred; with no height, the box is as tall as the drawing.
 */
const FIT = "display: block; width: 100%; height: 100%";

export default component<Params>((target, { params, fallback, fail, busy }) => {
  const draw = ({ value, level, margin }: Params) => {
    const drawn = toString(value, { type: "svg", errorCorrectionLevel: level, margin }).then((svg) => {
      target.innerHTML = svg;
      const drawing = target.querySelector("svg");
      drawing?.setAttribute("style", FIT);
      drawing?.setAttribute("role", "img");
      drawing?.setAttribute("aria-label", fallback.text);
    });
    busy(drawn.catch((error: unknown) => fail(error instanceof Error ? error.message : String(error))));
  };
  draw(params);
  return { update: draw, destroy: () => target.replaceChildren() };
});
