//! Independent input permissions and host-owned invitation administration.
//! Secret-store and HTTPS work never run in a Bevy input handler.

use super::{NetworkAction, NetworkState, Update, notify};
use crate::transport::invitations::{
    AdminApiKey, Invitation, InvitationProvider, InvitationTarget, PlatformSecretStore,
    SecretStore, TailscaleInvitations,
};
use std::{collections::BTreeMap, sync::Arc};

/// One explicitly authorized controller. Secrets are ephemeral and never serialized.
pub struct ControllerSource {
    /// Opaque source identity, independent of the editable display name.
    pub id: String,
    /// Human-readable name supplied by the host owner.
    pub name: String,
    /// Whether this source currently permits commands; local permission is separate.
    pub enabled: bool,
    /// Source-specific pairing credential, shown only in the host's sharing UI.
    pub pairing_secret: Option<String>,
    /// Tailscale acceptance link, separate from application command permission.
    pub invitation_url: Option<String>,
    /// Sanitized progress or network-invitation status.
    pub invitation_status: String,
}

/// Host-side ownership, omitted from all persisted preferences and diagnostic output.
#[derive(Default)]
pub(super) struct Administration {
    // Lazily start store access after the UI executor and wake path are available.
    initialized: bool,
    // Ignore late credential loads when a newer save/remove supersedes them.
    generation: u64,
    // Serialize store reads and mutations so Save cannot complete after a later Remove.
    credential_busy: bool,
    // Credential lives in a fixed-origin client, never in the public panel model.
    provider: Option<Arc<dyn InvitationProvider>>,
    // Keep opaque API IDs outside the UI so revocation never trusts pasted identifiers.
    invitations: BTreeMap<String, (InvitationTarget, String)>,
    // Bound mutation concurrency to one operation per source.
    busy: std::collections::BTreeSet<String>,
}

/// Results cross to the UI thread without exposing the underlying API token.
pub(super) enum SourceUpdate {
    /// A generation-checked credential-store operation produced a usable provider.
    Credentials(
        u64,
        Result<(Option<Arc<dyn InvitationProvider>>, String), String>,
    ),
    /// A source's pending create returned a network invitation or sanitized error.
    Created(
        String,
        Arc<dyn InvitationProvider>,
        Result<(InvitationTarget, Invitation), String>,
    ),
    /// A source's pending cancellation completed or failed.
    Revoked(String, Result<(), String>),
    /// Explicit refresh reconciles acceptance without granting any additional permission.
    Listed(InvitationTarget, Result<Vec<Invitation>, String>),
}

impl Administration {
    // Keychain is preferred on macOS; other platforms use a private, dedicated file.
    fn store() -> PlatformSecretStore {
        PlatformSecretStore::new(
            crate::preferences::Preferences::config_dir().join("secrets/tailscale-api-key"),
        )
    }

    // Exported credentials override saved credentials without silently overwriting them.
    fn load() -> Result<(Option<Arc<dyn InvitationProvider>>, String), String> {
        let (key, origin) =
            match AdminApiKey::from_environment().map_err(|error| error.to_string())? {
                Some(key) => (
                    Some(key),
                    "Using TAILSCALE_API_KEY from the host environment",
                ),
                None => (
                    Self::store().load().map_err(|error| error.to_string())?,
                    "Using securely saved host API token",
                ),
            };
        match key {
            Some(key) => Ok((
                Some(Arc::new(
                    TailscaleInvitations::new(key).map_err(|error| error.to_string())?,
                )),
                origin.into(),
            )),
            None => Ok((
                None,
                "Enter a user-owned Tailscale API token to create invitations".into(),
            )),
        }
    }
}

impl NetworkState {
    /// Begin one protected credential lookup off the main thread.
    pub(super) fn initialize_administration(&mut self) {
        if !self.administration.initialized {
            self.administration.initialized = true;
            self.load_administration(None, false);
        }
    }

