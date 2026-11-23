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
//! The dialog prints nothing. Its exit status is the answer:
//!
//! | Status | Meaning |
//! |---|---|
//! | 1 | declined: Escape, or the dialog closed, without the extra questions' values |
//! | 2 | misused: bad arguments, or a question too large to encode |
//! | 16–255 | answered: `16 + choice × states + state` |
//! | any other, or a signal | nobody was asked |
//!
//! `choice` is the index of the chosen button, counted over all sections; the cancel button
//! counts as one more button after the others, so a deny keeps the values of the extra
//! questions. `state` holds the value of each extra question in a mixed radix, the first one
//! lowest: a checkbox has 2 values (0 off, 1 on), a list one value per item (its index).
//! `states` is the product of those counts. A question must fit: `choices × states ≤ 240`.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Where xdg-desktop-portal-crownos installs the dialog.
pub const DIALOG_EXECUTABLE: &str = "/usr/lib/crownos-portal-dialog";
/// A development build of the dialog for nested tests. Debug builds only.
#[cfg(debug_assertions)]
const DIALOG_OVERRIDE: &str = "CROWN_CONSENT_DIALOG";
const GRANT: &str = "grant";
const TRUE: &str = "true";
const FALSE: &str = "false";

/// Escape, or the dialog closed: a decline without the values of the extra questions.
pub const DECLINED: u8 = 1;
/// Bad arguments, or a question too large to encode.
pub const MISUSED: u8 = 2;
/// The first status that holds an answer.
pub const FIRST_ANSWER: u8 = 16;
const ANSWER_CODES: usize = 256 - FIRST_ANSWER as usize;

/// One extra question under the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionShape {
    Checkbox,
    List { items: usize },
}

impl OptionShape {
    const fn values(self) -> usize {
        match self {
            Self::Checkbox => 2,
            Self::List { items } => items,
        }
    }
}

/// What a question asks: how many buttons, and which extra questions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionShape {
    pub choices: usize,
    pub options: Vec<OptionShape>,
}

/// The chosen button and the value of each extra question, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogAnswer {
    pub choice: usize,
    pub option_values: Vec<usize>,
}

impl QuestionShape {
    fn states(&self) -> Option<usize> {
        self.options.iter().try_fold(1_usize, |states, option| {
            states.checked_mul(option.values())
        })
    }

    /// Whether every answer to this question has a status.
    pub fn fits(&self) -> bool {
        self.choices > 0
            && self.options.iter().all(|option| option.values() > 0)
            && self
                .states()
                .and_then(|states| states.checked_mul(self.choices))
                .is_some_and(|codes| codes <= ANSWER_CODES)
    }

    /// The status for `answer`, or `None` when it is not an answer to this question.
    pub fn encode(&self, answer: &DialogAnswer) -> Option<u8> {
        if !self.fits()
            || answer.choice >= self.choices
            || answer.option_values.len() != self.options.len()
        {
            return None;
        }
        let mut state = 0;
        let mut radix = 1;
        for (option, value) in self.options.iter().zip(&answer.option_values) {
            if *value >= option.values() {
                return None;
            }
            state += value * radix;
            radix *= option.values();
        }
        let code = answer.choice * radix + state;
        u8::try_from(code + usize::from(FIRST_ANSWER)).ok()
    }

