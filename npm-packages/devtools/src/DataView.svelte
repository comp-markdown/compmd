<!-- What the document's frontmatter gives its render: its own fields (`md.doc`), its settings, each data import with
     what was loaded, and the inputs with their values. -->
<script lang="ts">
  import type { Frontmatter } from "@compmd/processor";
  import JsonTree from "./JsonTree.svelte";

  let { frontmatter }: { frontmatter: Frontmatter } = $props();

  /** How much an import loaded, in a few words. */
  function loaded(value: unknown): string {
    if (value === null) return "not loaded; the diagnostics say why";
    if (Array.isArray(value)) return `${String(value.length)} ${value.length === 1 ? "record" : "records"}`;
    if (typeof value === "object") return `${String(Object.keys(value).length)} fields`;
    return "one value";
  }

  const fields = $derived(Object.keys(frontmatter.doc).length);
  const settings = $derived(Object.keys(frontmatter.settings).length);
  const inputs = $derived(Object.keys(frontmatter.inputs).length);
</script>

<div class="data">
  <section>
    <h2>Fields <span>what <code>md.doc</code> holds</span></h2>
    {#if fields > 0}
      <JsonTree value={frontmatter.doc} open={1} />
    {:else}
      <p class="empty">The frontmatter has no fields of its own.</p>
    {/if}
  </section>

  <section>
    <h2>Data <span>what <code>md.data</code> holds, by import</span></h2>
    {#each frontmatter.data as data (data.name)}
      <article class="import">
        <h3>
          <code>{data.name}</code>
          <span>{loaded(data.value)} from <code>{data.from}</code>{#if data.fallback}, or <code>{data.fallback}</code> when that fails{/if}</span>
        </h3>
        {#if data.schema}
          <p class="schema">
            {#each data.schema as [field, type] (field)}
              <code>{field}: {typeof type === "string" ? type : JSON.stringify(type)}</code>
            {/each}
          </p>
        {/if}
        {#if data.value !== null}
          <JsonTree value={data.value} open={1} />
        {/if}
      </article>
    {:else}
      <p class="empty">The document imports no data.</p>
    {/each}
  </section>

  <section>
    <h2>Inputs <span>what <code>md.input</code> holds, with the values this render used</span></h2>
    {#if inputs > 0}
      <JsonTree value={frontmatter.inputs} open={2} />
    {:else}
      <p class="empty">The document declares no inputs.</p>
    {/if}
  </section>

  {#if settings > 0}
    <section>
      <h2>Settings <span>the rest of <code>md</code></span></h2>
      <JsonTree value={frontmatter.settings} open={2} />
    </section>
  {/if}
</div>
