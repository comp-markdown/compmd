// `minimermaid`: Mermaid's simplest flowcharts, drawn by hand in SVG, with no library: `graph LR` or `graph TD`, then
// chains of steps joined by `-->`, each `id` or `id[label]`.

/** What the host gives a component in the plain binding (CompMD spec §7.4). */
export interface Context<Params> {
  params: Params;
  /** The block's `alt`, or a leaf's fallback: the drawing's text alternative. */
  fallback: { text: string; element: Element };
}

export interface Params {
  /** The code block's text. */
  code: string;
  theme: "default" | "dark" | "neutral";
}

interface Chart {
  across: boolean;
  labels: Map<string, string>;
  edges: [string, string][];
}

const SVG = "http://www.w3.org/2000/svg";
const HAND = '"Bradley Hand", "Segoe Print", "Comic Sans MS", cursive';
const HEIGHT = 42;

const THEMES: Record<Params["theme"], { fill: string; stroke: string; text: string }> = {
  default: { fill: "color-mix(in srgb, var(--viz-series-1, #2563eb) 16%, transparent)", stroke: "var(--viz-series-1, #2563eb)", text: "currentColor" },
  dark: { fill: "#2f3540", stroke: "#8d96a7", text: "#fff" },
  neutral: { fill: "none", stroke: "currentColor", text: "currentColor" },
};

/** The chart `code` describes; what it doesn't understand, it leaves out. */
export function parse(code: string): Chart {
  const [head = "", ...statements] = code
    .split(/[;\n]/)
    .map((line) => line.trim())
    .filter(Boolean);
  const labels = new Map<string, string>();
  const edges: [string, string][] = [];
  for (const statement of statements) {
    const steps = statement
      .split("-->")
      .map((step) => /^([\w-]+)(?:\[(.*)\])?$/.exec(step.trim()))
      .filter((step) => step !== null);
    for (const [, id = "", label] of steps) if (label !== undefined || !labels.has(id)) labels.set(id, label ?? id);
    for (let i = 1; i < steps.length; i++) edges.push([steps[i - 1]?.[1] ?? "", steps[i]?.[1] ?? ""]);
  }
  return { across: /^(graph|flowchart)\s+LR/.test(head), labels, edges };
}

/** Each step's distance from the start: the longest chain of steps before it. */
function ranks({ labels, edges }: Chart): Map<string, number> {
  const rank = new Map([...labels.keys()].map((id) => [id, 0]));
  for (let pass = 0; pass < labels.size; pass++) {
    for (const [from, to] of edges) rank.set(to, Math.max(rank.get(to) ?? 0, (rank.get(from) ?? 0) + 1));
  }
  return rank;
}

/** A source of offsets seeded by `text`, so a step's wobble is the same at every render: `jitter(8)` is within ±4. */
function wobbly(text: string): (amount: number) => number {
  let h = 2166136261;
  for (const c of text) h = Math.imul(h ^ (c.codePointAt(0) ?? 0), 16777619);
  return (amount) => {
    h = Math.imul(h ^ (h >>> 15), 2246822507);
    h = Math.imul(h ^ (h >>> 13), 3266489909);
    h ^= h >>> 16;
    return ((h >>> 0) / 4294967296 - 0.5) * amount;
  };
}

/** A line drawn by hand: past its corners at both ends, a little off, and bowed in the middle. */
function stroke(jitter: (amount: number) => number, x1: number, y1: number, x2: number, y2: number): string {
  const length = Math.hypot(x2 - x1, y2 - y1);
  const [dx, dy] = [(x2 - x1) / length, (y2 - y1) / length];
  const [before, after] = [4 + jitter(6), 4 + jitter(6)];
  const bow = 2 + length / 18;
  const start = `${String(x1 - dx * before + jitter(4))},${String(y1 - dy * before + jitter(4))}`;
  const end = `${String(x2 + dx * after + jitter(4))},${String(y2 + dy * after + jitter(4))}`;
  return `M${start} Q${String((x1 + x2) / 2 + jitter(bow))},${String((y1 + y2) / 2 + jitter(bow))} ${end}`;
}

/** A box drawn by hand: each side twice, as a pen goes over it. */
function box(jitter: (amount: number) => number, width: number, height: number): string {
  const corners: [number, number][] = [
    [0, 0],
    [width, 0],
    [width, height],
    [0, height],
  ];
  const side = (i: number) => {
    const [x1, y1] = corners[i] ?? [0, 0];
    const [x2, y2] = corners[(i + 1) % 4] ?? [0, 0];
    return stroke(jitter, x1, y1, x2, y2);
  };
  return [0, 1, 2, 3, 0, 1, 2, 3].map(side).join(" ");
}

/** An arrow from (x1, y1) to (x2, y2), leaving and arriving along the flow: a curve, and a head of two strokes. */
function arrow(jitter: (amount: number) => number, across: boolean, x1: number, y1: number, x2: number, y2: number): { line: string; head: string } {
  const reach = (across ? x2 - x1 : y2 - y1) / 2;
  const c1 = across ? [x1 + reach, y1 + jitter(14)] : [x1 + jitter(14), y1 + reach];
  const c2 = across ? [x2 - reach, y2 + jitter(14)] : [x2 + jitter(14), y2 - reach];
  const angle = Math.atan2(y2 - (c2[1] ?? 0), x2 - (c2[0] ?? 0));
  const barb = (turn: number) => `M${String(x2)},${String(y2)} L${String(x2 - 11 * Math.cos(angle + turn) + jitter(2))},${String(y2 - 11 * Math.sin(angle + turn) + jitter(2))}`;
  return { line: `M${String(x1)},${String(y1)} C${c1.join(",")} ${c2.join(",")} ${String(x2)},${String(y2)}`, head: `${barb(0.5)} ${barb(-0.5)}` };
}

