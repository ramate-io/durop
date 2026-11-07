use super::*;
use std::sync::Mutex;

// Stateful stand-in exercises the same create/list/revoke boundary without remote writes.
#[derive(Default)]
struct MemoryInvitations(Mutex<Vec<Invitation>>);

impl InvitationProvider for MemoryInvitations {
    fn create(&self, target: InvitationTarget) -> InvitationFuture<'_, Invitation> {
        Box::pin(async move {
            target.endpoint(None)?;
            let invitation = Invitation {
                id: "fixture-1".into(),
                invite_url: "https://login.tailscale.com/admin/invite/fixture".into(),
                accepted: false,
            };
            self.0.lock().unwrap().push(invitation.clone());
            Ok(invitation)
        })
    }

    fn list(&self, _target: InvitationTarget) -> InvitationFuture<'_, Vec<Invitation>> {
        Box::pin(async move { Ok(self.0.lock().unwrap().clone()) })
    }

    fn revoke(&self, _target: InvitationTarget, id: String) -> InvitationFuture<'_, ()> {
        Box::pin(async move {
            self.0
                .lock()
                .unwrap()
                .retain(|invitation| invitation.id != id);
            Ok(())
        })
    }
}

#[tokio::test]
async fn provider_contract_is_usable_without_the_application_or_live_mutations() {
    let provider: Box<dyn InvitationProvider> = Box::<MemoryInvitations>::default();
    let target = InvitationTarget::Device("self-device".into());
    assert!(provider.list(target.clone()).await.unwrap().is_empty());
    let invitation = provider.create(target.clone()).await.unwrap();
    assert_eq!(
        provider.list(target.clone()).await.unwrap(),
        vec![invitation.clone()]
    );
    provider
        .revoke(target.clone(), invitation.id)
        .await
        .unwrap();
    assert!(provider.list(target).await.unwrap().is_empty());
}

#[test]
fn official_api_fixtures_and_least_privilege_bodies_are_validated() {
    let device = InvitationTarget::Device("n123CNTRL".into());
    assert_eq!(
        device.endpoint(None).unwrap().as_str(),
        "https://api.tailscale.com/api/v2/device/n123CNTRL/device-invites"
    );
    assert_eq!(
        device.endpoint(Some("12345")).unwrap().as_str(),
        "https://api.tailscale.com/api/v2/device-invites/12345"
    );
    assert_eq!(
        device.create_body(),
        serde_json::json!([{"multiUse":false,"allowExitNode":false}])
    );
    let tailnet = InvitationTarget::Tailnet("example.com".into());
    assert_eq!(
        tailnet.create_body(),
        serde_json::json!([{"role":"member"}])
    );
    assert_eq!(
        tailnet.endpoint(Some("29214")).unwrap().as_str(),
        "https://api.tailscale.com/api/v2/user-invites/29214"
    );
    let fixture = br#"[{"id":"12345","created":"2024-05-08T20:19:51Z","deviceId":11055,"inviteUrl":"https://login.tailscale.com/admin/invite/fixture","accepted":false}]"#;
    let parsed = TailscaleInvitations::parse(fixture).unwrap();
    assert_eq!(parsed[0].id, "12345");
    assert!(!format!("{:?}", parsed[0]).contains("/fixture"));
    assert!(
        TailscaleInvitations::parse(br#"[{"id":"x","inviteUrl":"https://evil.test/uinv/a"}]"#)
            .is_err()
    );
    assert!(
        TailscaleInvitations::parse(
            br#"[{"id":"x","inviteUrl":"http://login.tailscale.com/uinv/a"}]"#
        )
        .is_err()
    );
    assert!(device.endpoint(Some("../../other")).is_err());
    let hostile = InvitationTarget::Tailnet("https://evil.test/../x".into())
        .endpoint(None)
        .unwrap();
    assert_eq!(hostile.host_str(), Some("api.tailscale.com"));
}

#[test]
fn admin_credentials_are_distinct_from_enrollment_and_redacted() {
    for secret in [
        "tskey-auth-never-admin",
        "tskey-client-never-admin",
        "bearer\nheader",
        "",
    ] {
        assert!(AdminApiKey::from_secret(secret.into()).is_err());
    }
    let key = AdminApiKey::from_secret("tskey-api-test-only-not-real".into()).unwrap();
    assert!(!format!("{key:?}").contains(&key.0));
    let client = TailscaleInvitations::new(key).unwrap();
    assert!(client.authorization.is_sensitive());
    assert!(!format!("{:?}", client.authorization).contains("tskey"));
    for error in [
        InvitationError::Unauthorized,
        InvitationError::Unavailable,
        InvitationError::Storage,
    ] {
        assert!(!error.to_string().contains("tskey"));
    }
}

#[tokio::test]
async fn bounded_provider_rejects_excess_work_without_sending_a_request() {
    let key = AdminApiKey::from_secret("tskey-api-test-only-not-real".into()).unwrap();
    let provider = TailscaleInvitations::new(key).unwrap();
    let _first = provider.slots.try_acquire().unwrap();
    let _second = provider.slots.try_acquire().unwrap();
    assert_eq!(
        provider
            .list(InvitationTarget::Device("fixture".into()))
            .await,
        Err(InvitationError::RateLimited)
    );
}
