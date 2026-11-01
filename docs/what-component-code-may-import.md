# What component code may import

**Status: accepted (2026-11-01).**

The tour's idea map is a `js` fence that starts with `import * as d3 from "d3"`. In the playground it draws; on a host that doesn't offer d3 it shows its fallback, and the reader and the author are told only that it "failed". The spec says nothing about this import: it has no import map, no bare imports, no compiling and no libraries a host offers, yet every component the sandbox runs depends on all four. This design says what the spec leaves open, so the spec can say what CompMD does, and changes the code where the spec is right.

## Proposed decisions

1. **A component's code may import four kinds of module, and nothing else** (Q1): its binding's runtime; a library the processor offers; a library pack its pack names (as [packs from a registry](packs-from-a-registry.md), decision 8, defines); and, in a built module, the module's own files in its pack. Anything else resolves nowhere: the component renders its fallback with a new warning, **W121** (decision 5).
2. **A binding's runtime is the processor's, never bundled, and the processor says what it has** (Q2). §6.4 *Built modules* changes from "the libraries it uses bundled in" to "every library it imports bundled in, but its binding's runtime and the libraries it shares". The spec names a runtime by its framework and major version, as today (`react@19`, `svelte@5`), and leaves which of the framework's modules and which minor version to the processor: each processor publishes its runtimes and libraries, with their versions and module names, as its library definition. A pack says what it expects (decision 6), and a processor that doesn't have it says so when it resolves the pack.
3. **A processor MAY offer libraries, as it may offer ambient packs** (Q3). Code that imports one isn't portable. W113, today for ambient names and `host:` data, also covers an offered library being imported (optional, for portable documents).
4. **Source is compiled as the spec's bindings say, and its imports follow decision 1** (Q4). JSX compiles with React's automatic runtime (`react/jsx-runtime`), Svelte for the client; types are stripped. A component's source imports only bare names: a relative import in source has no file to resolve against, so it is W121.
5. **Code that can't load renders its fallback with W121** (Q5): an import that resolves nowhere, a compile error, a built module whose file doesn't match its hash. W118 stays for code that throws while it mounts or updates. A sandbox reports both by their codes, not as a bare "failed".
6. **A pack says what it needs, in `peerDependencies`** (Q6): its binding's runtime, with the lowest version it works with (`"svelte": "^5.40"`), the libraries it expects a processor to offer (`"d3": "^7"`), and library packs (`"@kit/d3": "^7.9"`). A processor checks them when it resolves the pack and reports a missing or too-old one at once (W121, on the import), instead of at render.
7. **A processor states the runtimes it has** (Q7). `preact@10` stays in the spec as an optional runtime; a processor whose sandbox has no Preact reports W102 when it validates, as the spec already says, instead of the sandbox failing at render.
8. **§15 rule 2 says where code comes from as it is:** "Its code and assets come from its pack, its binding's runtime and the libraries the processor offers it or its pack names (section 7.6)". The informative note's policy "allows only the pack's own files" becomes "only the processor's runtimes and libraries and the pack's own files".

## Today

- **The spec:** §7 lists the languages (`js`, `ts`, `jsx`, `tsx`, `svelte`) and runtimes (`react@19`, or `preact@10`; `svelte@5`) by major version. §6.4 says a built module has "the libraries it uses bundled in". §15 rule 2 says a strict component's "code and assets come from its pack". Nothing on imports inside component code; F026 and F035 are about `md.import`, not JavaScript imports. W118 covers code that throws while mounting or updating; nothing covers code that doesn't load.
- **The sandbox:** every frame has one import map, fixed when the frames are built and allowed by its hash in the policy. It holds React 19.3 and React DOM (`react`, `react/jsx-runtime`, `react-dom`, `react-dom/client`) and Svelte 5.57 (`svelte` and 12 of its subpaths, `svelte/internal/client` among them), and whatever `buildLibraries` adds. The compile frame compiles JSX and TSX with sucrase (automatic runtime) and Svelte with its compiler, and leaves imports as written for the map. A compile error, an import the map doesn't have, and a built module that doesn't match its hash all reach the host as reason `"failed"`, with a message and no code. It has no Preact: a `runtime=preact@10` component fails there, though the processor accepts it.
- **The pack template** (`@compmd/create-pack`) leaves `/^(react|react-dom|svelte)(\/|$)/` out of the build: every subpath, including ones the map doesn't have (`svelte/legacy`, `react/jsx-dev-runtime`). Nothing checks a build's imports against the map.
- **Libraries a host offers:** `libraries` in `compmd.config.json` (`@compmd/vite`); the playground and the website offer d3 7.9. Three fences in the tour and the website's `minimermaid.js` import `d3`.
- **The processor** never looks inside component code.

