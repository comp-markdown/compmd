// The generated fence between its comments: found, written again with the rest of the document kept, and comments that
// aren't one pair around one fence.
import { describe, expect, it } from "vitest";
import { generatedIn, generatedPart, opening, withGenerated } from "./generated.ts";

const heading = opening("qr", "2.2.0");
const fence = "```compmd component=qr-code signature\nparams:\n  value: string\n```";
const document = `---\nmd: {}\n---\n\n# QR codes\n\n${generatedPart(heading, fence)}\n\nProse after.\n`;

describe("generatedIn", () => {
  it("finds the fence between the comments, with its component", () => {
    const found = generatedIn(document, "");
    expect(found).toMatchObject({ kind: "generated", component: "qr-code", fence });
    if (found.kind !== "generated") return;
    const again = withGenerated(document, found, opening("qr", "2.3.0"), fence.replace("string", "string?"));
    expect(again).toContain("from qr 2.3.0's types");
    expect(again).toContain("value: string?");
    expect(again.startsWith("---\nmd: {}\n---\n\n# QR codes\n\n")).toBe(true);
    expect(again.endsWith("<!-- /compmd-pack:generated -->\n\nProse after.\n")).toBe(true);
  });

  it("gives the author's fence when there are no comments", () => {
    expect(generatedIn(`# QR\n\n${fence}\n`, "qr-code")).toEqual({ kind: "authors", fence });
    expect(generatedIn("# QR\n", "qr-code")).toEqual({ kind: "authors", fence: undefined });
  });

  it("says what is wrong with comments that aren't one pair around one fence", () => {
    const why = (text: string) => {
      const found = generatedIn(text, "");
      return found.kind === "broken" ? found.why : found.kind;
    };
    expect(why(`${heading}\n${fence}\n`)).toMatch(/1 opening and 0 closing/);
    expect(why(`${generatedPart(heading, fence)}\n${generatedPart(heading, fence)}`)).toMatch(/2 opening and 2 closing/);
    expect(why(`<!-- /compmd-pack:generated -->\n${heading}\n`)).toMatch(/closing .* comes before/);
    expect(why(generatedPart(heading, `${fence}\n\nA word.`))).toMatch(/one signature fence, and nothing else/);
    expect(why(generatedPart(heading, "No fence."))).toMatch(/one signature fence/);
  });
});
