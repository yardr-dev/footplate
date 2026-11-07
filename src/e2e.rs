//! The channel between a paired device and its yard: what a device that
//! paired by QR seals every call to a hosted yard in, so the edge that
//! carries the call cannot read it (the yardr design's sections 3.1 and
//! 3.2). A device makes one X25519 key pair for each yard and pins the
//! yard's channel public key beside it at pairing; on a yard's opening
//! it runs a Noise_IK handshake to that key (by `snow`, and no Noise
//! construction of footplate's own), and from then on each call is one
//! whole inner call — method, path, headers and body — sealed inside
//! one outer call on the fixed path `/user/e2e`, its answer opened
//! against the call that asked for it.
//!
//! Transport messages do not ride the handshake's counters, as the
//! yard serves a link's calls in no sealing order: each message
//! carries its sequence with it and is sealed with a fresh copy of its
//! direction's key at that nonce, with the session tag, a direction
//! byte, its call's id, its sequence and its record index as the AEAD's
//! associated data — and, on a call's first record in a direction, the
//! outer path and the stream marker it was sent with. A receiver
//! refuses a record of another call before it spends an opening on it,
//! keeps one sliding window of the sequences it opened in a direction,
//! and holds a call whole only once its final record has opened. This
//! module speaks bytes only: which calls are sealed, and what a refusal
//! is said as, are `api.rs`'s.

use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use snow::{Builder, HandshakeState};

/// The fixed outer path of an enveloped call, as the device addresses
/// it and as a first record binds it.
pub const OUTER_PATH: &str = "/user/e2e";
/// The session tag's size: sixteen random bytes the yard mints at a
/// handshake, named in the clear of every later message.
pub const TAG_SIZE: usize = 16;
/// A call id's size: sixteen random bytes the device mints once for
/// each call it opens, never reused in a session.
pub const CALL_SIZE: usize = 16;
/// A transport message's clear header: the tag, the call id, the
/// sequence and the record index.
const HEADER_SIZE: usize = TAG_SIZE + CALL_SIZE + 8 + 8;
/// The most plaintext one transport message carries: the Noise bound
/// of 65,535 bytes, its clear header and its AEAD tag counted.
pub const MAX_RECORD_PLAIN: usize = 65535 - HEADER_SIZE - 16;
/// How many opened sequences a receiver remembers in one direction of
/// a session: a record already opened there, or fallen below the
/// window, is refused.
pub const REPLAY_WINDOW: u64 = 65536;

/// The one Noise pattern and suite of a channel, as the yard runs it.
const PATTERN: &str = "Noise_IK_25519_ChaChaPoly_SHA256";
/// The directions a message travels, as the associated data names them.
const DIR_REQUEST: u8 = 0; // device to yard
const DIR_ANSWER: u8 = 1; // yard to device

/// Why a channel step did not go: a sentence for the log, never read.
/// What a person is told of one is `api.rs`'s word, not this.
pub type Result<T> = std::result::Result<T, String>;

fn snow_err(what: &str, err: snow::Error) -> String {
    format!("{what}: {err}")
}

/// Sixteen random bytes of the system's: a session tag the yard mints,
/// or a call id the device mints.
pub fn random16() -> Result<[u8; TAG_SIZE]> {
    let mut bytes = [0u8; TAG_SIZE];
    getrandom::fill(&mut bytes).map_err(|err| format!("no randomness to mint with: {err}"))?;
    Ok(bytes)
}

/// One X25519 channel key pair, as the yard keeps its half and as a
/// device makes its own for one yard: 32 bytes a half.
#[derive(Clone, PartialEq)]
pub struct ChannelKey {
    pub private: [u8; 32],
    pub public: [u8; 32],
}

impl ChannelKey {
    /// A fresh channel key pair, for a device at pairing.
    pub fn generate() -> Result<ChannelKey> {
        let params: snow::params::NoiseParams = PATTERN
            .parse()
            .map_err(|err| snow_err("the channel's pattern", err))?;
        let pair = Builder::new(params)
            .generate_keypair()
            .map_err(|err| snow_err("a channel key pair could not be made", err))?;
        Ok(ChannelKey {
            private: pair
                .private
                .try_into()
                .map_err(|_| "a channel private key is no 32 bytes".to_string())?,
            public: pair
                .public
                .try_into()
                .map_err(|_| "a channel public key is no 32 bytes".to_string())?,
        })
    }
}

/// A channel public key as it travels: in a pairing QR's fourth field,
/// in a device record beside its token's hash, and beside a device's
/// private key in the file that keeps it. Base64url, no padding, as
/// the yard writes it.
pub fn channel_public(public: &[u8; 32]) -> String {
    B64.encode(public)
}

/// A channel public key as [`channel_public`] writes it. What is no
/// such key — another encoding, another length, padding the yard
/// never writes — is refused here, before anything is sent under it.
pub fn parse_channel_public(text: &str) -> Result<[u8; 32]> {
    let bytes = B64
        .decode(text)
        .map_err(|_| format!("{text:?} is no channel public key"))?;
    let public: [u8; 32] = bytes
        .try_into()
        .map_err(|_| format!("{text:?} is no channel public key"))?;
    if channel_public(&public) != text {
        return Err(format!("{text:?} is no channel public key"));
    }
    Ok(public)
}

fn initiator(device_private: &[u8; 32], yard_public: &[u8; 32]) -> Result<HandshakeState> {
    let params: snow::params::NoiseParams = PATTERN
        .parse()
        .map_err(|err| snow_err("the channel's pattern", err))?;
    Builder::new(params)
        .local_private_key(device_private)
        .map_err(|err| snow_err("the device's channel key", err))?
        .remote_public_key(yard_public)
        .map_err(|err| snow_err("the yard's channel key", err))?
        .build_initiator()
        .map_err(|err| snow_err("a handshake could not be begun", err))
}

fn responder(yard_private: &[u8; 32]) -> Result<HandshakeState> {
    let params: snow::params::NoiseParams = PATTERN
        .parse()
        .map_err(|err| snow_err("the channel's pattern", err))?;
    Builder::new(params)
        .local_private_key(yard_private)
        .map_err(|err| snow_err("the yard's channel key", err))?
        .build_responder()
        .map_err(|err| snow_err("a handshake could not be begun", err))
}

