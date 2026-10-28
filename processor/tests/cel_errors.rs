//! What F024 says, and where: type errors the checker finds, and errors only evaluation finds, as when a value's type
//! is known only then (`dyn`, as in data whose records differ).

use compmd::cel::{BUDGET, Error, MAX_DEPTH, Program, Scope, Type, Value, Values};
use compmd::md::{Md, Signatures, TimeZones};
use serde_json::{Value as Json, json};

struct Utc;

impl TimeZones for Utc {
    fn offset(&self, zone: &str, _at: i64) -> Option<i32> {
        (zone == "UTC").then_some(0)
    }
}

/// 2026-10-27T09:00:00Z.
const NOW: i128 = 1_793_091_600 * 1_000_000_000;

/// The scope every check sees: `md.doc`, a list of `double`s, a `length` param, and `x`, `y` of type `dyn`.
fn scope() -> Scope {
    Scope::default()
        .with(
            "md",
            Type::of_json(&json!({ "doc": { "title": "Release", "price": 12.0 } })),
        )
        .with("rows", Type::list(Type::Double))
        .with(
            "gap",
            Type::declared(&compmd::tree::DeclaredType::Name("length".into())),
        )
        .with("x", Type::Dyn)
        .with("y", Type::Dyn)
}

/// The type error in `source`: its column and message.
fn refused(source: &str) -> (usize, String) {
    match Program::parse(source).and_then(|program| program.check(&scope(), &Signatures)) {
        Ok(ty) => panic!("{source} checks, as {ty}"),
        Err(error) => (error.column(), error.message().to_string()),
    }
}

/// The type of `source`.
fn type_of(source: &str) -> String {
    match Program::parse(source).and_then(|program| program.check(&scope(), &Signatures)) {
        Ok(ty) => ty.to_string(),
        Err(error) => panic!("{source}: {error}"),
    }
}

/// Evaluates `source` with `x` and `y` as given, without checking it first.
fn evaluate(source: &str, x: Json, y: Json) -> Result<Json, (usize, String)> {
    let program = Program::parse(source).map_err(|e| (e.column(), e.message().to_string()))?;
    let md = json!({ "doc": { "title": "Release" } });
    let values = [("md", md), ("x", x), ("y", y)]
        .into_iter()
        .fold(Values::default(), |values, (name, json)| {
            values.with(name, Value::from_json(&json))
        });
    program
        .evaluate(&values, &Md::new(NOW, "en", "UTC", &Utc))
        .map(|value| value.to_json())
        .map_err(|e| (e.column(), e.message().to_string()))
}

/// The evaluation error of `source` with `x`.
fn fails(source: &str, x: Json) -> (usize, String) {
    match evaluate(source, x, Json::Null) {
        Ok(value) => panic!("{source} gives {value}"),
        Err(error) => error,
    }
}

fn says(column: usize, message: &str) -> (usize, String) {
    (column, message.to_string())
}

#[test]
fn logic_and_conditions_take_bools() {
    assert_eq!(refused("1 && true"), says(3, "'&&' takes bools, not int"));
    assert_eq!(
        refused("true || 'a'"),
        says(6, "'||' takes bools, not string")
    );
    assert_eq!(
        refused("1 ? 2 : 3"),
        says(3, "the condition of '?:' must be a bool, not int")
    );
    assert_eq!(refused("!1"), says(1, "'!' can't be applied to int"));
    assert_eq!(refused("-'a'"), says(1, "'-' can't be applied to string"));
    assert_eq!(type_of("x ? 1 : 'a'"), "dyn");
    assert_eq!(type_of("-x"), "dyn");
    assert_eq!(
        fails("x ? 1 : 2", json!(1)),
        says(3, "expected a bool, not double")
    );
    assert_eq!(
        fails("!x", json!(1)),
        says(1, "'!' can't be applied to double")
    );
    assert_eq!(
        fails("-x", json!("a")),
        says(1, "'-' can't be applied to string")
    );
    assert_eq!(
        fails("x && true", json!(1)),
        says(3, "expected a bool, not double")
    );
}

#[test]
fn maps_and_lists_are_built_from_what_fits() {
    assert_eq!(
        refused("{1.5: 'a'}"),
        says(
            2,
            "a map key must be an int, uint, bool or string, not double"
        )
    );
    assert_eq!(type_of("{'a': 1, 'b': 'c'}"), "map(string, dyn)");
    assert_eq!(type_of("[1] + []"), "list(int)");
    assert_eq!(type_of("[] + ['a']"), "list(string)");
    assert_eq!(type_of("[1] + ['a']"), "list(dyn)");
    assert_eq!(
        fails("{x: 1}", json!(1.0)),
        says(2, "a map key can't be a double")
    );
    assert_eq!(
        fails("{'a': 1, 'a': 2}", json!(null)),
        says(10, "the same key twice")
    );
    assert_eq!(
        evaluate("{true: 1, 2u: 'b'}", json!(null), json!(null)),
        Ok(json!({ "true": 1, "2": "b" }))
    );
}

