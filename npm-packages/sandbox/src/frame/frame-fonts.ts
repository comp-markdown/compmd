// A component frame's fonts of the document's own (spec §9.4): its box's type names them by the faces the page
// declared, whose files the parent sends with the type; the frame declares each from its bytes, once, so what the
// component draws is in the box's font. A face the frame can't read leaves the fallback after it in the family.

import type { FrameFont } from "../protocol/messages.ts";

/** The files of the faces this window's frame declared, by name: what a frame nested in it is sent in turn. */
const HELD = new Map<string, FrameFont[]>();

/** The files of the faces `family`, a computed `font-family`, names among those this frame declared. */
export function heldFontsIn(family: string): FrameFont[] {
  return family.split(",").flatMap((part) => HELD.get(part.trim().replace(/^["']|["']$/g, "")) ?? []);
}

/**
 * Declares fonts in `document` as their files come.
 *
 * @example
 * const fonts = new FrameFonts(document);
 * await fonts.declare(message.fonts); // `font-family: "compmd-font-0-serif", …` now draws in it
 */
export class FrameFonts {
  readonly #document: Document;
  /** The faces declared, by name, weight and style. */
  readonly #declared = new Set<string>();

  constructor(document: Document) {
    this.#document = document;
  }

  /** Declares each of `fonts` not declared yet; resolves once each has loaded or failed to. */
  async declare(fonts: readonly FrameFont[]): Promise<void> {
    const Face = this.#document.defaultView?.FontFace;
    if (!Face) return;
    await Promise.all(
      fonts.map(async (font) => {
        const { name, weight, style, file } = font;
        const key = `${name}\u0000${weight}\u0000${style}`;
        if (this.#declared.has(key)) return;
        this.#declared.add(key);
        HELD.set(name, [...(HELD.get(name) ?? []), font]);
        try {
          const face = new Face(name, await file.arrayBuffer(), { weight: weight === "bold" ? "700" : "400", style, display: "swap" });
          this.#document.fonts.add(face);
          await face.load();
        } catch {
          // Not a font: the family's fallback shows.
        }
      }),
    );
  }
}
