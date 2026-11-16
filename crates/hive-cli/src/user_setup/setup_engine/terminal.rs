use super::super::{InterfaceLanguage, SetupError};
use super::layout;
use hive_core::setup::{AnswerInput, InputKind, Question};
use serde_json::Value;
use std::io::{BufRead, Write};

pub(super) struct Terminal<R, W> {
    pub reader: R,
    pub writer: W,
    pub language: InterfaceLanguage,
}

impl<R: BufRead, W: Write> Terminal<R, W> {
    pub fn say(&mut self, ko: &str, en: &str) -> Result<(), SetupError> {
        let line = if self.language == InterfaceLanguage::Ko {
            ko
        } else {
            en
        };
        writeln!(self.writer, "{line}").map_err(|error| io_error(&error))
    }

    pub fn ask(
        &mut self,
        question: &Question,
        current: Option<&Value>,
    ) -> Result<AnswerInput, SetupError> {
        self.ask_step(question, current, None)
    }

    /// Asks one question under a title line; `step` is `(number, total)` when known.
    pub fn ask_step(
        &mut self,
        question: &Question,
        current: Option<&Value>,
        step: Option<(usize, usize)>,
    ) -> Result<AnswerInput, SetupError> {
        let korean = self.language == InterfaceLanguage::Ko;
        let heading = question_heading(question, step, korean);
        loop {
            let mut lines = vec![String::new(), layout::title(&heading), String::new()];
            lines.extend(layout::block(question.prompt.get(korean), layout::INDENT));
            if !question.help.get(korean).is_empty() {
                lines.extend(layout::block(question.help.get(korean), layout::INDENT));
            }
            let shown = current.or(question.default.as_ref());
            for (index, option) in question.options.iter().enumerate() {
                let marker =
                    choice_marker(&option.value, current, question.default.as_ref(), korean);
                let text = format!("{}. {}{marker}", index + 1, option.label.get(korean));
                lines.extend(layout::wrap(&text, "    ", "       "));
            }
            if question.input == InputKind::List
                && !question.options.is_empty()
                && question.maximum.is_none_or(|maximum| maximum > 1)
            {
                let hint = if question.minimum.is_some_and(|minimum| minimum >= 1) {
                    if korean {
                        "하나 이상 선택: 번호나 이름을 쉼표나 공백으로 구분해 입력, 예: 1,2"
                    } else {
                        "Choose one or more: enter numbers or names separated by commas or spaces, e.g. 1,2"
                    }
                } else if korean {
                    "여러 개 선택 가능: 번호나 이름을 쉼표나 공백으로 구분해 입력, 예: 1,2"
                } else {
                    "You can choose several: enter numbers or names separated by commas or spaces, e.g. 1,2"
                };
                lines.extend(layout::block(hint, layout::INDENT));
            }
            if question.input == InputKind::Boolean {
                lines.extend(layout::block(
                    if korean { "예/아니요" } else { "yes/no" },
                    layout::INDENT,
                ));
            }
            if let Some(default) = shown {
                let label = if korean {
                    "Enter로 유지/선택"
                } else {
                    "Enter to keep/select"
                };
                let value = display_value(question, default, korean);
                if question.input == InputKind::Structured {
                    // JSON stays on one line so it can be copied back unchanged.
                    lines.push(format!("{}{label}: {value}", layout::INDENT));
                } else {
                    lines.extend(layout::wrap(
                        &format!("{label}: {value}"),
                        layout::INDENT,
                        "    ",
                    ));
                }
            }
            lines.push(String::new());
            for line in lines {
                writeln!(self.writer, "{line}").map_err(|error| io_error(&error))?;
            }
            write!(self.writer, "{} ", layout::input_marker()).map_err(|error| io_error(&error))?;
            self.writer.flush().map_err(|error| io_error(&error))?;
            let Some(line) = read_line(&mut self.reader)? else {
                return Ok(AnswerInput::Interrupted);
            };
            match question.parse_line(Some(&line), current) {
                Ok(answer) => return Ok(answer),
                Err(_) => self.say(
                    "입력값을 확인해 주세요. 표시된 선택지와 범위를 사용해 주세요. 중단은 :cancel입니다.",
                    "Check the answer. Use the listed choices and bounds. Enter :cancel to stop.",
                )?,
            }
        }
    }

