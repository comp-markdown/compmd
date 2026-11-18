// A built module in the plain binding (spec §6.4, §7.6): one bar per value, as SVG.
export default function mount(target, { params, fallback }) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", fallback.text);
  target.append(svg);
  const draw = ({ values }) => {
    const max = Math.max(1, ...values);
    svg.setAttribute("viewBox", `0 0 ${String(values.length * 24)} 100`);
    svg.setAttribute("width", String(values.length * 24));
    svg.replaceChildren(
      ...values.map((value, i) => {
        const bar = document.createElementNS("http://www.w3.org/2000/svg", "rect");
        const height = (value / max) * 100;
        bar.setAttribute("x", String(i * 24 + 2));
        bar.setAttribute("y", String(100 - height));
        bar.setAttribute("width", "20");
        bar.setAttribute("height", String(height));
        bar.setAttribute("fill", "currentColor");
        return bar;
      }),
    );
  };
  draw(params);
  return { update: draw, destroy: () => svg.remove() };
}