    /// The answer in an exit status. `None` is declined: status 1, a status out of the table,
    /// a status this question cannot have, or no status (a signal).
    pub fn decode(&self, status: Option<i32>) -> Option<DialogAnswer> {
        let code = usize::try_from(status?)
            .ok()?
            .checked_sub(usize::from(FIRST_ANSWER))?;
        let states = self.states()?;
        if !self.fits() || code >= states * self.choices {
            return None;
        }
        let mut state = code % states;
        let option_values = self
            .options
            .iter()
            .map(|option| {
                let value = state % option.values();
                state /= option.values();
                value
            })
            .collect();
        Some(DialogAnswer {
            choice: code / states,
            option_values,
        })
    }
}

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
    /// One value for each extra question, or `None` when the dialog closed without them.
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

    /// The dialog process, ready to run: no input, no output; the answer is the exit status.
    pub fn command(&self) -> Command {
        let mut command = Command::new(dialog_executable());
        command
            .args(self.arguments())
            .stdin(Stdio::null())
            .stdout(Stdio::null());
        command
    }

    /// What the exit status encodes: the grant button, the cancel button and the extra
    /// questions.
    pub fn shape(&self) -> QuestionShape {
        QuestionShape {
            choices: 2,
            options: self
                .options
                .iter()
                .map(|option| match option {
                    ConsentOption::Checkbox { .. } => OptionShape::Checkbox,
                    ConsentOption::List { items, .. } => OptionShape::List { items: items.len() },
                })
                .collect(),
        }
    }

    /// The answer in the exit status of the dialog; `None` when a signal ended it.
    ///
    /// # Errors
    ///
    /// [`NotAsked`] when the status is neither an answer nor a decline.
    pub fn answer(&self, exit_status: Option<i32>) -> Result<ConsentAnswer, NotAsked> {
        if exit_status == Some(i32::from(DECLINED)) {
            return Ok(ConsentAnswer {
                granted: false,
                option_values: None,
            });
        }
        let decoded = self
            .shape()
            .decode(exit_status)
            .ok_or(NotAsked(exit_status))?;
        let option_values = self
            .options
            .iter()
            .zip(decoded.option_values)
            .map(|(option, value)| match option {
                ConsentOption::Checkbox { .. } => (value == 1).to_string(),
                ConsentOption::List { .. } => value.to_string(),
            })
            .collect();
        Ok(ConsentAnswer {
            granted: decoded.choice == 0,
            option_values: Some(option_values),
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

    fn status(question: &ConsentQuestion, choice: usize, option_values: Vec<usize>) -> Option<i32> {
        question
            .shape()
            .encode(&DialogAnswer {
                choice,
                option_values,
            })
            .map(i32::from)
    }

    #[test]
    fn a_grant_and_a_deny_keep_the_values_of_the_extra_questions() {
        let question = question();
        let granted = question.answer(status(&question, 0, vec![1, 1])).unwrap();
        assert!(granted.granted && granted.checked(0));
        assert_eq!(granted.chosen(1), Some(1));
        let denied = question.answer(status(&question, 1, vec![1, 0])).unwrap();
        assert!(!denied.granted && denied.checked(0));
        assert_eq!(denied.chosen(1), Some(0));
        let closed = question.answer(Some(1)).unwrap();
        assert!(!closed.granted);
        assert_eq!(closed.option_values, None);
    }

    #[test]
    fn a_dialog_that_could_not_show_asked_nobody() {
        let question = question();
        for exit_status in [Some(0), Some(2), Some(15), Some(255), None] {
            assert_eq!(question.answer(exit_status), Err(NotAsked(exit_status)));
        }
    }

    #[test]
    fn every_answer_round_trips_through_its_status() {
        let shape = QuestionShape {
            choices: 3,
            options: vec![OptionShape::Checkbox, OptionShape::List { items: 4 }],
        };
        for choice in 0..3 {
            for checked in 0..2 {
                for item in 0..4 {
                    let answer = DialogAnswer {
                        choice,
                        option_values: vec![checked, item],
                    };
                    let status = shape.encode(&answer).map(i32::from);
                    assert!(status.is_some_and(|status| status >= 16));
                    assert_eq!(shape.decode(status), Some(answer));
                }
            }
        }
    }

    #[test]
    fn a_plain_question_uses_one_status_per_button() {
        let picker = QuestionShape {
            choices: 240,
            options: Vec::new(),
        };
        assert!(picker.fits());
        let last = DialogAnswer {
            choice: 239,
            option_values: Vec::new(),
        };
        assert_eq!(picker.encode(&last), Some(255));
        let too_many = QuestionShape {
            choices: 241,
            options: Vec::new(),
        };
        assert!(!too_many.fits());
    }

    #[test]
    fn declines_and_foreign_statuses_are_no_answer() {
        let shape = QuestionShape {
            choices: 1,
            options: vec![OptionShape::Checkbox],
        };
        for status in [
            None,
            Some(0),
            Some(1),
            Some(2),
            Some(15),
            Some(18),
            Some(300),
            Some(-1),
        ] {
            assert_eq!(shape.decode(status), None, "{status:?}");
        }
        assert_eq!(
            shape.encode(&DialogAnswer {
                choice: 1,
                option_values: vec![0]
            }),
            None
        );
    }
}
