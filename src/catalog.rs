//! Versioned pack data and immutable, session-pinned snapshots.
// Integration with the panel/host is intentionally deferred.

use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

type Result<T> = std::result::Result<T, CatalogError>;
#[derive(Debug)]
pub struct CatalogError(pub String);
impl std::fmt::Display for CatalogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for CatalogError {}
impl From<std::io::Error> for CatalogError {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}
fn invalid(message: impl Into<String>) -> CatalogError {
    CatalogError(message.into())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackManifest {
    pub schema_version: u32,
    pub pack_id: String,
    pub title: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipMetadata {
    pub id: String,
    pub title: String,
    pub asset: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub playback: Playback,
    pub visual: Option<Visual>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Playback {
    #[serde(default)]
    pub retrigger: Retrigger,
    #[serde(default, rename = "loop")]
    pub looping: bool,
    #[serde(default = "default_duration_ms")]
    pub max_duration_ms: u64,
}
fn default_duration_ms() -> u64 {
    10_000
}
impl Default for Playback {
    fn default() -> Self {
        Self {
            retrigger: Retrigger::default(),
            looping: false,
            max_duration_ms: default_duration_ms(),
        }
    }
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Retrigger {
    #[default]
    Restart,
    IgnoreIfPlaying,
    Stack,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Visual {
    pub anchor: Anchor,
    pub width_fraction: f64,
    pub fit: Fit,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fit {
    Contain,
    Cover,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecoderReadiness {
    Unchecked,
}
#[derive(Debug, Clone)]
pub struct Clip {
    pub identity: String,
    pub category: PathBuf,
    pub metadata: ClipMetadata,
    pub metadata_path: PathBuf,
    pub icon_path: PathBuf,
    pub asset_path: PathBuf,
    pub decoder_readiness: DecoderReadiness,
}
#[derive(Debug, Clone)]
pub struct Catalog {
    pub root: PathBuf,
    pub manifest: PackManifest,
    pub clips: Vec<Clip>,
    pub revision: String,
}
fn id_valid(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn read(path: &Path) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(invalid("file exceeds 256 MiB limit"));
    }
    if bytes.starts_with(b"version https://git-lfs.github.com/spec/v1") {
        return Err(invalid(format!(
            "{}: unresolved Git LFS pointer; materialize assets first",
            path.display()
        )));
    }
    Ok(bytes)
}
fn json<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    serde_json::from_slice(&read(path)?).map_err(|e| invalid(format!("{}: {e}", path.display())))
}
fn contained(root: &Path, path: &Path) -> Result<PathBuf> {
    let resolved =
        fs::canonicalize(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    if !resolved.starts_with(root) {
        return Err(invalid(format!(
            "{}: path escapes pack root",
            path.display()
        )));
    }
    Ok(resolved)
}
fn discover(
    root: &Path,
    dir: &Path,
    visited: &mut BTreeSet<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if visited.len() >= 10_000 || dir.strip_prefix(root).unwrap_or(dir).components().count() > 64 {
        return Err(invalid("pack directory count/depth limit exceeded"));
    }
    let canonical = contained(root, dir)?;
    if !visited.insert(canonical) {
        return Err(invalid(format!(
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
        let target = contained(root, &path)?;
        if target.is_dir() {
            discover(root, &path, visited, files)?;
        } else if target.is_file()
            && path.extension().is_some_and(|e| e == "json")
            && path != root.join("durop.json")
        {
            if files.len() >= 10_000 {
                return Err(invalid("pack exceeds 10000 metadata files"));
            }
            files.push(path);
        }
    }
    Ok(())
}
/// Load a trusted, quiescent directory. Cache preparation copies its validated files.
/// JSON discovery is recursive, excluding .git; ordering is lexical metadata path.
pub fn load_local(path: &Path) -> Result<Catalog> {
    let root = fs::canonicalize(path)?;
    let manifest: PackManifest = json(&contained(&root, &root.join("durop.json"))?)?;
    if manifest.schema_version != 1 {
        return Err(invalid("unsupported schemaVersion; expected 1"));
    }
    if !id_valid(&manifest.pack_id) || manifest.title.trim().is_empty() {
        return Err(invalid(
            "packId must use ASCII letters, digits, '-' or '_'; title must be nonempty",
        ));
    }
    let mut files = Vec::new();
    discover(&root, &root, &mut BTreeSet::new(), &mut files)?;
    files.sort();
    let mut ids = BTreeSet::new();
    let mut clips = Vec::new();
    for file in files {
        let metadata: ClipMetadata = json(&file)?;
        if !id_valid(&metadata.id) || metadata.title.trim().is_empty() {
            return Err(invalid(format!(
                "{}: invalid id or empty title",
                file.display()
            )));
        }
        if !ids.insert(metadata.id.clone()) {
            return Err(invalid(format!(
                "{}: duplicate clip id {}",
                file.display(),
                metadata.id
            )));
        }
        if metadata.playback.max_duration_ms == 0 || metadata.playback.max_duration_ms > 600_000 {
            return Err(invalid("maxDurationMs must be positive and at most 600000"));
        }
        if let Some(v) = &metadata.visual {
            if !v.width_fraction.is_finite() || !(0.0 < v.width_fraction && v.width_fraction <= 1.0)
            {
                return Err(invalid("widthFraction must be > 0 and <= 1"));
            }
        }
        let asset = Path::new(&metadata.asset);
        if asset.as_os_str().is_empty()
            || asset
                .components()
                .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
            || metadata.asset.contains('\\')
        {
            return Err(invalid(format!(
                "{}: asset must be a relative path without traversal",
                file.display()
            )));
        }
        let asset_path = contained(&root, &file.parent().unwrap().join(asset))?;
        let extension = asset
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ![
            "wav", "mp3", "ogg", "flac", "m4a", "aac", "mp4", "mov", "webm", "gif",
        ]
        .contains(&extension.as_str())
        {
            return Err(invalid(format!(
                "{}: unsupported media extension",
                file.display()
            )));
        }
        if !asset_path.is_file() {
            return Err(invalid("asset must be a regular file"));
        }
        let data = read(&asset_path)?;
        if data.is_empty() {
            return Err(invalid("media asset is empty"));
        }
        let icon_path = contained(&root, &file.with_extension("png")).map_err(|e| {
            invalid(format!(
                "{}: requires same-basename PNG icon: {e}",
                file.display()
            ))
        })?;
        let png = read(&icon_path)?;
        // Structural header checks only; decoder validation belongs to media preparation.
        if png.len() < 33
            || &png[..8] != b"\x89PNG\r\n\x1a\n"
            || png[8..12] != [0, 0, 0, 13]
            || &png[12..16] != b"IHDR"
            || png[16..20] == [0; 4]
            || png[20..24] == [0; 4]
        {
            return Err(invalid(format!(
                "{}: invalid PNG header",
                icon_path.display()
            )));
        }
        let relative = file.strip_prefix(&root).unwrap();
        clips.push(Clip {
            identity: format!("{}/{}", manifest.pack_id, metadata.id),
            category: relative.parent().unwrap().into(),
            metadata,
            metadata_path: file,
            icon_path,
            asset_path,
            decoder_readiness: DecoderReadiness::Unchecked,
        });
    }
    Ok(Catalog {
        root,
        manifest,
        clips,
        revision: "local-working-directory".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    pub(super) struct Temp(pub(super) PathBuf);
    impl Temp {
        pub(super) fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "durop-catalog-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    pub(super) fn fixture() -> Temp {
        let t = Temp::new();
        let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sample-pack");
        fs::create_dir(t.0.join("reactions")).unwrap();
        for file in [
            "durop.json",
            "reactions/chime.json",
            "reactions/chime.png",
            "reactions/chime.wav",
        ] {
            fs::copy(sample.join(file), t.0.join(file)).unwrap();
        }
        t
    }
    #[test]
    fn identity_survives_move() {
        let t = fixture();
        let a = load_local(&t.0).unwrap();
        fs::rename(t.0.join("reactions"), t.0.join("other")).unwrap();
        let b = load_local(&t.0).unwrap();
        assert_eq!(a.clips[0].identity, b.clips[0].identity);
        assert_eq!(b.clips[0].decoder_readiness, DecoderReadiness::Unchecked);
    }
    #[test]
    fn actionable_missing_icon_and_asset() {
        let t = fixture();
        fs::rename(
            t.0.join("reactions/chime.json"),
            t.0.join("reactions/new.json"),
        )
        .unwrap();
        assert!(
            load_local(&t.0)
                .unwrap_err()
                .0
                .contains("same-basename PNG")
        );
        fs::remove_file(t.0.join("reactions/chime.wav")).unwrap();
        assert!(load_local(&t.0).unwrap_err().0.contains("chime.wav"));
    }
    #[test]
    fn duplicate_bad_json_version_ranges_and_traversal() {
        let t = fixture();
        let file = t.0.join("reactions/chime.json");
        let original = fs::read(&file).unwrap();
        fs::copy(&file, t.0.join("reactions/duplicate.json")).unwrap();
        assert!(load_local(&t.0).unwrap_err().0.contains("duplicate"));
        fs::remove_file(t.0.join("reactions/duplicate.json")).unwrap();
        for (field, value, expected) in [
            ("asset", serde_json::json!("../../escape.wav"), "traversal"),
            ("asset", serde_json::json!("thing.exe"), "thing.exe"),
            (
                "playback",
                serde_json::json!({"maxDurationMs":0}),
                "positive",
            ),
            (
                "visual",
                serde_json::json!({"anchor":"center","widthFraction":1.1,"fit":"contain"}),
                "widthFraction",
            ),
        ] {
            let mut data: serde_json::Value = serde_json::from_slice(&original).unwrap();
            data[field] = value;
            fs::write(&file, serde_json::to_vec(&data).unwrap()).unwrap();
            assert!(load_local(&t.0).unwrap_err().0.contains(expected));
        }
        fs::write(&file, b"{").unwrap();
        assert!(load_local(&t.0).unwrap_err().0.contains("chime.json"));
        fs::write(&file, original).unwrap();
        fs::write(
            t.0.join("durop.json"),
            br#"{"schemaVersion":2,"packId":"sample","title":"X"}"#,
        )
        .unwrap();
        assert!(load_local(&t.0).unwrap_err().0.contains("schemaVersion"));
    }
    #[test]
    fn lfs_rejected() {
        let t = fixture();
        fs::write(
            t.0.join("reactions/chime.wav"),
            b"version https://git-lfs.github.com/spec/v1\noid sha256:abc\nsize 10\n",
        )
        .unwrap();
        assert!(load_local(&t.0).unwrap_err().0.contains("LFS"));
    }
    #[cfg(unix)]
    #[test]
    fn symlink_escape_and_cycle_rejected() {
        use std::os::unix::fs::symlink;
        let t = fixture();
        let outside = Temp::new();
        fs::write(outside.0.join("asset.wav"), b"data").unwrap();
        fs::remove_file(t.0.join("reactions/chime.wav")).unwrap();
        symlink(outside.0.join("asset.wav"), t.0.join("reactions/chime.wav")).unwrap();
        assert!(load_local(&t.0).unwrap_err().0.contains("escapes"));
        fs::remove_file(t.0.join("reactions/chime.wav")).unwrap();
        symlink(&t.0, t.0.join("reactions/cycle")).unwrap();
        assert!(load_local(&t.0).unwrap_err().0.contains("cycle"));
    }
}

const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;
