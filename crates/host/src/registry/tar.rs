//! A pack's tarball, read ([resolving registry packs], decisions 6 and 11): a gzipped tar of the package under
//! `package/`. Its regular files are kept, by their path in the package: those that are UTF-8 text for the processor,
//! and the others as bytes, for the host to show. It is read only after its hash is checked, and every step is bounded
//! (audit S10, S11): the gunzip by a cap, the entries by sizes that are whole numbers, and the files by the size their
//! registry declares.
//!
//! [resolving registry packs]: https://github.com/comp-markdown/compmd/blob/main/docs/resolving-registry-packs.md

use std::collections::BTreeMap;

use miniz_oxide::inflate::decompress_to_vec_with_limit;

/// A tar block.
const BLOCK: usize = 512;

/// Where npm puts a package in its tarball.
const PACKAGE: &str = "package/";

/// The gzip header's flags (RFC 1952).
const FHCRC: u8 = 0b0010;
const FEXTRA: u8 = 0b0100;
const FNAME: u8 = 0b1000;
const FCOMMENT: u8 = 0b1_0000;

/// Where the deflate stream starts in a gzip member: after its header and the optional fields its flags name.
fn deflate_start(bytes: &[u8]) -> Option<usize> {
    let [0x1f, 0x8b, 8, flags, ..] = *bytes else {
        return None;
    };
    let mut at = 10;
    if flags & FEXTRA != 0 {
        let length = usize::from(u16::from_le_bytes([*bytes.get(at)?, *bytes.get(at + 1)?]));
        at += 2 + length;
    }
    for flag in [FNAME, FCOMMENT] {
        if flags & flag != 0 {
            at += bytes.get(at..)?.iter().position(|&byte| byte == 0)? + 1;
        }
    }
    if flags & FHCRC != 0 {
        at += 2;
    }
    (at <= bytes.len()).then_some(at)
}

/// `bytes`, gunzipped, at most `cap` bytes of them.
///
/// # Errors
///
/// When it isn't gzip, or it unzips to more than `cap` bytes.
pub(crate) fn gunzip(bytes: &[u8], cap: usize) -> Result<Vec<u8>, String> {
    let start = deflate_start(bytes).ok_or_else(|| "the tarball isn't gzip".to_owned())?;
    decompress_to_vec_with_limit(&bytes[start..], cap).map_err(|error| match error.status {
        miniz_oxide::inflate::TINFLStatus::HasMoreOutput => {
            format!("it unzips to more than {cap} bytes")
        }
        _ => "the tarball isn't gzip".to_owned(),
    })
}

/// The text of a header field: up to its first NUL.
fn field(header: &[u8], start: usize, length: usize) -> String {
    let bytes = &header[start..start + length];
    let end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

/// An entry's size, its header's octal field: a whole number of bytes, `0` when the field is blank.
fn size_of(header: &[u8]) -> Result<usize, String> {
    let written = field(header, 124, 12);
    let written = written.trim();
    if !written.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) {
        return Err(format!(
            "a tar entry's size isn't a whole number of bytes: `{written}`"
        ));
    }
    if written.is_empty() {
        return Ok(0);
    }
    usize::from_str_radix(written, 8)
        .map_err(|_| format!("a tar entry's size isn't a whole number of bytes: `{written}`"))
}

/// The `path` record of a PAX extended header (`<length> path=<value>\n`), if it has one.
fn pax_path(data: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(data);
    text.split('\n').find_map(|record| {
        let (length, rest) = record.split_once(' ')?;
        (length.bytes().all(|byte| byte.is_ascii_digit()) && !length.is_empty())
            .then(|| rest.strip_prefix("path="))
            .flatten()
            .map(str::to_owned)
    })
}

/// The path of an entry in the package, or `None` for one outside `package/` or one that would leave it.
fn package_path(name: &str) -> Option<&str> {
    let path = name.strip_prefix(PACKAGE)?;
    let unsafe_path = path.is_empty()
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part == ".." || part == "." || part.is_empty());
    (!unsafe_path).then_some(path)
}

/// The regular files of a tar archive, by their path in the package.
///
/// # Errors
///
/// For an entry whose size isn't a whole number, one that runs past the archive's end, or files that take more than
/// `max_size` bytes together.
pub(crate) fn untar(archive: &[u8], max_size: usize) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut files = BTreeMap::new();
    let mut offset = 0;
    let mut total = 0usize;
    let mut long_name: Option<String> = None;
    while offset + BLOCK <= archive.len() {
        let header = &archive[offset..offset + BLOCK];
        if header.iter().all(|&byte| byte == 0) {
            break;
        }
        let size = size_of(header)?;
        let kind = header[156];
        let end = offset
            .checked_add(BLOCK + size)
            .filter(|end| *end <= archive.len())
            .ok_or_else(|| "a tar entry runs past the tarball's end".to_owned())?;
        let data = &archive[offset + BLOCK..end];
        offset += BLOCK + size.div_ceil(BLOCK) * BLOCK;
        let regular = kind == b'0' || kind == 0;
        if regular {
            total += size;
            if total > max_size {
                return Err(format!(
                    "its files take more than the {max_size} bytes its registry declares"
                ));
            }
        }
        match kind {
            b'x' => {
                long_name = pax_path(data);
                continue;
            }
            b'L' => {
                long_name = Some(field(data, 0, data.len()));
                continue;
            }
            _ => {}
        }
        let prefix = field(header, 345, 155);
        let short = field(header, 0, 100);
        let name = long_name.take().unwrap_or_else(|| {
            if prefix.is_empty() {
                short
            } else {
                format!("{prefix}/{short}")
            }
        });
        if let (true, Some(path)) = (regular, package_path(&name)) {
            files.insert(path.to_owned(), data.to_vec());
        }
    }
    Ok(files)
}

/// A release's files, split: the UTF-8 ones as text, what the processor reads, and the others as bytes, an image or
/// a font's file the host shows.
pub(crate) fn split(
    files: BTreeMap<String, Vec<u8>>,
) -> (BTreeMap<String, String>, BTreeMap<String, Vec<u8>>) {
    let mut text = BTreeMap::new();
    let mut binary = BTreeMap::new();
    for (path, bytes) in files {
        match String::from_utf8(bytes) {
            Ok(content) => {
                text.insert(path, content);
            }
            Err(error) => {
                binary.insert(path, error.into_bytes());
            }
        }
    }
    (text, binary)
}

#[cfg(test)]
pub(crate) mod tests;
