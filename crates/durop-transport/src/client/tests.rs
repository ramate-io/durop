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
