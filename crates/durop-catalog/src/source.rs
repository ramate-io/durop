//! Acquisition policies: copy local files or export Git blobs without checkout.

use super::*;
use sha2::{Digest, Sha256};
use std::{ffi::OsStr, process::Command};

// Bound total acquisition before writing an oversized snapshot to disk.
pub(super) const MAX_PACK_BYTES: u64 = 1024 * 1024 * 1024;

/// Stable cache identity and immutable revision of a materialized source.
///
/// Both fields participate in the cache key with a derived-format version.
/// Implementations must change the revision whenever materialized bytes or paths
/// change. Values must not contain credentials; Git transport diagnostics are
/// intentionally excluded.
///
/// Construction does not prove these claims: the cache validates materialized
/// file contents separately. Trusted source implementations are responsible for
/// computing a faithful identity/revision, rather than accepting one from a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRevision {
    /// Credential-free source identity; distinct sources may share revisions.
    pub identity: String,
    /// Exact commit or content digest identifying the copied snapshot.
    pub revision: String,
}

/// Materialize a pack independently of cache publication or session activation.
///
/// The cache supplies distinct empty, application-owned `pack_root` and
/// `scratch_root` directories, removed when the operation finishes. Write pack
/// files only under `pack_root`; temporary repository state belongs in scratch.
/// The result must use independent regular files, not links back to mutable
/// source data. The cache checks containment, file/pack bounds and catalog
/// validity before publication; implementations must also bound acquisition
/// itself. No repository scripts, hooks or media decoders may execute.
///
/// This synchronous operation may perform filesystem/network I/O and belongs on
/// a worker. Returning an error leaves active catalogs and their pins unchanged.
/// Implementations are trusted application code, not an extension interface for
/// executing pack-provided code; filesystem checks do not sandbox implementors.
pub trait CatalogSource: Send + Sync {
    /// Copy/export source bytes and return their stable cache identity/revision.
    ///
    /// # Errors
    /// Report unavailable/invalid sources without exposing credentials or remote
    /// stderr. Partially written output is discarded by the cache.
    fn materialize(&self, pack_root: &Path, scratch_root: &Path) -> Result<SourceRevision>;
}

/// A trusted local working directory, copied into independent cached files.
///
/// The directory must remain quiescent during preparation. Only its validated
/// manifest, clip metadata, icons and referenced media contribute to the digest.
#[derive(Debug, Clone)]
pub struct LocalCatalogSource {
    /// Explicitly selected source directory; canonicalized during acquisition.
    pub path: PathBuf,
}

impl LocalCatalogSource {
    /// Select a local directory without reading it until materialization.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    // Keep lexical locations: contained source symlinks are resolved while copying.
    fn required_files(catalog: &Catalog) -> BTreeSet<PathBuf> {
        let mut files = BTreeSet::from([PathBuf::from("durop.json")]);
        for clip in &catalog.clips {
            for path in [
                clip.metadata_path.clone(),
                clip.metadata_path.with_extension("png"),
                clip.metadata_path
                    .parent()
                    .unwrap()
                    .join(&clip.metadata.asset),
            ] {
                files.insert(path.strip_prefix(&catalog.root).unwrap().into());
            }
        }
        files
    }
}

impl CatalogSource for LocalCatalogSource {
    fn materialize(&self, pack_root: &Path, _scratch_root: &Path) -> Result<SourceRevision> {
        let source_root = fs::canonicalize(&self.path)?;
        if fs::canonicalize(pack_root)?.starts_with(&source_root) {
            return Err(CatalogError::new(
                "cache directory must be outside the source pack",
            ));
        }
        let original = Catalog::load(&self.path)?;
        let mut digest = Sha256::new();
        let mut total = 0;
        for relative in Self::required_files(&original) {
            let bytes = Catalog::read_file(&Catalog::resolve_path(
                &original.root,
                &original.root.join(&relative),
            )?)?;
            total += bytes.len() as u64;
            if total > MAX_PACK_BYTES {
                return Err(CatalogError::new("pack exceeds 1 GiB limit"));
            }
            let name = relative
                .to_str()
                .ok_or_else(|| CatalogError::new("non-UTF-8 pack path"))?;
            digest.update((name.len() as u64).to_be_bytes());
            digest.update(name.as_bytes());
            digest.update((bytes.len() as u64).to_be_bytes());
            digest.update(&bytes);
            let destination = pack_root.join(relative);
            fs::create_dir_all(destination.parent().unwrap())?;
            fs::write(destination, bytes)?;
        }
        Ok(SourceRevision {
            identity: format!("local:{}", original.root.display()),
            revision: format!("content-{:x}", digest.finalize()),
        })
    }
}

