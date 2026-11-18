<!-- A printed book's first pages, which only its typesetting shows (`build/book.css`): the cover, the title page, and
     the copyright page on its back. The guides' books and the spec's PDF open with them. -->
<script lang="ts">
  import logo from "../../branding/compmd-dark.svg?raw";
  import { AUTHOR } from "./site.ts";

  interface Props {
    /** The book's title. */
    title: string;
    /** What it is for, in a sentence. */
    subtitle: string;
    /** Which edition this is: what it follows, and when it was printed. */
    edition: string;
    /** Its address on the site, where the online edition is. */
    online: string;
    /** Its license, when it has one: what it covers, and where its terms are. */
    license?: { text: string; url: string } | undefined;
    /** When it was printed. */
    printed?: Date;
  }

  let { title, subtitle, edition, online, license, printed = new Date() }: Props = $props();

  const year = $derived(printed.getUTCFullYear());
  /** The cover's line: the subtitle's first sentence. */
  const tagline = $derived(subtitle.split(/(?<=\.)\s/)[0] ?? subtitle);
</script>

<div class="front-matter" hidden>
  <section class="cover">
    <!-- The logo for a dark ground, inlined, so the cover needs no file of the site's when it is printed. -->
    <div class="series">{@html logo}</div>
    <div class="cover-title">
      <p class="title">{title}</p>
      <p class="subtitle">{tagline}</p>
    </div>
    <p class="author">{AUTHOR}</p>
  </section>
  <section class="title-page">
    <div>
      <p class="title book-title">{title}</p>
      <p class="subtitle">{subtitle}</p>
      <p class="author">{AUTHOR}</p>
    </div>
    <p class="publisher">compmd.dev</p>
  </section>
  <section class="copyright">
    <p><i>{title}</i><br />{edition}</p>
    <p>Copyright © {year} {AUTHOR}.</p>
    {#if license}
      <p>{license.text} <a href={license.url}>{license.url}</a></p>
    {/if}
    <p>The online edition, which follows the packages and the spec as they change: <a href={online}>{online}</a></p>
    <p>Published at compmd.dev, {year}.</p>
  </section>
</div>
