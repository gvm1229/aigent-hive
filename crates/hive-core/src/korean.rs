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
#[serde(deny_unknown_fields)]
struct RulesPack {
    schema_version: u32,
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
#[serde(deny_unknown_fields)]
struct ProfileContract {
    max_change_rate: f64,
    max_touch_rate: f64,
    rewrite: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    id: String,
    category: String,
    severity: String,
    kind: String,
    terms: Vec<String>,
    threshold: usize,
}

/// Parsed rules whose shape, limits and non-overridable safety contract are valid.
#[derive(Clone, Debug)]
pub struct KoreanRulesPack(RulesPack);

impl KoreanRulesPack {
    /// Parse one bounded language pack without executing any supplied code.
    ///
    /// # Errors
    /// Returns an error for invalid fields, unsupported rules, or weakened safety limits.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        parse_rules(bytes).map(Self)
    }

    /// Return identity fields for binding to a separately verified manifest.
    #[must_use]
    pub fn identity(&self) -> (&str, &str, u32) {
        (
            &self.0.pack_id,
            &self.0.pack_version,
            self.0.transform_version,
        )
    }

    /// Inspect text with these validated rules.
    ///
    /// # Errors
    /// Returns an error for an invalid profile.
    pub fn inspect(&self, profile: KoreanProfile, text: &str) -> Result<KoreanInspection, String> {
        inspect_pack(&self.0, profile, text)
    }

    /// Apply deterministic checks; this is not a proof of complete semantic equivalence.
    ///
    /// # Errors
    /// Returns an error for an invalid profile.
    pub fn verify(
        &self,
        profile: KoreanProfile,
        before: &str,
        after: &str,
    ) -> Result<KoreanVerification, String> {
        verify_pack(&self.0, profile, before, after)
    }
}

