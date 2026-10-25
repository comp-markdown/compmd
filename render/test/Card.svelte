<script lang="ts">
  import type { Snippet } from "svelte";

  let { title, footer, children, compmd }: { title: string; footer?: Snippet; children?: Snippet; compmd: { headingLevel: number } } = $props();
  let clicks = $state(0);
</script>

<section>
  <h3 data-level={compmd.headingLevel}>{title}</h3>
  <button onclick={() => clicks++}>{clicks}</button>
  {@render children?.()}
  {@render footer?.()}
</section>