/// A pairing handshake a device has begun: its first message sent, its
/// answer still to open. The first message carries the pairing code as
/// its payload and the device's static key sealed inside it, where
/// only the yard can read either.
pub struct PairingOpening {
    hs: HandshakeState,
}

/// Begins a pairing handshake to the yard channel public key a device
/// pinned (from the pairing QR), with `code` as the first message's
/// payload: the device's half of a sealed redemption.
pub fn start_pairing(
    device_private: &[u8; 32],
    yard_public: &[u8; 32],
    code: &str,
) -> Result<(Vec<u8>, PairingOpening)> {
    let mut hs = initiator(device_private, yard_public)?;
    let mut first = vec![0u8; 1024 + code.len()];
    let n = hs
        .write_message(code.as_bytes(), &mut first)
        .map_err(|err| snow_err("a pairing handshake could not be written", err))?;
    first.truncate(n);
    Ok((first, PairingOpening { hs }))
}

impl PairingOpening {
    /// Opens the yard's sealed answer to this opening: the pairing's
    /// answer, readable by this device alone.
    pub fn open_answer(mut self, reply: &[u8]) -> Result<Vec<u8>> {
        let mut payload = vec![0u8; 65535];
        let n = self
            .hs
            .read_message(reply, &mut payload)
            .map_err(|err| snow_err("a pairing answer that does not open", err))?;
        payload.truncate(n);
        Ok(payload)
    }
}

/// A device pairing's first message as the yard opened it: the code it
/// carried and the device key it proved, as [`channel_public`] writes
/// it. The yard's half, and the stand-in yard of the tests'.
pub struct PairingHandshake {
    hs: HandshakeState,
    pub code: String,
    pub device_key: String,
}

/// Opens the first message of a pairing handshake to the yard's
/// channel key. A message that does not open — made to another yard's
/// key, or no handshake at all — is refused here, before any code is
/// read, so it is no try against the pairing.
pub fn open_pairing(yard_private: &[u8; 32], first: &[u8]) -> Result<PairingHandshake> {
    let mut hs = responder(yard_private)?;
    let mut payload = vec![0u8; 65535];
    let n = hs
        .read_message(first, &mut payload)
        .map_err(|err| snow_err("a pairing handshake that does not open", err))?;
    payload.truncate(n);
    let device = hs
        .get_remote_static()
        .ok_or_else(|| "a pairing handshake with no device key".to_string())?;
    let device: [u8; 32] = device
        .try_into()
        .map_err(|_| "a pairing handshake with no device key".to_string())?;
    Ok(PairingHandshake {
        hs,
        code: String::from_utf8_lossy(&payload).into_owned(),
        device_key: channel_public(&device),
    })
}

impl PairingHandshake {
    /// Seals `payload` as the handshake's second message: the one
    /// answer that holds the device's token.
    pub fn seal_answer(mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut reply = vec![0u8; 1024 + payload.len()];
        let n = self
            .hs
            .write_message(payload, &mut reply)
            .map_err(|err| snow_err("a pairing answer could not be sealed", err))?;
        reply.truncate(n);
        Ok(reply)
    }
}

/// A channel handshake a device has begun: its first message sent, its
/// answer still to open.
pub struct ChannelOpening {
    hs: HandshakeState,
}

/// Begins a channel handshake to the yard channel public key a device
/// pinned at pairing, with the device's own channel key as its static:
/// the device's half of opening a session. Its first message names no
/// call, which is how the yard tells it from an enveloped call.
pub fn start_channel(
    device_private: &[u8; 32],
    yard_public: &[u8; 32],
) -> Result<(Vec<u8>, ChannelOpening)> {
    let mut hs = initiator(device_private, yard_public)?;
    let mut first = vec![0u8; 1024];
    let n = hs
        .write_message(&[], &mut first)
        .map_err(|err| snow_err("a channel handshake could not be written", err))?;
    first.truncate(n);
    Ok((first, ChannelOpening { hs }))
}

impl ChannelOpening {
    /// Opens the yard's handshake answer, whose payload is the session
    /// tag the yard minted, and answers the session it founds: this
    /// device sends under the handshake's first transport key and
    /// opens under its second.
    pub fn open_answer(mut self, reply: &[u8]) -> Result<Arc<Session>> {
        let mut payload = vec![0u8; 65535];
        let n = self
            .hs
            .read_message(reply, &mut payload)
            .map_err(|err| snow_err("a channel handshake answer that does not open", err))?;
        payload.truncate(n);
        let tag: [u8; TAG_SIZE] = payload
            .try_into()
            .map_err(|_| "a channel handshake answer names no session tag".to_string())?;
        let (send, recv) = self.hs.dangerously_get_raw_split();
        Ok(Arc::new(Session::new(
            tag,
            send,
            recv,
            DIR_REQUEST,
            DIR_ANSWER,
        )))
    }
}

/// A channel handshake's first message as the yard opened it: the
/// device key it proved, as [`channel_public`] writes it, and the
/// state its answer is sealed with. The yard's half, and the stand-in
/// yard of the tests'.
pub struct ChannelHandshake {
    hs: HandshakeState,
    pub device_key: String,
}

/// Opens the first message of a channel handshake to the yard's
/// channel key. A message made to another yard's key, or no handshake
/// at all, is refused here, before any device is looked for.
pub fn open_channel(yard_private: &[u8; 32], first: &[u8]) -> Result<ChannelHandshake> {
    let mut hs = responder(yard_private)?;
    let mut payload = vec![0u8; 65535];
    let _ = hs
        .read_message(first, &mut payload)
        .map_err(|err| snow_err("a channel handshake that does not open", err))?;
    let device = hs
        .get_remote_static()
        .ok_or_else(|| "a channel handshake with no device key".to_string())?;
    let device: [u8; 32] = device
        .try_into()
        .map_err(|_| "a channel handshake with no device key".to_string())?;
    Ok(ChannelHandshake {
        hs,
        device_key: channel_public(&device),
    })
}

