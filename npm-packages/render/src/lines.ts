// Where each line of the text shows on the page, for a gutter of line numbers: from the same origins as `source.ts`.
// A line's place is where what is drawn from it shows: its text, found in the page's text nodes by their `\n`s, a code
// block's line in its `<pre>`, or, for a line with no text of its own on the page, the box of what its range drew.
// The index reads only the DOM's text; each line is measured when the generator reaches it.
import type { Placed } from "./source.ts";

/** A box, in CSS pixels from the page root's top left corner: as `getBoundingClientRect()` gives it, moved there. */
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** A line of the current text, 1-based, and the boxes what is drawn from it shows in, one per visual line it covers. */
export interface Line {
  line: number;
  rects: Rect[];
}

/** How `linesIn` measures: a range's or an element's boxes, in the viewport. */
export type Measure = (target: Range | Element) => readonly Rect[];

/** Part of a text node's text, `from` to `to` in its UTF-16 units, at `order` in the line's drawing. */
interface Segment {
  node: Text;
  from: number;
  to: number;
  order: number;
}

/** What a line drew: its text, in every drawing of it, and the elements whose ranges start on it, innermost first. */
interface Drew {
  segments: Segment[];
  elements: { element: Element; length: number }[];
}

/** An element, not a range: by its node type, since a page can render into another frame's document. */
const isElement = (target: Range | Element): target is Element => "nodeType" in target;

const measured: Measure = (target) => (isElement(target) ? [target.getBoundingClientRect()] : Array.from(target.getClientRects()));

/** The index of each line's start in `text`: `[0, …]`. */
function lineStarts(text: string): number[] {
  const starts = [0];
  for (let i = text.indexOf("\n"); i !== -1; i = text.indexOf("\n", i + 1)) starts.push(i + 1);
  return starts;
}

/**
 * The parts of each line of `code`, whose lines start at line `first`, in its text nodes as a highlighter split them.
 * An empty line is placed by its line break, which the page draws where the line is.
 */
function codeSegments(code: Element, first: number, order: number, add: (line: number, segment: Segment) => void): void {
  const text = code.textContent;
  const starts = lineStarts(text);
  const walker = code.ownerDocument.createTreeWalker(code, 4 /* NodeFilter.SHOW_TEXT */);
  let at = 0;
  let line = 0;
  for (let node = walker.nextNode() as Text | null; node; node = walker.nextNode() as Text | null) {
    const end = at + node.data.length;
    for (let i = line; i < starts.length && (starts[i] ?? 0) < end; i++) {
      const start = starts[i] ?? 0;
      const stop = Math.max((starts[i + 1] ?? text.length + 1) - 1, start + 1);
      const from = Math.max(start, at);
      const to = Math.min(stop, end);
      if (from < to) add(first + i, { node, from: from - at, to: to - at, order: order + from });
      // The next node starts with this line's rest, or a later line.
      if (stop <= end) line = i + 1;
    }
    at = end;
  }
}

/** What each line drew, by line: text segments in document order, elements by the length of their ranges. */
function drawnByLine(placed: Iterable<Placed>): Map<number, Drew> {
  const lines = new Map<number, Drew>();
  const drew = (line: number): Drew => {
    let found = lines.get(line);
    if (!found) lines.set(line, (found = { segments: [], elements: [] }));
    return found;
  };
  const add = (line: number, segment: Segment) => drew(line).segments.push(segment);
  for (const { dom, source, node, kept } of placed) {
    if (kept) continue;
    const { start, end } = source;
    const order = start.offset ?? 0;
    if (dom.nodeType === 3 /* Node.TEXT_NODE */) {
      const text = dom as Text;
      const starts = lineStarts(text.data);
      // Soft line breaks are the text's `\n`s; other text (an interpolation's value) places only the line it starts on.
      const count = starts.length === end.line - start.line + 1 ? starts.length : 1;
      for (let i = 0; i < count; i++) {
        const from = starts[i] ?? 0;
        const to = (starts[i + 1] ?? text.data.length + 1) - 1;
        if (from < to) add(start.line + i, { node: text, from, to, order: order + i });
      }
    } else if (dom.nodeType === 1 /* Node.ELEMENT_NODE */) {
      const element = dom as Element;
      const code = node.type === "code" && element.nodeName === "PRE" ? element.firstElementChild : null;
      if (code) {
        // A fenced block's code starts on the line after its fence; an indented one's, on its first line.
        const fenced = end.line - start.line + 1 > lineStarts(code.textContent).length;
        codeSegments(code, start.line + (fenced ? 1 : 0), order, add);
      } else {
        drew(start.line).elements.push({ element, length: (end.offset ?? 0) - (start.offset ?? 0) });
      }
    }
  }
  for (const found of lines.values()) found.elements.sort((a, b) => a.length - b.length);
  return lines;
}

