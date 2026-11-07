//! Pure launcher state and view models.
//!
//! This module has no `egui` dependency so registry mapping and activation
//! consistency are testable without a window.

use std::path::PathBuf;

use eqwec_updater::{ActivePointer, InstallLayout, InstallationRecord, Registry};

use crate::error::LauncherError;

/// Whether a product card can launch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LaunchState {
    /// The pointer, registry, tree, and entry point agree.
    Ready,
    /// The installation needs repair before it can launch.
    RepairRequired(String),
}

/// One product card.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductView {
    /// Stable product identifier.
    pub product_id: String,
    /// Installed version.
    pub version: String,
    /// Release channel.
    pub channel: String,
    /// Active tree relative path.
    pub active_tree: String,
    /// Installation time.
    pub installed_at: String,
    /// Retained previous versions.
    pub retained: Vec<String>,
    /// Launch readiness.
    pub launch: LaunchState,
}

/// Registry view state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegistryState {
    /// No registry file exists.
    NotInitialized,
    /// Registry parsed; may be empty.
    Ready(Vec<ProductView>),
    /// Registry is corrupt, oversized, or outside the layout.
    Error(String),
}

/// Profile-lock state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LockState {
    /// This process owns the profile.
    Acquired,
    /// The profile cannot be written safely.
    Unavailable(String),
}

/// Everything the window needs to render.
#[derive(Clone, Debug)]
pub struct LauncherState {
    /// Resolved profile data directory.
    pub data_dir: PathBuf,
    /// Lock state.
    pub lock: LockState,
    /// Registry state.
    pub registry: RegistryState,
}

impl LauncherState {
    /// Loads the registry for `data_dir`.
    #[must_use]
    pub fn load(data_dir: PathBuf, lock: LockState) -> Self {
        let registry = load_registry(&data_dir);
        Self {
            data_dir,
            lock,
            registry,
        }
    }

    /// Reloads the registry without touching the lock.
    pub fn refresh(&mut self) {
        self.registry = load_registry(&self.data_dir);
    }

    /// Returns whether mutating actions are allowed.
    #[must_use]
    pub const fn can_mutate(&self) -> bool {
        matches!(self.lock, LockState::Acquired)
            && !matches!(self.registry, RegistryState::Error(_))
    }
}

fn load_registry(data_dir: &PathBuf) -> RegistryState {
    let layout = match InstallLayout::new(data_dir) {
        Ok(layout) => layout,
        Err(error) => return RegistryState::Error(error.to_string()),
    };
    match Registry::load(&layout.registry_path()) {
        Ok(None) => RegistryState::NotInitialized,
        Ok(Some(registry)) => {
            if let Err(error) = registry.validate_against_layout(&layout) {
                return RegistryState::Error(error.to_string());
            }
            let views = registry
                .installations()
                .iter()
                .map(|record| view(&layout, record))
                .collect();
            RegistryState::Ready(views)
        }
        Err(error) => RegistryState::Error(error.to_string()),
    }
}

fn view(layout: &InstallLayout, record: &InstallationRecord) -> ProductView {
    ProductView {
        product_id: record.product_id().as_str().to_owned(),
        version: record.product_version().to_string(),
        channel: record.channel().as_str().to_owned(),
        active_tree: record.active_tree().as_str().to_owned(),
        installed_at: record.installed_at().to_string(),
        retained: record
            .retained()
            .iter()
            .map(|retained| retained.product_version().to_string())
            .collect(),
        launch: launch_state(layout, record),
    }
}

fn launch_state(layout: &InstallLayout, record: &InstallationRecord) -> LaunchState {
    let pointer = match ActivePointer::load(&layout.active_pointer_path(record.product_id())) {
        Ok(pointer) => pointer,
        Err(error) => {
            return LaunchState::RepairRequired(
                LauncherError::Pointer(format!("active pointer is invalid: {error}")).to_string(),
            );
        }
    };
    let Some(pointer) = pointer else {
        return LaunchState::RepairRequired(
            LauncherError::Pointer("active pointer is missing".to_owned()).to_string(),
        );
    };
    if pointer.tree() != record.active_tree() || pointer.package_sha256() != record.package_sha256()
    {
        return LaunchState::RepairRequired(
            LauncherError::Pointer("active pointer does not match the registry".to_owned())
                .to_string(),
        );
    }
    if !layout
        .tree_path(record.product_id(), record.active_tree())
        .is_dir()
    {
        return LaunchState::RepairRequired(
            LauncherError::Pointer("active tree is missing".to_owned()).to_string(),
        );
    }
    LaunchState::Ready
}

#[cfg(test)]
mod tests {
    use super::{LaunchState, LauncherState, LockState, RegistryState};
    use eqwec_core::{Channel, ProductId, ScopeId};
    use eqwec_security::{KeyId, SafeRelativePath, Sha256Digest};
    use eqwec_updater::{
        ActivePointer, InstallLayout, InstallationRecord, OperationId, Registry, Timestamp,
    };
    use semver::Version;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn temp_base() -> PathBuf {
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        let base =
            std::env::temp_dir().join(format!("eqwec-state-{}-{unique}", std::process::id()));
        fs::create_dir_all(&base).expect("create base");
        base
    }

