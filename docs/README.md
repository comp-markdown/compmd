# Design documents

The designs behind the spec and its implementation. Each starts with its status and leads with its decisions. A topic design decides *what* CompMD does and *why*; a build design decides *how* one package does a part of a topic design, and names the part it builds.

## Topics

- [Components from apps and packs](components-from-apps-and-packs.md): where component code comes from (an app, a pack, the registry), and at what trust; the parent of most designs below.
- [Packs from a registry](packs-from-a-registry.md): how a host lets documents use packs straight from an npm registry, what a registry pack is, and how packs share libraries.
- [What component code may import](what-component-code-may-import.md): what a bare import in a component resolves to: a runtime, a library the host offers, or a library pack.
- [Code renderers](code-renderers.md): a component that also draws the code blocks of one language (spec §7.7).
- [Handling pack loading states](handling-pack-loading-states.md): a placeholder, for telling a pack on its way from one that won't load.

## How the packages build them

- [Running components in the sandbox](running-components-in-the-sandbox.md): how `@compmd/sandbox` runs code at the three trust levels; builds the levels of *components from apps and packs*.
  - [The sandbox's frame protocol](sandbox-protocol.md): the messages between its windows.
- [The configuration and `@compmd/vite`](vite.md): `compmd.config.json` and the Vite plugin, for packs an app installs; builds H3 of *components from apps and packs*.
- [`@compmd/create-pack`, the pack template](create-pack.md): `npm create @compmd/pack`; builds S2 of *components from apps and packs*.
- [Resolving registry packs](resolving-registry-packs.md): how `@compmd/registry` resolves, fetches and checks packs; builds decisions 1 to 3 and 5 of *packs from a registry*.
- [Sharing library packs](sharing-library-packs.md): how the processor, `@compmd/registry` and `@compmd/sandbox` let packs share a library pack; builds decision 8 of *packs from a registry*.
- [The Mermaid pack](mermaid-pack.md): `@comp-markdown/mermaid`, a code renderer as a registry pack; builds the Mermaid example of *code renderers*.

The processor's own designs are next to it, in [`processor/docs/`](../processor/docs/): [incremental processing](../processor/docs/incremental.md), [param values](../processor/docs/param-values.md) and [positional hints](../processor/docs/positional-hints.md).

## The repository

- [CI on GitHub](ci.md): the checks every push and pull request runs.
- [Releases](releases.md): what is released, how it is versioned, and what a release does.