#[test]
fn fields_and_indexes() {
    assert_eq!(refused("rows.x"), says(6, "list(double) has no fields"));
    assert_eq!(
        refused("has(rows.x)"),
        says(1, "has() needs a map, not list(double)")
    );
    assert_eq!(
        refused("md.doc['titel']"),
        says(7, "unknown field 'titel'; did you mean 'title'?")
    );
    assert_eq!(type_of("md.doc[md.doc.title]"), "dyn");
    assert_eq!(type_of("md.doc['title']"), "string");
    assert_eq!(type_of("{'a': 1}.a"), "int");
    assert_eq!(type_of("x.y.z"), "dyn");
    assert_eq!(type_of("x[0]"), "dyn");
    assert_eq!(type_of("has(x.y)"), "bool");
    assert_eq!(
        refused("{'a': 1}[1]"),
        says(9, "map(string, int) can't be indexed with int")
    );
    assert_eq!(refused("1[0]"), says(2, "int can't be indexed with int"));
    assert_eq!(
        refused("rows['a']"),
        says(5, "a list index must be an int, not string")
    );
    assert_eq!(fails("x.y", json!(1)), says(3, "double has no fields"));
    assert_eq!(fails("x.y", json!({})), says(3, "no field 'y'"));
    assert_eq!(
        fails("has(x.y)", json!(1)),
        says(1, "has() needs a map, not double")
    );
    assert_eq!(
        fails("x[0]", json!("abc")),
        says(2, "string can't be indexed")
    );
    assert_eq!(
        fails("x['a']", json!([1])),
        says(2, "a list index must be an int, not string")
    );
    assert_eq!(fails("x['b']", json!({ "a": 1 })), says(2, "no key \"b\""));
    assert_eq!(
        fails("x[-1]", json!([1])),
        says(2, "index -1 is out of range for a list of 1")
    );
}

#[test]
fn macros_range_over_lists_and_maps() {
    assert_eq!(
        refused("1.5.all(v, true)"),
        says(5, "all() needs a list or a map, not double")
    );
    assert_eq!(
        refused("rows.all(r, 1)"),
        says(6, "all() needs a bool expression, not int")
    );
    assert_eq!(
        refused("rows.map(r, 1, r)"),
        says(6, "map() needs a bool expression, not int")
    );
    assert_eq!(
        refused("rows.filter(r, r)"),
        says(6, "filter() needs a bool expression, not double")
    );
    assert_eq!(type_of("{'a': 1}.filter(k, k == 'a')"), "list(string)");
    assert_eq!(type_of("md.doc.filter(k, k != '')"), "list(string)");
    assert_eq!(type_of("x.map(v, v)"), "list(dyn)");
    assert_eq!(
        fails("x.all(v, true)", json!(1)),
        says(3, "all() needs a list or a map, not double")
    );
    assert_eq!(
        fails("x.exists_one(v, v)", json!([1])),
        says(3, "expected a bool, not double")
    );
    assert_eq!(
        fails("x.map(v, v, v)", json!([1])),
        says(3, "expected a bool, not double")
    );
    assert_eq!(
        evaluate("x.exists_one(v, v > 1.0)", json!([2, 3]), json!(null)),
        Ok(json!(false))
    );
    assert_eq!(
        evaluate("x.all(v, v > 1.0)", json!([2, 3]), json!(null)),
        Ok(json!(true))
    );
    assert_eq!(
        evaluate("x.exists(v, v > 5.0)", json!([2, 3]), json!(null)),
        Ok(json!(false))
    );
    assert_eq!(
        fails("x.exists(v, v.a)", json!([{ "b": true }, { "a": false }])),
        says(15, "no field 'a'")
    );
}

