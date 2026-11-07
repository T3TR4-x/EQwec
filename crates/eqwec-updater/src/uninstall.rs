//! Journaled product uninstall.
//!
//! Uninstall is a transaction with two complete outcomes: the product is
//! installed, or its tree and registry record are gone. The journal records
//! `uninstall-intent` before any file is removed, so [`crate::recover`] can
//! finish an interrupted uninstall without guessing.
//!
//! Order:
//!
//! 1. validate the record and its containment;
//! 2. append `uninstall-intent`;
//! 3. remove the product root without following links;
//! 4. remove the registry record and write the registry atomically;
//! 5. append `uninstalled`.
//!
//! The registry is updated after the files, matching the Hub schema: a record
//! is removed only after the uninstall completes. Recovery retries the
//! registry step if the process dies between steps 3 and 5.

use eqwec_core::ProductId;

use crate::activation::{FaultInjector, FaultPoint, NoFaults, remove_product_tree};
use crate::error::UpdaterError;
use crate::journal::{
    IdempotencyKey, JournalContext, JournalRecord, OperationId, Phase, append_journal_record,
    read_journal,
};
use crate::layout::InstallLayout;
use crate::registry::Registry;
use crate::time::Timestamp;

/// Removes an installed product tree and its registry record.
///
/// # Errors
///
/// Returns [`UpdaterError::ProductNotInstalled`] when no record exists and the
/// operation is not a repeat, [`UpdaterError::InstallRootOutsideBase`] when the
/// record points outside the layout, and a typed journal, I/O, or registry
/// error when the transaction cannot complete.
pub fn uninstall(
    layout: &InstallLayout,
    registry: &mut Registry,
    product_id: &ProductId,
    operation_id: OperationId,
    idempotency_key: IdempotencyKey,
    updated_at: Timestamp,
) -> Result<(), UpdaterError> {
    uninstall_with_faults(
        layout,
        registry,
        product_id,
        operation_id,
        idempotency_key,
        updated_at,
        &NoFaults,
    )
}

/// Removes an installed product with a test fault injector.
///
/// # Errors
///
/// Same as [`uninstall`], plus [`UpdaterError::FaultInjected`] when the
/// injector aborts at [`FaultPoint::AfterUninstallTreeRemoved`]. An injected
/// fault leaves `uninstall-intent` unfinished so recovery resumes it.
pub fn uninstall_with_faults(
    layout: &InstallLayout,
    registry: &mut Registry,
    product_id: &ProductId,
    operation_id: OperationId,
    idempotency_key: IdempotencyKey,
    updated_at: Timestamp,
    faults: &dyn FaultInjector,
) -> Result<(), UpdaterError> {
    let contents = read_journal(&layout.journal_path())?;
    if let Some(unfinished) = contents.unfinished_operation() {
        return Err(UpdaterError::InvalidOperationTransition {
            expected: "no unfinished operation",
            found: unfinished.phase().as_str(),
        });
    }

    let Some(record) = registry.get(product_id).cloned() else {
        let already_uninstalled = contents.records().iter().any(|record| {
            record.phase() == Phase::Uninstalled
                && record.idempotency_key().as_str() == idempotency_key.as_str()
        });
        if already_uninstalled {
            return Ok(());
        }
        return Err(UpdaterError::ProductNotInstalled {
            product_id: product_id.as_str().to_owned(),
        });
    };

    let root = layout.product_root(product_id);
    if record.install_root() != root {
        return Err(UpdaterError::InstallRootOutsideBase);
    }
    layout.validate_install_root(&root)?;

    let context = JournalContext::new(
        Some(record.product_version().clone()),
        None,
        record.channel(),
        record.package_sha256(),
        None,
    )?;
    let intent = JournalRecord::new(
        operation_id.clone(),
        idempotency_key.clone(),
        product_id.clone(),
        Phase::UninstallIntent,
        updated_at,
        Some(context.clone()),
    )?;
    append_journal_record(&layout.journal_path(), &intent)?;

    if let Err(error) = remove_product_tree(layout, product_id) {
        record_failure(
            layout,
            &operation_id,
            &idempotency_key,
            product_id,
            &context,
            updated_at,
            "UNINSTALL_REMOVE_FAILED",
        )?;
        return Err(error);
    }

    faults.check(FaultPoint::AfterUninstallTreeRemoved)?;

    if let Err(error) = remove_record(layout, registry, product_id, &operation_id, updated_at) {
        record_failure(
            layout,
            &operation_id,
            &idempotency_key,
            product_id,
            &context,
            updated_at,
            "UNINSTALL_REGISTRY_FAILED",
        )?;
        return Err(error);
    }

    let done = JournalRecord::new(
        operation_id,
        idempotency_key,
        product_id.clone(),
        Phase::Uninstalled,
        updated_at,
        Some(context),
    )?;
    append_journal_record(&layout.journal_path(), &done)?;
    Ok(())
}

fn remove_record(
    layout: &InstallLayout,
    registry: &mut Registry,
    product_id: &ProductId,
    operation_id: &OperationId,
    updated_at: Timestamp,
) -> Result<(), UpdaterError> {
    registry.remove(product_id)?;
    registry.set_updated_at(updated_at);
    registry.write_atomic(&layout.registry_path(), operation_id)
}

fn record_failure(
    layout: &InstallLayout,
    operation_id: &OperationId,
    idempotency_key: &IdempotencyKey,
    product_id: &ProductId,
    context: &JournalContext,
    updated_at: Timestamp,
    error_code: &str,
) -> Result<(), UpdaterError> {
    let context = context
        .clone()
        .with_error_code(Some(error_code.to_owned()))?;
    let record = JournalRecord::new(
        operation_id.clone(),
        idempotency_key.clone(),
        product_id.clone(),
        Phase::Failed,
        updated_at,
        Some(context),
    )?;
    append_journal_record(&layout.journal_path(), &record)
}
