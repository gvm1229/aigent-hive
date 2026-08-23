//! Deterministic Korean post-editing inspection and preservation gates.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

const RULES_BYTES: &[u8] =
    include_bytes!("../../../harness/language-packs/im-not-ai/2.3.2/rules.json");
const MANIFEST_BYTES: &[u8] =
    include_bytes!("../../../harness/language-packs/im-not-ai/2.3.2/manifest.json");
const LICENSE_BYTES: &[u8] =
    include_bytes!("../../../harness/language-packs/im-not-ai/2.3.2/UPSTREAM-LICENSE.txt");

/// Supported output profiles.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum KoreanProfile {
    Response,
    ReleaseNote,
    Documentation,
    Technical,
    Verbatim,
}

impl KoreanProfile {
    /// Parse one stable public profile name.
    ///
    /// # Errors
    ///
    /// Returns an error when the name is not one of the five embedded profiles.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "response" => Ok(Self::Response),
            "release-note" => Ok(Self::ReleaseNote),
            "documentation" => Ok(Self::Documentation),
            "technical" => Ok(Self::Technical),
            "verbatim" => Ok(Self::Verbatim),
            _ => Err(format!("unsupported Korean profile: {value}")),
        }
    }

    /// Return the stable profile identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Response => "response",
            Self::ReleaseNote => "release-note",
            Self::Documentation => "documentation",
            Self::Technical => "technical",
            Self::Verbatim => "verbatim",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct RulesPack {
    pack_id: String,
    pack_version: String,
    transform_version: u32,
    profiles: BTreeMap<String, ProfileContract>,
    rules: Vec<Rule>,
    protected_spans: Vec<String>,
    prohibited_intents: Vec<String>,
    allowed_hygiene: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct ProfileContract {
    max_change_rate: f64,
    max_touch_rate: f64,
    rewrite: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct Rule {
    id: String,
    category: String,
    severity: String,
    kind: String,
    terms: Vec<String>,
    threshold: usize,
}

/// One deterministic style finding.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct KoreanFinding {
    pub rule_id: String,
    pub category: String,
    pub severity: String,
    pub count: usize,
    pub threshold: usize,
}

/// One byte-sensitive span that must survive rewriting.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProtectedSpan {
    pub kind: String,
    pub value: String,
}

/// Deterministic inspection output.
#[derive(Clone, Debug, Serialize)]
pub struct KoreanInspection {
    pub schema_version: u32,
    pub pack_id: String,
    pub pack_version: String,
    pub transform_version: u32,
    pub profile: KoreanProfile,
    pub korean_character_count: usize,
    pub sentence_count: usize,
    pub paragraph_count: usize,
    pub findings: Vec<KoreanFinding>,
    pub protected_spans: Vec<ProtectedSpan>,
    pub protected_span_contract: Vec<String>,
    pub route_hint: String,
    pub rewrite_allowed: bool,
    pub prohibited_intents: Vec<String>,
    pub allowed_hygiene: Vec<String>,
}

/// Deterministic before-and-after verification output.
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct KoreanVerification {
    pub schema_version: u32,
    pub profile: KoreanProfile,
    pub accepted: bool,
    pub fallback_required: bool,
    pub change_rate: f64,
    pub touch_rate: f64,
    pub max_change_rate: f64,
    pub max_touch_rate: f64,
    pub protected_spans_preserved: bool,
    pub modality_preserved: bool,
    pub original_digest: String,
    pub candidate_digest: String,
    pub failures: Vec<String>,
}

/// Return the embedded transformed rules bytes.
#[must_use]
pub const fn embedded_rules_bytes() -> &'static [u8] {
    RULES_BYTES
}

/// Return the embedded upstream provenance manifest bytes.
#[must_use]
pub const fn embedded_manifest_bytes() -> &'static [u8] {
    MANIFEST_BYTES
}

/// Return the shipped upstream license bytes.
#[must_use]
pub const fn embedded_license_bytes() -> &'static [u8] {
    LICENSE_BYTES
}

fn rules() -> Result<&'static RulesPack, String> {
    static RULES: OnceLock<Result<RulesPack, String>> = OnceLock::new();
    RULES
        .get_or_init(|| serde_json::from_slice(RULES_BYTES).map_err(|error| error.to_string()))
        .as_ref()
        .map_err(Clone::clone)
}

