<!-- JSON as a tree that folds: each object and array has a toggle, and a folded one shows a short summary (a tree
     node's type and its telling fields, a list's length). Folded children aren't rendered, so a large tree costs what
     is open. Levels above `open` start open. -->
<script lang="ts">
  import JsonTree from "./JsonTree.svelte";
  import { entriesOf, isBranch, leafText, summaryOf } from "./json.ts";

  let { value, name, depth = 0, open = 2 }: { value: unknown; name?: string; depth?: number; open?: number } = $props();

  let expanded = $state(false);
  $effect.pre(() => {
    expanded = depth < open;
  });
</script>

{#if isBranch(value)}
  <div class="branch">
    <button type="button" class="toggle" aria-expanded={expanded} onclick={() => (expanded = !expanded)}>
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="m3 2 4 3-4 3" /></svg>
      {#if name !== undefined}<span class="key">{name}</span>{/if}
      <span class="summary">{summaryOf(value)}</span>
    </button>
    {#if expanded}
      <div class="children">
        {#each entriesOf(value) as [key, child] (key)}
          <JsonTree value={child} name={key} depth={depth + 1} {open} />
        {/each}
      </div>
    {/if}
  </div>
{:else}
  <div class="leaf">
    {#if name !== undefined}<span class="key">{name}</span>{/if}
    <span class="value {value === null ? 'null' : typeof value}">{leafText(value)}</span>
  </div>
{/if}
