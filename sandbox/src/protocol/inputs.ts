// Which inputs a frame may set: those its uses bind a param to, which a parent checks each `set` against.

import { isComponent, walk, type ExpandedNode } from "@compmd/render";

/** The inputs the components in `nodes` bind their params to, slots and fallbacks included. */
export function boundInputs(nodes: readonly ExpandedNode[]): Set<string> {
  const inputs = new Set<string>();
  walk(nodes, (node) => {
    if (!isComponent(node)) return;
    for (const param of node.params) if (param.input !== undefined) inputs.add(param.input);
  });
  return inputs;
}
