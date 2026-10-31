<!-- A page of the CompMD Developer Guide: the contents or a chapter, rendered on the server only, as static HTML;
     `spec.ts` adds the scroll-spy and the phone menu, as on the spec's page. Beside the text, the book: its parts and
     chapters, the one being read open to its sections; under the text, the chapters before and after it. -->
<script lang="ts" module>
  /** A chapter to go to. */
  export interface ChapterLink {
    title: string;
    href: string;
  }
</script>

<script lang="ts">
  import type { Part } from "../build/guide.ts";
  import type { Heading } from "../build/spec.ts";
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";

  interface Props {
    /** The page as HTML, from `renderMarkdown`. */
    html: string;
    /** Its sections. */
    toc: Heading[];
    /** The path from this page to the site's root. */
    root: string;
    book: Part[];
    /** The chapter this page is; absent for the contents. */
    current?: string | undefined;
    previous?: ChapterLink | undefined;
    next?: ChapterLink | undefined;
    /** Its file on GitHub. */
    source: string;
    /** The title of the page: the chapter's, or the guide's. */
    title: string;
  }

  let { html, toc, root, book, current, previous, next, source, title }: Props = $props();
  const guide = $derived(`${root}guide/`);
</script>

<svelte:head>
  <title>{current === undefined ? title : `${title} · CompMD Developer Guide`}</title>
</svelte:head>

<Header {root} current="guide" />

<div class="status">
  <div class="wrap">
    <p>
      <strong>CompMD Developer Guide.</strong> Written for the <code>@compmd</code> packages at 0.1.1, and following them as they change.
      <a href={source}>This page on GitHub</a>
    </p>
  </div>
</div>

<div class="wrap reading">
  <nav class="toc book" aria-label="The guide">
    <button type="button" class="toc-toggle" aria-expanded="false" aria-controls="toc-list" hidden>Contents</button>
    <ol id="toc-list">
      <li class="level-2"><a href={guide} aria-current={current === undefined ? "page" : undefined}>Contents</a></li>
      {#each book as part (part.title)}
        <li class="part">{part.title}</li>
        {#each part.chapters as chapter (chapter.number)}
          {#if chapter.slug}
            <li class="chapter">
              <a href="{guide}{chapter.slug}/" aria-current={chapter.slug === current ? "page" : undefined}>{chapter.number}. {chapter.title}</a>
              {#if chapter.slug === current}
                <ol>
                  {#each toc.filter((heading) => heading.level === 2) as heading (heading.id)}
                    <li class="level-3"><a href="#{heading.id}">{heading.text}</a></li>
                  {/each}
                </ol>
              {/if}
            </li>
          {:else}
            <li class="chapter later"><span>{chapter.number}. {chapter.title}</span></li>
          {/if}
        {/each}
      {/each}
    </ol>
  </nav>
  <div>
    <article class="spec guide">
      {@html html}
    </article>
    {#if previous || next}
      <nav class="turn" aria-label="Chapters">
        {#if previous}<a class="previous" href={previous.href}><span>Previous</span>{previous.title}</a>{/if}
        {#if next}<a class="next" href={next.href}><span>Next</span>{next.title}</a>{/if}
      </nav>
    {/if}
  </div>
</div>

<Footer />