// Protocol permission is derived from explicit configuration, never a remote URL.
#[derive(Debug, Clone)]
enum GitRepository {
    Local(PathBuf),
    Https(String),
}

/// Git-backed source resolving a literal ref to an immutable commit.
///
/// Exports blobs from a temporary bare repository: no checkout, smudge/filter,
/// LFS fetching, submodule recursion or pack-script execution occurs. Local Git
/// trees cannot contain symlinks/gitlinks. Existing trusted Git credential helpers
/// can service HTTPS requests; URL credentials and Git stderr are never retained.
#[derive(Debug, Clone)]
pub struct GitCatalogSource {
    // Preserve the caller's explicit transport choice through every Git command.
    repository: GitRepository,
    // Only literal refs pass export validation; resolution produces the commit ID.
    reference: String,
}

impl GitCatalogSource {
    /// Select an explicit local Git repository and literal branch/tag/commit.
    pub fn local(path: impl Into<PathBuf>, reference: impl Into<String>) -> Self {
        Self {
            repository: GitRepository::Local(path.into()),
            reference: reference.into(),
        }
    }

    /// Select HTTPS without embedded credentials, query, fragment or percent escapes.
    ///
    /// Validation is deferred until materialization, like filesystem acquisition.
    pub fn https(url: impl Into<String>, reference: impl Into<String>) -> Self {
        Self {
            repository: GitRepository::Https(url.into()),
            reference: reference.into(),
        }
    }

    // Permit only the deliberately supported, credential-free HTTPS URL grammar.
    pub(super) fn validate_https(url: &str) -> Result<()> {
        let rest = url
            .strip_prefix("https://")
            .ok_or_else(|| CatalogError::new("Git source must be HTTPS"))?;
        let (host, path) = rest
            .split_once('/')
            .ok_or_else(|| CatalogError::new("HTTPS source requires a host and repository path"))?;
        if host.is_empty()
            || path.is_empty()
            || !host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b':')
            || !path
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/-_.~".contains(&b))
        {
            return Err(CatalogError::new(
                "invalid HTTPS source: credentials, query strings, fragments and escapes are not accepted",
            ));
        }
        Ok(())
    }
}

impl CatalogSource for GitCatalogSource {
    fn materialize(&self, pack_root: &Path, scratch_root: &Path) -> Result<SourceRevision> {
        let (identity, revision) = match &self.repository {
            GitRepository::Local(path) => {
                let path = fs::canonicalize(path)?;
                let revision = GitRunner::new(true).export(
                    scratch_root,
                    pack_root,
                    path.as_os_str(),
                    &self.reference,
                )?;
                (format!("git-local:{}", path.display()), revision)
            }
            GitRepository::Https(url) => {
                Self::validate_https(url)?;
                let revision = GitRunner::new(false).export(
                    scratch_root,
                    pack_root,
                    OsStr::new(url),
                    &self.reference,
                )?;
                (format!("git:{url}"), revision)
            }
        };
        Ok(SourceRevision { identity, revision })
    }
}

/// Source configuration retained for existing application callers.
///
/// Its trait implementation delegates acquisition to concrete source objects;
/// the cache itself contains no source-kind switch.
#[derive(Debug, Clone)]
pub enum PackSource {
    /// Copy a quiescent local directory, including uncommitted authoring changes.
    LocalDirectory(PathBuf),
    /// Fetch an explicitly selected local Git repository.
    LocalGit {
        /// Repository directory, canonicalized before fetching.
        path: PathBuf,
        /// Literal branch, tag or commit ID; revision expressions are rejected.
        reference: String,
    },
    /// Fetch ordinary Git HTTPS with trusted existing credential helpers.
    HttpsGit {
        /// HTTPS URL without credentials, query, fragment or escaped bytes.
        url: String,
        /// Literal branch, tag or commit ID resolved to an immutable commit.
        reference: String,
    },
}