    // Save/remove is explicit; then resolve the actual credential according to precedence.
    fn load_administration(&mut self, save: Option<String>, remove: bool) {
        if self.administration.credential_busy {
            self.admin_status =
                "Wait for the current protected credential operation to finish".into();
            return;
        }
        self.administration.credential_busy = true;
        self.administration.provider = None;
        self.admin_secret_available = false;
        self.administration.generation = self.administration.generation.wrapping_add(1);
        let generation = self.administration.generation;
        self.admin_status = "Checking protected host credentials…".into();
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        self.runtime.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                if let Some(secret) = save {
                    let key =
                        AdminApiKey::from_secret(secret).map_err(|error| error.to_string())?;
                    Administration::store()
                        .save(&key)
                        .map_err(|error| error.to_string())?;
                }
                if remove {
                    Administration::store()
                        .delete()
                        .map_err(|error| error.to_string())?;
                }
                Administration::load()
            })
            .await
            .unwrap_or_else(|_| Err("Protected credential operation failed".into()));
            notify(
                &updates,
                &wake,
                Update::Sources(SourceUpdate::Credentials(generation, result)),
            )
            .await;
        });
    }

    /// Dispatch independently scoped source intentions; never change the To selection.
    pub(super) fn source_action(&mut self, action: NetworkAction) {
        match action {
            NetworkAction::SetLocalEnabled(enabled) => {
                self.local_permission
                    .store(false, std::sync::atomic::Ordering::Release);
                self.local_permission = Arc::new(std::sync::atomic::AtomicBool::new(enabled));
                self.local_enabled = enabled;
            }
            NetworkAction::AddController { name } => {
                let name = name.trim();
                if name.is_empty()
                    || name.len() > 80
                    || name.chars().any(char::is_control)
                    || self.controllers.len() >= 16
                {
                    self.status =
                        "Use a short controller name; at most 16 sources are supported".into();
                    return;
                }
                let id = bevy::asset::uuid::Uuid::new_v4().to_string();
                match self.sources.issue(id.clone()) {
                    Ok(credential) => self.controllers.push(ControllerSource {
                        id,
                        name: name.into(),
                        enabled: true,
                        pairing_secret: Some(credential.bearer().into()),
                        invitation_url: None,
                        invitation_status:
                            "Share pairing with an existing network member, or create an invite"
                                .into(),
                    }),
                    Err(_) => self.status = "Could not create controller credential".into(),
                }
            }
            NetworkAction::SetControllerEnabled { id, enabled } => {
                if self.sources.set_enabled(&id, enabled)
                    && let Some(source) = self.controllers.iter_mut().find(|source| source.id == id)
                {
                    source.enabled = enabled;
                }
            }
            NetworkAction::RevokeController { id } => {
                self.sources.revoke(&id);
                // Cancel pending network invitations as well; application access ends immediately.
                self.revoke_invitation(&id);
                self.controllers.retain(|source| source.id != id);
            }
            NetworkAction::CreateInvitation { id } => self.create_invitation(id),
            NetworkAction::RevokeInvitation { id } => self.revoke_invitation(&id),
            NetworkAction::RefreshInvitations => self.refresh_invitations(),
            NetworkAction::SaveAdminSecret { secret } => {
                self.load_administration(Some(secret), false)
            }
            NetworkAction::RemoveAdminSecret => self.load_administration(None, true),
            NetworkAction::CopyControllerHandoff { id } => self.copy_handoff(&id),
            _ => {}
        }
    }

    // Fixed-origin invitation client performs only the explicit button's mutation.
    fn create_invitation(&mut self, id: String) {
        let Some(provider) = self.administration.provider.clone() else {
            self.admin_status = "Configure a user-owned API token first".into();
            return;
        };
        if !self.hosting {
            self.status = "Enable remote inputs before creating an invitation".into();
            return;
        }
        let Some(source) = self.controllers.iter_mut().find(|source| source.id == id) else {
            return;
        };
        if self.administration.busy.contains(&id)
            || self.administration.invitations.contains_key(&id)
        {
            source.invitation_status = "Refresh or cancel the existing invitation first".into();
            return;
        }
        self.administration.busy.insert(id.clone());
        source.invitation_status = "Creating one-use host invitation…".into();
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        self.runtime.spawn(async move {
            let result = async {
                let device = crate::transport::local_device_id()
                    .await
                    .map_err(|error| error.to_string())?;
                let target = InvitationTarget::Device(device);
                let invitation = provider
                    .create(target.clone())
                    .await
                    .map_err(|error| error.to_string())?;
                Ok((target, invitation))
            }
            .await;
            notify(
                &updates,
                &wake,
                Update::Sources(SourceUpdate::Created(id, provider, result)),
            )
            .await;
        });
    }

    // Revoking an invite does not disable local controls or other controllers.
    fn revoke_invitation(&mut self, id: &str) {
        if self.administration.busy.contains(id) {
            return;
        }
        let Some((target, invitation)) = self.administration.invitations.get(id).cloned() else {
            return;
        };
        let Some(provider) = self.administration.provider.clone() else {
            self.admin_status =
                "Restore the host API token to cancel the Tailscale invitation".into();
            return;
        };
        self.administration.busy.insert(id.into());
        if let Some(source) = self.controllers.iter_mut().find(|source| source.id == id) {
            source.invitation_status = "Canceling network invitation…".into();
        }
        let id = id.to_owned();
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        self.runtime.spawn(async move {
            let result = provider
                .revoke(target, invitation)
                .await
                .map_err(|error| error.to_string());
            notify(
                &updates,
                &wake,
                Update::Sources(SourceUpdate::Revoked(id, result)),
            )
            .await;
        });
    }

    // Reconcile only the local host device; no tailnet-wide enumeration is needed.
    fn refresh_invitations(&mut self) {
        let Some(provider) = self.administration.provider.clone() else {
            return;
        };
        if !self.administration.busy.insert("refresh".into()) {
            return;
        }
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        self.runtime.spawn(async move {
            let result = crate::transport::local_device_id().await;
            let target =
                InvitationTarget::Device(result.as_ref().ok().cloned().unwrap_or_default());
            let result = match result {
                Ok(_) => provider
                    .list(target.clone())
                    .await
                    .map_err(|error| error.to_string()),
                Err(_) => Err("Local Tailscale device is unavailable".into()),
            };
            notify(
                &updates,
                &wake,
                Update::Sources(SourceUpdate::Listed(target, result)),
            )
            .await;
        });
    }

    /// Apply asynchronous results without allowing revoked sources to regain permission.
    pub(super) fn source_update(&mut self, update: SourceUpdate) {
        match update {
            SourceUpdate::Credentials(generation, result)
                if generation == self.administration.generation =>
            {
                self.administration.credential_busy = false;
                match result {
                    Ok((provider, status)) => {
                        self.admin_secret_available = provider.is_some();
                        self.administration.provider = provider;
                        self.admin_status = status;
                    }
                    Err(error) => {
                        self.administration.provider = None;
                        self.admin_secret_available = false;
                        self.admin_status = error;
                    }
                }
            }
            SourceUpdate::Created(id, provider, result) => {
                self.administration.busy.remove(&id);
                match result {
                    Ok((target, invitation)) => {
                        if let Some(source) =
                            self.controllers.iter_mut().find(|source| source.id == id)
                        {
                            source.invitation_url = Some(invitation.invite_url);
                            source.invitation_status =
                                "Invitation ready; copy connection details to share privately"
                                    .into();
                            self.administration
                                .invitations
                                .insert(id, (target, invitation.id));
                        } else {
                            // A source removed during creation must not leave a known pending invite.
                            self.runtime.spawn(async move {
                                let _ = provider.revoke(target, invitation.id).await;
                            });
                        }
                    }
                    Err(error) => {
                        if let Some(source) =
                            self.controllers.iter_mut().find(|source| source.id == id)
                        {
                            source.invitation_status = error;
                        }
                    }
                }
            }
            SourceUpdate::Revoked(id, result) => {
                self.administration.busy.remove(&id);
                if result.is_ok() {
                    self.administration.invitations.remove(&id);
                }
                if let Some(source) = self.controllers.iter_mut().find(|source| source.id == id) {
                    match result {
                        Ok(()) => {
                            source.invitation_url = None;
                            source.invitation_status = "Invitation canceled. Previously accepted sharing is managed in Tailscale".into();
                        }
                        Err(error) => source.invitation_status = error,
                    }
                } else if let Err(error) = result {
                    self.admin_status = format!(
                        "Controller access revoked; invitation cancellation failed: {error}"
                    );
                }
            }
            SourceUpdate::Listed(target, result) => {
                self.administration.busy.remove("refresh");
                match result {
                    Ok(invitations) => {
                        for source in &mut self.controllers {
                            if let Some((saved_target, id)) =
                                self.administration.invitations.get(&source.id)
                                && saved_target == &target
                            {
                                source.invitation_status = match invitations.iter().find(|invite| &invite.id == id) {
                                Some(invite) if invite.accepted => "Network invitation accepted; Durop pairing is still required",
                                Some(_) => "Network invitation awaiting acceptance",
                                None => "Network invitation expired or no longer available",
                            }.into();
                            }
                        }
                    }
                    Err(error) => self.admin_status = error,
                }
            }
            _ => {}
        }
    }

    // Copy only after an explicit host gesture; administrative credentials are never included.
    fn copy_handoff(&mut self, id: &str) {
        if !self.hosting {
            self.status = "Enable remote inputs before copying connection details".into();
            return;
        }
        let Some(source) = self.controllers.iter().find(|source| source.id == id) else {
            return;
        };
        let Some(secret) = source.pairing_secret.as_ref() else {
            return;
        };
        let text = format!(
            "Durop controller: {}\n{}Host: {}\nDurop pairing token: {}\nStart Durop with --mode controller, then choose To → Tailscale and enter the host and pairing token.\nKeep these details private.",
            source.name,
            source
                .invitation_url
                .as_ref()
                .map(|url| format!("Accept Tailscale invitation: {url}\n"))
                .unwrap_or_default(),
            self.listen_address,
            secret,
        );
        #[cfg(target_os = "macos")]
        let copied = crate::app_macos::copy_text(&text);
        #[cfg(not(target_os = "macos"))]
        let copied = {
            let _ = text;
            false
        };
        self.status = if copied {
            "Private connection details copied"
        } else {
            "Clipboard unavailable; copy the host, pairing token and invitation shown in From"
        }
        .into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // No operating-system credential lookup or network request is needed for these state tests.
    fn network() -> NetworkState {
        NetworkState::for_ui_test(crate::commands::HostSnapshot {
            session_id: "source-tests".into(),
            catalog_revision: "revision".into(),
            sequence: 0,
            clips: vec![],
            instances: vec![],
        })
    }

    #[test]
    fn local_toggle_revokes_old_admissions_without_affecting_remote_permissions() {
        let mut network = network();
        network.source_action(NetworkAction::AddController {
            name: "Alice".into(),
        });
        network.source_action(NetworkAction::AddController { name: "Bob".into() });
        let previous = network.local_permission.clone();
        network.source_action(NetworkAction::SetLocalEnabled(false));
        assert!(!previous.load(std::sync::atomic::Ordering::Acquire));
        assert!(network.controllers.iter().all(|source| source.enabled));
        network.source_action(NetworkAction::SetLocalEnabled(true));
        assert!(!previous.load(std::sync::atomic::Ordering::Acquire));
        assert!(
            network
                .local_permission
                .load(std::sync::atomic::Ordering::Acquire)
        );
    }

    #[test]
    fn credential_failures_clear_capability_and_overlapping_store_mutations_are_rejected() {
        let mut network = network();
        network.administration.credential_busy = true;
        network.administration.generation = 3;
        network.load_administration(None, true);
        assert_eq!(
            network.administration.generation, 3,
            "a second store operation must not race the first"
        );
        assert!(network.admin_status.contains("Wait"));
        network.admin_secret_available = true;
        network.source_update(SourceUpdate::Credentials(
            3,
            Err("secure storage unavailable".into()),
        ));
        assert!(!network.admin_secret_available);
        assert!(network.administration.provider.is_none());
        assert!(!network.administration.credential_busy);
    }

    #[test]
    fn revoked_source_cannot_be_restored_by_a_stale_toggle_or_self_connection() {
        let mut network = network();
        network.source_action(NetworkAction::AddController {
            name: "Alice".into(),
        });
        let id = network.controllers[0].id.clone();
        network.source_action(NetworkAction::RevokeController { id: id.clone() });
        network.source_action(NetworkAction::SetControllerEnabled { id, enabled: true });
        assert!(network.controllers.is_empty());
        assert!(network.sources.list().is_empty());
        network.listen_address = "100.100.0.1:48732".into();
        network.address = "ws://100.100.0.1:48732/control".into();
        network.secret = "private input".into();
        network.connect();
        assert!(network.client_job.is_none());
        assert!(network.secret.is_empty());
        assert!(network.status.contains("cannot point back"));
    }
}
