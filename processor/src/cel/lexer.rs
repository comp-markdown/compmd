//! CEL's tokens (cel-spec's lexical structure), each with the code point it starts at.

use super::Error;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    Int(i64),
    Uint(u64),
    Double(f64),
    String(String),
    Ident(String),
    /// `9223372036854775808`: an `int` only right after a single `-`.
    MinIntMagnitude,
    /// Punctuation and operators: `(`, `==`, `&&`, …
    Punct(&'static str),
    End,
}

/// Longest first, so `==` is found before `=` would be.
const PUNCTUATION: [&str; 24] = [
    "==", "!=", "<=", ">=", "&&", "||", "(", ")", "[", "]", "{", "}", ".", ",", ":", "?", "+", "-",
    "*", "/", "%", "<", ">", "!",
];

const RESERVED: [&str; 17] = [
    "as",
    "break",
    "const",
    "continue",
    "else",
    "for",
    "function",
    "if",
    "import",
    "let",
    "loop",
    "package",
    "namespace",
    "return",
    "var",
    "void",
    "while",
];

/// The tokens of `source`, ending with [`Token::End`].
///
/// # Errors
///
/// An unknown character, an unterminated string, a bad escape or number, a bytes literal or a reserved word.
pub(crate) fn tokens(source: &str) -> Result<Vec<(Token, usize)>, Error> {
    let chars: Vec<char> = source.chars().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(&c) = chars.get(at) {
        let start = at;
        if c.is_whitespace() {
            at += 1;
        } else if c == '/' && chars.get(at + 1) == Some(&'/') {
            at = chars.len();
        } else if c.is_ascii_digit()
            || (c == '.' && chars.get(at + 1).is_some_and(char::is_ascii_digit))
        {
            let (token, next) = number(&chars, at)?;
            out.push((token, start));
            at = next;
        } else if c == '"' || c == '\'' {
            let (text, next) = string(&chars, at, false)?;
            out.push((Token::String(text), start));
            at = next;
        } else if c == '_' || c.is_ascii_alphabetic() {
            let quoted = matches!(chars.get(at + 1), Some('"' | '\''));
            if quoted && matches!(c, 'r' | 'R') {
                let (text, next) = string(&chars, at + 1, true)?;
                out.push((Token::String(text), start));
                at = next;
                continue;
            }
            if quoted && matches!(c, 'b' | 'B') {
                return Err(Error::new(
                    start,
                    "bytes literals aren't in the CompMD CEL profile",
                ));
            }
            let end = at
                + chars[at..]
                    .iter()
                    .take_while(|c| **c == '_' || c.is_ascii_alphanumeric())
                    .count();
            let word: String = chars[at..end].iter().collect();
            if RESERVED.contains(&word.as_str()) {
                return Err(Error::new(start, format!("'{word}' is a reserved word")));
            }
            out.push((Token::Ident(word), start));
            at = end;
        } else {
            let punct = PUNCTUATION
                .iter()
                .find(|p| {
                    p.chars()
                        .enumerate()
                        .all(|(i, c)| chars.get(at + i) == Some(&c))
                })
                .ok_or_else(|| unexpected_character(&chars, at))?;
            out.push((Token::Punct(punct), start));
            at += punct.len();
        }
    }
    out.push((Token::End, chars.len()));
    Ok(out)
}

fn unexpected_character(chars: &[char], at: usize) -> Error {
    let message = match chars[at] {
        '=' => "unexpected '='; write '==' to compare".to_string(),
        '&' => "unexpected '&'; write '&&'".to_string(),
        '|' => "unexpected '|'; write '||'".to_string(),
        c => format!("unexpected character '{c}'"),
    };
    Error::new(at, message)
}

