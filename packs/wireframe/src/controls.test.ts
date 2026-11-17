import { describe, expect, it } from "vitest";
import * as controls from "./controls.ts";
import { childOf, drawing, partOf, texts } from "./testing.ts";

/** `part` drawn by `draw`, the part written as `lines`. */
const drawn = (draw: (typeof controls)[keyof typeof controls], ...lines: string[]) => draw(partOf(...lines), drawing);

describe("text", () => {
  it("is a line of its text, styled by its flags, weight and size", () => {
    expect(drawn(controls.text, 'text "Hi"').outerHTML).toBe('<div class="wf-text">Hi</div>');
    expect(drawn(controls.text, 'text "Hi" bold italic muted').className).toBe("wf-text wf-bold wf-italic wf-muted");
    expect(drawn(controls.text, 'text "Hi" weight=semibold size=large').className).toBe("wf-text wf-weight-semibold wf-size-large");
  });
});

describe("button", () => {
  it("is a box of its label, filled when primary, with its icon and badge", () => {
    const button = drawn(controls.button, 'button "Inbox" primary icon=mail badge="4"');
    expect(button.className).toBe("wf-button wf-primary");
    expect([texts(button, ".wf-icon"), texts(button, ".wf-badge"), button.textContent]).toEqual([["M"], ["4"], "MInbox4"]);
  });

  it("is an icon alone without a label, and marked when disabled", () => {
    expect(drawn(controls.button, "button icon=gear").textContent).toBe("G");
    expect(drawn(controls.button, 'button "Go" disabled').className).toBe("wf-button wf-disabled");
  });
});

describe("input", () => {
  it("shows its placeholder, muted, or dots for a password", () => {
    expect(texts(drawn(controls.input, 'input placeholder="Email"'), ".wf-placeholder")).toEqual(["Email"]);
    expect(texts(drawn(controls.input, 'input placeholder="Password" type=password'), ".wf-dots-text")).toEqual(["••••••••"]);
    expect(drawn(controls.input, "input disabled").className).toBe("wf-input wf-disabled");
  });
});

describe("combo", () => {
  it("shows its label, then a box of its value and a chevron", () => {
    const combo = drawn(controls.combo, 'combo "Size" value="M" options="S, M, L"');
    expect([texts(combo, ".wf-label"), texts(combo, ".wf-value"), texts(combo, ".wf-chevron")]).toEqual([["Size"], ["M"], ["▾"]]);
  });

  it("shows its first option without a value, and nothing without either", () => {
    expect(texts(drawn(controls.combo, 'combo options="S, M, L"'), ".wf-value")).toEqual(["S"]);
    expect(texts(drawn(controls.combo, "combo"), ".wf-value")).toEqual([""]);
    expect(drawn(controls.combo, "combo").querySelector(".wf-label")).toBeNull();
  });
});

describe("marks", () => {
  const order = (choice: HTMLElement) => [...choice.children].map((child) => (child.classList.contains("wf-mark") ? "mark" : "label"));

  it("draws a label, then the mark; with label-right the mark first", () => {
    expect(order(drawn(controls.checkbox, 'checkbox "Remember me"'))).toEqual(["label", "mark"]);
    expect(order(drawn(controls.checkbox, 'checkbox "Remember me" label-right'))).toEqual(["mark", "label"]);
  });

  it("turns each mark on by its own flag", () => {
    const on = (choice: HTMLElement) => choice.querySelector(".wf-mark")?.classList.contains("wf-on");
    expect([on(drawn(controls.checkbox, 'checkbox "A" checked')), on(drawn(controls.checkbox, 'checkbox "A"'))]).toEqual([true, false]);
    expect([on(drawn(controls.radio, 'radio "A" selected')), on(drawn(controls.radio, 'radio "A"'))]).toEqual([true, false]);
    expect([on(drawn(controls.toggle, 'toggle "A" on')), on(drawn(controls.toggle, 'toggle "A" off'))]).toEqual([true, false]);
  });

  it("names its kind, and is marked when disabled", () => {
    expect(drawn(controls.radio, 'radio "A" disabled').className).toBe("wf-choice wf-radio wf-disabled");
    expect(drawn(controls.toggle, 'toggle "A"').querySelector(".wf-toggle-mark")).not.toBeNull();
  });
});

