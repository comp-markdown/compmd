import type { ReactElement } from "react";

interface Props {
  value: number;
  of: number;
  compmd: { set(param: string, value: unknown): void; fallback: { text: string } };
}

/** The most stars a document may ask for. */
const MOST = 10;

/** `value` as a whole number from 0 to `max`, whatever a document gave. */
function wholeUpTo(value: unknown, max: number): number {
  const number = Math.round(Number(value));
  return Number.isFinite(number) ? Math.min(Math.max(number, 0), max) : 0;
}

// A rating of `of` stars, `value` of them filled; a click sets the input `value` is bound to.
export default function Stars({ value, of, compmd }: Props): ReactElement {
  const count = Math.max(1, wholeUpTo(of, MOST));
  const filled = wholeUpTo(value, count);
  return (
    <span role="radiogroup" aria-label={compmd.fallback.text}>
      {Array.from({ length: count }, (_, i) => (
        <button
          key={i}
          type="button"
          role="radio"
          aria-checked={i + 1 === filled}
          aria-label={`${String(i + 1)} of ${String(count)}`}
          onClick={() => {
            compmd.set("value", i + 1);
          }}
        >
          {i < filled ? "★" : "☆"}
        </button>
      ))}
    </span>
  );
}
