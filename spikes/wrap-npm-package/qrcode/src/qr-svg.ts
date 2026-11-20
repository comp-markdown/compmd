// `qr-svg`: a QR code drawn by the library qrcode as SVG markup. The command writes a `draw()` that imports the library
// (docs/writing-a-pack.md, decision 6); the author wrote the call and the params, since a library has no props.
import { toString } from "qrcode";
import { draw } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them, written by the author. */
export interface Params {
  value: string;
  level: "L" | "M" | "Q" | "H";
  margin: number;
}

export default draw<Params>(({ value, level, margin }) => toString(value, { type: "svg", errorCorrectionLevel: level, margin }), { role: "img" });
