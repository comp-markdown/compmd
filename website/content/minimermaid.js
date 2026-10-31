// The landing page's code renderer: Mermaid's simplest flowcharts, drawn by hand with d3, as in the playground's tour.
import * as d3 from "d3";

// `graph LR` or `graph TD`, then chains of steps joined by `-->`, each `id` or `id[label]`.
function parse(code) {
  const [head = "", ...statements] = code.split(/[;\n]/).map((line) => line.trim()).filter(Boolean);
  const labels = new Map();
  const edges = [];
  for (const statement of statements) {
    const steps = statement.split("-->").map((step) => /^([\w-]+)(?:\[(.*)\])?$/.exec(step.trim())).filter(Boolean);
    for (const [, id, label] of steps) if (label || !labels.has(id)) labels.set(id, label ?? id);
    for (let i = 1; i < steps.length; i++) edges.push([steps[i - 1][1], steps[i][1]]);
  }
  return { across: /^(graph|flowchart)\s+LR/.test(head), labels, edges };
}

// Each step's distance from the start: the longest chain of steps before it.
function ranks(labels, edges) {
  const rank = new Map([...labels.keys()].map((id) => [id, 0]));
  for (let pass = 0; pass < labels.size; pass++) {
    for (const [from, to] of edges) rank.set(to, Math.max(rank.get(to), rank.get(from) + 1));
  }
  return rank;
}

// A source of numbers in [0, 1) seeded by `text`, so a step's wobble is the same at every render.
function wobbly(text) {
  let h = 2166136261;
  for (const c of text) h = Math.imul(h ^ c.codePointAt(0), 16777619);
  return (amount) => {
    h = Math.imul(h ^ (h >>> 15), 2246822507);
    h = Math.imul(h ^ (h >>> 13), 3266489909);
    return ((((h ^= h >>> 16) >>> 0) / 4294967296) - 0.5) * amount;
  };
}

// A line drawn by hand: past its corners at both ends, a little off, and bowed in the middle.
function stroke(jitter, x1, y1, x2, y2) {
  const length = Math.hypot(x2 - x1, y2 - y1);
  const [dx, dy] = [(x2 - x1) / length, (y2 - y1) / length];
  const [before, after] = [4 + jitter(6), 4 + jitter(6)];
  const bow = 2 + length / 18;
  const start = [x1 - dx * before + jitter(4), y1 - dy * before + jitter(4)];
  const end = [x2 + dx * after + jitter(4), y2 + dy * after + jitter(4)];
  return `M${start} Q${(x1 + x2) / 2 + jitter(bow)},${(y1 + y2) / 2 + jitter(bow)} ${end}`;
}

// A box drawn by hand: each side twice, as a pen goes over it.
function box(jitter, width, height) {
  const corners = [[0, 0], [width, 0], [width, height], [0, height]];
  return [1, 2].flatMap(() => corners.map((corner, i) => stroke(jitter, ...corner, ...corners[(i + 1) % 4]))).join(" ");
}

// An arrow from (x1, y1) to (x2, y2), leaving and arriving along the flow: a curve, and a head of two strokes.
function arrow(jitter, across, x1, y1, x2, y2) {
  const reach = (across ? x2 - x1 : y2 - y1) / 2;
  const c1 = across ? [x1 + reach, y1 + jitter(14)] : [x1 + jitter(14), y1 + reach];
  const c2 = across ? [x2 - reach, y2 + jitter(14)] : [x2 + jitter(14), y2 - reach];
  const angle = Math.atan2(y2 - c2[1], x2 - c2[0]);
  const barb = (turn) => `M${x2},${y2} L${x2 - 11 * Math.cos(angle + turn) + jitter(2)},${y2 - 11 * Math.sin(angle + turn) + jitter(2)}`;
  return { line: `M${x1},${y1} C${c1} ${c2} ${x2},${y2}`, head: `${barb(0.5)} ${barb(-0.5)}` };
}