fn parse_rules(bytes: &[u8]) -> Result<RulesPack, String> {
    if bytes.len() > 2 * 1024 * 1024 {
        return Err("Korean rules exceed the size limit".to_owned());
    }
    let pack: RulesPack =
        serde_json::from_slice(bytes).map_err(|error| format!("invalid Korean rules: {error}"))?;
    let version =
        Regex::new(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$").expect("version regex");
    if pack.schema_version != 1
        || pack.pack_id != "im-not-ai-korean-core"
        || pack.pack_version.len() > 64
        || !version.is_match(&pack.pack_version)
        || pack.transform_version == 0
        || pack.profiles.len() != 5
        || pack.rules.is_empty()
        || pack.rules.len() > 64
    {
        return Err("invalid Korean rule-pack identity or inventory".to_owned());
    }
    for (name, change_limit, touch_limit) in [
        ("response", 0.30, 0.50),
        ("release-note", 0.25, 0.40),
        ("documentation", 0.25, 0.40),
        ("technical", 0.20, 0.30),
        ("verbatim", 0.0, 0.0),
    ] {
        let contract = pack
            .profiles
            .get(name)
            .ok_or_else(|| format!("missing Korean profile: {name}"))?;
        if !(0.0..=change_limit).contains(&contract.max_change_rate)
            || !(0.0..=touch_limit).contains(&contract.max_touch_rate)
            || (name == "verbatim" && contract.rewrite)
        {
            return Err(format!("Korean profile weakens its safety limits: {name}"));
        }
    }
    let mut ids = BTreeSet::new();
    for rule in &pack.rules {
        let term_based = matches!(
            rule.kind.as_str(),
            "absolute"
                | "document-frequency"
                | "paragraph-frequency"
                | "suffix-density"
                | "ending-scope"
        );
        if rule.id.is_empty()
            || rule.id.len() > 64
            || !rule
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || !ids.insert(&rule.id)
            || rule.category.is_empty()
            || rule.category.len() > 64
            || !matches!(rule.severity.as_str(), "S1" | "S2")
            || (!term_based
                && !matches!(rule.kind.as_str(), "ending-monotony" | "sentence-opening"))
            || !(1..=10_000).contains(&rule.threshold)
            || rule.terms.len() > 16
            || (term_based && rule.terms.is_empty())
            || rule.terms.iter().any(|term| {
                term.is_empty() || term.len() > 256 || term.chars().any(char::is_control)
            })
        {
            return Err(format!("invalid or unsupported Korean rule: {}", rule.id));
        }
    }
    let baseline: RulesPack =
        serde_json::from_slice(RULES_BYTES).map_err(|error| error.to_string())?;
    for (actual, expected) in [
        (&pack.protected_spans, &baseline.protected_spans),
        (&pack.prohibited_intents, &baseline.prohibited_intents),
        (&pack.allowed_hygiene, &baseline.allowed_hygiene),
    ] {
        if actual.len() != expected.len()
            || actual.iter().collect::<BTreeSet<_>>() != expected.iter().collect::<BTreeSet<_>>()
        {
            return Err("Korean pack cannot replace the compiled integrity contract".to_owned());
        }
    }
    Ok(pack)
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
        .get_or_init(|| parse_rules(RULES_BYTES))
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
    inspect_pack(pack, profile, text)
}

fn inspect_pack(
    pack: &RulesPack,
    profile: KoreanProfile,
    text: &str,
) -> Result<KoreanInspection, String> {
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
    verify_pack(pack, profile, original, candidate)
}

fn verify_pack(
    pack: &RulesPack,
    profile: KoreanProfile,
    original: &str,
    candidate: &str,
) -> Result<KoreanVerification, String> {
    let contract = profile_contract(pack, profile)?;
    let change_rate = bigram_change_rate(original, candidate);
    let touch_rate = line_touch_rate(original, candidate);
    let protected_spans_preserved = protected_spans(original) == protected_spans(candidate);
    let modality_preserved = modality(original) == modality(candidate);
    let negation_preserved = negation_contexts(original) == negation_contexts(candidate);
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
    if !negation_preserved {
        failures.push("negation-context-changed".to_owned());
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
    for (index, character) in text.char_indices() {
        current.push(character);
        let boundary = character != '.'
            || text[index + character.len_utf8()..]
                .chars()
                .next()
                .is_none_or(char::is_whitespace);
        if matches!(character, '.' | '!' | '?' | '。') && boundary {
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

// Preserve negative clauses conservatively, including their subject. Counting negation words
// alone would accept swapping a prohibition between two subjects. Host review is still required
// for semantic changes outside these explicit forms.
fn negation_contexts(text: &str) -> Vec<String> {
    static NEGATION: OnceLock<Regex> = OnceLock::new();
    let pattern = NEGATION.get_or_init(|| {
        Regex::new(
            r"않|지\s*(?:못|말|마)|(?:^|\s)안\s|(?:^|\s)못(?:\s|하|해|했|할|한|합)|없|아니|아닙|아닌|아닐|아님|불가|금지",
        )
        .expect("negation regex")
    });
    sentences(text)
        .into_iter()
        .filter(|sentence| pattern.is_match(sentence))
        .map(|sentence| sentence.split_whitespace().collect::<Vec<_>>().join(" "))
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
    use super::{inspect, sanitize_text, verify, KoreanProfile, KoreanRulesPack};

    #[test]
    fn rules_reject_invalid_shape_limits_and_weakened_integrity() {
        let source: serde_json::Value =
            serde_json::from_slice(super::RULES_BYTES).expect("embedded rules");
        for (pointer, value) in [
            ("/schema_version", serde_json::json!(9)),
            ("/rules/0/threshold", serde_json::json!(0)),
            ("/rules/0/kind", serde_json::json!("unknown")),
            ("/profiles/response/max_change_rate", serde_json::json!(1.0)),
            ("/profiles/verbatim/rewrite", serde_json::json!(true)),
            ("/protected_spans", serde_json::json!([])),
        ] {
            let mut candidate = source.clone();
            *candidate.pointer_mut(pointer).expect("field") = value;
            assert!(
                KoreanRulesPack::parse(&serde_json::to_vec(&candidate).expect("bytes")).is_err(),
                "{pointer}"
            );
        }
        assert!(KoreanRulesPack::parse(b"{}").is_err());
        assert!(KoreanRulesPack::parse(super::RULES_BYTES).is_ok());
    }

    #[test]
    fn negation_context_preserves_polarity_and_subject_without_claiming_full_semantics() {
        let prefix =
            "검토 결과와 저장 위치를 확인했습니다. 나머지 설정과 실행 순서는 그대로 유지합니다. ";
        for (before, after) in [
            (
                "원본 v1.2는 삭제하지 않습니다.",
                "사본 v1.2는 삭제하지 않습니다.",
            ),
            (
                "이 설정은 자동 삭제가 아닐 것입니다.",
                "이 설정은 자동 삭제가 맞을 것입니다.",
            ),
            (
                "이 설정은 파일을 삭제 못합니다.",
                "이 설정은 파일을 삭제합니다.",
            ),
            ("파일을 삭제하지 않습니다.", "파일을 삭제합니다."),
            ("파일을 안 지웁니다.", "파일을 지웁니다."),
            (
                "원본은 삭제하지 않습니다. 사본은 삭제합니다.",
                "원본은 삭제합니다. 사본은 삭제하지 않습니다.",
            ),
        ] {
            let result = verify(
                KoreanProfile::Response,
                &format!("{prefix}{before}"),
                &format!("{prefix}{after}"),
            )
            .expect("verification");
            assert!(!result.accepted);
            assert!(result
                .failures
                .iter()
                .any(|failure| failure == "negation-context-changed"));
        }
        let text = "안내 문서에서 단어의 뜻을 확인했습니다.";
        assert!(
            verify(KoreanProfile::Response, text, text)
                .expect("unchanged")
                .accepted
        );
    }

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
