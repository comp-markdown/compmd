import { component } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  value: number;
  max: number;
}

export default component<Params>((target, { params, fallback }) => {
  const bar = document.createElement("progress");
  // A screen reader hears the fallback, which says in words what the bar shows.
  bar.setAttribute("aria-label", fallback.text);
  const numbers = document.createElement("span");
  target.append(bar, " ", numbers);
  const draw = ({ value, max }: Params) => {
    bar.max = max;
    bar.value = Math.min(Math.max(value, 0), max);
    numbers.textContent = `${String(value)} of ${String(max)}`;
  };
  draw(params);
  return {
    update: draw,
    destroy: () => {
      bar.remove();
      numbers.remove();
    },
  };
});
