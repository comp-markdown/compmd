import type { ReactElement } from "react";

interface Props {
  value: number;
  of: number;
  compmd: { set(param: string, value: unknown): void; fallback: { text: string } };
}

// A rating of `of` stars, `value` of them filled; a click sets the input `value` is bound to.
export default function Stars({ value, of, compmd }: Props): ReactElement {
  return (
    <span role="radiogroup" aria-label={compmd.fallback.text}>
      {Array.from({ length: of }, (_, i) => (
        <button
          key={i}
          type="button"
          role="radio"
          aria-checked={i + 1 === value}
          aria-label={`${String(i + 1)} of ${String(of)}`}
          onClick={() => {
            compmd.set("value", i + 1);
          }}
        >
          {i < value ? "★" : "☆"}
        </button>
      ))}
    </span>
  );
}