    fn at(seconds: i64) -> Timestamp {
        Timestamp::from_unix_seconds(seconds).expect("timestamp")
    }

    fn tree() -> SafeRelativePath {
        SafeRelativePath::new("versions/1.0.0-0123456789ab").expect("tree")
    }

    fn setup() -> (PathBuf, InstallLayout, ProductId) {
        let base = temp_base();
        let layout = InstallLayout::new(&base).expect("layout");
        layout.ensure_base_dirs().expect("dirs");
        let product = ProductId::new("eqwec-fixture").expect("product");
        fs::create_dir_all(layout.tree_path(&product, &tree())).expect("tree dir");
        let record = InstallationRecord::new(
            product.clone(),
            Version::parse("1.0.0").expect("version"),
            Channel::Stable,
            layout.product_root(&product),
            tree(),
            Vec::new(),
            KeyId::new("dev-fixture-2026").expect("key"),
            1,
            Sha256Digest::from_bytes([7; 32]),
            5,
            at(1_700_000_000),
        )
        .expect("record");
        let mut registry =
            Registry::new(ScopeId::new("default").expect("scope"), at(1_700_000_000));
        registry.upsert(record).expect("upsert");
        registry
            .write_atomic(&layout.registry_path(), &operation())
            .expect("write registry");
        (base, layout, product)
    }

    fn operation() -> OperationId {
        OperationId::parse(&"a".repeat(32)).expect("operation")
    }

    fn write_pointer(layout: &InstallLayout, product: &ProductId, sha: [u8; 32]) {
        ActivePointer::new(tree(), Sha256Digest::from_bytes(sha), at(1_700_000_100))
            .expect("pointer")
            .write_atomic(&layout.active_pointer_path(product), &operation())
            .expect("write pointer");
    }

    fn view_of(state: &LauncherState) -> &super::ProductView {
        match &state.registry {
            RegistryState::Ready(views) => views.first().expect("one view"),
            other => panic!("expected ready registry, got {other:?}"),
        }
    }

    #[test]
    fn missing_registry_is_not_initialized() {
        let base = temp_base();
        let state = LauncherState::load(base.clone(), LockState::Acquired);
        assert_eq!(state.registry, RegistryState::NotInitialized);
        fs::remove_dir_all(base).expect("cleanup");
    }

    #[test]
    fn corrupt_registry_is_an_error() {
        let (base, layout, _product) = setup();
        fs::write(layout.registry_path(), b"{not json").expect("corrupt registry");
        let state = LauncherState::load(base.clone(), LockState::Acquired);
        assert!(matches!(state.registry, RegistryState::Error(_)));
        assert!(!state.can_mutate());
        fs::remove_dir_all(base).expect("cleanup");
    }

    #[test]
    fn matching_pointer_is_ready() {
        let (base, layout, product) = setup();
        write_pointer(&layout, &product, [7; 32]);
        let state = LauncherState::load(base.clone(), LockState::Acquired);
        assert_eq!(view_of(&state).launch, LaunchState::Ready);
        fs::remove_dir_all(base).expect("cleanup");
    }

    #[test]
    fn missing_pointer_requires_repair() {
        let (base, _layout, _product) = setup();
        let state = LauncherState::load(base.clone(), LockState::Acquired);
        assert!(matches!(
            view_of(&state).launch,
            LaunchState::RepairRequired(_)
        ));
        fs::remove_dir_all(base).expect("cleanup");
    }

    #[test]
    fn mismatched_pointer_requires_repair() {
        let (base, layout, product) = setup();
        write_pointer(&layout, &product, [9; 32]);
        let state = LauncherState::load(base.clone(), LockState::Acquired);
        assert!(matches!(
            view_of(&state).launch,
            LaunchState::RepairRequired(_)
        ));
        fs::remove_dir_all(base).expect("cleanup");
    }

    #[test]
    fn missing_tree_requires_repair() {
        let (base, layout, product) = setup();
        write_pointer(&layout, &product, [7; 32]);
        fs::remove_dir_all(layout.tree_path(&product, &tree())).expect("remove tree");
        let state = LauncherState::load(base.clone(), LockState::Acquired);
        assert!(matches!(
            view_of(&state).launch,
            LaunchState::RepairRequired(_)
        ));
        fs::remove_dir_all(base).expect("cleanup");
    }

    #[test]
    fn unavailable_lock_blocks_mutation() {
        let base = temp_base();
        let state = LauncherState::load(
            base.clone(),
            LockState::Unavailable("another process".to_owned()),
        );
        assert!(!state.can_mutate());
        fs::remove_dir_all(base).expect("cleanup");
    }

    #[test]
    fn registry_path_is_inside_the_data_dir() {
        let base = temp_base();
        let layout = InstallLayout::new(&base).expect("layout");
        assert!(layout.registry_path().starts_with(Path::new(&base)));
        fs::remove_dir_all(base).expect("cleanup");
    }
}
