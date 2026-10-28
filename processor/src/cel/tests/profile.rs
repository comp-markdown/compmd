//! The CEL profile of spec §8.2, part by part: what each expression gives, or where it fails and when.

use crate::cel::{Program, Scope, Type, Value, Values};
use crate::host::Time;
use crate::md::{Md, Signatures};
use serde_json::{Value as Json, json};

/// What an expression gives.
#[derive(Debug, PartialEq)]
enum Outcome {
    /// This value, as JSON: `int` and `double` stay apart (`7` against `7.0`).
    Value(Json),
    /// A syntax or type error found before evaluating, at this 1-based column.
    Refused(usize),
    /// An error while evaluating, at this 1-based column.
    Failed(usize),
}

use Outcome::{Failed, Refused, Value as V};

/// UTC, and Berlin as UTC+2 (its summer time, which October 27 isn't in, so these tests don't depend on real data).
struct Zones;

impl Time for Zones {
    fn offset(&self, zone: &str, _at: i64) -> Option<i32> {
        match zone {
            "UTC" => Some(0),
            "Europe/Berlin" => Some(2 * 3600),
            _ => None,
        }
    }
}

/// 2026-10-27T09:00:00Z.
const NOW: i128 = 1_793_091_600 * 1_000_000_000;

/// The variables every expression sees: `md` (`doc`, `data`, `input`) and a definition's params.
fn variables() -> Vec<(&'static str, Json)> {
    vec![
        (
            "md",
            json!({
                "doc": { "title": "Release", "price": 12.0, "tags": ["b", "a"], "released": "2026-10-21" },
                "data": { "sales": [
                    { "region": "eu", "amount": 10.0, "year": 2025 },
                    { "region": "us", "amount": 5.0, "year": 2026, "note": "late" },
                    { "region": "eu", "amount": 2.5, "year": 2026 },
                ] },
                "input": { "rate": 0.5, "page": 1.0 },
            }),
        ),
        ("rows", json!([1.0, 2.0, 3.0])),
        ("count", json!(3.0)),
        ("price", json!(12.0)),
    ]
}

fn run_in(source: &str, lang: &str, tz: &str) -> Outcome {
    let (mut scope, mut values) = (Scope::default(), Values::default());
    for (name, json) in variables() {
        scope = scope.with(name, Type::of_json(&json));
        values = values.with(name, Value::from_json(&json));
    }
    let checked = Program::parse(source)
        .and_then(|program| program.check(&scope, &Signatures).map(|_| program));
    let program = match checked {
        Ok(program) => program,
        Err(error) => return Refused(error.at() + 1),
    };
    match program.evaluate(&values, &Md::new(NOW, lang, tz, &Zones)) {
        Ok(value) => V(value.to_json()),
        Err(error) => Failed(error.at() + 1),
    }
}

fn run(source: &str) -> Outcome {
    run_in(source, "en", "UTC")
}

