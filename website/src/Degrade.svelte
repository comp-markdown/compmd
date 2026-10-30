<!-- One document as CompMD renders it, and as a plain Markdown renderer shows it, switched by a toggle. -->
<script lang="ts">
  import snapshots from "virtual:snapshots";

  const { text, html, plain: plainHtml } = snapshots.degrade;
  let plain = $state(false);
</script>

<div class="toggle" role="group" aria-label="Rendered by">
  <button type="button" aria-pressed={!plain} onclick={() => (plain = false)}>CompMD</button>
  <button type="button" aria-pressed={plain} onclick={() => (plain = true)}>A plain Markdown renderer</button>
</div>
<div class="pair">
  <figure class="manuscript">
    <figcaption>release.comp.md</figcaption>
    <pre>{text}</pre>
  </figure>
  <div class={["proof", plain && "plain"]}>
    {#if plain}
      <div class="markdown">{@html plainHtml}</div>
    {:else}
      <div class="compmd-page">{@html html}</div>
    {/if}
  </div>
</div>