fn number(chars: &[char], start: usize) -> Result<(Token, usize), Error> {
    let bad = || Error::new(start, "malformed number");
    let digits_from = |from: usize, hex: bool| {
        chars[from..]
            .iter()
            .take_while(|c| {
                if hex {
                    c.is_ascii_hexdigit()
                } else {
                    c.is_ascii_digit()
                }
            })
            .count()
    };
    let unsigned = |end: usize| chars.get(end).is_some_and(|c| matches!(c, 'u' | 'U'));
    if chars[start] == '0' && matches!(chars.get(start + 1), Some('x' | 'X')) {
        let end = start + 2 + digits_from(start + 2, true);
        let digits: String = chars[start + 2..end].iter().collect();
        if unsigned(end) {
            return u64::from_str_radix(&digits, 16)
                .map(|n| (Token::Uint(n), end + 1))
                .map_err(|_| bad());
        }
        return i64::from_str_radix(&digits, 16)
            .map(|n| (Token::Int(n), end))
            .map_err(|_| bad());
    }
    let mut end = start + digits_from(start, false);
    let mut double = false;
    if chars.get(end) == Some(&'.') && chars.get(end + 1).is_some_and(char::is_ascii_digit) {
        double = true;
        end += 1 + digits_from(end + 1, false);
    }
    if matches!(chars.get(end), Some('e' | 'E')) {
        let sign = usize::from(matches!(chars.get(end + 1), Some('+' | '-')));
        let digits = digits_from(end + 1 + sign, false);
        if digits == 0 {
            return Err(bad());
        }
        double = true;
        end += 1 + sign + digits;
    }
    let text: String = chars[start..end].iter().collect();
    if double {
        return text
            .parse()
            .map(|n| (Token::Double(n), end))
            .map_err(|_| bad());
    }
    if unsigned(end) {
        return text
            .parse()
            .map(|n| (Token::Uint(n), end + 1))
            .map_err(|_| bad());
    }
    match text.parse::<i64>() {
        Ok(n) => Ok((Token::Int(n), end)),
        Err(_) if text == "9223372036854775808" => Ok((Token::MinIntMagnitude, end)),
        Err(_) => Err(Error::new(start, "int literal out of range")),
    }
}

/// The string literal whose first quote is at `start`: single or triple quoted, raw when `raw`.
fn string(chars: &[char], start: usize, raw: bool) -> Result<(String, usize), Error> {
    let quote = chars[start];
    let triple = chars.get(start + 1) == Some(&quote) && chars.get(start + 2) == Some(&quote);
    let width = if triple { 3 } else { 1 };
    let mut at = start + width;
    let mut out = String::new();
    loop {
        let unterminated = || Error::new(start, "unterminated string");
        let c = *chars.get(at).ok_or_else(unterminated)?;
        if c == quote && (!triple || chars.get(at + 1..at + 3) == Some(&[quote, quote])) {
            return Ok((out, at + width));
        }
        if !triple && matches!(c, '\n' | '\r') {
            return Err(unterminated());
        }
        if c == '\\' && !raw {
            let (escaped, next) = escape(chars, at)?;
            out.push(escaped);
            at = next;
        } else {
            out.push(c);
            at += 1;
        }
    }
}