impl ChannelHandshake {
    /// Seals the session tag as the handshake's second message and
    /// answers the session it founds: the yard opens under the
    /// handshake's first transport key and sends under its second.
    pub fn seal_answer(mut self, tag: &[u8; TAG_SIZE]) -> Result<(Vec<u8>, Arc<Session>)> {
        let mut reply = vec![0u8; 1024];
        let n = self
            .hs
            .write_message(tag, &mut reply)
            .map_err(|err| snow_err("a channel handshake answer could not be sealed", err))?;
        reply.truncate(n);
        let (initiator_key, responder_key) = self.hs.dangerously_get_raw_split();
        Ok((
            reply,
            Arc::new(Session::new(
                *tag,
                responder_key,
                initiator_key,
                DIR_ANSWER,
                DIR_REQUEST,
            )),
        ))
    }
}

/// What a call's first record in a direction binds beside the clear
/// header: the fixed outer path both ends know without it crossing,
/// and the outer stream marker's presence and value as the sender
/// sent them. A first record whose outer path or marker was rewritten
/// between the device and the yard fails its opening, and its call
/// with it, rather than silently reclassifying the call.
#[derive(Clone, Debug, PartialEq)]
pub struct FirstBinding {
    pub path: String,
    pub marker: String,
    pub present: bool,
}

impl FirstBinding {
    /// The binding of a call that carries no stream marker.
    pub fn plain() -> FirstBinding {
        FirstBinding {
            path: OUTER_PATH.to_string(),
            marker: String::new(),
            present: false,
        }
    }

    /// The binding of a call the device marked with `marker`.
    pub fn marked(marker: &str) -> FirstBinding {
        FirstBinding {
            path: OUTER_PATH.to_string(),
            marker: marker.to_string(),
            present: true,
        }
    }
}

/// Builds the AEAD's associated data for one message: the tag, the
/// direction, the call id, the sequence and the index, and, for a
/// call's first record, the outer path and the stream marker's
/// presence and value, each length-named so no two writings agree by
/// accident.
fn associated(
    tag: &[u8; TAG_SIZE],
    dir: u8,
    call: &[u8; CALL_SIZE],
    seq: u64,
    index: u64,
    first: Option<&FirstBinding>,
) -> Vec<u8> {
    let mut ad = Vec::with_capacity(HEADER_SIZE + 64);
    ad.extend_from_slice(tag);
    ad.push(dir);
    ad.extend_from_slice(call);
    ad.extend_from_slice(&seq.to_be_bytes());
    ad.extend_from_slice(&index.to_be_bytes());
    match first {
        None => ad.push(0),
        Some(first) => {
            ad.push(1);
            ad.extend_from_slice(&(first.path.len() as u16).to_be_bytes());
            ad.extend_from_slice(first.path.as_bytes());
            ad.push(u8::from(first.present));
            ad.extend_from_slice(&(first.marker.len() as u16).to_be_bytes());
            ad.extend_from_slice(first.marker.as_bytes());
        }
    }
    ad
}

/// The nonce a record's sequence names in its direction's cipher: the
/// sequence little-endian in the last eight of twelve bytes, as Noise
/// writes a ChaChaPoly nonce.
fn nonce_of(seq: u64) -> Nonce {
    let mut nonce = [0u8; 12];
    nonce[4..].copy_from_slice(&seq.to_le_bytes());
    Nonce::from(nonce)
}

fn seal_with(key: &[u8; 32], seq: u64, ad: &[u8], plain: &[u8]) -> Result<Vec<u8>> {
    ChaCha20Poly1305::new(Key::from_slice(key))
        .encrypt(
            &nonce_of(seq),
            Payload {
                msg: plain,
                aad: ad,
            },
        )
        .map_err(|_| "a record could not be sealed".to_string())
}

fn open_with(key: &[u8; 32], seq: u64, ad: &[u8], sealed: &[u8]) -> Result<Vec<u8>> {
    ChaCha20Poly1305::new(Key::from_slice(key))
        .decrypt(
            &nonce_of(seq),
            Payload {
                msg: sealed,
                aad: ad,
            },
        )
        .map_err(|_| "a transport message that does not open".to_string())
}

/// The receiving side's replay state: one sliding window of opened
/// sequences for the whole direction, and the closed calls whose
/// records it still refuses. Judged only on sequences that opened, so
/// a refused message marks nothing.
#[derive(Default)]
struct Replay {
    highest: Option<u64>,
    bits: Vec<u64>, // REPLAY_WINDOW bits, by sequence modulo the window
    closed: HashMap<[u8; CALL_SIZE], u64>,
}

impl Replay {
    fn bit(&self, seq: u64) -> bool {
        let at = (seq % REPLAY_WINDOW) as usize;
        self.bits[at / 64] & (1 << (at % 64)) != 0
    }

    fn set_bit(&mut self, seq: u64, on: bool) {
        let at = (seq % REPLAY_WINDOW) as usize;
        if on {
            self.bits[at / 64] |= 1 << (at % 64);
        } else {
            self.bits[at / 64] &= !(1 << (at % 64));
        }
    }

    /// Why `seq` of `call` may not open now, empty when it may.
    /// Nothing is marked here.
    fn refusal(&self, call: &[u8; CALL_SIZE], seq: u64) -> Option<String> {
        if self.closed.contains_key(call) {
            return Some("a record of a closed call".to_string());
        }
        let highest = self.highest?;
        if seq > highest {
            return None;
        }
        if highest - seq >= REPLAY_WINDOW {
            return Some(format!(
                "sequence {seq} has fallen below the replay window at {highest}"
            ));
        }
        if self.bit(seq) {
            return Some(format!("sequence {seq} already opened"));
        }
        None
    }

    /// Records `seq` as opened, re-judging the replay rule, so of two
    /// copies raced at once only the first is delivered, and advances
    /// the window and the closed marks' keeping.
    fn mark_opened(&mut self, call: &[u8; CALL_SIZE], seq: u64) -> Result<()> {
        if let Some(why) = self.refusal(call, seq) {
            return Err(why);
        }
        if self.bits.is_empty() {
            self.bits = vec![0; (REPLAY_WINDOW / 64) as usize];
        }
        match self.highest {
            None => self.highest = Some(seq),
            Some(highest) if seq > highest => {
                if seq - highest >= REPLAY_WINDOW {
                    self.bits.fill(0);
                } else {
                    for n in highest + 1..=seq {
                        self.set_bit(n, false);
                    }
                }
                self.highest = Some(seq);
            }
            _ => {}
        }
        self.set_bit(seq, true);
        if let Some(highest) = self.highest {
            self.closed
                .retain(|_, last| !(highest > *last && highest - *last > REPLAY_WINDOW));
        }
        Ok(())
    }

