<!-- A component that throws on a list with an empty item, as one does while its author is still typing the list. It
     keeps its `compmd` prop where a test can call its `fail`. -->
<script lang="ts">
  import { seen } from "./seen.ts";

  let { plots, compmd }: { plots: ({ number: number } | null)[]; compmd: { fail(message: string): void } } = $props();
  $effect.pre(() => {
    seen.compmd = compmd;
  });
</script>

<ol>
  {#each plots as plot, i (i)}
    <li>{plot!.number}</li>
  {/each}
</ol>
