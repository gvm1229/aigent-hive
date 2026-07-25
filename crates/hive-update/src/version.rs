use crate::UpdateError;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Strict three-component product version.
#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SemVersion {
    /// Breaking contract baseline.
    pub major: u64,
    /// Backward-compatible feature generation.
    pub minor: u64,
    /// Compatible fix generation.
    pub patch: u64,
}

impl FromStr for SemVersion {
    type Err = ReleasePolicyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty()
            || value.starts_with('v')
            || value.contains(['-', '+', ' ', '\t', '\r', '\n'])
        {
            return Err(ReleasePolicyError::InvalidVersion(value.to_owned()));
        }
        let mut parts = value.split('.');
        let major = parse_component(parts.next(), value)?;
        let minor = parse_component(parts.next(), value)?;
        let patch = parse_component(parts.next(), value)?;
        if parts.next().is_some() {
            return Err(ReleasePolicyError::InvalidVersion(value.to_owned()));
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

fn parse_component(value: Option<&str>, original: &str) -> Result<u64, ReleasePolicyError> {
    let value = value.ok_or_else(|| ReleasePolicyError::InvalidVersion(original.to_owned()))?;
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ReleasePolicyError::InvalidVersion(original.to_owned()));
    }
    value
        .parse()
        .map_err(|_| ReleasePolicyError::InvalidVersion(original.to_owned()))
}

impl Display for SemVersion {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Declared release intent.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseClass {
    /// Additive shipped behavior.
    Feature,
    /// Compatible fix without public-surface addition.
    Bugfix,
    /// No shipped artifact change.
    DocumentationOnly,
    /// Breaking compatibility baseline.
    Breaking,
}

/// Observed public release-surface delta.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceDelta {
    /// No shipped bytes or contracts changed.
    None,
    /// Implementation or packaging changed without a public addition.
    CompatibleFix,
    /// An additive feature, schema, Skill, or projection was added.
    AdditiveFeature,
    /// A supported contract was removed or changed incompatibly.
    Breaking,
}

/// Explicit user authority for a breaking release.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MajorApproval {
    /// Exact installed source version shown to the user.
    pub source_version: SemVersion,
    /// Exact target supplied by the user. No target is inferred.
    pub exact_target: SemVersion,
    /// Digest of the exact renderer dry-run plan.
    pub release_plan_digest: String,
    /// Digest of the independently observed surface and preservation report.
    pub compatibility_report_digest: String,
    /// Digest of the signed migration table.
    pub migration_table_digest: String,
    /// Separate human confirmation was recorded.
    pub human_confirmed: bool,
    /// Digest of the exact confirmation document bytes.
    pub confirmation_digest: String,
}

/// Accepted version transition.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VersionTransition {
    /// Same version and no shipped delta.
    Unchanged,
    /// Patch increment.
    Patch,
    /// Minor increment.
    Minor,
    /// Explicit breaking transition.
    Major,
}

/// Deterministic version-policy failure.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReleasePolicyError {
    /// Version is not exact `X.Y.Z`.
    InvalidVersion(String),
    /// Declared class and observed surface disagree.
    ClassificationMismatch(String),
    /// Target does not use the required minor or patch transition.
    InvalidTransition(String),
    /// Same-major release contains a breaking surface.
    SameMajorBreaking,
    /// Exact target and separately bound human approval are missing or invalid.
    MajorApprovalRequired,
}

impl Display for ReleasePolicyError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVersion(value) => write!(formatter, "invalid exact product version: {value}"),
            Self::ClassificationMismatch(message) | Self::InvalidTransition(message) => {
                formatter.write_str(message)
            }
            Self::SameMajorBreaking => {
                formatter.write_str("same-major release contains a breaking surface")
            }
            Self::MajorApprovalRequired => formatter.write_str(
                "breaking release requires an exact user-supplied target and separate bound human confirmation",
            ),
        }
    }
}

impl std::error::Error for ReleasePolicyError {}

impl From<ReleasePolicyError> for UpdateError {
    fn from(error: ReleasePolicyError) -> Self {
        Self::Compatibility(error.to_string())
    }
}

