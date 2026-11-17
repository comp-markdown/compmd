import { describe, expect, it } from "vitest";
import { DRAW } from "./draw.ts";
import { PARTS } from "./parts.ts";
import { drawn, FILES, ORDER, SETTINGS, texts } from "./testing.ts";

describe("the settings example", () => {
  const root = drawn(SETTINGS);

  it("draws the window with its title bar", () => {
    const window = root.querySelector<HTMLElement>(".wf-window");
    expect(window?.dataset.device).toBe("desktop");
    expect(root.querySelector(".wf-titlebar .wf-title")?.textContent).toBe("Settings");
  });

  it("draws tabs, the active one marked", () => {
    expect(texts(root, ".wf-tab")).toEqual(["General", "Privacy", "Advanced"]);
    expect(texts(root, ".wf-tab.wf-active")).toEqual(["General"]);
  });

  it("draws titled sections", () => {
    expect(texts(root, ".wf-section-title")).toEqual(["Notifications", "Theme"]);
  });

  it("draws each mark after its label, on when its flag says", () => {
    const toggle = root.querySelector(".wf-toggle");
    expect(toggle?.firstElementChild?.className).toBe("wf-label");
    expect(toggle?.querySelector(".wf-mark")?.classList.contains("wf-on")).toBe(true);
    expect(root.querySelector(".wf-checkbox .wf-mark.wf-on")).not.toBeNull();
    expect([...root.querySelectorAll(".wf-radio .wf-mark")].map((mark) => mark.classList.contains("wf-on"))).toEqual([false, true, false]);
  });

  it("places a row's parts by justify, and fills the primary button", () => {
    expect(root.querySelector<HTMLElement>(".wf-footer .wf-row")?.style.justifyContent).toBe("flex-end");
    expect(texts(root, ".wf-button.wf-primary")).toEqual(["Save"]);
  });
});

describe("the files example", () => {
  const root = drawn(FILES);

  it("draws each menu open below its title, its separator a line", () => {
    expect(texts(root, ".wf-menu-title")).toEqual(["File", "Edit"]);
    expect(texts(root, ".wf-menuitem-label")).toEqual(["New folder", "Close"]);
    expect(texts(root, ".wf-shortcut")).toEqual(["⌘N", "⌘W"]);
    expect(root.querySelector(".wf-menu-items [role=separator]")).not.toBeNull();
  });

  it("draws a col of its width and one that fills", () => {
    const [fixed, fill] = [...root.querySelectorAll<HTMLElement>(".wf-col")];
    expect(fixed?.style.flex).toBe("0 0 220px");
    expect(fill?.style.flex).toMatch(/^1 1 0(px)?$/);
  });

  it("draws the tree indented, a collapsed node closed", () => {
    expect(texts(root, ".wf-node-label")).toEqual(["▾Documents", "Reports", "▸Drafts", "Pictures"]);
    expect(texts(root, ".wf-node-label.wf-selected")).toEqual(["▾Documents"]);
    expect(root.querySelectorAll(".wf-tree .wf-tree")).toHaveLength(1);
  });

  it("draws the grid with each cell at its place", () => {
    const grid = root.querySelector<HTMLElement>(".wf-grid");
    expect(grid?.style.gridTemplateColumns).toBe("repeat(3, minmax(0, 1fr))");
    const cells = [...root.querySelectorAll<HTMLElement>(".wf-cell")];
    expect(cells.map((cell) => [cell.textContent, cell.style.gridRow, cell.style.gridColumn])).toEqual([
      ["report.pdf", "1", "1"],
      ["notes.txt", "1", "2"],
      ["photo.png", "1", "3"],
    ]);
  });
});

describe("the order example", () => {
  const root = drawn(ORDER);

  it("draws a phone's frame, with the title in its status bar", () => {
    expect(root.querySelector<HTMLElement>(".wf-window")?.dataset.device).toBe("phone");
    expect(root.querySelector(".wf-statusbar")?.textContent).toBe("Order");
  });

  it("draws the list's items with chevrons, and the progress filled to its value", () => {
    expect(texts(root, ".wf-item-text")).toEqual(["Coffee beans, 1 kg", "Filter papers"]);
    expect(root.querySelectorAll(".wf-item .wf-chevron")).toHaveLength(2);
    expect(root.querySelector(".wf-progress .wf-label")?.textContent).toBe("Step 2 of 3");
    expect(root.querySelector<HTMLElement>(".wf-progress-fill")?.style.width).toMatch(/^66\.66/);
  });

  it("numbers the annotation on its side and at its target", () => {
    expect(texts(root, ".wf-notes-bottom .wf-note")).toEqual(["1Disabled until an address is chosen"]);
    expect(root.querySelector(".wf-button .wf-marker-at")?.textContent).toBe("1");
    expect(root.querySelector(".wf-notes-right")).toBeNull();
  });
});

describe("the parts", () => {
  it("draws each part by its own function, and draws only parts the language has", () => {
    expect(Object.keys(DRAW).filter((name) => !(name in PARTS))).toEqual([]);
    const root = drawn('window:\n  row:\n    button "Go"\n    spacer');
    expect(texts(root, ".wf-row > .wf-button")).toEqual(["Go"]);
  });

  it("draws a part the pack doesn't draw yet as a box with its name and label, holding its children", () => {
    const root = drawn('window:\n  chip "New"\n  navbar:\n    leading:\n      backbutton');
    expect(texts(root, ".wf-named > .wf-name")).toEqual(["chip", "navbar", "leading", "backbutton"]);
    expect(texts(root, ".wf-named > .wf-label")).toEqual(["New"]);
    expect(root.querySelectorAll(".wf-named .wf-named .wf-named")).toHaveLength(1);
  });

  it("keeps every text as text", () => {
    expect(drawn(SETTINGS).textContent).toContain("Email me about replies");
  });
});

describe("annotations", () => {
  it("numbers each in the order written, on its side, the same number at its target", () => {
    const root = drawn('window:\n  button "A" id=a\n  button "B" id=b\nannotation "First" target="b" position=left\nannotation "Second" target="a" position=top');
    expect(texts(root, ".wf-notes-left .wf-note")).toEqual(["1First"]);
    expect(texts(root, ".wf-notes-top .wf-note")).toEqual(["2Second"]);
    expect(texts(root, ".wf-button")).toEqual(["A2", "B1"]);
  });

  it("draws no side without notes", () => {
    expect(drawn("window:\n  panel").querySelector(".wf-notes")).toBeNull();
  });
});