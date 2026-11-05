//! Provider-specific modal drafts with effect-free validation and explicit apply intents.
//!
//! Editing or canceling a draft never changes controller/catalog state. Only a validated
//! intent may be handed to the existing application actions. Pairing tokens stay in
//! ephemeral fields and are never represented by a serializable configuration type.

use crate::transport::SessionCredential;
use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    path::{Component, Path, PathBuf},
};

/// Maximum input bytes per modal field, independent of the editor's character limit.
const FIELD_LIMIT: usize = 4096;

/// Host implementation selected by the host dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostProvider {
    /// The local, in-process controller and existing native display controls.
    Native,
    /// An authenticated Durop session over an already configured Tailscale network.
    Tailscale,
    /// Internal Native-modal sharing action; the host generates its own pairing token.
    TailscaleListener,
}

/// Catalog acquisition implementation selected by the catalog dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogProvider {
    /// Offline reactions shipped inside the application bundle.
    Bundled,
    /// A user-selected absolute directory, acquired by the existing local provider.
    LocalDirectory,
    /// A credential-free HTTPS repository, explicit ref, and optional pack directory.
    GitHttps,
}

/// Provider identity determines the complete modal field schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// Configure a controller implementation without changing the catalog selection.
    Host(HostProvider),
    /// Configure a catalog source without changing controller permission.
    Catalog(CatalogProvider),
}

impl Provider {
    /// Human-readable dropdown/modal label, without user-entered values or secrets.
    pub fn label(self) -> &'static str {
        match self {
            Self::Host(HostProvider::Native) => "Native",
            Self::Host(HostProvider::Tailscale) => "Tailscale",
            Self::Host(HostProvider::TailscaleListener) => "Share this Mac",
            Self::Catalog(CatalogProvider::Bundled) => "Durop reactions",
            Self::Catalog(CatalogProvider::LocalDirectory) => "Local directory",
            Self::Catalog(CatalogProvider::GitHttps) => "Git HTTPS",
        }
    }

    // A fixed key list prevents field-schema mutation from bypassing provider validation.
    fn keys(self) -> &'static [ConfigurationFieldKey] {
        use ConfigurationFieldKey::*;
        match self {
            Self::Host(HostProvider::Native) => &[Display],
            Self::Host(HostProvider::Tailscale) => &[Endpoint, PairingToken],
            Self::Host(HostProvider::TailscaleListener) => &[Endpoint],
            Self::Catalog(CatalogProvider::Bundled | CatalogProvider::LocalDirectory) => {
                &[Directory]
            }
            Self::Catalog(CatalogProvider::GitHttps) => &[GitUrl, GitReference, GitDirectory],
        }
    }
}

/// Stable field identity used by validation, keyboard focus, and renderer labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationFieldKey {
    /// Primary/current native monitor choice.
    Display,
    /// Explicit Tailscale IP and port, optionally using the exact WebSocket target.
    Endpoint,
    /// Durop host-issued session credential, never a Tailscale API key.
    PairingToken,
    /// Bundled or external local catalog directory.
    Directory,
    /// Credential-free HTTPS Git repository URL.
    GitUrl,
    /// Branch, tag, or commit selected before catalog materialization.
    GitReference,
    /// Slash-separated directory containing the pack inside the repository.
    GitDirectory,
}

/// One renderer choice; values are stable tokens rather than parsed display labels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// Token written into the associated field when selected.
    pub value: String,
    /// User-facing option label.
    pub label: String,
}

/// An enumerated monitor supplied by the existing native display system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayChoice {
    /// Current application monitor index, checked again at Apply time.
    pub index: usize,
    /// Existing native monitor description shown in the modal.
    pub label: String,
}

