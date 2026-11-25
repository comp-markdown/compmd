use std::collections::BTreeMap;

use miniz_oxide::deflate::compress_to_vec;

use super::*;

/// A tar header for `name`, of `size` bytes and `kind`, its size written as `size_field` when given.
fn header(name: &str, size: usize, kind: u8, size_field: Option<&str>) -> Vec<u8> {
    let mut header = vec![0u8; BLOCK];
    header[..name.len()].copy_from_slice(name.as_bytes());
    let size = size_field.map_or_else(|| format!("{size:011o}"), str::to_owned);
    header[124..124 + size.len()].copy_from_slice(size.as_bytes());
    header[156] = kind;
    header
}

/// A tar of `entries`: each a name, its bytes and its type.
pub(crate) fn tar(entries: &[(&str, &[u8], u8)]) -> Vec<u8> {
    let mut archive = Vec::new();
    for (name, data, kind) in entries {
        archive.extend(header(name, data.len(), *kind, None));
        archive.extend(*data);
        archive.resize(archive.len().div_ceil(BLOCK) * BLOCK, 0);
    }
    archive.extend(vec![0u8; BLOCK * 2]);
    archive
}

/// `bytes` gzipped, with a file name in the header, as npm's tarballs have none but others may.
pub(crate) fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut gzipped = vec![0x1f, 0x8b, 8, FNAME, 0, 0, 0, 0, 0, 3];
    gzipped.extend(b"package.tar\0");
    gzipped.extend(compress_to_vec(bytes, 6));
    gzipped.extend([0; 8]);
    gzipped
}

/// A pack's tarball, of `files` by their path in the package.
pub(crate) fn tarball(files: &[(&str, &[u8])]) -> Vec<u8> {
    let entries: Vec<(String, &[u8], u8)> = files
        .iter()
        .map(|(path, data)| (format!("package/{path}"), *data, b'0'))
        .collect();
    let entries: Vec<(&str, &[u8], u8)> = entries
        .iter()
        .map(|(name, data, kind)| (name.as_str(), *data, *kind))
        .collect();
    gzip(&tar(&entries))
}

#[test]
fn a_tarball_gives_its_packages_regular_files() {
    let archive = tar(&[
        ("package/package.json", b"{}", b'0'),
        ("package/docs", b"", b'5'),
        ("package/docs/a.comp.md", b"# A", 0),
        ("other/x", b"outside", b'0'),
        ("package/../x", b"escapes", b'0'),
        ("package/./x", b"dot", b'0'),
        ("package/a\\b", b"backslash", b'0'),
        ("package/", b"", b'0'),
    ]);
    let files = untar(&archive, 1000).expect("files");
    assert_eq!(
        files.keys().collect::<Vec<_>>(),
        ["docs/a.comp.md", "package.json"]
    );
    assert_eq!(files["package.json"], b"{}");
}

#[test]
fn long_names_come_from_pax_and_gnu_headers_and_the_prefix() {
    let long = format!("package/{}/file.txt", "deep".repeat(30));
    let pax = format!("{} path={long}\n", long.len() + 12);
    let mut archive = tar(&[("pax", pax.as_bytes(), b'x'), ("short", b"from pax", b'0')]);
    archive.truncate(archive.len() - BLOCK * 2);
    archive.extend(tar(&[
        ("././@LongLink", format!("{long}2").as_bytes(), b'L'),
        ("short", b"from gnu", b'0'),
    ]));
    let files = untar(&archive, 1000).expect("files");
    assert_eq!(
        files[&format!("{}/file.txt", "deep".repeat(30))],
        b"from pax"
    );
    assert_eq!(
        files[&format!("{}/file.txt2", "deep".repeat(30))],
        b"from gnu"
    );
    let mut prefixed = header("file.txt", 2, b'0', None);
    prefixed[345..352].copy_from_slice(b"package");
    prefixed.extend(b"hi");
    prefixed.resize(BLOCK * 2, 0);
    prefixed.extend(vec![0u8; BLOCK * 2]);
    assert_eq!(untar(&prefixed, 10).expect("files")["file.txt"], b"hi");
    assert_eq!(pax_path(b"12 mtime=1\n"), None);
    assert_eq!(pax_path(b"x path=a\n"), None);
}

#[test]
fn every_step_is_bounded() {
    assert_eq!(
        untar(
            &tar(&[("package/a", b"12345", b'0'), ("package/b", b"6789", b'0')]),
            8
        ),
        Err("its files take more than the 8 bytes its registry declares".into())
    );
    let mut bad_size = header("package/a", 0, b'0', Some("-1"));
    bad_size.extend(vec![0u8; BLOCK * 2]);
    assert_eq!(
        untar(&bad_size, 8),
        Err("a tar entry's size isn't a whole number of bytes: `-1`".into())
    );
    let mut past_end = header("package/a", 4000, b'0', None);
    past_end.extend(vec![0u8; BLOCK]);
    assert_eq!(
        untar(&past_end, 10_000),
        Err("a tar entry runs past the tarball's end".into())
    );
    let mut blank = header("package/a", 0, b'0', Some(" "));
    blank.extend(vec![0u8; BLOCK * 2]);
    assert_eq!(untar(&blank, 0).expect("files")["a"], b"");
    let mut huge = header("package/a", 0, b'0', Some("77777777777"));
    huge.extend(vec![0u8; BLOCK]);
    assert_eq!(
        untar(&huge, usize::MAX),
        Err("a tar entry runs past the tarball's end".into())
    );
}

#[test]
fn gunzip_reads_gzip_up_to_its_cap() {
    let gzipped = gzip(b"hello, packs");
    assert_eq!(gunzip(&gzipped, 100).expect("bytes"), b"hello, packs");
    assert_eq!(
        gunzip(&gzipped, 5),
        Err("it unzips to more than 5 bytes".into())
    );
    assert_eq!(
        gunzip(b"not gzip", 100),
        Err("the tarball isn't gzip".into())
    );
    let mut broken = gzip(b"hello");
    broken.truncate(14);
    broken.extend([0xff; 4]);
    assert_eq!(gunzip(&broken, 100), Err("the tarball isn't gzip".into()));
    let mut fields = vec![
        0x1f,
        0x8b,
        8,
        FEXTRA | FCOMMENT | FHCRC,
        0,
        0,
        0,
        0,
        0,
        3,
        2,
        0,
        b'x',
        b'y',
    ];
    fields.extend(b"a comment\0");
    fields.extend([0, 0]);
    fields.extend(compress_to_vec(b"fields", 6));
    assert_eq!(gunzip(&fields, 100).expect("bytes"), b"fields");
    assert_eq!(
        deflate_start(&[0x1f, 0x8b, 8, FEXTRA, 0, 0, 0, 0, 0, 3, 200, 0]),
        None
    );
}

#[test]
fn text_and_binary_files_are_apart() {
    let files = BTreeMap::from([
        ("a.md".to_owned(), b"# A".to_vec()),
        ("b.png".to_owned(), vec![0x89, 0xff, 0x00]),
    ]);
    let (text, binary) = split(files);
    assert_eq!(text["a.md"], "# A");
    assert_eq!(binary["b.png"], [0x89, 0xff, 0x00]);
    let built = untar(
        &gunzip(&tarball(&[("index.comp.md", b"# I")]), 10_000).expect("tar"),
        100,
    )
    .expect("files");
    assert_eq!(built["index.comp.md"], b"# I");
}
