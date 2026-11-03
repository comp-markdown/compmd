# Handling pack loading states

**Status: proposed (2026-11-03).** For review; nothing here is built.

Everything a document needs loads in stage 4a, before anything is evaluated (spec §11), so an import is either there or F026, and data is either loaded or its `fallback`. A host that fetches while a reader reads has a third state: the import is on its way. A pack from a registry ([packs from a registry](packs-from-a-registry.md), decision 7) or remote data an editor fetches after an edit renders as if it had failed, with F026, until it arrives and the document renders again (§8.5). The playground shows it: its `?doc=npm` document lists F026 for `@comp-markdown/mermaid` while the pack is fetched, as if it didn't exist, and each diagram shows its code as if the pack couldn't be had.

## Proposed decisions

1. **Three states, and the host says which.** A source is *loaded* (the request holds it), *pending* (the host is getting it) or *unavailable* (neither). The processor can't know a source is on its way, since it fetches nothing, so the request says it: `pending` lists the packs and the data sources the host is getting. Anything neither held nor pending is unavailable, as today.
2. **Pending is not an error.** A pending import reports nothing: no F026, no F002 for its names' uses. Its names stand for a pending component, which renders its fallback, marked pending (decision 3). A pending data source is `null`, as a failed one is, with no F026. When the host gives up, it processes again without the source in `pending`, and the source is unavailable: F026, as today. The processor never times out; the host decides how long to wait (a registry error ends it at once).
3. **A reader can tell loading from failed.** A use of a pending component is its fallback inside a new expanded node, `componentPending`, with the component's name and its pack. `@compmd/render` draws it as the fallback in a box with `aria-busy="true"` and the class `compmd-pending`, which a host may style (the default is quiet: the fallback, a little faded). An unavailable component shows its fallback as today, with its diagnostic. A plain renderer shows a pending use as it shows the use anyway, since the fallback is the same text.
4. **Data is pending without a mark.** A pending data import has no node of its own to mark: an `each` over it renders its `empty` slot, as for `null` data. The host knows what it is getting, and the document renders again when it arrives. Marking what depends on pending data would mean following values through expressions; it isn't worth it now.
5. **Hosts and components learn it from the tree and the host's own loop.** The processor's result gives `pending` back: what it treated as pending, for a host's UI ("loading 1 pack"). A host-language component sees a pending use in its slot as an item whose `component` has `pending: true`, so a tabs component can still show a tab's label from its params while its content loads.
6. **The sandbox says when a component recovers, by use.** The sandbox's `error` event names a component, not a use, so a host that lists failures can't match a later recovery to the failure it listed. Each event gains `use`, an id the sandbox gives each mounted use, and a new `recovered` event, `{ component, use }`, comes when a use that failed draws again: after an `update` that follows `fail`, or a remount after a throw.
7. **Spec changes are small and optional for processors.** §11 and §6.4 gain the pending state as something a processor's host may give; a processor that never fetches while rendering has none, and nothing changes for it. §4.5 says a pending use renders its fallback marked pending; §12.3 gains `componentPending`; §10.2 says F026 isn't reported for a pending source; §8.5 already renders again when a source arrives.

## Today

- **The processor** takes packs and data only in the request. `sources(request)` lists the remote URLs, `host:` sources and packs (`NeededPack`) a render needs that the request doesn't hold. An import whose pack isn't held is F026 (*not provided*), and its names' uses render their fallbacks without F002 (§10.3). Remote data the request doesn't hold is F026 and `null`, or its `fallback`.
- **A host that fetches** (`@compmd/registry`'s `resolvePacks`, an editor's `OpenDocument.sources()` then `reload`) renders first with what it has, then again with what arrived. In between, the reader sees fallbacks, and the diagnostics list F026s that will go away.
- **The sandbox** reports a component that shows its fallback as `error`, `{ component, reason, code?, message }`, and reports nothing when it draws again.

## Q1. Who knows a source is pending

| | P1. The request says (recommended) | P2. The processor guesses from `sources` | P3. The host filters diagnostics |
|---|---|---|---|
| How | `pending: { packs: ["@kit/mermaid"], remote: [...], host: [...] }` in the request | anything `sources` lists is pending, until the host says it failed | the processor is unchanged; the host hides F026s for what it is fetching |
| A source that can't be had | unavailable once the host leaves it out of `pending`: F026 | needs a second signal, "failed", per source | F026, shown again |
| What a reader sees while loading | marked pending (decision 3) | marked pending | the fallback, as if failed: nothing to mark it by |
| A processor that never fetches | nothing changes | every missing source looks pending forever | nothing changes |

