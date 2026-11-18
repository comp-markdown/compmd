//! The render context: the time and time zone a document is read in, and the inputs the reader set.

use super::*;

#[test]
fn md_today_is_the_date_where_the_document_is_read() {
    let text = "Today is :value[the date]{of=(string(md.today()))}.\n";
    let today = |host: &Host| {
        shown(&host.process(text))
            .trim_start_matches("Today is ")
            .trim_end_matches('.')
            .to_string()
    };
    assert_eq!(today(&Host::default()), "2026-11-02");
    // At 20:30 UTC it is still the 2nd in UTC, and already the 3rd in Tokyo.
    let evening = |tz: &str| Host {
        context: RenderContext {
            tz: tz.into(),
            now: parse_time("2026-11-02T20:30:00Z").expect("a time"),
            ..Host::default().context
        },
        ..Host::default()
    };
    assert_eq!(today(&evening("UTC")), "2026-11-02");
    assert_eq!(today(&evening("Asia/Tokyo")), "2026-11-03");
}

#[test]
fn an_inputs_value_from_the_host_is_used_and_one_of_the_wrong_type_isnt() {
    let text = "---\nmd:\n  inputs:\n    region: eu | us = eu\n---\n\n{{md.input.region}}\n";
    let given = |value: Value| Host {
        context: RenderContext {
            inputs: serde_json::Map::from_iter([("region".to_string(), value)]),
            ..Host::default().context
        },
        ..Host::default()
    };
    assert_eq!(shown(&given(json!("us")).process(text)), "us");
    assert_eq!(shown(&given(json!(42)).process(text)), "eu", "its default");
}
