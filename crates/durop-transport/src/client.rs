//! Authenticated remote wire client with explicit targets and no reconnect/replay queue.
use super::*;
use std::collections::VecDeque;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async_with_config,
    tungstenite::client::IntoClientRequest,
};

/// A bounded request's transport or protocol result, without exposing secrets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerError {
    /// Target is not an explicit Tailscale IP and nonzero port.
    InvalidTarget,
    /// The connection has ended; create a new client to obtain a fresh snapshot.
    Disconnected,
    /// Upgrade, heartbeat, or request exceeded its deadline.
    Timeout,
    /// Response shape, request identity, or authoritative state violates the protocol.
    Protocol,
    /// The host or current connection cannot admit another command.
    Busy,
    /// The requested clip has no icon in the authenticated allowlist.
    UnknownIcon,
    /// The upgrade was denied or the selected host could not be reached.
    ConnectionDenied,
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

/// Authoritative state updates; a snapshot replaces any missed event history.
#[derive(Debug)]
pub enum ControllerUpdate {
    /// The next contiguous host event after the last applied state sequence.
    Event(HostEvent),
    /// A complete host snapshot obtained initially or after an event gap.
    Snapshot(HostSnapshot),
}

/// Bounded host packet shapes received on the single authenticated socket.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum Packet {
    // Replaces local state only after identity and sequence invariants pass.
    Snapshot {
        snapshot: HostSnapshot,
    },
    // The application validates its bounded metadata projection before displaying it.
    Catalog {
        catalog: Value,
    },
    // Both ID correlation and the shared PNG bounds are checked before returning bytes.
    Icon {
        clip_id: String,
        png: Vec<u8>,
    },
    // A command ID must match the single outstanding request on this socket.
    Outcome {
        command_id: String,
        outcome: CommandOutcome,
    },
    // Acknowledges cancellation after all earlier frames on this socket.
    Canceled {
        command_id: String,
    },
    // Gaps/overflow invalidate buffered history and force snapshot recovery.
    Event {
        event: HostEvent,
    },
    // Only the documented fixed codes remain recoverable without destroying the socket.
    Error {
        code: String,
    },
}

/// Correlate one outstanding request while allowing intervening snapshots/events.
enum Expected {
    // No ID is needed for browse/state requests because the client sends them serially.
    Snapshot,
    Catalog,
    Icon(String),
    Outcome(String),
    // A barrier drains any outcome that raced the abandoned command's cancellation.
    Canceled(String),
}

impl Expected {
    // State/event packets may arrive between request and reply; IDs must still match.
    fn matches(&self, packet: &Packet) -> bool {
        match (self, packet) {
            (Self::Snapshot, Packet::Snapshot { .. }) | (Self::Catalog, Packet::Catalog { .. }) => {
                true
            }
            (Self::Icon(expected), Packet::Icon { clip_id, .. }) => expected == clip_id,
            (Self::Outcome(expected), Packet::Outcome { command_id, .. })
            | (Self::Canceled(expected), Packet::Canceled { command_id }) => expected == command_id,
            _ => false,
        }
    }
}

/// Own this client in a Tokio task. Each mutable call has one deadline and sends only
/// on its current connection. Failure destroys that connection, including pending work.
/// There is no automatic reconnection; Connect always obtains a new host snapshot.
pub struct ControllerClient {
    // Taken on failure or disconnect; there is no reconnect handle or offline send buffer.
    socket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    // Last complete state, whose session/revision cannot change on this connection.
    snapshot: HostSnapshot,
    // Highest observed sequence, including a gap that the next snapshot must cover.
    sequence: u64,
    // At most 64 contiguous events awaiting the application consumer.
    events: VecDeque<HostEvent>,
    // A gap or overflow invalidates buffered history until a full snapshot arrives.
    needs_snapshot: bool,
    // Next scheduled ping; unrelated incoming events do not postpone it.
    heartbeat: tokio::time::Instant,
    // Expected Pong payload and its fixed deadline; arbitrary Pong frames do not renew it.
    pending_ping: Option<(Vec<u8>, tokio::time::Instant)>,
    // Distinguishes heartbeat payloads without storing identifying application data.
    ping_counter: u64,
    // Application-owned, nonblocking timing sink; keeps this crate independent of logging.
    acknowledgment_observer: Option<Arc<dyn Fn(Duration) + Send + Sync>>,
}

impl ControllerClient {
    #[cfg(any(test, feature = "test-support"))]
    /// Connect an isolated loopback fixture only in explicit test-support builds.
    pub async fn connect_loopback(
        target: SocketAddr,
        credential: SessionCredential,
    ) -> Result<Self, ControllerError> {
        if !target.ip().is_loopback() {
            return Err(ControllerError::InvalidTarget);
        }
        Self::connect_inner(target, credential).await
    }

    /// Authenticate an explicit Tailscale target and require its initial snapshot.
    pub async fn connect(
        target: SocketAddr,
        credential: SessionCredential,
    ) -> Result<Self, ControllerError> {
        if target.port() == 0 || !is_tailnet_ip(target.ip()) {
            return Err(ControllerError::InvalidTarget);
        }
        Self::connect_inner(target, credential).await
    }

