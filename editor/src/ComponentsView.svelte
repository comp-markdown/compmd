<!-- The components of the document in two lists: those it defines, and those it uses, wherever they come from, each
     use's line selecting it in the text. The imported and built-in components it doesn't use are folded away under
     them. -->
<script lang="ts">
  import type { Components, ListedComponent, Range } from "@compmd/processor";
  import { kindLabel, listsOf, paramText, slotsText, sourceText, usesText } from "./components.ts";

  let { components, onselect }: { components: Components; onselect: (range: Range) => void } = $props();

  const lists = $derived(listsOf(components));
</script>

{#snippet uses(ranges: Required<Range>[])}
  <span class="uses">
    <span class="count">{usesText(ranges.length)}</span>
    {#each ranges as range (`${String(range.start.line)}:${String(range.start.column)}`)}
      <button type="button" class="place" title="Select it in the text" onclick={() => onselect(range)}>{range.start.line}</button>
    {/each}
  </span>
{/snippet}

{#snippet name(component: ListedComponent)}
  <th scope="row">
    <code>{component.name}</code>
    {#if component.definedAs}<span class="defined-as">as <code>{component.definedAs}</code> there</span>{/if}
  </th>
{/snippet}

{#snippet signature(component: ListedComponent)}
  {@const slots = slotsText(component)}
  <td class="signature">
    {#if component.signature}
      {#each component.signature.params as param (param.name)}
        <code title={param.doc ?? undefined}>{paramText(param)}</code>
      {:else}
        <span class="empty">no params</span>
      {/each}
      {#if slots}<span class="slots">slots: {slots}</span>{/if}
    {:else}
      <span class="empty">no signature: params go unchecked</span>
    {/if}
  </td>
{/snippet}

<div class="data components">
  <section>
    <h2>Defined here <span>the components this document defines</span></h2>
    {#if lists.defined.length > 0}
      <table class="component-list">
        <tbody>
          {#each lists.defined as component (component.name)}
            <tr class:unused={component.uses.length === 0}>
              {@render name(component)}
              <td class="kind">{kindLabel(component.kind)}</td>
              {@render signature(component)}
              <td class="used">{@render uses(component.uses)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <p class="empty">The document defines no components.</p>
    {/if}
  </section>

  <section>
    <h2>Used <span>every component the document invokes, and where it comes from</span></h2>
    {#if lists.used.length > 0}
      <table class="component-list">
        <tbody>
          {#each lists.used as used (used.name)}
            <tr>
              {#if "unknown" in used}
                <th scope="row"><code>{used.name}</code></th>
                <td class="kind unknown">Unknown</td>
                <td class="source"><span class="empty">stands for no component; the diagnostics say why</span></td>
              {:else}
                {@render name(used)}
                <td class="kind">{kindLabel(used.kind)}</td>
                <td class="source">{sourceText(used.source)}</td>
              {/if}
              <td class="used">{@render uses(used.uses)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <p class="empty">The document invokes no components.</p>
    {/if}
    {#if lists.available.length > 0}
      <details>
        <summary>Available, not used ({lists.available.length})</summary>
        <table class="component-list">
          <tbody>
            {#each lists.available as component (`${component.source ?? ""} ${component.name}`)}
              <tr class="unused">
                {@render name(component)}
                <td class="kind">{kindLabel(component.kind)}</td>
                <td class="source">{sourceText(component.source)}</td>
                {@render signature(component)}
              </tr>
            {/each}
          </tbody>
        </table>
      </details>
    {/if}
  </section>
</div>