/// Benign current settings and provider defaults used when opening a fresh modal.
/// This type intentionally has no pairing-token field.
#[derive(Clone)]
pub struct ConfigurationContext {
    /// Currently enumerated monitors; Primary is added separately by the schema.
    pub displays: Vec<DisplayChoice>,
    /// Current native display selection, or Primary when absent.
    pub current_display: Option<usize>,
    /// Existing target text; validation still runs before applying a Tailscale draft.
    pub current_endpoint: String,
    /// Existing local source input, reused only if it is an absolute directory path.
    pub local_pack: String,
    /// Trusted current bundle resource path supplied by application configuration.
    pub bundled_path: PathBuf,
    /// Example/current HTTPS repository URL, without a fragment or credentials.
    pub git_url: String,
    /// Example/current branch, tag, or commit.
    pub git_reference: String,
    /// Example/current relative pack directory; empty means the repository root.
    pub git_directory: String,
}

impl fmt::Debug for ConfigurationContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigurationContext")
            .field("display_count", &self.displays.len())
            .field("current_display", &self.current_display)
            .finish_non_exhaustive()
    }
}

/// Renderer-independent schema and mutable input for one modal row.
/// Debug omits every value, including invalid input accidentally pasted into a benign field.
pub struct ConfigurationField {
    /// Stable provider field identity.
    pub key: ConfigurationFieldKey,
    /// Visible row label.
    pub label: &'static str,
    /// Ephemeral editor text; never persist the draft or its fields.
    pub value: String,
    /// Render masked input and avoid including its value in UI fingerprints/logs.
    pub secret: bool,
    /// Optional native choices; an empty list means ordinary text input.
    pub choices: Vec<Choice>,
    /// Show trusted bundled resource information without permitting edits.
    pub read_only: bool,
    /// Concise provider-specific meaning and input guidance.
    pub hint: &'static str,
}

impl fmt::Debug for ConfigurationField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigurationField")
            .field("key", &self.key)
            .field("value", &"[input omitted]")
            .field("secret", &self.secret)
            .field("read_only", &self.read_only)
            .finish()
    }
}

impl ConfigurationField {
    // Central construction keeps sensitive/read-only defaults explicit per provider.
    fn text(
        key: ConfigurationFieldKey,
        label: &'static str,
        value: String,
        hint: &'static str,
    ) -> Self {
        Self {
            key,
            label,
            value,
            secret: false,
            choices: vec![],
            read_only: false,
            hint,
        }
    }
}

/// An effect-free modal edit session; dropping/canceling it grants no permission.
#[derive(Debug)]
pub struct ConfigurationDraft {
    /// Provider whose exact schema and validation rules apply.
    pub provider: Provider,
    /// Bounded provider-specific fields, copied to/from the panel's existing editors.
    pub fields: Vec<ConfigurationField>,
}

/// Field-specific, privacy-safe validation failure suitable for an inline modal message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationError {
    /// Row that should receive attention or focus.
    pub field: ConfigurationFieldKey,
    /// Fixed message that never echoes the rejected input.
    pub message: &'static str,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Validated application intention; constructing it still performs no state changes.
/// This type is deliberately not serializable, and its credential has redacted Debug.
#[derive(Debug)]
pub enum ConfigurationIntent {
    /// Select the existing native display target; no network listener is created.
    Native {
        /// Existing native monitor index, or the dynamic primary display.
        display: Option<usize>,
    },
    /// Connect through the existing authenticated transport with an ephemeral token.
    Tailscale {
        /// Explicit tailnet address checked before connecting.
        endpoint: SocketAddr,
        /// Ephemeral Durop session credential, with a redacted Debug implementation.
        credential: SessionCredential,
    },
    /// Enable the existing local listener; runtime verification generates a fresh token.
    TailscaleListener {
        /// This Mac's selected tailnet address; OS assignment is checked at runtime.
        endpoint: SocketAddr,
    },
    /// Select the trusted offline pack inside the current application bundle.
    Bundled {
        /// Trusted path resolved against this running application bundle.
        path: PathBuf,
    },
    /// Ask the existing local catalog loader to acquire this directory asynchronously.
    LocalDirectory {
        /// Absolute user-selected path, to be checked by asynchronous catalog loading.
        path: PathBuf,
    },
    /// Ask the existing Git provider to resolve/export a validated source and subdirectory.
    GitHttps {
        /// Credential-free HTTPS repository URL.
        url: String,
        /// Explicit branch, tag, or commit.
        reference: String,
        /// Validated relative pack directory, or empty for repository root.
        directory: String,
    },
}

