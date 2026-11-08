//! A pairing code that is typed or read aloud, redeemed through a PAKE: the
//! device's half of the run (the yardr design's sections 3.1 and 6, beads
//! 16 and 17), as bytes only. A code that took no key from a QR has nothing
//! to pin a yard by but the code itself, so the device and the yard run
//! SPAKE2 with it as the password and exchange their channel keys under the
//! key the run derives. Whoever carries the run, and may rewrite every
//! message of it, gets one online guess at the code, spent at the
//! confirmation, and no way to test a recorded run against candidate codes
//! afterwards: that is the property of the protocol (RFC 9382), and the one
//! the yard's peer pairing, whose HMAC a reader can try codes against,
//! lacks.
//!
//! It holds only where the code never crossed the carrier: shown on the
//! yard's own screen or terminal, or read aloud from there. A code the edge
//! handed out, as it does a member's, is known to the edge, which can then
//! sit in the run as either end, so no member pairing is described as
//! covered by this, here or anywhere (`api.rs` and the window keep members
//! on the operator's QR).
//!
//! None of the PAKE is footplate's own: the run is pakery's (RFC 9382's
//! P-256 suite, which is the one the yard pins), and what is here is the
//! yard's pinned choices around it: the identities `A` (the device) and `B`
//! (the yard), no additional authenticated data, the password the cleaned
//! code's eight ASCII bytes with w the SHA-256 of them read as an integer
//! mod the group order (as the yard's library derives it), and the run's
//! key (the first half of the transcript's hash) spent through HKDF-SHA256
//! with no salt to a ChaCha20-Poly1305 key, a fresh 12-byte nonce before
//! each sealed message. The library states it has had no audit, and nothing
//! here claims one for it.
//!
//! The device's half is a run of three states, so that what must follow
//! the yard's confirmation cannot come before it: [`start`] makes the first
//! message, [`Opening::finish`] takes the yard's and makes the device's
//! confirmation, and only [`Confirming::verify`], which holds the yard's
//! confirmation to the code, gives the [`Verified`] run that seals the
//! device's key and opens the answer. The yard's half stands beside it for
//! the stand-in yard of the tests' alone.

use std::fmt;

use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce};
use pakery_core::SharedSecret;
use pakery_core::crypto::{CpaceGroup, Kdf};
use pakery_crypto::{HkdfSha256, P256Group, Spake2P256};
#[cfg(any(test, debug_assertions))]
use pakery_spake2::PartyB;
use pakery_spake2::{PartyA, PartyAState, Spake2Output};
use sha2::{Digest, Sha256};

/// The identities of the two parties, as the yard pins them.
const IDENTITY_A: &[u8] = b"A";
const IDENTITY_B: &[u8] = b"B";
/// The yard's library folds a non-empty AAD into the transcript as well as
/// the key schedule, which a client of the RFC does not: the pin leaves it
/// empty.
const NO_AAD: &[u8] = b"";
/// What the run's key is spent under, one info for each thing sealed.
const DEVICE_KEY_INFO: &str = "yardr pake device key v1";
const ANSWER_INFO: &str = "yardr pake answer v1";
/// A SPAKE2 message is a P-256 point, uncompressed: the tag and two
/// coordinates.
const POINT_SIZE: usize = 65;
const NONCE_SIZE: usize = 12;
/// A code's length, and the characters it is written in: digits and
/// capitals without I, L, O and U, so that none reads or sounds like
/// another.
const CODE_SIZE: usize = 8;
const ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

type Scalar = <P256Group as CpaceGroup>::Scalar;

/// Why a step of the run did not go.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// What was typed is no pairing code: nothing was made or sent.
    NoCode,
    /// The other side's confirmation does not hold: it did not show that
    /// it holds the code, so what answered is not the yard that was
    /// asked, or a message was rewritten on the way.
    Unconfirmed,
    /// A message that is none of the suite's, or a step that could not
    /// be made: a sentence for the log.
    Broken(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoCode => f.write_str(
                "That is no code: a code is eight letters and digits, as the yard printed them.",
            ),
            Error::Unconfirmed => {
                f.write_str("the confirmation does not hold: it did not show the code")
            }
            Error::Broken(why) => f.write_str(why),
        }
    }
}

