//! The CEL profile against the `cel` crate as an oracle, on expressions where the profile and CEL agree: no `int`
//! and `double` mixing, nothing the profile leaves out, no `double` written as a string. Both must give the same
//! value, or both an error.

use crate::cel::{Program, Scope, Values};
use crate::host::Time;
use crate::md::{Md, Signatures};
use serde_json::Value as Json;

struct Utc;

impl Time for Utc {
    fn offset(&self, _zone: &str, _at: i64) -> Option<i32> {
        Some(0)
    }
}

/// What the `cel` crate gives: a value, or `None` for any error.
fn oracle(source: &str) -> Option<Json> {
    let mut env = ::cel::Env::stdlib();
    ::cel::extensions::strings(&mut env).ok()?;
    let env = std::sync::Arc::new(env);
    let program = env.compile(source).ok()?;
    let context = ::cel::Context::with_env(env);
    program.execute(&context).ok()?.json().ok()
}

/// What the profile gives.
fn profile(source: &str) -> Option<Json> {
    let program = Program::parse(source).ok()?;
    program.check(&Scope::default(), &Signatures).ok()?;
    let md = Md::new(0, "en", "UTC", &Utc);
    program
        .evaluate(&Values::default(), &md)
        .ok()
        .map(|value| value.typed_json())
}

/// The expressions on which the two disagree, with what each gave.
fn disagreements<'a>(sources: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    sources
        .into_iter()
        .filter_map(|source| {
            let (expected, actual) = (oracle(source), profile(source));
            (expected != actual)
                .then(|| format!("{source}\n    cel:     {expected:?}\n    profile: {actual:?}"))
        })
        .collect()
}

