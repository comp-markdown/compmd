//! Opening a page in the browser ([the language server], decisions 12 and 13): the system's, with its own command and
//! no dependency (`open` on macOS, `xdg-open` elsewhere on Unix, `start` on Windows), or the one the setting
//! `compmd.preview.browser` names.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::io;
use std::process::Command;

/// The program and arguments that open `url` in `browser` (a program's name or path), or, when it is `None`, in the
/// browser of the system `os` (as `std::env::consts::OS` names it).
fn browser_command_on(url: &str, browser: Option<&str>, os: &str) -> Command {
    let (program, before): (&str, &[&str]) = match (browser, os) {
        (Some(browser), _) => (browser, &[]),
        (None, "macos") => ("open", &[]),
        // `start`'s first quoted argument is the window's title, so an empty one comes before the address.
        (None, "windows") => ("cmd", &["/c", "start", ""]),
        (None, _) => ("xdg-open", &[]),
    };
    let mut command = Command::new(program);
    command.args(before).arg(url);
    command
}

/// Opens `url` in `browser`, or the system's browser; the browser goes on on its own.
///
/// # Errors
///
/// When the command can't be started.
pub(crate) fn open_in_browser(url: &str, browser: Option<&str>) -> io::Result<()> {
    browser_command_on(url, browser, std::env::consts::OS)
        .spawn()
        .map(drop)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    /// The program and the arguments of `command`.
    fn parts(command: &Command) -> Vec<&OsStr> {
        [command.get_program()]
            .into_iter()
            .chain(command.get_args())
            .collect()
    }

    #[test]
    fn the_setting_names_the_browser_or_the_systems_opens_the_page() {
        let url = "http://127.0.0.1:4000/token/";
        assert_eq!(
            parts(&browser_command_on(url, Some("firefox"), "linux")),
            ["firefox", url]
        );
        assert_eq!(
            parts(&browser_command_on(url, None, "macos")),
            ["open", url]
        );
        assert_eq!(
            parts(&browser_command_on(url, None, "windows")),
            ["cmd", "/c", "start", "", url]
        );
        assert_eq!(
            parts(&browser_command_on(url, None, "linux")),
            ["xdg-open", url]
        );
        assert_eq!(
            parts(&browser_command_on(url, None, "freebsd")),
            ["xdg-open", url]
        );
        assert!(open_in_browser(url, Some("/no/such/browser")).is_err());
    }
}