fn broken(what: &str, err: impl fmt::Display) -> Error {
    Error::Broken(format!("{what}: {err}"))
}

/// A code as it was typed, in the one form both sides compute from, as the
/// yard cleans it: no dash or space, capitals, and the letters a code never
/// has read as the digits they look like. Only ASCII is read: a letter the
/// yard would fold into one of these by Unicode's rules is refused here,
/// before anything is sent, where a different password would be one wrong
/// guess at the yard.
pub fn clean(typed: &str) -> Result<String, Error> {
    let clean: String = typed
        .trim()
        .chars()
        .filter(|c| !matches!(c, '-' | ' '))
        .map(|c| match c.to_ascii_uppercase() {
            'I' | 'L' => '1',
            'O' => '0',
            c => c,
        })
        .collect();
    match clean.len() == CODE_SIZE && clean.chars().all(|c| ALPHABET.contains(c)) {
        true => Ok(clean),
        false => Err(Error::NoCode),
    }
}

/// The run's password scalar for a cleaned code: its SHA-256 as a
/// big-endian integer reduced mod the group order, as the yard's library
/// derives it. The reduction is the library's own (the digest stands in the
/// low half of the 64 bytes it reduces), so no group arithmetic is here.
fn password(clean: &str) -> Result<Scalar, Error> {
    let digest = Sha256::digest(clean.as_bytes());
    let mut wide = [0u8; 64];
    wide[32..].copy_from_slice(&digest);
    P256Group::scalar_from_wide_bytes(&wide).map_err(|err| broken("the code's password", err))
}

/// The system's randomness, in the trait the library draws its scalars by.
fn rng() -> impl rand_core::CryptoRng {
    rand_core::UnwrapErr(getrandom04::SysRng)
}

/// Whether `msg` has the shape of a message of the suite: the 65 bytes of
/// an uncompressed point, which is all the yard writes and all it takes.
/// The library goes on to refuse what is no point of the curve or is its
/// identity, and also takes a compressed encoding, which the yard's
/// transcript would never hold, so the shape is held here first.
fn shaped(msg: &[u8]) -> Result<(), Error> {
    match msg.len() == POINT_SIZE && msg[0] == 0x04 {
        true => Ok(()),
        false => Err(Error::Broken(format!(
            "a pairing message of {} bytes is no uncompressed point of the suite",
            msg.len()
        ))),
    }
}

/// What the run's key seals under `info`: the key HKDF-SHA256 gives with no
/// salt, as a ChaCha20-Poly1305 key.
fn cipher(key: &SharedSecret, info: &str) -> Result<ChaCha20Poly1305, Error> {
    let prk = HkdfSha256::extract(&[], key.as_bytes());
    let okm = HkdfSha256::expand(&prk, info.as_bytes(), 32)
        .map_err(|err| broken("the run's key could not be spent", err))?;
    Ok(ChaCha20Poly1305::new(Key::from_slice(&okm)))
}

/// `plain` sealed under the run's key for `info`: the nonce, then the sealed
/// bytes.
fn seal(key: &SharedSecret, info: &str, plain: &[u8]) -> Result<Vec<u8>, Error> {
    let mut nonce = [0u8; NONCE_SIZE];
    getrandom::fill(&mut nonce).map_err(|err| broken("no randomness to seal with", err))?;
    let sealed = cipher(key, info)?
        .encrypt(&Nonce::from(nonce), plain)
        .map_err(|_| Error::Broken("a message could not be sealed".to_string()))?;
    let mut out = nonce.to_vec();
    out.extend_from_slice(&sealed);
    Ok(out)
}

/// What [`seal`] sealed for `info`, opened. A message that does not open
/// was made under another key or turned on the way.
fn open(key: &SharedSecret, info: &str, sealed: &[u8]) -> Result<Vec<u8>, Error> {
    if sealed.len() < NONCE_SIZE {
        return Err(Error::Broken(format!(
            "a sealed message of {} bytes holds no nonce",
            sealed.len()
        )));
    }
    let (nonce, sealed) = sealed.split_at(NONCE_SIZE);
    cipher(key, info)?
        .decrypt(Nonce::from_slice(nonce), sealed)
        .map_err(|_| Error::Broken("a sealed message that does not open".to_string()))
}