    /// Keeps `call` as closed at its final record's sequence, until
    /// the window has passed it: a later record naming it is refused
    /// however fresh its sequence looks.
    fn close_call(&mut self, call: &[u8; CALL_SIZE], seq: u64) {
        self.closed.insert(*call, seq);
    }
}

/// One device-to-yard channel after its handshake: the two transport
/// keys, never mutated, and the sender's sequence allocator. Every
/// message is sealed or opened with a fresh copy of its direction's
/// key at the nonce the message names, so any number of calls and
/// stream pieces move at once, none waiting on another.
pub struct Session {
    tag: [u8; TAG_SIZE],
    send_key: [u8; 32],
    recv_key: [u8; 32],
    send_dir: u8,
    recv_dir: u8,
    next: AtomicU64,
    replay: Mutex<Replay>,
}

impl Session {
    fn new(
        tag: [u8; TAG_SIZE],
        send_key: [u8; 32],
        recv_key: [u8; 32],
        send_dir: u8,
        recv_dir: u8,
    ) -> Session {
        Session {
            tag,
            send_key,
            recv_key,
            send_dir,
            recv_dir,
            next: AtomicU64::new(0),
            replay: Mutex::new(Replay::default()),
        }
    }

    /// A session of known keys, for a test that holds both ends: the
    /// device's end sends under `send_key` in the request direction.
    #[cfg(test)]
    fn of_keys(
        tag: [u8; TAG_SIZE],
        send_key: [u8; 32],
        recv_key: [u8; 32],
        device_end: bool,
    ) -> Session {
        let (send_dir, recv_dir) = match device_end {
            true => (DIR_REQUEST, DIR_ANSWER),
            false => (DIR_ANSWER, DIR_REQUEST),
        };
        Session::new(tag, send_key, recv_key, send_dir, recv_dir)
    }

    /// Takes the session's next sequence once, or refuses at the
    /// reserved exhaustion value, so the allocator never wraps back
    /// to a nonce already used under this key.
    fn next_sequence(&self) -> Result<u64> {
        loop {
            let seq = self.next.load(Ordering::SeqCst);
            if seq == u64::MAX {
                return Err("the session's sequences are spent; handshake again".to_string());
            }
            if self
                .next
                .compare_exchange(seq, seq + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(seq);
            }
        }
    }

    /// Seals one transport message of the call `call` at record
    /// `index`, bound to `first` when it is the call's first record in
    /// this direction (`first` is some exactly at index 0): its
    /// sequence is the session's next, taken once and never again.
    pub fn seal(
        &self,
        call: &[u8; CALL_SIZE],
        index: u64,
        first: Option<&FirstBinding>,
        plain: &[u8],
    ) -> Result<Vec<u8>> {
        if plain.len() > MAX_RECORD_PLAIN {
            return Err(format!(
                "a record holds at most {MAX_RECORD_PLAIN} bytes of plaintext, not {}",
                plain.len()
            ));
        }
        if (index == 0) != first.is_some() {
            return Err("record 0 alone carries the first binding".to_string());
        }
        let seq = self.next_sequence()?;
        let ad = associated(&self.tag, self.send_dir, call, seq, index, first);
        let sealed = seal_with(&self.send_key, seq, &ad, plain)?;
        let mut msg = Vec::with_capacity(HEADER_SIZE + sealed.len());
        msg.extend_from_slice(&self.tag);
        msg.extend_from_slice(call);
        msg.extend_from_slice(&seq.to_be_bytes());
        msg.extend_from_slice(&index.to_be_bytes());
        msg.extend_from_slice(&sealed);
        Ok(msg)
    }

    /// Opens one transport message that names the call `want` (none
    /// takes the message's own word, for a caller that has no
    /// expectation yet) and answers its record index and plaintext.
    /// The clear header is checked against the session, the call and
    /// the first binding before a cipher is spent on it, so a message
    /// carried to the wrong session or call is refused unopened; a
    /// header rewritten in transit, or a first record whose outer
    /// path or marker was rewritten, fails the opening itself. A
    /// refusal marks nothing, so the honest message that follows
    /// still opens.
    pub fn open(
        &self,
        msg: &[u8],
        want: Option<&[u8; CALL_SIZE]>,
        first: Option<&FirstBinding>,
    ) -> Result<(u64, Vec<u8>)> {
        if msg.len() < HEADER_SIZE + 16 {
            return Err(format!(
                "a transport message is {} bytes, too short to open",
                msg.len()
            ));
        }
        let tag: &[u8; TAG_SIZE] = msg[..TAG_SIZE].try_into().expect("a tag's length");
        if *tag != self.tag {
            return Err("a transport message of no session this one knows".to_string());
        }
        let call: &[u8; CALL_SIZE] = msg[TAG_SIZE..TAG_SIZE + CALL_SIZE]
            .try_into()
            .expect("a call id's length");
        if let Some(want) = want
            && call != want
        {
            return Err("a transport message of another call".to_string());
        }
        let seq = u64::from_be_bytes(
            msg[HEADER_SIZE - 16..HEADER_SIZE - 8]
                .try_into()
                .expect("a sequence's length"),
        );
        let index = u64::from_be_bytes(
            msg[HEADER_SIZE - 8..HEADER_SIZE]
                .try_into()
                .expect("an index's length"),
        );
        if seq == u64::MAX {
            return Err("a transport message at the reserved sequence".to_string());
        }
        if (index == 0) != first.is_some() {
            return Err("record 0 alone carries the first binding".to_string());
        }
        if let Some(why) = self
            .replay
            .lock()
            .expect("the replay state")
            .refusal(call, seq)
        {
            return Err(why);
        }
        let ad = associated(tag, self.recv_dir, call, seq, index, first);
        let plain = open_with(&self.recv_key, seq, &ad, &msg[HEADER_SIZE..])?;
        self.replay
            .lock()
            .expect("the replay state")
            .mark_opened(call, seq)?;
        Ok((index, plain))
    }