/// Runs each row, and lists the ones that give something else.
fn check(rows: &[(&str, Outcome)]) {
    let wrong: Vec<String> = rows
        .iter()
        .filter_map(|(source, expected)| {
            let actual = run(source);
            (actual != *expected)
                .then(|| format!("{source}\n    expected {expected:?}\n    got      {actual:?}"))
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "{} of {} rows:\n{}",
        wrong.len(),
        rows.len(),
        wrong.join("\n")
    );
}

#[test]
fn operators() {
    check(&[
        ("1 + 2 * 3", V(json!(7))),
        ("\"a\" + \"b\"", V(json!("ab"))),
        ("10 / 4", V(json!(2))),
        ("10.0 / 4.0", V(json!(2.5))),
        ("7 % 3", V(json!(1))),
        ("!(1 < 2) || 3 >= 3", V(json!(true))),
        ("count > 2 ? \"many\" : \"few\"", V(json!("many"))),
        ("\"x\" in [\"x\", \"y\"]", V(json!(true))),
        ("[1, 2] + [3]", V(json!([1, 2, 3]))),
        ("{\"a\": 1}[\"a\"]", V(json!(1))),
    ]);
}

#[test]
fn ints_and_doubles_mix() {
    check(&[
        ("md.doc.price * 12", V(json!(144.0))),
        ("md.input.rate * 100 > 5", V(json!(true))),
        ("1 + 2.5", V(json!(3.5))),
        ("2 == 2.0", V(json!(true))),
        ("size(rows) / 2", V(json!(1))),
        ("double(size(rows)) / 2", V(json!(1.5))),
        ("1u + 2.0", Refused(1)),
    ]);
}

#[test]
fn list_indexes_are_ints() {
    check(&[
        ("rows[1]", V(json!(2.0))),
        ("rows[int(md.input.page)]", V(json!(2.0))),
        ("rows[md.input.page]", Refused(5)),
        ("rows[5]", Failed(5)),
    ]);
}

#[test]
fn macros() {
    check(&[
        ("has(md.doc.title)", V(json!(true))),
        ("has(md.doc.missing)", V(json!(false))),
        ("rows.all(r, r > 0.0)", V(json!(true))),
        ("rows.exists(r, r == 2.0)", V(json!(true))),
        ("rows.exists_one(r, r > 2.0)", V(json!(true))),
        ("rows.filter(r, r > 1.0)", V(json!([2.0, 3.0]))),
        ("md.data.sales.map(s, s.amount)", V(json!([10.0, 5.0, 2.5]))),
        (
            "md.data.sales.map(s, s.amount > 4.0, s.region)",
            V(json!(["eu", "us"])),
        ),
        (
            "md.doc.map(k, k)",
            V(json!(["title", "price", "tags", "released"])),
        ),
    ]);
}

#[test]
fn standard_functions() {
    check(&[
        ("size(\"héllo\")", V(json!(5))),
        (
            "\"abc\".contains(\"b\") && \"abc\".startsWith(\"a\") && \"abc\".endsWith(\"c\")",
            V(json!(true)),
        ),
        ("int(\"42\") + int(2.9)", V(json!(44))),
        ("double(\"1.5\")", V(json!(1.5))),
        ("string(3)", V(json!("3"))),
        (
            "string(timestamp(\"2026-10-21T10:00:00Z\") + duration(\"1h\"))",
            V(json!("2026-10-21T11:00:00Z")),
        ),
        ("duration(\"1h\") == duration(\"60m\")", V(json!(true))),
    ]);
}

#[test]
fn the_strings_extension() {
    check(&[
        ("\"abc\".charAt(1)", V(json!("b"))),
        (
            "\"banana\".indexOf(\"an\") + \"banana\".lastIndexOf(\"an\")",
            V(json!(4)),
        ),
        (
            "\"Ab\".lowerAscii() + \"Ab\".upperAscii()",
            V(json!("abAB")),
        ),
        ("\"a-b-c\".replace(\"-\", \"+\")", V(json!("a+b+c"))),
        ("\"a,b\".split(\",\")", V(json!(["a", "b"]))),
        ("\"hello\".substring(1, 3)", V(json!("el"))),
        ("\"  x  \".trim()", V(json!("x"))),
        ("[\"a\", \"b\"].join(\"-\")", V(json!("a-b"))),
    ]);
}

#[test]
fn what_the_profile_leaves_out_is_refused() {
    check(&[
        ("\"abc\".matches(\"a.c\")", Refused(7)),
        ("\"abc\".reverse()", Refused(7)),
        ("strings.quote(\"a\")", Refused(1)),
        ("\"%s\".format([\"a\"])", Refused(6)),
        ("math.greatest(1, 2)", Refused(1)),
        ("uint(1)", Refused(1)),
        ("'a'.quote()", Refused(5)),
        ("b\"abc\"", Refused(1)),
    ]);
}

#[test]
fn type_errors_are_found_before_evaluating() {
    check(&[
        ("\"a\" + 1", Refused(1)),
        ("md.doc.title * 2", Refused(1)),
        ("false && (\"a\" + 1 == 2)", Refused(11)),
        ("md.dco.title", Refused(4)),
        ("1 / 0", Failed(3)),
    ]);
}

#[test]
fn errors_in_and_and_or_are_absorbed() {
    check(&[
        ("1 / 0 == 1 || true", V(json!(true))),
        ("false && rows[9] == 1.0", V(json!(false))),
        ("rows[9] == 1.0 && false", V(json!(false))),
    ]);
}

#[test]
fn work_has_a_limit() {
    let ten = "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]";
    let three = format!("{ten}.map(a, {ten}.map(b, {ten}.map(c, a * b * c))).size()");
    let five =
        format!("{ten}.map(a, {ten}.map(b, {ten}.map(c, {ten}.map(d, {ten}.map(e, a))))).size()");
    check(&[(&three, V(json!(10))), (&five, Failed(157))]);
}

#[test]
fn numbers_are_written_as_javascript_writes_them() {
    check(&[
        ("string(3.0)", V(json!("3"))),
        ("string(12.5)", V(json!("12.5"))),
        ("string(1e6)", V(json!("1000000"))),
        ("string(1e21)", V(json!("1e+21"))),
        ("string(1.5e-7)", V(json!("1.5e-7"))),
        ("string(0.0 / 0.0)", V(json!("NaN"))),
        ("string(-1.0 / 0.0)", V(json!("-Infinity"))),
    ]);
}

#[test]
fn number_keys_match_by_value() {
    check(&[
        ("{1: 'a'}[1u]", V(json!("a"))),
        ("{1: 'a'}[1.0]", V(json!("a"))),
        ("1.0 in {1: 'a'}", V(json!(true))),
        ("{1: 'a'}[1.5]", Failed(9)),
        ("{1: 'a', 1u: 'b'}", Failed(10)),
        ("{1.0: 'a'}", Refused(2)),
        (
            "md.groupBy(md.data.sales, 'year')[2026].size()",
            V(json!(2)),
        ),
        (
            "md.groupBy(md.data.sales, 'year')[md.doc.price + 2014.0][0].region",
            V(json!("us")),
        ),
    ]);
}

#[test]
fn a_has_guard_lets_a_branch_read_a_field_the_type_lacks() {
    check(&[
        (
            "has(md.doc.subtitle) ? md.doc.subtitle : md.doc.title",
            V(json!("Release")),
        ),
        (
            "has(md.doc.subtitle) && md.doc.subtitle == 'x'",
            V(json!(false)),
        ),
        ("md.doc.subtitle", Refused(8)),
        (
            "has(md.doc.subtitle) ? md.doc.title : md.doc.subtitle",
            Refused(46),
        ),
        (
            "md.data.sales.filter(s, has(s.note)).map(s, s.note)",
            V(json!(["late"])),
        ),
    ]);
}

#[test]
fn md_functions() {
    check(&[
        ("md.sum(md.data.sales.map(s, s.amount))", V(json!(17.5))),
        (
            "md.daysBetween(\"2026-10-14\", \"2026-10-21\")",
            V(json!(7)),
        ),
        ("md.today()", V(json!("2026-10-27"))),
        ("md.today() >= md.doc.released", V(json!(true))),
        ("md.dateOf(md.now())", V(json!("2026-10-27"))),
        ("md.addDays(md.today(), -30)", V(json!("2026-09-27"))),
        ("md.addDays(md.today(), count)", V(json!("2026-10-30"))),
        ("md.startOf(md.today(), 'week')", V(json!("2026-10-25"))),
        ("md.startOf(md.today(), 'month')", V(json!("2026-10-01"))),
        ("md.startOf(md.today(), 'year')", V(json!("2026-01-01"))),
        (
            "md.date(md.doc.released, 'long')",
            V(json!("October 21, 2026")),
        ),
        (
            "md.date(md.doc.released, 'full')",
            V(json!("Wednesday, October 21, 2026")),
        ),
        (
            "md.date(md.doc.released, 'medium')",
            V(json!("Oct 21, 2026")),
        ),
        ("md.date(md.doc.released, 'short')", V(json!("10/21/26"))),
        ("md.date(md.now(), 'long')", V(json!("October 27, 2026"))),
        ("md.number(0.183 * 100, 0)", V(json!("18"))),
        ("md.number(1636.5, 2)", V(json!("1,636.50"))),
        ("md.number(2.5, 0)", V(json!("3"))),
        ("md.number(-2.5, 0)", V(json!("-3"))),
        (
            "md.currency(md.doc.price * 10, \"USD\")",
            V(json!("$120.00")),
        ),
        ("md.plural(count, \"item\", \"items\")", V(json!("items"))),
        ("md.plural(1, \"item\", \"items\")", V(json!("item"))),
        (
            "md.plural(2, {\"one\": \"file\", \"other\": \"files\"})",
            V(json!("files")),
        ),
        ("md.list(md.doc.tags)", V(json!("b and a"))),
        (
            "md.list(['Ana', 'Kenji', 'Priya'])",
            V(json!("Ana, Kenji, and Priya")),
        ),
        ("md.list(['Ana', 'Kenji'], 'or')", V(json!("Ana or Kenji"))),
        ("md.sort(md.doc.tags)", V(json!(["a", "b"]))),
        (
            "md.sortBy(md.data.sales, 'amount').map(s, s.amount)",
            V(json!([2.5, 5.0, 10.0])),
        ),
        ("md.reverse(rows)", V(json!([3.0, 2.0, 1.0]))),
        ("md.slice(rows, 1, 5)", V(json!([2.0, 3.0]))),
        ("md.min(rows) + md.max(rows)", V(json!(4.0))),
        ("md.max([])", V(json!(null))),
        ("md.sum([])", V(json!(0.0))),
        (
            "md.unique(md.data.sales.map(s, s.region))",
            V(json!(["eu", "us"])),
        ),
        (
            "md.groupBy(md.data.sales, 'region').map(k, k)",
            V(json!(["eu", "us"])),
        ),
        ("md.todya()", Refused(4)),
        ("md.sum(md.doc.tags)", Refused(4)),
        ("md.daysBetween(md.today(), 'soon')", Failed(4)),
    ]);
}

#[test]
fn timestamp_getters_read_md_tz_or_the_zone_given() {
    let t = "timestamp('2026-10-21T22:30:15.250Z')";
    let get = |getter: &str, tz: &str| run_in(&format!("{t}.{getter}"), "en", tz);
    // 2026-10-21 is a Wednesday, the 294th day of the year.
    let in_utc = [
        ("getFullYear()", 2026),
        ("getMonth()", 9),
        ("getDate()", 21),
        ("getDayOfMonth()", 20),
        ("getDayOfWeek()", 3),
        ("getDayOfYear()", 293),
        ("getHours()", 22),
        ("getMinutes()", 30),
        ("getSeconds()", 15),
        ("getMilliseconds()", 250),
    ];
    for (getter, expected) in in_utc {
        assert_eq!(get(getter, "UTC"), V(json!(expected)), "{getter}");
    }
    // Two hours east, it is already Thursday the 22nd.
    assert_eq!(get("getDate()", "Europe/Berlin"), V(json!(22)));
    assert_eq!(get("getDayOfWeek()", "Europe/Berlin"), V(json!(4)));
    assert_eq!(get("getHours()", "Europe/Berlin"), V(json!(0)));
    assert_eq!(get("getHours('+05:30')", "UTC"), V(json!(4)));
    assert_eq!(get("getMinutes('-01:15')", "UTC"), V(json!(15)));
    assert_eq!(get("getHours('UTC')", "Europe/Berlin"), V(json!(22)));
    assert_eq!(get("getHours('Mars/Olympus')", "UTC"), Failed(39));
    assert_eq!(get("getHours()", "Mars/Olympus"), Failed(39));
    assert_eq!(get("getHours(1)", "UTC"), Refused(39));
}

#[test]
fn duration_getters_count_whole_units() {
    check(&[
        ("duration('90m').getHours()", V(json!(1))),
        ("duration('90m').getMinutes()", V(json!(90))),
        ("duration('1.5s').getSeconds()", V(json!(1))),
        ("duration('1.5s').getMilliseconds()", V(json!(1500))),
        ("duration('-90m').getHours()", V(json!(-1))),
        ("duration('1h').getDate()", Refused(16)),
    ]);
}

#[test]
fn md_functions_follow_the_language_and_the_time_zone() {
    assert_eq!(
        run_in("md.date('2026-11-02', 'long')", "de", "UTC"),
        V(json!("2. November 2026"))
    );
    assert_eq!(
        run_in("md.number(1636.5, 2)", "de", "UTC"),
        V(json!("1.636,50"))
    );
    assert_eq!(
        run_in("md.currency(120.0, 'EUR')", "de", "UTC"),
        V(json!("120,00\u{a0}€"))
    );
    assert_eq!(
        run_in("md.list(['Ana', 'Kenji'], 'or')", "de", "UTC"),
        V(json!("Ana oder Kenji"))
    );
    assert_eq!(
        run_in("md.startOf(md.today(), 'week')", "de", "UTC"),
        V(json!("2026-10-26"))
    );
    assert_eq!(
        run_in(
            "md.plural(3, {'one': 'файл', 'few': 'файла', 'many': 'файлов', 'other': 'файла'})",
            "ru",
            "UTC"
        ),
        V(json!("файла"))
    );
    assert_eq!(
        run_in(
            "md.plural(5, {'one': 'файл', 'few': 'файла', 'many': 'файлов', 'other': 'файла'})",
            "ru",
            "UTC"
        ),
        V(json!("файлов"))
    );
    assert_eq!(
        run_in(
            "md.dateOf(timestamp('2026-10-27T23:30:00Z'))",
            "en",
            "Europe/Berlin"
        ),
        V(json!("2026-10-28"))
    );
    assert_eq!(run_in("md.today()", "en", "Mars/Olympus"), Failed(4));
    assert_eq!(
        run_in("md.number(1.5, 0)", "not a tag", "UTC"),
        V(json!("2"))
    );
}