/// The escape sequence at `at`, its backslash.
fn escape(chars: &[char], at: usize) -> Result<(char, usize), Error> {
    let bad = || Error::new(at, "bad escape sequence");
    let code = |from: usize, digits: usize, radix: u32| -> Result<(char, usize), Error> {
        let text: String = chars
            .get(from..from + digits)
            .ok_or_else(bad)?
            .iter()
            .collect();
        let n = u32::from_str_radix(&text, radix).map_err(|_| bad())?;
        char::from_u32(n)
            .map(|c| (c, from + digits))
            .ok_or_else(bad)
    };
    let simple = |c: char| Ok((c, at + 2));
    match chars.get(at + 1) {
        Some('n') => simple('\n'),
        Some('t') => simple('\t'),
        Some('r') => simple('\r'),
        Some('a') => simple('\u{7}'),
        Some('b') => simple('\u{8}'),
        Some('f') => simple('\u{c}'),
        Some('v') => simple('\u{b}'),
        Some(c @ ('\\' | '\'' | '"' | '`' | '?')) => simple(*c),
        Some('u') => code(at + 2, 4, 16),
        Some('U') => code(at + 2, 8, 16),
        Some('x' | 'X') => code(at + 2, 2, 16),
        Some('0'..='3') => code(at + 1, 3, 8),
        _ => Err(bad()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Result<Vec<Token>, (usize, String)> {
        tokens(source)
            .map(|tokens| tokens.into_iter().map(|(token, _)| token).collect())
            .map_err(|e| (e.at, e.message))
    }

    #[test]
    fn numbers_take_every_form() {
        assert_eq!(
            kinds("1 0x1F 2u 0xffU 1.5 .5 1e3 2E-2"),
            Ok(vec![
                Token::Int(1),
                Token::Int(31),
                Token::Uint(2),
                Token::Uint(255),
                Token::Double(1.5),
                Token::Double(0.5),
                Token::Double(1000.0),
                Token::Double(0.02),
                Token::End,
            ])
        );
        assert_eq!(
            kinds("9223372036854775808"),
            Ok(vec![Token::MinIntMagnitude, Token::End])
        );
        assert_eq!(kinds("9223372036854775809").map_err(|e| e.0), Err(0));
        assert_eq!(kinds("1e").map_err(|e| e.1), Err("malformed number".into()));
        assert_eq!(kinds("0x").map_err(|e| e.0), Err(0));
        assert_eq!(kinds("0xu").map_err(|e| e.0), Err(0));
        assert_eq!(kinds("99999999999999999999u").map_err(|e| e.0), Err(0));
        assert_eq!(kinds("1.").map(|t| t.len()), Ok(3));
    }

    #[test]
    fn strings_take_quotes_escapes_and_raw_forms() {
        assert_eq!(
            kinds(r#"'a' "b" '''c'd''' r'\n' R"\t" 'é\x41\101\n\?\`' "\U0001F600""#),
            Ok(vec![
                Token::String("a".into()),
                Token::String("b".into()),
                Token::String("c'd".into()),
                Token::String(r"\n".into()),
                Token::String(r"\t".into()),
                Token::String("éAA\n?`".into()),
                Token::String("😀".into()),
                Token::End,
            ])
        );
        assert_eq!(
            kinds("'''a\nb'''"),
            Ok(vec![Token::String("a\nb".into()), Token::End])
        );
        assert_eq!(
            kinds(r"'\r\a\b\f\v\t'"),
            Ok(vec![
                Token::String("\r\u{7}\u{8}\u{c}\u{b}\t".into()),
                Token::End
            ])
        );
        assert_eq!(kinds("'abc").map_err(|e| e.0), Err(0));
        assert_eq!(kinds("'a\nb'").map_err(|e| e.0), Err(0));
        assert_eq!(kinds(r"'\q'").map_err(|e| e.0), Err(1));
        assert_eq!(kinds(r"'\ud800'").map_err(|e| e.0), Err(1));
        assert_eq!(kinds(r"'\u12'").map_err(|e| e.0), Err(1));
        assert_eq!(kinds(r"'\x4g'").map_err(|e| e.0), Err(1));
        assert_eq!(
            kinds("b'abc'").map_err(|e| e.1),
            Err("bytes literals aren't in the CompMD CEL profile".into())
        );
    }

    #[test]
    fn words_punctuation_and_comments() {
        assert_eq!(
            kinds("a_1 >= _b // a comment"),
            Ok(vec![
                Token::Ident("a_1".into()),
                Token::Punct(">="),
                Token::Ident("_b".into()),
                Token::End,
            ])
        );
        assert_eq!(
            kinds("x / y"),
            Ok(vec![
                Token::Ident("x".into()),
                Token::Punct("/"),
                Token::Ident("y".into()),
                Token::End
            ])
        );
        assert_eq!(
            kinds("if").map_err(|e| e.1),
            Err("'if' is a reserved word".into())
        );
        assert_eq!(
            kinds("a = 1"),
            Err((2, "unexpected '='; write '==' to compare".into()))
        );
        assert_eq!(
            kinds("a & b").map_err(|e| e.1),
            Err("unexpected '&'; write '&&'".into())
        );
        assert_eq!(
            kinds("a | b").map_err(|e| e.1),
            Err("unexpected '|'; write '||'".into())
        );
        assert_eq!(
            kinds("é").map_err(|e| e.1),
            Err("unexpected character 'é'".into())
        );
    }

    #[test]
    fn positions_count_code_points() {
        let positions: Vec<usize> = tokens("'é' + x")
            .map(|t| t.into_iter().map(|(_, at)| at).collect())
            .unwrap_or_default();
        assert_eq!(positions, [0, 4, 6, 7]);
    }
}