const THEMES = {
  default: { fill: "color-mix(in srgb, var(--viz-series-1) 16%, transparent)", stroke: "var(--viz-series-1)", text: "currentColor" },
  dark: { fill: "#2f3540", stroke: "#8d96a7", text: "#fff" },
  neutral: { fill: "none", stroke: "currentColor", text: "currentColor" },
};

const HAND = '"Bradley Hand", "Segoe Print", "Comic Sans MS", cursive';

export default function mount(target, { params, fallback }) {
  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const svg = d3.select(target).append("svg").attr("role", "img").attr("aria-label", fallback.text).style("max-width", "100%").style("overflow", "visible");
  let drawn = false;

  function draw({ code, theme }) {
    svg.selectAll("*").remove();
    const { across, labels, edges } = parse(code);
    const rank = ranks(labels, edges);
    const colors = THEMES[theme];
    const height = 42;
    // Each box as wide as its label; each column as wide as its widest box, the steps centred in it.
    const widths = new Map([...labels].map(([id, label]) => [id, label.length * 8.5 + 36]));
    const widest = Math.max(0, ...widths.values());
    const columns = [];
    for (const [id, r] of rank) columns[r] = Math.max(columns[r] ?? 0, widths.get(id));
    const left = (r) => columns.slice(0, r).reduce((sum, column) => sum + (column ?? 0) + 56, 0);
    const filled = new Map();
    const at = new Map();
    for (const id of labels.keys()) {
      const r = rank.get(id);
      const n = filled.get(r) ?? 0;
      const width = widths.get(id);
      filled.set(r, n + 1);
      at.set(id, across ? { x: left(r) + (columns[r] - width) / 2, y: n * (height + 24), width } : { x: n * (widest + 24) + (widest - width) / 2, y: r * (height + 56), width });
    }
    const size = [...at.values()].reduce((max, { x, y, width }) => [Math.max(max[0], x + width), Math.max(max[1], y + height)], [0, 0]);
    svg.attr("viewBox", [-10, -10, size[0] + 20, size[1] + 20]).attr("width", size[0] + 20);
    const ink = svg.append("g").attr("fill", "none").attr("stroke", "currentColor").attr("stroke-width", 1.6).attr("stroke-linecap", "round").attr("stroke-opacity", 0.7);
    for (const [from, to] of edges) {
      const a = at.get(from);
      const b = at.get(to);
      const ends = across ? [a.x + a.width + 4, a.y + height / 2, b.x - 6, b.y + height / 2] : [a.x + a.width / 2, a.y + height + 4, b.x + b.width / 2, b.y - 6];
      const { line, head } = arrow(wobbly(from + to), across, ...ends);
      const path = ink.append("path").attr("d", line);
      const tip = ink.append("path").attr("d", head);
      if (!drawn && !still) {
        const length = path.node().getTotalLength();
        path.attr("stroke-dasharray", length).attr("stroke-dashoffset", length).transition().delay(rank.get(from) * 260).duration(500).ease(d3.easeCubicOut).attr("stroke-dashoffset", 0);
        tip.attr("opacity", 0).transition().delay(rank.get(from) * 260 + 450).duration(150).attr("opacity", 1);
      }
    }
    for (const [id, { x, y, width }] of at) {
      const jitter = wobbly(id);
      const step = svg.append("g").attr("transform", `translate(${x},${y}) rotate(${jitter(3)} ${width / 2} ${height / 2})`);
      // The fill a little off the outline, as a marker leaves it.
      step.append("rect").attr("x", 3 + jitter(6)).attr("y", 3 + jitter(6)).attr("width", width - 6).attr("height", height - 6).attr("rx", 10).attr("fill", colors.fill);
      step.append("path").attr("d", box(jitter, width, height)).attr("fill", "none").attr("stroke", colors.stroke).attr("stroke-width", 1.5).attr("stroke-linecap", "round");
      step.append("text").text(labels.get(id)).attr("x", width / 2).attr("y", height / 2).attr("text-anchor", "middle").attr("dominant-baseline", "central").attr("font-family", HAND).attr("font-size", 17).attr("fill", colors.text);
    }
    drawn = true;
  }

  draw(params);
  return {
    update: draw,
    destroy: () => svg.remove(),
  };
}