impl CatalogSource for PackSource {
    fn materialize(&self, pack_root: &Path, scratch_root: &Path) -> Result<SourceRevision> {
        match self {
            Self::LocalDirectory(path) => {
                LocalCatalogSource::new(path).materialize(pack_root, scratch_root)
            }
            Self::LocalGit { path, reference } => {
                GitCatalogSource::local(path, reference).materialize(pack_root, scratch_root)
            }
            Self::HttpsGit { url, reference } => {
                GitCatalogSource::https(url, reference).materialize(pack_root, scratch_root)
            }
        }
    }
}

// Git process policy belongs to the source adapter, not the loader or cache.
// The only per-operation state is whether file transport was explicitly selected.
pub(super) struct GitRunner {
    local: bool,
}

impl GitRunner {
    // File transport is enabled only for an explicitly selected local repository.
    pub(super) fn new(local: bool) -> Self {
        Self { local }
    }

    // Avoid launching nonexecutables while resolving external developer tooling.
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

    // Search only the caller-trusted PATH, without executing candidates.
    pub(super) fn find_executable(search: &std::ffi::OsStr) -> Result<PathBuf> {
        std::env::split_paths(search)
        .map(|directory| directory.join("git"))
        .find(|path| Self::usable_git_file(path))
        .ok_or_else(|| CatalogError::new("Git refresh requires a usable Git installation on PATH. Local, bundled and already cached packs do not require Git; no developer tools are installed automatically."))
    }

    #[cfg(target_os = "macos")]
    // Bypass Apple's installation launcher only when selected Git is present.
    pub(super) fn selected_developer_git(developer: &Path) -> Result<PathBuf> {
        let git = developer.join("usr/bin/git");
        if Self::usable_git_file(&git)
            && fs::canonicalize(&git).ok().as_deref() != Some(Path::new("/usr/bin/git"))
        {
            Ok(git)
        } else {
            Err(CatalogError::new(
                "Git refresh is unavailable: Apple's Git launcher has no usable selected Git. Install or configure a standalone Git on PATH, or use a local/bundled/cached pack. Durop does not install Command Line Tools.",
            ))
        }
    }

