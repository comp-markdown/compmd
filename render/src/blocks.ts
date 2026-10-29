// The page's blocks: the expanded tree's top-level nodes, each rendered once and kept until a splice replaces it. A
// splice (a processor's, or a whole tree as one) replaces some blocks and moves the ones after them; a render then
// renders the blocks not shown yet, reusing the views of those that went, and updates the others only where what the
// whole document has changed for them: footnote numbers, link definitions, the inputs. What only the whole document
// has comes from each block's summary, so a render costs what the blocks it renders cost, plus a little per block.
import { footnotesFrom, linksFrom, notesOf, type Footnotes, type LinkDefinition, type Notes } from "./document.ts";
import { stillOpen, tagsOf, withHtml, type Tag } from "./html.ts";
import { Keys, type Edit, type KeyOf, type Keyed } from "./keys.ts";
import { destroyAll, discard, Matcher, placedIn, viewsFor, type Pass, type Surface, type View } from "./render.ts";
import { shownOf, type Diagnostic, type Good, type Rules } from "./stable.ts";
import { fills, usesBreakpoints } from "./styles.ts";
import { walk, type ExpandedInput, type ExpandedNode, type Splice } from "./tree.ts";

/** What the page needs to know of a shown block without walking it again. */
interface Summary {
  notes: Notes;
  /** The depth of its last heading, which the heading level of components after it follows. */
  heading: number | undefined;
  /** Whether blocks it places in the page use breakpoints, and whether one fills the page. */
  container: boolean;
  fills: boolean;
  /** The tags of its raw HTML, which can pair with other blocks'. */
  tags: Tag[];
}

/** What a block shows (its node, or what the rules for invalid text say), with keys, and its node's ids. */
interface Shown extends Keyed {
  nodes: ExpandedNode[];
  summary: Summary;
}

interface Block {
  node: ExpandedNode;
  /** The code points and lines its origins are behind the text by: blocks before it changed since it came. */
  shift: number;
  lines: number;
  /** Absent until a render shows it. */
  shown?: Shown;
  /** Absent until a render renders it, and after a change to the blocks it renders with. */
  group?: Group;
}

/** Blocks rendered together, with their views: one block, or those whose raw HTML pairs across them. */
interface Group {
  blocks: Block[];
  views: View[];
}

const NOTHING: Shown = { nodes: [], ids: new Map(), keys: new Map(), summary: { notes: notesOf([]), heading: undefined, container: false, fills: false, tags: [] } };

const shown = (block: Block): Shown => block.shown ?? NOTHING;

function summaryOf(nodes: readonly ExpandedNode[]): Summary {
  let heading: number | undefined;
  walk(nodes, (node) => {
    if (node.type === "heading") heading = Number(node.depth);
  });
  const placed = placedIn(nodes);
  return {
    notes: notesOf(nodes),
    heading,
    container: placed.some((child) => usesBreakpoints(child.params)),
    fills: placed.some((child) => fills(child.params)),
    tags: tagsOf(nodes),
  };
}

function keyOfBlocks(blocks: readonly Block[]): KeyOf {
  const [only] = blocks;
  if (only && blocks.length === 1) return (node) => shown(only).keys.get(node);
  return (node) => {
    for (const block of blocks) {
      const key = shown(block).keys.get(node);
      if (key !== undefined) return key;
    }
    return undefined;
  };
}

const element = (tag: string, attributes: Record<string, string>, content: { text?: string; children?: ExpandedNode[] }): ExpandedNode => ({ type: "compmdElement", tag, attributes, ...content });

/** The footnotes' list, as GitHub shows it: a rule, then each definition with a link back to its first reference. */
function footnotesList({ definitions }: Footnotes): ExpandedNode[] {
  if (definitions.length === 0) return [];
  const items = definitions.map(({ number, children }) =>
    element("li", { id: `compmd-fn-${String(number)}` }, {
      children: [...children, element("a", { href: `#compmd-fnref-${String(number)}`, class: "compmd-footnote-back", "aria-label": `Back to reference ${String(number)}` }, { text: "↩" })],
    }),
  );
  return [element("section", { class: "compmd-footnotes" }, { children: [element("hr", {}, {}), element("ol", {}, { children: items })] })];
}

