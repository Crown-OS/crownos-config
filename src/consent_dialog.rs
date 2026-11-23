//! The consent dialog that every CrownOS component asks a privacy question with. A component
//! runs the root-owned dialog program itself and reads the answer from its output. No D-Bus
//! name is in the path, so no other process of the user can answer in the user's place. The
//! compositor keeps synthetic input away from this program's windows.
//!
//! The arguments and the output are those of xdg-desktop-portal-crownos (`chooser::ask` and
//! `dialog/src/alert.rs`):
//!
//! ```text
//! crownos-portal-dialog --icon access|warning --title T --message M --cancel DENY
//!     --choice grant=GRANT [--checkbox LABEL --initial true|false]
//!     [--list LABEL --item A --item B --initial INDEX]
//! ```
//!
//! The dialog prints `option VALUE` for each extra question. Then it prints `grant` and exits
//! with 0 when the user said yes, or exits with 1 when the user said no. Any other exit means
//! that nobody was asked.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Where xdg-desktop-portal-crownos installs the dialog.
pub const DIALOG_EXECUTABLE: &str = "/usr/lib/crownos-portal-dialog";
/// A development build of the dialog for nested tests. Debug builds only.
#[cfg(debug_assertions)]
const DIALOG_OVERRIDE: &str = "CROWN_CONSENT_DIALOG";
const GRANT: &str = "grant";
const OPTION_PREFIX: &str = "option ";
const DECLINED: i32 = 1;
const TRUE: &str = "true";
const FALSE: &str = "false";

/// The dialog program to run.
pub fn dialog_executable() -> PathBuf {
    #[cfg(debug_assertions)]
    if let Some(path) = std::env::var_os(DIALOG_OVERRIDE) {
        return PathBuf::from(path);
    }
    PathBuf::from(DIALOG_EXECUTABLE)
}

/// An extra question below the main one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentOption {
    /// Answers `true` or `false`.
    Checkbox { label: String, initially_on: bool },
    /// Answers the index of the chosen item.
    List {
        label: String,
        items: Vec<String>,
        initial: usize,
    },
}

/// A yes/no question with its extra questions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentQuestion {
    pub dangerous: bool,
    pub title: String,
    pub message: String,
    pub grant: String,
    pub deny: String,
    pub options: Vec<ConsentOption>,
}

/// The dialog could not ask: it failed to start or to show, or it was killed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the consent dialog asked nobody (exit {0:?})")]
pub struct NotAsked(pub Option<i32>);

/// What the user said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentAnswer {
    pub granted: bool,
    /// One value for each extra question, or `None` when the dialog did not print one for each
    /// (closed with Escape, or failed).
    pub option_values: Option<Vec<String>>,
}

impl ConsentAnswer {
    /// The checkbox at `index` was on.
    pub fn checked(&self, index: usize) -> bool {
        self.value(index) == Some(TRUE)
    }

    /// The item chosen in the list at `index`.
    pub fn chosen(&self, index: usize) -> Option<usize> {
        self.value(index)?.parse().ok()
    }

    fn value(&self, index: usize) -> Option<&str> {
        self.option_values.as_ref()?.get(index).map(String::as_str)
    }
}