describe("progress", () => {
  const width = (line: string) => drawn(controls.progress, line).querySelector<HTMLElement>(".wf-progress-fill")?.style.width;

  it("fills its bar to value of max, its label above it", () => {
    expect(width("progress value=1 max=4")).toBe("25%");
    expect(texts(drawn(controls.progress, 'progress value=1 max=4 label="Step 1"'), ".wf-label")).toEqual(["Step 1"]);
  });

  it("counts to 100 without a max, and keeps the fill between empty and full", () => {
    expect(width("progress value=30")).toBe("30%");
    expect(width("progress value=9 max=3")).toBe("100%");
    expect(width("progress value=1 max=0")).toBe("0%");
  });
});

describe("tabs, lists and trees", () => {
  it("draws tabs as a row of tabs, the active one marked, with badges", () => {
    const tabs = drawn(controls.tabs, "tabs:", '  tab "General"');
    expect(tabs.className).toBe("wf-tabs");
    const tab = controls.tab(childOf("tabs:", 'tab "Inbox" active badge="2"'), drawing);
    expect([tab.className, tab.textContent]).toEqual(["wf-tab wf-active", "Inbox2"]);
  });

  it("draws a list's items, with a chevron where they have one", () => {
    expect(drawn(controls.list, "list:", '  item "A"').tagName).toBe("UL");
    const item = (line: string) => controls.item(childOf("list:", line), drawing);
    expect(texts(item('item "Coffee" chevron'), ".wf-chevron")).toEqual(["›"]);
    expect(item('item "Coffee"').querySelector(".wf-chevron")).toBeNull();
  });

  it("draws a node open with its children, closed when collapsed, and a leaf without a chevron", () => {
    const node = (...lines: string[]) => controls.node(childOf("tree:", ...lines), drawing);
    const open = node('node "Docs" selected icon=folder:', '  node "Reports"');
    expect(texts(open, ".wf-node-label")).toEqual(["▾FDocs"]);
    expect(open.querySelector(".wf-node-label")?.classList.contains("wf-selected")).toBe(true);
    expect(texts(open, ".wf-tree > i")).toEqual(["node"]);
    const closed = node('node "Drafts" collapsed:', '  node "Old"');
    expect([texts(closed, ".wf-chevron"), closed.querySelector(".wf-tree")]).toEqual([["▸"], null]);
    expect(texts(node('node "Pictures"'), ".wf-chevron")).toEqual([""]);
  });
});

describe("menus", () => {
  it("draws a menu's title, its items open below it, or the title alone", () => {
    const menu = (...lines: string[]) => controls.menu(childOf("menubar:", ...lines), drawing);
    expect(texts(menu('menu "File":', '  menuitem "New"'), ".wf-menu-title")).toEqual(["File"]);
    expect(texts(menu('menu "File":', '  menuitem "New"'), ".wf-menu-items > i")).toEqual(["menuitem"]);
    expect(menu('menu "Edit"').querySelector(".wf-menu-items")).toBeNull();
    expect(drawn(controls.menubar, "menubar:", '  menu "File"').className).toBe("wf-menubar");
  });

  it("draws a menu item with its shortcut, marked when disabled", () => {
    const item = (line: string) => {
      const menuitem = childOf("menubar:", 'menu "File":', `  ${line}`).children[0];
      if (!menuitem) throw new Error("a menu item");
      return controls.menuitem(menuitem, drawing);
    };
    expect(texts(item('menuitem "Close" shortcut="⌘W" disabled'), ".wf-shortcut")).toEqual(["⌘W"]);
    expect(item('menuitem "Close" disabled').className).toBe("wf-menuitem wf-disabled");
    expect(item('menuitem "Close"').querySelector(".wf-shortcut")).toBeNull();
  });
});

describe("the rest", () => {
  it("draws a divider as a separator line", () => {
    expect(drawn(controls.divider, "divider").getAttribute("role")).toBe("separator");
  });

  it("draws an image as a box of its size and label, 96 pixels high by default", () => {
    const image = drawn(controls.image, 'image width=120 height=80 label="Hero"');
    expect([image.style.width, image.style.height, image.textContent]).toEqual(["120px", "80px", "Hero"]);
    expect([drawn(controls.image, "image").style.width, drawn(controls.image, "image").style.height]).toEqual(["", "96px"]);
  });

  it("draws an icon as its name's first letter, or a question mark without one", () => {
    expect(drawn(controls.iconPart, "icon name=search").textContent).toBe("S");
    expect(drawn(controls.iconPart, "icon").textContent).toBe("?");
  });

  it("draws a spacer as the row's slack", () => {
    expect(controls.spacer(childOf("row:", "spacer"), drawing).className).toBe("wf-spacer");
  });
});
