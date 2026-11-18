<!-- The Messages tab: the protocol on a timeline, a lane per frame, each message's copy opening as the trees do. Every
     value is shown as text, never as markup (docs/devtools.md, O1). -->
<script lang="ts">
  import JsonTree from "./JsonTree.svelte";
  import { clockOf, frameName } from "./labels.ts";
  import type { Timeline } from "./messages.ts";
  import Reported from "./Reported.svelte";

  let { timeline }: { timeline: Timeline } = $props();

  /** The messages opened, by their place in what was kept. */
  let opened = $state(new Set<number>());
  const toggle = (index: number) => {
    const next = new Set(opened);
    if (!next.delete(index)) next.add(index);
    opened = next;
  };
  const laneOf = (frame: string) => timeline.lanes.indexOf(frame) + 2;
</script>

<div class="messages" style:--lanes={timeline.lanes.length}>
  {#if timeline.rows.length === 0}
    <p class="note">No messages yet.</p>
  {:else}
    <div class="lanes" role="row">
      <span class="at"></span>
      {#each timeline.lanes as lane (lane)}
        <span class="lane-name" style:grid-column={laneOf(lane)}>{frameName(lane)}</span>
      {/each}
    </div>
    <ol class="timeline">
      {#each timeline.rows as row (row.index)}
        <li>
          <time class="at">{clockOf(row.at)}</time>
          <div class="message {row.way}" style:grid-column={laneOf(row.frame)}>
            <button type="button" class="toggle" aria-expanded={opened.has(row.index)} onclick={() => toggle(row.index)}>
              <span class="way" title={row.way === "down" ? "To the frame" : "From the frame"}>{row.way === "down" ? "↓" : "↑"}</span>
              <code>{row.type}</code>
            </button>
            <Reported by={row.by} />
            {#if opened.has(row.index)}
              <div class="tree"><JsonTree value={row.data} open={1} /></div>
            {/if}
          </div>
        </li>
      {/each}
    </ol>
  {/if}
</div>
