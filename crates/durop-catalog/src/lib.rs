//! Versioned pack validation and immutable catalog snapshots.
//!
//! [`Catalog::load`] validates a quiescent directory without copying it. For
//! session playback, [`CatalogCache`] asks a [`CatalogSource`] to materialize
//! files in an isolated workspace, validates them, then publishes a snapshot.
//! Source acquisition and cache activation are separate: preparing an update
//! never changes existing session pins or requires network access at trigger time.

#![deny(missing_docs)]

use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

mod cache;

mod source;

pub use cache::CatalogCache;
pub use source::{CatalogSource, GitCatalogSource, LocalCatalogSource, PackSource, SourceRevision};

/// Result of catalog validation, source acquisition or cache publication.
pub type Result<T> = std::result::Result<T, CatalogError>;

/// A contextual catalog failure, suitable for display before enabling playback.
///
/// Git errors deliberately exclude remote stderr and credentials.
#[derive(Debug)]
pub struct CatalogError(
    /// Human-readable validation or preparation diagnostic.
    pub String,
);

impl CatalogError {
    /// Construct a diagnostic without including tokens or remote stderr.
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CatalogError {}

impl From<std::io::Error> for CatalogError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

/// Required `durop.json` fields for schema version 1; unknown fields are rejected.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackManifest {
    /// Format version; only version 1 is currently supported.
    pub schema_version: u32,
    /// Stable namespace using ASCII letters, digits, hyphens and underscores.
    pub pack_id: String,
    /// Nonempty human-readable pack name.
    pub title: String,
}

impl PackManifest {
    // Reject unsupported authoring formats before discovering clip metadata.
    fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(CatalogError::new("unsupported schemaVersion; expected 1"));
        }
        if !Catalog::valid_id(&self.pack_id) || self.title.trim().is_empty() {
            return Err(CatalogError::new(
                "packId must use ASCII letters, digits, '-' or '_'; title must be nonempty",
            ));
        }
        Ok(())
    }
}

/// Author-supplied clip metadata, discovered recursively by `.json` path.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipMetadata {
    /// Pack-local stable identifier, independent of its category directory.
    pub id: String,
    /// Nonempty display/search title.
    pub title: String,
    /// Media path relative to this metadata file; traversal is forbidden.
    pub asset: String,
    /// Optional search labels; omitted tags default to an empty list.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Trigger/loop policy; omission uses [`Playback::default`].
    #[serde(default)]
    pub playback: Playback,
    /// Optional placement policy, including for audio-only clips.
    pub visual: Option<Visual>,
}

impl ClipMetadata {
    // Validate authoring policy before duplicate detection and opening media.
    fn validate(&self, file: &Path) -> Result<()> {
        if !Catalog::valid_id(&self.id) || self.title.trim().is_empty() {
            return Err(CatalogError::new(format!(
                "{}: invalid id or empty title",
                file.display()
            )));
        }
        self.playback.validate()?;
        if let Some(visual) = &self.visual {
            visual.validate()?;
        }
        let asset = Path::new(&self.asset);
        if asset.as_os_str().is_empty()
            || asset
                .components()
                .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
            || self.asset.contains('\\')
        {
            return Err(CatalogError::new(format!(
                "{}: asset must be a relative path without traversal",
                file.display()
            )));
        }
        Ok(())
    }
}

/// Host playback policy. The catalog validates bounds but does not run a decoder.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Playback {
    /// Policy for triggering an already active identity; defaults to restart.
    #[serde(default)]
    pub retrigger: Retrigger,
    /// Whether the host may repeat the clip within its maximum duration.
    #[serde(default, rename = "loop")]
    pub looping: bool,
    /// Positive lifetime cap in milliseconds, at most 600000; default 10000.
    #[serde(default = "Playback::default_duration_ms")]
    pub max_duration_ms: u64,
}

impl Playback {
    // Shared serde/default value keeps omitted policy fields consistent.
    fn default_duration_ms() -> u64 {
        10_000
    }

    // A looping clip must still have a finite host-enforced lifetime.
    fn validate(&self) -> Result<()> {
        if self.max_duration_ms == 0 || self.max_duration_ms > 600_000 {
            return Err(CatalogError::new(
                "maxDurationMs must be positive and at most 600000",
            ));
        }
        Ok(())
    }
}

impl Default for Playback {
    fn default() -> Self {
        Self {
            retrigger: Retrigger::default(),
            looping: false,
            max_duration_ms: Self::default_duration_ms(),
        }
    }
}

/// Behavior when the same clip identity is triggered again.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Retrigger {
    /// Stop the previous occurrence before beginning its replacement.
    #[default]
    Restart,
    /// Retain an existing occurrence instead of starting another.
    IgnoreIfPlaying,
    /// Allow independent occurrences, subject to the host's instance cap.
    Stack,
}

