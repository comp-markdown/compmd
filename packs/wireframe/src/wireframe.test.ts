import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import { parse } from "./parse.ts";
import { ORDER, SETTINGS } from "./testing.ts";
import mount, { failureOf } from "./wireframe.ts";

describe("wireframe", () => {
  it("draws the block as a wireframe labelled with its text alternative, with its stylesheet", async () => {
    const drawn = mountForTest(mount, { params: { code: SETTINGS }, fallback: "The settings screen" });
    await drawn.settled();
    const image = drawn.target.querySelector("[role=img]");
    expect(image?.getAttribute("aria-label")).toBe("The settings screen");
    expect(image?.querySelector(".wf-titlebar .wf-title")?.textContent).toBe("Settings");
    expect(image?.querySelector("style")?.textContent).toContain(".wf-wireframe");
    expect(drawn.failures).toEqual([]);
  });

  it("follows new code and the page's scheme, and goes when destroyed", async () => {
    const drawn = mountForTest(mount, { params: { code: SETTINGS }, fallback: "A screen" });
    await drawn.settled();
    drawn.update({ code: ORDER });
    await drawn.settled();
    expect(drawn.target.querySelector(".wf-window")?.getAttribute("data-device")).toBe("phone");
    const scheme = () => drawn.target.querySelector<HTMLElement>(".wf-wireframe")?.dataset.scheme;
    expect(scheme()).toBe("light");
    drawn.setScheme("dark");
    await drawn.settled();
    expect(scheme()).toBe("dark");
    drawn.destroy();
    expect(drawn.target.children).toHaveLength(0);
  });

  it("fails with the source's first error, where it is, and draws again once the code is fixed", async () => {
    const drawn = mountForTest(mount, { params: { code: 'window "A":\n  butn "Go"\n  spacer' }, fallback: "A screen" });
    await drawn.settled();
    expect(drawn.failures).toEqual(["line 2, column 3: no part `butn`: did you mean `button`? (and 1 more)"]);
    drawn.update({ code: 'window "A":\n  button "Go"' });
    await drawn.settled();
    expect(drawn.target.querySelector(".wf-button")?.textContent).toBe("Go");
  });
});

describe("the test page", () => {
  it("has blocks that are all valid but the one with a mistake", async () => {
    const demo = await readFile(join(import.meta.dirname, "..", "demo.comp.md"), "utf8");
    const blocks = [...demo.matchAll(/```wireframe[^\n]*\n([\s\S]*?)\n```/g)].map((match) => parse(match[1] ?? ""));
    expect(blocks.map((parsed) => parsed.errors.map(String))).toEqual([[], [], [], [], [expect.stringContaining("line 2, column 3: no part `butn`")]]);
  });
});

describe("the Author's Guide", () => {
  it("has a sketch that is valid", async () => {
    const guide = await readFile(join(import.meta.dirname, "../../../guides/authors/examples/wireframes/booking.comp.md"), "utf8");
    const blocks = [...guide.matchAll(/```wireframe[^\n]*\n([\s\S]*?)\n```/g)].map((match) => parse(match[1] ?? ""));
    expect(blocks.map((parsed) => parsed.errors.map(String))).toEqual([[]]);
  });
});

describe("failureOf", () => {
  it("says the first error, and how many more there are", () => {
    expect(failureOf([new Error("one")])).toBe("one");
    expect(failureOf([new Error("one"), new Error("two"), new Error("three")])).toBe("one (and 2 more)");
    expect(failureOf([])).toBe("nothing to draw");
  });
});
