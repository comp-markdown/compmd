// `hello`: draws its `value` in its box. Replace the drawing with yours; what it imports is bundled in.
import { component } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  value: string;
}

export default component<Params>((target, { params, fallback }) => {
  const shown = document.createElement("p");
  // The fallback is what a reader without the component sees: its text is what you draw, as text.
  shown.setAttribute("aria-label", fallback.text);
  shown.style.cssText = "display: inline-block; margin: 0; padding: 0.4em 0.9em; border: 2px solid currentColor; border-radius: 999px; font-weight: 600;";
  target.append(shown);
  const draw = ({ value }: Params) => {
    shown.textContent = value;
  };
  draw(params);
  return {
    update: draw,
    destroy: () => {
      shown.remove();
    },
  };
});