#[test]
fn operators_take_the_types_cel_gives_them() {
    assert_eq!(
        refused("1 == 'a'"),
        says(1, "'==' can't be applied to int and string")
    );
    assert_eq!(
        refused("'a' < 1"),
        says(1, "'<' can't be applied to string and int")
    );
    assert_eq!(
        refused("x < [1]"),
        says(1, "'<' can't be applied to dyn and list(int)")
    );
    assert_eq!(
        refused("1 in 'abc'"),
        says(1, "'in' can't be applied to int and string")
    );
    assert_eq!(
        refused("1 in md.doc"),
        says(1, "'in' can't be applied to int and map(string, dyn)")
    );
    assert_eq!(
        refused("x % 1.5"),
        says(
            1,
            "'%' can't be applied to dyn and double; wrap the number in int(…) to drop its fraction"
        )
    );
    assert_eq!(
        refused("x - 'a'"),
        says(1, "'-' can't be applied to dyn and string")
    );
    assert_eq!(
        refused("x * 'a'"),
        says(1, "'*' can't be applied to dyn and string")
    );
    assert_eq!(
        refused("rows[0] + 'a'"),
        says(1, "'+' can't be applied to double and string")
    );
    assert_eq!(type_of("x + 'a'"), "dyn");
    assert_eq!(type_of("x < 'a' && 1u < 2.0"), "bool");
    assert_eq!(
        type_of("'title' in md.doc && x in y && 1 in {1: 2}"),
        "bool"
    );
    assert_eq!(
        type_of("timestamp('2026-10-21T10:00:00Z') - duration('1h')"),
        "timestamp"
    );
    assert_eq!(
        type_of("duration('1h') + timestamp('2026-10-21T10:00:00Z')"),
        "timestamp"
    );
    assert_eq!(
        type_of("timestamp('2026-10-21T10:00:00Z') - timestamp('2026-10-21T10:00:00Z')"),
        "duration"
    );
    assert_eq!(type_of("-duration('1h') + duration('1m')"), "duration");
    assert_eq!(
        fails("x + 1", json!("a")),
        says(3, "'+' can't be applied to string and int")
    );
    assert_eq!(
        fails("x < 1", json!("a")),
        says(3, "'<' can't be applied to string and int")
    );
    assert_eq!(
        fails("1 in x", json!(1)),
        says(3, "'in' needs a list or a map, not double")
    );
    assert_eq!(
        fails("x % 2.0", json!(1)),
        says(3, "'%' can't be applied to double and double")
    );
    assert_eq!(
        evaluate("x / 0.0 < 1.0", json!(0.0), json!(null)),
        Ok(json!(false))
    );
    assert_eq!(
        evaluate("[x, y] == [y, x]", json!(1), json!("a")),
        Ok(json!(false))
    );
    assert_eq!(
        evaluate(
            "2.0 in {2: 'a'} && !(2.5 in {2: 'a'})",
            json!(null),
            json!(null)
        ),
        Ok(json!(true))
    );
}

#[test]
fn numbers_overflow_and_divide_by_zero_as_errors() {
    let run = |source: &str| evaluate(source, Json::Null, Json::Null);
    assert_eq!(
        run("9223372036854775807 + 1"),
        Err(says(21, "int overflow"))
    );
    assert_eq!(
        run("-9223372036854775808 / -1"),
        Err(says(22, "int overflow"))
    );
    assert_eq!(run("-(-9223372036854775808)"), Err(says(1, "int overflow")));
    assert_eq!(run("7 % 0"), Err(says(3, "division by zero")));
    assert_eq!(
        run("-9223372036854775808 % -1"),
        Err(says(22, "int overflow"))
    );
    assert_eq!(run("1u - 2u"), Err(says(4, "uint overflow")));
    assert_eq!(run("1u / 0u"), Err(says(4, "division by zero")));
    assert_eq!(run("7u % 4u + 7u * 2u"), Ok(json!(17)));
    assert_eq!(run("-9223372036854775808"), Ok(json!(i64::MIN)));
    assert_eq!(run("1.0 / 0.0 > 1.0"), Ok(json!(true)));
    assert_eq!(
        run("1u + 2.0"),
        Err(says(4, "'+' can't be applied to uint and double"))
    );
    assert_eq!(run("2.0 - 0.5 + (3 - 1.5)"), Ok(json!(3.0)));
    assert_eq!(
        run("-(1.5) + int(2) + (-(2.5) * 2u > 1u ? 1 : 0)"),
        Err(says(27, "'*' can't be applied to double and uint"))
    );
    assert_eq!(
        run("[-(1.5), int(2), 1u < 2u, -duration('1s')]"),
        Ok(json!([-1.5, 2, true, "-1s"]))
    );
}