const sameNumbers = (a: ReadonlyMap<string, number>, b: ReadonlyMap<string, number>): boolean => a.size === b.size && [...a].every(([identifier, number]) => b.get(identifier) === number);

const sameLinks = (a: ReadonlyMap<string, LinkDefinition>, b: ReadonlyMap<string, LinkDefinition>): boolean =>
  a.size === b.size && [...a].every(([identifier, { url, title }]) => b.get(identifier)?.url === url && b.get(identifier)?.title === title);

const sameFootnotes = (a: Footnotes["definitions"], b: Footnotes["definitions"]): boolean =>
  a.length === b.length && a.every((definition, i) => definition.identifier === b[i]?.identifier && definition.number === b[i].number && definition.children === b[i].children);

/** The page's blocks in `root`, which they fill. */
export class Blocks {
  readonly #root: HTMLElement;
  readonly #surface: Surface;
  readonly #keys = new Keys();
  #blocks: Block[] = [];
  /** The views of blocks that went or render again, for the next render to reuse. */
  #released: View[] = [];
  /** The last good renders of blocks that went, by key (rule 3 of `stable.ts`). */
  readonly #good = new Map<string, Good>();
  #inputs: Record<string, ExpandedInput> = {};
  #inputsChanged = false;
  #footnotes: Footnotes = { numbers: new Map(), definitions: [] };
  #footnoteViews: View[] = [];
  #links: ReadonlyMap<string, LinkDefinition> = new Map();

  constructor(root: HTMLElement, surface: Surface) {
    this.#root = root;
    this.#surface = surface;
  }

  /** How many blocks there are. */
  get length(): number {
    return this.#blocks.length;
  }

