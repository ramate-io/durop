use super::*;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error, client::IntoClientRequest},
};
type Client = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

struct Fixture {
    session: RemoteSession,
    commands: mpsc::Receiver<RemoteCommand>,
    emergency: mpsc::Receiver<RemoteCommand>,
    snapshot: watch::Sender<HostSnapshot>,
    events: broadcast::Sender<HostEvent>,
}
async fn fixture() -> Fixture {
    fixture_with_registry(None).await
}

// Optional registry makes legacy and per-source modes share the same real socket fixture.
async fn fixture_with_registry(registry: Option<SourceAccessRegistry>) -> Fixture {
    let (commands, commands_rx) = mpsc::channel(1);
    let (emergency, emergency_rx) = mpsc::channel(1);
    let (snapshot, snapshot_rx) = watch::channel(HostSnapshot {
        session_id: "test-session".into(),
        catalog_revision: "abc".into(),
        sequence: 0,
        clips: vec![],
        instances: vec![],
    });
    let (events, _) = broadcast::channel(4);
    let catalog = RemoteCatalog::new(
        serde_json::json!({"clips":[{"id":"applause"}]}),
        BTreeMap::from([(
            "applause".into(),
            include_bytes!("../../../assets/overlay-proof.png").to_vec(),
        )]),
    )
    .unwrap();
    // The loopback bypass exists only under cfg(test); production enable rejects it.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let session = start_with_access(
        listener,
        vec!["https://panel.example".into()],
        HostBridge {
            commands,
            emergency,
            snapshot: snapshot_rx,
            events: events.clone(),
        },
        catalog,
        BindingGuard::Loopback,
        registry,
    )
    .unwrap();
    Fixture {
        session,
        commands: commands_rx,
        emergency: emergency_rx,
        snapshot,
        events,
    }
}
fn request(session: &RemoteSession) -> tokio_tungstenite::tungstenite::http::Request<()> {
    let mut request = format!("ws://{}/control", session.address)
        .into_client_request()
        .unwrap();
    request.headers_mut().insert(
        "authorization",
        format!("Bearer {}", session.credential.bearer())
            .parse()
            .unwrap(),
    );
    request
        .headers_mut()
        .insert("sec-websocket-protocol", SUBPROTOCOL.parse().unwrap());
    request
}
async fn connect(session: &RemoteSession) -> Client {
    let (mut client, response) = connect_async(request(session)).await.unwrap();
    assert_eq!(response.headers()["sec-websocket-protocol"], SUBPROTOCOL);
    assert_eq!(receive(&mut client).await["type"], "snapshot");
    client
}
async fn receive(client: &mut Client) -> Value {
    let message = timeout(Duration::from_secs(2), client.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    serde_json::from_str(message.to_text().unwrap()).unwrap()
}
async fn send_json(client: &mut Client, value: Value) {
    client
        .send(Message::Text(value.to_string().into()))
        .await
        .unwrap();
}
fn command(id: &str, kind: Value) -> Value {
    serde_json::json!({"type":"command","envelope":{"protocolVersion":PROTOCOL_VERSION,"sessionId":"test-session","commandId":id,"catalogRevision":"abc","command":kind}})
}
async fn disconnected(client: &mut Client) {
    let next = timeout(Duration::from_secs(2), client.next())
        .await
        .expect("connection should terminate");
    assert!(matches!(
        next,
        None | Some(Err(_)) | Some(Ok(Message::Close(_)))
    ));
}

#[tokio::test]
async fn authentication_path_origin_and_version_are_required() {
    let mut fixture = fixture().await;
    for scenario in 0..6 {
        let mut request = request(&fixture.session);
        match scenario {
            0 => {
                request.headers_mut().remove("authorization");
            }
            1 => {
                request
                    .headers_mut()
                    .insert("authorization", "Bearer wrong".parse().unwrap());
            }
            2 => {
                request
                    .headers_mut()
                    .insert("origin", "https://evil.example".parse().unwrap());
            }
            3 => {
                *request.uri_mut() =
                    format!("ws://{}/control?token=wrong", fixture.session.address)
                        .parse()
                        .unwrap();
            }
            4 => {
                request
                    .headers_mut()
                    .insert("sec-websocket-protocol", "durop.v1".parse().unwrap());
            }
            5 => {
                request
                    .headers_mut()
                    .append("authorization", "Bearer duplicate".parse().unwrap());
            }
            _ => unreachable!(),
        }
        assert!(
            matches!(connect_async(request).await, Err(Error::Http(response)) if response.status() == StatusCode::FORBIDDEN)
        );
    }
    assert!(fixture.commands.try_recv().is_err());
    let mut request = request(&fixture.session);
    request
        .headers_mut()
        .insert("origin", "https://panel.example".parse().unwrap());
    let (mut client, _) = connect_async(request).await.unwrap();
    assert_eq!(
        receive(&mut client).await["snapshot"]["sessionId"],
        "test-session"
    );
    fixture.session.close().await;
}

#[tokio::test]
async fn catalog_icons_and_reconnect_snapshot_are_scoped() {
    let fixture = fixture().await;
    let mut client = connect(&fixture.session).await;
    send_json(&mut client, serde_json::json!({"type":"catalog"})).await;
    assert_eq!(
        receive(&mut client).await["catalog"]["clips"][0]["id"],
        "applause"
    );
    send_json(
        &mut client,
        serde_json::json!({"type":"icon","clip_id":"applause"}),
    )
    .await;
    assert_eq!(receive(&mut client).await["png"][0], 137);
    send_json(
        &mut client,
        serde_json::json!({"type":"icon","clip_id":"../../etc/passwd"}),
    )
    .await;
    assert_eq!(receive(&mut client).await["code"], "unknown_icon");
    client.close(None).await.unwrap();
    fixture
        .snapshot
        .send_modify(|snapshot| snapshot.sequence = 42);
    let (mut reconnected, _) = connect_async(request(&fixture.session)).await.unwrap();
    assert_eq!(receive(&mut reconnected).await["snapshot"]["sequence"], 42);
    fixture.session.close().await;
}

#[tokio::test]
async fn commands_reply_and_emergency_lane_bypasses_saturation() {
    let mut fixture = fixture().await;
    let mut first = connect(&fixture.session).await;
    send_json(
        &mut first,
        command(
            "play",
            serde_json::json!({"type":"play","clip_id":"applause"}),
        ),
    )
    .await;
    let mut second = connect(&fixture.session).await;
    send_json(
        &mut second,
        command(
            "full",
            serde_json::json!({"type":"play","clip_id":"applause"}),
        ),
    )
    .await;
    assert_eq!(receive(&mut second).await["code"], "busy");
    send_json(
        &mut first,
        command("stop", serde_json::json!({"type":"stop_all"})),
    )
    .await;
    let stop = timeout(Duration::from_secs(2), fixture.emergency.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(stop.is_active());
    assert_eq!(stop.envelope.command_id, "stop");
    let play = fixture.commands.recv().await.unwrap();
    assert!(
        !play.is_active(),
        "superseded play must never reach playback"
    );
    stop.reply
        .send(CommandOutcome::Accepted { instance_id: None })
        .unwrap();
    let response = receive(&mut first).await;
    assert_eq!(response["command_id"], "stop");
    assert_eq!(response["outcome"]["type"], "accepted");
    fixture.session.close().await;
}

#[tokio::test]
async fn disconnect_cancels_pending_command_and_revoke_closes_all_access() {
    let mut fixture = fixture().await;
    let mut client = connect(&fixture.session).await;
    send_json(
        &mut client,
        command(
            "play",
            serde_json::json!({"type":"play","clip_id":"applause"}),
        ),
    )
    .await;
    let queued = timeout(Duration::from_secs(2), fixture.commands.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(queued.is_active());
    client.close(None).await.unwrap();
    timeout(Duration::from_secs(2), async {
        while queued.is_active() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let mut other = connect(&fixture.session).await;
    send_json(
        &mut other,
        command(
            "pending-revoke",
            serde_json::json!({"type":"play","clip_id":"applause"}),
        ),
    )
    .await;
    let revoked_command = timeout(Duration::from_secs(2), fixture.commands.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(revoked_command.is_active());
    fixture.session.revoke();
    assert!(!revoked_command.is_active());
    assert!(!queued.is_active());
    disconnected(&mut other).await;
    assert!(connect_async(request(&fixture.session)).await.is_err());
    fixture.session.close().await;
}

#[tokio::test]
async fn malformed_oversized_and_wrong_identity_never_reach_host() {
    let mut fixture = fixture().await;
    let mut wrong_version = command("version", serde_json::json!({"type":"stop_all"}));
    wrong_version["envelope"]["protocolVersion"] = (PROTOCOL_VERSION + 1).into();
    let mut wrong_session = command("session", serde_json::json!({"type":"stop_all"}));
    wrong_session["envelope"]["sessionId"] = "stale".into();
    for text in [
        "not-json".to_string(),
        "x".repeat(MAX_INPUT + 1),
        wrong_version.to_string(),
        wrong_session.to_string(),
        "{\"type\":\"snapshot\",\"surprise\":true}".into(),
    ] {
        let mut client = connect(&fixture.session).await;
        client.send(Message::Text(text.into())).await.unwrap();
        disconnected(&mut client).await;
    }
    assert!(fixture.commands.try_recv().is_err());
    assert!(fixture.emergency.try_recv().is_err());
    fixture.session.close().await;
}

#[tokio::test]
async fn connection_count_and_request_rate_are_bounded() {
    let fixture = fixture().await;
    let mut clients = Vec::new();
    for _ in 0..MAX_CONNECTIONS {
        clients.push(connect(&fixture.session).await);
    }
    assert!(
        timeout(
            Duration::from_secs(2),
            connect_async(request(&fixture.session))
        )
        .await
        .unwrap()
        .is_err()
    );
    let client = &mut clients[0];
    for _ in 0..=REQUESTS_PER_SECOND {
        send_json(client, serde_json::json!({"type":"snapshot"})).await;
    }
    for _ in 0..REQUESTS_PER_SECOND {
        assert_eq!(receive(client).await["type"], "snapshot");
    }
    disconnected(client).await;
    fixture.session.close().await;
}

#[tokio::test]
async fn production_rejects_loopback_wildcard_lan_and_unassigned_tailnet() {
    for address in [
        "127.0.0.1:48732",
        "0.0.0.0:48732",
        "192.168.0.10:48732",
        "[::]:48732",
    ] {
        let config = SessionConfig {
            address: address.parse().unwrap(),
            allowed_origins: vec![],
        };
        assert!(matches!(
            config.validate(),
            Err(TransportError::InvalidConfiguration)
        ));
    }
    assert!(matches!(
        verify_tailnet_address("100.127.255.254".parse().unwrap()).await,
        Err(TransportError::TailnetUnavailable)
    ));
    assert!(is_tailnet_ip("fd7a:115c:a1e0::1".parse().unwrap()));
    assert!(!is_tailnet_ip("fd7a:115c:a1e1::1".parse().unwrap()));
    let token = SessionCredential(random_hex::<32>().unwrap());
    assert_eq!(token.bearer().len(), 64);
    assert!(!format!("{token:?}").contains(token.bearer()));
}

fn pairing_secret(session: &RemoteSession) -> SessionCredential {
    SessionCredential::from_secret(session.credential.bearer().to_string()).unwrap()
}

#[test]
fn png_icon_dimensions_are_bounded_before_decode() {
    let mut png = include_bytes!("../../../assets/overlay-proof.png").to_vec();
    assert!(valid_icon_png(&png));
    png[16..20].copy_from_slice(&65536u32.to_be_bytes());
    assert!(!valid_icon_png(&png));
    assert!(matches!(
        RemoteCatalog::new(
            serde_json::json!({}),
            BTreeMap::from([("huge".into(), png)])
        ),
        Err(TransportError::InvalidCatalog)
    ));
    assert!(!valid_icon_png(b"\x89PNG\r\n\x1a\n"));
    let mut oversized = include_bytes!("../../../assets/overlay-proof.png").to_vec();
    oversized.resize(MAX_ICON_PNG + 1, 0);
    assert!(!valid_icon_png(&oversized));
}

#[test]
fn local_tailnet_ip_prefers_assigned_ipv4() {
    let status = serde_json::json!({
        "BackendState": "Running",
        "Self": {
            "TailscaleIPs": ["fd7a:115c:a1e0::1", "100.75.38.86", "192.168.1.10"]
        }
    });
    let interfaces = [
        "fd7a:115c:a1e0::1".parse().unwrap(),
        "100.75.38.86".parse().unwrap(),
        "192.168.1.10".parse().unwrap(),
    ];
    assert_eq!(
        super::select_local_tailnet_ip(&status, &interfaces).unwrap(),
        "100.75.38.86".parse::<std::net::IpAddr>().unwrap()
    );
    assert!(
        super::select_local_tailnet_ip(
            &serde_json::json!({"BackendState": "Stopped"}),
            &interfaces
        )
        .is_none()
    );
    assert!(super::select_local_tailnet_ip(&status, &["10.0.0.1".parse().unwrap()]).is_none());
}

#[tokio::test]
async fn aborted_listener_task_revokes_observable_session_state() {
    let fixture = fixture().await;
    let mut client = connect(&fixture.session).await;
    fixture.session.task.abort();
    timeout(Duration::from_secs(2), async {
        while fixture.session.is_active() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    disconnected(&mut client).await;
    fixture.session.close().await;
}

#[tokio::test]
async fn production_client_browses_and_uses_canonical_command_replies() {
    let mut fixture = fixture().await;
    assert!(matches!(
        ControllerClient::connect(fixture.session.address, pairing_secret(&fixture.session)).await,
        Err(ControllerError::InvalidTarget)
    ));
    assert!(SessionCredential::from_secret("URL-token-or-invalid-secret".into()).is_err());
    assert!(matches!(
        ControllerClient::connect_loopback(
            fixture.session.address,
            SessionCredential::from_secret("f".repeat(64)).unwrap()
        )
        .await,
        Err(ControllerError::ConnectionDenied)
    ));
    let mut client = ControllerClient::connect_loopback(
        fixture.session.address,
        pairing_secret(&fixture.session),
    )
    .await
    .unwrap();
    assert_eq!(client.current_snapshot().session_id, "test-session");
    assert_eq!(
        client.catalog().await.unwrap()["clips"][0]["id"],
        "applause"
    );
    assert!(
        client
            .icon("applause")
            .await
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
    assert_eq!(
        client.icon("../../private").await,
        Err(ControllerError::UnknownIcon)
    );
    assert!(client.is_connected());
    let (outcome, ()) = tokio::join!(
        client.command(
            "typed-command".into(),
            "abc".into(),
            PlaybackCommand::Play {
                clip_id: "applause".into()
            }
        ),
        async {
            let command = fixture.commands.recv().await.unwrap();
            assert!(command.is_active());
            assert_eq!(command.envelope.command_id, "typed-command");
            assert_eq!(command.envelope.catalog_revision, "abc");
            command
                .reply
                .send(CommandOutcome::Accepted {
                    instance_id: Some("instance-1".into()),
                })
                .unwrap();
        }
    );
    assert_eq!(
        outcome.unwrap(),
        CommandOutcome::Accepted {
            instance_id: Some("instance-1".into())
        }
    );
    client.disconnect().await;
    assert_eq!(
        client
            .command("offline".into(), "abc".into(), PlaybackCommand::StopAll)
            .await,
        Err(ControllerError::Disconnected)
    );
    assert!(fixture.emergency.try_recv().is_err());
    fixture.session.close().await;
}

#[tokio::test]
async fn production_client_recovers_event_gaps_and_reconnects_without_replay() {
    use durop_protocol::EventKind;
    let mut fixture = fixture().await;
    let mut client = ControllerClient::connect_loopback(
        fixture.session.address,
        pairing_secret(&fixture.session),
    )
    .await
    .unwrap();
    fixture
        .snapshot
        .send_modify(|snapshot| snapshot.sequence = 2);
    fixture
        .events
        .send(HostEvent {
            sequence: 2,
            instance_id: None,
            event: EventKind::Stopped,
        })
        .unwrap();
    let update = timeout(Duration::from_secs(2), client.next_update())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(update, ControllerUpdate::Snapshot(snapshot) if snapshot.sequence == 2));
    fixture
        .events
        .send(HostEvent {
            sequence: 3,
            instance_id: None,
            event: EventKind::Finished,
        })
        .unwrap();
    assert!(
        matches!(client.next_update().await.unwrap(), ControllerUpdate::Event(event) if event.sequence == 3)
    );
    client.disconnect().await;
    fixture
        .snapshot
        .send_modify(|snapshot| snapshot.sequence = 7);
    let mut reconnect = ControllerClient::connect_loopback(
        fixture.session.address,
        pairing_secret(&fixture.session),
    )
    .await
    .unwrap();
    assert_eq!(reconnect.current_snapshot().sequence, 7);
    assert!(fixture.commands.try_recv().is_err());
    assert!(fixture.emergency.try_recv().is_err());
    fixture.session.revoke();
    assert!(reconnect.next_update().await.is_err());
    assert!(!reconnect.is_connected());
    assert_eq!(
        reconnect.catalog().await,
        Err(ControllerError::Disconnected)
    );
    fixture.session.close().await;
}

#[tokio::test]
async fn production_client_pending_timeout_cancels_without_replay() {
    let mut fixture = fixture().await;
    let mut client = ControllerClient::connect_loopback(
        fixture.session.address,
        pairing_secret(&fixture.session),
    )
    .await
    .unwrap();
    let (outcome, queued) = tokio::join!(
        client.command(
            "never-replied".into(),
            "abc".into(),
            PlaybackCommand::Play {
                clip_id: "applause".into()
            }
        ),
        async { fixture.commands.recv().await.unwrap() }
    );
    assert!(outcome.is_err());
    assert!(!queued.is_active());
    assert!(!client.is_connected());
    assert!(fixture.commands.try_recv().is_err());
    fixture.session.close().await;
}

#[tokio::test]
async fn production_client_rejects_snapshot_behind_observed_event() {
    let fixture = fixture().await;
    let mut client = ControllerClient::connect_loopback(
        fixture.session.address,
        pairing_secret(&fixture.session),
    )
    .await
    .unwrap();
    fixture
        .events
        .send(HostEvent {
            sequence: 2,
            instance_id: None,
            event: durop_protocol::EventKind::Stopped,
        })
        .unwrap();
    assert!(matches!(
        client.next_update().await,
        Err(ControllerError::Protocol)
    ));
    assert!(!client.is_connected());
    fixture.session.close().await;
}

// Authenticate using an independently issued source secret, never the compatibility token.
async fn connect_source(session: &RemoteSession, credential: &SessionCredential) -> Client {
    let mut request = request(session);
    request.headers_mut().insert(
        "authorization",
        format!("Bearer {}", credential.bearer()).parse().unwrap(),
    );
    let (mut client, _) = connect_async(request).await.unwrap();
    assert_eq!(receive(&mut client).await["type"], "snapshot");
    client
}

#[tokio::test]
async fn registry_scopes_command_ids_and_revokes_only_one_source() {
    let registry = SourceAccessRegistry::default();
    let a = registry.issue("alice".into()).unwrap();
    let b = registry.issue("bob".into()).unwrap();
    let mut fixture = fixture_with_registry(Some(registry.clone())).await;
    assert!(
        connect_async(request(&fixture.session)).await.is_err(),
        "compatibility credential must not bypass the registry"
    );
    let mut alice = connect_source(&fixture.session, &a).await;
    let mut bob = connect_source(&fixture.session, &b).await;
    send_json(
        &mut alice,
        command("same", serde_json::json!({"type":"stop_all"})),
    )
    .await;
    let alice_command = fixture.emergency.recv().await.unwrap();
    send_json(
        &mut bob,
        command("same", serde_json::json!({"type":"stop_all"})),
    )
    .await;
    let bob_command = fixture.emergency.recv().await.unwrap();
    assert_ne!(
        alice_command.envelope.command_id,
        bob_command.envelope.command_id
    );
    assert_eq!(alice_command.source_id.as_deref(), Some("alice"));
    assert_eq!(bob_command.source_id.as_deref(), Some("bob"));
    assert!(registry.set_enabled("alice", false));
    assert!(
        !alice_command.is_active(),
        "revocation is synchronous for host admission"
    );
    assert!(bob_command.is_active());
    disconnected(&mut alice).await;
    bob_command
        .reply
        .send(CommandOutcome::Accepted { instance_id: None })
        .unwrap();
    assert_eq!(
        receive(&mut bob).await["command_id"],
        "same",
        "wire reply uses original correlation identity"
    );
    send_json(&mut bob, serde_json::json!({"type":"catalog"})).await;
    assert_eq!(receive(&mut bob).await["type"], "catalog");
    assert!(registry.set_enabled("alice", true));
    assert!(
        !alice_command.is_active(),
        "re-enabling never revives a queued request"
    );
    let mut alice_again = connect_source(&fixture.session, &a).await;
    assert!(registry.revoke("alice"));
    disconnected(&mut alice_again).await;
    send_json(&mut bob, serde_json::json!({"type":"snapshot"})).await;
    assert_eq!(receive(&mut bob).await["type"], "snapshot");
    fixture.session.close().await;
}

#[tokio::test]
async fn guarded_submission_cancels_destination_admission_and_reuses_connection() {
    let mut fixture = fixture().await;
    let mut client = ControllerClient::connect_loopback(
        fixture.session.address,
        pairing_secret(&fixture.session),
    )
    .await
    .unwrap();
    let active = Arc::new(AtomicBool::new(true));
    let guard = || active.load(Ordering::Acquire);
    let envelope = Envelope {
        protocol_version: PROTOCOL_VERSION,
        session_id: "test-session".into(),
        command_id: "cancel-me".into(),
        catalog_revision: "abc".into(),
        command: PlaybackCommand::Play {
            clip_id: "applause".into(),
        },
    };
    let (outcome, ()) = tokio::join!(client.submit_guarded_envelope(envelope, &guard), async {
        let queued = fixture.commands.recv().await.unwrap();
        assert!(queued.is_active());
        active.store(false, Ordering::Release);
        timeout(Duration::from_secs(1), async {
            while queued.is_active() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("cancellation must invalidate the destination queue");
    });
    assert_eq!(outcome.unwrap(), None);
    assert!(client.is_connected());
    assert!(
        client.catalog().await.is_ok(),
        "cancel barrier must leave a usable socket"
    );
    client.disconnect().await;
    fixture.session.close().await;
}

#[tokio::test]
async fn cancel_is_scoped_to_its_connection_and_snapshot_changes_are_pushed() {
    let mut fixture = fixture().await;
    let mut first = connect(&fixture.session).await;
    let mut second = connect(&fixture.session).await;
    send_json(
        &mut first,
        command(
            "same",
            serde_json::json!({"type":"play","clip_id":"applause"}),
        ),
    )
    .await;
    let first_queued = fixture.commands.recv().await.unwrap();
    send_json(
        &mut second,
        command(
            "same",
            serde_json::json!({"type":"play","clip_id":"applause"}),
        ),
    )
    .await;
    let second_queued = fixture.commands.recv().await.unwrap();
    send_json(
        &mut first,
        serde_json::json!({"type":"cancel","command_id":"same"}),
    )
    .await;
    assert_eq!(receive(&mut first).await["type"], "canceled");
    assert!(!first_queued.is_active());
    assert!(
        second_queued.is_active(),
        "same wire ID on another connection must survive"
    );
    fixture
        .snapshot
        .send_modify(|snapshot| snapshot.sequence = 42);
    assert_eq!(receive(&mut first).await["snapshot"]["sequence"], 42);
    assert_eq!(receive(&mut second).await["snapshot"]["sequence"], 42);
    fixture.session.close().await;
}
