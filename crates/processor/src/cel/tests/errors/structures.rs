//! Maps and lists: building them, their fields and indexes, and the macros that range over them.

use super::*;

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
