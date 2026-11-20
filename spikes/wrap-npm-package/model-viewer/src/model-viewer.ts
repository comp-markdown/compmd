// `model-viewer`: the `<model-viewer>` custom element of @google/model-viewer in the plain binding (spec §7.6), as the
// command writes it once. Importing the package defines the element; three.js and lit are bundled in.
import "@google/model-viewer";
import { component } from "@compmd/pack/component";

/** The params, as the generated signature in `index.comp.md` declares them. */
export interface Params {
  src: string;
  alt?: string;
  poster?: string;
  cameraControls: boolean;
  autoRotate: boolean;
}

/** The element's attribute for each param: its property in kebab case, as it has no `custom-elements.json`. */
const ATTRIBUTES: Record<keyof Params, string> = { src: "src", alt: "alt", poster: "poster", cameraControls: "camera-controls", autoRotate: "auto-rotate" };

export default component<Params>((target, { params, fallback, fail }) => {
  const viewer = document.createElement("model-viewer");
  // The element reports a model it couldn't load as an event, not a throw: without this, an empty box and no W118.
  viewer.addEventListener("error", (event: Event) => {
    const { type, sourceError } = (event as CustomEvent<{ type?: string; sourceError?: unknown }>).detail;
    fail(`model-viewer couldn't load ${viewer.getAttribute("src") ?? ""}:${type ?? "error"}${sourceError instanceof Error ? `, ${sourceError.message}` : ""}`);
  });
  viewer.style.cssText = "display: block; width: 100%; height: 24rem";
  const draw = (next: Params) => {
    for (const [param, attribute] of Object.entries(ATTRIBUTES) as [keyof Params, string][]) {
      const value = param === "alt" ? (next.alt ?? fallback.text) : next[param];
      if (value === undefined || value === false) viewer.removeAttribute(attribute);
      else viewer.setAttribute(attribute, value === true ? "" : value);
    }
  };
  draw(params);
  target.append(viewer);
  return { update: draw, destroy: () => viewer.remove() };
});
