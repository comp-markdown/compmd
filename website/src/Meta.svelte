<!-- What a page says of itself in its head: its description, and the Open Graph and Twitter card a link to it shows on
     social sites and in chats, with the site's card (`build/card.ts`). -->
<script lang="ts">
  import { CARD, CARD_FILE } from "../build/card.ts";
  import { SITE } from "./site.ts";

  interface Props {
    title: string;
    /** One or two sentences, as a search result shows them. */
    description: string;
  }

  let { title, description }: Props = $props();
</script>

<svelte:head>
  <meta name="description" content={description} />
  <meta property="og:type" content="website" />
  <meta property="og:site_name" content="CompMD" />
  <meta property="og:title" content={title} />
  <meta property="og:description" content={description} />
  <meta property="og:image" content="{SITE}{CARD_FILE}" />
  <meta property="og:image:width" content={String(CARD.width)} />
  <meta property="og:image:height" content={String(CARD.height)} />
  <meta property="og:image:alt" content="CompMD: Markdown with components. Readable as text, rendered as a page." />
  <meta name="twitter:card" content="summary_large_image" />
</svelte:head>
