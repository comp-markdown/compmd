// Which inputs a frame may set: those its uses bind a param to, which a parent checks each `set` against.

import { isComponent, rendererOf, walk, type ExpandedNode } from "@compmd/render";

/** The inputs the components in `nodes` bind their params to, slots, fallbacks and rendered code blocks included. */
export function boundInputs(nodes: readonly ExpandedNode[]): Set<string> {
  const inputs = new Set<string>();
  walk(nodes, (node) => {
    const component = isComponent(node) ? node : rendererOf(node);
    for (const param of component?.params ?? []) if (param.input !== undefined) inputs.add(param.input);
  });
  return inputs;
}
