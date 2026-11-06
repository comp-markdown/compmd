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
  // The site's own header and footer, so the playground is one of its pages.
  import Footer from "../../website/src/Footer.svelte";
  import Header from "../../website/src/Header.svelte";

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

  /** The CompMD website, which the playground is part of, at `playground/`: relative, so the links work wherever the site is published. */
  const SITE = "../";
</script>

<Header root={SITE} current="playground" />

<div class="toolbar">
  <div class="title">
    <h1>Playground</h1>
    <p class="tagline">Write CompMD on the left, see the page on the right.</p>
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
</div>

<main>
  <div class="tools" {@attach (element) => untrack(() => attach(element))}></div>
</main>

<Footer />