impl ConfigurationIntent {
    /// Encode only catalog intentions for the existing LoadPack action.
    /// Host intentions return None so credentials cannot become a persisted source string.
    pub fn source_value(&self) -> Option<String> {
        match self {
            Self::Bundled { path } | Self::LocalDirectory { path } => {
                Some(path.to_string_lossy().into_owned())
            }
            Self::GitHttps {
                url,
                reference,
                directory,
            } => Some(if directory.is_empty() {
                format!("{url}#{reference}")
            } else {
                format!("{url}#{reference}:{directory}")
            }),
            Self::Native { .. } | Self::Tailscale { .. } | Self::TailscaleListener { .. } => None,
        }
    }
}

impl ConfigurationDraft {
    /// Open a fresh provider draft without touching application state or reusing a token.
    pub fn new(provider: Provider, context: &ConfigurationContext) -> Self {
        use ConfigurationFieldKey::*;
        let fields = match provider {
            Provider::Host(HostProvider::Native) => {
                let mut field = ConfigurationField::text(
                    Display,
                    "Display",
                    context
                        .current_display
                        .map_or_else(|| "primary".into(), |index| index.to_string()),
                    "Choose an existing native display; Primary follows the current primary monitor.",
                );
                field.choices.push(Choice {
                    value: "primary".into(),
                    label: "Primary display".into(),
                });
                field
                    .choices
                    .extend(context.displays.iter().map(|display| Choice {
                        value: display.index.to_string(),
                        label: display.label.clone(),
                    }));
                vec![field]
            }
            Provider::Host(HostProvider::Tailscale) => {
                let mut token = ConfigurationField::text(
                    PairingToken,
                    "Pairing token",
                    String::new(),
                    "Durop host-issued session credential. This is not a Tailscale API key; no login or network changes are performed.",
                );
                token.secret = true;
                vec![
                    ConfigurationField::text(
                        Endpoint,
                        "Network endpoint",
                        context.current_endpoint.clone(),
                        "Use the host's Tailscale IP:port or ws://IP:port/control.",
                    ),
                    token,
                ]
            }
            Provider::Host(HostProvider::TailscaleListener) => vec![ConfigurationField::text(
                Endpoint,
                "This Mac's endpoint",
                context.current_endpoint.clone(),
                "Use this Mac's assigned Tailscale IP:port. Durop generates the pairing token; installation/login and tailnet policy remain separate.",
            )],
            Provider::Catalog(CatalogProvider::Bundled) => {
                let mut field = ConfigurationField::text(
                    Directory,
                    "Bundled catalog",
                    context.bundled_path.to_string_lossy().into_owned(),
                    "Durop reactions are available offline inside this application bundle.",
                );
                field.read_only = true;
                vec![field]
            }
            Provider::Catalog(CatalogProvider::LocalDirectory) => {
                let initial = if Path::new(&context.local_pack).is_absolute() {
                    context.local_pack.clone()
                } else {
                    String::new()
                };
                vec![ConfigurationField::text(
                    Directory,
                    "Directory",
                    initial,
                    "Choose an absolute local pack directory. The loader checks its manifest/assets after Apply.",
                )]
            }
            Provider::Catalog(CatalogProvider::GitHttps) => vec![
                ConfigurationField::text(
                    GitUrl,
                    "Repository URL",
                    context.git_url.clone(),
                    "HTTPS only. Use external Git credential handling; never embed tokens in this URL.",
                ),
                ConfigurationField::text(
                    GitReference,
                    "Git ref",
                    context.git_reference.clone(),
                    "An explicit branch, tag, or commit; the loader pins the resolved commit.",
                ),
                ConfigurationField::text(
                    GitDirectory,
                    "Pack directory",
                    context.git_directory.clone(),
                    "Relative directory inside the repository; leave empty only when the pack is at its root.",
                ),
            ],
        };
        Self { provider, fields }
    }

