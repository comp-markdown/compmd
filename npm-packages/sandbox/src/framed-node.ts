// What a use frame is sent of its use (docs/sizing-components.md, decision 2): the page's box has the box's params, so
// the frame gets none of them, but for those that say how its content sits in the box or scrolls in it, and the box's
// direction and language as the page has them, for the frame's text and its component's `fit`.
import { BOX_PARAMS, isRtl, type ExpandedParam, type ExpandedUse } from "@compmd/render";

/** The box params a frame needs: they say what happens inside the box, which is the frame's. */
const INSIDE: ReadonlySet<string> = new Set(["object-fit", "object-position", "scroll"]);

const param = (key: string, value: string): ExpandedParam => ({ key, breakpoint: null, value });

/** `params` as the frame gets them, with `box`'s direction and language. */
function framedParams(params: readonly ExpandedParam[], box: Element): ExpandedParam[] {
  const lang = box.closest("[lang]")?.getAttribute("lang");
  const own = params.filter(({ key }) => !BOX_PARAMS.has(key) || INSIDE.has(key));
  return [...own, param("dir", isRtl(box) ? "rtl" : "ltr"), ...(lang ? [param("lang", lang)] : [])];
}

/**
 * `node`, a use framed in `box`, as its frame renders it: its own params, `object-fit`, `object-position` and
 * `scroll`, and `box`'s direction and language; a code block's renderer's params likewise.
 *
 * @example
 * child.part({ node: framedNode(node, box), inputs });
 */
export function framedNode(node: ExpandedUse, box: Element): ExpandedUse {
  if (node.type !== "code") return { ...node, params: framedParams(node.params, box) };
  const { component } = node;
  return component ? { ...node, component: { ...component, params: framedParams(component.params, box) } } : node;
}
