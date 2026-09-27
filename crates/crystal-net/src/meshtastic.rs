//! Meshtastic discovery and reliable, fragmented link transport.
//!
//! This module deliberately owns no radio configuration. Callers select an
//! already-configured channel and provide packet I/O for PRIVATE_APP (256).

use std::collections::{BTreeMap, VecDeque};

use crystal_core::multiplayer::{
    LinkHello, LinkMessage, LinkSessionIdentity, SessionSaveCheckpointFrame,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use web_time::{Duration, Instant};

use crate::{
    DEFAULT_MAX_FRAME_BYTES, EndpointError, LinkEndpoint, LinkEndpointEvent, LinkTransport,
    TransportError, decode_bare_link_message, encode_bare_link_message,
    validate_local_session_bootstrap,
};

pub const PRIVATE_APP_PORT: i32 = 256;
pub const MESHTASTIC_DATA_BYTES: usize = 233;
pub const GEOTHITE_HEADER_BYTES: usize = 20;
pub const FRAGMENT_DATA_BYTES: usize = MESHTASTIC_DATA_BYTES - GEOTHITE_HEADER_BYTES;
pub const MESHTASTIC_WIRE_VERSION: u8 = 1;
pub const DISCOVERY_INTERVAL: Duration = Duration::from_secs(15);
pub const PEER_EXPIRY: Duration = Duration::from_secs(45);
pub const SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
pub const RETRY_BASE: Duration = Duration::from_secs(2);
pub const MAX_RETRY_ROUNDS: u8 = 5;
const MAGIC: [u8; 2] = *b"GT";
const RECENT_MESSAGE_LIMIT: usize = 64;
const MAX_PENDING_MESSAGES: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum PacketKind {
    Data = 1,
    CompleteAck = 2,
    Missing = 3,
    Discovery = 4,
    Invite = 5,
    InviteResponse = 6,
    Disconnect = 7,
}

impl TryFrom<u8> for PacketKind {
    type Error = MeshtasticProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Data,
            2 => Self::CompleteAck,
            3 => Self::Missing,
            4 => Self::Discovery,
            5 => Self::Invite,
            6 => Self::InviteResponse,
            7 => Self::Disconnect,
            other => return Err(MeshtasticProtocolError::UnknownPacketKind(other)),
        })
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum MeshtasticProtocolError {
    #[error("Meshtastic packet is shorter than the Geothite header")]
    HeaderTooShort,
    #[error("Meshtastic packet exceeds the 233-byte data limit")]
    PacketTooLarge,
    #[error("Meshtastic packet has invalid Geothite magic")]
    InvalidMagic,
    #[error("Meshtastic transport version {actual} does not match {expected}")]
    VersionMismatch { expected: u8, actual: u8 },
    #[error("unknown Geothite Meshtastic packet kind {0}")]
    UnknownPacketKind(u8),
    #[error("fragment count or index is invalid")]
    InvalidFragment,
    #[error("fragment metadata conflicts with the active message")]
    ConflictingFragment,
    #[error("reassembled message exceeds the 64 KiB link limit")]
    MessageTooLarge,
    #[error("reassembled message CRC does not match")]
    CrcMismatch,
    #[error("outbound Meshtastic queue is full")]
    QueueFull,
    #[error("Meshtastic peer did not acknowledge message {message_id}")]
    RetryLimit { message_id: u32 },
    #[error("invalid Meshtastic lobby payload: {0}")]
    InvalidLobby(String),
}

