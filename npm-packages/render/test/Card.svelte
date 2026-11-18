<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, sideNote, children, compmd }: { title: string; sideNote?: Snippet; children?: Snippet; compmd: { headingLevel: number } } = $props();
  let clicks = $state(0);
</script>

<section>
  <h3 data-level={compmd.headingLevel}>{title}</h3>
  <button onclick={() => clicks++}>{clicks}</button>
  {@render children?.()}
  {@render sideNote?.()}
</section>
