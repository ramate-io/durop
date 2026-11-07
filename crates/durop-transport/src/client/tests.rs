use super::*;

#[tokio::test]
async fn canonical_envelope_and_reply_observer_have_no_application_dependency() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let (listener, address) = fake_host().await;
    let envelope = Envelope {
        protocol_version: PROTOCOL_VERSION,
        session_id: "session".into(),
        command_id: "preserved-id".into(),
        catalog_revision: "caller-supplied-revision".into(),
        command: PlaybackCommand::StopAll,
    };
    let expected = envelope.clone();
    let server = tokio::spawn(async move {
        let mut socket = accept(listener).await;
        let request = socket.next().await.unwrap().unwrap();
        let value: Value = serde_json::from_str(request.to_text().unwrap()).unwrap();
        assert_eq!(value["envelope"], serde_json::to_value(expected).unwrap());
        socket.send(Message::Text(serde_json::json!({"type":"outcome","command_id":"preserved-id","outcome":CommandOutcome::Rejected { reason: durop_protocol::Rejection::StaleCatalog }}).to_string().into())).await.unwrap();
        let _ = socket.next().await;
    });
    let mut client = ControllerClient::connect_loopback(
        address,
        SessionCredential::from_secret("a".repeat(64)).unwrap(),
    )
    .await
    .unwrap();
    let observations = Arc::new(AtomicUsize::new(0));
    let counted = observations.clone();
    client.observe_acknowledgments(move |_| {
        counted.fetch_add(1, Ordering::Relaxed);
    });
    let mut stale = envelope.clone();
    stale.session_id = "old-session".into();
    assert_eq!(
        client.submit_envelope(stale).await,
        Err(ControllerError::Protocol)
    );
    assert_eq!(observations.load(Ordering::Relaxed), 0);
    assert_eq!(
        client.submit_envelope(envelope).await.unwrap(),
        CommandOutcome::Rejected {
            reason: durop_protocol::Rejection::StaleCatalog
        }
    );
    assert_eq!(observations.load(Ordering::Relaxed), 1);
    client.disconnect().await;
    server.await.unwrap();
}

fn snapshot(sequence: u64) -> HostSnapshot {
    HostSnapshot {
        session_id: "session".into(),
        catalog_revision: "revision".into(),
        sequence,
        clips: vec![],
        instances: vec![],
    }
}

async fn fake_host() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    (listener, address)
}

async fn accept(listener: TcpListener) -> tokio_tungstenite::WebSocketStream<TcpStream> {
    let (tcp, _) = listener.accept().await.unwrap();
    #[allow(clippy::result_large_err)]
    let mut socket = accept_hdr_async_with_config(
        tcp,
        |_: &Request, mut response: Response| {
            response
                .headers_mut()
                .insert("sec-websocket-protocol", SUBPROTOCOL.parse().unwrap());
            Ok(response)
        },
        None,
    )
    .await
    .unwrap();
    socket
        .send(Message::Text(
            serde_json::json!({"type":"snapshot","snapshot":snapshot(0)})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    socket
}

#[tokio::test]
async fn silent_heartbeat_blackhole_expires_connection() {
    let (listener, address) = fake_host().await;
    let (release, released) = oneshot::channel();
    let server = tokio::spawn(async move {
        let _socket = accept(listener).await;
        let _ = released.await;
    });
    let mut client = ControllerClient::connect_loopback(
        address,
        SessionCredential::from_secret("a".repeat(64)).unwrap(),
    )
    .await
    .unwrap();
    client.heartbeat = tokio::time::Instant::now();
    assert!(matches!(
        timeout(Duration::from_secs(7), client.next_update())
            .await
            .unwrap(),
        Err(ControllerError::Timeout)
    ));
    assert!(!client.is_connected());
    let _ = release.send(());
    server.await.unwrap();
}

#[tokio::test]
async fn cancelling_gap_recovery_accepts_late_snapshot_before_next_reply() {
    let (listener, address) = fake_host().await;
    let (requested, observed) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut socket = accept(listener).await;
        socket.send(Message::Text(serde_json::json!({"type":"event","event":HostEvent { sequence: 2, instance_id: None, event: durop_protocol::EventKind::Stopped }}).to_string().into())).await.unwrap();
        let request = socket.next().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(request.to_text().unwrap()).unwrap()["type"],
            "snapshot"
        );
        requested.send(()).unwrap();
        let command = socket.next().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(command.to_text().unwrap()).unwrap()["type"],
            "command"
        );
        socket
            .send(Message::Text(
                serde_json::json!({"type":"snapshot","snapshot":snapshot(2)})
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        socket.send(Message::Text(serde_json::json!({"type":"outcome","command_id":"after-cancel","outcome":CommandOutcome::Accepted { instance_id: None }}).to_string().into())).await.unwrap();
        // Keep the socket alive until the caller closes it.
        let _ = socket.next().await;
    });
    let mut client = ControllerClient::connect_loopback(
        address,
        SessionCredential::from_secret("a".repeat(64)).unwrap(),
    )
    .await
    .unwrap();
    let mut update = Box::pin(client.next_update());
    tokio::select! { _ = &mut update => panic!("snapshot should still be pending"), _ = observed => {} }
    drop(update);
    assert_eq!(
        client
            .command(
                "after-cancel".into(),
                "revision".into(),
                PlaybackCommand::StopAll
            )
            .await
            .unwrap(),
        CommandOutcome::Accepted { instance_id: None }
    );
    assert_eq!(client.current_snapshot().sequence, 2);
    client.disconnect().await;
    server.await.unwrap();
}

