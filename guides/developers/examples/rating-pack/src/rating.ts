import { component } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them: `color` is optional, so it may be missing. */
export interface Params {
  value: number;
  max: number;
  size: "small" | "large";
  color?: string;
}

export default component<Params>((target, { params, fallback, set }) => {
  const stars = document.createElement("span");
  stars.setAttribute("role", "group");
  stars.setAttribute("aria-label", fallback.text);
  target.append(stars);
  const draw = ({ value, max, size, color }: Params) => {
    stars.style.fontSize = size === "large" ? "1.6em" : "1em";
    stars.style.color = color ?? "inherit";
    stars.replaceChildren(
      ...Array.from({ length: max }, (_, i) => {
        const star = document.createElement("button");
        star.type = "button";
        star.textContent = i < value ? "★" : "☆";
        star.setAttribute("aria-label", `${String(i + 1)} of ${String(max)}`);
        // The reader's choice goes to the input `value` is bound to; the host renders again with it.
        star.addEventListener("click", () => {
          set("value", i + 1);
        });
        return star;
      }),
    );
  };
  draw(params);
  return {
    update: draw,
    destroy: () => {
      stars.remove();
    },
  };
});