    /// Borrow a field value by stable key rather than assuming a renderer row index.
    pub fn value(&self, key: ConfigurationFieldKey) -> Option<&str> {
        self.fields
            .iter()
            .find(|field| field.key == key)
            .map(|field| field.value.as_str())
    }

    /// Validate without I/O, network calls, persistent writes, or application effects.
    /// Monitor choices are checked against the current context, not the opening snapshot.
    pub fn validate(&self, context: &ConfigurationContext) -> Result<(), ValidationError> {
        use ConfigurationFieldKey::*;
        let keys = self.provider.keys();
        if self.fields.len() != keys.len()
            || self
                .fields
                .iter()
                .zip(keys)
                .any(|(field, expected)| field.key != *expected)
        {
            return Err(ValidationError {
                field: keys[0],
                message: "The provider field schema changed; reopen configuration.",
            });
        }
        for field in &self.fields {
            if field.value.len() > FIELD_LIMIT || field.value.chars().any(char::is_control) {
                return Err(ValidationError {
                    field: field.key,
                    message: "Use a single field value of at most 4096 bytes without control characters.",
                });
            }
        }
        match self.provider {
            Provider::Host(HostProvider::Native) => {
                self.display(context)?;
            }
            Provider::Host(HostProvider::Tailscale) => {
                Self::endpoint(self.required(Endpoint))?;
                Self::credential(self.required(PairingToken))?;
            }
            Provider::Host(HostProvider::TailscaleListener) => {
                Self::endpoint(self.required(Endpoint))?;
            }
            Provider::Catalog(CatalogProvider::Bundled) => {
                if !context.bundled_path.is_absolute() {
                    return Err(ValidationError {
                        field: Directory,
                        message: "The bundled catalog path is unavailable.",
                    });
                }
            }
            Provider::Catalog(CatalogProvider::LocalDirectory) => {
                if !Path::new(self.required(Directory)).is_absolute() {
                    return Err(ValidationError {
                        field: Directory,
                        message: "Choose an absolute local directory path.",
                    });
                }
            }
            Provider::Catalog(CatalogProvider::GitHttps) => {
                Self::git_url(self.required(GitUrl))?;
                Self::git_reference(self.required(GitReference))?;
                Self::git_directory(self.required(GitDirectory))?;
            }
        }
        Ok(())
    }

    /// Consume a validated draft into data only; the caller chooses when to dispatch it.
    pub fn apply(
        mut self,
        context: &ConfigurationContext,
    ) -> Result<ConfigurationIntent, ValidationError> {
        self.validate(context)?;
        use ConfigurationFieldKey::*;
        Ok(match self.provider {
            Provider::Host(HostProvider::Native) => ConfigurationIntent::Native {
                display: self.display(context)?,
            },
            Provider::Host(HostProvider::Tailscale) => ConfigurationIntent::Tailscale {
                endpoint: Self::endpoint(self.required(Endpoint))?,
                credential: Self::credential(&self.take(PairingToken))?,
            },
            Provider::Host(HostProvider::TailscaleListener) => {
                ConfigurationIntent::TailscaleListener {
                    endpoint: Self::endpoint(self.required(Endpoint))?,
                }
            }
            Provider::Catalog(CatalogProvider::Bundled) => ConfigurationIntent::Bundled {
                path: context.bundled_path.clone(),
            },
            Provider::Catalog(CatalogProvider::LocalDirectory) => {
                ConfigurationIntent::LocalDirectory {
                    path: self.take(Directory).into(),
                }
            }
            Provider::Catalog(CatalogProvider::GitHttps) => ConfigurationIntent::GitHttps {
                url: self.take(GitUrl).trim().into(),
                reference: self.take(GitReference).trim().into(),
                directory: self.take(GitDirectory).trim().into(),
            },
        })
    }

