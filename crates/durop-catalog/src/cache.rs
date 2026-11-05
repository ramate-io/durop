//! Private owned preparation stages prevent publication before validation.

use super::source::MAX_PACK_BYTES;
use super::*;
use sha2::{Digest, Sha256};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

// Include derived-format policy in keys while retaining existing cache identities.
const CACHE_VERSION: &str = "catalog-v1";

// Separate concurrent preparations in the same application process.
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Application-owned immutable snapshots and the currently selected catalog.
///
/// Keep this directory outside local source packs and prevent packs from writing
/// into it. Preparation follows owned materialized -> validated -> published
/// stages; only the validated stage can publish. Pins remain valid until the
/// application explicitly removes their cache files; this cache performs no GC.
pub struct CatalogCache {
    // Canonical parent allows all preparations to publish via same-filesystem rename.
    root: PathBuf,
    // Activation is a single assignment after the entire preparation succeeds.
    active: Option<Arc<Catalog>>,
}

impl CatalogCache {
    /// Create/open an application-owned cache directory, initially without a pin.
    ///
    /// # Errors
    /// Returns filesystem failures when the root cannot be created/canonicalized.
    pub fn new(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)?;
        Ok(Self {
            root: fs::canonicalize(root)?,
            active: None,
        })
    }

    /// Retain the active catalog independently of later preparations/refreshes.
    pub fn pin(&self) -> Option<Arc<Catalog>> {
        self.active.clone()
    }

    /// Prepare and validate a source before atomically selecting its snapshot.
    ///
    /// # Errors
    /// Any acquisition, validation or publication error leaves the active pin
    /// unchanged. Existing callers may continue passing [`PackSource`], while
    /// alternate sources implement [`CatalogSource`] directly.
    pub fn refresh(&mut self, source: &dyn CatalogSource) -> Result<Arc<Catalog>> {
        let prepared = self.prepare(source)?;
        self.active = Some(prepared.clone());
        Ok(prepared)
    }

    /// Materialize, validate and publish offline files without changing the pin.
    ///
    /// Run on a worker: the source may perform blocking filesystem/network I/O.
    /// Source data is checked in an isolated workspace and published only after
    /// validation succeeds. Temporary files are removed on success or failure.
    ///
    /// # Errors
    /// Rejects invalid/linked/oversized materialized data, source failures and
    /// filesystem publication errors. It never partially replaces an active pin.
    pub fn prepare(&self, source: &dyn CatalogSource) -> Result<Arc<Catalog>> {
        MaterializedCatalog::acquire(&self.root, source)?
            .validate()?
            .publish(&self.root)
    }
}

// Own cleanup independently of which stage returns an error or unwinds.
struct StagingWorkspace {
    root: PathBuf,
    pack: PathBuf,
    scratch: PathBuf,
}

impl StagingWorkspace {
    // The workspace and destination share a filesystem for atomic publication.
    fn create(cache: &Path) -> Result<Self> {
        let root = cache.join(format!(
            ".staging-{}-{}",
            std::process::id(),
            STAGING_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let workspace = Self {
            pack: root.join("pack"),
            scratch: root.join("scratch"),
            root,
        };
        fs::create_dir(&workspace.pack)?;
        fs::create_dir(&workspace.scratch)?;
        Ok(workspace)
    }

    // Custom sources must not bypass the copy boundary or built-in size limits.
    fn validate_files(&self) -> Result<()> {
        let mut directories = vec![(self.pack.clone(), 0)];
        let mut visited = 0;
        let mut total = 0;
        while let Some((directory, depth)) = directories.pop() {
            visited += 1;
            if visited > 10_000 || depth > 64 {
                return Err(CatalogError::new(
                    "pack directory count/depth limit exceeded",
                ));
            }
            let metadata = fs::symlink_metadata(&directory)?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                return Err(CatalogError::new(
                    "materialized pack must contain independent regular files/directories, not symlinks",
                ));
            }
            for entry in fs::read_dir(directory)? {
                let path = entry?.path();
                let metadata = fs::symlink_metadata(&path)?;
                if metadata.file_type().is_symlink() {
                    return Err(CatalogError::new(
                        "materialized pack cannot contain symlinks",
                    ));
                }
                if metadata.is_dir() {
                    directories.push((path, depth + 1));
                    continue;
                }
                if !metadata.is_file() {
                    return Err(CatalogError::new(
                        "materialized pack must contain regular files",
                    ));
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if metadata.nlink() != 1 {
                        return Err(CatalogError::new(
                            "materialized files must not share hard links with mutable sources",
                        ));
                    }
                }
                if metadata.len() > Catalog::MAX_FILE_BYTES {
                    return Err(CatalogError::new("file exceeds 256 MiB limit"));
                }
                total += metadata.len();
                if total > MAX_PACK_BYTES {
                    return Err(CatalogError::new("pack exceeds 1 GiB limit"));
                }
            }
        }
        Ok(())
    }
}

impl Drop for StagingWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Acquisition cannot publish: its only transition requires structural validation.
struct MaterializedCatalog {
    workspace: StagingWorkspace,
    source: SourceRevision,
}

impl MaterializedCatalog {
    // Sources can leave partial output; workspace ownership guarantees cleanup.
    fn acquire(cache: &Path, source: &dyn CatalogSource) -> Result<Self> {
        let workspace = StagingWorkspace::create(cache)?;
        let source = source.materialize(&workspace.pack, &workspace.scratch)?;
        Ok(Self { workspace, source })
    }

    // Consuming the materialized state prevents reusing it after successful validation.
    fn validate(self) -> Result<ValidatedCatalog> {
        if self.source.identity.is_empty() || self.source.revision.is_empty() {
            return Err(CatalogError::new(
                "catalog source identity and revision must be nonempty",
            ));
        }
        self.workspace.validate_files()?;
        let catalog = Catalog::load(&self.workspace.pack)?;
        Ok(ValidatedCatalog {
            workspace: self.workspace,
            source: self.source,
            catalog,
        })
    }
}

// Private construction plus ownership makes validation a prerequisite to publishing.
struct ValidatedCatalog {
    workspace: StagingWorkspace,
    source: SourceRevision,
    catalog: Catalog,
}

impl ValidatedCatalog {
    // Length framing preserves the existing key scheme for all built-in sources.
    fn cache_key(&self) -> String {
        let mut digest = Sha256::new();
        for part in [CACHE_VERSION, &self.source.identity, &self.source.revision] {
            digest.update((part.len() as u64).to_be_bytes());
            digest.update(part.as_bytes());
        }
        format!("{:x}", digest.finalize())
    }

    // Only validated bytes can reach the atomic rename; existing snapshots win races.
    fn publish(self, cache: &Path) -> Result<Arc<Catalog>> {
        let destination = cache.join(self.cache_key());
        if !destination.exists() {
            match fs::rename(&self.catalog.root, &destination) {
                Ok(()) => (),
                Err(_) if destination.is_dir() => (),
                Err(error) => return Err(error.into()),
            }
        }

        // Resolve final paths again, including when another preparation won.
        let mut catalog = Catalog::load(&destination)?;
        catalog.revision = self.source.revision;
        // Keep the workspace alive through publication; Drop now clears scratch.
        drop(self.workspace);
        Ok(Arc::new(catalog))
    }
}
