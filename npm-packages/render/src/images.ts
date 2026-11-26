// A document's images, for a page that can't load them itself: a page in a frame without the network gets each image's
// file from its host, which fetches only the images of the trees it sent.
import { htmlImageSources, safeUrl } from "./html.ts";
import { textOf, walk, type ExpandedCode, type ExpandedNode } from "./tree.ts";

/**
 * Where a page's images come from: the URL to show for an image's `src`, as the document has it, or `undefined` when
 * it can't be had, or `{ refused: origin }` when the reader hasn't allowed its origin (spec §15). Its alt text shows
 * until then, and stays when it can't; a refused one has a way to allow its origin (`SurfaceOptions.allowOrigin`).
 */
export type ImageSource = (src: string) => Promise<string | undefined | Refused>;

/** An image whose origin the reader hasn't allowed (spec §15). */
export interface Refused {
  refused: string;
}

/**
 * The `src` of every image `nodes` can show, as the renderer asks an {@link ImageSource} for it: Markdown images, link
 * reference definitions (which images may name), and images in raw HTML, an example's included; with `schemes`, the host's own schemes, as
 * the renderer has them (`ownSchemes`); and a pack's image (`pack`) by the URL `packFile` gives it, as the renderer has
 * it (spec §12.3), none without it.
 *
 * @example
 * imageSources(tree.children); // ["./logo.png", "https://example.com/photo.png"]
 * imageSources(tree.children, ["app"]); // ["./logo.png", "app:file/7"]
 * imageSources(tree.children, [], (place) => `/packs/${place}`); // ["/packs/@acme/kit@1.4.2/icons/star.svg"]
 */
export function imageSources(nodes: readonly ExpandedNode[], schemes: readonly string[] = [], packFile?: (place: string) => string | undefined): string[] {
  const sources: string[] = [];
  walk(nodes, (node) => {
    if (node.type === "html") sources.push(...htmlImageSources(textOf(node.value), schemes));
    // An example's block renders its own document (spec §7.8), images and all.
    const example = node.type === "code" ? (node as ExpandedCode).document : undefined;
    if (example) sources.push(...imageSources(example.children, schemes, packFile));
    if (node.type !== "image" && node.type !== "definition") return;
    const url = node.pack === true ? packFile?.(textOf(node.url)) : textOf(node.url);
    if (url === undefined) return;
    const src = safeUrl(url, schemes);
    if (src !== undefined) sources.push(src);
  });
  return sources;
}