/// Remove invisible text-control characters without rewriting words.
#[must_use]
pub fn sanitize_text(text: &str) -> String {
    let filtered = text
        .chars()
        .filter(|character| {
            !matches!(
                *character,
                '\u{200B}'
                    | '\u{200C}'
                    | '\u{200D}'
                    | '\u{2060}'
                    | '\u{FEFF}'
                    | '\u{202A}'..='\u{202E}'
                    | '\u{2066}'..='\u{2069}'
            )
        })
        .collect::<Vec<_>>();
    compose_modern_hangul(&filtered)
}

fn compose_modern_hangul(input: &[char]) -> String {
    const S_BASE: u32 = 0xAC00;
    const L_BASE: u32 = 0x1100;
    const V_BASE: u32 = 0x1161;
    const T_BASE: u32 = 0x11A7;
    const L_COUNT: u32 = 19;
    const V_COUNT: u32 = 21;
    const T_COUNT: u32 = 28;
    let mut output = String::new();
    let mut index = 0;
    while index < input.len() {
        let lead = u32::from(input[index]);
        if (L_BASE..L_BASE + L_COUNT).contains(&lead) && index + 1 < input.len() {
            let vowel = u32::from(input[index + 1]);
            if (V_BASE..V_BASE + V_COUNT).contains(&vowel) {
                let mut tail = 0;
                if index + 2 < input.len() {
                    let candidate = u32::from(input[index + 2]);
                    if (T_BASE + 1..T_BASE + T_COUNT).contains(&candidate) {
                        tail = candidate - T_BASE;
                        index += 1;
                    }
                }
                let syllable = S_BASE
                    + (lead - L_BASE) * V_COUNT * T_COUNT
                    + (vowel - V_BASE) * T_COUNT
                    + tail;
                output.push(char::from_u32(syllable).expect("modern Hangul syllable"));
                index += 2;
                continue;
            }
        }
        output.push(input[index]);
        index += 1;
    }
    output
}

/// Inspect a finished Korean text with the embedded pack.
///
/// # Errors
///
/// Returns an error when the embedded rules pack or selected profile is invalid.
pub fn inspect(profile: KoreanProfile, text: &str) -> Result<KoreanInspection, String> {
    let pack = rules()?;
    let contract = profile_contract(pack, profile)?;
    let paragraphs = paragraphs(text);
    let sentences = sentences(text);
    let mut findings = Vec::new();
    for rule in &pack.rules {
        let count = count_rule(rule, text, &paragraphs, &sentences);
        if count >= rule.threshold {
            findings.push(KoreanFinding {
                rule_id: rule.id.clone(),
                category: rule.category.clone(),
                severity: rule.severity.clone(),
                count,
                threshold: rule.threshold,
            });
        }
    }
    findings.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
    let severe = findings.iter().filter(|item| item.severity == "S1").count();
    let route_hint = if !contract.rewrite || findings.is_empty() {
        "light"
    } else if severe > 0 || findings.len() >= 6 {
        "heavy"
    } else {
        "standard"
    };
    Ok(KoreanInspection {
        schema_version: 1,
        pack_id: pack.pack_id.clone(),
        pack_version: pack.pack_version.clone(),
        transform_version: pack.transform_version,
        profile,
        korean_character_count: text
            .chars()
            .filter(|character| ('가'..='힣').contains(character))
            .count(),
        sentence_count: sentences.len(),
        paragraph_count: paragraphs.len(),
        findings,
        protected_spans: protected_spans(text),
        protected_span_contract: pack.protected_spans.clone(),
        route_hint: route_hint.to_owned(),
        rewrite_allowed: contract.rewrite,
        prohibited_intents: pack.prohibited_intents.clone(),
        allowed_hygiene: pack.allowed_hygiene.clone(),
    })
}

/// Verify a host-rewritten candidate and require fallback on any invariant failure.
///
/// # Errors
///
/// Returns an error when the embedded rules pack or selected profile is invalid.
pub fn verify(
    profile: KoreanProfile,
    original: &str,
    candidate: &str,
) -> Result<KoreanVerification, String> {
    let pack = rules()?;
    let contract = profile_contract(pack, profile)?;
    let change_rate = bigram_change_rate(original, candidate);
    let touch_rate = line_touch_rate(original, candidate);
    let protected_spans_preserved = protected_spans(original) == protected_spans(candidate);
    let modality_preserved = modality(original) == modality(candidate);
    let mut failures = Vec::new();
    if !contract.rewrite && original != candidate {
        failures.push("verbatim-profile-changed".to_owned());
    }
    if change_rate > contract.max_change_rate {
        failures.push("change-rate-exceeded".to_owned());
    }
    if original.lines().count() >= 4 && touch_rate > contract.max_touch_rate {
        failures.push("touch-rate-exceeded".to_owned());
    }
    if !protected_spans_preserved {
        failures.push("protected-span-changed".to_owned());
    }
    if !modality_preserved {
        failures.push("modality-changed".to_owned());
    }
    let accepted = failures.is_empty();
    Ok(KoreanVerification {
        schema_version: 1,
        profile,
        accepted,
        fallback_required: !accepted,
        change_rate,
        touch_rate,
        max_change_rate: contract.max_change_rate,
        max_touch_rate: contract.max_touch_rate,
        protected_spans_preserved,
        modality_preserved,
        original_digest: crate::sha256_digest(original.as_bytes()),
        candidate_digest: crate::sha256_digest(candidate.as_bytes()),
        failures,
    })
}

