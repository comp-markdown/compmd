<!-- The document's inputs (spec §8.7), each with a control by its type; changing one renders the document again with
     that value, as the page's input components do. -->
<script lang="ts">
  import type { ExpandedInput } from "@compmd/processor";

  let { inputs, onset }: { inputs: Record<string, ExpandedInput>; onset: (name: string, value: unknown) => void } = $props();

  const text = (value: unknown) => (typeof value === "string" || typeof value === "number" ? String(value) : "");
</script>

<fieldset class="inputs">
  <legend>Inputs</legend>
  {#each Object.entries(inputs) as [name, input] (name)}
    <label>
      <code>{name}</code>
      {#if input.members}
        <select value={text(input.value)} onchange={(event) => onset(name, event.currentTarget.value)}>
          {#each input.members as member (member)}
            <option value={member}>{member}</option>
          {/each}
        </select>
      {:else if input.type === "boolean"}
        <input type="checkbox" checked={input.value === true} onchange={(event) => onset(name, event.currentTarget.checked)} />
      {:else if input.type === "number"}
        <input type="number" value={text(input.value)} onchange={(event) => onset(name, event.currentTarget.valueAsNumber)} />
      {:else if input.type === "date"}
        <input type="date" value={text(input.value)} onchange={(event) => onset(name, event.currentTarget.value)} />
      {:else}
        <input type="text" value={text(input.value)} onchange={(event) => onset(name, event.currentTarget.value)} />
      {/if}
    </label>
  {/each}
</fieldset>