    pub fn confirm(&mut self) -> Result<bool, SetupError> {
        writeln!(self.writer, "\n{}", layout::separator()).map_err(|error| io_error(&error))?;
        self.say(
            "이 변경을 적용할까요? [예/아니요, 기본: 아니요]",
            "Apply these changes? [yes/no, default: no]",
        )?;
        loop {
            write!(self.writer, "{} ", layout::input_marker()).map_err(|error| io_error(&error))?;
            self.writer.flush().map_err(|error| io_error(&error))?;
            let Some(line) = read_line(&mut self.reader)? else {
                return Ok(false);
            };
            match line.trim().to_lowercase().as_str() {
                "y" | "yes" | "예" | "네" => return Ok(true),
                "" | "n" | "no" | "아니요" | ":cancel" | "\u{3}" => return Ok(false),
                _ => self.say("예 또는 아니요를 입력해 주세요.", "Enter yes or no.")?,
            }
        }
    }
}

/// Title line: `[2/4] Project name`, or the short title alone when the count is unknown.
fn question_heading(question: &Question, step: Option<(usize, usize)>, korean: bool) -> String {
    let title = layout::question_title(&question.id, korean);
    match step {
        Some((number, total)) => format!("[{number}/{}] {title}", total.max(number)),
        None => format!("[{}] {title}", if korean { "질문" } else { "Question" }),
    }
}

/// Marks the choice that Enter keeps: the current value, otherwise the offered default.
fn choice_marker(
    value: &str,
    current: Option<&Value>,
    default: Option<&Value>,
    korean: bool,
) -> &'static str {
    let holds = |candidate: Option<&Value>| {
        candidate.is_some_and(|held| {
            held.as_str() == Some(value)
                || held
                    .as_array()
                    .is_some_and(|items| items.iter().any(|item| item.as_str() == Some(value)))
        })
    };
    match (holds(current), current.is_none() && holds(default), korean) {
        (true, _, true) => " (현재값)",
        (true, _, false) => " (current)",
        (false, true, true) => " (기본값)",
        (false, true, false) => " (default)",
        _ => "",
    }
}

fn read_line(reader: &mut impl BufRead) -> Result<Option<String>, SetupError> {
    let mut line = Vec::new();
    // Drain an overlong line without allowing unbounded allocation.
    let mut too_long = false;
    loop {
        let buffer = reader.fill_buf().map_err(|error| io_error(&error))?;
        if buffer.is_empty() {
            return Ok(None);
        }
        let newline = buffer.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(buffer.len(), |index| index + 1);
        if line.len() + consumed <= 65_536 && !too_long {
            line.extend_from_slice(&buffer[..consumed]);
        } else {
            too_long = true;
        }
        reader.consume(consumed);
        if newline.is_some() {
            break;
        }
    }
    if too_long {
        return Ok(Some("\0invalid-overlong-answer".to_owned()));
    }
    String::from_utf8(line)
        .map(Some)
        .map_err(|_| SetupError::Input("terminal input is not UTF-8".to_owned()))
}

pub(super) fn display_value(question: &Question, value: &Value, korean: bool) -> String {
    if question.input == InputKind::Structured {
        return value.to_string();
    }
    if value.is_null() {
        return if korean { "없음" } else { "None" }.into();
    }
    if let Some(values) = value.as_array() {
        return values
            .iter()
            .map(|value| display_value(question, value, korean))
            .collect::<Vec<_>>()
            .join(", ");
    }
    if let Some(boolean) = value.as_bool() {
        return match (boolean, korean) {
            (true, true) => "예",
            (false, true) => "아니요",
            (true, false) => "yes",
            (false, false) => "no",
        }
        .to_owned();
    }
    if let Some(option) = question
        .options
        .iter()
        .find(|option| value.as_str() == Some(&option.value))
    {
        return option.label.get(korean).to_owned();
    }
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned)
}

