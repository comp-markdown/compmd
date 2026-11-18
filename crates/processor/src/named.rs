//! Enums whose variants each have a name in the source text, such as functions and operators, written once.

/// Defines a fieldless enum whose variants each have a name, with `ALL`, `NAMES`, `name()` and `from_name()`.
macro_rules! named {
    (
        $(#[$meta:meta])*
        $vis:vis enum $enum:ident {
            $($(#[$variant_meta:meta])* $variant:ident = $name:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        $vis enum $enum {
            $($(#[$variant_meta])* $variant),+
        }

        #[allow(dead_code, reason = "not every named enum uses every helper")]
        impl $enum {
            /// Every variant, in declared order.
            $vis const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Every variant's name, in declared order.
            $vis const NAMES: &'static [&'static str] = &[$($name),+];

            /// The name as it is written.
            $vis fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),+
                }
            }

            /// The variant written `name`.
            $vis fn from_name(name: &str) -> Option<Self> {
                Self::ALL.iter().copied().find(|variant| variant.name() == name)
            }
        }
    };
}

pub(crate) use named;

/// Names as a sentence lists them: `"a", "b" or "c"`.
pub(crate) fn choices(names: &[&str]) -> String {
    let quoted: Vec<String> = names.iter().map(|name| format!("\"{name}\"")).collect();
    match quoted.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} or {last}", rest.join(", ")),
        _ => quoted.concat(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    named! {
        enum Fruit {
            Apple = "apple",
            Pear = "pear",
        }
    }

    #[test]
    fn names_go_both_ways() {
        assert_eq!(Fruit::ALL, [Fruit::Apple, Fruit::Pear]);
        assert_eq!(Fruit::NAMES, ["apple", "pear"]);
        assert_eq!(Fruit::from_name("pear").map(Fruit::name), Some("pear"));
        assert_eq!(Fruit::from_name("plum"), None);
    }

    #[test]
    fn choices_read_as_a_sentence() {
        assert_eq!(choices(&["a", "b", "c"]), "\"a\", \"b\" or \"c\"");
        assert_eq!(choices(&["a"]), "\"a\"");
        assert_eq!(choices(&[]), "");
    }
}
