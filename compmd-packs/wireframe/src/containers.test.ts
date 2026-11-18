import { describe, expect, it } from "vitest";
import { cell, col, footer, grid, header, panel, row, section, window } from "./containers.ts";
import { parse } from "./parse.ts";
import { childOf, drawing, partOf, texts } from "./testing.ts";

/** The window of `source`. */
function windowOf(source: string) {
  const parsed = parse(source).wireframe;
  if (!parsed) throw new Error("not a valid source");
  return window(parsed.window, drawing);
}

describe("window", () => {
  it("is a desktop window by default: a title bar with dots and its title, over its parts", () => {
    const drawn = windowOf('window "Settings":\n  panel');
    expect(drawn.dataset.device).toBe("desktop");
    expect(drawn.querySelectorAll(".wf-titlebar .wf-dot")).toHaveLength(3);
    expect(texts(drawn, ".wf-titlebar .wf-title")).toEqual(["Settings"]);
    expect(texts(drawn, ".wf-body > i")).toEqual(["panel"]);
  });

  it("is a browser with its title as the address", () => {
    const drawn = windowOf('window "shop.example" device=browser:\n  panel');
    expect(texts(drawn, ".wf-browserbar .wf-address")).toEqual(["shop.example"]);
  });

  it("is a phone or a tablet with its title in a status bar, and no dots", () => {
    for (const device of ["phone", "tablet"]) {
      const drawn = windowOf(`window "Order" device=${device}:\n  panel`);
      expect(drawn.dataset.device).toBe(device);
      expect(texts(drawn, ".wf-statusbar")).toEqual(["Order"]);
      expect(drawn.querySelector(".wf-dot")).toBeNull();
    }
  });

  it("has an empty title when it has none", () => {
    expect(texts(windowOf("window:\n  panel"), ".wf-title")).toEqual([""]);
  });
});

describe("boxes", () => {
  it("draws header, footer and panel as boxes of their children", () => {
    expect(header(partOf("header:", "  text \"Hi\""), drawing).outerHTML).toBe('<div class="wf-header"><i>text</i></div>');
    expect(header(partOf("header large:", "  text \"Hi\""), drawing).className).toBe("wf-header wf-large");
    expect(footer(partOf("footer:", "  button"), drawing).className).toBe("wf-footer");
    expect(panel(partOf("panel:", "  button"), drawing).className).toBe("wf-panel");
  });

  it("draws a section with its title and badge over its children", () => {
    const drawn = section(partOf('section "Theme" badge="new":', "  radio \"Light\""), drawing);
    expect(texts(drawn, ".wf-section-title")).toEqual(["Themenew"]);
    expect(texts(drawn, ".wf-section-title .wf-badge")).toEqual(["new"]);
    expect(texts(drawn, ".wf-section-body > i")).toEqual(["radio"]);
  });
});

describe("row", () => {
  const placed = (line: string) => row(partOf(line, "  button"), drawing).style.justifyContent;

  it("places its parts by justify", () => {
    expect([placed("row justify=start:"), placed("row justify=between:"), placed("row justify=around:"), placed("row justify=end:")]).toEqual(["flex-start", "space-between", "space-around", "flex-end"]);
  });

  it("places them by align when it has no justify, from the left by default", () => {
    expect([placed("row:"), placed("row align=center:"), placed("row align=right:")]).toEqual(["flex-start", "center", "flex-end"]);
  });

  it("lets justify win over align, as Wireloom does", () => {
    expect(placed("row align=right justify=between:")).toBe("space-between");
  });
});

describe("col", () => {
  /** The `flex` of the col written as `line`, in a row, with a button in it. */
  const flexOf = (line: string) => {
    return col(childOf("row:", line, "  button"), drawing).style.flex;
  };

  it("is as wide as its width, in pixels by default", () => {
    expect(flexOf("col 240:")).toBe("0 0 240px");
    expect(flexOf("col 30%:")).toBe("0 0 30%");
  });

  it("takes a share of the row with `fr`, and the row's slack with `fill` or no width", () => {
    expect(flexOf("col 2fr:")).toMatch(/^2 1 0(px)?$/);
    expect(flexOf("col fill:")).toMatch(/^1 1 0(px)?$/);
    expect(flexOf("col:")).toMatch(/^1 1 0(px)?$/);
  });
});

describe("grid and cell", () => {
  it("draws a grid of its columns and rows", () => {
    const drawn = grid(partOf("grid cols=3 rows=2:", '  cell row=1 col=1'), drawing);
    expect([drawn.style.gridTemplateColumns, drawn.style.gridTemplateRows]).toEqual(["repeat(3, minmax(0, 1fr))", "repeat(2, auto)"]);
  });

  it("places a cell at its row and column, its label over its children", () => {
    const drawn = cell(childOf("grid cols=3 rows=2:", 'cell "notes.txt" row=2 col=3:', "  icon"), drawing);
    expect([drawn.style.gridRow, drawn.style.gridColumn]).toEqual(["2", "3"]);
    expect(texts(drawn, ".wf-label")).toEqual(["notes.txt"]);
    expect(texts(drawn, "i")).toEqual(["icon"]);
  });

  it("draws a cell without a label as its children alone", () => {
    expect(cell(childOf("grid cols=1 rows=1:", "cell row=1 col=1"), drawing).querySelector(".wf-label")).toBeNull();
  });
});
