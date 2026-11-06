// A sparkline, inline: a week's temperatures as a small line as tall as the text around it, coloured along its length
// by temperature on one scale for every city, cool blue to warm red, a dot on the last day. Drawn with d3.
import * as d3 from "d3";

const WIDTH = 84;
const HEIGHT = 20;
const PAD = 3;

// One scale for every line, so a cold week and a hot one look it side by side: from -5 °C, blue, to 35 °C, red.
const colourOf = d3.scaleSequential(d3.interpolateRdYlBu).domain([35, -5]);

// Each sparkline's gradient needs an id of its own in the frame's document.
let drawn = 0;

export default function mount(target, { params, fallback }) {
  const id = `spark-${String(drawn++)}`;
  const svg = d3
    .select(target)
    .append("svg")
    .attr("viewBox", `0 0 ${WIDTH} ${HEIGHT}`)
    .attr("width", WIDTH)
    .attr("height", HEIGHT)
    .attr("role", "img")
    .attr("aria-label", fallback.text)
    .style("vertical-align", "middle")
    .style("overflow", "visible");

  function draw({ values = [] }) {
    svg.selectAll("*").remove();
    if (values.length < 2) return;
    const x = d3.scaleLinear().domain([0, values.length - 1]).range([PAD, WIDTH - PAD]);
    const y = d3.scaleLinear().domain(d3.extent(values)).range([HEIGHT - PAD, PAD]);
    // The line's colour at each day, from its temperature.
    svg
      .append("defs")
      .append("linearGradient")
      .attr("id", id)
      .attr("gradientUnits", "userSpaceOnUse")
      .attr("x1", PAD)
      .attr("x2", WIDTH - PAD)
      .selectAll("stop")
      .data(values)
      .join("stop")
      .attr("offset", (_value, i) => i / (values.length - 1))
      .attr("stop-color", (value) => colourOf(value));
    const line = d3
      .line()
      .x((_value, i) => x(i))
      .y((value) => y(value))
      .curve(d3.curveMonotoneX);
    svg.append("path").attr("d", line(values)).attr("fill", "none").attr("stroke", `url(#${id})`).attr("stroke-width", 2.2).attr("stroke-linecap", "round");
    const last = values.length - 1;
    svg.append("circle").attr("cx", x(last)).attr("cy", y(values[last])).attr("r", 2.6).attr("fill", colourOf(values[last]));
  }

  draw(params);
  return {
    update: draw,
    destroy: () => svg.remove(),
  };
}
