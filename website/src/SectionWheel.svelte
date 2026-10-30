<!-- The landing page's contents, a wheel at the right: the section being read in the middle, the ones before and after
     it above and below, smaller and fainter the further they are, fading out at the ends. It turns as the page scrolls;
     a section chosen in it scrolls into view. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { lookOf, placeOf } from "./wheel.ts";

  interface Props {
    sections: { id: string; label: string }[];
  }

  let { sections }: Props = $props();
  /** The reader's place: 2.5 is halfway through the third section. */
  let place = $state(0);
  const current = $derived(Math.min(sections.length - 1, Math.floor(place + 0.5)));

  onMount(() => {
    let queued = false;
    const update = () => {
      queued = false;
      const tops = sections.map(({ id }) => document.getElementById(id)?.getBoundingClientRect().top ?? 0);
      const last = document.getElementById(sections.at(-1)?.id ?? "");
      place = placeOf(tops, innerHeight * 0.35, last?.getBoundingClientRect().bottom ?? 0);
    };
    const onScroll = () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(update);
    };
    addEventListener("scroll", onScroll, { passive: true });
    addEventListener("resize", onScroll);
    update();
    return () => {
      removeEventListener("scroll", onScroll);
      removeEventListener("resize", onScroll);
    };
  });
</script>

<nav class="wheel" aria-label="On this page">
  <ol>
    {#each sections as section, index (section.id)}
      {@const look = lookOf(index - place)}
      <li style:--offset={look.offset} style:--scale={look.scale} style:opacity={look.opacity}>
        <a href="#{section.id}" aria-current={index === current ? "location" : undefined}>{section.label}</a>
      </li>
    {/each}
  </ol>
</nav>