  /** Applies `splice`, which `edit` made, to the blocks; what it changes shows at the next {@link render}. */
  apply(splice: Splice, edit: Edit | undefined): void {
    const added = splice.nodes.map((node): Block => ({ node, shift: 0, lines: 0 }));
    for (const gone of this.#blocks.splice(splice.from, splice.to - splice.from, ...added)) this.#release(gone);
    if (edit) this.#keys.edit(edit);
    if (splice.offset !== 0 || splice.lines !== 0) {
      for (let i = splice.from + added.length; i < this.#blocks.length; i++) {
        const block = this.#blocks[i];
        if (!block) continue;
        block.shift += splice.offset;
        block.lines += splice.lines;
      }
    }
    const inputs = splice.inputs ?? {};
    if (JSON.stringify(inputs) !== JSON.stringify(this.#inputs)) {
      this.#inputs = inputs;
      this.#inputsChanged = true;
    }
  }

  /** Renders the blocks not shown yet, by the rules for invalid text with `errors` in `text`, and what depends on them. */
  render(errors: readonly Diagnostic[], text: string | undefined): void {
    const rules: Rules = { errors, text, good: this.#good };
    for (const block of this.#blocks) block.shown ??= this.#show(block, rules);
    const segments = this.#segments();
    for (const segment of segments) {
      const [first] = segment;
      const group = first?.group;
      const rendered = group !== undefined && group.blocks.length === segment.length && segment.every((block) => block.group === group);
      if (!rendered) for (const block of segment) if (block.group) this.#dissolve(block.group);
    }
    const parts = this.#blocks.map((block) => shown(block).summary.notes);
    const footnotes = footnotesFrom(parts);
    const links = linksFrom(parts);
    const renumbered = !sameNumbers(footnotes.numbers, this.#footnotes.numbers);
    const relinked = !sameLinks(links, this.#links);
    const listed = sameFootnotes(footnotes.definitions, this.#footnotes.definitions);
    this.#footnotes = footnotes;
    this.#links = links;
    const heading = { level: 0 };
    const passOf = (blocks: readonly Block[]): Pass => ({ inputs: this.#inputs, heading, footnotes: footnotes.numbers, definitions: links, keyOf: keyOfBlocks(blocks), parent: "block" });
    const released = new Matcher(this.#released, (view) => view);
    this.#released = [];
    for (const segment of segments) {
      const group = segment[0]?.group;
      const nodes = () => withHtml(segment.flatMap((block) => shown(block).nodes));
      const summaries = segment.map((block) => shown(block).summary);
      if (!group) {
        const made: Group = { blocks: segment, views: viewsFor(this.#surface, released, nodes(), passOf(segment)) };
        for (const block of segment) block.group = made;
      } else if (this.#inputsChanged || (renumbered && summaries.some((s) => s.notes.references.length > 0)) || (relinked && summaries.some((s) => s.notes.usesLinks))) {
        const own = new Matcher(group.views, (view) => view);
        group.views = viewsFor(this.#surface, own, nodes(), passOf(segment));
        discard(own.rest());
      } else {
        for (const summary of summaries) if (summary.heading !== undefined) heading.level = summary.heading;
      }
    }
    // Before the views are placed: a view left between two kept ones would move one, which loses its focus.
    discard(released.rest());
    this.#inputsChanged = false;
    if (renumbered || !listed) {
      const own = new Matcher(this.#footnoteViews, (view) => view);
      this.#footnoteViews = viewsFor(this.#surface, own, footnotesList(footnotes), passOf(this.#blocks.filter((block) => shown(block).summary.notes.footnotes.length > 0)));
      discard(own.rest());
    }
    this.#place(segments);
    this.#keys.settle();
    this.#good.clear();
    this.#root.classList.toggle("compmd-container", this.#blocks.some((block) => shown(block).summary.container));
    this.#root.classList.toggle("compmd-fills", this.#blocks.some((block) => shown(block).summary.fills));
  }

  destroy(): void {
    for (const block of this.#blocks) if (block.group?.blocks[0] === block) destroyAll(block.group.views);
    destroyAll(this.#released);
    destroyAll(this.#footnoteViews);
    this.#root.replaceChildren();
  }

  #show(block: Block, rules: Rules): Shown {
    const keyed = this.#keys.assign([block.node], block.shift);
    const nodes = shownOf(block.node, block, keyed.keys, rules);
    return { ...keyed, nodes, summary: summaryOf(nodes) };
  }

  #release({ shown: was, shift, group }: Block): void {
    if (group) this.#dissolve(group);
    if (!was) return;
    this.#keys.release(was.ids, shift);
    for (const node of was.nodes) {
      const key = was.keys.get(node);
      if (key !== undefined) this.#good.set(key, { node, keys: was.keys });
    }
  }

  /** Releases a group's views, so its blocks render again. */
  #dissolve(group: Group): void {
    this.#released.push(...group.views);
    for (const block of group.blocks) delete block.group;
  }

  /** The blocks in runs that render together: each block alone, except where raw HTML pairs across blocks. */
  #segments(): Block[][] {
    const segments: Block[][] = [];
    let open: string[] = [];
    let segment: Block[] = [];
    for (const block of this.#blocks) {
      segment.push(block);
      const { tags } = shown(block).summary;
      if (tags.length > 0) open = stillOpen(open, tags);
      if (open.length === 0) {
        segments.push(segment);
        segment = [];
      }
    }
    if (segment.length > 0) segments.push(segment);
    return segments;
  }

  /** Puts every view in order, from the end, moving only those out of place. */
  #place(segments: readonly Block[][]): void {
    let next: Node | null = null;
    const place = (views: readonly View[]) => {
      for (let i = views.length - 1; i >= 0; i--) {
        const dom = views[i]?.dom;
        if (!dom) continue;
        if (dom.parentNode !== this.#root || dom.nextSibling !== next) this.#root.insertBefore(dom, next);
        next = dom;
      }
    };
    place(this.#footnoteViews);
    for (let i = segments.length - 1; i >= 0; i--) place(segments[i]?.[0]?.group?.views ?? []);
  }
}
