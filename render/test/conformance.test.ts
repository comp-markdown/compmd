// Every expanded tree in the conformance set (spec §13.2) renders, and the region layer builds the same document
// as the page: one region per top-level block, each told what the page around it knows.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import type { Mount } from "../src/contract.ts";
import { definitionsOf, footnotesOf } from "../src/document.ts";
import { createPage } from "../src/page.ts";
import { createRegion } from "../src/region.ts";
import { Stylesheet } from "../src/styles.ts";
import { isComponent, walk, type ExpandedNode, type ExpandedRoot } from "../src/tree.ts";

const CONFORMANCE = join(import.meta.dirname, "..", "..", "conformance");

function expandedTrees(): { name: string; tree: ExpandedRoot }[] {
  return readdirSync(CONFORMANCE).flatMap((area) => {
    const dir = join(CONFORMANCE, area);
    if (!statSync(dir).isDirectory()) return [];
    return readdirSync(dir).flatMap((name) => {
      const file = join(dir, name, "expanded.json");
      try {
        return [{ name: `${area}/${name}`, tree: JSON.parse(readFileSync(file, "utf8")) as ExpandedRoot }];
      } catch {
        return [];
      }
    });
  });
}

/** A plain component for every host-language component a tree uses, showing its name. */
function stubs(tree: ExpandedRoot): Record<string, Mount> {
  const names = new Set<string>();
  walk(tree.children, (node) => {
    if (isComponent(node)) names.add(node.name);
  });
  return Object.fromEntries(
    [...names].map((name) => [
      name,
      (target: Element) => {
        target.append(name);
      },
    ]),
  );
}

const TREES = expandedTrees();

describe("the conformance set's expanded trees", () => {
  it("are all here", () => {
    expect(TREES.length).toBeGreaterThanOrEqual(26);
  });

  for (const { name, tree } of TREES) {
    it(`${name}: renders as a page, and as a region per block with the same text`, () => {
      const errors: string[] = [];
      const options = { components: stubs(tree), onError: ({ component, reason }: { component: string; reason: string }) => void errors.push(`${component} ${reason}`) };
      const pageRoot = document.body.appendChild(document.createElement("main"));
      const page = createPage(pageRoot, options);
      page.render(tree);
      expect(errors).toEqual([]);

      const blocksRoot = document.body.appendChild(document.createElement("main"));
      blocksRoot.className = "compmd-page";
      const stylesheet = new Stylesheet(document);
      const footnotes = footnotesOf(tree.children);
      const definitions = definitionsOf(tree.children);
      let headingLevel = 0;
      const regions = tree.children.map((block: ExpandedNode) => {
        const target = blocksRoot.appendChild(document.createElement("div"));
        const region = createRegion(target, { ...options, stylesheet });
        region.render([block], { ...(tree.inputs ? { inputs: tree.inputs } : {}), headingLevel, footnotes: footnotes.numbers, definitions });
        if (block.type === "heading") headingLevel = Number(block.depth);
        return region;
      });
      const list = pageRoot.querySelector(".compmd-footnotes")?.textContent ?? "";
      expect(blocksRoot.textContent).toBe(pageRoot.textContent.slice(0, pageRoot.textContent.length - list.length));

      for (const region of regions) region.destroy();
      stylesheet.remove();
      page.destroy();
      pageRoot.remove();
      blocksRoot.remove();
    });
  }
});