## Q1. What an import in a component may resolve to

| | A. Only the runtime | B. The runtime and what the processor offers | C. B, plus what the pack names (recommended) |
|---|---|---|---|
| The tour's idea map (`d3`) | fails everywhere | works where d3 is offered | the same |
| A pack using a large library (three.js) | must bundle it | must bundle it, or hope the host offers it | names a library pack, shared (packs from a registry, R9) |
| Portability | full | a document says nothing about what it needs | a pack says it (Q6); a document's own fences still don't |

**Recommendation: C.** It is what the code does, plus library packs, already accepted for packs from a registry. A built module's own files (chunks) stay allowed, as the spec allows them today, with "SHOULD be one file" ([packs from a registry](packs-from-a-registry.md), R10).

A document's own fences could say what they need too, for example a `libraries` key in the frontmatter. Not proposed: a document that wants to state its dependencies can move its component into a pack, which can (Q6).

## Q2. A binding's runtime

A built module can't bundle its runtime: two copies of React in one frame break hooks, and a Svelte component compiled against one copy of `svelte/internal/client` can't share state with components compiled against another. So the runtime is the processor's, and a pack has to know what to leave out of its build.

| | R1. The spec lists each runtime's modules | R2. Each processor's library definition (recommended) |
|---|---|---|
| Where the list is | the spec: React 19 is `react`, `react/jsx-runtime`, `react-dom`, `react-dom/client`; Svelte 5 is `svelte` and the client modules its compiler emits (today `svelte/internal/client` and 11 more) | the processor's: `@compmd/sandbox` exports its runtimes and libraries, with versions and module names (today React 19.3, Svelte 5.57, and what the host adds) |
| A new minor version of Svelte, with a new internal module | a spec change | a processor release |
| A pack's build can check it | against the spec | against the processor it targets (the template checks against `@compmd/sandbox`'s) |
| A pack that needs more than a processor has | fails at render | says so in `peerDependencies`; the processor reports it when it resolves the pack (W121), before anything renders |

**Recommendation: R2.** The spec says what a runtime is (the framework, at the major the binding names), that a built module leaves it out, and that a pack states what it needs (Q6); a processor says which modules and versions it has. Svelte's internal modules change with its minor versions, which would otherwise be a spec change every time.

**Is that a worse experience?** Not if the mismatch shows up front. A pack built for Svelte 5.57 on a processor with 5.40: with R1 the spec can't tell the two apart, and the component fails at render with a load error; with R2 the pack's `"svelte": "^5.57"` meets the processor's 5.40 when the import resolves, and the author and reader see W121 naming the runtime and both versions. What R2 loses is a list that holds for every processor, which no pack could rely on anyway, since minor versions differ between processors. A processor SHOULD keep its runtimes at a recent minor.

## Q3. Libraries a processor offers

| | O1. Not allowed | O2. Allowed, not portable, like ambient packs (recommended) | O3. Allowed only through a declaration |
|---|---|---|---|
| The tour, the website | rewritten to bundle d3 or drop it | as they are | each fence declares d3 |
| What a reader on another host sees | the same everywhere | the fallback, with W121 naming d3 | the same as O2 |

**Recommendation: O2.** It matches ambient packs (§6.3): a processor MAY offer them, names from them aren't portable, and W113 warns, for authors who want portable documents. W113's row becomes "Name from an ambient pack, a `host:` data source, or a library the processor offers, used".

## Q4. Compiling source

The spec lists the languages but not what compiling them means, which every processor that runs source must agree on, or a component works on one and not another.

