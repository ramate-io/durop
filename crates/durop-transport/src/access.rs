//! Independent, revocable controller credentials; network membership grants no app access.

use crate::{SessionCredential, TransportError, random_hex};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use subtle::ConstantTimeEq;
use tokio::sync::watch;

/// Non-secret projection of one host-owned controller permission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceAccess {
    /// Host-assigned controller identity, never accepted from a remote message.
    pub source_id: String,
    /// Whether the current credential may authenticate and admit commands.
    pub enabled: bool,
}

/// Shared host authorization registry. Clones address the same permissions.
///
/// Issuing or revoking a source invalidates its existing connections and queued
/// requests immediately. Disabling preserves its credential for a later enable,
/// but a closed connection must authenticate again. No secrets appear in `list`.
#[derive(Clone, Default)]
pub struct SourceAccessRegistry {
    // Serializes credential replacement with authentication and permission transitions.
    entries: Arc<Mutex<BTreeMap<String, AccessEntry>>>,
}

/// The secret and generation belong to one source; old leases never revive.
struct AccessEntry {
    secret: SessionCredential,
    namespace: String,
    lease: AccessLease,
}

/// Authentication result owned by one connection, also attached to queued commands.
#[derive(Clone)]
pub(crate) struct AccessLease {
    pub(crate) source_id: String,
    namespace: String,
    pub(crate) active: Arc<AtomicBool>,
    pub(crate) revoked: watch::Sender<bool>,
}

impl AccessLease {
    // Each re-enable gets a fresh guard so old requests remain invalid forever.
    fn new(source_id: String, namespace: String) -> Self {
        Self {
            source_id,
            namespace,
            active: Arc::new(AtomicBool::new(true)),
            revoked: watch::channel(false).0,
        }
    }

    // Revoke synchronously before waking any task awaiting socket activity.
    fn revoke(&self) {
        self.active.store(false, Ordering::Release);
        self.revoked.send_replace(true);
    }

    /// Domain-separated fixed-size IDs prevent inter-controller dedupe collisions.
    pub(crate) fn command_id(&self, original: &str) -> String {
        let mut hash = Sha256::new();
        hash.update(b"durop-source-command-v1\0");
        hash.update(self.namespace.as_bytes());
        hash.update(b"\0");
        hash.update(original.as_bytes());
        format!("remote:{:x}", hash.finalize())
    }
}

impl SourceAccessRegistry {
    /// Generate an independent secret for a bounded host-assigned source identity.
    /// Reissuing replaces its credential and permanently invalidates all old leases.
    pub fn issue(&self, source_id: String) -> Result<SessionCredential, TransportError> {
        if source_id.is_empty() || source_id.len() > 128 || source_id.chars().any(char::is_control)
        {
            return Err(TransportError::InvalidConfiguration);
        }
        let secret = SessionCredential(random_hex::<32>()?);
        let namespace = random_hex::<16>()?;
        let entry = AccessEntry {
            secret: SessionCredential(secret.bearer().to_owned()),
            namespace: namespace.clone(),
            lease: AccessLease::new(source_id.clone(), namespace),
        };
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !entries.contains_key(&source_id) && entries.len() >= 64 {
            return Err(TransportError::InvalidConfiguration);
        }
        if let Some(old) = entries.insert(source_id, entry) {
            old.lease.revoke();
        }
        Ok(secret)
    }

    /// Remove one credential and immediately invalidate its connections and queues.
    /// Returns false if the source was already absent.
    pub fn revoke(&self, source_id: &str) -> bool {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = entries.remove(source_id) {
            entry.lease.revoke();
            true
        } else {
            false
        }
    }

    /// Toggle a source independently. Re-enabling permits fresh authentication only;
    /// requests and connections from before disable remain permanently invalid.
    pub fn set_enabled(&self, source_id: &str, enabled: bool) -> bool {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(entry) = entries.get_mut(source_id) else {
            return false;
        };
        if enabled && !entry.lease.active.load(Ordering::Acquire) {
            entry.lease = AccessLease::new(source_id.to_owned(), entry.namespace.clone());
        } else if !enabled {
            entry.lease.revoke();
        }
        true
    }

    /// Return bounded, sorted host UI rows without credentials or invitation URLs.
    pub fn list(&self) -> Vec<SourceAccess> {
        self.entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .iter()
            .map(|(id, entry)| SourceAccess {
                source_id: id.clone(),
                enabled: entry.lease.active.load(Ordering::Acquire),
            })
            .collect()
    }

    /// Match only host-issued secrets; no remote source identifier participates.
    pub(crate) fn authenticate(&self, authorization: &[u8]) -> Option<AccessLease> {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut matched = None;
        for entry in entries.values() {
            let expected = format!("Bearer {}", entry.secret.bearer());
            if bool::from(expected.as_bytes().ct_eq(authorization))
                && entry.lease.active.load(Ordering::Acquire)
            {
                matched = Some(entry.lease.clone());
            }
        }
        matched
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources_are_independent_and_old_leases_never_revive() {
        let registry = SourceAccessRegistry::default();
        let a = registry.issue("a".into()).unwrap();
        let b = registry.issue("b".into()).unwrap();
        let auth = |secret: &SessionCredential| format!("Bearer {}", secret.bearer());
        let old = registry.authenticate(auth(&a).as_bytes()).unwrap();
        let other = registry.authenticate(auth(&b).as_bytes()).unwrap();
        assert_ne!(old.command_id("same"), other.command_id("same"));
        assert!(registry.set_enabled("a", false));
        assert!(!old.active.load(Ordering::Acquire));
        assert!(registry.authenticate(auth(&a).as_bytes()).is_none());
        assert!(registry.set_enabled("a", true));
        assert!(registry.authenticate(auth(&a).as_bytes()).is_some());
        assert!(!old.active.load(Ordering::Acquire));
        let replacement = registry.issue("a".into()).unwrap();
        assert!(registry.authenticate(auth(&a).as_bytes()).is_none());
        assert!(
            registry
                .authenticate(auth(&replacement).as_bytes())
                .is_some()
        );
        assert!(registry.revoke("a"));
        assert!(other.active.load(Ordering::Acquire));
        assert_eq!(
            registry.list(),
            vec![SourceAccess {
                source_id: "b".into(),
                enabled: true
            }]
        );
    }
}
