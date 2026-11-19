<!-- The blog's layout, as `@compmd/blog` takes one (`build/blog.ts`): the site's header and footer around the place each
     page of the blog goes, its text in the guides' column and type, and a line under it on how the blog is made, said
     once here rather than in every post. One layout for every page of the blog, at any depth, so its links start at the
     site's root. -->
<script lang="ts">
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";
  import { REPOSITORY } from "./site.ts";

  interface Props {
    /** What goes where each page's content is: the blog tool's marker for it. */
    content: string;
  }

  let { content }: Props = $props();

  /** The site's root, from any page of the blog. */
  const ROOT = "/";
</script>

<Header root={ROOT} current="blog" />

<main class="wrap reading blog">
  <div class="text">
    <div class="spec">
      {@html content}
    </div>
    <p class="colophon">
      Every post is a CompMD document, built with <a href="{REPOSITORY}/tree/main/npm-packages/blog">@compmd/blog</a>, CompMD's blog tool. Follow the blog
      by its <a href="{ROOT}blog/feed.xml">feed</a>.
    </p>
  </div>
</main>

<Footer />
