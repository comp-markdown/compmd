// `counter`: a button that counts the reader's clicks, drawn with React inside the pack's `mount`.
import { component } from "@compmd/pack/component";
import { useState } from "react";
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";

/** The params, as the signature in `index.comp.md` declares them. */
export interface Params {
  label: string;
  start: number;
}

function Counter({ label, start }: Params) {
  const [count, setCount] = useState(start);
  return (
    <button type="button" onClick={() => setCount(count + 1)}>
      {label}: {count}
    </button>
  );
}

export default component<Params>((target, { params }) => {
  const root = createRoot(target);
  const render = (next: Params) => {
    flushSync(() => {
      root.render(<Counter {...next} />);
    });
  };
  render(params);
  return {
    update: render,
    destroy: () => {
      root.unmount();
    },
  };
});
