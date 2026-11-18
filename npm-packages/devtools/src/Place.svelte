<!-- A place in the document's text: a button that selects it there, or, with no text to select it in (the inspector),
     the place as text. -->
<script lang="ts">
  import type { Range } from "@compmd/processor";
  import type { Snippet } from "svelte";

  let { range, title, onselect, children, class: kind = "place" }: { range: Range; title: string; onselect: ((range: Range) => void) | undefined; children: Snippet; class?: string } = $props();
</script>

{#if onselect}
  <button type="button" class={kind} {title} onclick={() => onselect(range)}>{@render children()}</button>
{:else}
  <span class="{kind} still">{@render children()}</span>
{/if}
