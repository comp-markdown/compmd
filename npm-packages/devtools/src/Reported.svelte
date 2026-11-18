<!-- The mark on what a frame reported (docs/devtools.md, O6): code of that frame's level could have written it. -->
<script lang="ts">
  import type { FrameId } from "@compmd/sandbox";

  let { by }: { by: FrameId | undefined } = $props();
</script>

{#if by !== undefined}
  <span class="reported" title="Reported by frame {by}: code of its level could have changed what it says">reported by {by}</span>
{/if}
