//! Real-loopback routing qualification; all admission uses production transport and host policy.

use super::*;
use std::time::Instant;

// Each fixture owns the same executor/queue pump as the application, without native windows.
struct RoutingHost {
    network: NetworkState,
    host: crate::host::Host,
    catalog: Arc<Catalog>,
    session: RemoteSession,
    events: Vec<HostEvent>,
}

impl RoutingHost {
    // A relay advertises the destination's session while retaining its own unused local host.
    fn new(identity: &str, advertised: Option<HostSnapshot>) -> Self {
        let catalog = Arc::new(
            crate::catalog::load_local(
                &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("sample-packs/podcast-pack"),
            )
            .unwrap(),
        );
        let host = crate::host::Host::new(identity.into(), catalog.clone());
        let mut network =
            NetworkState::new(host.snapshot(), &NetworkConfig::default(), Arc::new(|| {}));
        network.administration = sources::Administration::for_test();
        if let Some(snapshot) = advertised {
            network.destination = crate::app::Destination::Remote;
            network.snapshots.send_replace(snapshot.clone());
            network.snapshot = Some(snapshot);
        }
        let session = network
            .runtime
            .block_on(transport::enable_test_loopback_with_registry(
                network.bridge.clone(),
                RemoteCatalogView::project(&catalog).unwrap(),
                network.sources.clone(),
            ))
            .unwrap();
        network.hosting = true;
        Self {
            network,
            host,
            catalog,
            session,
            events: Vec::new(),
        }
    }

    // Issue credentials through the actual From-source action rather than a test-only bypass.
    fn add_source(&mut self, name: &str) -> (String, SessionCredential) {
        self.network
            .source_action(NetworkAction::AddController { name: name.into() });
        let source = self.network.controllers.last().unwrap();
        (
            source.id.clone(),
            SessionCredential::from_secret(source.pairing_secret.clone().unwrap()).unwrap(),
        )
    }

    // Own a real authenticated destination socket and the production controller actor.
    fn route_to(&mut self, destination: &mut Self) {
        let (_, secret) = destination.add_source("relay");
        let (connecting, mut mailbox) =
            TailscaleControllerBackend::new().begin_connect("relay-local".into());
        self.network.controller = Some(connecting.connected(&destination.host.snapshot()));
        self.network.connected = true;
        let address = destination.session.address;
        let updates = self.network.update_tx.clone();
        let wake = self.network.wake.clone();
        self.network.client_job = Some(self.network.runtime.spawn(async move {
            let client = ControllerClient::connect_loopback(address, secret)
                .await
                .unwrap();
            controller_connected(
                0,
                client,
                &mut mailbox.normal,
                &mut mailbox.priority,
                &updates,
                &wake,
            )
            .await
            .unwrap();
        }));
    }

    // Apply the production ingress and outcome pump; preserve published events for assertions.
    fn pump(&mut self) {
        while let Ok(update) = self.network.updates.try_recv() {
            match update {
                Update::Remote(request) => self.network.admit_remote(request),
                Update::Snapshot(_, snapshot)
                    if self.network.destination == crate::app::Destination::Remote =>
                {
                    self.network.snapshots.send_replace(snapshot.clone());
                    self.network.snapshot = Some(snapshot);
                }
                _ => {}
            }
        }
        self.network
            .accept_pending(&mut self.host, self.session.session_id.as_str());
        let events = self.host.drain_events();
        self.network.publish_local(self.host.snapshot(), &events);
        self.events.extend(events);
    }

    // Capture the real transport request before application admission for revocation races.
    fn take_remote(&mut self) -> Option<RemoteCommand> {
        while let Ok(update) = self.network.updates.try_recv() {
            if let Update::Remote(request) = update {
                return Some(request);
            }
        }
        None
    }