/** An SVG element named `name`, with `attributes`, in `parent`. */
function add(parent: Element, name: string, attributes: Record<string, string | number>): SVGElement {
  const element = document.createElementNS(SVG, name);
  for (const [key, value] of Object.entries(attributes)) element.setAttribute(key, String(value));
  parent.append(element);
  return element;
}

export default function mount(target: HTMLElement, { params, fallback }: Context<Params>): { update(next: Params): void; destroy(): void } {
  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const svg = add(target, "svg", { role: "img", "aria-label": fallback.text });
  svg.style.maxWidth = "100%";
  svg.style.overflow = "visible";
  let drawn = false;

  const draw = ({ code, theme }: Params) => {
    svg.replaceChildren();
    const chart = parse(code);
    const rank = ranks(chart);
    const colors = THEMES[theme];
    // Each box as wide as its label; each column as wide as its widest box, the steps centred in it.
    const widths = new Map([...chart.labels].map(([id, label]) => [id, label.length * 8.5 + 36]));
    const widest = Math.max(0, ...widths.values());
    const columns: number[] = [];
    for (const [id, r] of rank) columns[r] = Math.max(columns[r] ?? 0, widths.get(id) ?? 0);
    const left = (r: number) => columns.slice(0, r).reduce((sum, column) => sum + column + 56, 0);
    const filled = new Map<number, number>();
    const at = new Map<string, { x: number; y: number; width: number }>();
    for (const id of chart.labels.keys()) {
      const r = rank.get(id) ?? 0;
      const n = filled.get(r) ?? 0;
      const width = widths.get(id) ?? 0;
      filled.set(r, n + 1);
      at.set(id, chart.across ? { x: left(r) + ((columns[r] ?? 0) - width) / 2, y: n * (HEIGHT + 24), width } : { x: n * (widest + 24) + (widest - width) / 2, y: r * (HEIGHT + 56), width });
    }
    const size = [...at.values()].reduce<[number, number]>(([w, h], { x, y, width }) => [Math.max(w, x + width), Math.max(h, y + HEIGHT)], [0, 0]);
    svg.setAttribute("viewBox", `-10 -10 ${String(size[0] + 20)} ${String(size[1] + 20)}`);
    svg.setAttribute("width", String(size[0] + 20));
    const ink = add(svg, "g", { fill: "none", stroke: "currentColor", "stroke-width": 1.6, "stroke-linecap": "round", "stroke-opacity": 0.7 });
    for (const [from, to] of chart.edges) {
      const a = at.get(from);
      const b = at.get(to);
      if (!a || !b) continue;
      const ends: [number, number, number, number] = chart.across ? [a.x + a.width + 4, a.y + HEIGHT / 2, b.x - 6, b.y + HEIGHT / 2] : [a.x + a.width / 2, a.y + HEIGHT + 4, b.x + b.width / 2, b.y - 6];
      const { line, head } = arrow(wobbly(from + to), chart.across, ...ends);
      const path = add(ink, "path", { d: line }) as SVGPathElement;
      const tip = add(ink, "path", { d: head });
      if (!drawn && !still) {
        // The arrows draw themselves in, one step after another, the first time.
        const length = path.getTotalLength();
        const delay = (rank.get(from) ?? 0) * 260;
        path.animate([{ strokeDasharray: length, strokeDashoffset: length }, { strokeDasharray: length, strokeDashoffset: 0 }], { duration: 500, delay, easing: "cubic-bezier(0.33, 1, 0.68, 1)", fill: "backwards" });
        tip.animate([{ opacity: 0 }, { opacity: 1 }], { duration: 150, delay: delay + 450, fill: "backwards" });
      }
    }
    for (const [id, { x, y, width }] of at) {
      const jitter = wobbly(id);
      const step = add(svg, "g", { transform: `translate(${String(x)},${String(y)}) rotate(${String(jitter(3))} ${String(width / 2)} ${String(HEIGHT / 2)})` });
      // The fill a little off the outline, as a marker leaves it.
      add(step, "rect", { x: 3 + jitter(6), y: 3 + jitter(6), width: width - 6, height: HEIGHT - 6, rx: 10, fill: colors.fill });
      add(step, "path", { d: box(jitter, width, HEIGHT), fill: "none", stroke: colors.stroke, "stroke-width": 1.5, "stroke-linecap": "round" });
      const text = add(step, "text", { x: width / 2, y: HEIGHT / 2, "text-anchor": "middle", "dominant-baseline": "central", "font-family": HAND, "font-size": 17, fill: colors.text });
      text.textContent = chart.labels.get(id) ?? id;
    }
    drawn = true;
  };

  draw(params);
  return {
    update: draw,
    destroy: () => {
      svg.remove();
    },
  };
}
