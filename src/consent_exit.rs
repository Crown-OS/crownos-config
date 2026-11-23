//! The exit status of `crownos-portal-dialog`: the one place that says how an answer is
//! encoded, for the dialog, the portal, the compositor and crown-contextd.
//!
//! | Status | Meaning |
//! |---|---|
//! | 0, 3–15 | not used; read as declined |
//! | 1 | declined: the cancel button, Escape, or the dialog closed |
//! | 2 | misused: bad arguments, or a question too large to encode |
//! | 16–255 | answered: `16 + choice × states + state` |
//!
//! `choice` is the index of the chosen button, counted over all sections. `state` is the value
//! of each extra question in a mixed radix, first question lowest: a checkbox has 2 values (0
//! off, 1 on) and a list has one value per item (the item index). `states` is the product of
//! those counts. A question fits when `choices × states ≤ 240`. A signal or any other status is
//! read as declined.

/// The cancel button, Escape, or the dialog closed.
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

#[cfg(test)]
mod tests {
    use super::*;

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
