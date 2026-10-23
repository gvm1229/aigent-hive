use super::super::{InterfaceLanguage, SetupError};
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
        let korean = self.language == InterfaceLanguage::Ko;
        loop {
            writeln!(self.writer, "\n{}", question.prompt.get(korean))
                .map_err(|error| io_error(&error))?;
            if !question.help.get(korean).is_empty() {
                writeln!(self.writer, "{}", question.help.get(korean))
                    .map_err(|error| io_error(&error))?;
            }
            for (index, option) in question.options.iter().enumerate() {
                writeln!(self.writer, "  {}. {}", index + 1, option.label.get(korean))
                    .map_err(|error| io_error(&error))?;
            }
            if question.input == InputKind::Boolean {
                self.say("예/아니요", "yes/no")?;
            }
            if let Some(default) = current.or(question.default.as_ref()) {
                let label = if korean {
                    "Enter로 유지/선택"
                } else {
                    "Enter to keep/select"
                };
                writeln!(
                    self.writer,
                    "{label}: {}",
                    display_value(question, default, korean)
                )
                .map_err(|error| io_error(&error))?;
            }
            write!(self.writer, "> ").map_err(|error| io_error(&error))?;
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
        self.say(
            "이 변경을 적용할까요? [예/아니요, 기본: 아니요]",
            "Apply these changes? [yes/no, default: no]",
        )?;
        loop {
            write!(self.writer, "> ").map_err(|error| io_error(&error))?;
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
