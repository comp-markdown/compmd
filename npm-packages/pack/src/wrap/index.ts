// `@compmd/pack/wrap`: making a pack of an npm package (docs/wrapping-an-npm-package.md), for `npm create @compmd/pack
// -- --from`, which writes the pack, and for `compmd-pack build`, which writes its signature at each build.

export { KINDS, readWrap, WRAP_FILE, type Kind, type Override, type Wrap } from "./config.ts";
export { generateSignature, paramsOf, type WrapReport } from "./generate.ts";
export { generatedPart, opening } from "./generated.ts";
export { installedPackage, type Installed } from "./installed.ts";
export { kindOf } from "./kind.ts";
export { isParam, mapProp, type Left, type Param } from "./map.ts";
export { elementTags, readProps, type Prop, type PropType } from "./props.ts";
export { risksOf } from "./risks.ts";
export { signatureFence } from "./signature.ts";
