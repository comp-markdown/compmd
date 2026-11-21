// Packages to wrap, as `npm install` puts them, with types of their own: a React component, a Svelte component, a
// custom element and a plain library, and the few types of React and Svelte their props read. For the tests of the
// wrap (docs/wrapping-an-npm-package.md).

/** React's types, as far as a component's props read them: its props, the DOM's own, and a CSS colour. */
const REACT_TYPES = `export type ComponentProps<T> = T extends (props: infer P) => unknown ? P : never;
export interface SVGProps<T> { className?: string; style?: object; width?: number; height?: number; onClick?: (event: T) => void }
type Color = "red" | "blue" | (string & {});
export interface CSSProperties { backgroundColor?: Color; color?: Color }
`;

/** Svelte's types, as far as a component's props read them. */
const SVELTE_TYPES = `export interface Component<P> { (internal: unknown, props: P): unknown }
export type ComponentProps<T> = T extends Component<infer P> ? P : never;
`;

/** A React component, `qr`, whose props extend the DOM's: only its own are params. */
const QR_TYPES = `import type { CSSProperties, SVGProps } from "react";
export interface QrProps extends SVGProps<SVGSVGElement> {
  /** What the code holds */
  value: string;
  /**
   * How much of it can be lost.
   * @default "L"
   */
  level?: "L" | "M" | "Q" | "H";
  bgColor?: CSSProperties["backgroundColor"];
  size?: number;
  title?: string | ((value: string) => string);
  onDone?: () => void;
  when?: Date;
  tags?: string[];
  options?: { margin: number };
  label?: "on top" | "below";
  any?: string | number;
}
declare const Qr: (props: QrProps) => unknown;
export default Qr;
`;

/** The files of the four packages, and of React's and Svelte's types, under a pack's `node_modules`. */
export const WRAPPED: Record<string, string | object> = {
  "node_modules/react/package.json": { name: "react", version: "19.3.0", types: "index.d.ts" },
  "node_modules/react/index.d.ts": REACT_TYPES,
  "node_modules/svelte/package.json": { name: "svelte", version: "5.57.2", types: "index.d.ts" },
  "node_modules/svelte/index.d.ts": SVELTE_TYPES,
  "node_modules/qr/package.json": { name: "qr", version: "2.2.0", types: "index.d.ts" },
  "node_modules/qr/index.d.ts": QR_TYPES,
  "node_modules/charts/package.json": { name: "charts", version: "1.0.0", types: "index.d.ts", peerDependencies: { svelte: "^5" } },
  "node_modules/charts/index.d.ts": 'import type { Component } from "svelte";\nexport interface BarChartProps {\n  data: object[];\n  /** @default 0.2 */\n  bandPadding?: number;\n  height?: number;\n}\nexport declare const BarChart: Component<BarChartProps>;\n',
  "node_modules/viewer/package.json": { name: "viewer", version: "4.0.0", types: "index.d.ts" },
  "node_modules/viewer/index.d.ts": 'export declare class Viewer extends HTMLElement { src: string; autoRotate: boolean; play(): void }\ndeclare global { interface HTMLElementTagNameMap { "x-viewer": Viewer } }\n',
  "node_modules/plain/package.json": { name: "plain", version: "1.5.4", type: "module", exports: "./index.js" },
  "node_modules/plain/index.js": "export const draw = (value) => `<svg>${value}</svg>`;\n",
};