#[test]
fn time_stays_in_range() {
    let run = |source: &str| evaluate(source, Json::Null, Json::Null);
    assert_eq!(
        run("timestamp('9999-12-31T23:59:59Z') + duration('1s')"),
        Err(says(35, "timestamp out of range"))
    );
    assert_eq!(
        run("timestamp('0001-01-01T00:00:00Z') - duration('1s')"),
        Err(says(35, "timestamp out of range"))
    );
    assert_eq!(
        run("timestamp('9999-12-31T23:59:59Z') - timestamp('0001-01-01T00:00:00Z')"),
        Ok(json!("315537897599s"))
    );
    assert_eq!(
        run("duration('2000000h') + duration('2000000h') - duration('1s')"),
        Ok(json!("14399999999s"))
    );
    assert_eq!(
        run("-(duration('87600000h') + duration('2000000h'))"),
        Err(says(25, "duration out of range"))
    );
    assert_eq!(
        run("timestamp(timestamp('2026-10-21T10:00:00Z')) == timestamp('2026-10-21T10:00:00Z')"),
        Ok(json!(true))
    );
    assert_eq!(run("duration(duration('1h'))"), Ok(json!("3600s")));
    assert_eq!(run("int(timestamp('1970-01-01T00:01:00Z'))"), Ok(json!(60)));
    assert_eq!(
        run("timestamp('2026-10-21')"),
        Err(says(
            1,
            "'2026-10-21' is not an RFC 3339 timestamp in CEL's range"
        ))
    );
    assert_eq!(
        run("duration('1d')"),
        Err(says(1, "'1d' is not a duration such as '1h30m'"))
    );
}

#[test]
fn conversions_fail_where_they_must() {
    assert_eq!(
        fails("int(x)", json!(9.3e18)),
        says(1, "9300000000000000000 doesn't fit an int")
    );
    assert_eq!(
        fails("int(18446744073709551615u)", json!(null)),
        says(1, "18446744073709551615 doesn't fit an int")
    );
    assert_eq!(fails("int(x)", json!("4x")), says(1, "'4x' is not an int"));
    assert_eq!(
        fails("int(x)", json!(true)),
        says(1, "int() doesn't take (bool)")
    );
    assert_eq!(
        fails("double(x)", json!("x")),
        says(1, "'x' is not a double")
    );
    assert_eq!(
        fails("double(x)", json!(true)),
        says(1, "double() doesn't take (bool)")
    );
    assert_eq!(
        fails("string(x)", json!(null)),
        says(1, "string() doesn't take (null)")
    );
    assert_eq!(
        fails("size(x)", json!(1)),
        says(1, "size() doesn't take (double)")
    );
    assert_eq!(
        fails("timestamp(x)", json!(1)),
        says(1, "timestamp() doesn't take (double)")
    );
    assert_eq!(
        fails("x.join()", json!([1])),
        says(3, "join() needs strings, not double")
    );
    assert_eq!(
        fails("x.join()", json!(1)),
        says(3, "double.join() doesn't take ()")
    );
    assert_eq!(
        fails("x.charAt('a')", json!("abc")),
        says(3, "string.charAt() doesn't take (string)")
    );
    assert_eq!(
        evaluate(
            "double(1u) + double(x) + double(2)",
            json!(1.5),
            json!(null)
        ),
        Ok(json!(4.5))
    );
    assert_eq!(
        evaluate(
            "int(x) + int(-2.9)",
            json!(-9_223_372_036_854_775_808.0),
            json!(null)
        ),
        Err(says(8, "int overflow"))
    );
    assert_eq!(
        evaluate("string(x) + string(y)", json!(1e21), json!(false)),
        Ok(json!("1e+21false"))
    );
    assert_eq!(
        evaluate("{'a': 1}.size() + size(x)", json!({ "b": 1 }), json!(null)),
        Ok(json!(2))
    );
    assert_eq!(
        evaluate("'ab'.lastIndexOf('abc')", json!(null), json!(null)),
        Ok(json!(-1))
    );
    assert_eq!(
        evaluate("'abc'.split('', 2)", json!(null), json!(null)),
        Ok(json!(["a", "bc"]))
    );
    for (value, kind) in [
        ("null", "null"),
        ("timestamp('2026-10-21T10:00:00Z')", "timestamp"),
        ("duration('1s')", "duration"),
        ("[]", "list"),
        ("true", "bool"),
        ("1u", "uint"),
    ] {
        assert_eq!(
            evaluate(&format!("{value}.y"), json!(null), json!(null)),
            Err((value.len() + 2, format!("{kind} has no fields")))
        );
    }
    assert_eq!(
        evaluate(
            "string(duration('90s')) + string(x)",
            json!(null),
            json!(null)
        ),
        Err(says(27, "string() doesn't take (null)"))
    );
}

