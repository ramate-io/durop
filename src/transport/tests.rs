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
}
async fn fixture() -> Fixture {
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
            b"\x89PNG\r\n\x1a\ntrusted-test-icon".to_vec(),
        )]),
    )
    .unwrap();
    // The loopback bypass exists only under cfg(test); production enable rejects it.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let session = start(
        listener,
        vec!["https://panel.example".into()],
        HostBridge {
            commands,
            emergency,
            snapshot: snapshot_rx,
            events,
        },
        catalog,
        BindingGuard::Loopback,
    )
    .unwrap();
    Fixture {
        session,
        commands: commands_rx,
        emergency: emergency_rx,
        snapshot,
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
    serde_json::json!({"type":"command","envelope":{"protocolVersion":1,"sessionId":"test-session","commandId":id,"catalogRevision":"abc","command":kind}})
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
                    .insert("sec-websocket-protocol", "durop.v2".parse().unwrap());
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
    wrong_version["envelope"]["protocolVersion"] = 2.into();
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
