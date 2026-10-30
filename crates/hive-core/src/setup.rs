//! Shared setup questions and input validation. No terminal, model, or filesystem access.

use crate::preferences::{DecisionOrigin, PreferenceScope};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

/// Both supported interface languages, stored with each question.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedText {
    /// English text.
    pub en: String,
    /// Korean text.
    pub ko: String,
}

impl LocalizedText {
    /// Return text in the selected language.
    #[must_use]
    pub fn get(&self, korean: bool) -> &str {
        if korean {
            &self.ko
        } else {
            &self.en
        }
    }
}

/// One explicitly selectable value.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    /// Stored value, independent of its translated label.
    pub value: String,
    /// User-facing label.
    pub label: LocalizedText,
}

/// How to parse and validate an answer.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputKind {
    /// Yes or no.
    Boolean,
    /// One listed choice.
    Choice,
    /// Several listed choices or free strings when no choices are listed.
    List,
    /// User-supplied text.
    Text,
    /// An integer in the inclusive bounds.
    Integer,
    /// Structured JSON, such as an existing role definition.
    Structured,
}

/// A condition on the current answer document.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    /// JSON Pointer to the prerequisite answer.
    pub key: String,
    /// Value that enables the question.
    pub equals: Value,
}

/// Canonical description used by terminals, answer files, and structured assistant input.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    /// Stable identifier, unchanged by translation.
    pub id: String,
    /// Meaning revision; increased when an old answer can no longer be reused.
    pub revision: u32,
    /// Bounded group returned by `--describe --section`.
    pub section: String,
    /// Owning setup scope.
    pub scope: PreferenceScope,
    /// Pointer to the stored preference.
    pub key: String,
    /// Input representation.
    pub input: InputKind,
    /// The actual question.
    pub prompt: LocalizedText,
    /// Consequences or required consent.
    pub help: LocalizedText,
    /// Choices, in display order.
    #[serde(default)]
    pub options: Vec<Choice>,
    /// Offered value, never an answer until accepted.
    #[serde(default)]
    pub default: Option<Value>,
    /// Inclusive minimum integer value or list/text length.
    #[serde(default)]
    pub minimum: Option<i64>,
    /// Inclusive maximum integer value or list/text length.
    #[serde(default)]
    pub maximum: Option<i64>,
    /// All prerequisites must match before asking.
    #[serde(default)]
    pub when: Vec<Condition>,
}

/// The versioned source of setup questions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QuestionCatalog {
    /// Catalog format version.
    pub schema_version: u32,
    /// Defaults policy revision.
    pub policy_version: String,
    /// Ordered question definitions.
    pub questions: Vec<Question>,
}

impl QuestionCatalog {
    /// Reject ambiguous identifiers, invalid pointers, or unusable defaults.
    ///
    /// # Errors
    /// Returns a contract error without echoing any supplied answer.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.policy_version.is_empty() {
            return Err("unsupported question catalog".to_owned());
        }
        let mut identities = BTreeSet::new();
        let mut keys = BTreeSet::new();
        for question in &self.questions {
            if question.revision == 0
                || !identifier(&question.id)
                || !identifier(&question.section)
                || !pointer(&question.key)
                || !identities.insert(question.id.as_str())
                || !keys.insert((format!("{:?}", question.scope), &question.key))
                || question.prompt.en.is_empty()
                || question.prompt.ko.is_empty()
                || question
                    .minimum
                    .zip(question.maximum)
                    .is_some_and(|(min, max)| min > max)
                || question
                    .when
                    .iter()
                    .any(|condition| !pointer(&condition.key))
            {
                return Err("invalid or duplicate setup question".to_owned());
            }
            let options: BTreeSet<_> = question
                .options
                .iter()
                .map(|option| &option.value)
                .collect();
            if options.len() != question.options.len()
                || question.options.iter().any(|option| {
                    option.value.is_empty()
                        || option.label.en.is_empty()
                        || option.label.ko.is_empty()
                })
                || (question.input == InputKind::Choice && question.options.is_empty())
            {
                return Err("invalid setup question choices".to_owned());
            }
            if let Some(default) = &question.default {
                question.validate_value(default)?;
            }
        }
        Ok(())
    }

    /// Inspect a bounded question group without reading unrelated settings.
    #[must_use]
    pub fn section(&self, scope: PreferenceScope, section: &str) -> Vec<&Question> {
        self.questions
            .iter()
            .filter(|q| q.scope == scope && q.section == section)
            .collect()
    }
}

