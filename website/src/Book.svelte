<!-- A book of the guides as one page, for its PDF: rendered on the server only, typeset as a book (`build/book.css`),
     printed, then removed. Its cover, title and copyright pages, its front matter (the contents), then each chapter. -->
<script lang="ts">
  import type { BookProps } from "../build/book.ts";
  import FrontMatter from "./FrontMatter.svelte";

  let { title, purpose, version, contents, chapters, online }: BookProps = $props();

  const month = new Intl.DateTimeFormat("en", { month: "long", year: "numeric", timeZone: "UTC" }).format(new Date());
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<article class="spec guide book-print" id="book">
  <FrontMatter {title} subtitle={purpose} edition="This edition: {month}, for the @compmd packages, version {version}." {online} />
  <section class="front">
    {@html contents}
  </section>
  {#each chapters as chapter, index (index)}
    <section class={["chapter", index === 0 && "first"]}>
      {@html chapter}
    </section>
  {/each}
</article>
