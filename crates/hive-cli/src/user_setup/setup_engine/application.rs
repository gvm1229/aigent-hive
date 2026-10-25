//! Shared lifecycle bookkeeping; callers retain their input, locks and execution limits.
use crate::user_setup::{
    progress_store::{ProgressPhase, ProgressSnapshot, ProgressStore, SetupProgress},
    SetupError,
};

pub(super) fn begin(
    store: &ProgressStore,
    snapshot: &ProgressSnapshot,
    progress: &mut SetupProgress,
) -> Result<(), SetupError> {
    progress.phase = ProgressPhase::Applying;
    store.save(snapshot, progress)?;
    store.record_applying(progress)
}

pub(super) fn finish(
    store: &ProgressStore,
    progress: &SetupProgress,
    application: Result<crate::ActionResult, SetupError>,
) -> Result<crate::ActionResult, SetupError> {
    match application {
        Ok(result) => {
            ProgressStore::open(store.user_root_path(), None)?.record_completion(progress)?;
            Ok(result)
        }
        Err(error) => {
            store.record_failure(progress)?;
            Err(error)
        }
    }
}