    // Keep all waits bounded and let the background runtime process socket notifications.
    fn wait_until(&mut self, mut done: impl FnMut(&mut Self) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.pump();
            if done(self) {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "routing operation did not finish"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

// A source task owns its client for the complete request so no socket is accidentally canceled.
fn send_command(
    host: &RoutingHost,
    secret: SessionCredential,
    id: &str,
    command: PlaybackCommand,
) -> tokio::task::JoinHandle<Result<CommandOutcome, transport::ControllerError>> {
    let address = host.session.address;
    let revision = host.catalog.revision.clone();
    let id = id.to_owned();
    host.network.runtime.spawn(async move {
        let mut client = ControllerClient::connect_loopback(address, secret).await?;
        client.command(id, revision, command).await
    })
}

#[test]
fn local_and_two_remote_sources_share_policy_without_identity_collisions() {
    let mut host = RoutingHost::new("three-sources", None);
    let (alice_id, alice_secret) = host.add_source("Alice");
    let (_, bob_secret) = host.add_source("Bob");
    host.network
        .queue_local(Envelope {
            protocol_version: crate::commands::PROTOCOL_VERSION,
            session_id: host.session.session_id.clone(),
            command_id: "same".into(),
            catalog_revision: host.catalog.revision.clone(),
            command: PlaybackCommand::StopAll,
        })
        .unwrap();
    let alice = send_command(&host, alice_secret, "same", PlaybackCommand::StopAll);
    let bob = send_command(&host, bob_secret, "same", PlaybackCommand::StopAll);
    host.wait_until(|_| alice.is_finished() && bob.is_finished());
    assert!(matches!(
        host.network.runtime.block_on(alice).unwrap().unwrap(),
        CommandOutcome::Accepted { .. }
    ));
    assert!(matches!(
        host.network.runtime.block_on(bob).unwrap().unwrap(),
        CommandOutcome::Accepted { .. }
    ));
    let identities: BTreeSet<_> = host
        .events
        .iter()
        .filter_map(|event| match &event.event {
            crate::commands::EventKind::Accepted { command_id } => Some(command_id),
            _ => None,
        })
        .collect();
    assert_eq!(
        identities.len(),
        3,
        "local and each controller require independent dedupe identities"
    );
    host.network
        .source_action(NetworkAction::SetControllerEnabled {
            id: alice_id,
            enabled: false,
        });
    assert!(host.network.local_enabled);
    assert_eq!(
        host.network
            .controllers
            .iter()
            .filter(|source| source.enabled)
            .count(),
        1
    );
}

#[test]
fn disabling_a_source_cancels_already_admitted_work_and_preserves_other_inputs() {
    let mut host = RoutingHost::new("revocation", None);
    let (alice_id, alice_secret) = host.add_source("Alice");
    let (_, bob_secret) = host.add_source("Bob");
    let alice = send_command(
        &host,
        alice_secret,
        "pending",
        PlaybackCommand::Play {
            clip_id: host.catalog.clips[0].identity.clone(),
        },
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let queued = loop {
        if let Some(request) = host.take_remote() {
            break request;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    host.network.admit_remote(queued);
    assert_eq!(host.network.pending.len(), 1);
    host.network
        .source_action(NetworkAction::SetControllerEnabled {
            id: alice_id.clone(),
            enabled: false,
        });
    host.pump();
    assert!(
        host.host.snapshot().instances.is_empty(),
        "permission must survive the shared ingress guard"
    );
    host.network
        .source_action(NetworkAction::SetControllerEnabled {
            id: alice_id.clone(),
            enabled: true,
        });
    host.network
        .source_action(NetworkAction::RevokeController { id: alice_id });
    let bob = send_command(&host, bob_secret, "survives", PlaybackCommand::StopAll);
    host.wait_until(|_| alice.is_finished() && bob.is_finished());
    assert!(host.network.runtime.block_on(alice).unwrap().is_err());
    assert!(matches!(
        host.network.runtime.block_on(bob).unwrap().unwrap(),
        CommandOutcome::Accepted { .. }
    ));
    assert!(host.network.local_enabled);
}

#[test]
fn relay_returns_destination_rejection_and_preserves_deduplication() {
    let mut destination = RoutingHost::new("destination", None);
    let mut relay = RoutingHost::new("relay-local", Some(destination.host.snapshot()));
    relay.route_to(&mut destination);
    let (_, secret) = relay.add_source("remote input");
    let address = relay.session.address;
    let revision = relay.catalog.revision.clone();
    let clip = relay.catalog.clips[0].identity.clone();
    let task = relay.network.runtime.spawn(async move {
        let mut client = ControllerClient::connect_loopback(address, secret)
            .await
            .unwrap();
        assert_eq!(
            client
                .command(
                    "reject".into(),
                    revision.clone(),
                    PlaybackCommand::Play {
                        clip_id: "not-in-destination".into()
                    }
                )
                .await
                .unwrap(),
            CommandOutcome::Rejected {
                reason: Rejection::UnknownClip
            }
        );
        let first = client
            .command(
                "repeat".into(),
                revision.clone(),
                PlaybackCommand::Play {
                    clip_id: clip.clone(),
                },
            )
            .await
            .unwrap();
        let duplicate = client
            .command(
                "repeat".into(),
                revision,
                PlaybackCommand::Play { clip_id: clip },
            )
            .await
            .unwrap();
        assert_eq!(first, duplicate);
        assert!(matches!(first, CommandOutcome::Accepted { .. }));
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    while !task.is_finished() {
        relay.pump();
        destination.pump();
        assert!(
            Instant::now() < deadline,
            "relay did not return destination outcome"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    relay.network.runtime.block_on(task).unwrap();
    assert_eq!(destination.host.snapshot().instances.len(), 1);
    assert!(
        relay.host.snapshot().instances.is_empty(),
        "selected remote To must not play on this Mac"
    );
}

#[test]
fn in_flight_relay_revocation_cancels_destination_queue_and_another_source_survives() {
    let mut destination = RoutingHost::new("destination-cancel", None);
    let mut relay = RoutingHost::new("relay-local-cancel", Some(destination.host.snapshot()));
    relay.route_to(&mut destination);
    let (alice_id, alice_secret) = relay.add_source("Alice");
    let (_, bob_secret) = relay.add_source("Bob");
    let placement = VisualPlacement {
        width_bps: 4200,
        x_bps: 1234,
        y_bps: 8765,
        fit: durop_protocol::VisualFit::Catalog,
    };
    let visual = relay
        .catalog
        .clips
        .iter()
        .find(|clip| crate::host::Host::default_placement(clip).is_some())
        .unwrap();
    let alice = send_command(
        &relay,
        alice_secret,
        "cancel-in-flight",
        PlaybackCommand::PlayVisual {
            clip_id: visual.identity.clone(),
            placement,
        },
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let queued_at_destination = loop {
        relay.pump();
        if let Some(request) = destination.take_remote() {
            break request;
        }
        assert!(
            Instant::now() < deadline,
            "forwarded request did not reach destination transport"
        );
        std::thread::sleep(Duration::from_millis(1));
    };
    assert!(queued_at_destination.is_active());
    assert!(matches!(queued_at_destination.envelope.command,
        PlaybackCommand::PlayVisual { placement: actual, .. } if actual == placement));
    relay
        .network
        .source_action(NetworkAction::RevokeController { id: alice_id });
    let cancel_deadline = Instant::now() + Duration::from_secs(1);
    while queued_at_destination.is_active() {
        relay.pump();
        assert!(
            Instant::now() < cancel_deadline,
            "source revocation must cancel downstream admission before host policy"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    destination.network.admit_remote(queued_at_destination);
    destination.pump();
    assert!(destination.host.snapshot().instances.is_empty());
    let bob = send_command(&relay, bob_secret, "other-source", PlaybackCommand::StopAll);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !bob.is_finished() || !alice.is_finished() {
        relay.pump();
        destination.pump();
        assert!(
            Instant::now() < deadline,
            "cancel must preserve the selected destination connection"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(relay.network.runtime.block_on(alice).unwrap().is_err());
    assert!(matches!(
        relay.network.runtime.block_on(bob).unwrap().unwrap(),
        CommandOutcome::Accepted { .. }
    ));
}

#[test]
fn relay_pushes_destination_snapshot_to_an_idle_source() {
    let mut destination = RoutingHost::new("destination-snapshots", None);
    let mut relay = RoutingHost::new("relay-local-snapshots", Some(destination.host.snapshot()));
    relay.route_to(&mut destination);
    let (_, secret) = relay.add_source("observer");
    let address = relay.session.address;
    let (connected, ready) = std::sync::mpsc::sync_channel(1);
    let observer = relay.network.runtime.spawn(async move {
        let mut client = ControllerClient::connect_loopback(address, secret)
            .await
            .unwrap();
        connected.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if let ControllerUpdate::Snapshot(snapshot) = client.next_update().await.unwrap()
                    && snapshot.sequence > 0
                {
                    assert_eq!(snapshot.session_id, "destination-snapshots");
                    break;
                }
            }
        })
        .await
        .expect("idle upstream source must receive destination snapshots");
    });
    ready.recv_timeout(Duration::from_secs(1)).unwrap();
    destination
        .network
        .queue_local(Envelope {
            protocol_version: crate::commands::PROTOCOL_VERSION,
            session_id: destination.host.session_id().into(),
            command_id: "destination-local-stop".into(),
            catalog_revision: destination.catalog.revision.clone(),
            command: PlaybackCommand::StopAll,
        })
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !observer.is_finished() {
        relay.pump();
        destination.pump();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    relay.network.runtime.block_on(observer).unwrap();
}

#[test]
fn relay_preserves_visual_overrides_and_reset_at_the_destination() {
    let mut destination = RoutingHost::new("destination-visual", None);
    let mut relay = RoutingHost::new("relay-visual", Some(destination.host.snapshot()));
    relay.route_to(&mut destination);
    let (_, secret) = relay.add_source("visual controller");
    let clip_id = relay
        .catalog
        .clips
        .iter()
        .find(|clip| crate::host::Host::default_placement(clip).is_some())
        .unwrap()
        .identity
        .clone();
    let initial = VisualPlacement {
        width_bps: 4200,
        x_bps: 1234,
        y_bps: 8765,
        fit: durop_protocol::VisualFit::Catalog,
    };
    let changed = VisualPlacement {
        width_bps: 7000,
        x_bps: 8000,
        y_bps: 2000,
        fit: durop_protocol::VisualFit::Cover,
    };
    let address = relay.session.address;
    let revision = relay.catalog.revision.clone();
    let task = relay.network.runtime.spawn(async move {
        let mut client = ControllerClient::connect_loopback(address, secret)
            .await
            .unwrap();
        let accepted = client
            .command(
                "play-visual".into(),
                revision.clone(),
                PlaybackCommand::PlayVisual {
                    clip_id,
                    placement: initial,
                },
            )
            .await
            .unwrap();
        let CommandOutcome::Accepted {
            instance_id: Some(instance_id),
        } = accepted
        else {
            panic!("destination must accept visual playback");
        };
        for (index, placement) in [Some(changed), None].into_iter().enumerate() {
            assert_eq!(
                client
                    .command(
                        format!("update-{index}"),
                        revision.clone(),
                        PlaybackCommand::SetVisual {
                            instance_id: instance_id.clone(),
                            placement
                        }
                    )
                    .await
                    .unwrap(),
                CommandOutcome::Accepted {
                    instance_id: Some(instance_id.clone())
                }
            );
        }
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    while !task.is_finished() {
        relay.pump();
        destination.pump();
        assert!(
            Instant::now() < deadline,
            "relay did not return visual outcomes"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    relay.network.runtime.block_on(task).unwrap();
    let snapshot = destination.host.snapshot();
    assert_eq!(snapshot.instances.len(), 1);
    assert_eq!(snapshot.instances[0].placement, None);
    assert_eq!(
        snapshot.instances[0].state,
        crate::commands::InstanceState::Preparing
    );
    let updates: Vec<_> = destination
        .events
        .iter()
        .filter_map(|event| {
            if let crate::commands::EventKind::VisualChanged { placement } = event.event {
                Some(placement)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(updates, vec![Some(initial), Some(changed), None]);
    assert!(
        relay.host.snapshot().instances.is_empty(),
        "visual controls must use the selected destination"
    );
}
