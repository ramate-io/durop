use super::source::GitRunner;
use super::tests::{Temp, fixture};
use super::*;
use std::{process::Command, sync::Arc};

// Exercise third-party acquisition without granting it publication privileges.
struct FixtureSource {
    fixture: PathBuf,
    identity: &'static str,
    revision: &'static str,
    fault: Option<&'static str>,
}

impl CatalogSource for FixtureSource {
    fn materialize(&self, pack: &Path, scratch: &Path) -> Result<SourceRevision> {
        assert!(fs::read_dir(pack)?.next().is_none());
        assert!(fs::read_dir(scratch)?.next().is_none());
        fs::write(scratch.join("temporary-secret"), b"not published")?;
        LocalCatalogSource::new(&self.fixture).materialize(pack, scratch)?;
        match self.fault {
            Some("acquisition") => return Err(CatalogError::new("source unavailable")),
            Some("validation") => fs::write(pack.join("durop.json"), b"invalid")?,
            Some("oversized") => fs::File::create(pack.join("oversized.bin"))?
                .set_len(Catalog::MAX_FILE_BYTES + 1)?,
            #[cfg(unix)]
            Some("symlink") => std::os::unix::fs::symlink(
                self.fixture.join("reactions/chime.wav"),
                pack.join("linked.wav"),
            )?,
            Some("hardlink") => fs::hard_link(
                self.fixture.join("reactions/chime.wav"),
                pack.join("linked.wav"),
            )?,
            _ => (),
        }
        Ok(SourceRevision {
            identity: self.identity.into(),
            revision: self.revision.into(),
        })
    }
}

#[test]
fn injected_source_must_validate_before_publication_and_cleans_every_stage() {
    let input = fixture();
    let storage = Temp::new();
    let mut cache = CatalogCache::new(storage.0.clone()).unwrap();
    let mut source = FixtureSource {
        fixture: input.0.clone(),
        identity: "fake",
        revision: "one",
        fault: None,
    };
    let first = cache.refresh(&source).unwrap();
    assert_eq!(first.revision, "one");
    assert!(!first.root.join("temporary-secret").exists());
    let published = fs::read_dir(&storage.0).unwrap().count();
    for fault in ["acquisition", "validation", "hardlink", "oversized"] {
        source.fault = Some(fault);
        assert!(cache.refresh(&source).is_err(), "{fault}");
        assert!(Arc::ptr_eq(&cache.pin().unwrap(), &first));
        assert_eq!(fs::read_dir(&storage.0).unwrap().count(), published);
    }

    #[cfg(unix)]
    {
        source.fault = Some("symlink");
        assert!(cache.prepare(&source).is_err());
        assert_eq!(fs::read_dir(&storage.0).unwrap().count(), published);
    }
    source.fault = None;
    source.revision = "two";
    let prepared = cache.prepare(&source).unwrap();
    assert_ne!(first.root, prepared.root);
    assert!(Arc::ptr_eq(&cache.pin().unwrap(), &first));
    source.identity = "another-source";
    assert_ne!(cache.prepare(&source).unwrap().root, prepared.root);
    source.revision = "";
    assert!(cache.prepare(&source).is_err());
    drop(input);
    assert!(first.clips[0].asset_path.is_file());
}

#[test]
fn concrete_sources_share_the_object_safe_cache_contract() {
    let repo = repository();
    let local = LocalCatalogSource::new(&repo.0);
    let git = GitCatalogSource::local(&repo.0, "HEAD");
    let storage = Temp::new();
    let cache = CatalogCache::new(storage.0.clone()).unwrap();
    for source in [&local as &dyn CatalogSource, &git as &dyn CatalogSource] {
        let first = cache.prepare(source).unwrap();
        let repeat = cache.prepare(source).unwrap();
        assert_eq!(first.root, repeat.root);
        assert_eq!(first.clips[0].identity, "sample/chime");
        assert_eq!(
            fs::read(&first.clips[0].asset_path).unwrap(),
            fs::read(repo.0.join("reactions/chime.wav")).unwrap()
        );
        assert!(!first.root.join(".git").exists());
    }
}

// Commit a local authored fixture so export tests need no network access.
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

// Supply identity per command without changing user/repository Git config.
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
    assert!(Catalog::read_file(&pin.clips[0].asset_path).is_ok());
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
        Catalog::read_file(&first.clips[0].asset_path).unwrap(),
        Catalog::read_file(&repo.0.join("reactions/chime.wav")).unwrap()
    );
    let mut meta: serde_json::Value =
        serde_json::from_slice(&fs::read(repo.0.join("reactions/chime.json")).unwrap()).unwrap();
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
    assert!(Catalog::read_file(&pin.clips[0].asset_path).is_ok());
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
        Catalog::read_file(&first.clips[0].asset_path).unwrap(),
        Catalog::read_file(&next.clips[0].asset_path).unwrap()
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
            GitRunner::run_command(command, std::time::Duration::from_millis(100))
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
        assert!(GitCatalogSource::validate_https(url).is_err());
    }
    assert!(GitCatalogSource::validate_https("https://example.org/org/pack.git").is_ok());
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
    assert!(
        GitRunner::find_executable(&search)
            .unwrap_err()
            .0
            .contains("Local, bundled")
    );
    let candidate = directory.0.join("git");
    fs::write(&candidate, b"not an executable").unwrap();
    fs::set_permissions(&candidate, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(GitRunner::find_executable(&search).is_err());
    fs::set_permissions(&candidate, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(GitRunner::find_executable(&search).unwrap(), candidate);
    // Detection checks readiness without launching an arbitrary file or the
    // Apple installation stub. Actual Git errors remain bounded/suppressed.
}

#[cfg(target_os = "macos")]
#[test]
fn absent_selected_apple_git_does_not_request_installation() {
    let directory = Temp::new();
    let error = GitRunner::selected_developer_git(&directory.0).unwrap_err();
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
    let media = Catalog::read_file(&first.clips[0].asset_path).unwrap();
    assert_eq!(
        media,
        Catalog::read_file(&local.clips[0].asset_path).unwrap()
    );
    assert_eq!(
        Catalog::read_file(&first.clips[0].icon_path).unwrap(),
        Catalog::read_file(&local.clips[0].icon_path).unwrap()
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
    assert_eq!(Catalog::read_file(&pin.clips[0].asset_path).unwrap(), media);
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
    let revision = GitRunner::new(false)
        .export(
            &staging.0,
            &data,
            std::ffi::OsStr::new("https://github.com/ramate-io/durop.git"),
            expected,
        )
        .unwrap();
    assert_eq!(revision, expected);
    let catalog = load_local(&data.join("examples/sample-pack")).unwrap();
    assert_eq!(catalog.clips[0].identity, "sample/chime");
    assert_eq!(
        catalog.clips[0].decoder_readiness,
        DecoderReadiness::Unchecked
    );
}
