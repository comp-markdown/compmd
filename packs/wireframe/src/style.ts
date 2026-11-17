// The sketch look (LANGUAGE.md, decision 5): greys from the host's tokens (spec §9.4), which are built from the
// text colour and so suit light and dark; borders a little uneven; a page's hand-written font when it sets one
// (`--compmd-wireframe-font`), else the system's sans-serif, since a frame doesn't get the page's font; printed as it
// shows.

const LINE = "var(--compmd-border-strong, 1px solid color-mix(in srgb, currentColor 45%, transparent))";
const FAINT = "var(--compmd-border-subtle, 1px solid color-mix(in srgb, currentColor 20%, transparent))";
const FILL = "var(--compmd-surface-subtle, color-mix(in srgb, currentColor 6%, transparent))";
const STRONG = "var(--compmd-surface-strong, color-mix(in srgb, currentColor 12%, transparent))";
const SPACE = "var(--compmd-space-s, 0.5rem)";
/** A hand-drawn box's corners: each a little different. */
const UNEVEN = "6px 3px 7px 2px / 2px 7px 3px 6px";

/** The stylesheet for what `draw` makes. */
export const STYLE = `
.wf-wireframe { display: grid; grid-template: "top top top" auto "left window right" auto "bottom bottom bottom" auto / auto minmax(0, 1fr) auto; gap: ${SPACE}; font-family: var(--compmd-wireframe-font, ui-sans-serif, system-ui, sans-serif); font-size: 0.9em; line-height: 1.4; break-inside: avoid; print-color-adjust: exact; }
.wf-wireframe[data-scheme="dark"] { color-scheme: dark; }
.wf-wireframe[data-scheme="light"] { color-scheme: light; }
.wf-wireframe * { box-sizing: border-box; }
.wf-window { grid-area: window; border: ${LINE}; border-width: 1.5px; border-radius: ${UNEVEN}; overflow: hidden; min-width: 0; }
.wf-window[data-device="phone"] { justify-self: center; width: min(100%, 22rem); border-width: 2px; border-radius: 26px 22px 24px 20px / 20px 24px 22px 26px; padding: ${SPACE}; }
.wf-window[data-device="tablet"] { justify-self: center; width: min(100%, 40rem); border-width: 2px; border-radius: 18px 14px 16px 12px / 12px 16px 14px 18px; padding: ${SPACE}; }
.wf-bar { display: flex; align-items: center; gap: ${SPACE}; padding: 0.35em ${SPACE}; background: ${FILL}; border-bottom: ${FAINT}; }
.wf-statusbar { justify-content: center; background: none; border-bottom: none; }
.wf-dots { display: flex; gap: 0.3em; }
.wf-dot { width: 0.6em; height: 0.6em; border: ${LINE}; border-radius: 50%; }
.wf-title { font-weight: 600; }
.wf-titlebar .wf-title { flex: 1; text-align: center; }
.wf-item-text, .wf-menuitem-label { min-width: 0; overflow-wrap: anywhere; }
.wf-address { flex: 1; padding: 0.1em ${SPACE}; border: ${FAINT}; border-radius: ${UNEVEN}; }
.wf-body, .wf-header, .wf-footer, .wf-panel, .wf-section-body, .wf-col, .wf-cell, .wf-named { display: flex; flex-direction: column; gap: ${SPACE}; min-width: 0; }
.wf-body { padding: ${SPACE}; }
.wf-header { padding-bottom: ${SPACE}; border-bottom: ${FAINT}; }
.wf-header.wf-large { font-size: 1.25em; }
.wf-footer { padding-top: ${SPACE}; border-top: ${FAINT}; }
.wf-panel { padding: ${SPACE}; border: ${LINE}; border-style: dashed; border-radius: ${UNEVEN}; }
.wf-section { border: ${FAINT}; border-radius: ${UNEVEN}; }
.wf-section-title { display: flex; align-items: center; gap: ${SPACE}; padding: 0.25em ${SPACE}; font-weight: 600; border-bottom: ${FAINT}; }
.wf-section-body { padding: ${SPACE}; }
.wf-row { display: flex; flex-direction: row; flex-wrap: wrap; align-items: center; gap: ${SPACE}; min-width: 0; }
.wf-grid { display: grid; gap: ${SPACE}; }
.wf-cell { padding: ${SPACE}; border: ${FAINT}; border-radius: ${UNEVEN}; }
.wf-text.wf-bold, .wf-weight-bold { font-weight: 700; }
.wf-weight-semibold { font-weight: 600; }
.wf-weight-light { font-weight: 300; }
.wf-text.wf-italic { font-style: italic; }
.wf-text.wf-muted, .wf-placeholder, .wf-shortcut, .wf-disabled, .wf-label, .wf-name { opacity: 0.65; }
.wf-size-small { font-size: 0.85em; }
.wf-size-large { font-size: 1.25em; }
.wf-button, .wf-input, .wf-combo-box { display: inline-flex; align-items: center; gap: 0.4em; padding: 0.3em 0.8em; border: ${LINE}; border-radius: ${UNEVEN}; }
.wf-button { justify-content: center; }
.wf-button.wf-primary { background: ${STRONG}; border-width: 2px; font-weight: 600; }
.wf-input { min-width: 10em; }
.wf-combo { display: inline-flex; align-items: center; gap: ${SPACE}; }
.wf-combo-box { min-width: 8em; justify-content: space-between; }
.wf-choice { display: inline-flex; align-items: center; gap: 0.5em; }
.wf-choice .wf-label { opacity: 1; }
.wf-mark { display: inline-block; flex: none; width: 1em; height: 1em; border: ${LINE}; border-radius: 2px 3px 2px 3px; }
.wf-radio-mark { border-radius: 50%; }
.wf-mark.wf-on { background: radial-gradient(currentColor 0 35%, transparent 40%); }
.wf-checkbox-mark.wf-on { display: inline-flex; align-items: center; justify-content: center; background: none; }
.wf-checkbox-mark.wf-on::after { content: "✓"; font-size: 0.85em; line-height: 1; }
.wf-toggle-mark { width: 2em; border-radius: 1em; background: radial-gradient(circle at 25% 50%, currentColor 0 22%, transparent 26%); }
.wf-toggle-mark.wf-on { background: ${STRONG} radial-gradient(circle at 75% 50%, currentColor 0 22%, transparent 26%); }
.wf-progress { display: flex; flex-direction: column; gap: 0.2em; }
.wf-progress-bar { display: block; height: 0.6em; border: ${LINE}; border-radius: ${UNEVEN}; overflow: hidden; }
.wf-progress-fill { display: block; height: 100%; background: ${STRONG}; }
.wf-tabs { display: flex; gap: 0.2em; border-bottom: ${LINE}; }
.wf-tab { display: inline-flex; align-items: center; gap: 0.3em; padding: 0.3em 0.8em; border: ${FAINT}; border-bottom: none; border-radius: 6px 4px 0 0; }
.wf-tab.wf-active { background: ${FILL}; font-weight: 600; border: ${LINE}; border-bottom: none; }
.wf-list, .wf-tree { list-style: none; margin: 0; padding: 0; }
.wf-item { display: flex; justify-content: space-between; padding: 0.35em ${SPACE}; border-bottom: ${FAINT}; }
.wf-tree .wf-tree { padding-inline-start: 1.2em; }
.wf-node-label { display: inline-flex; align-items: center; gap: 0.3em; padding: 0.1em 0.3em; border-radius: ${UNEVEN}; }
.wf-node-label.wf-selected { background: ${STRONG}; }
.wf-chevron { display: inline-block; min-width: 0.8em; opacity: 0.65; }
.wf-menubar { display: flex; align-items: flex-start; gap: ${SPACE}; padding: 0.2em ${SPACE}; background: ${FILL}; border-bottom: ${FAINT}; }
.wf-menu { display: inline-flex; flex-direction: column; }
.wf-menu-title { padding: 0.1em 0.4em; }
.wf-menu-items { display: flex; flex-direction: column; min-width: 10em; margin-top: 0.2em; padding: 0.2em 0; border: ${LINE}; border-radius: ${UNEVEN}; }
.wf-menuitem { display: flex; justify-content: space-between; gap: 1em; padding: 0.15em ${SPACE}; }
.wf-divider { display: block; border-top: ${FAINT}; margin: 0.2em 0; }
.wf-image { display: flex; align-items: center; justify-content: center; max-width: 100%; border: ${LINE}; border-radius: ${UNEVEN}; background: linear-gradient(to top right, transparent calc(50% - 0.5px), color-mix(in srgb, currentColor 35%, transparent) 50%, transparent calc(50% + 0.5px)), linear-gradient(to bottom right, transparent calc(50% - 0.5px), color-mix(in srgb, currentColor 35%, transparent) 50%, transparent calc(50% + 0.5px)); }
.wf-image-label { padding: 0 0.3em; background: Canvas; }
.wf-icon { display: inline-flex; align-items: center; justify-content: center; flex: none; width: 1.3em; height: 1.3em; font-size: 0.8em; border: ${LINE}; border-radius: 3px; }
.wf-badge { padding: 0 0.4em; font-size: 0.8em; border: ${FAINT}; border-radius: 1em; background: ${FILL}; }
.wf-spacer { flex: 1; }
.wf-named { padding: ${SPACE}; border: ${LINE}; border-style: dotted; border-radius: ${UNEVEN}; }
.wf-name { font-family: ui-monospace, monospace; font-size: 0.8em; }
.wf-notes { margin: 0; padding: 0; list-style: none; display: flex; flex-direction: column; gap: ${SPACE}; max-width: 16em; }
.wf-notes-top { grid-area: top; } .wf-notes-left { grid-area: left; } .wf-notes-right { grid-area: right; } .wf-notes-bottom { grid-area: bottom; }
.wf-notes-top, .wf-notes-bottom { flex-direction: row; flex-wrap: wrap; max-width: none; }
.wf-note { display: flex; gap: 0.4em; align-items: baseline; font-style: italic; }
.wf-marker { display: inline-flex; align-items: center; justify-content: center; flex: none; min-width: 1.4em; height: 1.4em; font-size: 0.75em; font-style: normal; font-weight: 700; border: ${LINE}; border-radius: 50%; background: Canvas; }
.wf-marker-at { margin-inline-start: 0.3em; }
@media (max-width: 30rem) { .wf-wireframe { grid-template: "top" auto "window" auto "left" auto "right" auto "bottom" auto / minmax(0, 1fr); } .wf-notes { max-width: none; } }
`;