/// A run the device has begun: its first message sent, the yard's second
/// still to come.
pub struct Opening {
    state: PartyAState<Spake2P256>,
}

/// Begins a run on the code as it was typed, and answers the device's
/// first message: the blinded point of the suite, 65 bytes uncompressed.
pub fn start(typed: &str) -> Result<(Vec<u8>, Opening), Error> {
    let w = password(&clean(typed)?)?;
    let (first, state) =
        PartyA::<Spake2P256>::start(&w, IDENTITY_A, IDENTITY_B, NO_AAD, &mut rng())
            .map_err(|err| broken("the first message could not be made", err))?;
    Ok((first, Opening { state }))
}

/// As [`start`], from the scalar a test fixes, to replay a recorded run.
#[cfg(test)]
fn start_with(typed: &str, x: &Scalar) -> Result<(Vec<u8>, Opening), Error> {
    let w = password(&clean(typed)?)?;
    let (first, state) =
        PartyA::<Spake2P256>::start_with_scalar(&w, x, IDENTITY_A, IDENTITY_B, NO_AAD)
            .map_err(|err| broken("the first message could not be made", err))?;
    Ok((first, Opening { state }))
}

impl Opening {
    /// Takes the yard's second message and makes the device's
    /// confirmation of the run. A message that is no point of the suite,
    /// or is its identity, is refused here, before anything is made of it.
    pub fn finish(self, second: &[u8]) -> Result<Confirming, Error> {
        shaped(second)?;
        let run = self
            .state
            .finish(second)
            .map_err(|err| broken("the yard's message does not open", err))?;
        Ok(Confirming { run })
    }
}

/// A run whose confirmation the device has made: the yard's still to be
/// held to the code.
pub struct Confirming {
    run: Spake2Output,
}

impl Confirming {
    /// The device's confirmation: 32 bytes, the HMAC of the transcript
    /// under its confirmation key (RFC 9382 section 4).
    pub fn confirmation(&self) -> &[u8] {
        &self.run.confirmation_mac
    }

    /// Holds the yard's confirmation to the code. Only a run that passes
    /// has a yard at its other end that holds the code, and until then
    /// the device seals nothing and trusts nothing it is answered.
    pub fn verify(self, yard: &[u8]) -> Result<Verified, Error> {
        self.run
            .verify_peer_confirmation(yard)
            .map_err(|_| Error::Unconfirmed)?;
        Ok(Verified {
            key: self.run.into_session_key(),
        })
    }
}

/// A run whose yard has shown it holds the code: the key it derived is
/// the device's to seal its own channel key under and to open the yard's
/// answer with.
pub struct Verified {
    key: SharedSecret,
}

impl Verified {
    /// The device's channel public key sealed under the run's key, for
    /// the yard to record beside the token it issues.
    pub fn seal_key(&self, public: &[u8; 32]) -> Result<Vec<u8>, Error> {
        seal(&self.key, DEVICE_KEY_INFO, public)
    }

    /// Opens the yard's sealed answer to this run: the pairing's answer,
    /// with the yard's channel public key beside the token, readable by
    /// this device alone.
    pub fn open_answer(&self, sealed: &[u8]) -> Result<Vec<u8>, Error> {
        open(&self.key, ANSWER_INFO, sealed)
    }
}

/// A run the yard has answered: its second message sent, the device's
/// confirmation still to hold to the code. The yard's half, and the
/// stand-in yard of the tests'.
#[cfg(any(test, debug_assertions))]
pub struct Responder {
    run: Spake2Output,
    confirmed: bool,
}

/// Answers a run's first message with the yard's second, on the code of
/// the pairing it has open. A first message that is no point of the suite
/// is refused before any confirmation can be tried against it.
#[cfg(any(test, debug_assertions))]
pub fn answer(code: &str, first: &[u8]) -> Result<(Vec<u8>, Responder), Error> {
    shaped(first)?;
    let w = password(&clean(code)?)?;
    let (second, state) =
        PartyB::<Spake2P256>::start(&w, IDENTITY_A, IDENTITY_B, NO_AAD, &mut rng())
            .map_err(|err| broken("the second message could not be made", err))?;
    let run = state
        .finish(first)
        .map_err(|err| broken("the device's message does not open", err))?;
    Ok((
        second,
        Responder {
            run,
            confirmed: false,
        },
    ))
}

