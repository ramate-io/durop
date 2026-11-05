//! Benign local settings; credentials and remote access are never persisted.
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// Maximum serialized settings size accepted from disk.
const LIMIT: usize = 16 * 1024;

/// Relocatable reference to the sample pack inside the running application.
const BUNDLED_PACK: &str = "@bundled/podcast-pack";

/// Persisted local UI settings; session credentials and destination are intentionally absent.
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Preferences {
    /// External pack source or the relocatable bundled-pack token.
    pub pack: String,
    /// Last catalog search text.
    pub query: String,
    /// Preferred display index, resolved again against current monitors at launch.
    pub display: Option<usize>,
}

impl Preferences {
    /// Recognize historical absolute bundle paths so app relocation does not break defaults.
    fn legacy_bundled_pack(source: &str) -> bool {
        let path = Path::new(source);
        path.is_absolute()
            && path.ends_with("Contents/Resources/sample-packs/podcast-pack")
            && path
                .ancestors()
                .nth(4)
                .and_then(Path::file_name)
                .is_some_and(|name| name.to_string_lossy().ends_with(".app"))
    }

    /// Persist bundled resources as a relocatable token; preserve external sources exactly.
    pub fn stored_pack(source: &str, current_default: &Path) -> String {
        if Path::new(source) == current_default
            || source == BUNDLED_PACK
            || Self::legacy_bundled_pack(source)
        {
            BUNDLED_PACK.into()
        } else {
            source.into()
        }
    }

    /// Resolve the bundled token or a legacy path against the currently running app.
    pub fn resolved_pack(source: &str, current_default: &Path) -> PathBuf {
        if source.is_empty() || source == BUNDLED_PACK || Self::legacy_bundled_pack(source) {
            current_default.into()
        } else {
            source.into()
        }
    }

    /// Choose the user home, falling back to temporary storage when HOME is unavailable.
    fn home() -> PathBuf {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir)
    }

    /// Return the platform-specific directory for benign, per-user settings.
    pub fn config_dir() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            Self::home().join("Library/Application Support/io.ramate.durop")
        }
        #[cfg(not(target_os = "macos"))]
        {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| Self::home().join(".config"))
                .join("io.ramate.durop")
        }
    }

    /// Return the platform-specific directory for disposable pack snapshots.
    pub fn cache_dir() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            Self::home().join("Library/Caches/io.ramate.durop")
        }
        #[cfg(not(target_os = "macos"))]
        {
            std::env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| Self::home().join(".cache"))
                .join("io.ramate.durop")
        }
    }

    /// Load benign settings, returning defaults for absent, corrupt or oversized files.
    pub fn load() -> Preferences {
        Self::load_at(&Self::config_dir().join("preferences.json"))
    }

    /// Read at most the configured byte limit before deserializing settings.
    fn load_at(path: &Path) -> Preferences {
        let mut bytes = Vec::new();
        let Ok(file) = std::fs::File::open(path) else {
            return Preferences::default();
        };
        if file.take(LIMIT as u64 + 1).read_to_end(&mut bytes).is_err() || bytes.len() > LIMIT {
            return Preferences::default();
        }
        serde_json::from_slice(&bytes).unwrap_or_default()
    }

    /// Atomically persist these settings without recording credentials or remote access.
    pub fn save(&self) -> std::io::Result<()> {
        self.save_at(&Self::config_dir().join("preferences.json"))
    }

    /// Write a private temporary file, flush it and replace the destination atomically.
    fn save_at(&self, path: &Path) -> std::io::Result<()> {
        let bytes = serde_json::to_vec(self)?;
        if bytes.len() > LIMIT {
            return Err(std::io::Error::other("preferences exceed size limit"));
        }
        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::other("missing preferences directory"))?;
        std::fs::create_dir_all(parent)?;
        let temporary = parent.join(format!(
            ".preferences-{}.tmp",
            bevy::asset::uuid::Uuid::new_v4()
        ));
        let result = (|| {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_data()?;
            std::fs::rename(&temporary, path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn relocated_bundle_defaults_follow_current_resources_and_external_packs_stay_exact() {
        let old = "/tmp/DuropSmoke.app/Contents/Resources/sample-packs/podcast-pack";
        let current =
            Path::new("/Applications/Durop.app/Contents/Resources/sample-packs/podcast-pack");
        assert_eq!(Preferences::stored_pack(old, current), BUNDLED_PACK);
        assert_eq!(Preferences::resolved_pack(old, current), current);
        assert_eq!(Preferences::resolved_pack(BUNDLED_PACK, current), current);
        for external in [
            "/Users/me/packs/podcast-pack",
            "/Users/me/Contents/Resources/sample-packs/podcast-pack",
            "https://example.com/pack.git#main",
        ] {
            assert_eq!(Preferences::stored_pack(external, current), external);
            assert_eq!(
                Preferences::resolved_pack(external, current),
                Path::new(external)
            );
        }
    }
    #[test]
    fn bounded_atomic_preferences_preserve_only_local_settings() {
        let root =
            std::env::temp_dir().join(format!("durop-prefs-{}", bevy::asset::uuid::Uuid::new_v4()));
        let path = root.join("preferences.json");
        let prefs = Preferences {
            pack: "pack".into(),
            query: "日本".into(),
            display: Some(2),
        };
        prefs.save_at(&path).unwrap();
        assert!(Preferences::load_at(&path) == prefs);
        let json = std::fs::read_to_string(&path).unwrap();
        assert!(!json.contains("secret"));
        assert!(!json.contains("destination"));
        std::fs::write(&path, "bad").unwrap();
        assert!(Preferences::load_at(&path) == Preferences::default());
        std::fs::write(&path, vec![b' '; LIMIT + 1]).unwrap();
        assert!(Preferences::load_at(&path) == Preferences::default());
        std::fs::remove_dir_all(root).unwrap();
    }
}
