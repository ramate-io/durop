//! Native controller with explicit targets and no reconnect/replay queue.
use super::*;
use std::collections::VecDeque;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async_with_config,
    tungstenite::client::IntoClientRequest,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerError {
    InvalidTarget,
    Disconnected,
    Timeout,
    Protocol,
    Busy,
    UnknownIcon,
    ConnectionDenied,
}

#[cfg(test)]
mod tests {
    use super::*;
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
            socket.send(Message::Text(serde_json::json!({"type":"event","event":HostEvent { sequence: 2, instance_id: None, event: crate::commands::EventKind::Stopped }}).to_string().into())).await.unwrap();
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
}
impl std::fmt::Display for ControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidTarget => "select a Tailscale address and port",
            Self::Disconnected => "controller disconnected; reconnect for a fresh snapshot",
            Self::Timeout => "controller request timed out; reconnect",
            Self::Protocol => "invalid remote protocol response",
            Self::Busy => "host busy",
            Self::UnknownIcon => "unknown catalog icon",
            Self::ConnectionDenied => "remote connection denied or unavailable",
        })
    }
}
impl std::error::Error for ControllerError {}

#[derive(Debug)]
pub enum ControllerUpdate {
    Event(HostEvent),
    Snapshot(HostSnapshot),
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Packet {
    Snapshot {
        snapshot: HostSnapshot,
    },
    Catalog {
        catalog: Value,
    },
    Icon {
        clip_id: String,
        png: Vec<u8>,
    },
    Outcome {
        command_id: String,
        outcome: CommandOutcome,
    },
    Event {
        event: HostEvent,
    },
    Error {
        code: String,
    },
}
enum Expected {
    Snapshot,
    Catalog,
    Icon(String),
    Outcome(String),
}
impl Expected {
    fn matches(&self, packet: &Packet) -> bool {
        match (self, packet) {
            (Self::Snapshot, Packet::Snapshot { .. }) | (Self::Catalog, Packet::Catalog { .. }) => {
                true
            }
            (Self::Icon(expected), Packet::Icon { clip_id, .. }) => expected == clip_id,
            (Self::Outcome(expected), Packet::Outcome { command_id, .. }) => expected == command_id,
            _ => false,
        }
    }
}

/// Own this client in a Tokio task. Each mutable call has one deadline and sends only
/// on its current connection. Failure destroys that connection, including pending work.
/// There is no automatic reconnection; Connect always obtains a new host snapshot.
pub struct ControllerClient {
    socket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    snapshot: HostSnapshot,
    sequence: u64,
    events: VecDeque<HostEvent>,
    needs_snapshot: bool,
    heartbeat: tokio::time::Instant,
    pending_ping: Option<(Vec<u8>, tokio::time::Instant)>,
    ping_counter: u64,
}
impl ControllerClient {
    #[cfg(test)]
    pub(crate) async fn connect_loopback(
        target: SocketAddr,
        credential: SessionCredential,
    ) -> Result<Self, ControllerError> {
        if !target.ip().is_loopback() {
            return Err(ControllerError::InvalidTarget);
        }
        Self::connect_inner(target, credential).await
    }
    pub async fn connect(
        target: SocketAddr,
        credential: SessionCredential,
    ) -> Result<Self, ControllerError> {
        if target.port() == 0 || !is_tailnet_ip(target.ip()) {
            return Err(ControllerError::InvalidTarget);
        }
        Self::connect_inner(target, credential).await
    }
    async fn connect_inner(
        target: SocketAddr,
        credential: SessionCredential,
    ) -> Result<Self, ControllerError> {
        // SocketAddr prevents DNS redirects, arbitrary URL paths, and URL credentials.
        let mut request = format!("ws://{target}/control")
            .into_client_request()
            .map_err(|_| ControllerError::InvalidTarget)?;
        request.headers_mut().insert(
            "authorization",
            format!("Bearer {}", credential.bearer())
                .parse()
                .map_err(|_| ControllerError::Protocol)?,
        );
        request.headers_mut().insert(
            "sec-websocket-protocol",
            SUBPROTOCOL.parse().expect("constant HTTP header"),
        );
        let config = WebSocketConfig::default()
            .max_message_size(Some(MAX_OUTPUT))
            .max_frame_size(Some(MAX_OUTPUT))
            .write_buffer_size(0)
            .max_write_buffer_size(MAX_INPUT + 1024);
        let (socket, response) = timeout(
            HANDSHAKE_TIMEOUT,
            connect_async_with_config(request, Some(config), true),
        )
        .await
        .map_err(|_| ControllerError::Timeout)?
        .map_err(|_| ControllerError::ConnectionDenied)?;
        if response
            .headers()
            .get("sec-websocket-protocol")
            .and_then(|value| value.to_str().ok())
            != Some(SUBPROTOCOL)
        {
            return Err(ControllerError::Protocol);
        }
        // The empty identity exists only until the required first snapshot is received.
        let mut client = Self {
            socket: Some(socket),
            snapshot: HostSnapshot {
                session_id: String::new(),
                catalog_revision: String::new(),
                sequence: 0,
                clips: vec![],
                instances: vec![],
            },
            sequence: 0,
            events: VecDeque::new(),
            needs_snapshot: false,
            heartbeat: tokio::time::Instant::now() + Duration::from_secs(20),
            pending_ping: None,
            ping_counter: 0,
        };
        let initial = timeout(IO_TIMEOUT, client.read())
            .await
            .map_err(|_| ControllerError::Timeout)??;
        let Packet::Snapshot { snapshot } = initial else {
            return Err(ControllerError::Protocol);
        };
        client.apply_snapshot(snapshot)?;
        Ok(client)
    }
    pub fn is_connected(&self) -> bool {
        self.socket.is_some()
    }
    pub fn current_snapshot(&self) -> &HostSnapshot {
        &self.snapshot
    }
    pub async fn disconnect(&mut self) {
        if let Some(mut socket) = self.socket.take() {
            let _ = timeout(IO_TIMEOUT, socket.close(None)).await;
        }
        self.events.clear();
    }
    pub async fn command(
        &mut self,
        command_id: String,
        catalog_revision: String,
        command: PlaybackCommand,
    ) -> Result<CommandOutcome, ControllerError> {
        if command_id.is_empty() || command_id.len() > 128 {
            return Err(ControllerError::Protocol);
        }
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION,
            session_id: self.snapshot.session_id.clone(),
            command_id: command_id.clone(),
            catalog_revision,
            command,
        };
        let roundtrip_started = std::time::Instant::now();
        let packet = self
            .roundtrip(
                serde_json::json!({"type":"command", "envelope":envelope}),
                Expected::Outcome(command_id),
            )
            .await?;
        crate::diagnostics::controller_ack(roundtrip_started.elapsed());
        self.resync_if_needed().await?;
        let Packet::Outcome { outcome, .. } = packet else {
            return Err(ControllerError::Protocol);
        };
        Ok(outcome)
    }
    pub async fn catalog(&mut self) -> Result<Value, ControllerError> {
        let packet = self
            .roundtrip(serde_json::json!({"type":"catalog"}), Expected::Catalog)
            .await?;
        self.resync_if_needed().await?;
        let Packet::Catalog { catalog } = packet else {
            return Err(ControllerError::Protocol);
        };
        Ok(catalog)
    }
    pub async fn icon(&mut self, clip_id: &str) -> Result<Vec<u8>, ControllerError> {
        if clip_id.is_empty() || clip_id.len() > 256 {
            return Err(ControllerError::UnknownIcon);
        }
        let packet = self
            .roundtrip(
                serde_json::json!({"type":"icon", "clip_id":clip_id}),
                Expected::Icon(clip_id.into()),
            )
            .await?;
        self.resync_if_needed().await?;
        let Packet::Icon { png, .. } = packet else {
            return Err(ControllerError::Protocol);
        };
        if !valid_icon_png(&png) {
            self.socket = None;
            return Err(ControllerError::Protocol);
        }
        Ok(png)
    }
    pub async fn refresh_snapshot(&mut self) -> Result<HostSnapshot, ControllerError> {
        let packet = self
            .roundtrip(serde_json::json!({"type":"snapshot"}), Expected::Snapshot)
            .await?;
        let Packet::Snapshot { snapshot } = packet else {
            return Err(ControllerError::Protocol);
        };
        self.apply_snapshot(snapshot)?;
        Ok(self.snapshot.clone())
    }
    /// Can be selected alongside UI actions. Cancelling this read does not queue a
    /// button press or reconnect. Snapshot updates replace any missed event history.
    pub async fn next_update(&mut self) -> Result<ControllerUpdate, ControllerError> {
        if self.needs_snapshot {
            return self
                .refresh_snapshot()
                .await
                .map(ControllerUpdate::Snapshot);
        }
        if let Some(event) = self.events.pop_front() {
            return Ok(ControllerUpdate::Event(event));
        }
        loop {
            let packet = match self.read().await {
                Ok(packet) => packet,
                Err(error) => {
                    self.socket = None;
                    return Err(error);
                }
            };
            match packet {
                Packet::Snapshot { snapshot } => {
                    self.apply_snapshot(snapshot)?;
                    return Ok(ControllerUpdate::Snapshot(self.snapshot.clone()));
                }
                Packet::Event { event } => {
                    self.record_event(event);
                    if self.needs_snapshot {
                        return self
                            .refresh_snapshot()
                            .await
                            .map(ControllerUpdate::Snapshot);
                    }
                    if let Some(event) = self.events.pop_front() {
                        return Ok(ControllerUpdate::Event(event));
                    }
                }
                _ => {
                    self.socket = None;
                    return Err(ControllerError::Protocol);
                }
            }
        }
    }
    async fn resync_if_needed(&mut self) -> Result<(), ControllerError> {
        if self.needs_snapshot {
            self.refresh_snapshot().await?;
        }
        Ok(())
    }
    fn apply_snapshot(&mut self, snapshot: HostSnapshot) -> Result<(), ControllerError> {
        if snapshot.session_id.is_empty()
            || snapshot.session_id.len() > 128
            || snapshot.sequence < self.sequence
            || (!self.snapshot.session_id.is_empty()
                && (snapshot.session_id != self.snapshot.session_id
                    || snapshot.catalog_revision != self.snapshot.catalog_revision))
        {
            self.socket = None;
            return Err(ControllerError::Protocol);
        }
        self.sequence = snapshot.sequence;
        self.snapshot = snapshot;
        self.events.clear();
        self.needs_snapshot = false;
        Ok(())
    }
    fn record_event(&mut self, event: HostEvent) {
        if event.sequence <= self.sequence {
            return;
        }
        if self.needs_snapshot
            || event.sequence != self.sequence.saturating_add(1)
            || self.events.len() == 64
        {
            self.sequence = event.sequence;
            self.events.clear();
            self.needs_snapshot = true;
            return;
        }
        self.sequence = event.sequence;
        self.events.push_back(event);
    }
    async fn roundtrip(
        &mut self,
        request: Value,
        expected: Expected,
    ) -> Result<Packet, ControllerError> {
        let result = timeout(IO_TIMEOUT, self.exchange(request, expected))
            .await
            .unwrap_or(Err(ControllerError::Timeout));
        if matches!(result, Err(error) if error != ControllerError::Busy && error != ControllerError::UnknownIcon)
        {
            self.socket = None;
            self.events.clear();
        }
        result
    }
    async fn exchange(
        &mut self,
        request: Value,
        expected: Expected,
    ) -> Result<Packet, ControllerError> {
        let text = request.to_string();
        if text.len() > MAX_INPUT {
            return Err(ControllerError::Protocol);
        }
        self.socket
            .as_mut()
            .ok_or(ControllerError::Disconnected)?
            .send(Message::Text(text.into()))
            .await
            .map_err(|_| ControllerError::Disconnected)?;
        loop {
            let packet = self.read().await?;
            if expected.matches(&packet) {
                return Ok(packet);
            }
            match packet {
                Packet::Event { event } => self.record_event(event),
                Packet::Snapshot { snapshot } => self.apply_snapshot(snapshot)?,
                Packet::Error { code } => {
                    return Err(match code.as_str() {
                        "busy" => ControllerError::Busy,
                        "unknown_icon" => ControllerError::UnknownIcon,
                        _ => ControllerError::Protocol,
                    });
                }
                _ => return Err(ControllerError::Protocol),
            }
        }
    }
    async fn read(&mut self) -> Result<Packet, ControllerError> {
        loop {
            let deadline = self
                .pending_ping
                .as_ref()
                .map(|(_, deadline)| *deadline)
                .unwrap_or(self.heartbeat);
            if self.pending_ping.is_some() && tokio::time::Instant::now() >= deadline {
                return Err(ControllerError::Timeout);
            }
            let socket = self.socket.as_mut().ok_or(ControllerError::Disconnected)?;
            let message = match tokio::time::timeout_at(deadline, socket.next()).await {
                Ok(next) => next
                    .ok_or(ControllerError::Disconnected)?
                    .map_err(|_| ControllerError::Disconnected)?,
                Err(_) => {
                    if self.pending_ping.is_some() {
                        return Err(ControllerError::Timeout);
                    }
                    self.ping_counter = self.ping_counter.wrapping_add(1);
                    let ping = self.ping_counter.to_be_bytes().to_vec();
                    timeout(IO_TIMEOUT, socket.send(Message::Ping(ping.clone().into())))
                        .await
                        .map_err(|_| ControllerError::Timeout)?
                        .map_err(|_| ControllerError::Disconnected)?;
                    self.heartbeat = tokio::time::Instant::now() + Duration::from_secs(20);
                    self.pending_ping = Some((ping, tokio::time::Instant::now() + IO_TIMEOUT));
                    continue;
                }
            };
            match message {
                Message::Text(text) => {
                    return serde_json::from_str(&text).map_err(|_| ControllerError::Protocol);
                }
                Message::Ping(bytes) => timeout(IO_TIMEOUT, socket.send(Message::Pong(bytes)))
                    .await
                    .map_err(|_| ControllerError::Timeout)?
                    .map_err(|_| ControllerError::Disconnected)?,
                Message::Pong(bytes) => {
                    if self
                        .pending_ping
                        .as_ref()
                        .is_some_and(|(expected, _)| expected.as_slice() == bytes.as_ref())
                    {
                        self.pending_ping = None;
                    }
                }
                _ => return Err(ControllerError::Disconnected),
            }
        }
    }
}
