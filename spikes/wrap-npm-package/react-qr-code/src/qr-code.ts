// `qr-code`: react-qr-code's `QRCode` in the plain binding (spec §7.6), as the command writes it once. React and React
// DOM stay imports, which the frames' import map gives; react-qr-code and what it uses are bundled in.
import { createElement } from "react";
import { createRoot } from "react-dom/client";
import QRCode from "react-qr-code";
import { component } from "@compmd/pack/component";

/** The params, as the generated signature in `index.comp.md` declares them. */
export interface Params {
  value: string;
  bgColor?: string;
  fgColor?: string;
  level?: "L" | "M" | "H" | "Q";
  title?: string;
}

/**
 * The drawing fits its box (docs/writing-a-pack.md, decision 8): as large as the box's width and height allow, its
 * aspect ratio kept (SVG's default `meet`) and centred; with no height, the box is as tall as the drawing. `size` is
 * left out.
 */
const FILL = { display: "block", width: "100%", height: "100%" };

export default component<Params>((target, { params, fallback }) => {
  const root = createRoot(target);
  // `title` is the label (`compmd.wrap.json`): the use's fallback text when the document gives no `title`.
  const draw = (next: Params) => {
    root.render(createElement(QRCode, { ...next, title: next.title ?? fallback.text, style: FILL }));
  };
  draw(params);
  return { update: draw, destroy: () => root.unmount() };
});