impl From<MeshtasticProtocolError> for TransportError {
    fn from(error: MeshtasticProtocolError) -> Self {
        Self::Meshtastic {
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshAppPacket {
    pub from: u32,
    pub to: u32,
    pub channel: u8,
    pub port: i32,
    pub want_ack: bool,
    pub payload: Vec<u8>,
}

/// Nonblocking packet boundary shared by native workers and browser bridges.
pub trait MeshtasticPacketIo {
    fn local_node_id(&self) -> u32;
    fn send_packet(&mut self, packet: MeshAppPacket) -> Result<(), String>;
    fn poll_packets(&mut self) -> Result<Vec<MeshAppPacket>, String>;
    fn is_congested(&self) -> bool {
        false
    }
}

impl<T: MeshtasticPacketIo + ?Sized> MeshtasticPacketIo for Box<T> {
    fn local_node_id(&self) -> u32 {
        (**self).local_node_id()
    }

    fn send_packet(&mut self, packet: MeshAppPacket) -> Result<(), String> {
        (**self).send_packet(packet)
    }

    fn poll_packets(&mut self) -> Result<Vec<MeshAppPacket>, String> {
        (**self).poll_packets()
    }

    fn is_congested(&self) -> bool {
        (**self).is_congested()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WirePacket {
    kind: PacketKind,
    session_nonce: u32,
    message_id: u32,
    fragment_index: u16,
    fragment_count: u16,
    crc32: u32,
    payload: Vec<u8>,
}

impl WirePacket {
    fn encode(&self) -> Result<Vec<u8>, MeshtasticProtocolError> {
        if self.payload.len() > FRAGMENT_DATA_BYTES {
            return Err(MeshtasticProtocolError::PacketTooLarge);
        }
        let mut bytes = Vec::with_capacity(GEOTHITE_HEADER_BYTES + self.payload.len());
        bytes.extend_from_slice(&MAGIC);
        bytes.push(MESHTASTIC_WIRE_VERSION);
        bytes.push(self.kind as u8);
        bytes.extend_from_slice(&self.session_nonce.to_be_bytes());
        bytes.extend_from_slice(&self.message_id.to_be_bytes());
        bytes.extend_from_slice(&self.fragment_index.to_be_bytes());
        bytes.extend_from_slice(&self.fragment_count.to_be_bytes());
        bytes.extend_from_slice(&self.crc32.to_be_bytes());
        bytes.extend_from_slice(&self.payload);
        Ok(bytes)
    }

    fn decode(bytes: &[u8]) -> Result<Self, MeshtasticProtocolError> {
        if bytes.len() < GEOTHITE_HEADER_BYTES {
            return Err(MeshtasticProtocolError::HeaderTooShort);
        }
        if bytes.len() > MESHTASTIC_DATA_BYTES {
            return Err(MeshtasticProtocolError::PacketTooLarge);
        }
        if bytes[..2] != MAGIC {
            return Err(MeshtasticProtocolError::InvalidMagic);
        }
        if bytes[2] != MESHTASTIC_WIRE_VERSION {
            return Err(MeshtasticProtocolError::VersionMismatch {
                expected: MESHTASTIC_WIRE_VERSION,
                actual: bytes[2],
            });
        }
        let packet = Self {
            kind: bytes[3].try_into()?,
            session_nonce: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            message_id: u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
            fragment_index: u16::from_be_bytes(bytes[12..14].try_into().unwrap()),
            fragment_count: u16::from_be_bytes(bytes[14..16].try_into().unwrap()),
            crc32: u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
            payload: bytes[20..].to_vec(),
        };
        if packet.kind == PacketKind::Data
            && (packet.fragment_count == 0 || packet.fragment_index >= packet.fragment_count)
        {
            return Err(MeshtasticProtocolError::InvalidFragment);
        }
        Ok(packet)
    }
}

fn fragment_payload(
    session_nonce: u32,
    message_id: u32,
    payload: &[u8],
) -> Result<Vec<Vec<u8>>, MeshtasticProtocolError> {
    if payload.is_empty() || payload.len() > DEFAULT_MAX_FRAME_BYTES {
        return Err(MeshtasticProtocolError::MessageTooLarge);
    }
    let count = payload.len().div_ceil(FRAGMENT_DATA_BYTES);
    let count = u16::try_from(count).map_err(|_| MeshtasticProtocolError::MessageTooLarge)?;
    let crc32 = crc32fast::hash(payload);
    payload
        .chunks(FRAGMENT_DATA_BYTES)
        .enumerate()
        .map(|(index, chunk)| {
            WirePacket {
                kind: PacketKind::Data,
                session_nonce,
                message_id,
                fragment_index: index as u16,
                fragment_count: count,
                crc32,
                payload: chunk.to_vec(),
            }
            .encode()
        })
        .collect()
}

#[derive(Debug)]
struct Reassembly {
    message_id: u32,
    fragment_count: u16,
    crc32: u32,
    fragments: BTreeMap<u16, Vec<u8>>,
    last_update: Instant,
    last_missing_request: Option<Instant>,
}

impl Reassembly {
    fn new(packet: &WirePacket, now: Instant) -> Self {
        Self {
            message_id: packet.message_id,
            fragment_count: packet.fragment_count,
            crc32: packet.crc32,
            fragments: BTreeMap::new(),
            last_update: now,
            last_missing_request: None,
        }
    }

    fn insert(
        &mut self,
        packet: WirePacket,
        now: Instant,
    ) -> Result<Option<Vec<u8>>, MeshtasticProtocolError> {
        if packet.message_id != self.message_id
            || packet.fragment_count != self.fragment_count
            || packet.crc32 != self.crc32
        {
            return Err(MeshtasticProtocolError::ConflictingFragment);
        }
        self.last_update = now;
        match self.fragments.get(&packet.fragment_index) {
            Some(existing) if existing != &packet.payload => {
                return Err(MeshtasticProtocolError::ConflictingFragment);
            }
            Some(_) => {}
            None => {
                self.fragments.insert(packet.fragment_index, packet.payload);
            }
        }
        let total = self.fragments.values().map(Vec::len).sum::<usize>();
        if total > DEFAULT_MAX_FRAME_BYTES {
            return Err(MeshtasticProtocolError::MessageTooLarge);
        }
        if self.fragments.len() != usize::from(self.fragment_count) {
            return Ok(None);
        }
        let mut joined = Vec::with_capacity(total);
        for index in 0..self.fragment_count {
            joined.extend_from_slice(
                self.fragments
                    .get(&index)
                    .ok_or(MeshtasticProtocolError::InvalidFragment)?,
            );
        }
        if crc32fast::hash(&joined) != self.crc32 {
            return Err(MeshtasticProtocolError::CrcMismatch);
        }
        Ok(Some(joined))
    }

    fn missing(&self) -> Vec<u16> {
        (0..self.fragment_count)
            .filter(|index| !self.fragments.contains_key(index))
            .collect()
    }
}

#[derive(Debug)]
struct OutboundMessage {
    id: u32,
    crc32: u32,
    fragments: Vec<Vec<u8>>,
    pending_fragments: VecDeque<u16>,
    sent_at: Option<Instant>,
    retry_round: u8,
}

/// Reliable point-to-point transport used after lobby invitation negotiation.
#[derive(Debug)]
pub struct MeshtasticLinkTransport<I> {
    io: I,
    session: LinkSessionIdentity,
    session_nonce: u32,
    peer_node_id: u32,
    channel: u8,
    next_message_id: u32,
    outbound: VecDeque<OutboundMessage>,
    inbound: Option<Reassembly>,
    decoded: VecDeque<LinkMessage>,
    recent: VecDeque<u32>,
    last_activity: Instant,
    connected: bool,
}

impl<I: MeshtasticPacketIo> MeshtasticLinkTransport<I> {
    pub fn new(
        io: I,
        session: LinkSessionIdentity,
        session_nonce: u32,
        peer_node_id: u32,
        channel: u8,
    ) -> Result<Self, TransportError> {
        if peer_node_id == 0 || peer_node_id == io.local_node_id() || channel > 7 {
            return Err(TransportError::Meshtastic {
                message: "invalid peer node or channel".into(),
            });
        }
        session
            .validate()
            .map_err(|error| TransportError::InvalidSession {
                message: error.to_string(),
            })?;
        Ok(Self {
            io,
            session,
            session_nonce,
            peer_node_id,
            channel,
            next_message_id: 1,
            outbound: VecDeque::new(),
            inbound: None,
            decoded: VecDeque::new(),
            recent: VecDeque::new(),
            last_activity: Instant::now(),
            connected: true,
        })
    }

    pub fn is_idle_timed_out(&self) -> bool {
        Instant::now().duration_since(self.last_activity) >= SESSION_IDLE_TIMEOUT
    }

    pub fn into_io(self) -> I {
        self.io
    }

    fn send_wire(&mut self, payload: Vec<u8>, want_ack: bool) -> Result<(), TransportError> {
        self.io
            .send_packet(MeshAppPacket {
                from: self.io.local_node_id(),
                to: self.peer_node_id,
                channel: self.channel,
                port: PRIVATE_APP_PORT,
                want_ack,
                payload,
            })
            .map_err(|message| TransportError::Meshtastic { message })
    }

    fn send_control(
        &mut self,
        kind: PacketKind,
        message_id: u32,
        crc32: u32,
        payload: Vec<u8>,
    ) -> Result<(), TransportError> {
        let packet = WirePacket {
            kind,
            session_nonce: self.session_nonce,
            message_id,
            fragment_index: 0,
            fragment_count: 0,
            crc32,
            payload,
        };
        self.send_wire(packet.encode()?, true)
    }

    fn flush_outbound(&mut self, now: Instant) -> Result<(), TransportError> {
        if self.io.is_congested() {
            return Ok(());
        }
        let Some(front) = self.outbound.front_mut() else {
            return Ok(());
        };
        if let Some(index) = front.pending_fragments.pop_front() {
            let frame = front
                .fragments
                .get(usize::from(index))
                .cloned()
                .ok_or(MeshtasticProtocolError::InvalidFragment)?;
            if front.pending_fragments.is_empty() {
                front.sent_at = Some(now);
            }
            return self.send_wire(frame, true);
        }
        let retry_after = RETRY_BASE.saturating_mul(1_u32 << front.retry_round.min(4));
        if front
            .sent_at
            .is_some_and(|sent| now.duration_since(sent) < retry_after)
        {
            return Ok(());
        }
        front.retry_round = front.retry_round.saturating_add(1);
        if front.retry_round > MAX_RETRY_ROUNDS {
            self.connected = false;
            return Err(MeshtasticProtocolError::RetryLimit {
                message_id: front.id,
            }
            .into());
        }
        front.pending_fragments = (0..front.fragments.len() as u16).collect();
        front.sent_at = None;
        Ok(())
    }

    fn handle_packet(&mut self, packet: MeshAppPacket, now: Instant) -> Result<(), TransportError> {
        if packet.from != self.peer_node_id
            || packet.to != self.io.local_node_id()
            || packet.channel != self.channel
            || packet.port != PRIVATE_APP_PORT
        {
            return Ok(());
        }
        let wire = WirePacket::decode(&packet.payload)?;
        if wire.session_nonce != self.session_nonce {
            return Ok(());
        }
        self.last_activity = now;
        match wire.kind {
            PacketKind::Data => {
                if self.recent.contains(&wire.message_id) {
                    self.send_control(
                        PacketKind::CompleteAck,
                        wire.message_id,
                        wire.crc32,
                        Vec::new(),
                    )?;
                    return Ok(());
                }
                if self
                    .inbound
                    .as_ref()
                    .is_some_and(|active| active.message_id != wire.message_id)
                {
                    return Ok(());
                }
                let active = self
                    .inbound
                    .get_or_insert_with(|| Reassembly::new(&wire, now));
                let message_id = wire.message_id;
                let crc32 = wire.crc32;
                if let Some(payload) = active.insert(wire, now)? {
                    let message = decode_bare_link_message(&payload, &self.session)?;
                    self.decoded.push_back(message);
                    self.recent.push_back(message_id);
                    while self.recent.len() > RECENT_MESSAGE_LIMIT {
                        self.recent.pop_front();
                    }
                    self.inbound = None;
                    self.send_control(PacketKind::CompleteAck, message_id, crc32, Vec::new())?;
                }
            }
            PacketKind::CompleteAck => {
                if self
                    .outbound
                    .front()
                    .is_some_and(|front| front.id == wire.message_id && front.crc32 == wire.crc32)
                {
                    self.outbound.pop_front();
                }
            }
            PacketKind::Missing => {
                if let Some(front) = self
                    .outbound
                    .front_mut()
                    .filter(|front| front.id == wire.message_id && front.crc32 == wire.crc32)
                {
                    let missing = wire
                        .payload
                        .chunks_exact(4)
                        .flat_map(|bytes| {
                            let start = u16::from_be_bytes([bytes[0], bytes[1]]);
                            let end = u16::from_be_bytes([bytes[2], bytes[3]]);
                            start..=end
                        })
                        .filter(|index| usize::from(*index) < front.fragments.len())
                        .collect::<Vec<_>>();
                    if !missing.is_empty() {
                        front.retry_round = front.retry_round.saturating_add(1);
                        if front.retry_round > MAX_RETRY_ROUNDS {
                            self.connected = false;
                            return Err(MeshtasticProtocolError::RetryLimit {
                                message_id: front.id,
                            }
                            .into());
                        }
                        front.pending_fragments = missing.into();
                        front.sent_at = None;
                    }
                }
            }
            PacketKind::Disconnect => self.connected = false,
            PacketKind::Discovery | PacketKind::Invite | PacketKind::InviteResponse => {}
        }
        Ok(())
    }

    fn request_missing_if_stalled(&mut self, now: Instant) -> Result<(), TransportError> {
        let Some(active) = self.inbound.as_mut() else {
            return Ok(());
        };
        let since = active.last_missing_request.unwrap_or(active.last_update);
        if now.duration_since(since) < RETRY_BASE {
            return Ok(());
        }
        let message_id = active.message_id;
        let crc32 = active.crc32;
        let mut payload = Vec::new();
        let mut missing = active.missing().into_iter().peekable();
        while let Some(start) = missing.next() {
            let mut end = start;
            while missing.peek().is_some_and(|next| *next == end + 1) {
                end = missing.next().expect("peeked missing fragment");
            }
            if payload.len() + 4 > FRAGMENT_DATA_BYTES {
                break;
            }
            payload.extend_from_slice(&start.to_be_bytes());
            payload.extend_from_slice(&end.to_be_bytes());
        }
        active.last_missing_request = Some(now);
        self.send_control(PacketKind::Missing, message_id, crc32, payload)
    }
}

impl<I: MeshtasticPacketIo> LinkTransport for MeshtasticLinkTransport<I> {
    fn send(&mut self, message: LinkMessage) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        if self.outbound.len() >= MAX_PENDING_MESSAGES {
            return Err(MeshtasticProtocolError::QueueFull.into());
        }
        let payload = encode_bare_link_message(&message, &self.session)?;
        let crc32 = crc32fast::hash(&payload);
        let id = self.next_message_id;
        self.next_message_id = self.next_message_id.wrapping_add(1).max(1);
        self.outbound.push_back(OutboundMessage {
            id,
            crc32,
            fragments: fragment_payload(self.session_nonce, id, &payload)?,
            pending_fragments: (0..payload.len().div_ceil(FRAGMENT_DATA_BYTES) as u16).collect(),
            sent_at: None,
            retry_round: 0,
        });
        self.flush_outbound(Instant::now())
    }

    fn poll(&mut self) -> Result<Vec<LinkMessage>, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }
        let now = Instant::now();
        if self
            .inbound
            .as_ref()
            .is_some_and(|active| now.duration_since(active.last_update) >= SESSION_IDLE_TIMEOUT)
        {
            self.inbound = None;
        }
        for packet in self
            .io
            .poll_packets()
            .map_err(|message| TransportError::Meshtastic { message })?
        {
            self.handle_packet(packet, now)?;
        }
        self.request_missing_if_stalled(now)?;
        self.flush_outbound(now)?;
        Ok(self.decoded.drain(..).collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshtasticLinkSessionEvent {
    Endpoint(LinkEndpointEvent),
    GameplayReady,
}

/// Completes the same mandatory hello/checkpoint bootstrap as hosted and TCP links.
#[derive(Debug)]
pub struct MeshtasticLinkSession<I> {
    endpoint: LinkEndpoint<MeshtasticLinkTransport<I>>,
    checkpoint: SessionSaveCheckpointFrame,
    checkpoint_sent: bool,
    gameplay_ready_emitted: bool,
}

impl<I: MeshtasticPacketIo> MeshtasticLinkSession<I> {
    pub fn new(
        transport: MeshtasticLinkTransport<I>,
        hello: LinkHello,
        checkpoint: SessionSaveCheckpointFrame,
    ) -> Result<Self, EndpointError> {
        validate_local_session_bootstrap(&hello, &checkpoint)?;
        let mut endpoint = LinkEndpoint::new(transport, hello)?;
        endpoint.send_hello()?;
        Ok(Self {
            endpoint,
            checkpoint,
            checkpoint_sent: false,
            gameplay_ready_emitted: false,
        })
    }

    pub fn send(&mut self, message: LinkMessage) -> Result<(), EndpointError> {
        self.endpoint.send(message)
    }

    pub fn is_ready_for_gameplay(&self) -> bool {
        self.endpoint.is_ready_for_gameplay()
    }

    pub fn poll(&mut self) -> Result<Vec<MeshtasticLinkSessionEvent>, EndpointError> {
        let mut events = self
            .endpoint
            .poll()?
            .into_iter()
            .map(MeshtasticLinkSessionEvent::Endpoint)
            .collect::<Vec<_>>();
        if self.endpoint.is_ready() && !self.checkpoint_sent {
            self.endpoint
                .send(LinkMessage::SessionSaveCheckpoint(self.checkpoint.clone()))?;
            self.checkpoint_sent = true;
        }
        if self.endpoint.is_ready_for_gameplay() && !self.gameplay_ready_emitted {
            self.gameplay_ready_emitted = true;
            events.push(MeshtasticLinkSessionEvent::GameplayReady);
        }
        Ok(events)
    }

    pub fn into_io(self) -> I {
        self.endpoint.into_transport().into_io()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CableClubRoom {
    TradeCenter = 1,
    TimeCapsule = 2,
    Colosseum = 3,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryAdvertisement {
    pub protocol_version: u16,
    pub room: CableClubRoom,
    pub node_id: u32,
    pub display_name: String,
    pub compatibility_digest: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredPeer {
    pub advertisement: DiscoveryAdvertisement,
    pub last_seen: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LobbyInvite {
    room: CableClubRoom,
    node_id: u32,
    nonce: u32,
    compatibility_digest: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LobbyInviteResponse {
    room: CableClubRoom,
    node_id: u32,
    request_nonce: u32,
    responder_nonce: u32,
    accepted: bool,
    compatibility_digest: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingMeshtasticInvite {
    pub from_node: u32,
    pub room: CableClubRoom,
    pub nonce: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiatedMeshtasticSession {
    pub peer_node: u32,
    pub room: CableClubRoom,
    pub session_nonce: u32,
    pub local_is_host: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshtasticLobbyEvent {
    PeerSeen(DiscoveryAdvertisement),
    PeerExpired { node_id: u32 },
    Invite(IncomingMeshtasticInvite),
    InviteDeclined { node_id: u32 },
    SessionAccepted(NegotiatedMeshtasticSession),
}

#[derive(Debug, Clone, Copy)]
struct PendingInvite {
    peer_node: u32,
    room: CableClubRoom,
    nonce: u32,
    sent_at: Instant,
}

/// Automatic Cable Club discovery and direct invitation negotiation.
#[derive(Debug)]
pub struct MeshtasticLobby<I> {
    io: I,
    channel: u8,
    local: DiscoveryAdvertisement,
    room: Option<CableClubRoom>,
    peers: BTreeMap<u32, DiscoveredPeer>,
    pending_invite: Option<PendingInvite>,
    last_discovery: Option<Instant>,
    next_nonce: u32,
}

impl<I: MeshtasticPacketIo> MeshtasticLobby<I> {
    pub fn new(
        io: I,
        channel: u8,
        display_name: impl Into<String>,
        session: &LinkSessionIdentity,
    ) -> Result<Self, MeshtasticProtocolError> {
        let display_name = display_name.into();
        if channel > 7
            || display_name.is_empty()
            || display_name.len() > 64
            || display_name.chars().any(char::is_control)
            || io.local_node_id() == 0
        {
            return Err(MeshtasticProtocolError::InvalidLobby(
                "channel, local node, or display name is invalid".into(),
            ));
        }
        Ok(Self {
            local: DiscoveryAdvertisement {
                protocol_version: MESHTASTIC_WIRE_VERSION.into(),
                room: CableClubRoom::TradeCenter,
                node_id: io.local_node_id(),
                display_name,
                compatibility_digest: compatibility_digest(session),
            },
            io,
            channel,
            room: None,
            peers: BTreeMap::new(),
            pending_invite: None,
            last_discovery: None,
            next_nonce: 1,
        })
    }

    pub fn enter_room(&mut self, room: CableClubRoom) {
        self.room = Some(room);
        self.local.room = room;
        self.peers.clear();
        self.pending_invite = None;
        self.last_discovery = None;
    }

    pub fn leave_room(&mut self) {
        self.room = None;
        self.peers.clear();
        self.pending_invite = None;
        self.last_discovery = None;
    }

    pub fn peers(&self) -> impl Iterator<Item = &DiscoveredPeer> {
        self.peers.values()
    }

    pub fn has_pending_invite(&self) -> bool {
        self.pending_invite.is_some()
    }

    pub fn into_io(self) -> I {
        self.io
    }

    pub fn invite(&mut self, peer_node: u32) -> Result<(), MeshtasticProtocolError> {
        let room = self.room.ok_or_else(|| {
            MeshtasticProtocolError::InvalidLobby("not waiting in a Cable Club room".into())
        })?;
        let peer = self.peers.get(&peer_node).ok_or_else(|| {
            MeshtasticProtocolError::InvalidLobby("selected peer is no longer available".into())
        })?;
        if peer.advertisement.room != room
            || peer.advertisement.compatibility_digest != self.local.compatibility_digest
        {
            return Err(MeshtasticProtocolError::InvalidLobby(
                "selected peer is incompatible".into(),
            ));
        }
        let nonce = self.take_nonce();
        let invite = LobbyInvite {
            room,
            node_id: self.local.node_id,
            nonce,
            compatibility_digest: self.local.compatibility_digest,
        };
        self.send_lobby(PacketKind::Invite, peer_node, true, nonce, &invite)?;
        self.pending_invite = Some(PendingInvite {
            peer_node,
            room,
            nonce,
            sent_at: Instant::now(),
        });
        Ok(())
    }

    pub fn respond(
        &mut self,
        invite: &IncomingMeshtasticInvite,
        accepted: bool,
    ) -> Result<Option<NegotiatedMeshtasticSession>, MeshtasticProtocolError> {
        if self.room != Some(invite.room) {
            return Err(MeshtasticProtocolError::InvalidLobby(
                "invitation no longer matches the selected room".into(),
            ));
        }
        let responder_nonce = self.take_nonce();
        let response = LobbyInviteResponse {
            room: invite.room,
            node_id: self.local.node_id,
            request_nonce: invite.nonce,
            responder_nonce,
            accepted,
            compatibility_digest: self.local.compatibility_digest,
        };
        self.send_lobby(
            PacketKind::InviteResponse,
            invite.from_node,
            true,
            invite.nonce,
            &response,
        )?;
        Ok(accepted.then(|| NegotiatedMeshtasticSession {
            peer_node: invite.from_node,
            room: invite.room,
            session_nonce: session_nonce(invite.nonce, responder_nonce),
            local_is_host: lower_node_is_host(self.local.node_id, invite.from_node),
        }))
    }

    pub fn poll(&mut self) -> Result<Vec<MeshtasticLobbyEvent>, MeshtasticProtocolError> {
        let now = Instant::now();
        let mut events = Vec::new();
        if self.room.is_some()
            && self
                .last_discovery
                .is_none_or(|last| now.duration_since(last) >= DISCOVERY_INTERVAL)
            && !self.io.is_congested()
        {
            self.send_lobby(
                PacketKind::Discovery,
                u32::MAX,
                false,
                0,
                &self.local.clone(),
            )?;
            self.last_discovery = Some(now);
        }
        for packet in self
            .io
            .poll_packets()
            .map_err(MeshtasticProtocolError::InvalidLobby)?
        {
            if packet.from == self.local.node_id
                || packet.channel != self.channel
                || packet.port != PRIVATE_APP_PORT
                || (packet.to != self.local.node_id && packet.to != u32::MAX)
            {
                continue;
            }
            let wire = match WirePacket::decode(&packet.payload) {
                Ok(wire) => wire,
                Err(_) => continue,
            };
            if crc32fast::hash(&wire.payload) != wire.crc32 {
                continue;
            }
            match wire.kind {
                PacketKind::Discovery if self.room.is_some() => {
                    let advertisement: DiscoveryAdvertisement = decode_lobby(&wire.payload)?;
                    if advertisement.node_id != packet.from
                        || advertisement.protocol_version != u16::from(MESHTASTIC_WIRE_VERSION)
                        || Some(advertisement.room) != self.room
                        || advertisement.compatibility_digest != self.local.compatibility_digest
                    {
                        continue;
                    }
                    self.peers.insert(
                        packet.from,
                        DiscoveredPeer {
                            advertisement: advertisement.clone(),
                            last_seen: now,
                        },
                    );
                    events.push(MeshtasticLobbyEvent::PeerSeen(advertisement));
                }
                PacketKind::Invite if self.room.is_some() => {
                    let invite: LobbyInvite = decode_lobby(&wire.payload)?;
                    if invite.node_id == packet.from
                        && Some(invite.room) == self.room
                        && invite.compatibility_digest == self.local.compatibility_digest
                    {
                        events.push(MeshtasticLobbyEvent::Invite(IncomingMeshtasticInvite {
                            from_node: packet.from,
                            room: invite.room,
                            nonce: invite.nonce,
                        }));
                    }
                }
                PacketKind::InviteResponse => {
                    let response: LobbyInviteResponse = decode_lobby(&wire.payload)?;
                    if let Some(pending) = self.pending_invite.filter(|pending| {
                        pending.peer_node == packet.from
                            && pending.room == response.room
                            && pending.nonce == response.request_nonce
                            && response.node_id == packet.from
                            && response.compatibility_digest == self.local.compatibility_digest
                    }) {
                        self.pending_invite = None;
                        if response.accepted {
                            events.push(MeshtasticLobbyEvent::SessionAccepted(
                                NegotiatedMeshtasticSession {
                                    peer_node: packet.from,
                                    room: response.room,
                                    session_nonce: session_nonce(
                                        pending.nonce,
                                        response.responder_nonce,
                                    ),
                                    local_is_host: lower_node_is_host(
                                        self.local.node_id,
                                        packet.from,
                                    ),
                                },
                            ));
                        } else {
                            events.push(MeshtasticLobbyEvent::InviteDeclined {
                                node_id: packet.from,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        let expired = self
            .peers
            .iter()
            .filter_map(|(node, peer)| {
                (now.duration_since(peer.last_seen) >= PEER_EXPIRY).then_some(*node)
            })
            .collect::<Vec<_>>();
        for node_id in expired {
            self.peers.remove(&node_id);
            events.push(MeshtasticLobbyEvent::PeerExpired { node_id });
        }
        if self
            .pending_invite
            .is_some_and(|pending| now.duration_since(pending.sent_at) >= SESSION_IDLE_TIMEOUT)
        {
            if let Some(pending) = self.pending_invite.take() {
                events.push(MeshtasticLobbyEvent::InviteDeclined {
                    node_id: pending.peer_node,
                });
            }
        }
        Ok(events)
    }

    fn take_nonce(&mut self) -> u32 {
        let nonce = self.next_nonce.max(1);
        self.next_nonce = self.next_nonce.wrapping_add(1).max(1);
        nonce
    }

    fn send_lobby<T: Serialize>(
        &mut self,
        kind: PacketKind,
        to: u32,
        want_ack: bool,
        message_id: u32,
        value: &T,
    ) -> Result<(), MeshtasticProtocolError> {
        let payload = bincode::serde::encode_to_vec(value, bincode::config::standard())
            .map_err(|error| MeshtasticProtocolError::InvalidLobby(error.to_string()))?;
        let payload = WirePacket {
            kind,
            session_nonce: 0,
            message_id,
            fragment_index: 0,
            fragment_count: 0,
            crc32: crc32fast::hash(&payload),
            payload,
        }
        .encode()?;
        self.io
            .send_packet(MeshAppPacket {
                from: self.local.node_id,
                to,
                channel: self.channel,
                port: PRIVATE_APP_PORT,
                want_ack,
                payload,
            })
            .map_err(MeshtasticProtocolError::InvalidLobby)
    }
}

fn decode_lobby<T: for<'de> Deserialize<'de>>(
    payload: &[u8],
) -> Result<T, MeshtasticProtocolError> {
    let (value, consumed) = bincode::serde::decode_from_slice(payload, bincode::config::standard())
        .map_err(|error| MeshtasticProtocolError::InvalidLobby(error.to_string()))?;
    if consumed != payload.len() {
        return Err(MeshtasticProtocolError::InvalidLobby(
            "lobby payload has trailing bytes".into(),
        ));
    }
    Ok(value)
}

/// Compute the compact discovery identity; the full handshake remains authoritative.
pub fn compatibility_digest(session: &LinkSessionIdentity) -> [u8; 16] {
    let mut hasher = Sha256::new();
    hasher.update(&session.protocol_version().to_be_bytes());
    hasher.update(session.modpack().id().as_bytes());
    hasher.update(session.modpack().hash().as_bytes());
    hasher.update(session.pack_content_hash().as_bytes());
    let mut digest = [0_u8; 16];
    digest.copy_from_slice(&hasher.finalize()[..16]);
    digest
}

pub fn session_nonce(local_nonce: u32, remote_nonce: u32) -> u32 {
    let (low, high) = if local_nonce <= remote_nonce {
        (local_nonce, remote_nonce)
    } else {
        (remote_nonce, local_nonce)
    };
    let mut bytes = [0_u8; 8];
    bytes[..4].copy_from_slice(&low.to_be_bytes());
    bytes[4..].copy_from_slice(&high.to_be_bytes());
    crc32fast::hash(&bytes).max(1)
}

pub fn lower_node_is_host(local_node: u32, remote_node: u32) -> bool {
    local_node < remote_node
}

#[cfg(all(feature = "meshtastic", not(target_arch = "wasm32")))]
pub mod native {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
        },
        thread,
        time::Duration as StdDuration,
    };

    use meshtastic::{
        api::{StreamApi, StreamHandle},
        protobufs::{self, from_radio, mesh_packet, to_radio},
        utils,
    };
    use tokio::io::{AsyncRead, AsyncWrite};

    use super::{MESHTASTIC_DATA_BYTES, MeshAppPacket, MeshtasticPacketIo, PRIVATE_APP_PORT};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum NativeMeshtasticConnection {
        Serial(String),
        Tcp(String),
        Ble(String),
    }

    #[derive(Debug)]
    pub struct NativeMeshtasticIo {
        local_node_id: u32,
        outbound: SyncSender<MeshAppPacket>,
        inbound: Receiver<MeshAppPacket>,
        errors: Receiver<String>,
        congested: Arc<AtomicBool>,
    }

    impl NativeMeshtasticIo {
        pub fn connect(connection: NativeMeshtasticConnection) -> Result<Self, String> {
            let (outbound_tx, outbound_rx) = mpsc::sync_channel(64);
            let (inbound_tx, inbound_rx) = mpsc::sync_channel(64);
            let (ready_tx, ready_rx) = mpsc::sync_channel(1);
            let (error_tx, error_rx) = mpsc::channel();
            let congested = Arc::new(AtomicBool::new(false));
            let worker_congested = Arc::clone(&congested);
            thread::Builder::new()
                .name("geothite-meshtastic".into())
                .spawn(move || {
                    let runtime = match tokio::runtime::Runtime::new() {
                        Ok(runtime) => runtime,
                        Err(error) => {
                            let message = format!("create Meshtastic runtime: {error}");
                            let _ = ready_tx.send(Err(message.clone()));
                            let _ = error_tx.send(message);
                            return;
                        }
                    };
                    let result = runtime.block_on(run_connection(
                        connection,
                        outbound_rx,
                        inbound_tx,
                        ready_tx,
                        worker_congested,
                    ));
                    if let Err(error) = result {
                        let _ = error_tx.send(error);
                    }
                })
                .map_err(|error| format!("start Meshtastic worker: {error}"))?;
            let local_node_id = ready_rx
                .recv_timeout(StdDuration::from_secs(20))
                .map_err(|_| "timed out waiting for Meshtastic radio identity".to_string())??;
            Ok(Self {
                local_node_id,
                outbound: outbound_tx,
                inbound: inbound_rx,
                errors: error_rx,
                congested,
            })
        }
    }

    impl MeshtasticPacketIo for NativeMeshtasticIo {
        fn local_node_id(&self) -> u32 {
            self.local_node_id
        }

        fn send_packet(&mut self, packet: MeshAppPacket) -> Result<(), String> {
            if packet.from != self.local_node_id
                || packet.port != PRIVATE_APP_PORT
                || packet.channel > 7
                || packet.payload.len() > MESHTASTIC_DATA_BYTES
            {
                return Err("invalid outbound Meshtastic application packet".into());
            }
            self.outbound.try_send(packet).map_err(|error| match error {
                TrySendError::Full(_) => "Meshtastic radio queue is full".into(),
                TrySendError::Disconnected(_) => "Meshtastic worker disconnected".into(),
            })
        }

        fn poll_packets(&mut self) -> Result<Vec<MeshAppPacket>, String> {
            if let Ok(error) = self.errors.try_recv() {
                return Err(error);
            }
            let mut packets = Vec::new();
            loop {
                match self.inbound.try_recv() {
                    Ok(packet) => packets.push(packet),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        return Err("Meshtastic worker disconnected".into());
                    }
                }
            }
            Ok(packets)
        }

        fn is_congested(&self) -> bool {
            self.congested.load(Ordering::Relaxed)
        }
    }

    async fn run_connection(
        connection: NativeMeshtasticConnection,
        outbound: Receiver<MeshAppPacket>,
        inbound: SyncSender<MeshAppPacket>,
        ready: mpsc::SyncSender<Result<u32, String>>,
        congested: Arc<AtomicBool>,
    ) -> Result<(), String> {
        match connection {
            NativeMeshtasticConnection::Serial(port) => {
                let stream = utils::stream::build_serial_stream(port, None, None, None)
                    .map_err(|error| error.to_string())?;
                run_stream(stream, outbound, inbound, ready, congested).await
            }
            NativeMeshtasticConnection::Tcp(address) => {
                let stream = utils::stream::build_tcp_stream(address)
                    .await
                    .map_err(|error| error.to_string())?;
                run_stream(stream, outbound, inbound, ready, congested).await
            }
            NativeMeshtasticConnection::Ble(identifier) => {
                let ble_id = if identifier.contains(':')
                    || (identifier.len() == 12
                        && identifier.bytes().all(|byte| byte.is_ascii_hexdigit()))
                {
                    utils::stream::BleId::from_mac_address(&identifier)
                        .map_err(|error| error.to_string())?
                } else {
                    utils::stream::BleId::from_name(&identifier)
                };
                let stream = utils::stream::build_ble_stream(ble_id, StdDuration::from_secs(10))
                    .await
                    .map_err(|error| error.to_string())?;
                run_stream(stream, outbound, inbound, ready, congested).await
            }
        }
    }

    async fn run_stream<S>(
        stream: StreamHandle<S>,
        outbound: Receiver<MeshAppPacket>,
        inbound: SyncSender<MeshAppPacket>,
        ready: mpsc::SyncSender<Result<u32, String>>,
        congested: Arc<AtomicBool>,
    ) -> Result<(), String>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (mut received, api) = StreamApi::new().connect(stream).await;
        let mut api = api
            .configure(utils::generate_rand_id())
            .await
            .map_err(|error| error.to_string())?;
        let mut local_node = None;
        let mut ready = Some(ready);
        loop {
            tokio::select! {
                packet = received.recv() => {
                    let Some(packet) = packet else {
                        return Err("Meshtastic radio connection closed".into());
                    };
                    match packet.payload_variant {
                        Some(from_radio::PayloadVariant::MyInfo(info)) => {
                            if info.my_node_num != 0 && local_node != Some(info.my_node_num) {
                                local_node = Some(info.my_node_num);
                                if let Some(sender) = ready.take() {
                                    let _ = sender.send(Ok(info.my_node_num));
                                }
                            }
                        }
                        Some(from_radio::PayloadVariant::Packet(packet)) => {
                            if let Some(app_packet) = decode_mesh_packet(packet) {
                                let _ = inbound.try_send(app_packet);
                            }
                        }
                        Some(from_radio::PayloadVariant::QueueStatus(status)) => {
                            congested.store(status.res != 0 || status.free == 0, Ordering::Relaxed);
                        }
                        _ => {}
                    }
                }
                _ = tokio::time::sleep(StdDuration::from_millis(20)) => {
                    loop {
                        match outbound.try_recv() {
                            Ok(packet) => {
                                send_mesh_packet(&mut api, packet).await?;
                            }
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => {
                                let _ = api.disconnect().await;
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
    }

    async fn send_mesh_packet(
        api: &mut meshtastic::api::ConnectedStreamApi,
        packet: MeshAppPacket,
    ) -> Result<(), String> {
        let mesh_packet = protobufs::MeshPacket {
            from: packet.from,
            to: packet.to,
            channel: u32::from(packet.channel),
            id: utils::generate_rand_id(),
            want_ack: packet.want_ack,
            payload_variant: Some(mesh_packet::PayloadVariant::Decoded(protobufs::Data {
                portnum: PRIVATE_APP_PORT,
                payload: packet.payload,
                ..Default::default()
            })),
            ..Default::default()
        };
        api.send_to_radio_packet(Some(to_radio::PayloadVariant::Packet(mesh_packet)))
            .await
            .map_err(|error| error.to_string())
    }

    fn decode_mesh_packet(packet: protobufs::MeshPacket) -> Option<MeshAppPacket> {
        let mesh_packet::PayloadVariant::Decoded(data) = packet.payload_variant? else {
            return None;
        };
        if data.portnum != PRIVATE_APP_PORT || data.payload.len() > MESHTASTIC_DATA_BYTES {
            return None;
        }
        Some(MeshAppPacket {
            from: packet.from,
            to: packet.to,
            channel: u8::try_from(packet.channel).ok()?,
            port: data.portnum,
            want_ack: packet.want_ack,
            payload: data.payload,
        })
    }
}

#[cfg(all(feature = "meshtastic", target_arch = "wasm32"))]
pub mod browser {
    use std::cell::RefCell;

    use meshtastic::{Message as _, protobufs};
    use wasm_bindgen::prelude::*;

    use super::{MESHTASTIC_DATA_BYTES, MeshAppPacket, MeshtasticPacketIo, PRIVATE_APP_PORT};

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name = crystalMeshtasticSendToRadio)]
        fn send_to_radio(bytes: &[u8]) -> bool;
    }

    #[derive(Default)]
    struct BrowserState {
        local_node: u32,
        packets: Vec<MeshAppPacket>,
        error: Option<String>,
        congested: bool,
    }

    thread_local! {
        static STATE: RefCell<BrowserState> = RefCell::new(BrowserState::default());
    }

    #[derive(Debug)]
    pub struct BrowserMeshtasticIo {
        local_node: u32,
    }

    impl BrowserMeshtasticIo {
        pub fn connect() -> Result<Self, String> {
            let local_node = STATE.with_borrow(|state| state.local_node);
            if local_node == 0 {
                return Err("connect a browser Meshtastic radio before starting the game".into());
            }
            Ok(Self { local_node })
        }
    }

    impl MeshtasticPacketIo for BrowserMeshtasticIo {
        fn local_node_id(&self) -> u32 {
            self.local_node
        }

        fn send_packet(&mut self, packet: MeshAppPacket) -> Result<(), String> {
            if packet.from != self.local_node
                || packet.channel > 7
                || packet.port != PRIVATE_APP_PORT
                || packet.payload.len() > MESHTASTIC_DATA_BYTES
            {
                return Err("invalid browser Meshtastic packet".into());
            }
            let packet = protobufs::MeshPacket {
                from: packet.from,
                to: packet.to,
                channel: u32::from(packet.channel),
                id: js_sys::Math::random().to_bits() as u32 | 1,
                want_ack: packet.want_ack,
                payload_variant: Some(protobufs::mesh_packet::PayloadVariant::Decoded(
                    protobufs::Data {
                        portnum: PRIVATE_APP_PORT,
                        payload: packet.payload,
                        ..Default::default()
                    },
                )),
                ..Default::default()
            };
            let bytes = protobufs::ToRadio {
                payload_variant: Some(protobufs::to_radio::PayloadVariant::Packet(packet)),
            }
            .encode_to_vec();
            if send_to_radio(&bytes) {
                Ok(())
            } else {
                Err("browser Meshtastic device rejected a write".into())
            }
        }

        fn poll_packets(&mut self) -> Result<Vec<MeshAppPacket>, String> {
            STATE.with_borrow_mut(|state| {
                if let Some(error) = state.error.take() {
                    return Err(error);
                }
                Ok(state.packets.drain(..).collect())
            })
        }

        fn is_congested(&self) -> bool {
            STATE.with_borrow(|state| state.congested)
        }
    }

    #[wasm_bindgen]
    pub fn crystal_meshtastic_begin() -> Vec<u8> {
        protobufs::ToRadio {
            payload_variant: Some(protobufs::to_radio::PayloadVariant::WantConfigId(
                (js_sys::Math::random().to_bits() as u32).max(1),
            )),
        }
        .encode_to_vec()
    }

    #[wasm_bindgen]
    pub fn crystal_meshtastic_receive(bytes: &[u8]) -> Result<(), JsValue> {
        let packet = protobufs::FromRadio::decode(bytes)
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        match packet.payload_variant {
            Some(protobufs::from_radio::PayloadVariant::MyInfo(info)) => {
                STATE.with_borrow_mut(|state| state.local_node = info.my_node_num);
            }
            Some(protobufs::from_radio::PayloadVariant::Packet(packet)) => {
                let Some(protobufs::mesh_packet::PayloadVariant::Decoded(data)) =
                    packet.payload_variant
                else {
                    return Ok(());
                };
                if data.portnum == PRIVATE_APP_PORT
                    && data.payload.len() <= MESHTASTIC_DATA_BYTES
                    && packet.channel <= 7
                {
                    STATE.with_borrow_mut(|state| {
                        state.packets.push(MeshAppPacket {
                            from: packet.from,
                            to: packet.to,
                            channel: packet.channel as u8,
                            port: data.portnum,
                            want_ack: packet.want_ack,
                            payload: data.payload,
                        });
                    });
                }
            }
            Some(protobufs::from_radio::PayloadVariant::QueueStatus(status)) => {
                STATE.with_borrow_mut(|state| {
                    state.congested = status.res != 0 || status.free == 0;
                });
            }
            _ => {}
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn crystal_meshtastic_connected_node() -> u32 {
        STATE.with_borrow(|state| state.local_node)
    }

    #[wasm_bindgen]
    pub fn crystal_meshtastic_error(message: String) {
        STATE.with_borrow_mut(|state| state.error = Some(message));
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crystal_core::save::SaveModpackIdentity;

    use super::*;

    #[derive(Default)]
    struct MockMesh {
        inboxes: HashMap<u32, Vec<MeshAppPacket>>,
        duplicate_data: bool,
        congested: bool,
    }

    #[derive(Clone)]
    struct MockIo {
        node: u32,
        mesh: Rc<RefCell<MockMesh>>,
    }

    impl MeshtasticPacketIo for MockIo {
        fn local_node_id(&self) -> u32 {
            self.node
        }

        fn send_packet(&mut self, packet: MeshAppPacket) -> Result<(), String> {
            let mut mesh = self.mesh.borrow_mut();
            let recipients = if packet.to == u32::MAX {
                mesh.inboxes
                    .keys()
                    .copied()
                    .filter(|node| *node != self.node)
                    .collect::<Vec<_>>()
            } else {
                vec![packet.to]
            };
            for recipient in recipients {
                mesh.inboxes
                    .entry(recipient)
                    .or_default()
                    .push(packet.clone());
                if mesh.duplicate_data
                    && WirePacket::decode(&packet.payload)
                        .is_ok_and(|wire| wire.kind == PacketKind::Data)
                {
                    mesh.inboxes
                        .entry(recipient)
                        .or_default()
                        .push(packet.clone());
                }
            }
            Ok(())
        }

        fn poll_packets(&mut self) -> Result<Vec<MeshAppPacket>, String> {
            Ok(std::mem::take(
                self.mesh.borrow_mut().inboxes.entry(self.node).or_default(),
            ))
        }

        fn is_congested(&self) -> bool {
            self.mesh.borrow().congested
        }
    }

    fn mock_pair() -> (MockIo, MockIo, Rc<RefCell<MockMesh>>) {
        let mesh = Rc::new(RefCell::new(MockMesh {
            inboxes: HashMap::from([(10, Vec::new()), (20, Vec::new())]),
            ..Default::default()
        }));
        (
            MockIo {
                node: 10,
                mesh: Rc::clone(&mesh),
            },
            MockIo {
                node: 20,
                mesh: Rc::clone(&mesh),
            },
            mesh,
        )
    }

    fn test_session() -> LinkSessionIdentity {
        LinkSessionIdentity::new(
            "mesh-test",
            SaveModpackIdentity::new(
                "core-modular",
                "1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd",
            )
            .unwrap(),
            "0102030401020304010203040102030401020304010203040102030401020304",
        )
        .unwrap()
    }

    #[test]
    fn fragments_never_exceed_meshtastic_limit_and_reassemble_out_of_order() {
        let payload = (0..1000).map(|value| value as u8).collect::<Vec<_>>();
        let frames = fragment_payload(7, 11, &payload).expect("fragment");
        assert!(
            frames
                .iter()
                .all(|frame| frame.len() <= MESHTASTIC_DATA_BYTES)
        );
        let mut decoded = frames
            .iter()
            .map(|frame| WirePacket::decode(frame).expect("decode"))
            .collect::<Vec<_>>();
        decoded.reverse();
        let now = Instant::now();
        let mut reassembly = Reassembly::new(&decoded[0], now);
        let mut complete = None;
        for packet in decoded {
            complete = reassembly.insert(packet, now).expect("insert").or(complete);
        }
        assert_eq!(complete.as_deref(), Some(payload.as_slice()));
    }

    #[test]
    fn corrupt_complete_payload_is_rejected() {
        let frames = fragment_payload(1, 1, &[4; 500]).expect("fragment");
        let now = Instant::now();
        let mut packets = frames
            .iter()
            .map(|frame| WirePacket::decode(frame).unwrap())
            .collect::<Vec<_>>();
        packets[0].payload[0] ^= 0xff;
        let mut reassembly = Reassembly::new(&packets[0], now);
        let mut error = None;
        for packet in packets {
            if let Err(found) = reassembly.insert(packet, now) {
                error = Some(found);
            }
        }
        assert_eq!(error, Some(MeshtasticProtocolError::CrcMismatch));
    }

    #[test]
    fn fragmentation_accepts_exact_link_limit_and_rejects_larger_messages() {
        let maximum = vec![0x5a; DEFAULT_MAX_FRAME_BYTES];
        let frames = fragment_payload(1, 1, &maximum).unwrap();
        assert_eq!(frames.len(), DEFAULT_MAX_FRAME_BYTES.div_ceil(FRAGMENT_DATA_BYTES));
        assert!(frames.iter().all(|frame| frame.len() <= MESHTASTIC_DATA_BYTES));
        assert_eq!(
            fragment_payload(1, 2, &vec![0; DEFAULT_MAX_FRAME_BYTES + 1]),
            Err(MeshtasticProtocolError::MessageTooLarge)
        );
    }

    #[test]
    fn stalled_reassembly_requests_compact_missing_ranges_once_per_backoff() {
        let (a_io, _b_io, mesh) = mock_pair();
        let session = test_session();
        let mut transport = MeshtasticLinkTransport::new(a_io, session, 77, 20, 0).unwrap();
        let frames = fragment_payload(77, 9, &[7; FRAGMENT_DATA_BYTES * 4]).unwrap();
        let now = Instant::now();
        let mut active = Reassembly::new(&WirePacket::decode(&frames[0]).unwrap(), now);
        active.insert(WirePacket::decode(&frames[0]).unwrap(), now).unwrap();
        active.insert(WirePacket::decode(&frames[3]).unwrap(), now).unwrap();
        transport.inbound = Some(active);
        transport.request_missing_if_stalled(now + RETRY_BASE).unwrap();
        transport.request_missing_if_stalled(now + RETRY_BASE).unwrap();
        let packets = mesh.borrow_mut().inboxes.remove(&20).unwrap();
        assert_eq!(packets.len(), 1);
        let request = WirePacket::decode(&packets[0].payload).unwrap();
        assert_eq!(request.kind, PacketKind::Missing);
        assert_eq!(request.payload, [0, 1, 0, 2]);
    }

    #[test]
    fn session_nonce_is_symmetric_and_nonzero() {
        assert_eq!(session_nonce(10, 99), session_nonce(99, 10));
        assert_ne!(session_nonce(0, 0), 0);
        assert!(lower_node_is_host(10, 99));
        assert!(!lower_node_is_host(99, 10));
    }

    #[test]
    fn transport_orders_messages_and_deduplicates_radio_delivery() {
        let (a_io, b_io, mesh) = mock_pair();
        mesh.borrow_mut().duplicate_data = true;
        let session = test_session();
        let mut a = MeshtasticLinkTransport::new(a_io, session.clone(), 77, 20, 0).unwrap();
        let mut b = MeshtasticLinkTransport::new(b_io, session, 77, 10, 0).unwrap();
        let first = LinkMessage::Disconnect {
            player_id: 10,
            reason: "first".into(),
        };
        let second = LinkMessage::Disconnect {
            player_id: 10,
            reason: "second".into(),
        };
        a.send(first.clone()).unwrap();
        a.send(second.clone()).unwrap();
        assert_eq!(b.poll().unwrap(), vec![first]);
        a.poll().unwrap();
        assert_eq!(b.poll().unwrap(), vec![second]);
        a.poll().unwrap();
        assert!(b.poll().unwrap().is_empty());
    }

    #[test]
    fn transport_ignores_wrong_session_and_bounds_congested_queue() {
        let (a_io, mut b_io, mesh) = mock_pair();
        let session = test_session();
        let mut a = MeshtasticLinkTransport::new(a_io, session, 77, 20, 0).unwrap();
        let wrong = fragment_payload(88, 1, &[1, 2, 3]).unwrap().remove(0);
        b_io.send_packet(MeshAppPacket {
            from: 20,
            to: 10,
            channel: 0,
            port: PRIVATE_APP_PORT,
            want_ack: true,
            payload: wrong,
        })
        .unwrap();
        assert!(a.poll().unwrap().is_empty());
        mesh.borrow_mut().congested = true;
        for index in 0..MAX_PENDING_MESSAGES {
            a.send(LinkMessage::Disconnect {
                player_id: 10,
                reason: format!("queued-{index}"),
            })
            .unwrap();
        }
        assert!(matches!(
            a.send(LinkMessage::Disconnect {
                player_id: 10,
                reason: "overflow".into()
            }),
            Err(TransportError::Meshtastic { .. })
        ));
    }

    #[test]
    fn compatible_lobbies_discover_and_negotiate_the_same_session() {
        let (a_io, b_io, _) = mock_pair();
        let session = test_session();
        let mut a = MeshtasticLobby::new(a_io, 0, "ALICE", &session).unwrap();
        let mut b = MeshtasticLobby::new(b_io, 0, "BOB", &session).unwrap();
        a.enter_room(CableClubRoom::Colosseum);
        b.enter_room(CableClubRoom::Colosseum);
        a.poll().unwrap();
        let b_events = b.poll().unwrap();
        assert!(b_events.iter().any(
            |event| matches!(event, MeshtasticLobbyEvent::PeerSeen(peer) if peer.node_id == 10)
        ));
        let a_events = a.poll().unwrap();
        assert!(a_events.iter().any(
            |event| matches!(event, MeshtasticLobbyEvent::PeerSeen(peer) if peer.node_id == 20)
        ));
        a.invite(20).unwrap();
        let invite = b
            .poll()
            .unwrap()
            .into_iter()
            .find_map(|event| match event {
                MeshtasticLobbyEvent::Invite(invite) => Some(invite),
                _ => None,
            })
            .unwrap();
        let accepted_by_b = b.respond(&invite, true).unwrap().unwrap();
        let accepted_by_a = a
            .poll()
            .unwrap()
            .into_iter()
            .find_map(|event| match event {
                MeshtasticLobbyEvent::SessionAccepted(session) => Some(session),
                _ => None,
            })
            .unwrap();
        assert_eq!(accepted_by_a.session_nonce, accepted_by_b.session_nonce);
        assert!(accepted_by_a.local_is_host);
        assert!(!accepted_by_b.local_is_host);
    }
}