fn profile_contract(pack: &RulesPack, profile: KoreanProfile) -> Result<ProfileContract, String> {
    pack.profiles.get(profile.as_str()).copied().ok_or_else(|| {
        format!(
            "profile is absent from the Korean pack: {}",
            profile.as_str()
        )
    })
}

fn paragraphs(text: &str) -> Vec<&str> {
    text.split("\n\n")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect()
}

fn sentences(text: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        current.push(character);
        if matches!(character, '.' | '!' | '?' | '。') {
            let value = current.trim();
            if !value.is_empty() {
                values.push(value.to_owned());
            }
            current.clear();
        }
    }
    if !current.trim().is_empty() {
        values.push(current.trim().to_owned());
    }
    values
}

fn count_rule(rule: &Rule, text: &str, paragraphs: &[&str], sentences: &[String]) -> usize {
    match rule.kind.as_str() {
        "absolute" | "document-frequency" => rule
            .terms
            .iter()
            .map(|term| text.matches(term).count())
            .sum(),
        "paragraph-frequency" => paragraphs
            .iter()
            .filter(|paragraph| {
                rule.terms
                    .iter()
                    .map(|term| paragraph.matches(term).count())
                    .sum::<usize>()
                    >= rule.threshold
            })
            .count()
            .saturating_mul(rule.threshold),
        "ending-monotony" => ending_monotony(sentences, rule.threshold),
        "suffix-density" => text
            .split_whitespace()
            .filter(|token| rule.terms.iter().any(|suffix| token.ends_with(suffix)))
            .count(),
        "ending-scope" => {
            let start = text.len().saturating_mul(3) / 4;
            let ending = text.get(start..).unwrap_or(text);
            rule.terms
                .iter()
                .map(|term| ending.matches(term).count())
                .sum()
        }
        "sentence-opening" => repeated_sentence_openings(sentences, rule.threshold),
        _ => 0,
    }
}

fn ending_monotony(sentences: &[String], threshold: usize) -> usize {
    if sentences.len() < threshold {
        return 0;
    }
    let mut counts = BTreeMap::new();
    for sentence in sentences {
        let ending = sentence
            .trim_end_matches(|character: char| {
                character.is_ascii_punctuation() || character == '。'
            })
            .chars()
            .rev()
            .take(2)
            .collect::<String>();
        *counts.entry(ending).or_insert(0_usize) += 1;
    }
    counts.values().copied().max().unwrap_or(0)
}

fn repeated_sentence_openings(sentences: &[String], threshold: usize) -> usize {
    let mut counts = BTreeMap::new();
    for sentence in sentences {
        if let Some(first) = sentence.split_whitespace().next() {
            *counts.entry(first).or_insert(0_usize) += 1;
        }
    }
    counts
        .values()
        .copied()
        .filter(|count| *count >= threshold)
        .sum()
}

fn protected_spans(text: &str) -> Vec<ProtectedSpan> {
    static PATTERNS: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();
    let patterns = PATTERNS.get_or_init(|| {
        vec![
            (
                "markdown-link",
                Regex::new(r"\[[^\]]+\]\([^)]+\)").expect("link regex"),
            ),
            ("url", Regex::new(r"https?://[^\s)>]+").expect("URL regex")),
            ("code", Regex::new(r"`[^`\r\n]+`").expect("code regex")),
            (
                "version",
                Regex::new(r"\bv?\d+\.\d+(?:\.\d+)?(?:-[A-Za-z0-9.]+)?\b").expect("version regex"),
            ),
            (
                "number",
                Regex::new(
                    r"\b\d[\d,]*(?:\.\d+)?(?:%|ms|초|분|시간|MB|MiB|GB|원|개|건|명|회|배|점)?\b",
                )
                .expect("number regex"),
            ),
            (
                "path",
                Regex::new(r"(?:[A-Za-z]:\\|/)[^\s`]+").expect("path regex"),
            ),
        ]
    });
    let mut values = BTreeSet::new();
    for (kind, pattern) in patterns {
        for matched in pattern.find_iter(text) {
            values.insert(ProtectedSpan {
                kind: (*kind).to_owned(),
                value: matched.as_str().to_owned(),
            });
        }
    }
    for (open, close) in [('“', '”'), ('‘', '’'), ('"', '"')] {
        let mut start = None;
        for (index, character) in text.char_indices() {
            if character == open && start.is_none() {
                start = Some(index);
            } else if character == close {
                if let Some(from) = start.take() {
                    values.insert(ProtectedSpan {
                        kind: "quotation".to_owned(),
                        value: text[from..index + character.len_utf8()].to_owned(),
                    });
                }
            }
        }
    }
    values.into_iter().collect()
}