    /// Seals a whole call or answer's joined plaintext as a record
    /// run: the plaintext chopped into records of at most
    /// [`MAX_RECORD_PLAIN`], in order, the first bound to `first`, and
    /// the sealed final record closing the run. An empty plaintext
    /// still travels as one empty record, so the run of a call always
    /// holds its first.
    pub fn seal_whole(
        self: &Arc<Self>,
        call: &[u8; CALL_SIZE],
        first: &FirstBinding,
        plain: &[u8],
    ) -> Result<Vec<u8>> {
        let mut sealer = CallSealer::new(self.clone(), call, first);
        let mut run = sealer.seal(plain)?;
        run.extend_from_slice(&sealer.seal_final()?);
        Ok(run)
    }

    /// Opens a whole call or answer's record run and joins its
    /// records in the order they stand: each record's index must be
    /// the next its call expects here, and the run must close with
    /// its final record — a run that ends without it is truncated
    /// and nothing of it is answered. The first record is bound to
    /// `first`, and the run must hold at least that one.
    pub fn open_whole(
        self: &Arc<Self>,
        run: &[u8],
        call: &[u8; CALL_SIZE],
        first: &FirstBinding,
    ) -> Result<Vec<u8>> {
        let msgs = split_records(run)?;
        if msgs.is_empty() {
            return Err("an enveloped call with no first record".to_string());
        }
        let mut opener = CallOpener::new(self.clone(), call, first);
        let mut plain = Vec::new();
        for (at, msg) in msgs.iter().enumerate() {
            let (piece, last) = opener.open(msg)?;
            if last && at != msgs.len() - 1 {
                return Err("a record stands after a call's final record".to_string());
            }
            plain.extend_from_slice(&piece);
        }
        opener.close()?;
        Ok(plain)
    }
}

/// A final record's plaintext for `data_count` data records holding
/// `total` bytes between them.
fn final_plain(data_count: u64, total: u64) -> [u8; 16] {
    let mut plain = [0u8; 16];
    plain[..8].copy_from_slice(&data_count.to_be_bytes());
    plain[8..].copy_from_slice(&total.to_be_bytes());
    plain
}

/// Whether `plain`, opened at `index` with `total` data bytes before
/// it, is the final record it closes: sixteen bytes naming its own
/// index as the data count and the length that stands before it.
fn is_final(plain: &[u8], index: u64, total: u64) -> bool {
    plain.len() == 16
        && u64::from_be_bytes(plain[..8].try_into().expect("a count's length")) == index
        && u64::from_be_bytes(plain[8..].try_into().expect("a length's length")) == total
}

/// Seals one call's records in one direction, in the order its
/// plaintext is handed over: a whole call's run at once, or a streamed
/// answer as it flushes. Its index counts the call's records from
/// zero and its final record closes the call; the session's sequences
/// it takes as it seals, so calls seal at once in any order.
pub struct CallSealer {
    session: Arc<Session>,
    call: [u8; CALL_SIZE],
    first: FirstBinding,
    index: u64, // the next record's index; the data records sealed
    total: u64, // the data records' plaintext bytes
    done: bool, // the final record is sealed
}

impl CallSealer {
    /// Starts sealing a call's records: `first` binds the first
    /// record, as [`Session::seal`]'s does.
    pub fn new(session: Arc<Session>, call: &[u8; CALL_SIZE], first: &FirstBinding) -> CallSealer {
        CallSealer {
            session,
            call: *call,
            first: first.clone(),
            index: 0,
            total: 0,
            done: false,
        }
    }

    /// Seals `chunk` as the call's next data records, chopped to
    /// [`MAX_RECORD_PLAIN`], and answers them framed as a record run.
    /// An empty chunk seals nothing yet.
    pub fn seal(&mut self, chunk: &[u8]) -> Result<Vec<u8>> {
        if self.done {
            return Err("a call sealed past its final record".to_string());
        }
        let mut run = Vec::new();
        let mut rest = chunk;
        while !rest.is_empty() {
            let mut n = rest.len().min(MAX_RECORD_PLAIN);
            // A data piece that would itself read as the final record
            // for its place is split, so no honest data is ever taken
            // for the close: the receiver's one test stays unambiguous.
            if is_final(&rest[..n], self.index, self.total) {
                n = 8;
            }
            self.seal_one(&mut run, &rest[..n])?;
            rest = &rest[n..];
        }
        Ok(run)
    }

    fn seal_one(&mut self, run: &mut Vec<u8>, plain: &[u8]) -> Result<()> {
        let msg = self.session.seal(
            &self.call,
            self.index,
            match self.index {
                0 => Some(&self.first),
                _ => None,
            },
            plain,
        )?;
        frame_record(run, &msg);
        self.index += 1;
        self.total += plain.len() as u64;
        Ok(())
    }

    /// Seals the call's final record and answers it framed: its index
    /// is the data count, its plaintext that count and the data
    /// records' total length. A call that has sealed no data yet seals
    /// one empty record first, so its run always holds a first record
    /// to bind.
    pub fn seal_final(&mut self) -> Result<Vec<u8>> {
        if self.done {
            return Err("a call sealed past its final record".to_string());
        }
        let mut run = Vec::new();
        if self.index == 0 {
            self.seal_one(&mut run, &[])?;
        }
        let msg = self.session.seal(
            &self.call,
            self.index,
            None,
            &final_plain(self.index, self.total),
        )?;
        frame_record(&mut run, &msg);
        self.done = true;
        Ok(run)
    }
}

/// Opens one call's records in one direction as they arrive, each at
/// the index the last left off: a whole run's records in their order,
/// or a streamed answer's as its pieces cross. Records before a gap
/// stand delivered; the record at a gap, and any after it, open as
/// nothing.
pub struct CallOpener {
    session: Arc<Session>,
    call: [u8; CALL_SIZE],
    first: FirstBinding,
    index: u64, // the next record's index
    total: u64, // the data records' plaintext bytes
    done: bool, // the final record opened
}