impl Question {
    /// Whether the current answers satisfy this question's prerequisites.
    #[must_use]
    pub fn applicable(&self, answers: &Value) -> bool {
        self.when
            .iter()
            .all(|condition| answers.pointer(&condition.key) == Some(&condition.equals))
    }

    /// Validate a typed answer using the same rules as terminal input.
    ///
    /// # Errors
    /// Rejects type, choice, bounds, duplicate, or control-character violations.
    pub fn validate_value(&self, value: &Value) -> Result<(), String> {
        let valid = match self.input {
            InputKind::Boolean => value.is_boolean(),
            InputKind::Integer => value.as_i64().is_some_and(|n| self.in_bounds(n)),
            InputKind::Text => value.as_str().is_some_and(|s| {
                !s.chars().any(char::is_control)
                    && self.in_bounds(i64::try_from(s.chars().count()).unwrap_or(i64::MAX))
            }),
            InputKind::Choice => value
                .as_str()
                .is_some_and(|s| self.options.iter().any(|o| o.value == s)),
            InputKind::List => value.as_array().is_some_and(|items| {
                let unique: BTreeSet<_> = items.iter().filter_map(Value::as_str).collect();
                unique.len() == items.len()
                    && self.in_bounds(i64::try_from(items.len()).unwrap_or(i64::MAX))
                    && unique.iter().all(|item| {
                        !item.is_empty()
                            && !item.chars().any(char::is_control)
                            && (self.options.is_empty()
                                || self.options.iter().any(|o| o.value == *item))
                    })
            }),
            InputKind::Structured => value.is_array() || value.is_object(),
        };
        if valid {
            Ok(())
        } else {
            Err("answer does not match the question".to_owned())
        }
    }

    /// Parse one line. End-of-input is supplied separately and never becomes a negative answer.
    ///
    /// # Errors
    /// Rejects invalid input without retaining it or disclosing it in the error.
    pub fn parse_line(
        &self,
        line: Option<&str>,
        offered: Option<&Value>,
    ) -> Result<AnswerInput, String> {
        let Some(line) = line else {
            return Ok(AnswerInput::Interrupted);
        };
        let line = line.trim();
        if line == ":cancel" || line.contains('\u{3}') {
            return Ok(AnswerInput::Interrupted);
        }
        if line.len() > 65_536 {
            return Err("answer exceeds the size limit".to_owned());
        }
        let (value, origin) = if line.is_empty() {
            let default = offered
                .or(self.default.as_ref())
                .ok_or("an answer is required")?;
            (default.clone(), DecisionOrigin::AcceptedDefault)
        } else {
            (self.parse_value(line)?, DecisionOrigin::Explicit)
        };
        self.validate_value(&value)?;
        Ok(AnswerInput::Answered { value, origin })
    }

    fn parse_value(&self, line: &str) -> Result<Value, String> {
        match self.input {
            InputKind::Boolean => match line.to_lowercase().as_str() {
                "y" | "yes" | "true" | "예" | "네" => Ok(Value::Bool(true)),
                "n" | "no" | "false" | "아니요" => Ok(Value::Bool(false)),
                _ => Err("choose yes or no".to_owned()),
            },
            InputKind::Choice => Ok(Value::String(self.choice_value(line).to_owned())),
            InputKind::List => {
                if line.starts_with('[') {
                    serde_json::from_str(line).map_err(|_| "invalid list input".to_owned())
                } else {
                    Ok(Value::Array(
                        line.split(',')
                            .map(|item| Value::String(self.choice_value(item.trim()).to_owned()))
                            .collect(),
                    ))
                }
            }
            InputKind::Text => Ok(Value::String(line.to_owned())),
            InputKind::Integer => line
                .parse::<i64>()
                .map(Value::from)
                .map_err(|_| "enter a whole number".to_owned()),
            InputKind::Structured => {
                serde_json::from_str(line).map_err(|_| "invalid structured answer".to_owned())
            }
        }
    }

