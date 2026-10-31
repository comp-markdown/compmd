// `__component__`: draws its `value` in its box. Replace the drawing with yours; what it imports is bundled in.

/** What the host gives a component in the plain binding (CompMD spec §7.4). */
export interface Context<Params> {
  params: Params;
  /** The fallback a reader without the component sees: use its text as the text alternative of what you draw. */
  fallback: { text: string; element: Element };
}

export interface Params {
  value: string;
}

export default function mount(target: HTMLElement, { params, fallback }: Context<Params>): { update(next: Params): void; destroy(): void } {
  const shown = document.createElement("p");
  shown.setAttribute("aria-label", fallback.text);
  shown.style.cssText = "display: inline-block; margin: 0; padding: 0.4em 0.9em; border: 2px solid currentColor; border-radius: 999px; font-weight: 600;";
  target.append(shown);
  const draw = ({ value }: Params) => {
    shown.textContent = value;
  };
  draw(params);
  return {
    update: draw,
    destroy: () => {
      shown.remove();
    },
  };
}