#[test]
fn functions_take_the_arguments_they_are_declared_with() {
    assert_eq!(refused("size(1)"), says(1, "size() doesn't take (int)"));
    assert_eq!(refused("int(true)"), says(1, "int() doesn't take (bool)"));
    assert_eq!(
        refused("'a'.int()"),
        says(5, "string.int() doesn't take ()")
    );
    assert_eq!(
        refused("'abc'.charAt(md.doc.price)"),
        says(
            7,
            "string.charAt() doesn't take (double); wrap the number in int(…) to drop its fraction"
        )
    );
    assert_eq!(
        refused("md.doc.price % 2"),
        says(
            1,
            "'%' can't be applied to double and int; wrap the number in int(…) to drop its fraction"
        )
    );
    assert_eq!(
        refused("rows[md.doc.price]"),
        says(
            5,
            "a list index must be an int, not double; wrap the number in int(…) to drop its fraction"
        )
    );
    assert_eq!(
        refused("'a'.lowerascii()"),
        says(
            5,
            "unknown function 'lowerascii'; did you mean 'lowerAscii'?"
        )
    );
    assert_eq!(
        refused("'a'.frobnicate()"),
        says(5, "unknown function 'frobnicate'")
    );
    assert_eq!(
        refused("'a'.matches('a')"),
        says(5, "matches() isn't in the CompMD CEL profile")
    );
    assert_eq!(
        refused("md.today(1)"),
        says(4, "md.today() doesn't take (int)")
    );
    assert_eq!(
        refused("md.xyzzy()"),
        says(4, "unknown function 'md.xyzzy'")
    );
    assert_eq!(
        refused("md.todya()"),
        says(4, "unknown function 'md.todya'; did you mean 'md.today'?")
    );
    assert_eq!(
        refused("md.dco"),
        says(4, "unknown field 'dco'; did you mean 'doc'?")
    );
    assert_eq!(
        refused("row[0]"),
        says(1, "unknown name 'row'; did you mean 'rows'?")
    );
    assert_eq!(refused("rows.map(r, q)"), says(13, "unknown name 'q'"));
    assert_eq!(
        type_of("['a'].join() + 'a'.substring(1) + 'a'.trim()"),
        "string"
    );
}

#[test]
fn grammar_types_take_only_equality_and_concatenation() {
    assert_eq!(type_of("gap == '1rem' && gap != x"), "bool");
    assert_eq!(type_of("gap + ' auto'"), "string");
    assert_eq!(type_of("'calc(' + gap + ')'"), "string");
    assert_eq!(
        refused("gap < 'x'"),
        says(1, "'<' can't be applied to length and string")
    );
    assert_eq!(
        refused("gap.size()"),
        says(5, "length.size() doesn't take ()")
    );
    assert_eq!(
        refused("gap.startsWith('1')"),
        says(5, "length.startsWith() doesn't take (string)")
    );
}

#[test]
fn unchecked_programs_still_fail_safely() {
    assert_eq!(
        fails("nowhere", json!(null)),
        says(1, "unknown name 'nowhere'")
    );
    assert_eq!(
        fails("md.doc.missing", json!(null)),
        says(8, "no field 'missing'")
    );
}

#[test]
fn the_limits_are_public() {
    assert_eq!((BUDGET, MAX_DEPTH), (100_000, 64));
    let doubling = (0..30).fold("'ab'".to_string(), |s, _| format!("[{s}].map(v, v + v)[0]"));
    assert_eq!(
        evaluate(&doubling, Json::Null, Json::Null).map_err(|(_, message)| message),
        Err("the expression takes too much work".into())
    );
    let error = Program::parse("1 +").expect_err("incomplete");
    assert_eq!(error.to_string(), "column 4: the expression ends too early");
}

#[test]
fn errors_say_which_kind_they_are() {
    let parse = |source: &str| Program::parse(source).expect("parses");
    assert!(matches!(
        Program::parse("1 +"),
        Err(Error::Syntax { at: 3, .. })
    ));
    assert!(matches!(
        parse("'a' + 1").check(&scope(), &Signatures),
        Err(Error::Type { at: 0, .. })
    ));
    let md = Md::new(NOW, "en", "UTC", &Utc);
    assert!(matches!(
        parse("1 / 0").evaluate(&Values::default(), &md),
        Err(Error::Evaluation { at: 2, .. })
    ));
    let ten = "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]";
    let five =
        format!("{ten}.map(a, {ten}.map(b, {ten}.map(c, {ten}.map(d, {ten}.map(e, a))))).size()");
    let budget = parse(&five)
        .evaluate(&Values::default(), &md)
        .expect_err("too much work");
    assert!(matches!(budget, Error::Budget { .. }));
    assert_eq!(budget.message(), "the expression takes too much work");
    assert_eq!(
        budget.to_string(),
        format!("column {}: {}", budget.column(), budget.message())
    );
}