/// Author-requested overlay placement; dimensions are relative to a display.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Visual {
    /// Display-relative attachment point.
    pub anchor: Anchor,
    /// Fraction of overlay width, finite and strictly greater than zero up to one.
    pub width_fraction: f64,
    /// Aspect-preserving image fitting behavior.
    pub fit: Fit,
}

impl Visual {
    // Reject nonfinite or unusable dimensions before rendering is configured.
    fn validate(&self) -> Result<()> {
        if !(self.width_fraction.is_finite()
            && 0.0 < self.width_fraction
            && self.width_fraction <= 1.0)
        {
            return Err(CatalogError::new("widthFraction must be > 0 and <= 1"));
        }
        Ok(())
    }
}

/// Anchor within the selected overlay display.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Anchor {
    /// Attach to the upper-left corner.
    TopLeft,
    /// Attach to the upper-right corner.
    TopRight,
    /// Attach to the lower-left corner.
    BottomLeft,
    /// Attach to the lower-right corner.
    BottomRight,
    /// Center on the display.
    Center,
}

/// Preserve media aspect ratio while fitting the requested overlay area.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fit {
    /// Fit the whole image inside the area.
    Contain,
    /// Fill the area by cropping excess image edges.
    Cover,
}

/// Decoder qualification, separate from successful structural catalog validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecoderReadiness {
    /// Media is present and allowed by extension; the backend must still probe it.
    Unchecked,
}

/// A validated clip with stable identity and contained file locations.
#[derive(Debug, Clone)]
pub struct Clip {
    /// `packId/id`, stable across directory/category moves.
    pub identity: String,
    /// Metadata's parent directory relative to the pack root.
    pub category: PathBuf,
    /// Validated authoring metadata.
    pub metadata: ClipMetadata,
    /// Lexical metadata location used to preserve source-relative asset paths.
    pub metadata_path: PathBuf,
    /// Canonical same-basename PNG location within the pack.
    pub icon_path: PathBuf,
    /// Canonical referenced media location within the pack.
    pub asset_path: PathBuf,
    /// Explicitly unprobed until media preparation occurs.
    pub decoder_readiness: DecoderReadiness,
}

impl Clip {
    // Resolve only after identity validation/deduplication, keeping diagnostics focused.
    fn load_files(
        root: &Path,
        pack_id: &str,
        file: PathBuf,
        metadata: ClipMetadata,
    ) -> Result<Self> {
        let asset = Path::new(&metadata.asset);
        let asset_path = Catalog::resolve_path(root, &file.parent().unwrap().join(asset))?;
        let extension = asset
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ![
            "wav", "mp3", "ogg", "flac", "m4a", "aac", "mp4", "mov", "webm", "gif",
        ]
        .contains(&extension.as_str())
        {
            return Err(CatalogError::new(format!(
                "{}: unsupported media extension",
                file.display()
            )));
        }
        if !asset_path.is_file() {
            return Err(CatalogError::new("asset must be a regular file"));
        }
        if Catalog::read_file(&asset_path)?.is_empty() {
            return Err(CatalogError::new("media asset is empty"));
        }
        let icon_path =
            Catalog::resolve_path(root, &file.with_extension("png")).map_err(|error| {
                CatalogError::new(format!(
                    "{}: requires same-basename PNG icon: {error}",
                    file.display()
                ))
            })?;
        Self::validate_icon(&icon_path)?;
        let category = file.strip_prefix(root).unwrap().parent().unwrap().into();
        Ok(Self {
            identity: format!("{pack_id}/{}", metadata.id),
            category,
            metadata,
            metadata_path: file,
            icon_path,
            asset_path,
            decoder_readiness: DecoderReadiness::Unchecked,
        })
    }

    // This is a structural header check, deliberately not a PNG decoder promise.
    fn validate_icon(path: &Path) -> Result<()> {
        let png = Catalog::read_file(path)?;
        if png.len() < 33
            || &png[..8] != b"\x89PNG\r\n\x1a\n"
            || png[8..12] != [0, 0, 0, 13]
            || &png[12..16] != b"IHDR"
            || png[16..20] == [0; 4]
            || png[20..24] == [0; 4]
        {
            return Err(CatalogError::new(format!(
                "{}: invalid PNG header",
                path.display()
            )));
        }
        Ok(())
    }
}

