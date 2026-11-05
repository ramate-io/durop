//! Versioned pack data and immutable, session-pinned snapshots.
// Integration with the panel/host is intentionally deferred.

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
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
        if let Some(v) = &metadata.visual
            && !(v.width_fraction.is_finite() && 0.0 < v.width_fraction && v.width_fraction <= 1.0)
        {
            return Err(invalid("widthFraction must be > 0 and <= 1"));
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

const MAX_FILE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_PACK_BYTES: u64 = 1024 * 1024 * 1024;
const CACHE_VERSION: &str = "catalog-v1";
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// HTTPS sources must contain no credentials, query, fragment or percent escapes.
/// Local Git repositories are only accepted through the explicit LocalGit variant.
#[derive(Debug, Clone)]
pub enum PackSource {
    LocalDirectory(PathBuf),
    LocalGit { path: PathBuf, reference: String },
    HttpsGit { url: String, reference: String },
}
/// Cache is application-owned. Do not permit packs to write into this directory.
/// Call prepare on a worker; pin() gives a session its own immutable Arc.
pub struct CatalogCache {
    root: PathBuf,
    active: Option<Arc<Catalog>>,
}
impl CatalogCache {
    pub fn new(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(&root)?;
        Ok(Self {
            root: fs::canonicalize(root)?,
            active: None,
        })
    }
    pub fn pin(&self) -> Option<Arc<Catalog>> {
        self.active.clone()
    }
    /// No active state changes on error. Existing pins retain their revision/assets.
    pub fn refresh(&mut self, source: &PackSource) -> Result<Arc<Catalog>> {
        let prepared = self.prepare(source)?;
        self.active = Some(prepared.clone());
        Ok(prepared)
    }
    /// Prepare offline files without activating a new session revision.
    pub fn prepare(&self, source: &PackSource) -> Result<Arc<Catalog>> {
        let staging = self.root.join(format!(
            ".staging-{}-{}",
            std::process::id(),
            STAGING_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&staging)?;
        let result = self.prepare_in(source, &staging);
        let _ = fs::remove_dir_all(&staging);
        result
    }
    fn prepare_in(&self, source: &PackSource, staging: &Path) -> Result<Arc<Catalog>> {
        let data = staging.join("pack");
        fs::create_dir(&data)?;
        let (identity, revision) = match source {
            PackSource::LocalDirectory(path) => {
                let source_root = fs::canonicalize(path)?;
                if self.root.starts_with(&source_root) {
                    return Err(invalid("cache directory must be outside the source pack"));
                }
                let original = load_local(path)?;
                let mut files = BTreeSet::from([PathBuf::from("durop.json")]);
                for clip in &original.clips {
                    files.insert(
                        clip.metadata_path
                            .strip_prefix(&original.root)
                            .unwrap()
                            .into(),
                    );
                    // Preserve lexical asset locations even when a contained symlink resolves elsewhere.
                    files.insert(
                        clip.metadata_path
                            .with_extension("png")
                            .strip_prefix(&original.root)
                            .unwrap()
                            .into(),
                    );
                    files.insert(
                        clip.metadata_path
                            .parent()
                            .unwrap()
                            .join(&clip.metadata.asset)
                            .strip_prefix(&original.root)
                            .unwrap()
                            .into(),
                    );
                }
                let mut digest = Sha256::new();
                let mut total = 0;
                for relative in files {
                    let bytes = read(&contained(&original.root, &original.root.join(&relative))?)?;
                    total += bytes.len() as u64;
                    if total > MAX_PACK_BYTES {
                        return Err(invalid("pack exceeds 1 GiB limit"));
                    }
                    let name = relative
                        .to_str()
                        .ok_or_else(|| invalid("non-UTF-8 pack path"))?;
                    digest.update((name.len() as u64).to_be_bytes());
                    digest.update(name.as_bytes());
                    digest.update((bytes.len() as u64).to_be_bytes());
                    digest.update(&bytes);
                    let destination = data.join(relative);
                    fs::create_dir_all(destination.parent().unwrap())?;
                    fs::write(destination, bytes)?;
                }
                (
                    format!("local:{}", original.root.display()),
                    format!("content-{:x}", digest.finalize()),
                )
            }
            PackSource::LocalGit { path, reference } => {
                let path = fs::canonicalize(path)?;
                let revision = export_git(staging, &data, path.as_os_str(), reference, true)?;
                (format!("git-local:{}", path.display()), revision)
            }
            PackSource::HttpsGit { url, reference } => {
                validate_https(url)?;
                let revision =
                    export_git(staging, &data, std::ffi::OsStr::new(url), reference, false)?;
                (format!("git:{url}"), revision)
            }
        };
        // Validate before publication. A pack cannot partially replace an existing snapshot.
        load_local(&data)?;
        let mut digest = Sha256::new();
        for part in [CACHE_VERSION, &identity, &revision] {
            digest.update((part.len() as u64).to_be_bytes());
            digest.update(part.as_bytes());
        }
        let destination = self.root.join(format!("{:x}", digest.finalize()));
        if !destination.exists() {
            match fs::rename(&data, &destination) {
                Ok(()) => (),
                Err(_) if destination.is_dir() => (),
                Err(e) => return Err(e.into()),
            }
        }
        let mut catalog = load_local(&destination)?;
        catalog.revision = revision;
        Ok(Arc::new(catalog))
    }
}
fn validate_https(url: &str) -> Result<()> {
    let rest = url
        .strip_prefix("https://")
        .ok_or_else(|| invalid("Git source must be HTTPS"))?;
    let (host, path) = rest
        .split_once('/')
        .ok_or_else(|| invalid("HTTPS source requires a host and repository path"))?;
    if host.is_empty()
        || path.is_empty()
        || !host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b':')
        || !path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/-_.~".contains(&b))
    {
        return Err(invalid(
            "invalid HTTPS source: credentials, query strings, fragments and escapes are not accepted",
        ));
    }
    Ok(())
}
fn usable_git_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        true
    }
}
fn find_git(search: &std::ffi::OsStr) -> Result<PathBuf> {
    std::env::split_paths(search)
        .map(|directory| directory.join("git"))
        .find(|path| usable_git_file(path))
        .ok_or_else(|| invalid("Git refresh requires a usable Git installation on PATH. Local, bundled and already cached packs do not require Git; no developer tools are installed automatically."))
}
#[cfg(target_os = "macos")]
fn selected_developer_git(developer: &Path) -> Result<PathBuf> {
    let git = developer.join("usr/bin/git");
    if usable_git_file(&git)
        && fs::canonicalize(&git).ok().as_deref() != Some(Path::new("/usr/bin/git"))
    {
        Ok(git)
    } else {
        Err(invalid(
            "Git refresh is unavailable: Apple's Git launcher has no usable selected Git. Install or configure a standalone Git on PATH, or use a local/bundled/cached pack. Durop does not install Command Line Tools.",
        ))
    }
}
fn git_executable() -> Result<PathBuf> {
    let search = std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into());
    let git = find_git(&search)?;
    #[cfg(target_os = "macos")]
    if fs::canonicalize(&git).ok().as_deref() == Some(Path::new("/usr/bin/git")) {
        // /usr/bin/git is a launcher that can display a developer-tools install
        // prompt on a clean Mac. Check selected tools without invoking that stub.
        let mut select = Command::new("/usr/bin/xcode-select");
        select.arg("-p");
        let bytes = run_git_command(select, std::time::Duration::from_secs(2))
            .map_err(|_| invalid("Git refresh requires usable external Git. Apple's Git launcher has no selected developer tools; Durop will not invoke an installation prompt. Use a local/bundled/cached pack, or configure standalone Git on PATH."))?;
        let directory = String::from_utf8(bytes)
            .map_err(|_| invalid("invalid selected developer-tools path"))?;
        return selected_developer_git(Path::new(directory.trim()));
    }
    Ok(git)
}
/// Bounded output and wall time; errors deliberately omit Git stderr/URLs to avoid credential leaks.
fn git(repo: &Path, args: &[&std::ffi::OsStr], local: bool) -> Result<Vec<u8>> {
    let mut command = Command::new(git_executable()?);
    command
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "protocol.allow=never",
            "-c",
            "protocol.https.allow=always",
            "-c",
            if local {
                "protocol.file.allow=always"
            } else {
                "protocol.file.allow=never"
            },
            "-c",
            "fetch.recurseSubmodules=false",
            "-c",
            "credential.interactive=false",
        ])
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_LFS_SKIP_SMUDGE", "1");
    run_git_command(command, std::time::Duration::from_secs(120)).map_err(|error| invalid(format!("Git refresh could not complete; check that Git is usable and the source/ref is available. Local/bundled/cached packs remain usable. {error}")))
}
fn run_git_command(mut command: Command, timeout: std::time::Duration) -> Result<Vec<u8>> {
    use std::{
        process::Stdio,
        thread,
        time::{Duration, Instant},
    };
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child.stdout.take().unwrap();
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout
            .take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = sender.send(result);
    });
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > timeout {
            stop_git(&mut child);
            return Err(invalid("Git operation timed out"));
        }
        thread::sleep(Duration::from_millis(10));
    };
    let remaining = timeout.saturating_sub(start.elapsed());
    let bytes = match receiver.recv_timeout(remaining) {
        Ok(result) => result?,
        Err(_) => {
            stop_git(&mut child);
            return Err(invalid("Git output timed out"));
        }
    };
    if !status.success() {
        return Err(invalid(
            "Git operation failed (source/ref unavailable or rejected); diagnostics suppressed to protect credentials",
        ));
    }
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(invalid("Git object/listing exceeds 256 MiB limit"));
    }
    Ok(bytes)
}
// Git fetch may spawn remote helpers. Kill its isolated group on Unix, including
// when the top-level process exited but a helper retained its output pipe.
fn stop_git(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        unsafe extern "C" {
            fn kill(pid: std::os::raw::c_int, signal: std::os::raw::c_int) -> std::os::raw::c_int;
        }
        if let Ok(pid) = i32::try_from(child.id()) {
            // SAFETY: Git was spawned in its own process group whose ID is its PID.
            // SIGKILL is 9 on the supported Linux/macOS targets. No pointers cross FFI.
            unsafe {
                kill(-pid, 9);
            }
        }
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn export_git(
    staging: &Path,
    data: &Path,
    source: &std::ffi::OsStr,
    reference: &str,
    local: bool,
) -> Result<String> {
    use std::ffi::OsStr;
    if reference.is_empty()
        || reference.starts_with('-')
        || !reference
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
    {
        return Err(invalid(
            "invalid Git reference; select a branch, tag or commit",
        ));
    }
    let repo = staging.join("objects");
    fs::create_dir(&repo)?;
    git(
        &repo,
        &[
            OsStr::new("init"),
            OsStr::new("--bare"),
            OsStr::new("--template=/dev/null"),
        ],
        local,
    )?;
    git(
        &repo,
        &[
            OsStr::new("fetch"),
            OsStr::new("--depth=1"),
            OsStr::new("--no-tags"),
            OsStr::new("--no-recurse-submodules"),
            OsStr::new("--"),
            source,
            OsStr::new(reference),
        ],
        local,
    )?;
    let revision = String::from_utf8(git(
        &repo,
        &[
            OsStr::new("rev-parse"),
            OsStr::new("--verify"),
            OsStr::new("FETCH_HEAD^{commit}"),
        ],
        local,
    )?)
    .map_err(|_| invalid("invalid Git revision"))?
    .trim()
    .to_owned();
    if ![40, 64].contains(&revision.len()) || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(invalid("Git did not resolve an immutable commit"));
    }
    let listing = git(
        &repo,
        &[
            OsStr::new("ls-tree"),
            OsStr::new("-r"),
            OsStr::new("-z"),
            OsStr::new(&revision),
        ],
        local,
    )?;
    let mut total = 0u64;
    for entry in listing.split(|b| *b == 0).filter(|x| !x.is_empty()) {
        let entry = std::str::from_utf8(entry).map_err(|_| invalid("Git paths must be UTF-8"))?;
        let (header, name) = entry
            .split_once('\t')
            .ok_or_else(|| invalid("invalid Git tree"))?;
        let fields: Vec<_> = header.split(' ').collect();
        if fields.len() != 3 || !["100644", "100755"].contains(&fields[0]) || fields[1] != "blob" {
            return Err(invalid("Git packs cannot contain symlinks or submodules"));
        }
        let path = Path::new(name);
        if path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
            || name.contains('\\')
            || path.components().any(|c| c.as_os_str() == ".git")
        {
            return Err(invalid("unsafe Git tree path"));
        }
        let bytes = git(
            &repo,
            &[
                OsStr::new("cat-file"),
                OsStr::new("blob"),
                OsStr::new(fields[2]),
            ],
            local,
        )?;
        total += bytes.len() as u64;
        if total > MAX_PACK_BYTES {
            return Err(invalid("pack exceeds 1 GiB limit"));
        }
        let destination = data.join(path);
        fs::create_dir_all(destination.parent().unwrap())?;
        fs::write(destination, bytes)?;
    }
    Ok(revision)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/sample-pack");
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
    fn defaults_format_policy_and_deterministic_order() {
        let t = fixture();
        let loaded = load_local(&t.0).unwrap();
        assert_eq!(loaded.clips[0].metadata.playback.max_duration_ms, 1000);
        let path = t.0.join("reactions/chime.json");
        let mut meta: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        meta.as_object_mut().unwrap().remove("playback");
        fs::write(&path, serde_json::to_vec(&meta).unwrap()).unwrap();
        assert_eq!(
            load_local(&t.0).unwrap().clips[0]
                .metadata
                .playback
                .max_duration_ms,
            10000
        );
        fs::write(t.0.join("reactions/thing.exe"), b"data").unwrap();
        meta["asset"] = serde_json::json!("thing.exe");
        fs::write(&path, serde_json::to_vec(&meta).unwrap()).unwrap();
        assert!(
            load_local(&t.0)
                .unwrap_err()
                .0
                .contains("unsupported media")
        );
        meta["asset"] = serde_json::json!("chime.wav");
        meta["id"] = serde_json::json!("another");
        fs::write(
            t.0.join("reactions/aaa.json"),
            serde_json::to_vec(&meta).unwrap(),
        )
        .unwrap();
        fs::copy(
            t.0.join("reactions/chime.png"),
            t.0.join("reactions/aaa.png"),
        )
        .unwrap();
        meta["id"] = serde_json::json!("chime");
        fs::write(&path, serde_json::to_vec(&meta).unwrap()).unwrap();
        let loaded = load_local(&t.0).unwrap();
        assert_eq!(
            loaded
                .clips
                .iter()
                .map(|c| c.identity.as_str())
                .collect::<Vec<_>>(),
            vec!["sample/another", "sample/chime"]
        );
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

#[cfg(test)]
mod cache_tests {
    use super::tests::{Temp, fixture};
    use super::*;
    fn repository() -> Temp {
        let t = fixture();
        let status = Command::new("git")
            .args(["-C"])
            .arg(&t.0)
            .args(["init", "--template=/dev/null"])
            .output()
            .unwrap()
            .status;
        assert!(status.success());
        commit(&t.0);
        t
    }
    fn commit(path: &Path) {
        for args in [
            vec!["add", "--all"],
            vec![
                "-c",
                "user.name=Catalog Test",
                "-c",
                "user.email=catalog@example.invalid",
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "-m",
                "Fixture",
            ],
        ] {
            assert!(
                Command::new("git")
                    .arg("-C")
                    .arg(path)
                    .args(args)
                    .output()
                    .unwrap()
                    .status
                    .success()
            );
        }
    }
    #[test]
    fn local_snapshot_survives_source_deletion_and_failed_refresh() {
        let t = fixture();
        let cache_dir = Temp::new();
        let mut cache = CatalogCache::new(cache_dir.0.clone()).unwrap();
        let first = cache
            .refresh(&PackSource::LocalDirectory(t.0.clone()))
            .unwrap();
        let pin = cache.pin().unwrap();
        fs::write(t.0.join("durop.json"), b"bad").unwrap();
        assert!(
            cache
                .refresh(&PackSource::LocalDirectory(t.0.clone()))
                .is_err()
        );
        assert!(Arc::ptr_eq(&cache.pin().unwrap(), &first));
        drop(t);
        assert!(pin.clips[0].asset_path.is_file());
        assert!(read(&pin.clips[0].asset_path).is_ok());
    }
    #[test]
    fn git_exact_commits_pin_updates_without_filters_or_hooks() {
        let repo = repository();
        let cache_dir = Temp::new();
        let mut cache = CatalogCache::new(cache_dir.0.clone()).unwrap();
        // A filter that would fail if any worktree checkout/smudge were attempted.
        fs::write(repo.0.join(".gitattributes"), "*.wav filter=forbidden\n").unwrap();
        commit(&repo.0);
        let source = PackSource::LocalGit {
            path: repo.0.clone(),
            reference: "HEAD".into(),
        };
        let first = cache.refresh(&source).unwrap();
        let pin = cache.pin().unwrap();
        assert_eq!(
            first.clips[0].identity,
            load_local(&repo.0).unwrap().clips[0].identity
        );
        assert_eq!(first.revision.len(), 40);
        assert_eq!(
            read(&first.clips[0].asset_path).unwrap(),
            read(&repo.0.join("reactions/chime.wav")).unwrap()
        );
        let mut meta: serde_json::Value =
            serde_json::from_slice(&fs::read(repo.0.join("reactions/chime.json")).unwrap())
                .unwrap();
        meta["title"] = serde_json::json!("New title");
        fs::write(
            repo.0.join("reactions/chime.json"),
            serde_json::to_vec(&meta).unwrap(),
        )
        .unwrap();
        commit(&repo.0);
        let prepared = cache.prepare(&source).unwrap();
        assert_ne!(prepared.revision, first.revision);
        assert!(Arc::ptr_eq(&cache.pin().unwrap(), &first));
        cache.refresh(&source).unwrap();
        assert_eq!(pin.clips[0].metadata.title, "Chime");
        assert_eq!(cache.pin().unwrap().clips[0].metadata.title, "New title");
        fs::write(
            repo.0.join("reactions/chime.wav"),
            b"version https://git-lfs.github.com/spec/v1\n",
        )
        .unwrap();
        commit(&repo.0);
        assert!(cache.refresh(&source).is_err());
        assert_eq!(cache.pin().unwrap().revision, prepared.revision);
        drop(repo);
        assert!(read(&pin.clips[0].asset_path).is_ok());
    }
    #[cfg(unix)]
    #[test]
    fn local_contained_symlinks_are_copied_as_independent_files() {
        let t = fixture();
        fs::rename(t.0.join("reactions/chime.wav"), t.0.join("original.wav")).unwrap();
        std::os::unix::fs::symlink("../original.wav", t.0.join("reactions/chime.wav")).unwrap();
        let dir = Temp::new();
        let cache = CatalogCache::new(dir.0.clone()).unwrap();
        let first = cache
            .prepare(&PackSource::LocalDirectory(t.0.clone()))
            .unwrap();
        assert!(
            !fs::symlink_metadata(&first.clips[0].asset_path)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        fs::write(t.0.join("original.wav"), b"changed source data").unwrap();
        let next = cache
            .prepare(&PackSource::LocalDirectory(t.0.clone()))
            .unwrap();
        assert_ne!(first.revision, next.revision);
        assert_ne!(
            read(&first.clips[0].asset_path).unwrap(),
            read(&next.clips[0].asset_path).unwrap()
        );
    }
    #[cfg(unix)]
    #[test]
    fn timeout_bounds_child_and_inherited_stdout() {
        for script in ["sleep 30", "sleep 30 &"] {
            let mut command = Command::new("sh");
            command.args(["-c", script]);
            let start = std::time::Instant::now();
            assert!(
                run_git_command(command, std::time::Duration::from_millis(100))
                    .unwrap_err()
                    .0
                    .contains("timed out")
            );
            assert!(start.elapsed() < std::time::Duration::from_secs(2));
        }
    }
    #[test]
    fn hostile_urls_and_refs_are_rejected() {
        for url in [
            "file:///tmp/repo",
            "https://token@example.org/a",
            "https://example.org/a?token=x",
            "https://example.org/a#x",
            "https://example.org/%2e%2e/a",
            "ext::command",
        ] {
            assert!(validate_https(url).is_err());
        }
        assert!(validate_https("https://example.org/org/pack.git").is_ok());
        let dir = Temp::new();
        let cache = CatalogCache::new(dir.0.clone()).unwrap();
        assert!(
            cache
                .prepare(&PackSource::LocalGit {
                    path: dir.0.clone(),
                    reference: "--upload-pack=bad".into()
                })
                .is_err()
        );
    }
    #[cfg(unix)]
    #[test]
    fn missing_git_is_actionable_without_executing_candidates() {
        use std::os::unix::fs::PermissionsExt;
        let directory = Temp::new();
        let search = std::env::join_paths([&directory.0]).unwrap();
        assert!(find_git(&search).unwrap_err().0.contains("Local, bundled"));
        let candidate = directory.0.join("git");
        fs::write(&candidate, b"not an executable").unwrap();
        fs::set_permissions(&candidate, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(find_git(&search).is_err());
        fs::set_permissions(&candidate, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(find_git(&search).unwrap(), candidate);
        // Detection checks readiness without launching an arbitrary file or the
        // Apple installation stub. Actual Git errors remain bounded/suppressed.
    }
    #[cfg(target_os = "macos")]
    #[test]
    fn absent_selected_apple_git_does_not_request_installation() {
        let directory = Temp::new();
        let error = selected_developer_git(&directory.0).unwrap_err();
        assert!(error.0.contains("does not install Command Line Tools"));
    }
    #[cfg(unix)]
    #[test]
    fn git_symlinks_rejected_without_following() {
        let repo = repository();
        std::os::unix::fs::symlink("chime.wav", repo.0.join("reactions/link.wav")).unwrap();
        commit(&repo.0);
        let dir = Temp::new();
        let cache = CatalogCache::new(dir.0.clone()).unwrap();
        assert!(
            cache
                .prepare(&PackSource::LocalGit {
                    path: repo.0.clone(),
                    reference: "HEAD".into()
                })
                .unwrap_err()
                .0
                .contains("symlinks")
        );
    }

    #[test]
    #[ignore = "network acceptance; run scripts/catalog-https-acceptance.py"]
    fn https_authenticated_cache_acceptance() {
        let url = std::env::var("DUROP_TEST_HTTPS_URL").unwrap();
        let source_path = PathBuf::from(std::env::var_os("DUROP_TEST_REPOSITORY").unwrap());
        let source = PackSource::HttpsGit {
            url,
            reference: "HEAD".into(),
        };
        let dir = Temp::new();
        let mut cache = CatalogCache::new(dir.0.clone()).unwrap();
        let first = cache.refresh(&source).unwrap();
        let pin = cache.pin().unwrap();
        let local = load_local(&source_path).unwrap();
        let head = || {
            let output = Command::new("git")
                .arg("-C")
                .arg(&source_path)
                .args(["rev-parse", "HEAD"])
                .output()
                .unwrap();
            assert!(output.status.success());
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        };
        assert_eq!(first.revision, head());
        assert_eq!(first.clips[0].identity, local.clips[0].identity);
        assert_eq!(
            first.clips[0].decoder_readiness,
            DecoderReadiness::Unchecked
        );
        let media = read(&first.clips[0].asset_path).unwrap();
        assert_eq!(media, read(&local.clips[0].asset_path).unwrap());
        assert_eq!(
            read(&first.clips[0].icon_path).unwrap(),
            read(&local.clips[0].icon_path).unwrap()
        );
        let metadata_path = source_path.join("reactions/chime.json");
        let mut metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(&metadata_path).unwrap()).unwrap();
        metadata["title"] = serde_json::json!("HTTPS update");
        fs::write(metadata_path, serde_json::to_vec(&metadata).unwrap()).unwrap();
        commit(&source_path);
        let next = cache.prepare(&source).unwrap();
        assert_eq!(next.revision, head());
        assert_ne!(next.revision, first.revision);
        assert!(Arc::ptr_eq(&cache.pin().unwrap(), &first));
        cache.refresh(&source).unwrap();
        assert_eq!(pin.clips[0].metadata.title, "Chime");
        assert_eq!(cache.pin().unwrap().clips[0].metadata.title, "HTTPS update");
        fs::write(source_path.join("durop.json"), b"invalid").unwrap();
        commit(&source_path);
        assert!(cache.refresh(&source).is_err());
        assert_eq!(cache.pin().unwrap().revision, next.revision);
        // Make the served repository unavailable; cached bytes still load offline.
        fs::rename(source_path.join(".git"), source_path.join("offline.git")).unwrap();
        assert!(cache.refresh(&source).is_err());
        assert_eq!(cache.pin().unwrap().revision, next.revision);
        assert_eq!(read(&pin.clips[0].asset_path).unwrap(), media);
        assert_eq!(
            load_local(&pin.root).unwrap().clips[0].identity,
            local.clips[0].identity
        );
    }

    #[test]
    #[ignore = "requires public GitHub HTTPS access"]
    fn https_public_export_acceptance() {
        let staging = Temp::new();
        let data = staging.0.join("pack");
        fs::create_dir(&data).unwrap();
        let expected = "79a70e2421c27350b8e8545a9e50f702b86552fc";
        let revision = export_git(
            &staging.0,
            &data,
            std::ffi::OsStr::new("https://github.com/ramate-io/durop.git"),
            expected,
            false,
        )
        .unwrap();
        assert_eq!(revision, expected);
        let catalog = load_local(&data.join("../../examples/sample-pack")).unwrap();
        assert_eq!(catalog.clips[0].identity, "sample/chime");
        assert_eq!(
            catalog.clips[0].decoder_readiness,
            DecoderReadiness::Unchecked
        );
    }
}
