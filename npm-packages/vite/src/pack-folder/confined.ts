// A path in a pack, as the processor's `FilePath` reads one (spec §8.6): `/`-separated names that can't leave the
// pack's folder. A name with `\` or `:` is a Windows path of its own, which a host joining it to a folder would follow
// out of the pack.

/** Whether `name` is a file's or a folder's name in a pack: not empty, `.` or `..`, with no `\`, `:` or NUL. */
const isName = (name: string): boolean => name !== "" && name !== "." && name !== ".." && !/[\\:\0]/.test(name);

/**
 * Whether `path` is a file's path in a pack: `dist/chart.js`, never `./a.js`, `../a.js`, `/a.js`, `a\b.js` or `C:/a.js`.
 *
 * @example
 * isPackPath("dist/chart.js"); // true
 * isPackPath("..\\secrets.txt"); // false
 */
export const isPackPath = (path: string): boolean => path.split("/").every(isName);

/**
 * The path in the pack that `relative` (`./card.comp.md`, `../icons/star.svg`) names, seen from the folder of the file
 * at `from`, a path in the pack; `undefined` when it starts at a root (`/`), leaves the pack,
 * names its folder, or a name isn't a file's name.
 *
 * @example
 * joinedInPack("parts/cards.comp.md", "../icons/star.svg"); // "icons/star.svg"
 * joinedInPack("index.comp.md", "./..\\..\\secrets.txt"); // undefined
 */
export function joinedInPack(from: string, relative: string): string | undefined {
  if (relative.startsWith("/")) return undefined;
  const names = from.split("/").slice(0, -1);
  for (const part of relative.split("/")) {
    if (part === "" || part === ".") continue;
    if (part !== "..") names.push(part);
    else if (names.pop() === undefined) return undefined;
  }
  const path = names.join("/");
  return isPackPath(path) ? path : undefined;
}