    // Missing external Git must not trigger a developer-tools installation prompt.
    fn executable() -> Result<PathBuf> {
        let search = std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into());
        let git = Self::find_executable(&search)?;
        #[cfg(target_os = "macos")]
        if fs::canonicalize(&git).ok().as_deref() == Some(Path::new("/usr/bin/git")) {
            // /usr/bin/git is a launcher that can display a developer-tools install
            // prompt on a clean Mac. Check selected tools without invoking that stub.
            let mut select = Command::new("/usr/bin/xcode-select");
            select.arg("-p");
            let bytes = Self::run_command(select, std::time::Duration::from_secs(2)).map_err(|_| {
                CatalogError::new(
                    "Git refresh requires usable external Git. Apple's Git launcher has no selected developer tools; Durop will not invoke an installation prompt. Use a local/bundled/cached pack, or configure standalone Git on PATH.",
                )
            })?;
            let directory = String::from_utf8(bytes)
                .map_err(|_| CatalogError::new("invalid selected developer-tools path"))?;
            return Self::selected_developer_git(Path::new(directory.trim()));
        }
        Ok(git)
    }

    /// Bounded output and wall time; errors deliberately omit Git stderr/URLs to avoid credential leaks.
    fn run(&self, repo: &Path, args: &[&std::ffi::OsStr]) -> Result<Vec<u8>> {
        let mut command = Command::new(Self::executable()?);
        command
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "protocol.allow=never",
                "-c",
                "protocol.https.allow=always",
                "-c",
                if self.local {
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
        Self::run_command(command, std::time::Duration::from_secs(120)).map_err(|error| {
            CatalogError::new(format!(
                "Git refresh could not complete; check that Git is usable and the source/ref is available. Local/bundled/cached packs remain usable. {error}"
            ))
        })
    }

    // Drain bounded output concurrently so large objects cannot deadlock a pipe.
    pub(super) fn run_command(
        mut command: Command,
        timeout: std::time::Duration,
    ) -> Result<Vec<u8>> {
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
                .take(Catalog::MAX_FILE_BYTES + 1)
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
                Self::stop(&mut child);
                return Err(CatalogError::new("Git operation timed out"));
            }
            thread::sleep(Duration::from_millis(10));
        };
        let remaining = timeout.saturating_sub(start.elapsed());
        let bytes = match receiver.recv_timeout(remaining) {
            Ok(result) => result?,
            Err(_) => {
                Self::stop(&mut child);
                return Err(CatalogError::new("Git output timed out"));
            }
        };
        if !status.success() {
            return Err(CatalogError::new(
                "Git operation failed (source/ref unavailable or rejected); diagnostics suppressed to protect credentials",
            ));
        }
        if bytes.len() as u64 > Catalog::MAX_FILE_BYTES {
            return Err(CatalogError::new(
                "Git object/listing exceeds 256 MiB limit",
            ));
        }
        Ok(bytes)
    }

    // Git fetch may spawn remote helpers. Kill its isolated group on Unix, including
    // when the top-level process exited but a helper retained its output pipe.
    fn stop(child: &mut std::process::Child) {
        #[cfg(unix)]
        {
            unsafe extern "C" {
                fn kill(
                    pid: std::os::raw::c_int,
                    signal: std::os::raw::c_int,
                ) -> std::os::raw::c_int;
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

    // Resolve the ref and export bounded regular blobs without worktree actions.
    pub(super) fn export(
        &self,
        staging: &Path,
        data: &Path,
        source: &std::ffi::OsStr,
        reference: &str,
    ) -> Result<String> {
        use std::ffi::OsStr;
        if reference.is_empty()
            || reference.starts_with('-')
            || !reference
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
        {
            return Err(CatalogError::new(
                "invalid Git reference; select a branch, tag or commit",
            ));
        }
        let repo = staging.join("objects");
        fs::create_dir(&repo)?;
        self.run(
            &repo,
            &[
                OsStr::new("init"),
                OsStr::new("--bare"),
                OsStr::new("--template=/dev/null"),
            ],
        )?;
        self.run(
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
        )?;
        let revision = String::from_utf8(self.run(
            &repo,
            &[
                OsStr::new("rev-parse"),
                OsStr::new("--verify"),
                OsStr::new("FETCH_HEAD^{commit}"),
            ],
        )?)
        .map_err(|_| CatalogError::new("invalid Git revision"))?
        .trim()
        .to_owned();
        if ![40, 64].contains(&revision.len()) || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(CatalogError::new("Git did not resolve an immutable commit"));
        }
        let listing = self.run(
            &repo,
            &[
                OsStr::new("ls-tree"),
                OsStr::new("-r"),
                OsStr::new("-z"),
                OsStr::new(&revision),
            ],
        )?;
        let mut total = 0u64;
        for entry in listing.split(|b| *b == 0).filter(|x| !x.is_empty()) {
            let entry = std::str::from_utf8(entry)
                .map_err(|_| CatalogError::new("Git paths must be UTF-8"))?;
            let (header, name) = entry
                .split_once('\t')
                .ok_or_else(|| CatalogError::new("invalid Git tree"))?;
            let fields: Vec<_> = header.split(' ').collect();
            if fields.len() != 3
                || !["100644", "100755"].contains(&fields[0])
                || fields[1] != "blob"
            {
                return Err(CatalogError::new(
                    "Git packs cannot contain symlinks or submodules",
                ));
            }
            let path = Path::new(name);
            if path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
                || name.contains('\\')
                || path.components().any(|c| c.as_os_str() == ".git")
            {
                return Err(CatalogError::new("unsafe Git tree path"));
            }
            let bytes = self.run(
                &repo,
                &[
                    OsStr::new("cat-file"),
                    OsStr::new("blob"),
                    OsStr::new(fields[2]),
                ],
            )?;
            total += bytes.len() as u64;
            if total > MAX_PACK_BYTES {
                return Err(CatalogError::new("pack exceeds 1 GiB limit"));
            }
            let destination = data.join(path);
            fs::create_dir_all(destination.parent().unwrap())?;
            fs::write(destination, bytes)?;
        }
        Ok(revision)
    }
}
