// `__component__`: draws with __from__, which `npm create @compmd/pack -- --from __from__` installed, and the build
// bundles in. A library has no props to read, so the drawing and its params are yours to write
// (docs/writing-a-pack.md, decision 6): call the library below, and declare its params in `index.comp.md`.
import * as library from "__from__";
import { draw } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  value: string;
}

export default draw<Params>(
  ({ value }) => {
    // Draw `value` with the library here, and give back an element or SVG markup: `draw` puts it in the box, fits an
    // SVG, an `img` or a `canvas` in it, and shows the fallback when this throws or its promise rejects.
    const drawing = document.createElement("p");
    drawing.textContent = `${value}, to draw with __from__ (${Object.keys(library).join(", ")})`;
    return drawing;
  },
  { role: "img" },
);