/// A structurally validated catalog. Cache preparation gives it an immutable revision.
#[derive(Debug, Clone)]
pub struct Catalog {
    /// Canonical directory containing this catalog's files.
    pub root: PathBuf,
    /// Validated pack namespace and title.
    pub manifest: PackManifest,
    /// Clips ordered lexically by metadata path, with unique pack-local IDs.
    pub clips: Vec<Clip>,
    /// Exact Git commit/content digest for cached data; a working-directory marker otherwise.
    pub revision: String,
}

impl Catalog {
    // A single file/object limit also bounds Git output readers.
    const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;

    /// Validate a trusted, quiescent directory without copying or activating it.
    ///
    /// Every JSON file except root `durop.json` and `.git` content is metadata.
    /// Containment, schema, identities and file headers are checked; decoding is
    /// intentionally deferred. Use [`CatalogCache`] to retain independent files.
    ///
    /// # Errors
    /// Returns a focused error for invalid authoring data, escaping paths, file
    /// limits or filesystem failures. The source must not change during loading.
    pub fn load(path: &Path) -> Result<Self> {
        let root = fs::canonicalize(path)?;
        let manifest: PackManifest =
            Self::parse_json(&Self::resolve_path(&root, &root.join("durop.json"))?)?;
        manifest.validate()?;
        let mut files = Vec::new();
        Self::discover_metadata(&root, &root, &mut BTreeSet::new(), &mut files)?;
        files.sort();
        let mut ids = BTreeSet::new();
        let mut clips = Vec::new();
        for file in files {
            let metadata: ClipMetadata = Self::parse_json(&file)?;
            metadata.validate(&file)?;
            if !ids.insert(metadata.id.clone()) {
                return Err(CatalogError::new(format!(
                    "{}: duplicate clip id {}",
                    file.display(),
                    metadata.id
                )));
            }
            clips.push(Clip::load_files(&root, &manifest.pack_id, file, metadata)?);
        }
        Ok(Self {
            root,
            manifest,
            clips,
            revision: "local-working-directory".into(),
        })
    }

    // Identity rules shared by manifest namespaces and clip IDs.
    fn valid_id(s: &str) -> bool {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    }

    // Bound authored-file reads and reject unresolved LFS pointers.
    fn read_file(path: &Path) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take(Self::MAX_FILE_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > Self::MAX_FILE_BYTES {
            return Err(CatalogError::new("file exceeds 256 MiB limit"));
        }
        if bytes.starts_with(b"version https://git-lfs.github.com/spec/v1") {
            return Err(CatalogError::new(format!(
                "{}: unresolved Git LFS pointer; materialize assets first",
                path.display()
            )));
        }
        Ok(bytes)
    }

    // Include the authoring path in serde diagnostics.
    fn parse_json<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
        serde_json::from_slice(&Self::read_file(path)?)
            .map_err(|e| CatalogError::new(format!("{}: {e}", path.display())))
    }

    // Resolve symlinks before opening files; the caller supplies a canonical root.
    fn resolve_path(root: &Path, path: &Path) -> Result<PathBuf> {
        let resolved = fs::canonicalize(path)
            .map_err(|e| CatalogError::new(format!("{}: {e}", path.display())))?;
        if !resolved.starts_with(root) {
            return Err(CatalogError::new(format!(
                "{}: path escapes pack root",
                path.display()
            )));
        }
        Ok(resolved)
    }

    // Bound recursive discovery and reject directory aliases/cycles.
    fn discover_metadata(
        root: &Path,
        dir: &Path,
        visited: &mut BTreeSet<PathBuf>,
        files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if visited.len() >= 10_000
            || dir.strip_prefix(root).unwrap_or(dir).components().count() > 64
        {
            return Err(CatalogError::new(
                "pack directory count/depth limit exceeded",
            ));
        }
        let canonical = Self::resolve_path(root, dir)?;
        if !visited.insert(canonical) {
            return Err(CatalogError::new(format!(
                "{}: directory alias or symlink cycle",
                dir.display()
            )));
        }
        let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            if entry.file_name() == ".git" {
                continue;
            }
            let path = entry.path();
            let target = Self::resolve_path(root, &path)?;
            if target.is_dir() {
                Self::discover_metadata(root, &path, visited, files)?;
            } else if target.is_file()
                && path.extension().is_some_and(|e| e == "json")
                && path != root.join("durop.json")
            {
                if files.len() >= 10_000 {
                    return Err(CatalogError::new("pack exceeds 10000 metadata files"));
                }
                files.push(path);
            }
        }
        Ok(())
    }
}

/// Compatibility entry point for [`Catalog::load`].
///
/// This validates source files in place; use cache preparation for offline,
/// session-pinned copies. Errors and source-quiescence requirements are identical.
pub fn load_local(path: &Path) -> Result<Catalog> {
    Catalog::load(path)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod cache_tests;
