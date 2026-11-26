//! What a request on the preview page's port asks for (docs/editor-support.md, decision 8): everything is under the
//! token, `/<token>/`: the page and its files, and the routes it reads beside it, `events`, `request.json`,
//! `registry/` and `files/`.

/// A request on the page's port.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PageRoute<'a> {
    /// The page's file at this path; `index.html` for the page itself.
    File(&'a str),
    /// The events.
    Events,
    /// The document's request but its text.
    Request,
    /// A path of the registries' proxy, under `registry/`.
    Registry(&'a str),
    /// A project file's path, as the URL has it, under `files/`.
    Project(&'a str),
    /// Anything else, a path without the token among it.
    Unknown,
}

/// The route of `url`, a request's path with its query, on the page whose token is `token`.
pub(crate) fn page_route<'a>(url: &'a str, token: &str) -> PageRoute<'a> {
    let path = url.split(['?', '#']).next().unwrap_or_default();
    let Some(rest) = path
        .strip_prefix('/')
        .and_then(|path| path.strip_prefix(token))
        .and_then(|path| path.strip_prefix('/'))
    else {
        return PageRoute::Unknown;
    };
    match rest {
        "" => PageRoute::File("index.html"),
        "events" => PageRoute::Events,
        "request.json" => PageRoute::Request,
        _ if rest.starts_with("files/") => PageRoute::Project(&rest["files/".len()..]),
        _ => match rest.strip_prefix("registry/") {
            Some(registry) => PageRoute::Registry(registry),
            None if rest.split('/').any(|part| part.is_empty() || part == "..") => {
                PageRoute::Unknown
            }
            None => PageRoute::File(rest),
        },
    }
}

/// The page `html` with the script that gives it the frames' `origin`, as `createSandbox` reads it, first in its head.
pub(crate) fn with_origin(html: &str, origin: &str) -> String {
    let origin = serde_json::Value::String(origin.to_owned());
    let script =
        format!("<script>globalThis.__COMPMD__ = Object.freeze({{ origin: {origin} }});</script>");
    match html.find("<head>") {
        Some(at) => {
            let after = at + "<head>".len();
            format!("{}{script}{}", &html[..after], &html[after..])
        }
        None => format!("{script}{html}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_is_under_the_token() {
        let token = "0123abcd";
        assert_eq!(
            page_route("/0123abcd/", token),
            PageRoute::File("index.html")
        );
        assert_eq!(
            page_route("/0123abcd/assets/index.js?v=1", token),
            PageRoute::File("assets/index.js")
        );
        assert_eq!(page_route("/0123abcd/events", token), PageRoute::Events);
        assert_eq!(
            page_route("/0123abcd/request.json", token),
            PageRoute::Request
        );
        assert_eq!(
            page_route("/0123abcd/registry/@kit%2fnotes", token),
            PageRoute::Registry("@kit%2fnotes")
        );
        assert_eq!(
            page_route("/0123abcd/files/posts/frost.png?v=2", token),
            PageRoute::Project("posts/frost.png")
        );
        for elsewhere in [
            "/",
            "/0123abcd",
            "/other/",
            "/0123abcdx/",
            "/0123abcd/../x",
            "/0123abcd/a//b",
        ] {
            assert_eq!(
                page_route(elsewhere, token),
                PageRoute::Unknown,
                "{elsewhere}"
            );
        }
    }

    #[test]
    fn the_page_gets_the_frames_origin_first_in_its_head() {
        let page = with_origin(
            "<!doctype html><html><head><title>P</title></head></html>",
            "http://127.0.0.1:4000",
        );
        assert_eq!(
            page,
            "<!doctype html><html><head><script>globalThis.__COMPMD__ = Object.freeze({ origin: \"http://127.0.0.1:4000\" });</script><title>P</title></head></html>"
        );
        assert!(with_origin("<p>", "x").starts_with("<script>"));
    }
}
