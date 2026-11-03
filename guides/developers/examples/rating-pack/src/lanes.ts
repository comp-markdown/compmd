import { component, type Slots } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. `as` is the author's name for a record. */
export interface Params {
  rows: Record<string, unknown>[];
  by: string;
  as: string;
}

/** One instance of the template slot: its record, and the blocks the template rendered for it. */
interface Instance {
  value: unknown;
  items: { element: Element }[];
}

export default component<Params>((target, { params, slots }) => {
  const board = document.createElement("div");
  board.style.cssText = "display: flex; gap: 1rem; align-items: start;";
  target.append(board);
  const draw = ({ by }: Params, { default: instances = [] }: Slots) => {
    const lanes = new Map<string, HTMLElement>();
    for (const { value, items } of instances as Instance[]) {
      const field = (value as Record<string, unknown> | null)?.[by];
      const name = typeof field === "string" ? field : "";
      let lane = lanes.get(name);
      if (!lane) {
        lane = document.createElement("section");
        lane.append(Object.assign(document.createElement("h3"), { textContent: name }));
        lanes.set(name, lane);
      }
      // The renderer made these elements, and keeps what is in them live: the component only places them.
      lane.append(...items.map(({ element }) => element));
    }
    board.replaceChildren(...lanes.values());
  };
  draw(params, slots);
  return {
    update: draw,
    destroy: () => {
      board.remove();
    },
  };
});