    fn choice_value<'a>(&'a self, input: &'a str) -> &'a str {
        input
            .parse::<usize>()
            .ok()
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| self.options.get(index))
            .map_or(input, |choice| choice.value.as_str())
    }

    fn in_bounds(&self, value: i64) -> bool {
        self.minimum.is_none_or(|minimum| value >= minimum)
            && self.maximum.is_none_or(|maximum| value <= maximum)
    }
}

/// Input outcome; interruption cannot be confused with `false`.
#[derive(Clone, Debug, PartialEq)]
pub enum AnswerInput {
    /// A validated value with its selection meaning.
    Answered {
        /// Canonical value.
        value: Value,
        /// Direct input versus accepting the displayed default.
        origin: DecisionOrigin,
    },
    /// EOF, cancellation, or a disconnected terminal.
    Interrupted,
}

/// Explicit interaction switches. Software-update approval is deliberately not an input here.
#[derive(Clone, Copy, Debug, Default)]
pub struct InteractionRequest {
    /// Require a terminal.
    pub interactive: bool,
    /// Forbid input waits.
    pub non_interactive: bool,
    /// Machine output forbids input waits.
    pub json: bool,
}

/// Decided input mode before any setup writes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractionMode {
    /// Both input and user-guidance output are terminals.
    Terminal,
    /// Return pending questions instead of waiting.
    NonInteractive,
    /// An explicitly requested terminal is unavailable.
    NeedsInput,
}