P2 turns every typo in an import into "loading" until something says otherwise, and P3 can't tell a reader anything. P1 puts the state where it is known: in the host's loop.

## Q2. What a pending use is in the expanded tree

| | N1. A `componentPending` node around the fallback (recommended) | N2. A `pending` flag on the nodes the fallback makes | N3. The use kept, as a component node with `pending: true` |
|---|---|---|---|
| A container's fallback (its slots as Markdown, several blocks) | one node holding them all | the flag on each block | the use with its slots, which renderers render as fallback |
| What a renderer that doesn't know it does | renders the children (an unknown node with children, as for any node it doesn't know) | renders the fallback, unmarked | needs the component, or shows nothing: a break for renderers |
| Telling it apart from an unavailable use | yes | yes, block by block | yes |
| Matching it to the use when it arrives (keys, origins) | the node has the use's origin, so the renderer's keys move the use in place | the blocks' origins | the use's origin |

N1 keeps the fallback whole and the meaning in one place. It is a new node type, which the tree schema and `@compmd/render` learn; a renderer that doesn't know it renders its children, so the fallback still shows.

## Q3. What the reader sees

- **Pending:** the fallback, in a box with `aria-busy="true"` (screen readers announce it as busy, and announce the change when it renders), the class `compmd-pending`, and by default a little faded. No spinner: the fallback is useful text, and a document with ten diagrams shouldn't spin ten times.
- **Unavailable:** the fallback as today, unfaded, and the diagnostic in the host's list.
- **When it arrives:** the use draws in place; with origins, the renderer's keys keep the rest of the page as it was.

For the playground's `npm` document: while `@comp-markdown/mermaid` loads, each diagram shows its code, faded and busy, and the diagnostics list nothing; once it arrives, the diagrams draw. With `@comp-markdown/mermaid@9`, which npm doesn't have, the resolver fails at once: the blocks show their code unfaded, and F026 says why.

## Q4. When a host gives up

The processor never decides that a pending source has failed. The host does: a registry that answers "no such version", a network error, or a time the host chooses for a request that never answers. `@compmd/registry`'s resolver ends a pack's pending state on the first two, and takes a timeout option for the third (30 seconds unless the host says otherwise).

## Q5. A component that recovers

| | R1. `use` ids and a `recovered` event (recommended) | R2. `recovered` by component name | R3. A full list of current failures on each change |
|---|---|---|---|
| A host that lists failures | removes the one with that `use` | removes all of that component's, even those still failing | replaces its list |
| Messages | one per recovery | one per recovery | one list per render, mostly unchanged |
| What it needs | an id per mounted use, in the sandbox | nothing new | the sandbox to keep every use's state |

R1 matches each recovery to its failure. The id is the sandbox's, opaque, and stable for a use as long as it stays mounted (a keyed node that moves keeps it).

## What changes

**The spec:** §6.4 and §11: a processor's host may say a pack or data source is pending; §4.5: a pending use renders its fallback, marked pending; §10.2: F026 isn't reported for a pending source; §12.3 and the tree schema: `componentPending`, with `name`, `pack` and `children`; the host contract (§7.4): a slot item's `component` may have `pending: true`.

**The processor:** the request's `pending`; pending packs bound as pending, their uses expanded as `componentPending`; pending data `null` without F026; the result's `pending`.

**`@compmd/registry`:** `resolvePacks` gives what it is still fetching, for the request's `pending`, and ends a pack's pending state on failure or after its timeout.

**`@compmd/render`:** `componentPending` drawn as its children in a busy box (`compmd-pending`, `aria-busy`), with the default style; a slot item for a pending use with `pending: true`.

**`@compmd/sandbox`:** `use` on `error`, and the `recovered` event.

**The playground and the editor:** pass `pending` while the resolver works, so the `npm` document shows loading, not failure; the editor's diagnostics list stays clean while packs load.

**The guides:** the Developer's Guide's *Packs from npm* and *Rendering and styling* say how a host shows loading, and the diagnostics appendix that F026 waits for a host to give up.

## Not in this change

- Marking what depends on pending data (decision 4).
- Progress (bytes of a pack's tarball): a host's UI can show it from its own fetch.
- A pending state for a component's own code once its pack has arrived: compiling and loading a module in a frame is the sandbox's, and shows as today.