#[cfg(any(test, debug_assertions))]
impl Responder {
    /// Holds the device's confirmation to the code and answers the yard's.
    /// One that does not hold is [`Error::Unconfirmed`]: one online guess
    /// at the code, spent.
    pub fn confirm(&mut self, device: &[u8]) -> Result<Vec<u8>, Error> {
        self.run
            .verify_peer_confirmation(device)
            .map_err(|_| Error::Unconfirmed)?;
        self.confirmed = true;
        Ok(self.run.confirmation_mac.clone())
    }

    fn confirmed(&self) -> Result<(), Error> {
        match self.confirmed {
            true => Ok(()),
            false => Err(Error::Broken("the run is not confirmed".to_string())),
        }
    }

    /// Opens the device's sealed channel public key, in a confirmed run
    /// alone.
    pub fn open_key(&self, sealed: &[u8]) -> Result<[u8; 32], Error> {
        self.confirmed()?;
        open(&self.run.session_key, DEVICE_KEY_INFO, sealed)?
            .try_into()
            .map_err(|plain: Vec<u8>| {
                Error::Broken(format!(
                    "a device key of {} bytes is no channel public key",
                    plain.len()
                ))
            })
    }

    /// Seals the pairing's answer for the device that ran this, in a
    /// confirmed run alone.
    pub fn seal_answer(&self, plain: &[u8]) -> Result<Vec<u8>, Error> {
        self.confirmed()?;
        seal(&self.run.session_key, ANSWER_INFO, plain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn unhex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&text[at..at + 2], 16).unwrap())
            .collect()
    }

    /// The device scalar a replay fixes: the 64 bytes 0x00 to 0x3f,
    /// reduced by the library.
    fn fixed() -> Scalar {
        let wide: Vec<u8> = (0u8..64).collect();
        P256Group::scalar_from_wide_bytes(&wide).unwrap()
    }

    /// A run recorded against the yard's own library (spake2-go v0.0.1,
    /// with yardkey's seal and open copied as they stand): the device's
    /// scalar fixed, the yard's random. The device's first message and its
    /// confirmation are what this side makes, so a test holds them; the
    /// yard's second message, its confirmation and its sealed answer are
    /// what the yard's library made, so a test opens them.
    const CODE: &str = "K7QM-2XPD";
    const FIRST: &str = "04adce33bc9da5545c2390ec13b7aee8498e7973255f699db216578be09f083eb5a5b42add86107e0f9ac715f2d96cb0d8d748c60ed4db481b2816b127cb369f47";
    const SECOND: &str = "042b6da207dcfb7366cb3b8e66519da2e7040eb052ea301e0f08e43a3bf42df2ea754d019e0fe4d02c18bffd3379a0cdf4229aa64143536b539c92f1f61ba4f77b";
    const DEVICE_CONFIRMATION: &str =
        "60e84f12df3def1160e41c3d10bffa5312002bed88703cc0a9ca003455ddfd5c";
    const YARD_CONFIRMATION: &str =
        "b1e14ad4c71c7edd351fabbff7d9ca3b4c7a01d3cea4106c36030e0c7df9498a";
    const KEY: &str = "873159305e28cc54cef73b79bbc92277";
    const ANSWER: &str = r#"{"token":"yardr-dev-paired-1","user":"jo","device":"laptop","yard":"south","channel_key":"CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQk"}"#;
    const SEALED_ANSWER: &str = "1665f162ab5ee982b1bd6e332334125d6e83e0465e8cc51ac277d1a3bcc69c11cfe81d124ef4093b1b77f34c2bcd69145e7203c9bc615376ddcbb69f4ea6b190ae4168a4daa618a7918a9d2d25148c2fe44efd5bf89f0d2eabe684be38a3b3964606ad208f601ea6c5f9eee6b8481b16047d99485141101dbfa432b26b04bd70e95c6d884d6b8f10466ddb003bcdbcb8689653e384c1c54b2a68d93355dd15d36b402d";

    #[test]
    fn a_typed_code_is_cleaned_as_the_yard_cleans_it() {
        for (typed, clean_form) in [
            ("K7QM-2XPD", Some("K7QM2XPD")),
            ("k7qm 2xpd", Some("K7QM2XPD")),
            ("K7qm-2xPD", Some("K7QM2XPD")),
            ("  K7QM2XPD\n", Some("K7QM2XPD")),
            // The letters a code never has read as the digits they look like.
            ("1L0O-ILOi", Some("11001101")),
            // Too short, too long, a letter no code has, another script.
            ("K7QM-2XP", None),
            ("K7QM-2XPDK", None),
            ("K7QM-2XPU", None),
            ("", None),
            ("K7QM-2XPÐ", None),
            // Unicode folds this to an I and the yard reads it as 1; a
            // device that read it so would differ only by a wrong guess.
            ("ıııııııı", None),
        ] {
            assert_eq!(
                clean(typed).ok().as_deref(),
                clean_form,
                "{typed:?} is cleaned as the yard cleans it"
            );
        }
        assert_eq!(clean("nope"), Err(Error::NoCode));
    }

    #[test]
    fn a_run_recorded_against_the_yards_library_replays() {
        // The password the run is on: SHA-256 of the cleaned code, mod the
        // group order. A difference in it would show in every message
        // below, the confirmations first.
        assert_eq!(
            hex(&P256Group::scalar_to_bytes(&password("K7QM2XPD").unwrap())),
            "c3462bb133f97384ff3ca127e47ef2fd127b01693cdafd196dde75968e25719c"
        );
        let (first, opening) = start_with(CODE, &fixed()).unwrap();
        assert_eq!(hex(&first), FIRST);
        let confirming = opening.finish(&unhex(SECOND)).unwrap();
        assert_eq!(hex(confirming.confirmation()), DEVICE_CONFIRMATION);
        let verified = confirming.verify(&unhex(YARD_CONFIRMATION)).unwrap();
        // The key the yard sealed its answer under, and the answer it
        // sealed, opened here.
        assert_eq!(hex(verified.key.as_bytes()), KEY);
        let answer = verified.open_answer(&unhex(SEALED_ANSWER)).unwrap();
        assert_eq!(String::from_utf8(answer).unwrap(), ANSWER);
    }

    /// A whole run between the device's half and the yard's, and what each
    /// seals for the other.
    fn run(device_code: &str, yard_code: &str) -> (Confirming, Responder, Vec<u8>) {
        let (first, opening) = start(device_code).unwrap();
        let (second, responder) = answer(yard_code, &first).unwrap();
        let confirming = opening.finish(&second).unwrap();
        (confirming, responder, second)
    }

    #[test]
    fn a_run_gives_the_device_the_yards_answer_and_the_yard_the_devices_key() {
        let (confirming, mut responder, _) = run("K7QM-2XPD", "k7qm2xpd");
        // Nothing is opened or sealed in a run that is not confirmed.
        assert!(responder.open_key(&[0; 60]).is_err());
        assert!(responder.seal_answer(b"{}").is_err());
        let yard = responder.confirm(confirming.confirmation()).unwrap();
        let verified = confirming.verify(&yard).unwrap();
        let device = [5u8; 32];
        assert_eq!(
            responder.open_key(&verified.seal_key(&device).unwrap()),
            Ok(device)
        );
        let answer = responder.seal_answer(b"{\"token\":\"t\"}").unwrap();
        assert_eq!(verified.open_answer(&answer).unwrap(), b"{\"token\":\"t\"}");
        // A message sealed for another purpose does not open as this one.
        assert!(
            responder
                .open_key(&seal(&verified.key, ANSWER_INFO, &device).unwrap())
                .is_err()
        );
        // A key of another length is no channel key.
        assert!(
            responder
                .open_key(&seal(&verified.key, DEVICE_KEY_INFO, &[1; 31]).unwrap())
                .is_err()
        );
    }

    #[test]
    fn a_wrong_code_is_held_to_the_yards_confirmation_and_buys_the_device_nothing() {
        let (confirming, mut responder, _) = run("K7QM-2XPD", "K7QM-2XPE");
        // The yard refuses the device's confirmation, and sends none.
        assert_eq!(
            responder.confirm(confirming.confirmation()),
            Err(Error::Unconfirmed)
        );
        assert!(responder.open_key(&[0; 60]).is_err());
        // A yard that answers on its own, with a confirmation of any
        // bytes, is no yard that holds the code: nothing opens past it.
        assert!(matches!(
            confirming.verify(&[0u8; 32]),
            Err(Error::Unconfirmed)
        ));
    }

    #[test]
    fn a_message_turned_on_the_way_ends_the_run_at_its_next_check() {
        // The yard's second message is another run's: the device's
        // confirmation is of a transcript the yard never held, and the yard
        // refuses it.
        let (first, opening) = start("K7QM-2XPD").unwrap();
        let (_, mut responder) = answer("K7QM-2XPD", &first).unwrap();
        let (other, _) = answer("K7QM-2XPD", &first).unwrap();
        let confirming = opening.finish(&other).unwrap();
        assert_eq!(
            responder.confirm(confirming.confirmation()),
            Err(Error::Unconfirmed)
        );
        // The device's first message is another's: the yard answers a
        // message that is not the one the device made, and refuses the
        // device's confirmation of its own.
        let (_, opening) = start("K7QM-2XPD").unwrap();
        let (elsewhere, _) = start("K7QM-2XPD").unwrap();
        let (second, mut responder) = answer("K7QM-2XPD", &elsewhere).unwrap();
        let confirming = opening.finish(&second).unwrap();
        assert_eq!(
            responder.confirm(confirming.confirmation()),
            Err(Error::Unconfirmed)
        );
        // The yard's confirmation turned: the device refuses it.
        let (confirming, mut responder, _) = run("K7QM-2XPD", "K7QM-2XPD");
        let mut yard = responder.confirm(confirming.confirmation()).unwrap();
        yard[0] ^= 1;
        assert!(matches!(confirming.verify(&yard), Err(Error::Unconfirmed)));
        // A sealed message turned does not open, either way.
        let (confirming, mut responder, _) = run("K7QM-2XPD", "K7QM-2XPD");
        let yard = responder.confirm(confirming.confirmation()).unwrap();
        let verified = confirming.verify(&yard).unwrap();
        let mut sealed = verified.seal_key(&[5; 32]).unwrap();
        *sealed.last_mut().unwrap() ^= 1;
        assert!(responder.open_key(&sealed).is_err());
        let mut sealed = responder.seal_answer(b"{}").unwrap();
        sealed[NONCE_SIZE] ^= 1;
        assert!(verified.open_answer(&sealed).is_err());
        assert!(verified.open_answer(&[0; 4]).is_err());
    }

    #[test]
    fn what_is_no_point_of_the_suite_is_refused_before_the_library_sees_it() {
        let (first, _) = start("K7QM-2XPD").unwrap();
        let mut off_curve = first.clone();
        *off_curve.last_mut().unwrap() ^= 1;
        let mut compressed = vec![0x02];
        compressed.extend_from_slice(&first[1..33]);
        let mut infinity = vec![0x04];
        infinity.extend_from_slice(&[0u8; 64]);
        for (what, message) in [
            ("an empty message", Vec::new()),
            ("a lone byte", vec![0x04]),
            ("a compressed point", compressed),
            ("the identity's one byte", vec![0x00]),
            ("a prefix and zeros", infinity),
            ("a point off the curve", off_curve),
            ("a point too long", [first.clone(), vec![0]].concat()),
        ] {
            assert!(
                start("K7QM-2XPD").unwrap().1.finish(&message).is_err(),
                "{what} is refused by the device"
            );
            assert!(
                answer("K7QM-2XPD", &message).is_err(),
                "{what} is refused by the yard"
            );
        }
    }

    #[test]
    fn a_recorded_run_is_no_oracle_for_the_code() {
        // An observer holds the recorded messages: the two points and the
        // two confirmations. The only check they offer is the yard's, and
        // a fresh run of the yard, whatever code it is on, refuses a
        // recorded confirmation: the right code and a wrong one are told
        // apart by nothing the recording lets a reader do. That the run
        // leaks nothing to test offline is RFC 9382's and the library's
        // vectors', not this test's; this holds the use of it to that.
        for candidate in ["K7QM-2XPD", "K7QM-2XPE", "00000000", "ZZZZ-ZZZZ"] {
            let (_, mut responder) = answer(candidate, &unhex(FIRST)).unwrap();
            assert_eq!(
                responder.confirm(&unhex(DEVICE_CONFIRMATION)),
                Err(Error::Unconfirmed),
                "a recorded confirmation is refused by a new run on {candidate}"
            );
        }
    }
}
