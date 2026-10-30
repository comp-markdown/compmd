<!-- The snippets of "Use it in your app", one tab each. Their code is the files in `snippets/`, which tests run. -->
<script lang="ts">
  import editor from "../snippets/editor.ts?raw";
  import page from "../snippets/page.ts?raw";

  const SNIPPETS = [
    { id: "page", label: "Render a page", about: "Process a document and render it into an element.", code: page },
    { id: "editor", label: "Keep it open in an editor", about: "Each edit processes and renders only the blocks it touches.", code: editor },
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
    <pre><code>{snippet.code}</code></pre>
  </div>
{/each}
