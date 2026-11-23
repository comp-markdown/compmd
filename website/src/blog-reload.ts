// In the dev server only, on the blog's pages: the page reloads when a file of the blog is saved (docs/blog.md,
// decision 2). The other pages of the site aren't told.
import { BLOG_CHANGED } from "./blog-changed.ts";

import.meta.hot?.on(BLOG_CHANGED, () => {
  location.reload();
});
