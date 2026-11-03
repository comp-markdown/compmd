<!-- The Sandbox tab: the page's windows as a tree, from the host's page down to the frames its frames make. -->
<script lang="ts">
  import type { FrameNode } from "./frames.ts";
  import FrameTree from "./FrameTree.svelte";

  let { tree, dropped }: { tree: FrameNode; dropped: Readonly<Record<string, number>> } = $props();

  const lost = $derived(Object.entries(dropped).filter(([, count]) => count > 0));
</script>

<div class="sandbox">
  <p class="legend">Each frame's level and network, the components in it and how long they took. What a frame reports of itself and the frames in it is marked: code of its level could have changed it.</p>
  {#if lost.length > 0}
    <p class="note">Only the last 2,000 events of each source are kept: {lost.map(([source, count]) => `${String(count)} dropped of ${source === "" ? "the host's page" : `frame ${source}`}`).join(", ")}.</p>
  {/if}
  <ul class="frames root">
    <FrameTree frame={tree} />
  </ul>
</div>
