<!-- One window of the Sandbox tab, with the frames in it: its level and network, what it renders and how long its last
     render took, the components mounted in it with their times, and its fallbacks. What a frame reported is marked. -->
<script lang="ts">
  import type { FrameNode } from "./frames.ts";
  import FrameTree from "./FrameTree.svelte";
  import { frameName, msText, rendersText } from "./labels.ts";
  import Reported from "./Reported.svelte";

  let { frame }: { frame: FrameNode } = $props();
</script>

<li class="frame {frame.level}">
  <div class="head">
    <span class="name">{frameName(frame.id)}</span>
    <span class="level">{frame.level === "host" ? "trusted" : frame.level}</span>
    {#if frame.renders}<span class="renders">{rendersText(frame.renders)}</span>{/if}
    {#if frame.level === "permissive"}
      <span class="network">{frame.network?.length ? frame.network.join(", ") : "no network"}</span>
    {/if}
    {#if frame.rendered !== undefined}<span class="time" title="Its last render, painted">rendered in {msText(frame.rendered)}</span>{/if}
    <Reported by={frame.by} />
  </div>
  {#each frame.claims as claim, i (i)}
    <p class="claim">
      It says it is {claim.level}, rendering {rendersText(claim.renders)}{#if claim.network}, with {claim.network.join(", ") || "no network"}{/if}.
      <Reported by={claim.by} />
    </p>
  {/each}
  {#if frame.uses.length > 0 || frame.fallbacks.length > 0}
    <ul class="uses">
      {#each frame.uses as use (use.use)}
        <li>
          <code>{use.component}</code>
          <span class="time">mounted in {msText(use.ms)}{#if use.updated !== undefined}, last updated in {msText(use.updated)} ({use.updates === 1 ? "1 update" : `${String(use.updates)} updates`}){/if}</span>
          <Reported by={use.by} />
        </li>
      {/each}
      {#each frame.fallbacks as fallback, i (i)}
        <li class="fallback">
          <code>{fallback.component}</code> shows its fallback: <span class="reason">{fallback.reason}</span>, {fallback.message}
          <Reported by={fallback.by} />
        </li>
      {/each}
    </ul>
  {/if}
  {#if frame.frames.length > 0}
    <ul class="frames">
      {#each frame.frames as child (child.id)}
        <FrameTree frame={child} />
      {/each}
    </ul>
  {/if}
</li>