/** Each drawing of a line's text: a new one starts where the order goes back, as each iteration of an `each` does. */
function drawings(segments: readonly Segment[]): Segment[][] {
  const runs: Segment[][] = [];
  let run: Segment[] = [];
  for (const segment of segments) {
    const last = run.at(-1);
    if (last && segment.order <= last.order) {
      runs.push(run);
      run = [];
    }
    run.push(segment);
  }
  if (run.length > 0) runs.push(run);
  return runs;
}

/** Boxes on one visual line merged into one, those whose spans overlap vertically; empty boxes left out. */
function byVisualLine(boxes: readonly Rect[]): Rect[] {
  const lines: { left: number; top: number; right: number; bottom: number }[] = [];
  for (const { x, y, width, height } of boxes) {
    if (width === 0 && height === 0) continue;
    const box = { left: x, top: y, right: x + width, bottom: y + height };
    const line = lines.find((other) => box.top < other.bottom && other.top < box.bottom);
    if (!line) {
      lines.push(box);
      continue;
    }
    line.left = Math.min(line.left, box.left);
    line.top = Math.min(line.top, box.top);
    line.right = Math.max(line.right, box.right);
    line.bottom = Math.max(line.bottom, box.bottom);
  }
  return lines.sort((a, b) => a.top - b.top || a.left - b.left).map(({ left, top, right, bottom }) => ({ x: left, y: top, width: right - left, height: bottom - top }));
}

/** Where `drew` shows: its first drawing of text that shows, else the innermost element whose range starts there. */
function boxesOf(drew: Drew, document: Document, measure: Measure): Rect[] {
  for (const run of drawings(drew.segments)) {
    const first = run[0];
    const last = run.at(-1);
    if (!first || !last) continue;
    const range = document.createRange();
    range.setStart(first.node, first.from);
    range.setEnd(last.node, last.to);
    const boxes = byVisualLine(measure(range));
    if (boxes.length > 0) return boxes;
  }
  for (const { element } of drew.elements) {
    const boxes = byVisualLine(measure(element));
    if (boxes.length > 0) return boxes;
  }
  return [];
}

/**
 * Each line of the current text that draws something, in line order from `from`, with where it shows in `root`: its
 * boxes from the top left corner of `root`'s content, scrolled or not. Each line is measured when the generator
 * reaches it, so a caller that stops early measures no more.
 */
export function* linesIn(placed: Iterable<Placed>, root: Element, from = 1, measure: Measure = measured): Generator<Line> {
  const lines = drawnByLine(placed);
  for (const line of [...lines.keys()].filter((line) => line >= from).sort((a, b) => a - b)) {
    const drew = lines.get(line);
    if (!drew) continue;
    const boxes = boxesOf(drew, root.ownerDocument, measure);
    if (boxes.length === 0) continue;
    const [corner] = measure(root);
    const dx = root.scrollLeft - (corner?.x ?? 0);
    const dy = root.scrollTop - (corner?.y ?? 0);
    yield { line, rects: boxes.map(({ x, y, width, height }) => ({ x: x + dx, y: y + dy, width, height })) };
  }
}
