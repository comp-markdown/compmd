//! The media type of a pack's file a frame may load, by its extension: an image (an SVG as `image/svg+xml`) or a font's
//! file (spec §9.4). Only these are given to a frame: a link opened from a `blob:` URL would run a pack's page or
//! script with the host's page's origin, so a pack's other files never are.

/// The types, by extension in lower case.
const FILE_TYPES: [(&str, &str); 12] = [
    ("svg", "image/svg+xml"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("avif", "image/avif"),
    ("ico", "image/x-icon"),
    ("woff2", "font/woff2"),
    ("woff", "font/woff"),
    ("ttf", "font/ttf"),
    ("otf", "font/otf"),
];

/// The media type of the file at `path` when a frame may load it, by its extension; `None` for any other file.
pub(crate) fn file_type_of(path: &str) -> Option<&'static str> {
    let (_, extension) = path.rsplit('/').next()?.rsplit_once('.')?;
    let extension = extension.to_ascii_lowercase();
    FILE_TYPES
        .iter()
        .find(|(known, _)| *known == extension)
        .map(|(_, kind)| *kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_and_fonts_have_their_type_and_nothing_else_has_one() {
        assert_eq!(file_type_of("icons/star.svg"), Some("image/svg+xml"));
        assert_eq!(file_type_of("fonts/harbour.WOFF2"), Some("font/woff2"));
        assert_eq!(file_type_of("a.b/photo.jpeg"), Some("image/jpeg"));
        assert_eq!(file_type_of(".png"), Some("image/png"));
        assert_eq!(file_type_of("guide.comp.md"), None);
        assert_eq!(file_type_of("dist/mermaid.js"), None);
        assert_eq!(file_type_of("png"), None);
        assert_eq!(file_type_of("icons.svg/readme"), None);
    }
}
