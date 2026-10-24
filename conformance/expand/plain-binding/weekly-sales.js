export default function mount(target, { params, fallback }) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("role", "img");
  svg.setAttribute("aria-label", fallback.text);
  target.append(svg);
  const draw = (rows) => { svg.textContent = rows.map((r) => r.total).join(" "); };
  draw(params.rows);
  return { update: (next) => draw(next.rows), destroy: () => svg.remove() };
}
