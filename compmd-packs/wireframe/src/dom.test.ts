import { describe, expect, it } from "vitest";
import { attribute, badge, childrenOf, element, flag, flagged, icon, numberOf, textOf } from "./dom.ts";
import { drawing, partOf } from "./testing.ts";

describe("element", () => {
  it("makes an element with a `wf-` class per name, its strings as text, leaving out what is undefined", () => {
    const made = element(document, "span", ["button", "primary"], "Go", undefined, element(document, "b", ["badge"], "2"));
    expect(made.tagName).toBe("SPAN");
    expect(made.className).toBe("wf-button wf-primary");
    expect(made.childNodes).toHaveLength(2);
    expect(made.textContent).toBe("Go2");
  });
});

describe("reading a part", () => {
  const part = partOf('button "Save" primary badge="3" icon=save id=go');

  it("gives its positional text, or undefined", () => {
    expect(textOf(part)).toBe("Save");
    expect(textOf(part, 1)).toBeUndefined();
    expect(textOf(partOf("col 240"))).toBeUndefined();
  });

  it("gives an attribute as text, whatever its kind", () => {
    expect(attribute(part, "badge")).toBe("3");
    expect(attribute(part, "icon")).toBe("save");
    expect(attribute(partOf("image width=120"), "width")).toBe("120");
    expect(attribute(partOf("slider range=0-100 value=5"), "range")).toBe("0-100");
    expect(attribute(part, "accent")).toBeUndefined();
  });

  it("gives a number attribute as a number, and nothing for one that isn't", () => {
    expect(numberOf(partOf("progress value=2 max=3"), "max")).toBe(3);
    expect(numberOf(part, "badge")).toBeUndefined();
  });

  it("tells its flags, and gives those of a list it has", () => {
    expect(flag(part, "primary")).toBe(true);
    expect(flag(part, "disabled")).toBe(false);
    expect(flagged(part, ["disabled", "primary"])).toEqual(["primary"]);
  });

  it("draws its children with the drawing it is given", () => {
    expect(childrenOf(partOf("row:", "  button", "  spacer"), drawing).map((child) => child.textContent)).toEqual(["button", "spacer"]);
  });
});

describe("badges and icons", () => {
  it("draws a badge only when there is text for it", () => {
    expect(badge(document, "3")?.outerHTML).toBe('<span class="wf-badge">3</span>');
    expect(badge(document, undefined)).toBeUndefined();
  });

  it("draws an icon as its name's first letter, named in its title", () => {
    const drawn = icon(document, "search");
    expect([drawn?.textContent, drawn?.title, drawn?.className]).toEqual(["S", "search", "wf-icon"]);
    expect(icon(document, "")?.textContent).toBe("");
    expect(icon(document, undefined)).toBeUndefined();
  });
});
