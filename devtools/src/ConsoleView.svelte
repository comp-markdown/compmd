<!-- The Console tab: what went wrong, newest last, each line with its time; what a frame reported is marked as its. -->
<script lang="ts">
  import type { ConsoleEntry } from "./console.ts";
  import type { Range } from "@compmd/processor";
  import { clockOf, frameName } from "./labels.ts";
  import Place from "./Place.svelte";
  import Reported from "./Reported.svelte";

  /** `onselect` selects a diagnostic's place in the text; without it, as in the inspector, places are text. */
  let { entries, onselect }: { entries: readonly ConsoleEntry[]; onselect?: ((range: Range) => void) | undefined } = $props();
</script>

<ol class="console" aria-label="What went wrong, newest last">
  {#each entries as entry, i (i)}
    <li class={entry.kind}>
      <time class="at">{clockOf(entry.at)}</time>
      {#if entry.kind === "diagnostics"}
        {#if entry.diagnostics.length === 0}
          <span>No diagnostics.</span>
        {:else}
          <span>{entry.diagnostics.length === 1 ? "1 diagnostic" : `${String(entry.diagnostics.length)} diagnostics`}:</span>
          <ul>
            {#each entry.diagnostics as d, j (j)}
              <li class={d.severity}>
                <code class="code">{d.code}</code>
                {#if d.file === undefined}
                  <Place range={d.range} title="Select it in the text" {onselect}>{d.range.start.line}:{d.range.start.column}</Place>
                {:else}
                  {d.file} {d.range.start.line}:{d.range.start.column}
                {/if}
                {d.message}
              </li>
            {/each}
          </ul>
        {/if}
      {:else if entry.kind === "problem"}
        <span>{entry.message}</span>
      {:else if entry.kind === "fallback"}
        <span><code class="code">{entry.code ?? entry.reason}</code> <code>{entry.component}</code> shows its fallback in {frameName(entry.frame)}: {entry.message}</span>
        <Reported by={entry.by} />
      {:else if entry.kind === "refused"}
        <span>A <code>set</code> of <code>{entry.input}</code> from {frameName(entry.frame)} was refused: none of its uses binds it.</span>
        <Reported by={entry.by} />
      {:else}
        <span>{frameName(entry.frame)} was stopped: {entry.why === "hung" ? "it stopped answering" : "it is of another protocol version"}.</span>
        <Reported by={entry.by} />
      {/if}
    </li>
  {:else}
    <li class="empty">Nothing went wrong.</li>
  {/each}
</ol>
