//! Functions and conversions, time in range, names that don't exist, the limits, and the kinds of error.

use super::*;

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
    let md = Md::new(NOW, "en", "UTC", &());
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
        format!("column {}: {}", budget.at() + 1, budget.message())
    );
}
