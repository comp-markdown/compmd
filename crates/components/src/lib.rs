//! `compmd-components`: the component reader of [CompMD](https://compmd.dev/) ([the language server], decision 11,
//! Q5): what a host-language component takes, read from its source with `oxc`, for component analysis (spec §7.1,
//! §13.1, W104).
//!
//! It reads what the source states, and nothing it would have to guess: a React component's props from its default
//! export's first parameter (JSX or TSX), a Svelte 5 component's from what its instance script gives `$props()`, each
//! with its type where TypeScript states one a signature's type can be checked against. A component whose props it
//! can't read is left unchecked.
//!
//! ```
//! use compmd::{Ambient, Language, ParseOptions, Processor};
//! use compmd::analysis::ComponentProps;
//! use compmd_components::ComponentReader;
//!
//! let props = ComponentReader.props(Language::Jsx, "export default function Badge({ label }) { return null; }");
//! assert_eq!(props.map(|props| props.props.len()), Some(1));
//!
//! let text = "```jsx component=badge\nexport default function Badge({ label }) { return null; }\n```\n\n```compmd component=badge signature\nparams:\n  label: string\n  size: number\n```\n";
//! let resolved = Processor::new("a.comp.md", text).parse(&ParseOptions::default()).analysing(ComponentReader).resolve((), (), &Ambient::default());
//! assert_eq!(resolved.diagnostics()[0].message, "the signature declares `size`, which a.comp.md has no prop for (`size`)");
//! ```
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

mod binding;
mod react;
mod svelte;
mod types;

use compmd::Language;
use compmd::analysis::{ComponentProps, Props};

/// The component reader: JSX, TSX and Svelte 5. Plain JavaScript and TypeScript components take no props from a
/// binding (spec §7.6), so it reads none.
#[derive(Clone, Copy, Debug, Default)]
pub struct ComponentReader;

impl ComponentProps for ComponentReader {
    fn props(&self, language: Language, code: &str) -> Option<Props> {
        match language {
            Language::Jsx | Language::Tsx => react::props(code, language == Language::Tsx),
            Language::Svelte => svelte::props(code),
            Language::Js | Language::Ts => None,
        }
    }
}

#[cfg(test)]
mod tests;