impl CallOpener {
    /// Starts opening a call's records: `first` is what the first
    /// record must be bound to, as [`Session::open`]'s is.
    pub fn new(session: Arc<Session>, call: &[u8; CALL_SIZE], first: &FirstBinding) -> CallOpener {
        CallOpener {
            session,
            call: *call,
            first: first.clone(),
            index: 0,
            total: 0,
            done: false,
        }
    }

    /// Opens one transport message of the call: its plaintext, and
    /// whether it was the final record, which carries no plaintext of
    /// the call's own. A record at any index but the next expected is
    /// refused at the gap and marks nothing: the honest record that
    /// belongs there still opens after it.
    pub fn open(&mut self, msg: &[u8]) -> Result<(Vec<u8>, bool)> {
        if self.done {
            return Err("a record stands after a call's final record".to_string());
        }
        // The index stands in the clear header: a record at any other
        // index is the gap, refused before any opening is spent on
        // it, so its sequence is unmarked and it still opens in its
        // place.
        if let Some((_, _, index)) = message_header(msg)
            && index != self.index
        {
            return Err(format!(
                "a sealed record stands at an index its call does not expect: record {index} where {} was due",
                self.index
            ));
        }
        let (index, plain) = self.session.open(
            msg,
            Some(&self.call),
            match self.index {
                0 => Some(&self.first),
                _ => None,
            },
        )?;
        if index != self.index {
            return Err(format!(
                "a sealed record stands at an index its call does not expect: record {index} where {} was due",
                self.index
            ));
        }
        if is_final(&plain, index, self.total) {
            self.done = true;
            if let Some((_, seq, _)) = message_header(msg) {
                self.session
                    .replay
                    .lock()
                    .expect("the replay state")
                    .close_call(&self.call, seq);
            }
            return Ok((Vec::new(), true));
        }
        self.index += 1;
        self.total += plain.len() as u64;
        Ok((plain, false))
    }

    /// Ends the call at its outer ending: nothing once the final
    /// record opened, and a truncation while it has not — the part
    /// that arrived is reported truncated, never read as a short
    /// complete answer.
    pub fn close(&self) -> Result<()> {
        match self.done {
            true => Ok(()),
            false => Err("a sealed stream ended without its final record".to_string()),
        }
    }
}

/// A transport message's clear header: its call, sequence and record
/// index, none for a message too short to hold one.
fn message_header(msg: &[u8]) -> Option<([u8; CALL_SIZE], u64, u64)> {
    if msg.len() < HEADER_SIZE {
        return None;
    }
    Some((
        msg[TAG_SIZE..TAG_SIZE + CALL_SIZE]
            .try_into()
            .expect("a call id's length"),
        u64::from_be_bytes(
            msg[HEADER_SIZE - 16..HEADER_SIZE - 8]
                .try_into()
                .expect("a sequence's length"),
        ),
        u64::from_be_bytes(
            msg[HEADER_SIZE - 8..HEADER_SIZE]
                .try_into()
                .expect("an index's length"),
        ),
    ))
}

/// Appends one transport message to a record run as it travels: a
/// four-byte big-endian length, then the message. A run of records is
/// the body of a whole enveloped call or answer.
pub fn frame_record(run: &mut Vec<u8>, msg: &[u8]) {
    run.extend_from_slice(&(msg.len() as u32).to_be_bytes());
    run.extend_from_slice(msg);
}

/// Splits a record run into its transport messages. A length that
/// runs past the run, or bytes left over, are no run; a zero-length
/// record is the router's beat, no message, and is dropped here,
/// before any opening, as a device drops it.
pub fn split_records(run: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut out = Vec::new();
    let mut rest = run;
    while !rest.is_empty() {
        if rest.len() < 4 {
            return Err("a record run ends inside a length".to_string());
        }
        let n = u32::from_be_bytes(rest[..4].try_into().expect("a length's length")) as usize;
        rest = &rest[4..];
        if n > rest.len() {
            return Err(format!(
                "a record claims {n} bytes of a run with {} left",
                rest.len()
            ));
        }
        if n > 0 {
            out.push(rest[..n].to_vec());
        }
        rest = &rest[n..];
    }
    Ok(out)
}

/// Reads the transport messages of a record run as they cross, a
/// streamed answer's pieces among them: a four-byte length, then that
/// many bytes of one message, the record length and not the carrier's
/// framing saying where a message ends. A zero-length record is the
/// router's beat, framing and no message: it is read past before any
/// opening, so one forged, dropped or replayed alters and reveals no
/// inner byte.
pub struct RecordReader<R> {
    read: R,
}

impl<R: Read> RecordReader<R> {
    /// Starts reading records from `read`.
    pub fn new(read: R) -> RecordReader<R> {
        RecordReader { read }
    }

    /// The run's next transport message: none at a record boundary
    /// with nothing left. An ending inside a length or a record is a
    /// truncation, as an outer ending without the final record is; a
    /// read that gives up for its timeout says so as it did, so a
    /// stream's silence is told from its ending.
    pub fn next_message(&mut self) -> std::io::Result<Option<Vec<u8>>> {
        loop {
            let mut len = [0u8; 4];
            if !read_exact_or_eof(&mut self.read, &mut len)? {
                return Ok(None);
            }
            let n = u32::from_be_bytes(len) as usize;
            if n == 0 {
                continue; // the beat: no message, nothing to open
            }
            if n > 65535 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("a record of {n} bytes passes the Noise bound"),
                ));
            }
            let mut msg = vec![0u8; n];
            self.read.read_exact(&mut msg).map_err(|err| match err {
                err if err.kind() == std::io::ErrorKind::UnexpectedEof => std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "a record run ends inside a record",
                ),
                err => err,
            })?;
            return Ok(Some(msg));
        }
    }
}

