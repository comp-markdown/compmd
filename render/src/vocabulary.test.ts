// The renderer against the processor's fixed vocabulary (`conformance/params.json`, which a processor test keeps up
// to date): it styles exactly the params the processor takes, and gives every token CSS. A token whose CSS is the
// token itself, a CSS keyword of the property, is listed here, so a new token is a decision, not an accident.
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { ATTRIBUTES, blockCss, COMMON, LAYOUT } from "./styles.ts";

interface Known {
  name: string;
  tokens: string[];
  flag: boolean;
}

const vocabulary = JSON.parse(readFileSync(join(import.meta.dirname, "../../conformance/params.json"), "utf8")) as Record<"common" | "attributes" | "layout", Known[]>;

/** Tokens CSS takes as they are. */
const PASSED_THROUGH: Record<string, string[]> = {
  ...Object.fromEntries(["width", "height", "min-width", "max-width", "inset-inline-start", "inset-inline-end", "inset-block-start", "inset-block-end"].map((key) => [key, ["auto"]])),
  border: ["none"],
  "align-self": ["start", "center", "end", "stretch"],
  "justify-self": ["start", "center", "end"],
  display: ["block", "flex", "grid", "contents"],
  "flex-direction": ["row", "column"],
  "align-items": ["start", "center", "end", "stretch"],
  "justify-content": ["start", "center", "end", "space-between", "space-around"],
  "flex-wrap": ["wrap", "nowrap"],
};

/** Tokens that mean no CSS at all. */
const NOTHING: Record<string, string[]> = { sticky: ["none"] };

const names = (list: Known[]) => list.map((param) => param.name);

describe("the fixed vocabulary", () => {
  it("styles exactly the processor's common and layout params, and leaves the attributes to the box", () => {
    expect(COMMON).toEqual(names(vocabulary.common));
    expect(LAYOUT).toEqual(names(vocabulary.layout));
    expect(ATTRIBUTES).toEqual(names(vocabulary.attributes));
  });

  for (const param of [...vocabulary.common, ...vocabulary.layout]) {
    it(`gives each of \`${param.name}\`'s tokens its CSS`, () => {
      for (const token of param.tokens) {
        const value = param.flag ? token === "true" : token;
        const css = blockCss([{ key: param.name, breakpoint: null, value }], "grid", true);
        if (param.name === "show" || param.name === "hide") {
          expect(css[param.name]).toBe(token);
          continue;
        }
        const written = Object.values(css.base);
        if (NOTHING[param.name]?.includes(token) || value === false) {
          expect(written, `${param.name}=${token}`).toEqual([]);
          continue;
        }
        expect(written, `\`${param.name}=${token}\` has no CSS`).not.toEqual([]);
        if (written.every((css) => css === token)) expect(PASSED_THROUGH[param.name], `\`${param.name}=${token}\` passes through as it is; list it if that is right`).toContain(token);
      }
    });
  }
});
