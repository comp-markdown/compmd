<!-- The playground: the CompMD website's page that embeds the Developer Tools (`@compmd/devtools`). It keeps what is the
     site's: the header with the menu of documents and the host's stylesheets the author can try, the About, the links
     and the footer. The tools open the chosen document, with the playground's sandbox and its registry. -->
<script lang="ts">
  import { mountDevtools, type DevtoolsHandle, type Registries } from "@compmd/devtools";
  import { NpmRegistry } from "@compmd/registry";
  import { untrack } from "svelte";
  import { registry, sandbox } from "virtual:compmd";
  import { EXAMPLES, exampleIndex } from "./documents.ts";
  import { keep, recall } from "./stored.ts";
  import { HOST_STYLES, stylesheetOf, type HostStyle } from "./styles.ts";
  import markDark from "../../logo/mark-dark.svg";
  import markLight from "../../logo/mark-light.svg";

  let chosen = $state(exampleIndex(new URLSearchParams(location.search).get("doc")));
  const example = $derived(EXAMPLES[chosen]);

  /** The host's stylesheet the page shows in, remembered between visits. */
  const STYLE_KEY = "compmd-editor-host-style";
  let hostStyle: HostStyle = $state(HOST_STYLES.find(({ id }) => id === recall(STYLE_KEY))?.id ?? "reference");

  /** The registries the configuration names, which the tools fetch the packs a document imports from. */
  const registries: Registries | undefined = registry && {
    registries: Object.fromEntries(Object.entries(registry.registries).flatMap(([scope, each]) => (each instanceof NpmRegistry ? [[scope, each.url]] : []))),
    fetch: registry.fetch,
    ...(registry.maxPackSize === undefined ? {} : { maxPackSize: registry.maxPackSize }),
  };

  /** The tools, in their element, with the document `?doc=` names and the passage `?select=` names. */
  let tools: DevtoolsHandle | undefined;
  function attach(element: HTMLElement): () => void {
    const first = EXAMPLES[chosen];
    if (!first) return () => undefined;
    const select = /[?&]select=([^&]*)/.exec(location.search)?.[1];
    tools = mountDevtools(element, {
      sandbox,
      document: first,
      ...(registries ? { registries } : {}),
      stylesheets: stylesheetOf(hostStyle),
      ...(select === undefined ? {} : { select }),
      shareLink: (passage) => `${location.origin}${location.pathname}?doc=${example?.id ?? ""}&select=${passage}`,
    });
    // For the bench, which reads the editor's timings.
    (window as { compmdEditor?: DevtoolsHandle }).compmdEditor = tools;
    return () => {
      tools?.destroy();
      tools = undefined;
    };
  }

  $effect(() => {
    keep(STYLE_KEY, hostStyle);
    tools?.styles(stylesheetOf(hostStyle));
  });

  /** What the page is for: open on the first visit, until it is closed. */
  const ABOUT_KEY = "compmd-editor-about";
  let about = $state(recall(ABOUT_KEY) !== "closed");
  function toggleAbout(): void {
    about = !about;
    keep(ABOUT_KEY, about ? "open" : "closed");
  }

  /**
   * The CompMD website, which the playground is part of, at `playground/`: relative, so the links work wherever the
   * site is published. The repository.
   */
  const SITE = "../";
  const REPOSITORY = "https://github.com/comp-markdown/compmd";
</script>

<header>
  <div class="title">
    <a class="home" href={SITE} title="The CompMD website">
      <picture class="logo">
        <source media="(prefers-color-scheme: dark)" srcset={markDark} />
        <img src={markLight} alt="" width="40" height="26" />
      </picture>
      <h1>CompMD Playground</h1>
    </a>
    <button type="button" class="about-toggle" aria-label="About" title="About" aria-expanded={about} aria-controls="about" onclick={toggleAbout}>i</button>
  </div>
  <label class="document">
    <span>Document</span>
    <select
      bind:value={chosen}
      onchange={() => {
        if (example) tools?.open(example);
      }}
    >
      {#each EXAMPLES as each, i (each.name)}
        <option value={i}>{each.name}</option>
      {/each}
    </select>
  </label>
  <label class="document" title="The page's look is its host's, not the document's: try the stylesheets a host might give it.">
    <span>Host stylesheet</span>
    <select bind:value={hostStyle}>
      {#each HOST_STYLES as { id, label } (id)}
        <option value={id}>{label}</option>
      {/each}
    </select>
  </label>
  <nav class="links" aria-label="CompMD">
    <a href="{SITE}spec/">Spec</a>
    <a href={REPOSITORY} target="_blank" rel="noreferrer">GitHub</a>
  </nav>
</header>

{#if about}
  <section class="about" id="about" aria-label="About the playground">
    <p>
      Write a <a href="https://github.com/comp-markdown/compmd" target="_blank" rel="noreferrer">CompMD</a> document on the left and see it
      rendered on the right as you type: components, data, layout and all. The tabs show what went wrong, the components the document can
      name and where it uses them, the sandbox's frames, what the frontmatter gives the page, what a plain Markdown renderer makes of the
      same text, the messages between the frames, and the trees the processor builds. Beside the document, try the stylesheets a host might
      give the page; above the tabs, other values for the document's inputs. Under the text, a line says what is at the caret and what it
      takes, with its values to click, and the diagnostics point at what's wrong and offer fixes. The document never leaves your browser.
      When it imports a pack the playground doesn't have, your browser fetches the pack from npm, which then learns your address and the
      pack's name and version: never the document, or what it shows. When it loads data from a site, the playground asks you first, the
      first time, and remembers your answer; that site then learns your address and what the document asks it for.
      {#if __COMMIT__}
        Built from commit <a href="{REPOSITORY}/commit/{__COMMIT__}" target="_blank" rel="noreferrer"><code>{__COMMIT__.slice(0, 7)}</code></a>.
      {/if}
    </p>
    <button type="button" class="close" aria-label="Close" onclick={toggleAbout}>
      <svg viewBox="0 0 12 12" aria-hidden="true"><path d="m3 3 6 6m0-6-6 6" /></svg>
    </button>
  </section>
{/if}

<main>
  <div class="tools" {@attach (element) => untrack(() => attach(element))}></div>
</main>

<footer class="statusbar">
  <span>Copyright © 2026 Hakan Serce.</span>
  <span>The editor is <a href="https://github.com/comp-markdown/compmd/blob/main/LICENSE-CODE" target="_blank" rel="noreferrer">MIT licensed</a>.</span>
</footer>