#[tokio::test]
async fn visual_commands_and_events_roundtrip_without_losing_reset() {
    use durop_protocol::{EventKind, VisualPlacement};
    let placement = VisualPlacement {
        width_bps: 3200,
        x_bps: 1000,
        y_bps: 9000,
    };
    let commands = [
        PlaybackCommand::PlayVisual {
            clip_id: "pack/clip".into(),
            placement,
        },
        PlaybackCommand::SetVisual {
            instance_id: "session:1".into(),
            placement: Some(placement),
        },
        PlaybackCommand::SetVisual {
            instance_id: "session:1".into(),
            placement: None,
        },
    ];
    let expected = commands.clone();
    let (listener, address) = fake_host().await;
    let server = tokio::spawn(async move {
        let mut socket = accept(listener).await;
        for (index, command) in expected.into_iter().enumerate() {
            let message = socket.next().await.unwrap().unwrap();
            let request: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
            assert_eq!(
                request["envelope"]["command"],
                serde_json::to_value(command).unwrap()
            );
            let event = HostEvent {
                sequence: index as u64 + 1,
                instance_id: Some("session:1".into()),
                event: EventKind::VisualChanged {
                    placement: if index == 2 { None } else { Some(placement) },
                },
            };
            socket
                .send(Message::Text(
                    serde_json::json!({"type":"event","event":event})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            socket.send(Message::Text(serde_json::json!({"type":"outcome","command_id":index.to_string(),"outcome":CommandOutcome::Accepted { instance_id: Some("session:1".into()) }}).to_string().into())).await.unwrap();
        }
        let _ = socket.next().await;
    });
    let mut client = ControllerClient::connect_loopback(
        address,
        SessionCredential::from_secret("a".repeat(64)).unwrap(),
    )
    .await
    .unwrap();
    for (index, command) in commands.into_iter().enumerate() {
        client
            .command(index.to_string(), "revision".into(), command)
            .await
            .unwrap();
        let ControllerUpdate::Event(event) = client.next_update().await.unwrap() else {
            panic!("expected visual event")
        };
        assert_eq!(
            event.event,
            EventKind::VisualChanged {
                placement: if index == 2 { None } else { Some(placement) }
            }
        );
    }
    client.disconnect().await;
    server.await.unwrap();
}

#[tokio::test]
async fn invalid_visual_event_bounds_disconnect_the_controller() {
    let (listener, address) = fake_host().await;
    let server = tokio::spawn(async move {
        let mut socket = accept(listener).await;
        socket.send(Message::Text(serde_json::json!({"type":"event","event":{"sequence":1,"instanceId":"session:1","event":{"type":"visual_changed","placement":{"widthBps":0,"xBps":0,"yBps":0}}}}).to_string().into())).await.unwrap();
        let _ = socket.next().await;
    });
    let mut client = ControllerClient::connect_loopback(
        address,
        SessionCredential::from_secret("a".repeat(64)).unwrap(),
    )
    .await
    .unwrap();
    assert!(matches!(
        client.next_update().await,
        Err(ControllerError::Protocol)
    ));
    assert!(!client.is_connected());
    server.await.unwrap();
}

#[test]
fn invalid_visual_snapshot_bounds_are_rejected_before_state_replacement() {
    let mut snapshot = snapshot(0);
    snapshot.instances.push(durop_protocol::InstanceSnapshot {
        instance_id: "session:1".into(),
        clip_id: "pack/clip".into(),
        generation: 1,
        state: durop_protocol::InstanceState::Playing,
        placement: Some(durop_protocol::VisualPlacement {
            width_bps: 1,
            x_bps: 10_001,
            y_bps: 0,
        }),
    });
    assert!(!Packet::Snapshot { snapshot }.has_valid_placement());
}

#[tokio::test]
async fn cancellation_barrier_drains_a_racing_outcome_before_the_next_request() {
    let (listener, address) = fake_host().await;
    let server = tokio::spawn(async move {
        let mut socket = accept(listener).await;
        let cancel = socket.next().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(cancel.to_text().unwrap()).unwrap()["type"],
            "cancel"
        );
        for packet in [
            serde_json::json!({"type":"outcome","command_id":"abandoned","outcome":CommandOutcome::Accepted { instance_id: None }}),
            serde_json::json!({"type":"canceled","command_id":"abandoned"}),
        ] {
            socket
                .send(Message::Text(packet.to_string().into()))
                .await
                .unwrap();
        }
        let catalog = socket.next().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(catalog.to_text().unwrap()).unwrap()["type"],
            "catalog"
        );
        socket
            .send(Message::Text(
                serde_json::json!({"type":"catalog","catalog":{"clips":[]}})
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        let _ = socket.next().await;
    });
    let mut client = ControllerClient::connect_loopback(
        address,
        SessionCredential::from_secret("a".repeat(64)).unwrap(),
    )
    .await
    .unwrap();
    client
        .roundtrip(
            serde_json::json!({"type":"cancel","command_id":"abandoned"}),
            Expected::Canceled("abandoned".into()),
        )
        .await
        .unwrap();
    assert_eq!(
        client.catalog().await.unwrap(),
        serde_json::json!({"clips":[]})
    );
    client.disconnect().await;
    server.await.unwrap();
}

#[tokio::test]
async fn unsupported_cancellation_fails_closed() {
    let (listener, address) = fake_host().await;
    let active = Arc::new(AtomicBool::new(true));
    let revoke = active.clone();
    let server = tokio::spawn(async move {
        let mut socket = accept(listener).await;
        let command = socket.next().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(command.to_text().unwrap()).unwrap()["type"],
            "command"
        );
        revoke.store(false, Ordering::Release);
        let cancel = socket.next().await.unwrap().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(cancel.to_text().unwrap()).unwrap()["type"],
            "cancel"
        );
        // Older peers reject unknown frames by closing the authenticated connection.
        socket.close(None).await.unwrap();
    });
    let mut client = ControllerClient::connect_loopback(
        address,
        SessionCredential::from_secret("a".repeat(64)).unwrap(),
    )
    .await
    .unwrap();
    let envelope = Envelope {
        protocol_version: PROTOCOL_VERSION,
        session_id: "session".into(),
        command_id: "legacy-cancel".into(),
        catalog_revision: "revision".into(),
        command: PlaybackCommand::StopAll,
    };
    assert!(
        client
            .submit_guarded_envelope(envelope, &|| active.load(Ordering::Acquire))
            .await
            .is_err()
    );
    assert!(!client.is_connected());
    server.await.unwrap();
}
