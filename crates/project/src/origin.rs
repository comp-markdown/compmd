//! The origins and registries a configuration names, read as JavaScript's `URL` reads them, so the crate and
//! `@compmd/vite` agree on what is one.

use url::Url;

/// Whether `url`'s host is this machine's: `localhost`, `127.0.0.1` or `[::1]`, which may be reached over `http:`.
fn is_local(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}

/// Why `url`'s scheme is wrong, if it is: it must be `https:`, or `http:` for this machine.
fn scheme_problem(url: &Url, written: &str) -> Option<String> {
    let secure = url.scheme() == "https" || (url.scheme() == "http" && is_local(url));
    (!secure).then(|| format!("`{written}` must be `https:` (`http:` only for localhost)"))
}

/// Whether `url` has a query or a fragment with something in it, as `URL`'s `search` and `hash` say.
fn has_query_or_fragment(url: &Url) -> bool {
    url.query().is_some_and(|query| !query.is_empty())
        || url.fragment().is_some_and(|fragment| !fragment.is_empty())
}

/// `written` as `https://host[:port]`, or why it isn't an origin a pack may reach or a document's data may come from.
pub(crate) fn origin_of(written: &str) -> Result<String, String> {
    let url = Url::parse(written).map_err(|_| format!("`{written}` isn't a URL"))?;
    if let Some(problem) = scheme_problem(&url, written) {
        return Err(problem);
    }
    if written.contains('*') || url.host_str().is_some_and(|host| host.contains('*')) {
        return Err(format!("`{written}` has a wildcard: name each origin"));
    }
    if url.path() != "/" || has_query_or_fragment(&url) || !url.username().is_empty() {
        return Err(format!(
            "`{written}` must be an origin alone, without a path"
        ));
    }
    Ok(url.origin().ascii_serialization())
}

/// `written` as a registry's URL, ending in `/`, or why it can't be one.
pub(crate) fn registry_url_of(written: &str) -> Result<String, String> {
    let url = Url::parse(written).map_err(|_| {
        format!("`{written}` isn't a URL: a registry is named in full, such as `https://registry.npmjs.org/`")
    })?;
    if let Some(problem) = scheme_problem(&url, written) {
        return Err(problem);
    }
    if !url.username().is_empty() || url.password().is_some() || has_query_or_fragment(&url) {
        return Err(format!(
            "`{written}` must have no credentials, query or fragment: a registry's token belongs to the host's proxy"
        ));
    }
    let href = url.as_str();
    Ok(if href.ends_with('/') {
        href.to_owned()
    } else {
        format!("{href}/")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_origin_is_normalized_as_url_writes_it() {
        assert_eq!(
            origin_of("https://CDN.Example.com"),
            Ok("https://cdn.example.com".into())
        );
        assert_eq!(
            origin_of("https://cdn.example.com:443/"),
            Ok("https://cdn.example.com".into())
        );
        assert_eq!(
            origin_of("https://cdn.example.com:8443"),
            Ok("https://cdn.example.com:8443".into())
        );
        assert_eq!(
            origin_of("http://localhost:5173"),
            Ok("http://localhost:5173".into())
        );
        assert_eq!(
            origin_of("http://[::1]:5173"),
            Ok("http://[::1]:5173".into())
        );
        assert_eq!(
            origin_of("https://bücher.example"),
            Ok("https://xn--bcher-kva.example".into())
        );
    }

    #[test]
    fn an_origin_is_secure_named_and_alone() {
        assert_eq!(
            origin_of("cdn.example.com"),
            Err("`cdn.example.com` isn't a URL".into())
        );
        assert_eq!(
            origin_of("http://cdn.example.com"),
            Err("`http://cdn.example.com` must be `https:` (`http:` only for localhost)".into())
        );
        assert_eq!(
            origin_of("https://*.example.com"),
            Err("`https://*.example.com` has a wildcard: name each origin".into())
        );
        for written in [
            "https://cdn.example.com/x",
            "https://cdn.example.com?a=1",
            "https://cdn.example.com#a",
            "https://me@cdn.example.com",
        ] {
            assert_eq!(
                origin_of(written),
                Err(format!(
                    "`{written}` must be an origin alone, without a path"
                ))
            );
        }
    }

    #[test]
    fn a_registry_is_a_secure_url_ending_in_a_slash() {
        assert_eq!(
            registry_url_of("https://registry.npmjs.org"),
            Ok("https://registry.npmjs.org/".into())
        );
        assert_eq!(
            registry_url_of("https://npm.example.com/api/npm/"),
            Ok("https://npm.example.com/api/npm/".into())
        );
        assert_eq!(
            registry_url_of("http://localhost:4873"),
            Ok("http://localhost:4873/".into())
        );
        assert_eq!(
            registry_url_of("registry.npmjs.org"),
            Err("`registry.npmjs.org` isn't a URL: a registry is named in full, such as `https://registry.npmjs.org/`".into())
        );
        assert_eq!(
            registry_url_of("http://registry.npmjs.org"),
            Err("`http://registry.npmjs.org` must be `https:` (`http:` only for localhost)".into())
        );
        for written in [
            "https://me:token@npm.example.com",
            "https://:token@npm.example.com",
            "https://npm.example.com/?token=x",
            "https://npm.example.com/#x",
        ] {
            assert_eq!(
                registry_url_of(written),
                Err(format!(
                    "`{written}` must have no credentials, query or fragment: a registry's token belongs to the host's proxy"
                ))
            );
        }
    }
}