/// Validate release classification and product-version movement.
///
/// A major target is never calculated. The caller must supply an exact target
/// here; transaction preparation separately binds human confirmation to the
/// computed plan, compatibility/preservation report, and migration table.
///
/// # Errors
///
/// Returns a policy error when classification and observed surface differ, the
/// requested version movement is invalid, or the exact major target is missing.
pub fn classify_release(
    current: SemVersion,
    target: SemVersion,
    declared: ReleaseClass,
    observed: SurfaceDelta,
    exact_major_target: Option<SemVersion>,
) -> Result<VersionTransition, ReleasePolicyError> {
    if target.major == current.major && observed == SurfaceDelta::Breaking {
        return Err(ReleasePolicyError::SameMajorBreaking);
    }
    let expected_class = match observed {
        SurfaceDelta::None => ReleaseClass::DocumentationOnly,
        SurfaceDelta::CompatibleFix => ReleaseClass::Bugfix,
        SurfaceDelta::AdditiveFeature => ReleaseClass::Feature,
        SurfaceDelta::Breaking => ReleaseClass::Breaking,
    };
    if declared != expected_class {
        return Err(ReleasePolicyError::ClassificationMismatch(format!(
            "declared {declared:?} release does not match observed {observed:?} surface"
        )));
    }

    if target.major != current.major {
        if target.major != current.major.saturating_add(1) || target.minor != 0 || target.patch != 0
        {
            return Err(ReleasePolicyError::InvalidTransition(format!(
                "breaking release must move {current} to the exact next-major .0.0 target"
            )));
        }
        if declared != ReleaseClass::Breaking || exact_major_target != Some(target) {
            return Err(ReleasePolicyError::MajorApprovalRequired);
        }
        return Ok(VersionTransition::Major);
    }

    match declared {
        ReleaseClass::DocumentationOnly if target == current => Ok(VersionTransition::Unchanged),
        ReleaseClass::Bugfix
            if target.minor == current.minor && target.patch == current.patch.saturating_add(1) =>
        {
            Ok(VersionTransition::Patch)
        }
        ReleaseClass::Feature
            if target.minor == current.minor.saturating_add(1) && target.patch == 0 =>
        {
            Ok(VersionTransition::Minor)
        }
        ReleaseClass::Breaking => Err(ReleasePolicyError::SameMajorBreaking),
        _ => Err(ReleasePolicyError::InvalidTransition(format!(
            "{declared:?} release cannot move {current} to {target}"
        ))),
    }
}

pub(crate) fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(value: &str) -> SemVersion {
        value.parse().expect("valid test version")
    }

    #[test]
    fn exact_versions_reject_semver_extensions_and_leading_zeroes() {
        for invalid in ["", "v0.7.0", "0.7", "0.7.0-alpha", "0.7.0+build", "00.7.0"] {
            assert!(invalid.parse::<SemVersion>().is_err(), "{invalid}");
        }
        assert_eq!(version("0.7.0").to_string(), "0.7.0");
    }

    #[test]
    fn feature_requires_minor_and_bugfix_requires_patch() {
        let current = version("0.6.0");
        assert_eq!(
            classify_release(
                current,
                version("0.7.0"),
                ReleaseClass::Feature,
                SurfaceDelta::AdditiveFeature,
                None,
            ),
            Ok(VersionTransition::Minor)
        );
        assert_eq!(
            classify_release(
                current,
                version("0.6.1"),
                ReleaseClass::Bugfix,
                SurfaceDelta::CompatibleFix,
                None,
            ),
            Ok(VersionTransition::Patch)
        );
        assert!(classify_release(
            current,
            version("0.6.1"),
            ReleaseClass::Feature,
            SurfaceDelta::AdditiveFeature,
            None,
        )
        .is_err());
        assert!(classify_release(
            current,
            version("0.7.0"),
            ReleaseClass::Bugfix,
            SurfaceDelta::CompatibleFix,
            None,
        )
        .is_err());
    }

    #[test]
    fn major_never_proceeds_without_exact_separate_confirmation() {
        let current = version("0.6.0");
        let target = version("1.0.0");
        assert!(classify_release(
            current,
            target,
            ReleaseClass::Breaking,
            SurfaceDelta::Breaking,
            None,
        )
        .is_err());
        let approval = MajorApproval {
            source_version: current,
            exact_target: target,
            release_plan_digest: format!("sha256:{}", "b".repeat(64)),
            compatibility_report_digest: format!("sha256:{}", "c".repeat(64)),
            migration_table_digest: format!("sha256:{}", "d".repeat(64)),
            human_confirmed: true,
            confirmation_digest: format!("sha256:{}", "a".repeat(64)),
        };
        assert_eq!(
            classify_release(
                current,
                target,
                ReleaseClass::Breaking,
                SurfaceDelta::Breaking,
                Some(approval.exact_target),
            ),
            Ok(VersionTransition::Major)
        );
        let wrong = MajorApproval {
            exact_target: version("2.0.0"),
            ..approval
        };
        assert!(classify_release(
            current,
            target,
            ReleaseClass::Breaking,
            SurfaceDelta::Breaking,
            Some(wrong.exact_target),
        )
        .is_err());
    }

    #[test]
    fn breaking_release_rejects_skipped_or_nonzero_next_major_targets() {
        let current = version("0.7.4");
        for target in [version("2.0.0"), version("1.1.0"), version("1.0.1")] {
            assert!(matches!(
                classify_release(
                    current,
                    target,
                    ReleaseClass::Breaking,
                    SurfaceDelta::Breaking,
                    Some(target),
                ),
                Err(ReleasePolicyError::InvalidTransition(_))
            ));
        }
        let target = version("1.0.0");
        assert_eq!(
            classify_release(
                current,
                target,
                ReleaseClass::Breaking,
                SurfaceDelta::Breaking,
                Some(target),
            ),
            Ok(VersionTransition::Major)
        );
    }

    #[test]
    fn pre_one_same_major_breaking_change_is_rejected() {
        assert_eq!(
            classify_release(
                version("0.6.0"),
                version("0.7.0"),
                ReleaseClass::Breaking,
                SurfaceDelta::Breaking,
                None,
            ),
            Err(ReleasePolicyError::SameMajorBreaking)
        );
    }
}