/// Expressions chosen for CEL's corners.
const CORPUS: &[&str] = &[
    // Integers: overflow, division, signs, unsigned, hexadecimal.
    "1 + 2 * 3 - 4 / 2 % 3",
    "-7 / 2",
    "-7 % 3",
    "9223372036854775807 + 1",
    "-9223372036854775808 - 1",
    "-9223372036854775808",
    "9223372036854775807 * 2",
    "1 / 0",
    "5 % 0",
    "1u + 2u",
    "0u - 1u",
    "18446744073709551615u",
    "0x1F + 0x10",
    // Doubles.
    "1.5 * 2.0",
    "1.0 / 0.0 == 2.0 / 0.0",
    "1e3",
    // Comparison and equality.
    "'a' < 'b' && 'b' <= 'b' && 'c' > 'b'",
    "true == true && false != true",
    "1 == 1.0",
    "1 == 1u",
    "[1, 2] == [1, 2]",
    "[1, 2] == [2, 1]",
    "{'a': 1} == {'a': 1}",
    "null == null",
    // Lists and maps.
    "2 in [1, 2, 3]",
    "'b' in {'a': 1, 'b': 2}",
    "[1, 2, 3][2]",
    "[1, 2, 3][3]",
    "[1, 2, 3][-1]",
    "{'a': 1}['b']",
    "{'a': {'b': 2}}.a.b",
    "true ? 1 : 2",
    "false ? 1 : 2",
    // Macros.
    "[1, 2, 3].map(x, x * x)",
    "[1, 2, 3].filter(x, x % 2 == 1)",
    "[1, 2, 3].all(x, x > 0)",
    "[1, 2, 3].exists(x, x > 2)",
    "[1, 2, 3].exists_one(x, x > 1)",
    "[1, 2, 3].map(x, x > 1, x * 10)",
    "{'a': 1, 'b': 2}.all(k, k != 'c')",
    "has({'a': 1}.a) && !has({'a': 1}.b)",
    // Sizes and conversions.
    "[].size() == 0",
    "size('héllo') + size([1]) + size({'a': 1})",
    "'abc'.size()",
    "int('-12') + int(3.99) + int(-3.99)",
    "int(1u)",
    "int('x')",
    "int(1e19)",
    "double('2.5') + double(2)",
    "string(12) + string(true) + string(1u) + string('s')",
    "string(2.5)",
    // Timestamps and durations.
    "timestamp('2026-10-21T10:00:00Z') < timestamp('2026-10-21T11:00:00Z')",
    "string(timestamp('2026-10-21T12:00:00+02:00'))",
    "string(timestamp('2026-10-21T10:00:00.250Z'))",
    "timestamp('2026-10-21T10:00:00Z') - timestamp('2026-10-21T09:30:00Z') == duration('30m')",
    "duration('1h30m') > duration('90m')",
    "duration('1.5h') == duration('90m')",
    "timestamp('nope')",
    "duration('3d')",
    // Strings: code points, counts, edges.
    "'hello'.contains('ell') && !'hello'.startsWith('l') && 'hello'.endsWith('lo')",
    "'hello'.charAt(0) + 'hello'.charAt(4) + 'hello'.charAt(5)",
    "'hello'.charAt(6)",
    "'hello'.indexOf('l') + 'hello'.indexOf('l', 3) + 'hello'.indexOf('z')",
    "'hello'.lastIndexOf('l') + 'hello'.lastIndexOf('l', 2)",
    "'hello'.indexOf('')",
    "'hello'.lastIndexOf('')",
    "'hello'.indexOf('l', 9)",
    "'héllo'.indexOf('l')",
    "'HeLLo'.lowerAscii() + 'HeLLo'.upperAscii()",
    "'ÀÉ'.lowerAscii()",
    "'aaa'.replace('a', 'b')",
    "'aaa'.replace('a', 'b', 1)",
    "'aaa'.replace('a', 'b', 0)",
    "'aaa'.replace('a', 'b', -1)",
    "'a,b,c'.split(',')",
    "'a,b,c'.split(',', 2)",
    "'a,b,c'.split(',', 0)",
    "'a,b,c'.split(',', -1)",
    "'abc'.split('')",
    "''.split(',')",
    "'hello'.substring(2)",
    "'hello'.substring(1, 4)",
    "'hello'.substring(3, 2)",
    "'hello'.substring(9)",
    "'héllo'.substring(1, 2)",
    "'  \\t x y \\n'.trim()",
    "['a', 'b', 'c'].join()",
    "['a', 'b', 'c'].join(', ')",
    "[].join('-')",
    // Literals.
    "'\\u00e9\\x41\\101\\n'",
    "r'\\n'",
    "'''a\nb'''",
    // Errors, absorbed or not.
    "1 + 'a'",
    "true && 1 / 0 == 1",
    "1 / 0 == 1 && false",
    "1 / 0 == 1 || true",
];

#[test]
fn chosen_expressions_agree_with_the_oracle() {
    let wrong = disagreements(CORPUS.iter().copied());
    assert!(
        wrong.is_empty(),
        "{} of {} disagree:\n{}",
        wrong.len(),
        CORPUS.len(),
        wrong.join("\n")
    );
}

/// A small deterministic random source (xorshift), so a failure can be run again.
struct Random(u64);

impl Random {
    fn below(&mut self, n: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % n
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[usize::try_from(self.below(items.len() as u64)).unwrap_or(0)]
    }
}

/// Typed random expressions, so that most of them check and both sides have something to compute.
struct Generator(Random);

impl Generator {
    fn int(&mut self, depth: usize) -> String {
        if depth == 0 || self.0.below(4) == 0 {
            let leaves = [
                "0",
                "1",
                "2",
                "3",
                "7",
                "-1",
                "-5",
                "10",
                "100",
                "9223372036854775807",
                "-9223372036854775808",
            ];
            return self.0.pick(&leaves).into();
        }
        let d = depth - 1;
        match self.0.below(10) {
            0 => format!("({} + {})", self.int(d), self.int(d)),
            1 => format!("({} - {})", self.int(d), self.int(d)),
            2 => format!("({} * {})", self.int(d), self.int(d)),
            3 => format!("({} / {})", self.int(d), self.int(d)),
            4 => format!("({} % {})", self.int(d), self.int(d)),
            5 => format!("size({})", self.string(d)),
            6 => format!("{}[{}]", self.list(d), self.int(d)),
            7 => format!("({} ? {} : {})", self.boolean(d), self.int(d), self.int(d)),
            8 => format!("{}.indexOf({})", self.string(d), self.string(d)),
            _ => format!("-{}", self.int(d)),
        }
    }

