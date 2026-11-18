// The Console's drawer, docked under the CompMD Inspector's views as a browser's developer tools dock theirs: how tall
// it may be, so the views above it and the Console in it both stay usable.

/** The Console's height when the tools open, in pixels. */
export const DRAWER_HEIGHT = 220;

/** The least height of the Console, with room for a line and its input, and of the views above it. */
export const DRAWER_MIN = 96;
export const VIEWS_MIN = 120;

/** The step of the arrow keys on the drawer's handle, in pixels. */
export const DRAWER_STEP = 24;

/**
 * The drawer's height for `wanted`, in a pane `available` pixels tall: at least {@link DRAWER_MIN}, and leaving the views
 * above it {@link VIEWS_MIN}; in a pane too short for both, the drawer's least.
 *
 * @example
 * drawerHeight(500, 400); // 280: the views keep 120
 */
export function drawerHeight(wanted: number, available: number): number {
  return Math.max(DRAWER_MIN, Math.min(wanted, available - VIEWS_MIN));
}
