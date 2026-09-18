//! Recheck only the evidence behind returned claims; never repair on a read path.

use super::{
    claim_locator, parse_claim_markdown, pin_absolute_directory, rag_error, read_bounded_required,
    sha256_digest, AssertionStatus, CanonicalClaim, CollectionRegistry, CollectionState, Path,
    RagStore, WikiError, MAX_CLAIM_BYTES, USER_ROOT_COLLECTION_ID,
};

impl RagStore {
    pub(super) fn validate_claim_sources(
        &self,
        registry: &CollectionRegistry,
        claim: &CanonicalClaim,
    ) -> Result<(), WikiError> {
        let Some(metadata) = &claim.scan_metadata else {
            return Ok(());
        };
        // Promoted facts retain the originating claim locator. Resolve at most one
        // hop and require its canonical identity before touching a source directory.
        let source_claim;
        let source = if claim.collection_id == USER_ROOT_COLLECTION_ID {
            let relative = Path::new(&claim.provenance.locator);
            let bytes = read_bounded_required(
                &self.root,
                relative,
                MAX_CLAIM_BYTES,
                "promoted source claim",
            )?;
            source_claim = parse_claim_markdown(
                &claim.provenance.locator,
                std::str::from_utf8(&bytes).map_err(|_| stale_source())?,
            )
            .map_err(rag_error)?;
            if source_claim.collection_id == USER_ROOT_COLLECTION_ID
                || claim_locator(&source_claim.collection_id, &source_claim.claim_id)
                    != claim.provenance.locator
                || source_claim.status == AssertionStatus::Superseded
                || source_claim.provenance.digest != claim.provenance.digest
                || source_claim
                    .scan_metadata
                    .as_ref()
                    .map(|value| &value.evidence)
                    != Some(&metadata.evidence)
            {
                return Err(stale_source());
            }
            &source_claim
        } else {
            claim
        };
        let collection = registry
            .collections
            .iter()
            .find(|entry| entry.collection_id == source.collection_id)
            .filter(|entry| entry.state == CollectionState::Attached)
            .ok_or_else(stale_source)?;
        let root = pin_absolute_directory(
            Path::new(
                collection
                    .local_locator
                    .as_deref()
                    .ok_or_else(stale_source)?,
            ),
            "claim source root",
        )?;
        if metadata.evidence.is_empty() || metadata.evidence.len() > 16 {
            return Err(stale_source());
        }
        for evidence in &metadata.evidence {
            let bytes = read_bounded_required(
                &root,
                Path::new(&evidence.locator),
                crate::scan::ScanLimits::default().max_file_bytes,
                "claim source evidence",
            )?;
            if sha256_digest(&bytes) != evidence.content_digest {
                return Err(stale_source());
            }
        }
        Ok(())
    }
}

fn stale_source() -> WikiError {
    WikiError::Verification(
        "claim source evidence is unavailable or changed; explicitly rescan and review before reuse"
            .to_owned(),
    )
}