The spec says, in §7.6, per binding:
- **React:** JSX with the automatic runtime, imported from `react/jsx-runtime`; TypeScript's types removed.
- **Svelte:** compiled by Svelte 5's compiler for the client, its styles scoped to the component.
- **Plain:** `js` runs as written; `ts` has its types removed.
- **Imports in source** are bare names, resolved as in Q1. A relative import (`./helpers.js`) is W121: a fence has no file, and a file passed as text has no place to resolve from. (A built module's relative imports are its own files, as today.)

How a processor compiles (in a frame, on a server, ahead of time) stays its own choice, as [the sandbox](running-components-in-the-sandbox.md) design's Q4 decided for `@compmd/sandbox`.

## Q5. Code that can't load

| | E1. Widen W118 | E2. Widen W102 | E3. A new W121 (recommended) |
|---|---|---|---|
| Meaning today | threw while mounting or updating | language or runtime not supported | — |
| Fit | a reader sees the same fallback, but an author needs to know the code never ran | W102 is the processor's capability, known before rendering | one warning: the code couldn't load, with why (the import, the compile error, the hash) |

**Recommendation: E3**, a warning like W102 and W118, since the fallback renders and the rest of the document works. W121: "Component code couldn't load: an import that resolves nowhere, a compile error, or a built module that doesn't match its file; fallback rendered | the invocation, or the import (Q6)". Stage: render, or resolve for a pack's missing `peerDependencies`.

A sandbox reports by code: `@compmd/sandbox`'s errors carry W118 or W121 instead of `"threw"` and `"failed"`, so a host shows them as diagnostics, and the editor next to the processor's.

The processor could also find a bad import before rendering, by scanning the source's imports. Not proposed now: it needs an import scanner for each language in the Rust processor, and a pack's `peerDependencies` (Q6) already catches the common case up front.

## Q6. What a pack says it needs

`peerDependencies` is how npm says "the host provides this"; a pack lists there:

| Entry | Means | A processor without it |
|---|---|---|
| `"react": "^19.2"`, `"svelte": "^5.40"` | its binding's runtime, at least that version | W121 on the import: the runtime is too old |
| `"d3": "^7"` | a library the processor offers | W121 on the import, naming it |
| `"@kit/d3": "^7.9"` | a library pack (packs from a registry, R9) | the resolver gets it from the registry; F026 if it can't |

The import stays: components of the pack that don't use what's missing still work, and those that do render fallbacks with W121. `@compmd/create-pack` writes the entries from what the build left out, so a pack can't forget one.

## Q7. Runtimes a processor has

The spec already says a processor without a runtime reports W102 (§7). The Rust processor accepts `preact@10` because the spec lists it, but `@compmd/sandbox` has no Preact. So the processor's configuration says which runtimes the host runs (by default `react@19` and `svelte@5`), and W102 follows from it, before rendering. Adding Preact to the sandbox is a separate change, if a host wants it.

## What changes

**The spec:**
- §6.4 *Built modules*: what is bundled and what isn't (decision 2).
- §7.6: that a binding's runtime is the processor's, named by framework and major, with its modules and minor version in the processor's library definition (Q2); what compiling its source means (Q4); a new paragraph on what component code may import (Q1, Q3).
- §6.3: offered libraries alongside ambient packs; W113 widened.
- §15 rule 2 and the informative note (decision 8).
- §10.2: W121, its recovery and stages; W113's row.
- `CHANGELOG.md`; conformance cases for W102 with a configured runtime list and for W121 from `peerDependencies`.

**The processor:** the configured runtimes and libraries, with their versions, from the host's library definition (W102); `peerDependencies` checked when a pack resolves (W121); W118 and W121 in its diagnostics, for hosts that report them from the sandbox.

**`@compmd/sandbox`:** errors carry W118 or W121; its library definition (the runtimes and the libraries a host adds, with versions and module names) is exported, for the processor's checks and the template's.

**`@compmd/create-pack`:** externals from the runtime list, not a pattern; the check fails a build whose imports aren't in it; `peerDependencies` written from what was left out.

**`@compmd/vite`:** offered libraries passed to the processor, so W113 and Q6's check know them.

**Docs:** [components from apps and packs](components-from-apps-and-packs.md) (Q3 and S1's *Spec* line, *Today*, *What would change*: the sandbox's code is in `sandbox/`, not the playground's); [running components in the sandbox](running-components-in-the-sandbox.md) (the import map paragraph, and its *Today*).

## Not in this change

- A document declaring the libraries its own fences use (Q1).
- The processor scanning source for imports (Q5).
- Preact in the sandbox (Q7).
- Library packs themselves: [packs from a registry](packs-from-a-registry.md), R9.