    /// Explicit cancellation consumes the draft and emits no intent or side effect.
    pub fn cancel(self) {}

    // Validation checks the complete schema first, so required keys are present.
    fn required(&self, key: ConfigurationFieldKey) -> &str {
        self.value(key).unwrap_or_default()
    }

    // Move sensitive input into the one-shot intent instead of retaining a second draft copy.
    fn take(&mut self, key: ConfigurationFieldKey) -> String {
        self.fields
            .iter_mut()
            .find(|field| field.key == key)
            .map(|field| std::mem::take(&mut field.value))
            .unwrap_or_default()
    }

    // Primary is dynamic; a numbered monitor must still exist when Apply is pressed.
    fn display(&self, context: &ConfigurationContext) -> Result<Option<usize>, ValidationError> {
        let value = self.required(ConfigurationFieldKey::Display);
        if value == "primary" {
            return Ok(None);
        }
        value
            .parse::<usize>()
            .ok()
            .filter(|index| {
                context
                    .displays
                    .iter()
                    .any(|display| display.index == *index)
            })
            .map(Some)
            .ok_or(ValidationError {
                field: ConfigurationFieldKey::Display,
                message: "Choose a display that is currently available.",
            })
    }

    // Preserve the transport's explicit-IP Tailscale restriction and exact request path.
    fn endpoint(value: &str) -> Result<SocketAddr, ValidationError> {
        let invalid = || ValidationError {
            field: ConfigurationFieldKey::Endpoint,
            message: "Use an explicit Tailscale IP and nonzero port, optionally ws://IP:port/control.",
        };
        let value = value.trim();
        let address = if let Some(rest) = value.strip_prefix("ws://") {
            rest.strip_suffix("/control").ok_or_else(invalid)?
        } else {
            value
        };
        let endpoint: SocketAddr = address.parse().map_err(|_| invalid())?;
        let tailnet = match endpoint.ip() {
            IpAddr::V4(ip) => ip.octets()[0] == 100 && (64..128).contains(&ip.octets()[1]),
            IpAddr::V6(ip) => ip.segments()[..3] == [0xfd7a, 0x115c, 0xa1e0],
        };
        if !tailnet || endpoint.port() == 0 {
            return Err(invalid());
        }
        Ok(endpoint)
    }

    // The transport owns the credential representation and redacted Debug contract.
    fn credential(value: &str) -> Result<SessionCredential, ValidationError> {
        SessionCredential::from_secret(value.trim().into()).map_err(|_| ValidationError {
            field: ConfigurationFieldKey::PairingToken,
            message: "Enter the host app's 64-character pairing token, not a Tailscale API key.",
        })
    }

