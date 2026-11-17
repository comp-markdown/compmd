import { describe, expect, it } from "vitest";
import { DRAW, draw } from "./draw.ts";
import { parse } from "./parse.ts";
import { STYLE } from "./style.ts";

/** `source` drawn, or a failure naming its errors. */
function drawn(source: string): HTMLElement {
  const { wireframe, errors } = parse(source);
  if (!wireframe) throw new Error(errors.map(String).join("\n"));
  return draw(wireframe, document);
}

const texts = (root: HTMLElement, selector: string) => [...root.querySelectorAll(selector)].map((found) => found.textContent);

// LANGUAGE.md's examples.
const SETTINGS = `window "Settings":
  tabs:
    tab "General" active
    tab "Privacy"
    tab "Advanced"
  section "Notifications":
    toggle "Email me about replies" on
    checkbox "Weekly summary" checked
  section "Theme":
    radio "Light" group="theme"
    radio "Dark" group="theme" selected
    radio "Follow the system" group="theme"
  footer:
    row justify=end:
      button "Cancel"
      button "Save" primary`;

const FILES = `window "Files":
  menubar:
    menu "File":
      menuitem "New folder" shortcut="⌘N"
      separator
      menuitem "Close" shortcut="⌘W"
    menu "Edit"
  row:
    col 220:
      tree:
        node "Documents" selected:
          node "Reports"
          node "Drafts" collapsed
        node "Pictures"
    col fill:
      grid cols=3 rows=2:
        cell "report.pdf" row=1 col=1
        cell "notes.txt" row=1 col=2
        cell "photo.png" row=1 col=3`;

const ORDER = `window "Order" device=phone:
  header:
    text "Your basket" bold
  list:
    item "Coffee beans, 1 kg" chevron
    item "Filter papers" chevron
  progress value=2 max=3 label="Step 2 of 3"
  button "Pay now" primary id=pay
annotation "Disabled until an address is chosen" target="pay" position=bottom`;

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

describe("the other parts", () => {
  it("draws a part the pack doesn't draw yet as a box with its name and label", () => {
    const root = drawn('window:\n  chip "New"');
    expect(texts(root, ".wf-named .wf-name")).toEqual(["chip"]);
    expect(texts(root, ".wf-named .wf-label")).toEqual(["New"]);
  });

  it("draws a password as dots, and a placeholder as it is", () => {
    const root = drawn('window:\n  input placeholder="Email"\n  input placeholder="Password" type=password');
    expect(texts(root, ".wf-input")).toEqual(["Email", "••••••••"]);
  });

  it("places a row by align when it has no justify", () => {
    const root = drawn("window:\n  row align=center:\n    button");
    expect(root.querySelector<HTMLElement>(".wf-row")?.style.justifyContent).toBe("center");
  });

  it("puts the label on the other side with label-right", () => {
    const root = drawn('window:\n  checkbox "Remember me" label-right');
    expect(root.querySelector(".wf-checkbox")?.firstElementChild?.classList.contains("wf-mark")).toBe(true);
  });

  it("draws a combo's label, then its value or first option", () => {
    const root = drawn('window:\n  combo "Size" options="S, M, L"');
    expect(texts(root, ".wf-combo .wf-label")).toEqual(["Size"]);
    expect(texts(root, ".wf-combo .wf-value")).toEqual(["S"]);
  });

  it("draws an icon as its first letter, and an image of its size", () => {
    const root = drawn('window:\n  icon name="search"\n  image width=120 height=80 label="Hero"');
    expect(texts(root, ".wf-icon")).toEqual(["S"]);
    const image = root.querySelector<HTMLElement>(".wf-image");
    expect([image?.style.width, image?.style.height, image?.textContent]).toEqual(["120px", "80px", "Hero"]);
  });

  it("keeps every text as text", () => {
    expect(drawn(SETTINGS).textContent).toContain("Email me about replies");
  });
});

describe("the style", () => {
  it("styles every class the drawing makes, but those that only name a part", () => {
    const classes = new Set([SETTINGS, FILES, ORDER, 'window:\n  chip "x"'].flatMap((source) => [...drawn(source).querySelectorAll("*"), drawn(source)].flatMap((found) => [...found.classList])));
    const named = (name: string) => name.slice("wf-".length) in DRAW;
    const missing = [...classes].filter((name) => !named(name) && !new RegExp(`\\.${name}(?![\\w-])`).test(STYLE));
    expect(missing).toEqual([]);
  });

  it("takes its greys from the host's tokens, and follows the page's scheme", () => {
    expect(STYLE).toContain("var(--compmd-border-strong");
    expect(STYLE).toContain("var(--compmd-surface-subtle");
    expect(STYLE).toContain('[data-scheme="dark"]');
  });
});
