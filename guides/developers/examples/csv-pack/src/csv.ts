// `csv`: draws a ```csv block, its text in `code`, as a table; the first row is its header unless `header=false`.
import { component } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them, and `code`, which every code renderer has. */
export interface Params {
  code: string;
  header: boolean;
}

/** The rows of `code`: fields split at commas, a field in double quotes holding commas and `""` for a quote. */
export function rowsOf(code: string): string[][] {
  return code
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map((line) => [...line.matchAll(/(?:^|,)(?:"((?:[^"]|"")*)"|([^,]*))/g)].map(([, quoted, plain]) => quoted?.replaceAll('""', '"') ?? plain?.trim() ?? ""));
}

export default component<Params>((target, { params, fallback }) => {
  const table = document.createElement("table");
  // The block's `alt`, or a leaf's fallback: the table's caption.
  const caption = Object.assign(document.createElement("caption"), { textContent: fallback.text });
  target.append(table);
  const cells = (row: string[], tag: "th" | "td") => {
    const tr = document.createElement("tr");
    tr.append(...row.map((text) => Object.assign(document.createElement(tag), { textContent: text })));
    return tr;
  };
  const draw = ({ code, header }: Params) => {
    const [first = [], ...rest] = rowsOf(code);
    const head = header ? [cells(first, "th")] : [];
    const body = (header ? rest : [first, ...rest]).map((row) => cells(row, "td"));
    table.replaceChildren(caption, ...head, ...body);
  };
  draw(params);
  return {
    update: draw,
    destroy: () => {
      table.remove();
    },
  };
});