    /// Perform the shared bounded handshake; only the explicit test-support entry accepts loopback.
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
            acknowledgment_observer: None,
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

    /// Whether a connection remains available as of its last observed I/O state.
    /// Continue polling updates so heartbeat deadlines can detect an idle blackhole.
    pub fn is_connected(&self) -> bool {
        self.socket.is_some()
    }

    /// Borrow the latest complete authoritative snapshot, excluding later event deltas.
    pub fn current_snapshot(&self) -> &HostSnapshot {
        &self.snapshot
    }

    /// Observe controller-local reply latency before snapshot recovery.
    /// The callback must be nonblocking and receives no credentials or identifiers.
    pub fn observe_acknowledgments(&mut self, observer: impl Fn(Duration) + Send + Sync + 'static) {
        self.acknowledgment_observer = Some(Arc::new(observer));
    }

    /// Close this socket and discard buffered events without creating a retry queue.
    pub async fn disconnect(&mut self) {
        if let Some(mut socket) = self.socket.take() {
            let _ = timeout(IO_TIMEOUT, socket.close(None)).await;
        }
        self.events.clear();
    }

    /// Build one command in the authenticated session and wait for its correlated outcome.
    /// A retry must deliberately reuse its command ID; it is never automatic.
    pub async fn command(
        &mut self,
        command_id: String,
        catalog_revision: String,
        command: PlaybackCommand,
    ) -> Result<CommandOutcome, ControllerError> {
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION,
            session_id: self.snapshot.session_id.clone(),
            command_id,
            catalog_revision,
            command,
        };
        self.submit_envelope(envelope).await
    }

    /// Send one canonical request without changing its identity or revision.
    /// A request from another session is rejected before reaching the socket.
    pub async fn submit_envelope(
        &mut self,
        envelope: Envelope,
    ) -> Result<CommandOutcome, ControllerError> {
        if envelope.command_id.is_empty()
            || envelope.command_id.len() > 128
            || envelope.session_id != self.snapshot.session_id
            || envelope.protocol_version != PROTOCOL_VERSION
        {
            return Err(ControllerError::Protocol);
        }
        let command_id = envelope.command_id.clone();
        let roundtrip_started = std::time::Instant::now();
        let packet = self
            .roundtrip(
                serde_json::json!({"type":"command", "envelope":envelope}),
                Expected::Outcome(command_id),
            )
            .await?;
        if let Some(observer) = &self.acknowledgment_observer {
            observer(roundtrip_started.elapsed());
        }
        self.resync_if_needed().await?;
        let Packet::Outcome { outcome, .. } = packet else {
            return Err(ControllerError::Protocol);
        };
        Ok(outcome)
    }

    /// Submit while preserving an upstream admission guard across a relay hop.
    ///
    /// The guard is checked before sending and at most every ten milliseconds while
    /// waiting. Revocation cancels this socket's pending host admission and drains
    /// its acknowledgment barrier, leaving unrelated controllers connected. `None`
    /// means the upstream source ended; a destination command already accepted
    /// before cancellation cannot be undone. Guard closures must be nonblocking.
    pub async fn submit_guarded_envelope(
        &mut self,
        envelope: Envelope,
        active: &(dyn Fn() -> bool + Send + Sync),
    ) -> Result<Option<CommandOutcome>, ControllerError> {
        if !active() {
            return Ok(None);
        }
        let command_id = envelope.command_id.clone();
        let result = tokio::select! {
            biased;
            _ = async {
                loop {
                    if !active() { break; }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            } => None,
            result = self.submit_envelope(envelope) => Some(result),
        };
        match result {
            Some(outcome) => outcome.map(Some),
            None => {
                self.roundtrip(
                    serde_json::json!({"type":"cancel", "command_id":command_id}),
                    Expected::Canceled(command_id),
                )
                .await?;
                Ok(None)
            }
        }
    }

    /// Fetch bounded controller-safe metadata over the authenticated connection.
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

    /// Fetch one allowlisted, dimension-bounded PNG by clip identity, never a path.
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

    /// Replace local state with a current snapshot without replaying any actions.
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

    /// Recover event gaps before returning control to another application action.
    async fn resync_if_needed(&mut self) -> Result<(), ControllerError> {
        if self.needs_snapshot {
            self.refresh_snapshot().await?;
        }
        Ok(())
    }

    /// Replace state only when sequence, session, and revision remain authoritative.
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

    /// Buffer at most 64 contiguous events; gaps force a full snapshot instead.
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

    /// Apply one total request deadline and destroy failed sockets without retrying.
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

    /// Send once and consume only the correlated reply, preserving intervening state.
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
                Packet::Outcome { command_id, .. } if matches!(&expected, Expected::Canceled(id) if id == &command_id) =>
                    {}
                Packet::Error { code }
                    if code == "busy" && matches!(expected, Expected::Canceled(_)) => {}
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

    /// Read bounded packets and enforce matching-Pong liveness while otherwise idle.
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

#[cfg(test)]
mod tests;