fn io_error(error: &std::io::Error) -> SetupError {
    SetupError::Input(format!("terminal connection ended: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    #[test]
    fn structured_defaults_remain_valid_json_when_copied_back_into_the_question() {
        let catalog = super::super::catalog().unwrap();
        let question = catalog
            .questions
            .iter()
            .find(|question| question.id == "project-roles")
            .unwrap();
        for value in [
            json!([]),
            json!([{"role_id":"reviewer","responsibilities":["review"]}]),
        ] {
            for korean in [false, true] {
                let displayed = display_value(question, &value, korean);
                assert_eq!(serde_json::from_str::<Value>(&displayed).unwrap(), value);
                assert!(
                    matches!(question.parse_line(Some(&displayed),None).unwrap(),AnswerInput::Answered {value:parsed,..} if parsed==value)
                );
            }
        }
    }

    #[test]
    fn invalid_answer_repeats_only_the_current_question_and_eof_never_answers_no() {
        let catalog = super::super::catalog().unwrap();
        let question = catalog
            .questions
            .iter()
            .find(|q| q.id == "usage-enforcement")
            .unwrap();
        let mut terminal = Terminal {
            reader: Cursor::new(b"maybe\nyes\n"),
            writer: Vec::new(),
            language: InterfaceLanguage::En,
        };
        assert!(
            matches!(terminal.ask(question, None).unwrap(), AnswerInput::Answered { value, .. } if value == json!(true))
        );
        assert_eq!(
            terminal.ask(question, None).unwrap(),
            AnswerInput::Interrupted
        );
        let output = String::from_utf8(terminal.writer).unwrap();
        assert!(output.contains("Check the answer."));
        assert!(!output.contains("maybe"));
    }

    #[test]
    fn app_choice_shows_multi_select_hint_and_accepts_space_separated_numbers() {
        let catalog = super::super::catalog().unwrap();
        let question = catalog
            .questions
            .iter()
            .find(|q| q.id == "user-hosts")
            .unwrap();
        for (language, hint) in [
            (
                InterfaceLanguage::En,
                "Choose one or more: enter numbers or names separated by commas or spaces, e.g. 1,2",
            ),
            (
                InterfaceLanguage::Ko,
                "하나 이상 선택: 번호나 이름을 쉼표나 공백으로 구분해 입력, 예: 1,2",
            ),
        ] {
            let mut terminal = Terminal {
                reader: Cursor::new(b"1 2\n"),
                writer: Vec::new(),
                language,
            };
            assert!(
                matches!(terminal.ask(question, None).unwrap(), AnswerInput::Answered { value, .. } if value == json!(["codex", "claude"]))
            );
            let output = String::from_utf8(terminal.writer).unwrap();
            // Wrapping at 80 columns may break the hint across lines; its words stay unchanged.
            assert!(output.split_whitespace().collect::<Vec<_>>().join(" ").contains(hint));
        }
    }

    #[test]
    fn confirmation_defaults_to_no_in_both_languages() {
        for language in [InterfaceLanguage::Ko, InterfaceLanguage::En] {
            for input in ["", "\n", ":cancel\n", "아니요\n"] {
                let mut terminal = Terminal {
                    reader: Cursor::new(input.as_bytes()),
                    writer: Vec::new(),
                    language,
                };
                assert!(!terminal.confirm().unwrap());
            }
        }
    }

    #[test]
    fn partial_line_at_disconnect_is_not_an_answer_and_choices_use_localized_labels() {
        let mut reader = Cursor::new(b"n");
        assert!(read_line(&mut reader).unwrap().is_none());
        let catalog = super::super::catalog().unwrap();
        let question = catalog
            .questions
            .iter()
            .find(|q| q.id == "user-contexts")
            .unwrap();
        assert_eq!(
            display_value(question, &json!(["non-developer"]), true),
            "일반 지식 작업"
        );
    }
}