/// Reads `buf` whole: false where the stream ended before its first
/// byte, and a truncation where it ended inside it.
fn read_exact_or_eof(read: &mut impl Read, buf: &mut [u8]) -> std::io::Result<bool> {
    let mut filled = 0;
    while filled < buf.len() {
        match read.read(&mut buf[filled..]) {
            Ok(0) if filled == 0 => return Ok(false),
            Ok(0) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "a record run ends inside a length",
                ));
            }
            Ok(n) => filled += n,
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
            Err(err) => return Err(err),
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> ChannelKey {
        // A key pair of one byte repeated is no pair snow will sign
        // with: the public half must be the private's own. Tests that
        // need a working pair generate one; this is for encodings.
        ChannelKey {
            private: [byte; 32],
            public: [byte; 32],
        }
    }

    fn pair() -> (ChannelKey, ChannelKey) {
        (
            ChannelKey::generate().unwrap(),
            ChannelKey::generate().unwrap(),
        )
    }

    /// Both ends of one session over fixed keys, as a handshake
    /// between them founds: the device's end and the yard's.
    fn ends() -> (Arc<Session>, Arc<Session>) {
        let tag = [7u8; TAG_SIZE];
        let device_to_yard = [3u8; 32];
        let yard_to_device = [5u8; 32];
        (
            Arc::new(Session::of_keys(tag, device_to_yard, yard_to_device, true)),
            Arc::new(Session::of_keys(tag, yard_to_device, device_to_yard, false)),
        )
    }

    #[test]
    fn a_channel_public_key_round_trips_and_no_other_text_does() {
        let (_, yard) = pair();
        let text = channel_public(&yard.public);
        assert_eq!(text.len(), 43);
        assert_eq!(parse_channel_public(&text).unwrap(), yard.public);
        for bad in ["", "AAAA", &format!("{text}="), &format!(" {text}"), "!!!!"] {
            assert!(parse_channel_public(bad).is_err(), "{bad:?}");
        }
        let _ = key(1); // the encoding helper's pair stands unused here
    }

    #[test]
    fn a_pairing_opens_at_the_yard_it_was_made_for_and_nowhere_else() {
        let (device, yard) = pair();
        let other = ChannelKey::generate().unwrap();
        let (first, opening) = start_pairing(&device.private, &yard.public, "K7QM-2XPD").unwrap();
        // Neither the code nor the device key crosses readable.
        assert!(!first.windows(8).any(|w| w == b"K7QM-2XPD"));
        let opened = open_pairing(&yard.private, &first).unwrap();
        assert_eq!(opened.code, "K7QM-2XPD");
        assert_eq!(opened.device_key, channel_public(&device.public));
        assert!(open_pairing(&other.private, &first).is_err());
        let reply = opened.seal_answer(b"{\"token\":\"yardr-x\"}").unwrap();
        assert_eq!(
            opening.open_answer(&reply).unwrap(),
            b"{\"token\":\"yardr-x\"}"
        );
        // A second opening spends nothing of the first's state.
        let (first2, opening2) = start_pairing(&device.private, &yard.public, "K7QM-2XPD").unwrap();
        let opened2 = open_pairing(&yard.private, &first2).unwrap();
        let reply2 = opened2.seal_answer(b"two").unwrap();
        assert_eq!(opening2.open_answer(&reply2).unwrap(), b"two");
    }

    #[test]
    fn a_channel_handshake_founds_a_session_both_ends_share() {
        let (device, yard) = pair();
        let (first, opening) = start_channel(&device.private, &yard.public).unwrap();
        let opened = open_channel(&yard.private, &first).unwrap();
        assert_eq!(opened.device_key, channel_public(&device.public));
        let tag = random16().unwrap();
        let (reply, yard_end) = opened.seal_answer(&tag).unwrap();
        let device_end = opening.open_answer(&reply).unwrap();
        let _ = &yard_end;
        // One whole call each way, under the keys the handshake split.
        let call = random16().unwrap();
        let binding = FirstBinding::plain();
        let run = device_end
            .seal_whole(&call, &binding, b"the inner call")
            .unwrap();
        assert_eq!(
            yard_end.open_whole(&run, &call, &binding).unwrap(),
            b"the inner call"
        );
        let back = random16().unwrap();
        let run = yard_end
            .seal_whole(&back, &binding, b"the inner answer")
            .unwrap();
        assert_eq!(
            device_end.open_whole(&run, &back, &binding).unwrap(),
            b"the inner answer"
        );
        // A handshake to another yard's key does not open here.
        let stranger = ChannelKey::generate().unwrap();
        let (first, _) = start_channel(&device.private, &stranger.public).unwrap();
        assert!(open_channel(&yard.private, &first).is_err());
    }

    #[test]
    fn records_open_out_of_sealing_order_and_only_for_their_own_call() {
        let (device, yard) = ends();
        let binding = FirstBinding::plain();
        let one = [1u8; CALL_SIZE];
        let two = [2u8; CALL_SIZE];
        // Two calls sealed in one session, opened in the other order:
        // explicit sequences, not the handshake's counters.
        let run_one = device.seal_whole(&one, &binding, b"one").unwrap();
        let big = vec![b'x'; MAX_RECORD_PLAIN * 2 + 5];
        let run_two = device.seal_whole(&two, &binding, &big).unwrap();
        assert_eq!(yard.open_whole(&run_two, &two, &binding).unwrap(), big);
        assert_eq!(yard.open_whole(&run_one, &one, &binding).unwrap(), b"one");
        // A record of one call carried to the other is refused
        // unopened, and marks nothing for the honest record.
        let (device, yard) = ends();
        let run = device.seal_whole(&one, &binding, b"one").unwrap();
        let msgs = split_records(&run).unwrap();
        let (index, _) = yard.open(&msgs[0], Some(&one), Some(&binding)).unwrap();
        assert_eq!(index, 0);
        let (device, yard) = ends();
        let run = device.seal_whole(&one, &binding, b"one").unwrap();
        let msgs = split_records(&run).unwrap();
        assert!(yard.open(&msgs[0], Some(&two), Some(&binding)).is_err());
        assert!(yard.open(&msgs[0], Some(&one), Some(&binding)).is_ok());
    }

    #[test]
    fn a_replayed_or_rewritten_record_is_refused_and_marks_nothing() {
        let (device, yard) = ends();
        let binding = FirstBinding::plain();
        let call = [9u8; CALL_SIZE];
        let run = device.seal_whole(&call, &binding, b"hello").unwrap();
        let msgs = split_records(&run).unwrap();
        let mut opener = CallOpener::new(yard.clone(), &call, &binding);
        let (plain, last) = opener.open(&msgs[0]).unwrap();
        assert_eq!(plain, b"hello");
        assert!(!last);
        // The same record again: refused by the window, and the call
        // still completes on its honest final record.
        assert!(opener.open(&msgs[0]).is_err());
        let (_, last) = opener.open(&msgs[1]).unwrap();
        assert!(last);
        opener.close().unwrap();
        // A record of the closed call, however fresh, is refused.
        let late = device.seal(&call, 5, None, b"late").unwrap();
        assert!(yard.open(&late, Some(&call), None).is_err());
        // A rewritten sequence fails the opening and burns nothing:
        // the honest message with that sequence still opens.
        let (device, yard) = ends();
        let honest = device.seal(&call, 0, Some(&binding), b"honest").unwrap();
        let mut forged = honest.clone();
        forged[HEADER_SIZE - 1] ^= 1; // another record index in the clear
        assert!(yard.open(&forged, Some(&call), Some(&binding)).is_err());
        assert!(yard.open(&honest, Some(&call), Some(&binding)).is_ok());
    }

    #[test]
    fn a_gap_is_refused_and_a_truncated_run_is_no_whole() {
        let (device, yard) = ends();
        let binding = FirstBinding::plain();
        let call = [4u8; CALL_SIZE];
        let big = vec![b'y'; MAX_RECORD_PLAIN + 1];
        let run = device.seal_whole(&call, &binding, &big).unwrap();
        let msgs = split_records(&run).unwrap();
        assert!(msgs.len() >= 3);
        // The second record first: a gap, refused before any opening,
        // and the honest first record still opens after it.
        let mut opener = CallOpener::new(yard.clone(), &call, &binding);
        assert!(opener.open(&msgs[1]).is_err());
        assert!(opener.open(&msgs[0]).is_ok());
        // A run with no final record opens as nothing whole.
        let (device, yard) = ends();
        let run = device.seal_whole(&call, &binding, b"part").unwrap();
        let msgs = split_records(&run).unwrap();
        assert!(
            yard.open_whole(
                &msgs[..1].iter().fold(Vec::new(), |mut run, msg| {
                    frame_record(&mut run, msg);
                    run
                }),
                &call,
                &binding
            )
            .is_err()
        );
    }

    #[test]
    fn the_first_record_binds_the_path_and_the_marker() {
        let (device, yard) = ends();
        let call = [6u8; CALL_SIZE];
        let marked = FirstBinding::marked("long-lived");
        let run = device.seal_whole(&call, &marked, b"events").unwrap();
        assert_eq!(yard.open_whole(&run, &call, &marked).unwrap(), b"events");
        // A marker dropped or added between the ends fails the call.
        let (device, yard) = ends();
        let run = device.seal_whole(&call, &marked, b"events").unwrap();
        assert!(
            yard.open_whole(&run, &call, &FirstBinding::plain())
                .is_err()
        );
        let (device, yard) = ends();
        let run = device
            .seal_whole(&call, &FirstBinding::plain(), b"events")
            .unwrap();
        assert!(yard.open_whole(&run, &call, &marked).is_err());
    }

    /// The yard's own transport (Go, flynn/noise) over fixed keys:
    /// its sealed bytes open here, and this device's sealing of the
    /// same record is byte for byte its. Made by a yardkey test over
    /// the same tag, keys, call and binding, then removed there.
    #[test]
    fn the_yards_sealed_record_opens_and_seals_the_same() {
        fn unhex(text: &str) -> Vec<u8> {
            (0..text.len())
                .step_by(2)
                .map(|at| u8::from_str_radix(&text[at..at + 2], 16).unwrap())
                .collect()
        }
        let tag: [u8; TAG_SIZE] = unhex("000102030405060708090a0b0c0d0e0f")
            .try_into()
            .unwrap();
        let send: [u8; 32] =
            unhex("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
                .try_into()
                .unwrap();
        let recv: [u8; 32] =
            unhex("6465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f80818283")
                .try_into()
                .unwrap();
        let call: [u8; CALL_SIZE] = unhex("101112131415161718191a1b1c1d1e1f")
            .try_into()
            .unwrap();
        let binding = FirstBinding::marked("long-lived");
        let device = Session::of_keys(tag, send, recv, true);
        let yard = Session::of_keys(tag, recv, send, false);
        let theirs = unhex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0000000000000000000000000000000070dd2e5dc2c6dfb0610530388fbe21c563177fcff3566fa69816",
        );
        let ours = device
            .seal(&call, 0, Some(&binding), b"hello yard")
            .unwrap();
        assert_eq!(ours, theirs);
        let (index, plain) = yard.open(&theirs, Some(&call), Some(&binding)).unwrap();
        assert_eq!((index, plain.as_slice()), (0, b"hello yard".as_slice()));
        let back = unhex(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f000000000000000000000000000000007036073b1cc86f27181581f7d4b7cfba8599c7bfde02e0c16f44e8ad",
        );
        let (index, plain) = device.open(&back, Some(&call), Some(&binding)).unwrap();
        assert_eq!((index, plain.as_slice()), (0, b"hello device".as_slice()));
    }

    #[test]
    fn a_beat_is_no_message_and_a_record_run_reads_past_it() {
        let (device, yard) = ends();
        let binding = FirstBinding::plain();
        let call = [8u8; CALL_SIZE];
        let mut run = device.seal_whole(&call, &binding, b"one").unwrap();
        run.extend_from_slice(&[0, 0, 0, 0]); // the router's beat
        run.extend_from_slice(&device.seal_whole(&call, &binding, b"two").unwrap());
        // The second run names a closed call and is refused record by
        // record; the reader still reads past the beat between them.
        let mut reader = RecordReader::new(std::io::Cursor::new(run));
        let mut opener = CallOpener::new(yard.clone(), &call, &binding);
        let mut whole = Vec::new();
        let mut finals = 0;
        while let Some(msg) = reader.next_message().unwrap() {
            match opener.open(&msg) {
                Ok((plain, last)) => {
                    whole.extend_from_slice(&plain);
                    finals += u8::from(last);
                }
                Err(_) => break,
            }
        }
        assert_eq!(whole, b"one");
        assert_eq!(finals, 1);
    }
}