fn modality(text: &str) -> BTreeMap<&'static str, usize> {
    ["해야", "필요", "가능", "듯", "보인다", "수 있다", "수 없다"]
        .into_iter()
        .map(|term| (term, text.matches(term).count()))
        .collect()
}

fn bigram_counts(text: &str) -> BTreeMap<(char, char), usize> {
    let characters = text.chars().collect::<Vec<_>>();
    let mut counts = BTreeMap::new();
    for pair in characters.windows(2) {
        *counts.entry((pair[0], pair[1])).or_insert(0) += 1;
    }
    counts
}

fn bigram_change_rate(left: &str, right: &str) -> f64 {
    if left == right {
        return 0.0;
    }
    let left_counts = bigram_counts(left);
    let right_counts = bigram_counts(right);
    let left_total = left_counts.values().sum::<usize>();
    let right_total = right_counts.values().sum::<usize>();
    let denominator = left_total.max(right_total);
    if denominator == 0 {
        return 1.0;
    }
    let common = left_counts
        .iter()
        .map(|(key, count)| count.min(right_counts.get(key).unwrap_or(&0)))
        .sum::<usize>();
    1.0 - ratio(common, denominator)
}

fn line_touch_rate(left: &str, right: &str) -> f64 {
    let left = left.lines().collect::<Vec<_>>();
    let right = right.lines().collect::<Vec<_>>();
    let total = left.len().max(right.len());
    if total == 0 {
        return 0.0;
    }
    let changed = (0..total)
        .filter(|index| left.get(*index) != right.get(*index))
        .count();
    ratio(changed, total)
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    let numerator = u32::try_from(numerator).unwrap_or(u32::MAX);
    let denominator = u32::try_from(denominator).unwrap_or(u32::MAX);
    f64::from(numerator) / f64::from(denominator)
}

#[cfg(test)]
mod tests {
    use super::{inspect, sanitize_text, verify, KoreanProfile};

    #[test]
    fn detects_frequency_and_absolute_patterns() {
        let text =
            "분석을 통해 확인했다. 결과를 통해 비교했다. 자료를 통해 검증했다. 보여진 결과다.";
        let result = inspect(KoreanProfile::Response, text).expect("inspection");
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.rule_id == "A-2"));
        assert!(result
            .findings
            .iter()
            .any(|finding| finding.rule_id == "A-8"));
        assert_eq!(result.route_hint, "heavy");
    }

    #[test]
    fn verification_preserves_numbers_links_quotes_and_modality() {
        let original = "`hive test`는 12.5%를 기록했다. [근거](https://example.com)는 “유지해야 한다”라고 썼다.";
        let safe =
            "`hive test` 결과는 12.5%였다. [근거](https://example.com)는 “유지해야 한다”라고 썼다.";
        let changed =
            "`hive test` 결과는 13%였다. [근거](https://example.net)는 “유지한다”라고 썼다.";
        assert!(
            verify(KoreanProfile::Response, original, safe)
                .expect("safe verification")
                .accepted
        );
        let rejected = verify(KoreanProfile::Response, original, changed).expect("rejected");
        assert!(!rejected.accepted);
        assert!(rejected.fallback_required);
        assert!(!rejected.protected_spans_preserved);
        assert!(!rejected.modality_preserved);
    }

    #[test]
    fn verbatim_rejects_any_change_and_sanitize_removes_only_controls() {
        assert!(
            !verify(KoreanProfile::Verbatim, "원문", "수정")
                .expect("verbatim")
                .accepted
        );
        assert_eq!(
            sanitize_text("\u{1100}\u{1161}\u{200B}나\u{202E}다"),
            "가나다"
        );
    }
}