impl InteractionRequest {
    /// Resolve explicit options and terminal availability before applying anything.
    ///
    /// # Errors
    /// Rejects contradictory interactive and machine options.
    pub fn resolve(
        self,
        stdin_terminal: bool,
        guidance_terminal: bool,
    ) -> Result<InteractionMode, String> {
        if self.interactive && (self.non_interactive || self.json) {
            return Err(
                "interactive mode conflicts with non-interactive or JSON output".to_owned(),
            );
        }
        if self.non_interactive || self.json {
            return Ok(InteractionMode::NonInteractive);
        }
        if stdin_terminal && guidance_terminal {
            return Ok(InteractionMode::Terminal);
        }
        Ok(if self.interactive {
            InteractionMode::NeedsInput
        } else {
            InteractionMode::NonInteractive
        })
    }
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn pointer(value: &str) -> bool {
    if !value.starts_with('/') || value.len() < 2 || value.chars().any(char::is_control) {
        return false;
    }
    let mut characters = value.chars();
    while let Some(character) = characters.next() {
        if character == '~' && !matches!(characters.next(), Some('0' | '1')) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn question(kind: InputKind) -> Question {
        Question {
            id: "test-question".to_owned(),
            revision: 1,
            section: "test".to_owned(),
            scope: PreferenceScope::User,
            key: "/enabled".to_owned(),
            input: kind,
            prompt: LocalizedText {
                en: "Enable?".to_owned(),
                ko: "켤까요?".to_owned(),
            },
            help: LocalizedText {
                en: String::new(),
                ko: String::new(),
            },
            options: Vec::new(),
            default: None,
            minimum: None,
            maximum: None,
            when: Vec::new(),
        }
    }

    #[test]
    fn cancellation_and_eof_are_never_negative_answers() {
        let mut q = question(InputKind::Boolean);
        q.default = Some(json!(false));
        for line in [None, Some(":cancel"), Some("\u{3}")] {
            assert_eq!(q.parse_line(line, None).unwrap(), AnswerInput::Interrupted);
        }
        assert_eq!(
            q.parse_line(Some(""), None).unwrap(),
            AnswerInput::Answered {
                value: json!(false),
                origin: DecisionOrigin::AcceptedDefault,
            }
        );
        assert_eq!(
            q.parse_line(Some("아니요"), None).unwrap(),
            AnswerInput::Answered {
                value: json!(false),
                origin: DecisionOrigin::Explicit,
            }
        );
        assert!(q.parse_line(Some("possibly"), None).is_err());
    }

    #[test]
    fn terminal_and_structured_inputs_share_integer_bounds() {
        let mut q = question(InputKind::Integer);
        q.minimum = Some(0);
        q.maximum = Some(99);
        for n in [-1, 0, 20, 99, 100] {
            assert_eq!(
                q.parse_line(Some(&n.to_string()), None).is_ok(),
                q.validate_value(&json!(n)).is_ok()
            );
        }
        for input in ["20.5", "0x20", "", "unknown"] {
            assert!(q.parse_line(Some(input), None).is_err());
        }
    }

    #[test]
    fn numbered_choices_and_lists_reject_duplicates_and_unknown_values() {
        let mut q = question(InputKind::List);
        q.options = vec![
            Choice {
                value: "codex".to_owned(),
                label: LocalizedText {
                    en: "Codex".to_owned(),
                    ko: "Codex".to_owned(),
                },
            },
            Choice {
                value: "claude".to_owned(),
                label: LocalizedText {
                    en: "Claude Code".to_owned(),
                    ko: "Claude Code".to_owned(),
                },
            },
        ];
        q.minimum = Some(1);
        assert_eq!(
            q.parse_line(Some("1,2"), None).unwrap(),
            AnswerInput::Answered {
                value: json!(["codex", "claude"]),
                origin: DecisionOrigin::Explicit,
            }
        );
        for value in ["1,1", "1,codex", "0", "3", "[]", "[true]"] {
            assert!(q.parse_line(Some(value), None).is_err());
        }
    }

    #[test]
    fn question_conditions_do_not_infer_an_unanswered_opt_in() {
        let mut q = question(InputKind::Boolean);
        q.when.push(Condition {
            key: "/usage_guard/enforcement_enabled".to_owned(),
            equals: json!(true),
        });
        assert!(!q.applicable(&json!({})));
        assert!(!q.applicable(&json!({"usage_guard":{"enforcement_enabled":false}})));
        assert!(q.applicable(&json!({"usage_guard":{"enforcement_enabled":true}})));
    }

    #[test]
    fn all_mode_combinations_obey_machine_output_and_terminal_contracts() {
        for bits in 0..32 {
            let options = InteractionRequest {
                interactive: bits & 1 != 0,
                non_interactive: bits & 2 != 0,
                json: bits & 4 != 0,
            };
            let stdin = bits & 8 != 0;
            let guidance = bits & 16 != 0;
            let result = options.resolve(stdin, guidance);
            if options.interactive && (options.non_interactive || options.json) {
                assert!(result.is_err());
            } else if options.non_interactive || options.json {
                assert_eq!(result.unwrap(), InteractionMode::NonInteractive);
            } else if stdin && guidance {
                assert_eq!(result.unwrap(), InteractionMode::Terminal);
            } else if options.interactive {
                assert_eq!(result.unwrap(), InteractionMode::NeedsInput);
            } else {
                assert_eq!(result.unwrap(), InteractionMode::NonInteractive);
            }
        }
    }

    #[test]
    fn malformed_catalogs_cannot_offer_invalid_or_duplicate_questions() {
        let mut q = question(InputKind::Boolean);
        q.default = Some(json!(false));
        let mut catalog = QuestionCatalog {
            schema_version: 1,
            policy_version: "0.14.0".to_owned(),
            questions: vec![q.clone()],
        };
        catalog.validate().unwrap();
        catalog.questions.push(q);
        assert!(catalog.validate().is_err());
        catalog.questions.pop();
        catalog.questions[0].default = Some(json!("false"));
        assert!(catalog.validate().is_err());
        catalog.questions[0].default = Some(json!(false));
        catalog.questions[0].key = "/bad~escape".to_owned();
        assert!(catalog.validate().is_err());
    }
}
