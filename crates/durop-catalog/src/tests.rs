use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
// Distinct temporary directories let parallel tests mutate independent packs.
static NEXT: AtomicU64 = AtomicU64::new(0);

// Remove fixture and cache files even when an assertion unwinds.
pub(super) struct Temp(pub(super) PathBuf);

impl Temp {
    // Allocate outside source/cache roots used by individual test cases.
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

// Copy only the minimal authored pack, keeping fixture mutations test-local.
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
    let mut meta: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
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