    fn boolean(&mut self, depth: usize) -> String {
        if depth == 0 || self.0.below(4) == 0 {
            return self.0.pick(&["true", "false"]).into();
        }
        let d = depth - 1;
        match self.0.below(10) {
            0 => format!("({} < {})", self.int(d), self.int(d)),
            1 => format!("({} == {})", self.int(d), self.int(d)),
            2 => format!("({} >= {})", self.string(d), self.string(d)),
            3 => format!("({} && {})", self.boolean(d), self.boolean(d)),
            4 => format!("({} || {})", self.boolean(d), self.boolean(d)),
            5 => format!("!{}", self.boolean(d)),
            6 => format!("{}.contains({})", self.string(d), self.string(d)),
            7 => format!("({} in {})", self.int(d), self.list(d)),
            8 => format!(
                "{}.exists(v{depth}, v{depth} > {})",
                self.list(d),
                self.int(d)
            ),
            _ => format!(
                "{}.all(v{depth}, v{depth} != {})",
                self.list(d),
                self.int(d)
            ),
        }
    }

    fn string(&mut self, depth: usize) -> String {
        if depth == 0 || self.0.below(4) == 0 {
            return self
                .0
                .pick(&["''", "'a'", "'ab'", "'héllo'", "'a,b'", "' x '", "'AbC'"])
                .into();
        }
        let d = depth - 1;
        match self.0.below(9) {
            0 => format!("({} + {})", self.string(d), self.string(d)),
            1 => format!("{}.substring({})", self.string(d), self.int(d)),
            2 => format!("{}.charAt({})", self.string(d), self.int(d)),
            3 => format!("{}.lowerAscii()", self.string(d)),
            4 => format!(
                "{}.replace({}, {})",
                self.string(d),
                self.string(d),
                self.string(d)
            ),
            5 => format!("{}.trim()", self.string(d)),
            6 => format!("string({})", self.int(d)),
            7 => format!(
                "{}.split({}).join({})",
                self.string(d),
                self.string(d),
                self.string(d)
            ),
            _ => format!(
                "({} ? {} : {})",
                self.boolean(d),
                self.string(d),
                self.string(d)
            ),
        }
    }

    fn list(&mut self, depth: usize) -> String {
        if depth == 0 || self.0.below(3) == 0 {
            return self
                .0
                .pick(&["[]", "[1]", "[1, 2, 3]", "[-1, 0, 7]"])
                .into();
        }
        let d = depth - 1;
        match self.0.below(4) {
            0 => format!("({} + {})", self.list(d), self.list(d)),
            1 => format!("{}.map(w{depth}, w{depth} * {})", self.list(d), self.int(d)),
            2 => format!(
                "{}.filter(w{depth}, w{depth} > {})",
                self.list(d),
                self.int(d)
            ),
            _ => format!("[{}, {}]", self.int(d), self.int(d)),
        }
    }
}

#[test]
fn random_expressions_agree_with_the_oracle() {
    let mut generator = Generator(Random(0x5eed_cafe));
    let sources: Vec<String> = (0..10_000)
        .map(|i| match i % 4 {
            0 => generator.int(5),
            1 => generator.boolean(5),
            2 => generator.string(5),
            _ => generator.list(5),
        })
        .collect();
    // Agreeing only on errors would prove little: most expressions must give a value.
    let values = sources
        .iter()
        .filter(|source| profile(source).is_some())
        .count();
    assert!(
        values * 5 > sources.len() * 2,
        "only {values} of {} give a value",
        sources.len()
    );
    let wrong = disagreements(sources.iter().map(String::as_str));
    let first: Vec<&str> = wrong.iter().take(15).map(String::as_str).collect();
    assert!(
        wrong.is_empty(),
        "{} of {} disagree; the first ones:\n{}",
        wrong.len(),
        sources.len(),
        first.join("\n")
    );
}
