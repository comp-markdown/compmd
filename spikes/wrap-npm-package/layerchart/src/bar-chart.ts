// `bar-chart`: layerchart's `BarChart`, a Svelte 5 component, in the plain binding (spec §7.6), as the command writes it
// once. Svelte stays an import, which the frames' import map gives; layerchart, compiled from its `.svelte` sources
// with that Svelte, and d3 are bundled in.
import { mount, unmount, type ComponentProps } from "svelte";
import { BarChart } from "layerchart";
import { component } from "@compmd/pack/component";

/** `BarChart`'s props from the params: its props type has an index signature, which no named type meets. */
const propsOf = (params: Params) => params as ComponentProps<typeof BarChart>;

/** The params, as the generated signature in `index.comp.md` declares them. */
export interface Params {
  data?: Record<string, unknown>[];
  x?: string;
  y?: string;
  orientation?: "horizontal" | "vertical";
  bandPadding?: number;
}

/** The chart's height when the use's box has none (`height=12rem` gives it one, spec §9.2). */
const DEFAULT_HEIGHT = "18rem";

export default component<Params>((target, { params }) => {
  // layerchart takes its size from its container: the chart fills a box of its own, as tall as the use's box, which
  // is the renderer's to style.
  const box = target.appendChild(document.createElement("div"));
  box.style.height = target.clientHeight > 0 ? "100%" : DEFAULT_HEIGHT;
  // A plain module can't hold Svelte's `$state`, so a new param mounts the chart again.
  let chart = mount(BarChart, { target: box, props: propsOf(params) });
  return {
    update: (next) => {
      void unmount(chart);
      chart = mount(BarChart, { target: box, props: propsOf(next) });
    },
    destroy: () => void unmount(chart),
  };
});
