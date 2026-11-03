// Where an offset of a text area's text is drawn, with its lines wrapped as the text area wraps them: a text area
// doesn't say, so a copy of its text, styled as it is, is laid out out of sight and measured.

/** The styles that decide where a text area's text wraps and how tall its lines are. */
const LAYOUT = [
  "box-sizing",
  "padding-top",
  "padding-right",
  "padding-bottom",
  "padding-left",
  "border-top-width",
  "border-right-width",
  "border-bottom-width",
  "border-left-width",
  "font-family",
  "font-size",
  "font-weight",
  "font-style",
  "letter-spacing",
  "line-height",
  "tab-size",
  "text-indent",
  "word-spacing",
] as const;

/**
 * How far below the top of `textarea`'s content the line holding `offset` starts, in pixels, as the text area lays
 * it out.
 */
export function caretTop(textarea: HTMLTextAreaElement, offset: number): number {
  const document = textarea.ownerDocument;
  const style = getComputedStyle(textarea);
  const mirror = document.createElement("div");
  for (const property of LAYOUT) mirror.style.setProperty(property, style.getPropertyValue(property));
  Object.assign(mirror.style, { position: "absolute", visibility: "hidden", insetBlockStart: "0", insetInlineStart: "-9999px", whiteSpace: "pre-wrap", overflowWrap: "break-word", borderStyle: "solid" });
  // A scroll bar takes from the width the text wraps in.
  mirror.style.width = `${String(textarea.clientWidth + Number.parseFloat(style.borderLeftWidth) + Number.parseFloat(style.borderRightWidth))}px`;
  mirror.textContent = textarea.value.slice(0, offset);
  const marker = mirror.appendChild(document.createElement("span"));
  marker.textContent = "​";
  document.body.appendChild(mirror);
  const top = marker.offsetTop - Number.parseFloat(style.paddingTop);
  mirror.remove();
  return top;
}