    // Match the deliberately limited credential-free grammar of the Git provider.
    fn git_url(value: &str) -> Result<(), ValidationError> {
        let invalid = || ValidationError {
            field: ConfigurationFieldKey::GitUrl,
            message: "Use a credential-free HTTPS repository URL without query, fragment, or escapes.",
        };
        let rest = value.trim().strip_prefix("https://").ok_or_else(invalid)?;
        let (host, path) = rest.split_once('/').ok_or_else(invalid)?;
        if host.is_empty()
            || path.is_empty()
            || !host
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b".-:".contains(&byte))
            || !path
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"/-_.~".contains(&byte))
        {
            return Err(invalid());
        }
        Ok(())
    }

    // Refs are passed as Git data, never options, revision expressions, or shell fragments.
    fn git_reference(value: &str) -> Result<(), ValidationError> {
        let value = value.trim();
        if value.is_empty()
            || value.starts_with('-')
            || value.contains("..")
            || value.contains("//")
            || value.ends_with(['/', '.'])
            || value
                .split('/')
                .any(|part| part.starts_with('.') || part.ends_with(".lock"))
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"/._-".contains(&byte))
        {
            return Err(ValidationError {
                field: ConfigurationFieldKey::GitReference,
                message: "Choose a branch, tag, or commit without options or revision expressions.",
            });
        }
        Ok(())
    }

    // Directory encoding shares '#' and ':' delimiters with the application source syntax.
    fn git_directory(value: &str) -> Result<(), ValidationError> {
        let value = value.trim();
        if value.is_empty() {
            return Ok(());
        }
        if value.contains(['\\', ':', '#', '?', '%'])
            || value.split('/').any(|part| {
                part.is_empty() || part == "." || part == ".." || part.eq_ignore_ascii_case(".git")
            })
            || Path::new(value)
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(ValidationError {
                field: ConfigurationFieldKey::GitDirectory,
                message: "Use a relative pack directory without empty, dot, parent, .git, or escaped components.",
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // No test directory needs to exist: configuration validates syntax, not acquisition.
    fn context() -> ConfigurationContext {
        ConfigurationContext {
            displays: vec![
                DisplayChoice {
                    index: 0,
                    label: "Primary monitor".into(),
                },
                DisplayChoice {
                    index: 3,
                    label: "Recording monitor".into(),
                },
            ],
            current_display: Some(3),
            current_endpoint: "100.100.0.10:48732".into(),
            local_pack: "/not-mounted/example-pack".into(),
            bundled_path: "/example/Durop.app/Contents/Resources/sample-packs/reactions".into(),
            git_url: "https://github.com/ramate-io/durop.git".into(),
            git_reference: "main".into(),
            git_directory: "sample-packs/reactions".into(),
        }
    }

    // Editing strings is the only operation the panel editor needs from a field row.
    fn edit(draft: &mut ConfigurationDraft, key: ConfigurationFieldKey, value: &str) {
        draft
            .fields
            .iter_mut()
            .find(|field| field.key == key)
            .unwrap()
            .value = value.into();
    }

    #[test]
    fn provider_schemas_match_native_tailscale_and_catalog_requirements() {
        let context = context();
        for provider in [
            Provider::Host(HostProvider::Native),
            Provider::Host(HostProvider::Tailscale),
            Provider::Host(HostProvider::TailscaleListener),
            Provider::Catalog(CatalogProvider::Bundled),
            Provider::Catalog(CatalogProvider::LocalDirectory),
            Provider::Catalog(CatalogProvider::GitHttps),
        ] {
            let draft = ConfigurationDraft::new(provider, &context);
            assert_eq!(
                draft
                    .fields
                    .iter()
                    .map(|field| field.key)
                    .collect::<Vec<_>>(),
                provider.keys()
            );
        }
        let native = ConfigurationDraft::new(Provider::Host(HostProvider::Native), &context);
        assert_eq!(native.fields[0].choices.len(), 3);
        let tailscale = ConfigurationDraft::new(Provider::Host(HostProvider::Tailscale), &context);
        assert!(tailscale.fields[1].secret);
        assert!(tailscale.fields[1].value.is_empty());
        assert!(tailscale.fields[1].hint.contains("not a Tailscale API key"));
        let bundled =
            ConfigurationDraft::new(Provider::Catalog(CatalogProvider::Bundled), &context);
        assert!(bundled.fields[0].read_only);
    }

    #[test]
    fn native_apply_checks_the_current_monitor_list_without_changing_it() {
        let mut context = context();
        let draft = ConfigurationDraft::new(Provider::Host(HostProvider::Native), &context);
        assert!(matches!(
            draft.apply(&context).unwrap(),
            ConfigurationIntent::Native { display: Some(3) }
        ));
        assert_eq!(context.current_display, Some(3));
        let mut unplugged = ConfigurationDraft::new(Provider::Host(HostProvider::Native), &context);
        context.displays.retain(|display| display.index != 3);
        assert_eq!(
            unplugged.validate(&context).unwrap_err().field,
            ConfigurationFieldKey::Display
        );
        edit(&mut unplugged, ConfigurationFieldKey::Display, "primary");
        assert!(matches!(
            unplugged.apply(&context).unwrap(),
            ConfigurationIntent::Native { display: None }
        ));
    }

    #[test]
    fn listener_configuration_never_requests_or_reuses_a_controller_secret() {
        let context = context();
        let provider = Provider::Host(HostProvider::TailscaleListener);
        let draft = ConfigurationDraft::new(provider, &context);
        assert_eq!(provider.label(), "Share this Mac");
        assert_eq!(draft.fields.len(), 1);
        assert!(!draft.fields[0].secret);
        assert!(draft.value(ConfigurationFieldKey::PairingToken).is_none());
        let intent = draft.apply(&context).unwrap();
        assert!(intent.source_value().is_none());
        assert!(
            matches!(intent, ConfigurationIntent::TailscaleListener { endpoint } if endpoint == "100.100.0.10:48732".parse().unwrap())
        );
    }

    #[test]
    fn tailscale_validates_endpoint_and_ephemeral_app_token_without_network_calls() {
        let context = context();
        let mut draft = ConfigurationDraft::new(Provider::Host(HostProvider::Tailscale), &context);
        assert_eq!(
            draft.validate(&context).unwrap_err().field,
            ConfigurationFieldKey::PairingToken
        );
        let token = "abcdef01".repeat(8);
        edit(&mut draft, ConfigurationFieldKey::PairingToken, &token);
        assert!(!format!("{draft:?}").contains(&token));
        let intent = draft.apply(&context).unwrap();
        assert!(!format!("{intent:?}").contains(&token));
        assert!(intent.source_value().is_none());
        let ConfigurationIntent::Tailscale {
            endpoint,
            credential,
        } = intent
        else {
            panic!("wrong intent");
        };
        assert_eq!(endpoint, "100.100.0.10:48732".parse().unwrap());
        assert_eq!(credential.bearer(), token);
        assert_eq!(context.current_endpoint, "100.100.0.10:48732");
    }

    #[test]
    fn tailscale_rejects_public_lan_loopback_credentials_in_urls_and_invalid_tokens() {
        for value in [
            "127.0.0.1:48732",
            "0.0.0.0:48732",
            "192.168.1.10:48732",
            "100.63.0.1:48732",
            "100.128.0.1:48732",
            "100.100.0.10:0",
            "host.example:48732",
            "ws://token@100.100.0.10:48732/control",
            "ws://100.100.0.10:48732/control?token=x",
            "wss://100.100.0.10:48732/control",
        ] {
            assert!(
                ConfigurationDraft::endpoint(value).is_err(),
                "accepted {value}"
            );
        }
        assert!(ConfigurationDraft::endpoint("ws://100.100.0.10:48732/control").is_ok());
        assert!(ConfigurationDraft::endpoint("[fd7a:115c:a1e0::1]:48732").is_ok());
        for token in ["", "tskey-api-not-an-app-token", "not-hex", "a"] {
            assert!(ConfigurationDraft::credential(token).is_err());
        }
    }

    #[test]
    fn catalog_apply_encodes_bundled_local_git_and_repository_root_sources() {
        let context = context();
        let bundled =
            ConfigurationDraft::new(Provider::Catalog(CatalogProvider::Bundled), &context)
                .apply(&context)
                .unwrap();
        assert_eq!(
            bundled.source_value().unwrap(),
            context.bundled_path.to_string_lossy()
        );
        let local =
            ConfigurationDraft::new(Provider::Catalog(CatalogProvider::LocalDirectory), &context)
                .apply(&context)
                .unwrap();
        assert_eq!(local.source_value().unwrap(), context.local_pack);
        let mut git =
            ConfigurationDraft::new(Provider::Catalog(CatalogProvider::GitHttps), &context);
        assert_eq!(
            git.apply(&context).unwrap().source_value().unwrap(),
            "https://github.com/ramate-io/durop.git#main:sample-packs/reactions"
        );
        git = ConfigurationDraft::new(Provider::Catalog(CatalogProvider::GitHttps), &context);
        edit(&mut git, ConfigurationFieldKey::GitDirectory, "");
        assert_eq!(
            git.apply(&context).unwrap().source_value().unwrap(),
            "https://github.com/ramate-io/durop.git#main"
        );
    }

    #[test]
    fn git_rejects_secrets_options_and_escaping_pack_directories_before_apply() {
        for url in [
            "http://github.com/ramate-io/durop.git",
            "https://token@github.com/ramate-io/durop.git",
            "https://github.com/ramate-io/durop.git?token=secret",
            "https://github.com/repo#main",
            "https://github.com/repo%2fprivate",
        ] {
            assert!(ConfigurationDraft::git_url(url).is_err());
        }
        for reference in [
            "",
            "--upload-pack=evil",
            "main..other",
            "HEAD~1",
            "refs//heads/main",
            "topic.lock",
        ] {
            assert!(ConfigurationDraft::git_reference(reference).is_err());
        }
        for directory in [
            "/root",
            "../pack",
            "pack/../other",
            "./pack",
            "a//b",
            "a/",
            "a\\b",
            ".git",
            "a/.GIT/b",
            "C:/pack",
            "pack#fragment",
            "pack:other",
            "pack%2fother",
        ] {
            assert!(
                ConfigurationDraft::git_directory(directory).is_err(),
                "accepted {directory}"
            );
        }
        assert!(ConfigurationDraft::git_reference("refs/heads/feature-provider").is_ok());
        assert!(ConfigurationDraft::git_directory("sample-packs/reactions").is_ok());
    }

    #[test]
    fn cancel_schema_mutation_and_oversized_fields_emit_no_apply_intent() {
        let context = context();
        let mut canceled =
            ConfigurationDraft::new(Provider::Host(HostProvider::Tailscale), &context);
        edit(
            &mut canceled,
            ConfigurationFieldKey::PairingToken,
            &"a".repeat(64),
        );
        canceled.cancel();
        assert_eq!(context.current_endpoint, "100.100.0.10:48732");
        let reopened = ConfigurationDraft::new(Provider::Host(HostProvider::Tailscale), &context);
        assert_eq!(
            reopened.value(ConfigurationFieldKey::PairingToken),
            Some("")
        );
        let mut malformed =
            ConfigurationDraft::new(Provider::Catalog(CatalogProvider::LocalDirectory), &context);
        malformed.fields.clear();
        assert!(malformed.apply(&context).is_err());
        let mut oversized =
            ConfigurationDraft::new(Provider::Catalog(CatalogProvider::LocalDirectory), &context);
        edit(
            &mut oversized,
            ConfigurationFieldKey::Directory,
            &"x".repeat(FIELD_LIMIT + 1),
        );
        assert!(oversized.apply(&context).is_err());
    }

    #[test]
    fn secret_bearing_drafts_and_intents_cannot_be_serialized() {
        // Negative trait assertions fail compilation if generic persistence is later added.
        trait NotSerializable<Marker> {}
        impl<T> NotSerializable<()> for T {}
        impl<T: serde::Serialize> NotSerializable<u8> for T {}
        fn not_serializable<T: NotSerializable<Marker>, Marker>() {}
        not_serializable::<ConfigurationDraft, _>();
        not_serializable::<ConfigurationIntent, _>();
    }
}