impl ConsentQuestion {
    /// The dialog's arguments, in the portal's order.
    pub fn arguments(&self) -> Vec<String> {
        let icon = if self.dangerous { "warning" } else { "access" };
        let mut arguments: Vec<String> = [
            "--icon",
            icon,
            "--title",
            &self.title,
            "--message",
            &self.message,
            "--cancel",
            &self.deny,
            "--choice",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        arguments.push(format!("{GRANT}={}", self.grant));
        for option in &self.options {
            match option {
                ConsentOption::Checkbox {
                    label,
                    initially_on,
                } => arguments.extend([
                    "--checkbox".to_owned(),
                    label.clone(),
                    "--initial".to_owned(),
                    if *initially_on { TRUE } else { FALSE }.to_owned(),
                ]),
                ConsentOption::List {
                    label,
                    items,
                    initial,
                } => {
                    arguments.extend(["--list".to_owned(), label.clone()]);
                    for item in items {
                        arguments.extend(["--item".to_owned(), item.clone()]);
                    }
                    arguments.extend(["--initial".to_owned(), initial.to_string()]);
                }
            }
        }
        arguments
    }

    /// The dialog process, ready to run: no input, the answer on stdout.
    pub fn command(&self) -> Command {
        let mut command = Command::new(dialog_executable());
        command
            .args(self.arguments())
            .stdin(Stdio::null())
            .stdout(Stdio::piped());
        command
    }

    /// The answer in the output of a dialog that exited with `exit_code`.
    ///
    /// # Errors
    ///
    /// [`NotAsked`] when the exit code is neither a yes nor a no.
    pub fn answer(&self, exit_code: Option<i32>, stdout: &[u8]) -> Result<ConsentAnswer, NotAsked> {
        let text = std::str::from_utf8(stdout).unwrap_or_default();
        let granted = match exit_code {
            Some(0) => text.lines().next_back().map(str::trim) == Some(GRANT),
            Some(DECLINED) => false,
            other => return Err(NotAsked(other)),
        };
        let values: Vec<String> = text
            .lines()
            .filter_map(|line| line.strip_prefix(OPTION_PREFIX))
            .map(|value| value.trim().to_owned())
            .collect();
        let option_values = values
            .len()
            .checked_sub(self.options.len())
            .and_then(|first| values.get(first..))
            .map(<[String]>::to_vec);
        Ok(ConsentAnswer {
            granted,
            option_values,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn question() -> ConsentQuestion {
        ConsentQuestion {
            dangerous: false,
            title: "Let notes use your screen context?".to_owned(),
            message: "exe:/usr/bin/notes asks what you work on.".to_owned(),
            grant: "Allow".to_owned(),
            deny: "Don't allow".to_owned(),
            options: vec![
                ConsentOption::Checkbox {
                    label: "Remember this choice".to_owned(),
                    initially_on: false,
                },
                ConsentOption::List {
                    label: "The app may see".to_owned(),
                    items: vec!["Window list".to_owned(), "Titles".to_owned()],
                    initial: 0,
                },
            ],
        }
    }

    #[test]
    fn the_arguments_are_the_portal_ones() {
        assert_eq!(
            question().arguments(),
            [
                "--icon",
                "access",
                "--title",
                "Let notes use your screen context?",
                "--message",
                "exe:/usr/bin/notes asks what you work on.",
                "--cancel",
                "Don't allow",
                "--choice",
                "grant=Allow",
                "--checkbox",
                "Remember this choice",
                "--initial",
                "false",
                "--list",
                "The app may see",
                "--item",
                "Window list",
                "--item",
                "Titles",
                "--initial",
                "0",
            ]
        );
    }

    #[test]
    fn a_grant_needs_exit_0_and_the_grant_line() {
        let question = question();
        let answer = question
            .answer(Some(0), b"option true\noption 1\ngrant\n")
            .unwrap();
        assert!(answer.granted && answer.checked(0));
        assert_eq!(answer.chosen(1), Some(1));
        assert!(!question.answer(Some(0), b"option true\n").unwrap().granted);
        let denied = question
            .answer(Some(1), b"option false\noption 0\n")
            .unwrap();
        assert!(!denied.granted && !denied.checked(0));
        let closed = question.answer(Some(1), b"").unwrap();
        assert_eq!(closed.option_values, None);
    }

    #[test]
    fn a_dialog_that_could_not_show_asked_nobody() {
        let question = question();
        assert_eq!(question.answer(Some(2), b""), Err(NotAsked(Some(2))));
        assert_eq!(question.answer(None, b"grant\n"), Err(NotAsked(None)));
    }
}
