//! A signature fence's text (spec §7.1, §7.3), written from what a component's props declare, or brought up to date
//! with them line by line: the writer's own declarations, comments and other keys (`renders`, `print`) stay as they
//! are written.

use super::declarations::Declared;

/// The text of a new signature fence for `declared`, its lines.
pub(super) fn generated(declared: &Declared) -> Vec<String> {
    let mut lines = params_section(
        declared
            .params
            .iter()
            .flat_map(|param| param.lines.clone())
            .collect(),
    );
    lines.extend(slots_section(&declared.slots));
    if lines.is_empty() {
        lines.push("params: {}".into());
    }
    lines
}

/// `body`, a signature fence's lines, with its params and slots those `declared` gives: those it declares that a prop
/// still has kept as written, those with no prop dropped (unless the component takes others, `rest`), the new ones
/// added after them; its slots written again when they aren't the props' (`slots`, what it declares now).
pub(super) fn updated(
    body: &[&str],
    declared: &Declared,
    rest: bool,
    slots: &[String],
) -> Vec<String> {
    let sections = sections(body);
    let mut lines = Vec::new();
    let mut wrote_params = false;
    let mut wrote_slots = false;
    for section in &sections {
        match section.key.as_deref() {
            Some("params") => {
                lines.extend(updated_params(&section.lines, declared, rest));
                wrote_params = true;
            }
            Some("slots") => {
                if same_names(slots, &declared.slots) {
                    lines.extend(section.lines.iter().map(|line| (*line).to_owned()));
                } else {
                    lines.extend(slots_section(&declared.slots));
                }
                wrote_slots = true;
            }
            _ => lines.extend(section.lines.iter().map(|line| (*line).to_owned())),
        }
    }
    if !wrote_params {
        let new = params_section(
            declared
                .params
                .iter()
                .flat_map(|param| param.lines.clone())
                .collect(),
        );
        lines.splice(0..0, new);
    }
    if !wrote_slots {
        lines.extend(slots_section(&declared.slots));
    }
    lines
}

/// The `params` key with `entries` under it; nothing without entries.
fn params_section(entries: Vec<String>) -> Vec<String> {
    if entries.is_empty() {
        return entries;
    }
    let mut lines = vec!["params:".to_owned()];
    lines.extend(entries);
    lines
}

/// The `slots` key listing `slots`; nothing without slots.
fn slots_section(slots: &[String]) -> Vec<String> {
    if slots.is_empty() {
        return Vec::new();
    }
    vec![format!("slots: [{}]", slots.join(", "))]
}

/// Whether two lists of slot names hold the same names.
fn same_names(a: &[String], b: &[String]) -> bool {
    let sorted = |names: &[String]| {
        let mut names = names.to_vec();
        names.sort();
        names
    };
    sorted(a) == sorted(b)
}

/// A top-level key of a signature with its lines, the lines under it among them; `None` for what is before the first.
struct Section<'b> {
    key: Option<String>,
    lines: Vec<&'b str>,
}

/// `body`'s top-level keys, in order.
fn sections<'b>(body: &[&'b str]) -> Vec<Section<'b>> {
    let mut sections = vec![Section {
        key: None,
        lines: Vec::new(),
    }];
    for line in body {
        let key = (!line.starts_with([' ', '\t', '#']) && !line.trim().is_empty())
            .then(|| line.split(':').next().unwrap_or_default().trim().to_owned());
        match key {
            Some(key) => sections.push(Section {
                key: Some(key),
                lines: vec![line],
            }),
            None => sections.last_mut().expect("one at least").lines.push(line),
        }
    }
    sections
}

/// The lines of the `params` section `section` brought up to date. A section written in flow style (`params: {…}`),
/// which lines can't be kept from, is written again from the props.
fn updated_params(section: &[&str], declared: &Declared, rest: bool) -> Vec<String> {
    let generated = || {
        params_section(
            declared
                .params
                .iter()
                .flat_map(|param| param.lines.clone())
                .collect(),
        )
    };
    // A section has its key's line first.
    let (header, entries) = (section[0], &section[1..]);
    if header.trim_end() != "params:" {
        return generated();
    }
    let mut kept: Vec<String> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for entry in entries_of(entries) {
        let has_prop = declared.params.iter().any(|param| param.name == entry.name);
        if has_prop || rest {
            kept.extend(entry.lines.iter().map(|line| (*line).to_owned()));
            names.push(entry.name);
        }
    }
    let written = section.join("\n");
    for param in &declared.params {
        let placeholder = written.contains(&format!("# {}:", param.name));
        if !names.contains(&param.name) && !placeholder {
            kept.extend(param.lines.iter().cloned());
        }
    }
    params_section(kept)
}

/// A param's lines in a `params` section: the comments above it, its key's line and the lines under it.
struct Entry<'b> {
    name: String,
    lines: Vec<&'b str>,
}

/// The entries of a `params` section's lines; a comment of its own with no key after it, such as a param left for
/// the writer, is an entry of the name it starts with.
fn entries_of<'b>(lines: &[&'b str]) -> Vec<Entry<'b>> {
    let indent = |line: &str| line.len() - line.trim_start().len();
    let child = lines
        .iter()
        .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map_or(2, |line| indent(line));
    let mut entries: Vec<Entry<'b>> = Vec::new();
    let mut pending: Vec<&'b str> = Vec::new();
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            pending.push(line);
        } else if indent(line) == child {
            let name = trimmed
                .split(':')
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned();
            pending.push(line);
            entries.push(Entry {
                name,
                lines: std::mem::take(&mut pending),
            });
        } else if let Some(last) = entries.last_mut() {
            last.lines.push(line);
        }
    }
    // Comments after the last param: each a param left for the writer, by the name it starts with.
    for line in pending {
        let name = line
            .trim_start()
            .trim_start_matches('#')
            .trim_start()
            .split(':')
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned();
        entries.push(Entry {
            name,
            lines: vec![line],
        });
    }
    entries
}

#[cfg(test)]
mod tests;
