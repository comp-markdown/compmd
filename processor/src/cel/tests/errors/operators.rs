//! Operators: logic and conditions, comparisons and arithmetic, numbers that overflow, and grammar types.

use super::*;

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
