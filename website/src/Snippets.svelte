<!-- The tabs of "Use it in your app", each a few files of code that runs: the guide's quick start, the site's own pack
     (what renders the live `stars` below), and the editor's snippet, which a test runs. -->
<script lang="ts">
  import mainTs from "../../guide/examples/quick-start/main.ts?raw";
  import viteConfig from "../../guide/examples/quick-start/vite.config.ts?raw";
  import appDocument from "../app/index.comp.md?raw";
  import stars from "../app/Stars.svelte?raw";
  import config from "../compmd.config.json?raw";
  import editor from "../snippets/editor.ts?raw";

  const SNIPPETS = [
    {
      id: "vite",
      label: "In a Vite app",
      about: "The plugin gives the processor your packs and serves the sandbox; the page opens a document and renders it in a sandboxed frame.",
      files: [
        { name: "vite.config.ts", code: viteConfig },
        { name: "main.ts", code: mainTs },
      ],
    },
    {
      id: "components",
      label: "Your own components",
      about: "Your components are a pack folder: a signature for each, and one line of configuration that trusts them. This site's, below.",
      files: [
        { name: "compmd.config.json", code: config },
        { name: "app/index.comp.md", code: appDocument },
        { name: "app/Stars.svelte", code: stars },
      ],
    },
    {
      id: "editor",
      label: "In an editor",
      about: "Keep a document open: each edit processes and renders only the blocks it touches.",
      files: [{ name: "editor.ts", code: editor }],
    },
  ];

  let chosen = $state(SNIPPETS[0]?.id);
</script>

<div class="tabs" role="tablist" aria-label="Snippets">
  {#each SNIPPETS as snippet (snippet.id)}
    <button
      type="button"
      role="tab"
      id="tab-{snippet.id}"
      aria-controls="snippet-{snippet.id}"
      aria-selected={chosen === snippet.id}
      tabindex={chosen === snippet.id ? 0 : -1}
      onclick={() => (chosen = snippet.id)}>{snippet.label}</button>
  {/each}
</div>
{#each SNIPPETS as snippet (snippet.id)}
  <div class="snippet" role="tabpanel" id="snippet-{snippet.id}" aria-labelledby="tab-{snippet.id}" hidden={chosen !== snippet.id}>
    <p>{snippet.about}</p>
    {#each snippet.files as file (file.name)}
      <figure>
        <figcaption>{file.name}</figcaption>
        <pre><code>{file.code}</code></pre>
      </figure>
    {/each}
  </div>
{/each}
