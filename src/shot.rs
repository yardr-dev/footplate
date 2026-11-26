//! The window against a fake yard, for the screenshots. `FOOTPLATE_SHOT`
//! names the state to open at; scripts/shot-linux.sh and scripts/shot-mac.sh
//! pass it through, and docs/building.md names the states. This module is a
//! debug build's alone: a release build neither compiles it nor reads the
//! variable.

use std::fs;
use std::io;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{DateTime, Duration, Local};
use serde_json::json;

use crate::fake::{Live, Posthog, Router, Yard};
use crate::yard::Env;

/// The states the window can be shot in, as `FOOTPLATE_SHOT` names them:
/// the conversation with the yardmaster, chat with the sidebar's groups
/// full: crew working and stopped, a linked yard's people, this yard's
/// people, and unread notices, a long wrapped thread with an
/// image under its newest mail, which holds the whole-text Markdown
/// sample, a long conversation over three days, grouped by sender, with
/// short links, a correction and a code block of one line, that
/// conversation with the pointer posed on the code block, a letter under
/// its head, that conversation with the pointer posed on the block
/// itself too, so its copy control shows, that conversation with a
/// letter's head scrolled to the list's top and the pointer posed on
/// it, so its bar of reactions stops at the list's top, clear of the
/// Screen control in the header, the yardmaster's conversation of 120
/// mails, ninety of them unread and forty of those older than its newest
/// fifty, with three of the assembly's unread after them, opened as the
/// window reads back to the oldest unread mail, so that its row shows no
/// count where the assembly's shows three, an open decision card of a plain
/// ask, that card with a selection posed over its context, that card
/// with one of jo's beads changed, so Lines carries its badge, an open
/// card of a run ask, the frame's card (a short-command run ask with
/// its moves row, its bead's thread and its image under it), that card
/// with the bead's flow served for its head's stage dots, an open card of a teammate invite, the
/// teammates view open on its row with a paired and a waiting
/// teammate, a weighty run ask's card beside a routine one's,
/// a weighty mapping ask's card beside a routine one's, a weighty run
/// ask's card whose weight line carries a pack file's Trojan name, a yard that is
/// stopped, a yard at an
/// address, the settings on each of their sections, the voice section
/// with a voice chosen and its list open, and with a microphone chosen
/// among those a stand-in lists and its list open, the telemetry
/// section on with the install id and the next payload and off with
/// the consent words, the yardmaster's conversation with a voice session
/// standing in it, as a tap on the composer's Voice button makes one, so that
/// the button is Stop, against a stand-in for OpenAI's socket with the
/// caption of what the person has said (two asks, a pause between), and the
/// same conversation with
/// voice on and no session, the button saying Voice, and with a line typed
/// in the composer, the button saying Send, the yards with a
/// far yard's rename form open in its card, a running
/// yard whose server is then stopped, shot reconnecting with a draft in
/// the composer, its Send held, an open card whose note carries a page, an image and a file, that
/// card on a yard with no web view set, its note carrying one page, with the sentence on its
/// row, and several pages, with the sentence once under them, a
/// mail with a page and an image, that mail's image open full size over
/// the window, that mail with its page alone on a yard whose web view
/// answers the edge's login, which no browser opening it carries the token
/// for, so its row says it is behind a login and offers no Open, that mail
/// with its page alone on a far yard whose web view is the browser's address
/// at the edge, so its row carries Open, with the edge's query, and the
/// settings' Connection of that far yard, the first
/// screen's way to a yard at an address, with a
/// pairing's http address refused under its fields, the
/// conversation with one of a
/// linked yard whose mail was refused over the hour, the first screen
/// with a way's fields open after the yard answered, with the line that
/// esc opens it, step 2 of setting up a yard here, posed with a
/// canned doctor of two providers and their model offers, the first
/// screen with the hosted way's fields open and filled, and the away
/// screen of a hosted yard that is signed out, the away screen of a
/// yard at an address that no longer lets the device in, with a way's
/// fields open, the arrivals panel open
/// on the arrivals row with its tracks, chips and the quiet depots
/// folded, the departures board open under the toggle, one
/// arrival's bead open from the panel, its body holding the whole-text
/// Markdown sample, one
/// depot's board open on its row with what landed under its landed
/// station, the Line window open on yardr's depot with its flows
/// named by their source, its first stage's detail and the crew
/// below, that window on a yard-wide rule's stage with its serving
/// picker and a group marked, with a removal refused where a bead stands in the
/// stage, and on a pack's flow, its Stages row's controls under the
/// card's own words for it, the Lines pane against a yard too old
/// to serve it, saying so in plain words, the grants view open on its
/// row with a read, an update and an `--as` grant, and a source's named
/// read call and update call, the Pair a phone pane open on its row
/// with a pairing's QR, code and minutes left, a pairing run ask's card
/// with the door its approval crosses, that approval's QR screen with
/// its QR, code, minutes and End, and its ended state, an open card of a
/// secret ask posed
/// with a value typed, hidden and shown, that card with the yard's
/// refusal of its answer, the card of a harness key's ask, which says where
/// the value goes in the harness's words, an open card of a setup ask whose account
/// stands, posed with a host typed above it, the switcher's list open over four yards, one
/// of them away, that list shut a second after it drew, as a click
/// outside it shuts it, that list over an own and a member hosted yard,
/// the member marked, the settings' yards with a member yard's pair this
/// device pressed, while the operator holds a pairing open at the yard
/// and the edge names its yard user, and with none open, the device
/// paired as the member's yard user, the yardmaster's conversation naming an ask and
/// two bead ids, the cheatsheet open over an open card, the About card open
/// over the yardmaster's conversation, with the yardr the shot script
/// keeps and with none kept, the Feedback form open over it, with a message
/// typed, against a capture of this machine's that stands in for PostHog,
/// and the
/// yardmaster's conversation with the parties' reactions under three
/// mails, with the bar of reactions to tap floating over one of them, the
/// yardmaster's conversation with replies among its mails, each head saying
/// which mail it answers, the
/// yardmaster's conversation with the notice a flows view that did not
/// answer leaves under the mail, the yardmaster's conversation with
/// the bead its chip names open in the drawer beside it, the yard row
/// open on a hook delivery whose photo the yard holds, with a document it
/// did not take under the photo, the yardmaster's screen open over
/// the window, as its Screen control opens it, that card on a
/// hosted yard from before the yard served the screen down the link from
/// its edge, which refuses the read as such, saying the screen is read on
/// the yard's machine, with no Screen offered, that card on a yard that
/// refuses the read in many words, which wrap inside it, and that card on a
/// pane wider than the window, scrolled to the end of its lines.
/// a mapping card ("Who is this?") that holds the mail of a sender no user maps,
/// shown on the card above its options as an outside message the operator
/// reads before a pick, the same card dealt with by "Let in, once" with the
/// mail it released read whole under it, and the yard row open on a hook
/// delivery read whole under its notice,
/// the Decide list with three beads parked for jo's move under the open ask
/// and the history folded, one of the parked beads open on its card with the
/// moves the yard offers, and that card stood at its end, where the moves and
/// the line for their note are, the Lines details of the first of those beads
/// with the same moves under its body, those details stood at their end with
/// the move back picked and its note typed, and the same details on a yard
/// from before the call, which say in one line who moves the stage, the
/// arrivals pane over the rich yard with the depot tree, the head's stats
/// and the tracks, and yardr's default flow open from that tree as its
/// list, a stage's context drawer open on its Group tab with a serving
/// group marked, and one
/// bead served ten plain moves, open on its card stood at its end, folded
/// behind the one control and unfolded, and the Decide list on a yard whose
/// yardr refuses the call that names its arguments, with the words that the
/// yard is too old where the section would stand,
/// The stopped yard answers nothing, as the refused one answers nowhere:
/// both open on a yard that is down, as the posed setup does. The yard at
/// an address is opened from the settings the shot script writes; the
/// rename shot opens that same far yard, with its rename form open in
/// its card; the
/// signed-out yard from settings the state writes itself, against a
/// throwaway config, with a session the edge no longer knows, and the
/// hosted screen's yard from the same settings, with a session the
/// edge still knows; the away
/// yard from settings the state writes itself, against a throwaway
/// config, with a device the yard no longer knows; the
/// switcher's four yards, in both its states, from settings the state
/// writes itself, with adventuretime open, this machine second, plantime
/// live at its listener, and droplet away; the member switcher's own and
/// member hosted yards, from settings the state writes itself, with this
/// machine open; the
/// settings open over a running yard, each state on its section;
/// the page's card stands expanded at
/// its end, with the web view taken as answering, so the row carries Open
/// and run. The mail's conversation opens on the mail, with the web view
/// taken as answering, so the row carries Open; a click on its image
/// opens it over the window, and the image state serves the same mail
/// with it open. The mail-refused state serves that mail with the web view
/// taken as behind a login; the mail-edge state serves it from a far yard,
/// whose settings the state writes itself with the edge's browser address as
/// its web view, taken as answering, with no sandbox origin to ask (the
/// edge serves none, and the shot calls no host); the
/// settings-connection-edge state opens that far yard's settings on their
/// Connection section. The
/// undelivered conversation opens on the row the shot names,
/// with the hour's line under the mail. The answered yard serves the
/// yardmaster's conversation, as the running ones do; the window opens
/// with the way's fields standing, so the yard answers under them. The
/// script answers the switch's yardr for every state, so the status row
/// draws the same on every shot.
pub const STATES: &[&str] = &[
    "yardmaster",
    "crew",
    "thread",
    "column",
    "column-hover",
    "column-copy",
    "column-top",
    "older-unread",
    "plain",
    "plain-select",
    "lines",
    "run",
    "frame",
    "decide",
    "teammates",
    "teammates-list",
    "weighty",
    "weighty-mapping",
    "held-message",
    "delivered-message",
    "hook-message",
    "weighty-trojan",
    "stopped",
    "far",
    "settings",
    "settings-appearance",
    "settings-yards",
    "settings-yards-rename",
    "settings-agents",
    "settings-notifications",
    "settings-connection",
    "settings-connection-edge",
    "settings-voice",
    "settings-voice-list",
    "settings-voice-microphone",
    "settings-telemetry",
    "settings-telemetry-off",
    "voice",
    "voice-ready",
    "voice-draft",
    "reconnecting",
    "page",
    "page-unset",
    "page-unset-many",
    "mail",
    "image",
    "image-menu",
    "mail-refused",
    "mail-edge",
    "refusal",
    "undelivered",
    "answered",
    "setup",
    "setup-terminal",
    "setup-other",
    "setup-linux",
    "setup-copy",
    "hosted",
    "signedout",
    "away",
    "arrivals",
    "departures",
    "arrival",
    "flow",
    "line-window",
    "line-edit",
    "line-sent",
    "line-picker",
    "line-refused",
    "line-pack",
    "line-stage-edit",
    "line-stage-review",
    "line-stage-copy",
    "line-stage-copying",
    "line-stage-copied",
    "line-stage-drift",
    "line-stage-drift-flow",
    "line-stage-drift-drawer",
    "old-lines",
    "grants",
    "pairing",
    "pair-ask",
    "pair-qr",
    "pair-ended",
    "secret",
    "secret-shown",
    "secret-refused",
    "secret-harness",
    "secret-harness-empty",
    "setup-host",
    "switcher",
    "switcher-shut",
    "switcher-member",
    "member-pair",
    "refs",
    "decisions",
    "decisions-filtered",
    "decisions-more",
    "waiting",
    "waiting-folded",
    "waiting-end",
    "waiting-many",
    "waiting-many-open",
    "old-waiting",
    "lines-waiting",
    "lines-waiting-end",
    "lines-help",
    "lines-panel",
    "lines-departures",
    "lines-siding",
    "lines-arrivals",
    "lines-flow",
    "lines-board",
    "lines-context",
    "lines-served",
    "crew-context",
    "crew-prime",
    "keys",
    "about",
    "about-missing",
    "feedback",
    "reactions",
    "reactions-hover",
    "reactions-zoom",
    "reply",
    "notice",
    "drawer",
    "delivery",
    "screen",
    "screen-hosted",
    "screen-refused",
    "screen-wide",
];

/// The image in the `page`, `mail`, `image` and `image-menu` states, and its
/// hash: the yard serves the bytes, and the note or the mail names them.
const IMAGE: &[u8] = include_bytes!("fixtures/shot-page.png");
const IMAGE_HASH: &str = "ca2421063b23c6ff5739c2000b22125bea109764fff6e643eccea0c00f6fe6c3";

/// The overlay the `image` state opens with: the hash it shows, and its
/// bytes for the line while they are not there yet.
pub(crate) const IMAGE_OPEN: (&str, u64) = (IMAGE_HASH, IMAGE.len() as u64);

/// Where the `image-menu` state opens the picture's menu, in the window's
/// pixels: inside the mail's picture, where a right press on it would have
/// been. The picture stands at the same place in the thread at any window
/// size the shots take, since the thread lays out from its top left.
pub(crate) const IMAGE_MENU_AT: (f32, f32) = (600., 360.);

/// The screen the `screen` state opens with: a real Claude Code pane,
/// captured with the `SessionScreen` form, served as the call's answer.
const SCREEN_SHOT: &str = include_str!("fixtures/screen-claude.ansi");

/// The screen the `screen-wide` state opens with: that pane's words laid
/// out as a terminal 200 columns wide draws them, wrapped at its width, its
/// rules and the nudge's band drawn to its edge. The window the shots open
/// is narrower than that, so the card scrolls sideways.
const SCREEN_WIDE_SHOT: &str = include_str!("fixtures/screen-wide.ansi");

/// The overlay the `screen` state opens with: the session it reads, one
/// of the shot yard's crew.
pub(crate) const SCREEN_OPEN: &str = "yardmaster";

/// What a yard from before it served a screen down the link from its edge
/// says to one asked of it there, as it worded it: the `screen-hosted`
/// state's answer, which the card answers with where the screen is read, in
/// no red.
const SCREEN_OVER_THE_LINK: &str = "a session's screen is not read over the link: this request came down the link from the edge. Read it on the yard's machine. The edge saw the device's token with it: remove the device and add it again (yardr user device rm).";

/// What the yard says to a screen asked of it in plain HTTP from another
/// machine, as it words it: the `screen-refused` state's answer, a refusal
/// of many words that stays red and has to wrap inside the card.
const SCREEN_REFUSAL: &str = "a session's screen is taken over https only, or from the yard's own machine: this request reached the yard's listener in plain HTTP from another machine. Put the proxy or tunnel that ends TLS on the yard's machine, or Read it on the yard's machine. The device's token was sent with it and may have crossed a network in the clear: if it did, remove the device and add it again (yardr user device rm).";

/// The mail the `held-message`, `delivered-message` and `hook-message`
/// states show, as the Cloudflare worker posts it and the yard stores it:
/// long enough for its body to run over a few lines, and short of the fold.
fn held_mail() -> String {
    json!({
        "message_id": "<c0ffee@example.net>",
        "from": {"address": "ada@example.net", "name": "Ada Okoye"},
        "to": ["yardmaster@yardr.dev"],
        "subject": "The shed plans, revised",
        "date": "2026-10-13T09:40:00Z",
        "text": "Hello yardmaster,\n\nThe revised shed plans are attached. The north wall is a hand wider than the first drawing said, so the posts move. Could you tell the builders before they cut anything?\n\nThanks, Ada",
        "attachments": [{"name": "shed-plans-rev2.pdf", "size": 184320, "type": "application/pdf"}],
    })
    .to_string()
}

/// The pages in the `page` and `mail` states: the note's, which Open
/// takes to the browser, and a mail's, which Open takes there too, with
/// no run. Named, never served: the yard is asked for neither.
const PAGE_HASH: &str = "abababababababababababababababababababababababababababababababab";
const MAIL_PAGE_HASH: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";

/// The file in the `page` state: the note's third row, under its name,
/// which Open takes to the browser. Named, never served, as the pages.
const FILE_HASH: &str = "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef";

/// The second and third pages in the `page-unset-many` state: the note's
/// rows beside `PAGE_HASH`, named, never served, as its pages.
const PAGE_UNSET_HASH: &str = "1212121212121212121212121212121212121212121212121212121212121212";
const PAGE_UNSET_HASH_2: &str = "3434343434343434343434343434343434343434343434343434343434343434";

/// The fake yard and its private socket directory, which go away with the
/// screenshot run.
pub(super) struct ShotYard {
    yard: std::sync::Arc<Yard>,
    directory: PathBuf,
    /// The hosted yard and the edge in front of it, for `signedout` and
    /// `screen-hosted` alone: the window opens the hosted yard through
    /// the edge, whose session is none the edge knows for the first, and
    /// one it still knows for the second.
    edge: Option<(Yard, Router)>,
    /// The switcher's other live yards, for `switcher` and `switcher-shut`
    /// alone: adventuretime and plantime at their listeners, held from the
    /// start beside this machine.
    extra: Vec<std::sync::Arc<Yard>>,
}

impl ShotYard {
    fn start(root: &Path, state: &str) -> io::Result<(PathBuf, ShotYard)> {
        let directory = (0..1000)
            .find_map(|suffix| {
                let directory =
                    root.join(format!("footplate-shot-{}-{suffix}", std::process::id()));
                match fs::DirBuilder::new().mode(0o700).create(&directory) {
                    Ok(()) => Some(Ok(directory)),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => None,
                    Err(error) => Some(Err(error)),
                }
            })
            .unwrap_or_else(|| {
                Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "too many screenshot yard directories",
                ))
            })?;
        let socket = directory.join("yard.sock");
        let yard = Yard::start_at(&socket);
        populate(&yard, state);
        Ok((
            socket,
            ShotYard {
                yard: std::sync::Arc::new(yard),
                directory,
                edge: None,
                extra: Vec::new(),
            },
        ))
    }
}

impl Drop for ShotYard {
    fn drop(&mut self) {
        self.yard.stop();
        for extra in &self.extra {
            extra.stop();
        }
        let _ = fs::remove_dir_all(&self.directory);
    }
}

/// The run against a fake yard serving `FOOTPLATE_SHOT`'s state, when the
/// variable names one, else the run as ever. The fake yard listens at a
/// socket of its own, and the run's socket points there, so nothing is
/// asked of the real yard. A name it does not know is refused, before any
/// window opens.
pub(super) fn start_if_asked(env: Env) -> (Env, Option<ShotYard>) {
    let state = std::env::var("FOOTPLATE_SHOT").unwrap_or_default();
    if state.is_empty() {
        return (env, None);
    }
    if !STATES.contains(&state.as_str()) {
        eprintln!(
            "footplate: FOOTPLATE_SHOT={state:?} names no state: {}",
            STATES.join(", ")
        );
        std::process::exit(2);
    }
    // The feedback shot's window sends to a capture of this machine's, held
    // for the run: a press on Send in a window left up (scripts/shot-mac.sh
    // hold) goes nowhere else. The tests start none, so none polls its port
    // for the rest of their run.
    if state == "feedback" {
        let _ = POSTHOG.set(Posthog::start());
    }
    // The voice shots' session is made against a stand-in for OpenAI's
    // socket, held for the run in the same way. Only the first taps the
    // button: the others show it before a tap.
    if matches!(state.as_str(), "voice" | "voice-ready" | "voice-draft") {
        let _ = TALKING.set(stand_voice());
    }
    let (env, shot) = start_for_state(env, &state);
    if state == "line-picker"
        && let Some(shot) = shot.as_ref()
    {
        // Let the window read its Line and open the stream before the
        // picker shot's route change: the detail then shows the same
        // change a person would see arrive while the Line is open.
        let yard = shot.yard.clone();
        std::thread::spawn(move || {
            for _ in 0..200 {
                if !yard.asked("Line").is_empty() && !yard.streams_asked().is_empty() {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    yard.answer("Head", r#"{"result": 21}"#);
                    yard.emit(r#"{"seq": 21, "kind": "route_set", "actor": "mckean", "at": "2026-11-14T09:30:00Z", "data": {"was": {"id": "r-3", "group": "builders"}, "is": {"id": "r-3", "group": "assembly"}}}"#);
                    std::thread::sleep(std::time::Duration::from_millis(250));
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        });
    }
    (env, shot)
}

/// The size a shot's window opens at, in pixels, when `SHOT_SIZE` names one
/// (`WIDTHxHEIGHT`, as scripts/shot-linux.sh takes it), else none: the window
/// opens at its usual size. The Mac's window sizes itself, since a shot there
/// drives no pointer to drag it larger; the floor shot asks for the window's
/// own minimum.
pub(super) fn window_size() -> Option<(f32, f32)> {
    size_from(&std::env::var("SHOT_SIZE").unwrap_or_default())
}

/// A size as `SHOT_SIZE` words it: two positive whole numbers joined by an
/// `x`. Anything else is none.
fn size_from(words: &str) -> Option<(f32, f32)> {
    let (width, height) = words.split_once('x')?;
    let (width, height): (u32, u32) = (width.parse().ok()?, height.parse().ok()?);
    (width > 0 && height > 0).then_some((width as f32, height as f32))
}

/// The capture the feedback shot sends to, which stands in for PostHog.
static POSTHOG: std::sync::OnceLock<Posthog> = std::sync::OnceLock::new();

/// The project the feedback shot offers, whatever the constants of the tree
/// are: that capture, with a key and a survey that are none of the real
/// ones. None in every other state.
pub(crate) fn feedback() -> Option<crate::feedback::Project> {
    crate::feedback::Project::at(POSTHOG.get()?.address(), "phc_shot", "shot-survey")
}

/// What the voice shots' session stands against: a stand-in for OpenAI's
/// Live socket on a loopback port, held for the run, as the capture of the
/// feedback shot is. The shots reach no OpenAI and no microphone, and a
/// press in a window left up (scripts/shot-mac.sh hold) goes nowhere else.
struct Talking {
    url: String,
    _socket: crate::voice::stand_in::SocketLive,
}

static TALKING: std::sync::OnceLock<Talking> = std::sync::OnceLock::new();

/// The key the voice shot's session is made with: the stand-in takes it, and
/// no OpenAI has heard of it.
pub(crate) const VOICE_KEY: &str = "sk-shot";

/// Where the voice shots' session stands, which their window starts it against
/// where it taps the Voice button (chrome.rs: the `voice` shot taps it, the
/// others leave it for a hand). None in every other state.
pub(crate) fn voice() -> Option<&'static str> {
    TALKING.get().map(|talking| talking.url.as_str())
}

/// The microphones the microphone shot's machine lists: a stand-in's, named
/// as a Mac names its own, since the names of the machine a picture is taken
/// on are no part of it. The shot's choice is the second.
pub(crate) fn inputs() -> Vec<crate::audio::Input> {
    [
        (
            "MacBook Pro Microphone",
            "coreaudio:BuiltInMicrophoneDevice",
        ),
        (
            "Studio Display Microphone",
            "coreaudio:AppleUSBAudioEngine:Apple Inc.:Studio Display:4D7",
        ),
        (
            "Jabra Evolve2 65",
            "coreaudio:AppleUSBAudioEngine:Jabra:Evolve2 65:0A4F",
        ),
    ]
    .map(|(description, id)| crate::audio::Input {
        id: id.to_string(),
        description: description.to_string(),
    })
    .into()
}

/// The stand-in for the voice shot: a socket that serves a `Live` whose
/// person asks the yardmaster a question once the session has started, and
/// after a pause asks another, so that the caption stands under the head in
/// the middle of the session the shot shows, with both asks in it: the model
/// raises no delegation, so nothing takes the words. The words come with no
/// question mark, as the transcripts of runs 3 to 6 have them: the server's
/// transcript marks none.
fn stand_voice() -> Talking {
    let live = std::sync::Arc::new(Live::start());
    // An hour on, as a session's moment stands in the window's tests: the
    // default is a past one, which would end the session as it starts.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() as i64);
    live.expires_at(now + 3600);
    let socket = crate::voice::stand_in::live_socket(live.clone(), &format!("Bearer {VOICE_KEY}"));
    let url = socket.url.clone();
    std::thread::spawn(move || {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while std::time::Instant::now() < until {
            if live
                .sent()
                .iter()
                .any(|event| event["type"] == "session.start")
            {
                live.person_says("What's blocking the release", 0, 1800);
                // Past the pause that ends an utterance: the second ask is
                // another, and the caption keeps the first beside it.
                live.person_says("And who owns it", 4200, 5400);
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });
    Talking {
        url,
        _socket: socket,
    }
}

/// The run for `state`: the fake yard serves it, and the run's socket
/// points there. For `stopped`, `refusal`, `setup`, `setup-terminal`,
/// `setup-other`, `setup-linux`, `setup-copy` and `hosted` no yard listens:
/// the socket names a directory that holds none, so the window opens on a
/// yard that is down, and the switch says the stopped one is stopped. The
/// refusal shot is the first screen, with the pairing's fields open on it;
/// the setup shot is the first run posed at step 2, the setup-terminal shot
/// at the step that asks about the PATH line, and the setup-other shot at
/// the step that lists what the machine has, with a claude that is not
/// Claude Code; the setup-linux and setup-copy shots are that step on a
/// Linux box, with herdr to install in one click and with herdr's command
/// to copy; the hosted shot is
/// the first screen with the hosted way's fields open on it. The signedout
/// shot opens the hosted yard south through an edge that no longer knows its
/// session, so the window opens on the away screen of a yard that is signed
/// out; the screen-hosted shot opens that same yard through an edge that
/// still knows it, so the window opens on the yard, and the yard's refusal of
/// the screen (it is one from before the screen came down the link) leaves
/// the window with no Screen offered.
/// The away shot opens the yard far at its address with a device the yard no
/// longer knows, so the window opens on the away screen that offers to pair
/// again, with a way's fields open. The page-unset shots open the yard far
/// at its address with a device the yard knows, so the window opens on the
/// yard, whose web view is unset: the card's note carries one page in the
/// first and several in the second. The edge shots open that far yard with
/// the browser's address at the edge saved as its web view: the mail's row
/// carries Open in the first, and the settings stand on Connection in the
/// second.
fn start_for_state(env: Env, state: &str) -> (Env, Option<ShotYard>) {
    if matches!(
        state,
        "stopped"
            | "refusal"
            | "setup"
            | "setup-terminal"
            | "setup-other"
            | "setup-linux"
            | "setup-copy"
            | "hosted"
    ) {
        let mut env = env;
        env.socket = std::env::temp_dir().join(format!(
            "footplate-shot-{}-{state}/yard.sock",
            std::process::id()
        ));
        return (env, None);
    }
    let (socket, mut shot) = match ShotYard::start(&std::env::temp_dir(), state) {
        Ok(shot) => shot,
        Err(error) => {
            eprintln!("footplate: the screenshot yard has nowhere to listen: {error}");
            std::process::exit(1);
        }
    };
    if state == "signedout" || state == "screen-hosted" {
        // The window opens south through the edge, as the sign-in-again
        // test has it: the session stands in its file, and the edge
        // knows it until it is revoked there. Signedout's is revoked,
        // so the window opens on the away screen; the hosted screen's
        // stands, so the window opens on the yard.
        let hosted = Yard::start_tcp();
        populate(&hosted, state);
        let edge = Router::start(&hosted, crate::fake::SESSION);
        if state == "signedout" {
            edge.revoke(crate::fake::SESSION);
        }
        write_hosted_settings(edge.address(), state);
        shot.edge = Some((hosted, edge));
    }
    if state == "away" {
        // The window opens far at its address, as the pair-again test
        // has it: the token stands in its file, and the yard knew the
        // device until its operator removed it there. The yard is held
        // beside the shot's, so its listener stands for the shot.
        let far = Yard::start_tcp();
        populate(&far, state);
        far.add_device("footplate-shot-phone");
        far.remove_device(crate::fake::TOKEN);
        write_away_settings(far.address());
        shot.extra = vec![std::sync::Arc::new(far)];
    }
    if matches!(
        state,
        "page-unset" | "page-unset-many" | "mail-edge" | "settings-connection-edge"
    ) {
        // The window opens far at its address, as the away shot has it:
        // the token stands in its file, and the yard knows the device,
        // so the window opens on the yard, whose web view is unset, or,
        // in the edge states, the browser's address at the edge. The
        // yard is held beside the shot's, so its listener stands for the
        // shot.
        let far = Yard::start_tcp();
        populate(&far, state);
        write_far_settings(far.address(), state);
        shot.extra = vec![std::sync::Arc::new(far)];
    }
    if state == "switcher" || state == "switcher-shut" {
        // The switcher's other live yards, held from the start: adventuretime
        // with two mails and one decision, plantime with twelve mails, so
        // the counts stand one and two figures in their column.
        let adventuretime = Yard::start_tcp();
        switcher_adventuretime(&adventuretime);
        let plantime = Yard::start_tcp();
        switcher_plantime(&plantime);
        write_switcher_settings(adventuretime.address(), plantime.address());
        shot.extra = vec![
            std::sync::Arc::new(adventuretime),
            std::sync::Arc::new(plantime),
        ];
    }
    if state == "switcher-member" {
        // The member shot's hosted yards, held from the start: an own yard
        // and a member yard of another account's behind one edge, each with
        // one mail, so the list marks the member with its operator's
        // handle beside the own yard's bare name.
        let far = Yard::start_tcp();
        populate(&far, state);
        far.answer(
            "UserInbox",
            r#"{"result": {"mail": [{"seq": 11, "from": "yardmaster", "to": "jo", "text": "From afar.", "read": false}], "unread": 1, "last": 11, "from": ["yardmaster"]}}"#,
        );
        far.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
        let edge = Router::start(&far, crate::fake::SESSION);
        write_member_settings(edge.address());
        shot.edge = Some((far, edge));
    }
    if state == "member-pair" {
        // A member's yard of another account's behind an edge that knows
        // their session. The shot opens the settings' yards and its pair
        // this device action; the form asks for a QR the operator opened on
        // the yard's own screen. It sends no pairing request through the
        // edge, whose pairing page can read or rewrite the code and key.
        let far = Yard::start_tcp();
        populate(&far, state);
        far.answer(
            "UserInbox",
            r#"{"result": {"mail": [{"seq": 11, "from": "yardmaster", "to": "eve", "text": "From afar.", "read": false}], "unread": 1, "last": 11, "from": ["yardmaster"]}}"#,
        );
        far.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
        far.answer_for(
            "GetUser",
            "name",
            "eve",
            r#"{"result": {"name": "eve", "cursor": 10, "notification_cursor": 0}}"#,
        );
        let edge = Router::start(&far, crate::fake::SESSION);
        edge.member(crate::fake::SESSION, "eve");
        write_member_pair_settings(edge.address());
        shot.edge = Some((far, edge));
    }
    if state == "reconnecting" {
        // The yard answers, the window reads it, and then its server is
        // stopped: the shot is the reconnecting state. The stop waits for
        // the window's read of the open conversation, and a breath after
        // it for the reply to land, so the shot is the conversation as
        // last seen; past half a minute it stops anyway, and the shot is
        // whatever that is. The event before the stop only marks the
        // stream as heard, so the window notices the stop at once: mail
        // read elsewhere, by nobody shown, changes nothing.
        let yard = shot.yard.clone();
        std::thread::spawn(move || {
            for _ in 0..600 {
                if !yard.asked("Conversation").is_empty() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            std::thread::sleep(std::time::Duration::from_millis(1500));
            yard.emit(
                r#"{"seq": 99, "kind": "mail_read", "data": {"reader": "nobody", "through": 0}}"#,
            );
            std::thread::sleep(std::time::Duration::from_millis(300));
            yard.stop();
        });
    }
    let mut env = env;
    env.socket = socket;
    (env, Some(shot))
}

/// The look the shot script asks for in `SHOT_LOOK`, `light` or `dark`, for
/// the states that write their own settings, so that none is left at the
/// system's own where one was asked.
fn look_asked() -> crate::theme::Look {
    look_named(&std::env::var("SHOT_LOOK").unwrap_or_default())
}

/// The look a name asks for; none, or another, is the system's own, as the
/// settings have it by default.
fn look_named(name: &str) -> crate::theme::Look {
    match name {
        "light" => crate::theme::Look::Light,
        "dark" => crate::theme::Look::Dark,
        _ => crate::theme::Look::Auto,
    }
}

/// The settings the `signedout` and `screen-hosted` shots open: the
/// hosted yard south at the edge's address, with the account it was
/// signed in as, its device token and the session in files beside them,
/// as the sign-in-again test keeps them. The shot script runs against a
/// throwaway config, which keeps no settings; where a settings file
/// already stands, the state is refused rather than overwrite it. The
/// window opens in the look the script asked for.
fn write_hosted_settings(address: &str, state: &str) {
    let Some(file) = crate::settings::file() else {
        eprintln!("footplate: the {state} shot keeps no settings without a home");
        std::process::exit(1);
    };
    if file.exists() {
        eprintln!(
            "footplate: the {state} shot writes its own settings, and {} already stands: shoot it against a throwaway config, as scripts/shot-linux.sh and scripts/shot-mac.sh do",
            file.display()
        );
        std::process::exit(2);
    }
    keep_hosted_settings(&file, address, look_asked(), state);
}

/// The hosted shots' settings, device token and session, in `file`'s
/// directory, with the window in `look`.
fn keep_hosted_settings(file: &Path, address: &str, look: crate::theme::Look, state: &str) {
    let mut south =
        crate::settings::Yard::hosted("south", address, "yard-south.token", "yard-south.session");
    south.account = crate::fake::KRATOS_EMAIL.to_string();
    crate::settings::Settings {
        look,
        ..Default::default()
    }
    .with_yard(None, south, Some(file))
    .and_then(|set| {
        set.opening("south")
            .write(file)
            .map_err(|error| error.to_string())
    })
    .unwrap_or_else(|error| {
        eprintln!("footplate: the {state} shot keeps no settings: {error}");
        std::process::exit(1);
    });
    let edge = crate::api::Address::parse(address).unwrap_or_else(|error| {
        eprintln!("footplate: the {state} shot's edge has no address: {error}");
        std::process::exit(1);
    });
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    crate::yard::token_place(&dir.join("yard-south.token"))
        .and_then(|place| place.keep(crate::fake::TOKEN, &edge))
        .unwrap_or_else(|error| {
            eprintln!("footplate: the {state} shot keeps no token: {error}");
            std::process::exit(1);
        });
    crate::yard::keep_session(&dir.join("yard-south.session"), crate::fake::SESSION, &edge)
        .unwrap_or_else(|error| {
            eprintln!("footplate: the {state} shot keeps no session: {error}");
            std::process::exit(1);
        });
}

/// The settings the `away` shot opens: the yard far at its address,
/// open, with its device token in a file beside them, as the
/// pair-again test keeps them. The shot script runs against a
/// throwaway config, which keeps no settings; where a settings file
/// already stands, the state is refused rather than overwrite it. The
/// window opens in the look the script asked for.
fn write_away_settings(address: &str) {
    let Some(file) = crate::settings::file() else {
        eprintln!("footplate: the away shot keeps no settings without a home");
        std::process::exit(1);
    };
    if file.exists() {
        eprintln!(
            "footplate: the away shot writes its own settings, and {} already stands: shoot it against a throwaway config, as scripts/shot-linux.sh and scripts/shot-mac.sh do",
            file.display()
        );
        std::process::exit(2);
    }
    keep_away_settings(&file, address, look_asked());
}

/// The `away` shot's settings and device token, in `file`'s directory, with
/// the window in `look`.
fn keep_away_settings(file: &Path, address: &str, look: crate::theme::Look) {
    crate::settings::Settings {
        look,
        ..Default::default()
    }
    .with_yard(
        None,
        crate::settings::Yard::far("far", address, "yard-far.token"),
        Some(file),
    )
    .and_then(|set| {
        set.opening("far")
            .write(file)
            .map_err(|error| error.to_string())
    })
    .unwrap_or_else(|error| {
        eprintln!("footplate: the away shot keeps no settings: {error}");
        std::process::exit(1);
    });
    let edge = crate::api::Address::parse(address).unwrap_or_else(|error| {
        eprintln!("footplate: the away shot's yard has no address: {error}");
        std::process::exit(1);
    });
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    crate::yard::token_place(&dir.join("yard-far.token"))
        .and_then(|place| place.keep(crate::fake::TOKEN, &edge))
        .unwrap_or_else(|error| {
            eprintln!("footplate: the away shot keeps no token: {error}");
            std::process::exit(1);
        });
}

/// The settings the `page-unset`, `page-unset-many`, `mail-edge` and
/// `settings-connection-edge` shots open: the yard far at its address, with
/// its device token in a file beside them, as the away shot keeps them, and
/// the yard knows the device, so the window opens on the yard. The edge
/// shots keep [`EDGE_VIEW`] as its web view. The shot script runs against a
/// throwaway config, which keeps no settings; where a settings file already
/// stands, the state is refused rather than overwrite it. The window
/// opens in the look the script asked for.
fn write_far_settings(address: &str, state: &str) {
    let Some(file) = crate::settings::file() else {
        eprintln!("footplate: the {state} shot keeps no settings without a home");
        std::process::exit(1);
    };
    if file.exists() {
        eprintln!(
            "footplate: the {state} shot writes its own settings, and {} already stands: shoot it against a throwaway config, as scripts/shot-linux.sh and scripts/shot-mac.sh do",
            file.display()
        );
        std::process::exit(2);
    }
    keep_far_settings(&file, address, look_asked(), state);
}

/// The browser's address at the edge, as the edge shots keep it for their
/// far yard's web view: the one a yard behind the edge is entered by, and
/// the name of a yard of the shot's account. The shots ask no host at it.
const EDGE_VIEW: &str = "https://my.yardr.dev/?yard=mac";

/// The far shots' settings and device token, in `file`'s directory, with
/// the window in `look`.
fn keep_far_settings(file: &Path, address: &str, look: crate::theme::Look, state: &str) {
    // The edge shots' yard is entered at the edge's browser address; an
    // empty one leaves the view unset.
    let view = match state.ends_with("-edge") {
        true => EDGE_VIEW,
        false => "",
    };
    crate::settings::Yard::far("far", address, "yard-far.token")
        .with_web_view(view)
        .and_then(|yard| {
            crate::settings::Settings {
                look,
                ..Default::default()
            }
            .with_yard(None, yard, Some(file))
        })
        .and_then(|set| {
            set.opening("far")
                .write(file)
                .map_err(|error| error.to_string())
        })
        .unwrap_or_else(|error| {
            eprintln!("footplate: the {state} shot keeps no settings: {error}");
            std::process::exit(1);
        });
    let edge = crate::api::Address::parse(address).unwrap_or_else(|error| {
        eprintln!("footplate: the {state} shot's yard has no address: {error}");
        std::process::exit(1);
    });
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    crate::yard::token_place(&dir.join("yard-far.token"))
        .and_then(|place| place.keep(crate::fake::TOKEN, &edge))
        .unwrap_or_else(|error| {
            eprintln!("footplate: the {state} shot keeps no token: {error}");
            std::process::exit(1);
        });
}

/// The settings the switcher shots open: four yards in the screenshot's
/// order, adventuretime at its listener and open, this machine at the
/// shot's socket, droplet away, then plantime at its listener. Droplet
/// keeps no token, so it is never reached: the window asks nothing of the
/// network for it. The shot script runs against a throwaway config, which
/// keeps no settings; where a settings file already stands, the state is
/// refused rather than overwrite it. The window opens in the look the script
/// asked for.
fn write_switcher_settings(adventuretime_at: &str, plantime_at: &str) {
    let Some(file) = crate::settings::file() else {
        eprintln!("footplate: the switcher shot keeps no settings without a home");
        std::process::exit(1);
    };
    if file.exists() {
        eprintln!(
            "footplate: the switcher shot writes its own settings, and {} already stands: shoot it against a throwaway config, as scripts/shot-linux.sh and scripts/shot-mac.sh do",
            file.display()
        );
        std::process::exit(2);
    }
    let adventuretime = crate::settings::Yard::far(
        "adventuretime.tail5c8d.ts.net",
        adventuretime_at,
        "yard-adventuretime.token",
    );
    // This machine's yard is the shot's socket.
    let home = crate::settings::Yard::near(crate::settings::HERE, "");
    let plantime = crate::settings::Yard::far("plantime", plantime_at, "yard-plantime.token");
    let droplet =
        crate::settings::Yard::far("droplet", "https://droplet.example.net", "droplet.token");
    crate::settings::Settings {
        look: look_asked(),
        ..Default::default()
    }
    .with_yard(Some(crate::settings::HERE), adventuretime, Some(&file))
    .and_then(|set| set.with_yard(None, home, Some(&file)))
    .and_then(|set| set.with_yard(None, droplet, Some(&file)))
    .and_then(|set| set.with_yard(None, plantime, Some(&file)))
    .and_then(|set| {
        set.opening("adventuretime.tail5c8d.ts.net")
            .write(&file)
            .map_err(|error| error.to_string())
    })
    .unwrap_or_else(|error| {
        eprintln!("footplate: the switcher shot keeps no settings: {error}");
        std::process::exit(1);
    });
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    for (token, at) in [
        ("yard-adventuretime.token", adventuretime_at),
        ("yard-plantime.token", plantime_at),
    ] {
        let edge = crate::api::Address::parse(at).unwrap_or_else(|error| {
            eprintln!("footplate: the switcher shot's yard has no address: {error}");
            std::process::exit(1);
        });
        crate::yard::token_place(&dir.join(token))
            .and_then(|place| place.keep(crate::fake::TOKEN, &edge))
            .unwrap_or_else(|error| {
                eprintln!("footplate: the switcher shot keeps no token: {error}");
                std::process::exit(1);
            });
    }
}

/// The settings the `switcher-member` shot opens: this machine at the
/// shot's socket and open, an own hosted yard and a member yard of
/// another account's at the edge, with the account, its handle and which
/// of its yards each names, and the device tokens and sessions in files
/// beside them. The shot script runs against a throwaway config, which
/// keeps no settings; where a settings file already stands, the state is
/// refused rather than overwrite it. The window opens in the look the
/// script asked for, with the switcher's list standing.
fn write_member_settings(edge_at: &str) {
    let Some(file) = crate::settings::file() else {
        eprintln!("footplate: the switcher-member shot keeps no settings without a home");
        std::process::exit(1);
    };
    if file.exists() {
        eprintln!(
            "footplate: the switcher-member shot writes its own settings, and {} already stands: shoot it against a throwaway config, as scripts/shot-linux.sh and scripts/shot-mac.sh do",
            file.display()
        );
        std::process::exit(2);
    }
    let mut own =
        crate::settings::Yard::hosted("own", edge_at, "yard-own.token", "yard-own.session");
    own.account = crate::fake::KRATOS_EMAIL.to_string();
    own.handle = crate::fake::KRATOS_HANDLE.to_string();
    own.edge_yard = "airy".to_string();
    let mut south =
        crate::settings::Yard::hosted("south", edge_at, "yard-south.token", "yard-south.session");
    south.account = crate::fake::KRATOS_EMAIL.to_string();
    south.handle = crate::fake::KRATOS_HANDLE.to_string();
    south.edge_yard = "alice/lab".to_string();
    crate::settings::Settings {
        look: look_asked(),
        ..Default::default()
    }
    .with_yard(None, own, Some(&file))
    .and_then(|set| set.with_yard(None, south, Some(&file)))
    .and_then(|set| {
        set.opening(crate::settings::HERE)
            .write(&file)
            .map_err(|error| error.to_string())
    })
    .unwrap_or_else(|error| {
        eprintln!("footplate: the switcher-member shot keeps no settings: {error}");
        std::process::exit(1);
    });
    let edge = crate::api::Address::parse(edge_at).unwrap_or_else(|error| {
        eprintln!("footplate: the switcher-member shot's edge has no address: {error}");
        std::process::exit(1);
    });
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    for (token, session) in [
        ("yard-own.token", "yard-own.session"),
        ("yard-south.token", "yard-south.session"),
    ] {
        crate::yard::token_place(&dir.join(token))
            .and_then(|place| place.keep(crate::fake::TOKEN, &edge))
            .unwrap_or_else(|error| {
                eprintln!("footplate: the switcher-member shot keeps no token: {error}");
                std::process::exit(1);
            });
        crate::yard::keep_session(&dir.join(session), crate::fake::SESSION, &edge).unwrap_or_else(
            |error| {
                eprintln!("footplate: the switcher-member shot keeps no session: {error}");
                std::process::exit(1);
            },
        );
    }
}

/// The settings the member pairing shots open: this machine's yard, and a
/// member yard of another account's at the edge, open, with the account,
/// its handle and which of its yards the yard names, and the session in a
/// file beside them. No device token stands: pairing this device is what
/// the shot shows. The shot script runs against a throwaway config, which
/// keeps no settings; where a settings file already stands, the state is
/// refused rather than overwrite it. The window opens in the look the
/// script asked for.
fn write_member_pair_settings(edge_at: &str) {
    let Some(file) = crate::settings::file() else {
        eprintln!("footplate: the member pairing shots keep no settings without a home");
        std::process::exit(1);
    };
    if file.exists() {
        eprintln!(
            "footplate: the member pairing shots write their own settings, and {} already stands: shoot them against a throwaway config, as scripts/shot-linux.sh and scripts/shot-mac.sh do",
            file.display()
        );
        std::process::exit(2);
    }
    let mut south =
        crate::settings::Yard::hosted("south", edge_at, "yard-south.token", "yard-south.session");
    south.account = crate::fake::KRATOS_EMAIL.to_string();
    south.handle = crate::fake::KRATOS_HANDLE.to_string();
    south.edge_yard = "alice/lab".to_string();
    crate::settings::Settings {
        look: look_asked(),
        ..Default::default()
    }
    .with_yard(None, south, Some(&file))
    .and_then(|set| {
        set.opening("south")
            .write(&file)
            .map_err(|error| error.to_string())
    })
    .unwrap_or_else(|error| {
        eprintln!("footplate: the member pairing shots keep no settings: {error}");
        std::process::exit(1);
    });
    let edge = crate::api::Address::parse(edge_at).unwrap_or_else(|error| {
        eprintln!("footplate: the member pairing shots' edge has no address: {error}");
        std::process::exit(1);
    });
    let dir = file.parent().unwrap_or_else(|| Path::new("."));
    crate::yard::keep_session(&dir.join("yard-south.session"), crate::fake::SESSION, &edge)
        .unwrap_or_else(|error| {
            eprintln!("footplate: the member pairing shots keep no session: {error}");
            std::process::exit(1);
        });
}

/// The adventuretime yard of the `switcher` shot: jo's, with two mails
/// and one open decision, so its row carries 3 with
/// "1 decision · 2 mail" in its tooltip.
fn switcher_adventuretime(yard: &Yard) {
    populate(yard, "switcher");
    yard.answer(
        "UserInbox",
        r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "From adventuretime.", "read": false},
            {"seq": 13, "from": "assembly", "to": "jo", "text": "Assembled.", "read": false}], "unread": 2, "last": 13, "from": ["yardmaster", "assembly"]}}"#,
    );
    yard.answer(
        "UserAsks",
        r#"{"result": {"open": [
            {"bead": "bead-a", "bead_title": "Move the links", "topic": "topic:linear-out",
             "ask": 3, "question": "Move them now?", "asker": "yardmaster",
             "since": "2026-10-10T10:00:00Z"}], "decided": []}}"#,
    );
}

/// The plantime yard of the `switcher` shot: jo's, with twelve mails, so
/// its row carries 12 beside adventuretime's 3: one and two figures in the
/// counts' column.
fn switcher_plantime(yard: &Yard) {
    populate(yard, "switcher");
    let mail: Vec<_> = (0..12)
        .map(|n| {
            json!({"seq": 40 + n, "from": "yardmaster", "to": "jo",
                   "text": format!("No. {} from plantime.", n + 1), "read": false})
        })
        .collect();
    yard.answer(
        "UserInbox",
        &json!({"result": {"mail": mail, "unread": 12, "last": 51, "from": ["yardmaster"]}})
            .to_string(),
    );
    yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
}

/// Doctor's answer for the `setup` shot: git and herdr installed, and
/// two installed providers with the labels and the model offers the
/// second step shows. The first run stands at that step with it, having
/// fetched and run nothing.
pub(crate) fn setup_doctor() -> crate::setup::Doctor {
    serde_json::from_value(json!({
        "yard": false,
        "machine": [
            {"program": "herdr", "for": "it starts every agent in a pane of its own", "found": true,
             "path": "/usr/bin/herdr", "server": {"state": "not_running", "output": "no server"}},
            {"program": "git", "for": "a depot is a git repository", "found": true, "path": "/usr/bin/git"},
            {"program": "claude", "for": "a harness a yard can be created on", "found": true,
             "path": "/usr/bin/claude", "harness": "claude", "harness_login": true,
             "provider": "Anthropic", "label": "Claude Code",
             "offers": [
                 {"model": "opus", "label": "Opus 5.5", "recommended": "yardmaster"},
                 {"model": "fable", "label": "Fable 5.1"},
                 {"model": "sonnet", "label": "Sonnet 5"}]},
            {"program": "codex", "for": "a harness a yard can be created on", "found": true,
             "path": "/usr/bin/codex", "harness": "codex",
             "provider": "OpenAI", "label": "Codex",
             "offers": [
                 {"model": "gpt-6-luna", "label": "GPT-6 Luna", "recommended": "yardmaster"},
                 {"model": "gpt-6-astra", "label": "GPT-6 Astra"}]}
        ],
        "faults": []
    }))
    .expect("the setup shot's doctor")
}

/// Doctor's answer for the `setup-other` shot: git and herdr installed, a
/// claude on PATH that answers its version flag as another program, in the
/// shape yardr gives it (the item itself, with what it printed and the way to
/// fetch the real one, and again among the harnesses that can be fetched), and
/// a codex that can be fetched. The first run stands at the step that lists
/// them, having fetched and run nothing, and reads the claude it fetched
/// before as standing (`FirstRun::shot_needs`).
pub(crate) fn other_doctor() -> crate::setup::Doctor {
    let claude = json!({
        "program": "claude", "for": "a harness a yard can be created on", "found": false,
        "get": "npm install -g @anthropic-ai/claude-code (https://claude.com/claude-code)",
        "harness": "claude", "harness_login": true, "label": "Claude Code", "provider": "Anthropic",
        "fetch": {"version": "2.1.296", "url": "https://downloads.claude.ai/claude/2.1.296",
                  "sha256": "00", "size": 1, "path": "/Users/jo/.local/bin/claude",
                  "run": ["install", "2.1.296"]}
    });
    let mut other = claude.clone();
    other["path"] = json!("/usr/local/bin/claude");
    other["optional"] = json!(true);
    other["version"] = json!({"state": "not_harness", "output": "claude-tools 0.4.2"});
    serde_json::from_value(json!({
        "yard": false,
        "machine": [
            {"program": "herdr", "for": "it starts every agent in a pane of its own", "found": true,
             "path": "/usr/bin/herdr", "server": {"state": "not_running", "output": "no server"}},
            {"program": "git", "for": "a depot is a git repository", "found": true, "path": "/usr/bin/git"},
            other,
            {"program": "a harness", "for": "the agent yardr starts: a yard is created on one",
             "found": false, "get": "install one of the agents herdr starts (claude, codex, ...)",
             "any_of": [
                 claude,
                 {"program": "codex", "for": "a harness a yard can be created on", "found": false,
                  "get": "npm install -g @openai/codex", "harness": "codex",
                  "harness_login": true, "label": "Codex", "provider": "OpenAI",
                  "fetch": {"version": "0.160.1", "url": "https://github.com/openai/codex/releases/codex.tar.gz",
                            "sha256": "00", "size": 1, "path": "/Users/jo/.local/bin/codex",
                            "member": "codex-aarch64-apple-darwin"}}]}
        ],
        "faults": ["a harness: not on PATH"]
    }))
    .expect("the setup-other shot's doctor")
}

/// Doctor's answer for the `setup-linux` and `setup-copy` shots, as yardr
/// gives it on a Linux box that has nothing but the system's own programs:
/// herdr missing, with the command to paste and, where yardr pins a file for
/// the machine (`pinned`), the file too; git missing, which has a page and
/// no file; and no harness, which on Linux has no file either. The first
/// run stands at the step that lists them, having fetched and run nothing
/// (`FirstRun::shot_needs`).
pub(crate) fn linux_doctor(pinned: bool) -> crate::setup::Doctor {
    let mut herdr = json!({
        "program": "herdr",
        "for": "it starts every agent in a pane of its own, the yardmaster first: without it nothing runs",
        "found": false,
        "get": "curl -fsSL https://herdr.dev/install.sh | sh (more: https://herdr.dev)",
        "command": "curl -fsSL https://herdr.dev/install.sh | sh"
    });
    if pinned {
        herdr["fetch"] = json!({
            "version": "0.9.4",
            "url": "https://github.com/herdrdev/herdr/releases/download/v0.9.4/herdr-linux-x86_64",
            "sha256": "00", "size": 1, "path": "/home/jo/.local/bin/herdr",
            "verified": {"from": "https://herdr.dev/docs/install/", "on": "2026-11-05"}});
    }
    serde_json::from_value(json!({
        "yard": false,
        "machine": [
            herdr,
            {"program": "git",
             "for": "a depot is a git repository: beads are worked on branches, in worktrees",
             "found": false,
             "get": "https://git-scm.com/downloads (on macOS: xcode-select --install)"},
            {"program": "a harness", "for": "the agent yardr starts: a yard is created on one",
             "found": false, "get": "install one of the agents herdr starts (claude, codex, ...)"}
        ],
        "faults": ["herdr: not on PATH"]
    }))
    .expect("the setup-linux shot's doctor")
}

/// Serves `state` from `yard`: the yard of the window's tests, with the
/// mails and the asks the state opens at.
fn populate(yard: &Yard, state: &str) {
    // As the window's tests have it (window::tests::a_yard_of_jo): jo is
    // the yard's operator, and the login name is no user of it, so the
    // window is for jo whatever it runs as.
    yard.answer("Head", r#"{"result": 20}"#);
    yard.answer(
        "Crew",
        r#"{"result": [{"name": "assembly", "enabled": true, "state": "stopped"},
                       {"name": "yardmaster", "enabled": true, "state": "running", "status": "idle"}]}"#,
    );
    yard.answer(
        "Peers",
        r#"{"result": [{"name": "north", "send": ["mail"], "receive": ["mail"],
            "said": {"roster": [{"name": "clerk", "kind": "crew", "role": "Keeps the books", "mail": true}], "at": "2026-10-08T09:30:00Z"}}]}"#,
    );
    yard.answer(
        "Users",
        r#"{"result": [{"name": "jo", "cursor": 5, "notification_cursor": 0}]}"#,
    );
    yard.answer(
        "GetUser",
        r#"{"result": {"name": "jo", "cursor": 5, "notification_cursor": 0}}"#,
    );
    // The login name is refused once, as the window's tests have it: a
    // client of the socket asks after it, and falls back to the
    // operator. At an address the token says who the window is for, and
    // that read is not refused: the first load opens the window instead
    // of the follower's retry.
    if !yard.at_listener() {
        yard.answer_once_status(
            "GetUser",
            200,
            r#"{"error": {"code": "not-found", "message": "user: not found"}}"#,
        );
    }
    yard.answer("Settings", r#"{"result": {"operator": "jo"}}"#);
    yard.answer("AckUserInbox", "{}");
    yard.answer(
        "UserNotifications",
        r#"{"result": {"notifications": [], "unread": 0, "last": 0}}"#,
    );
    yard.answer("AckUserNotifications", "{}");
    yard.answer("Mail", "{}");
    yard.answer(
        "MailPeer",
        r#"{"result": {"peer": "north", "kind": "mail", "at": "2026-10-08T09:30:00Z"}}"#,
    );
    // A shot takes no pick, but the cards stand open: what one would call.
    yard.answer("AnswerAsk", "{}");
    yard.answer("AddNote", "{}");
    yard.answer("DeferAsk", "{}");
    yard.answer("DeclineAsk", "{}");
    // The arrivals board every state but old-lines loads: three rows
    // in served order, one with an amber remark, one with none, and one
    // with a red remark and no expected arrival. The window counts it
    // in the arrivals row; the departures state draws the rows flat in
    // this order. Old-lines serves neither call, so the yard answers
    // no such method to both, as a yardr from before the Lines view.
    if state != "old-lines" {
        yard.answer(
            "Arrivals",
            &json!({"result": {"rows": [
            {"bead": "yd-zulu", "title": "Zulu's late turn",
             "platform": "draft", "line": "yardr", "expected": "5h–8h",
             "remark": "late", "tone": "amber"},
            {"bead": "yd-alpha", "title": "Alpha moves first",
             "platform": "review", "line": "yardr", "expected": "20m–35m",
             "remark": "", "tone": "none"},
            {"bead": "footplate-p512", "title": "The sign-in's password label",
             "platform": "new", "line": "footplate", "expected": "—",
             "remark": "held by jo", "tone": "red", "held_by": "jo"},
        ], "platforms": ["new", "review", "draft"]}})
            .to_string(),
        );
        // The flows view every state but old-lines loads: the three arrivals
        // travelling, yardr's two on its default flow's turn and quiet stages,
        // footplate's one on a fault stage, each stage with its terminal
        // flag and its routed group's seats. The window counts the totals
        // in the timetable's rows.
        yard.answer(
            "Flows",
            &json!({"result": {
                "travelling": 3,
                "depots": [
                    {"name": "yardr", "kind": "git", "travelling": 2, "flows": [
                        {"name": "default", "type": "", "source": "store", "stages": [
                            {"stage": "new", "group": "muse-builders", "human": false,
                             "lamp": "turn", "terminal": false, "limit": 3, "taken": 2,
                             "beads": [
                                {"bead": "yd-zulu", "title": "Zulu's late turn",
                                 "expected": "5h–8h", "remark": "late",
                                 "tone": "amber", "working": false}]},
                            {"stage": "review", "group": "mckean", "human": true,
                             "lamp": "none", "terminal": false, "limit": 1, "taken": 0,
                             "beads": [
                                {"bead": "yd-alpha", "title": "Alpha moves first",
                                 "expected": "20m–35m", "remark": "",
                                 "tone": "none", "working": false}]},
                        ]},
                    ], "elsewhere": []},
                    {"name": "footplate", "kind": "git", "travelling": 1, "flows": [
                        {"name": "default", "type": "", "source": "store", "stages": [
                            {"stage": "new", "group": "muse-builders", "human": false,
                             "lamp": "fault", "terminal": false, "limit": 2, "taken": 1,
                             "beads": [
                                {"bead": "footplate-p512",
                                 "title": "The sign-in's password label",
                                 "expected": "—", "remark": "held by jo",
                                 "tone": "red", "held_by": "jo",
                                 "working": false}]},
                        ]},
                    ], "elsewhere": []},
                ],
                "unassigned": [],
            }})
            .to_string(),
        );
        // The grants every state but old-lines loads: none, but for the
        // grants state, which holds a read, an update and an `--as` grant,
        // and a source's named read call and update call.
        // Old-lines serves none of the three calls, as a yardr from before
        // them all.
        if state != "grants" {
            yard.answer("SourceGrants", r#"{"result": []}"#);
        }
        // The teammates every state loads: none, but for the
        // teammates-list state, which holds a paired and a waiting
        // teammate.
        if state != "teammates-list" {
            yard.answer("Teammates", r#"{"result": []}"#);
        }
        // The pairing every state loads: none open, but for the pairing
        // state, which holds one for jo's iphone with its code, expiry
        // and QR string.
        if state != "pairing" {
            yard.answer(
                "DevicePairState",
                r#"{"error": {"code": "not-found", "message": "an open device pairing: not found"}}"#,
            );
        }
        // The line the Line window reads: yardr's flows named by
        // their source — a store flow, a pack's, a repository's and
        // a directory's — with an exec stage's cmd whole, a prompt
        // behind Read and a labelled yard-wide rule under Other rules.
        // The editor's shot and the hand-off's read the same line: the
        // builders' prompt is a file the yard wrote, named by its digest.
        // The pack's flow has three stages, so that its middle one has
        // every control of the Stages row live.
        if matches!(
            state,
            "line-window"
                | "line-edit"
                | "line-sent"
                | "line-picker"
                | "line-refused"
                | "line-pack"
                | "line-stage-edit"
                | "line-stage-review"
                | "line-stage-copy"
                | "line-stage-copying"
                | "lines-served"
        ) {
            // The stage files of the pack's flow and the repository's
            // stage: the yard's own copy, which the pack's `new` was
            // saved over once (its digest and the text it replaced stand
            // in the read) and whose empty `## Ask` the yard warns of, and
            // the repository's, which is copied into the yard before its
            // stages edit.
            let managed_new = "A builder implements the bead here, on branch `yard/<bead>` in the bead's own worktree.\n\nBefore the bead leaves as `done`:\n\n- the work is committed on the branch, and the depot's test command passes\n  on the last commit;\n- a note on the bead says what changed and what was verified.\n\nOutcomes: `done` → review · `question` → decide\n\n## Ask\n";
            let managed_prior = "A builder implements the bead here, on branch `yard/<bead>`.\n\nBefore the bead leaves as `done`:\n\n- the work is committed on the branch, and the depot's test command passes\n  on the last commit.\n\nOutcomes: `done` → review · `question` → decide\n";
            let repo_new = "A new bead waits here until someone has read it and shaped it.\n\nOutcomes: `done` → build\n";
            yard.answer_for(
                "Line",
                "depot",
                "yardr",
                &json!({"result": {"depot": "yardr", "kind": "git", "flows": [
                    {"name": "default", "type": "", "source": "store", "stages": [
                        {"stage": "new", "next": ["review"], "outcomes": [{"name": "done", "to": "review"}],
                         "route": {"id": "r-1", "scope": "this depot's own",
                           "filters": {"type": "", "labels": [], "priority": "", "originator": ""},
                           "chain": [{"name": "builders", "runner": "herdr", "kind": "claude", "model": "opus",
                             "prompt": {"path": "/home/jo/.yardr/roles/builders.c34328a2b18796ffa525e37ab5a91bb12a4c43108f583a5fce0e2a4a92f83be1.md",
                                        "source": "file",
                                        "found": "/home/jo/.yardr/roles/builders.c34328a2b18796ffa525e37ab5a91bb12a4c43108f583a5fce0e2a4a92f83be1.md",
                                        "text": "Build the thing.\nRun the depot's test command before handing on.\n",
                                        "digest": "c34328a2b18796ffa525e37ab5a91bb12a4c43108f583a5fce0e2a4a92f83be1",
                                        "carriable": true}}], "limit": 3},
                         "other_routes": [{"id": "r-2", "scope": "yard-wide",
                           "filters": {"type": "task", "labels": ["eyes"], "priority": "P0-P1", "originator": "any"},
                           "group": "reviewers"}],
                         "open": 1, "stage_file": {"path": ".yardr/flows/default/new.md", "text": "Start here.\n"}},
                        {"stage": "review", "next": ["merged"], "terminal": false,
                         "route": {"id": "r-3", "scope": "yard-wide",
                           "filters": {"type": "", "labels": [], "priority": "", "originator": "any"},
                           "chain": [{"name": "assembly", "when": "Weekly<80", "runner": "exec", "cmd": "scripts/merge-local --fast"}],
                           "limit": 1},
                         "other_routes": [], "open": 0},
                        {"stage": "merged", "next": [], "terminal": true, "other_routes": [], "open": 0}]},
                    {"name": "managed", "type": "", "source": "pack starter", "stages": [
                        {"stage": "new", "next": ["review"], "terminal": false,
                         "route": {"id": "r-4", "scope": "yard-wide",
                           "filters": {"type": "", "labels": [], "priority": "", "originator": ""},
                           "chain": [{"name": "people", "runner": "manual", "members": ["jo"]}], "limit": 1},
                         "other_routes": [], "open": 0,
                         "stage_file": {"path": "/home/jo/.yardr/flows/managed/new.md",
                           "text": managed_new, "digest": crate::fake::digest(managed_new),
                           "prior_text": managed_prior, "prior_digest": crate::fake::digest(managed_prior),
                           "warning": "/home/jo/.yardr/flows/managed/new.md: the ## Ask section is empty"}},
                        {"stage": "review", "next": ["merged"], "terminal": false,
                         "route": {"id": "r-5", "scope": "yard-wide",
                           "filters": {"type": "", "labels": [], "priority": "", "originator": ""},
                           "chain": [{"name": "people", "runner": "manual", "members": ["jo"]}], "limit": 1},
                         "other_routes": [], "open": 0},
                        {"stage": "merged", "next": [], "terminal": true, "other_routes": [], "open": 0}]},
                    {"name": "repo", "type": "", "source": "repo@abcdef0", "stages": [
                        {"stage": "new", "next": [], "terminal": false, "other_routes": [], "open": 0,
                         "stage_file": {"path": ".yardr/flows/default/new.md",
                           "text": repo_new, "digest": crate::fake::digest(repo_new)}}]},
                    {"name": "dirflow", "type": "", "source": "repo (directory)", "directory": "/home/jo/notes", "stages": [
                        {"stage": "inbox", "next": [], "terminal": false, "other_routes": [], "open": 0}]},
                ], "crew": [{"name": "yardmaster", "prompt": {"path": "/home/jo/.yardr/roles/yardmaster.md",
                              "source": "file", "found": "/home/jo/.yardr/roles/yardmaster.md",
                              "text": "Mind the yard.\n", "carriable": true}},
                            {"name": "signalman"}]}})
                .to_string(),
            );
        }
        // The drift shots read yardr's depot as one store flow whose
        // stage files the repository holds too: just as `yardr flow
        // copy-repo` leaves it, or after a stage's file was edited.
        if matches!(
            state,
            "line-stage-copied"
                | "line-stage-drift"
                | "line-stage-drift-flow"
                | "line-stage-drift-drawer"
        ) {
            yard.answer_for(
                "Line",
                "depot",
                "yardr",
                &line_drift(state != "line-stage-copied"),
            );
        }
        // The drawer's depot reads the same way: footplate's default
        // flow at new, with its stage file, its serving group and the
        // group's prompt, and review standing unserved.
        if state == "lines-context" {
            yard.answer_for(
                "Line",
                "depot",
                "footplate",
                &json!({"result": {"depot": "footplate", "kind": "git", "flows": [
                    {"name": "default", "type": "task", "source": "store", "stages": [
                        {"stage": "new", "next": ["review"], "outcomes": [{"name": "done", "to": "review"}],
                         "route": {"id": "r-1", "scope": "this depot's own",
                           "filters": {"type": "", "labels": [], "priority": "", "originator": ""},
                           "chain": [{"name": "muse-builders", "runner": "herdr", "kind": "muse", "model": "opus",
                             "prompt": {"path": "/home/jo/.yardr/roles/muse-builder.md",
                                        "source": "file",
                                        "found": "/home/jo/.yardr/roles/muse-builder.md",
                                        "text": "# Builder (muse)\n\nYou implement one bead, in its own worktree on branch `yard/<bead>`.\n\n1. Read the bead first (`yardr prime --bead <id>`): its goal, notes and edges.\n2. Run the depot's test command on the untouched base before you change anything.\n3. Write a plan note on the bead, then start.\n",
                                        "digest": "9d01c7a2e5f3b84610ad3c2f5e7b9a0c1d4e6f8a2b3c5d7e9f0a1b2c3d4e5f60",
                                        "carriable": true}}], "limit": 2},
                         "other_routes": [], "open": 1,
                         "stage_file": {"path": ".yardr/flows/default/new.md",
                           "text": "A builder implements the bead here, on branch `yard/<bead>` in the bead's own worktree.\n\nBefore the bead leaves as `done`:\n\n- the work is committed on the branch, and the depot's test command passes on the last commit;\n- a note on the bead says what changed and what was verified, and ends with the test line.\n\nOutcomes: `done` → review · `question` → decide · back to backlog\n"}},
                        {"stage": "review", "next": ["merged"], "terminal": false,
                         "other_routes": [], "open": 0},
                        {"stage": "merged", "next": [], "terminal": true, "other_routes": [], "open": 0}]},
                ], "crew": []}})
                .to_string(),
            );
        }
        // The crew drawer's member reads the first depot's line:
        // footplate's, with the yardmaster's role prompt whole and
        // carriable, and its prime, which the Prime tab lists: the role,
        // the operator's words, three memory titles and four more, and
        // the rules of footplate's checkout and none of notes'.
        if matches!(state, "crew-context" | "crew-prime") {
            yard.answer("Settings", r#"{"result": {"operator": "jo"}}"#);
            yard.answer("Prime", &crew_prime());
            yard.answer("Flows", &flows_frame());
            yard.answer_for(
                "Line",
                "depot",
                "footplate",
                &json!({"result": {"depot": "footplate", "kind": "git", "flows": [],
                "crew": [{"name": "yardmaster",
                  "prompt": {"path": ".yardr/roles/yardmaster.md", "source": "file",
                    "found": "/home/jo/.yardr/roles/yardmaster.md",
                    "text": crew_role_text(),
                    "digest": "c34328a2b18796ffa525e37ab5a91bb12a4c43108f583a5fce0e2a4a92f83be1",
                    "carriable": true}},
                 {"name": "signalman"}]}})
                .to_string(),
            );
        }
    }
    match state {
        // The yard at an address is opened from the settings, not from
        // this yard, and the settings open over a running one, each
        // state on its section: both serve the yardmaster's
        // conversation, as does the yard that is stopped once it is
        // read, the one that answers under the first screen's open
        // fields, the signed-out yard's, which the window never reaches
        // past the edge, the timetable states', which the window
        // opens on the panel, the board, the bead and the flow, the
        // switcher's, which the window opens with the switcher's list
        // standing, as it does for the one that shuts it by a click
        // outside and the member one, the notice's, whose flows view
        // does not answer, the
        // screen's, whose session's screen opens over it, the
        // hosted screen's, whose card opens over it and learns from the
        // yard's refusal that the screen is read on its machine, the
        // refused screen's, whose card opens over it with the
        // yard's long refusal in red, the wide screen's, whose card
        // opens over it on a pane wider than the window, and the voice
        // shots', the first of which taps the Voice button once the yard
        // is read.
        "yardmaster"
        | "far"
        | "settings"
        | "settings-appearance"
        | "settings-yards"
        | "settings-yards-rename"
        | "settings-agents"
        | "settings-notifications"
        | "settings-connection"
        | "settings-connection-edge"
        | "settings-voice"
        | "settings-voice-list"
        | "settings-voice-microphone"
        | "settings-telemetry"
        | "settings-telemetry-off"
        | "voice"
        | "voice-ready"
        | "voice-draft"
        | "reconnecting"
        | "answered"
        | "signedout"
        | "away"
        | "arrivals"
        | "departures"
        | "arrival"
        | "flow"
        | "lines-arrivals"
        | "lines-flow"
        | "lines-board"
        | "lines-context"
        | "lines-served"
        | "crew-context"
        | "crew-prime"
        | "line-window"
        | "line-edit"
        | "line-sent"
        | "line-picker"
        | "line-refused"
        | "line-pack"
        | "line-stage-edit"
        | "line-stage-review"
        | "line-stage-copy"
        | "line-stage-copying"
        | "line-stage-copied"
        | "line-stage-drift"
        | "line-stage-drift-flow"
        | "line-stage-drift-drawer"
        | "old-lines"
        | "switcher"
        | "switcher-shut"
        | "switcher-member"
        | "member-pair"
        | "about"
        | "about-missing"
        | "feedback"
        | "notice"
        | "screen"
        | "screen-hosted"
        | "screen-refused"
        | "screen-wide" => {
            // The conversation reads over two days, with a mail in each
            // sender's group: the yardmaster shot shows the column, the
            // grouping and the separators.
            let inbox = json!({"result": {"mail": [
                {"seq": 12, "from": "yardmaster", "to": "jo",
                 "text": "The evening's work is shaped: three beads, one question each.",
                 "read": false},
                {"seq": 14, "from": "yardmaster", "to": "jo",
                 "text": "They are green. Shall I land it?", "read": false},
                {"seq": 15, "from": "yardmaster", "to": "jo",
                 "text": "The second lands behind it.", "read": false},
            ], "unread": 3, "last": 15, "from": ["yardmaster"]}});
            yard.answer("UserInbox", &inbox.to_string());
            let conversation = json!({"result": [
                {"seq": 12, "at": "2026-10-13T09:30:00Z", "from": "yardmaster", "to": "jo",
                 "text": "The evening's work is shaped: three beads, one question each.",
                 "read": false},
                {"seq": 13, "at": "2026-10-13T09:32:00Z", "from": "jo", "to": "yardmaster",
                 "text": "Land the first when its tests are green.", "read": false},
                {"seq": 14, "at": "2026-10-13T09:35:00Z", "from": "yardmaster", "to": "jo",
                 "text": "They are green. Shall I land it?", "read": false},
                {"seq": 15, "at": "2026-10-13T09:37:00Z", "from": "yardmaster", "to": "jo",
                 "text": "The second lands behind it.", "read": false},
                {"seq": 16, "at": "2026-10-14T08:05:00Z", "from": "jo", "to": "yardmaster",
                 "text": "Both are landed.", "read": false},
                {"seq": 17, "at": "2026-10-14T08:09:00Z", "from": "jo", "to": "yardmaster",
                 "text": "What is next?", "read": false},
            ]});
            yard.answer("Conversation", &conversation.to_string());
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
            // One arrival's bead, which the arrival state opens: the
            // body, two labels, two notes, and three asks, two of them
            // open. The body holds the whole-text Markdown sample, so
            // the shot shows the table, quote, named link and rule in
            // a bead's body.
            if state == "arrival" {
                yard.answer("Bead", &bead_zulu("Two yards answer tonight:\n\n| Yard | Waiting | Working |\n| :--- | ------: | ------: |\n| home | none | 3 |\n| airy | 2 | none |\n\n> Hold the autumn timetable until the folded\n> carriers are printed.\n\nSee the [release page](https://yardr.dev/releases) before you start.\n\n---"));
            }
            // The call for the flows is taken and the connection closed
            // with no reply, as by a server that went away over it: the
            // yard reads on without them and says so in the notice that
            // stands under the mail.
            if state == "notice" {
                yard.answer_nothing("Flows");
            }
            // The captured pane the screen shot opens with.
            if state == "screen" {
                yard.answer(
                    "SessionScreen",
                    &json!({"result": {"screen": SCREEN_SHOT}}).to_string(),
                );
            }
            // The wide pane the screen-wide shot opens with.
            if state == "screen-wide" {
                yard.answer(
                    "SessionScreen",
                    &json!({"result": {"screen": SCREEN_WIDE_SHOT}}).to_string(),
                );
            }
            // The refusal of a hosted yard from before the screen came
            // down the link, which the card answers with where the
            // screen is read.
            if state == "screen-hosted" {
                yard.answer(
                    "SessionScreen",
                    &json!({"error": {"code": "error", "message": SCREEN_OVER_THE_LINK}})
                        .to_string(),
                );
            }
            // The yard's refusal of a screen asked in plain HTTP, which
            // the card shows in red.
            if state == "screen-refused" {
                yard.answer(
                    "SessionScreen",
                    &json!({"error": {"code": "error", "message": SCREEN_REFUSAL}}).to_string(),
                );
            }
            // The switch the telemetry shots open on: on with the
            // install id and the next payload, off with the consent
            // words and Turn on.
            if state == "settings-telemetry" {
                yard.answer(
                    "TelemetryShow",
                    &json!({"result": {
                        "install_id": "3f6d1c2e-9a4b-4e7a-b5c1-2d8f0a6e4b90",
                        "telemetry": "on",
                        "since": "2026-10-26",
                        "sending_day": "2026-10-28",
                        "source_days": ["2026-10-27"],
                        "hosted": false,
                        "request": {
                            "api_key": "phc_shot_fake_write_only_key",
                            "batch": [
                                {"event": "yardr_daily",
                                 "distinct_id": "3f6d1c2e-9a4b-4e7a-b5c1-2d8f0a6e4b90",
                                 "timestamp": "2026-10-28T00:00:00Z",
                                 "uuid": "0e3c8b2a-1432-4a51-9f39-1bc7a1b1d001",
                                 "properties": {"$process_person_profile": true,
                                    "$geoip_disable": true, "version": "v0.9.1",
                                    "os": "darwin", "telemetry_age_days": 2}},
                                {"event": "yardr_crew_day",
                                 "distinct_id": "3f6d1c2e-9a4b-4e7a-b5c1-2d8f0a6e4b90",
                                 "timestamp": "2026-10-28T00:00:00Z",
                                 "uuid": "451d5fc1-4a12-4c20-9d7b-1f9df88ca004",
                                 "properties": {"$process_person_profile": true,
                                    "$geoip_disable": true, "kind": "claude",
                                    "model": "default", "running_crew_sessions": 2,
                                    "hosted": false}},
                            ],
                        },
                    }})
                    .to_string(),
                );
            }
            if state == "settings-telemetry-off" {
                yard.answer(
                    "TelemetryShow",
                    r#"{"result": {"telemetry": "off", "sending_day": "2026-10-28", "hosted": false, "request": null, "no_batch": "off"}}"#,
                );
            }
        }
        "teammates-list" => {
            // The teammates view open on its row: jo's own operator row
            // with a paired laptop, hal waiting, and sam paired with a
            // tablet and a spare never used. No asks, so decisions opens
            // on the teammates.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer("Conversation", r#"{"result": []}"#);
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
            yard.answer(
                "Teammates",
                &json!({"result": [
                    {"name": "jo", "role": "operator", "email": "jo@example.net",
                     "invited_by": "jo", "invited_at": "2026-10-08T09:30:00Z",
                     "devices": [{"name": "laptop", "last_used": "2026-10-08T09:30:00Z"}]},
                    {"name": "hal", "role": "member", "email": "hal@example.net",
                     "invited_by": "jo", "invited_at": "2026-10-08T09:30:00Z",
                     "devices": []},
                    {"name": "sam", "role": "member", "email": "sam@example.net",
                     "invited_by": "jo", "invited_at": "2026-10-08T09:30:00Z",
                     "devices": [
                        {"name": "tablet", "last_used": "2026-10-08T09:30:00Z"},
                        {"name": "spare"}]},
                ]})
                .to_string(),
            );
        }
        "grants" => {
            // The grants view open on its row: a read grant and an update
            // grant to the yardmaster, the update as ada, a telegram
            // read whose history was not kept, and the yardmaster's grants
            // of two named calls of crm, which the yard serves the kinds
            // of: customer a read, note an update. No asks, so decisions
            // opens on the grants.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer("Conversation", r#"{"result": []}"#);
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
            yard.answer(
                "SourceGrants",
                &json!({"result": [
                    {"source": "google-mail", "crew": "yardmaster",
                     "calls": ["read"],
                     "at": "2026-10-17T19:02:00Z", "by": "mckean",
                     "given": {"read": {
                        "at": "2026-10-17T19:02:00Z", "by": "mckean",
                        "bead": "yd-c546", "ask": 409, "asker": "yardmaster"}}},
                    {"source": "google-mail", "crew": "yardmaster", "as": "ada",
                     "calls": ["send"],
                     "at": "2026-10-17T19:04:00Z", "by": "mckean",
                     "given": {"send": {
                        "at": "2026-10-17T19:04:00Z", "by": "mckean",
                        "bead": "yd-c546", "ask": 412, "asker": "yardmaster"}}},
                    {"source": "telegram", "group": "builders",
                     "calls": ["read"],
                     "at": "2026-10-12T10:15:00Z", "by": "mckean"},
                    {"source": "crm", "crew": "yardmaster",
                     "calls": ["customer", "note"],
                     "at": "2026-10-17T19:06:00Z", "by": "mckean",
                     "given": {
                        "customer": {
                            "at": "2026-10-17T19:06:00Z", "by": "mckean",
                            "bead": "yd-c546", "ask": 415, "asker": "yardmaster"},
                        "note": {
                            "at": "2026-10-17T19:06:00Z", "by": "mckean",
                            "bead": "yd-c546", "ask": 415, "asker": "yardmaster"}},
                     "kinds": {"customer": "read", "note": "update"}},
                ]})
                .to_string(),
            );
        }
        "pairing" => {
            // The Pair a phone pane open on its row: a pairing for jo's
            // iphone with its code, an expiry ten minutes out, and the QR
            // string a phone scans. No asks, so decisions opens on the
            // pairing.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer("Conversation", r#"{"result": []}"#);
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
            let ends = chrono::Utc::now() + Duration::seconds(630);
            yard.answer(
                "DevicePairState",
                &json!({"result": {"user": "jo", "device": "iphone", "state": "open",
                    "expires": ends.to_rfc3339(), "code": "K7QM-2XPD",
                    "qr": "https://edge.example.com#K7QM-2XPD#jo/demo"}})
                .to_string(),
            );
        }
        "pair-ask" => {
            // A pairing run ask's card: the yard's weight and sentences,
            // with the door its approval crosses beside the weight.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer("Conversation", r#"{"result": []}"#);
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [queued_pairing(false)], "decided": []}}).to_string(),
            );
            yard.answer("Bead", &bead_pairing(false));
        }
        "pair-qr" => {
            // A pairing approval's QR screen: the decided card stands
            // where the approval left it, with the transient posed as
            // the approval answered it, so the screen draws its QR, code
            // and minutes with End.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer("Conversation", r#"{"result": []}"#);
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [], "decided": [queued_pairing(true)]}}).to_string(),
            );
            yard.answer("Bead", &bead_pairing(true));
        }
        "pair-ended" => {
            // A pairing approval's ended state: the decided card with
            // the sitting posed ended, as End leaves it.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer("Conversation", r#"{"result": []}"#);
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [], "decided": [queued_pairing(true)]}}).to_string(),
            );
            yard.answer("Bead", &bead_pairing(true));
        }
        "delivery" => {
            // The yard row open on a hook delivery: ada's Telegram message
            // of a photo of the fence, which the yard holds, and a
            // document it did not take, over its limit. A bead's creation
            // stands before it, the one notice that is no delivery. No mail
            // and no asks, so the row shows the notices alone.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer("Conversation", r#"{"result": []}"#);
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
            yard.answer(
                "UserNotifications",
                &json!({"result": {"notifications": [
                    {"seq": 24, "at": "2026-10-13T09:30:00Z", "kind": "created",
                     "bead": "yd-fence", "text": "Paint the fence was opened", "read": true},
                    {"seq": 25, "at": "2026-10-13T09:41:00Z", "kind": "hook",
                     "text": "hook (all) telegram message by ada: the fence, painted",
                     "read": false},
                ], "unread": 1, "last": 25}})
                .to_string(),
            );
            yard.answer(
                "DeliveryFiles",
                &json!({"result": [
                    {"hash": IMAGE_HASH, "type": "image/png", "bytes": IMAGE.len()},
                    {"hash": "", "type": "", "bytes": 9437184, "name": "shed-plans.pdf",
                     "not_taken": "document of 9437184 bytes, over the yard's limit of 8388608 (yardr settings set blob_max_bytes <bytes>)"},
                ]})
                .to_string(),
            );
            yard.blob(IMAGE_HASH, IMAGE);
        }
        "crew" => {
            // Chat with the sidebar's groups full: the yardmaster
            // working, brakeman idle and assembly stopped, north's clerk
            // and warden with rue, jo and ada for people, and three
            // unread notices. The unread mail is elsewhere than the open
            // conversation, so Crew carries its badge, as Decide does for
            // the open ask and Lines for one of jo's beads added past
            // the head.
            yard.answer(
                "Crew",
                r#"{"result": [{"name": "assembly", "enabled": true, "state": "stopped"},
                               {"name": "yardmaster", "enabled": true, "state": "running", "status": "working"},
                               {"name": "brakeman", "enabled": true, "state": "running", "status": "idle"}]}"#,
            );
            yard.answer(
                "Peers",
                r#"{"result": [{"name": "north", "send": ["mail"], "receive": ["mail"],
                    "said": {"roster": [{"name": "clerk", "kind": "crew", "role": "Keeps the books", "mail": true},
                                        {"name": "warden", "kind": "crew", "role": "Watches the north gate", "mail": true},
                                        {"name": "rue", "kind": "user", "role": "Operates the north link", "mail": true}],
                             "at": "2026-10-08T09:30:00Z"}}]}"#,
            );
            yard.answer(
                "Users",
                r#"{"result": [{"name": "jo", "cursor": 5, "notification_cursor": 0},
                               {"name": "ada", "cursor": 9, "notification_cursor": 4}]}"#,
            );
            yard.answer(
                "UserNotifications",
                &json!({"result": {"notifications": [
                    {"seq": 10, "kind": "created", "bead": "yd-barn",
                     "text": "Paint the barn was opened", "read": false},
                    {"seq": 11, "kind": "closed", "bead": "yd-shed",
                     "text": "Shed the tools was closed", "read": false},
                    {"seq": 12, "kind": "advanced", "bead": "yd-barn",
                     "text": "Paint the barn moved to review", "read": false},
                ], "unread": 3, "last": 12}})
                .to_string(),
            );
            yard.answer(
                "UserInbox",
                &json!({"result": {"mail": [
                    {"seq": 12, "from": "yardmaster", "to": "jo",
                     "text": "The evening's work is shaped: three beads, one question each.",
                     "read": false},
                    {"seq": 13, "from": "brakeman", "to": "jo",
                     "text": "The second lands behind it.", "read": false},
                    {"seq": 14, "from": "brakeman", "to": "jo",
                     "text": "Green across the board.", "read": false},
                    {"seq": 15, "from": "clerk@north", "to": "jo",
                     "text": "The books balance.", "read": false},
                ], "unread": 4, "last": 15,
                "from": ["yardmaster", "brakeman", "clerk@north"]}})
                .to_string(),
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "The evening's work is shaped: three beads, one question each.", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [
                    {"bead": "bead-a", "bead_title": "Move the links", "topic": "topic:linear-out",
                     "ask": 3, "question": "Move them now?", "asker": "yardmaster",
                     "since": "2026-10-10T10:00:00Z"},
                ], "decided": []}})
                .to_string(),
            );
            yard.emit(r#"{"seq": 21, "kind": "created", "actor": "yardmaster", "bead": "yd-barn", "data": {"title": "Paint the barn", "stage": "new", "type": "task", "originator": "jo"}}"#);
        }
        "plain" | "plain-select" | "lines" | "keys" => {
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [
                    {"bead": "bead-a", "bead_title": "Move the links", "topic": "topic:linear-out",
                     "ask": 3, "question": "Move them now?", "asker": "yardmaster",
                     "since": "2026-10-10T10:00:00Z"},
                ], "decided": []}})
                .to_string(),
            );
            yard.answer(
                "Bead",
                &json!({"result": {
                    "id": "bead-a", "title": "Move the links",
                    "body": "The body of bead-a.",
                    "stage": "new", "labels": ["topic:linear-out"],
                    "notes": [{"seq": 2, "kind": "note", "author": "yardmaster",
                               "text": "the test passed", "at": "2026-10-10T09:30:00Z"}],
                    "asks": [{
                        "ask": 3, "to": "jo", "asker": "yardmaster",
                        "since": "2026-10-10T10:00:00Z",
                        "question": "Move them now?",
                        "context": ["The links file declares the new source.",
                                    "The move was tested."],
                        "options": [
                            {"label": "Move them now",
                             "consequence": "all 64 links point tonight"},
                            {"label": "After Friday",
                             "consequence": "keep the old source"},
                            {"label": "You decide",
                             "consequence": "leave it to yardmaster"}
                        ],
                        "recommend": 1, "why": "the test passed",
                        "state": "open"
                    }]
                }})
                .to_string(),
            );
            // The lines badge's bead: one of jo's added past the head,
            // so the rail's Lines carries its badge beside the others.
            if state == "lines" {
                yard.emit(r#"{"seq": 21, "kind": "created", "actor": "yardmaster", "bead": "yd-barn", "data": {"title": "Paint the barn", "stage": "new", "type": "task", "originator": "jo"}}"#);
            }
        }
        "page" => {
            // The plain ask above, whose note carries a page, an image and
            // a file: the yard holds the image, and the page and the file
            // are never asked for, since Open takes them to the browser,
            // run beside Open for the page alone. The mail carries a page
            // too, which Open takes to the browser, with no run.
            yard.blob(IMAGE_HASH, IMAGE);
            yard.answer(
                "UserInbox",
                &json!({"result": {"mail": [
                    {"seq": 12, "from": "yardmaster", "to": "jo",
                     "text": "The mockup is on the bead.", "read": false},
                ], "unread": 1, "last": 12, "from": ["yardmaster"]}})
                .to_string(),
            );
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo",
                     "text": "The mockup is on the bead.",
                     "files": [{"hash": MAIL_PAGE_HASH, "type": "text/html; charset=utf-8",
                                "bytes": 18000}],
                     "read": false},
                ]})
                .to_string(),
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [
                    {"bead": "bead-a", "bead_title": "Move the links", "topic": "topic:linear-out",
                     "ask": 3, "question": "Move them now?", "asker": "yardmaster",
                     "since": "2026-10-10T10:00:00Z"},
                ], "decided": []}})
                .to_string(),
            );
            yard.answer(
                "Bead",
                &json!({"result": {
                    "id": "bead-a", "title": "Move the links",
                    "body": "",
                    "stage": "new", "labels": ["topic:linear-out"],
                    "notes": [{"seq": 2, "kind": "note", "author": "yardmaster",
                               "text": "the mockup, like this:", "at": "2026-10-10T09:30:00Z",
                               "files": [
                                   {"hash": PAGE_HASH, "type": "text/html; charset=utf-8",
                                    "bytes": 42000},
                                   {"hash": IMAGE_HASH, "type": "image/png",
                                    "bytes": IMAGE.len()},
                                   {"hash": FILE_HASH, "type": "application/pdf",
                                    "bytes": 900, "name": "figures.pdf"},
                               ]}],
                    "asks": [{
                        "ask": 3, "to": "jo", "asker": "yardmaster",
                        "since": "2026-10-10T10:00:00Z",
                        "question": "Move them now?",
                        "context": ["The links file declares the new source."],
                        "options": [
                            {"label": "Move them now",
                             "consequence": "all 64 links point tonight"},
                            {"label": "You decide",
                             "consequence": "leave it to yardmaster"}
                        ],
                        "recommend": 0, "why": "",
                        "state": "open"
                    }]
                }})
                .to_string(),
            );
        }
        "page-unset" | "page-unset-many" => {
            // The plain ask above, on a yard whose web view is unset: the
            // note carries one page in the first state, so the row says
            // the sentence, and three pages and a file in the second, so
            // the sentence stands once under the rows. The pages and the
            // file are never asked for, since Open would take them to the
            // browser; with no view there is no Open. The mail carries
            // none.
            yard.answer(
                "UserInbox",
                &json!({"result": {"mail": [
                    {"seq": 12, "from": "yardmaster", "to": "jo",
                     "text": "The mockup is on the bead.", "read": false},
                ], "unread": 1, "last": 12, "from": ["yardmaster"]}})
                .to_string(),
            );
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo",
                     "text": "The mockup is on the bead.", "read": false},
                ]})
                .to_string(),
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [
                    {"bead": "bead-a", "bead_title": "Move the links", "topic": "topic:linear-out",
                     "ask": 3, "question": "Move them now?", "asker": "yardmaster",
                     "since": "2026-10-10T10:00:00Z"},
                ], "decided": []}})
                .to_string(),
            );
            let many = state == "page-unset-many";
            let files = match many {
                true => vec![
                    json!({"hash": PAGE_HASH, "type": "text/html; charset=utf-8",
                           "bytes": 42000}),
                    json!({"hash": PAGE_UNSET_HASH, "type": "text/html; charset=utf-8",
                           "bytes": 18000}),
                    json!({"hash": PAGE_UNSET_HASH_2, "type": "text/html; charset=utf-8",
                           "bytes": 9000}),
                    json!({"hash": FILE_HASH, "type": "application/pdf",
                           "bytes": 900, "name": "figures.pdf"}),
                ],
                false => vec![
                    json!({"hash": PAGE_HASH, "type": "text/html; charset=utf-8",
                           "bytes": 42000}),
                ],
            };
            let words = match many {
                true => "the mockups, like this:",
                false => "the mockup, like this:",
            };
            yard.answer(
                "Bead",
                &json!({"result": {
                    "id": "bead-a", "title": "Move the links",
                    "body": "",
                    "stage": "new", "labels": ["topic:linear-out"],
                    "notes": [{"seq": 2, "kind": "note", "author": "yardmaster",
                               "text": words, "at": "2026-10-10T09:30:00Z",
                               "files": files}],
                    "asks": [{
                        "ask": 3, "to": "jo", "asker": "yardmaster",
                        "since": "2026-10-10T10:00:00Z",
                        "question": "Move them now?",
                        "context": ["The links file declares the new source."],
                        "options": [
                            {"label": "Move them now",
                             "consequence": "all 64 links point tonight"},
                            {"label": "You decide",
                             "consequence": "leave it to yardmaster"}
                        ],
                        "recommend": 0, "why": "",
                        "state": "open"
                    }]
                }})
                .to_string(),
            );
        }
        "thread" => {
            // A long wrapped thread from alternating senders, the
            // newest mail holding the whole-text Markdown sample, above
            // one with an image: the window opens on the sample, so the
            // shot shows the reading column with wrapped mail and the
            // table, quote, named link and rule. The yard holds the
            // image, as in the mail state.
            yard.blob(IMAGE_HASH, IMAGE);
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 12, "at": "2026-10-13T09:30:00Z", "from": "yardmaster", "to": "jo",
                     "text": "New draft is in Gmail on Shai's thread: it says managers cannot change names today, people can edit their own, a manager option is coming with no date promised, and I'll let him know.",
                     "read": true},
                    {"seq": 13, "at": "2026-10-13T09:40:00Z", "from": "jo", "to": "yardmaster",
                     "text": "Can we get permission to edit?", "read": true},
                    {"seq": 14, "at": "2026-10-13T09:50:00Z", "from": "yardmaster", "to": "jo",
                     "text": "Not with the Gmail pack as it is: it has four calls, and by design no edit, delete, send or labels. To allow draft edits I'd write a second source file for Gmail, and you'd approve it as a new source plus a grant.",
                     "read": true},
                    {"seq": 15, "at": "2026-10-13T10:00:00Z", "from": "jo", "to": "yardmaster",
                     "text": "Write it, and send Shai the new draft once the grant lands. Tell him the manager option is coming.",
                     "read": true},
                    {"seq": 16, "at": "2026-10-13T10:10:00Z", "from": "yardmaster", "to": "jo",
                     "text": "The grant is in: the second source is merged to main, reviewed with screenshots, check:all passed. Shai's draft still says 'coming'; you can now tell him it's live once deployed.",
                     "read": true},
                    {"seq": 17, "at": "2026-10-13T10:20:00Z", "from": "jo", "to": "yardmaster",
                     "text": "Can I see the screenshot?", "read": true},
                    {"seq": 18, "at": "2026-10-13T10:30:00Z", "from": "yardmaster", "to": "jo",
                     "text": "The three screenshots from the builder's note: the editor with the override typed in and the reset button, the list and header after saving, and after reset.",
                     "files": [{"hash": IMAGE_HASH, "type": "image/png",
                                "bytes": IMAGE.len()}],
                     "read": true},
                    {"seq": 19, "at": "2026-10-13T10:40:00Z", "from": "yardmaster", "to": "jo",
                     "text": "Two yards answer tonight:\n\n| Yard | Waiting | Working |\n| :--- | ------: | ------: |\n| home | none | 3 |\n| airy | 2 | none |\n\n> Hold the autumn timetable until the folded\n> carriers are printed.\n\nSee the [release page](https://yardr.dev/releases) before you start.\n\n---",
                     "read": true},
                ]})
                .to_string(),
            );
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
        }
        "column" | "column-hover" | "column-copy" | "column-top" => {
            // A long conversation over three days, grouped by sender, with
            // a call link, a calendar link and a correction: the shot shows
            // the reading column, the grouping, the separators and the
            // short links. The days count back from today, so the
            // separators read Today, Yesterday and the weekday. The hover
            // state poses the pointer on the last mail, a code block of
            // one line that reads under the one above's head: its time
            // stands in the gutter, and its bar of reactions floats on its
            // top edge. The copy state poses it on the block too, so its
            // copy control shows, clear of the bar. The top state scrolls
            // the list to stand mail 15's head at its top and poses the
            // pointer on it: the bar has no room above the letter there,
            // and stops at the list's top, clear of the Screen control.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            let zone = chrono::Local::now().format("%:z").to_string();
            let day = |ago: i64, at: &str| {
                let date = chrono::Local::now().date_naive() - chrono::Duration::days(ago);
                format!("{date}T{at}{zone}")
            };
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 12, "at": day(2, "20:14:00"), "from": "yardmaster", "to": "jo",
                     "text": "The evening's work is shaped: three beads, one question each.",
                     "read": true},
                    {"seq": 13, "at": day(2, "20:16:00"), "from": "yardmaster", "to": "jo",
                     "text": "The first is ready to land.",
                     "read": true},
                    {"seq": 14, "at": day(2, "20:17:00"), "from": "jo", "to": "yardmaster",
                     "text": "here's the teams link for the video call: https://teams.microsoft.com/l/meetup-join/19%3ameeting_N2Q4YzQefBDwL/0?context=chat",
                     "read": true},
                    {"seq": 15, "at": day(2, "20:19:00"), "from": "yardmaster", "to": "jo",
                     "text": "I'll put it on the event as its location.",
                     "read": true},
                    {"seq": 16, "at": day(2, "20:21:00"), "from": "yardmaster", "to": "jo",
                     "text": "correction the edit card is ask 6893, not 6891",
                     "read": true},
                    {"seq": 17, "at": day(1, "09:30:00"), "from": "jo", "to": "yardmaster",
                     "text": "applied the docs pack",
                     "read": true},
                    {"seq": 18, "at": day(1, "09:33:00"), "from": "yardmaster", "to": "jo",
                     "text": "Thanks, google-docs now has replace. The invite moved: https://calendar.google.com/calendar/event?action=TEMPLATE&tmeid=abc123&dates=20261019",
                     "read": true},
                    {"seq": 19, "at": day(1, "09:35:00"), "from": "yardmaster", "to": "jo",
                     "text": "Once you approve the grant I read it and make the edits.",
                     "read": true},
                    {"seq": 20, "at": day(0, "08:05:00"), "from": "jo", "to": "yardmaster",
                     "text": "approved",
                     "read": true},
                    {"seq": 21, "at": day(0, "08:07:00"), "from": "yardmaster", "to": "jo",
                     "text": "Done: the Teams link is on Monday's call with Liana, as the location and in the description.",
                     "read": true},
                    {"seq": 22, "at": day(0, "08:09:00"), "from": "yardmaster", "to": "jo",
                     "text": "The second lands behind it.",
                     "read": true},
                    {"seq": 23, "at": day(0, "08:10:00"), "from": "yardmaster", "to": "jo",
                     "text": "```\nyardr review -g box-approvers\n```",
                     "read": true},
                ]})
                .to_string(),
            );
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
        }
        "older-unread" => {
            // The yardmaster's conversation of 120 mails, the last ninety
            // unread, and three of the assembly's unread after them. The
            // yard serves the conversation in pages, as the real one
            // does: the newest fifty, 71 to 120, are all unread, and the
            // oldest unread mail, 31, stands on the page before. The
            // window reads back to it as the conversation opens, so the
            // yardmaster's row shows no count while the reader stands at
            // the newest, and the assembly's, which is not open, shows its
            // three.
            let start = DateTime::parse_from_rfc3339("2026-10-13T09:30:00Z").unwrap();
            let mail = |seq: u64, from: &str, read: bool| {
                let at = start + Duration::minutes(3 * seq as i64);
                json!({"seq": seq, "at": at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                       "from": from, "to": "jo", "read": read,
                       "text": format!("No. {seq}: all is quiet.")})
            };
            let waiting: Vec<_> = (31..=120)
                .map(|seq| mail(seq, "yardmaster", false))
                .chain((121..=123).map(|seq| mail(seq, "assembly", false)))
                .collect();
            yard.answer(
                "UserInbox",
                &json!({"result": {"mail": waiting, "unread": waiting.len(), "last": 123,
                    "from": ["assembly", "yardmaster"]}})
                .to_string(),
            );
            yard.converse(
                "yardmaster",
                (1..=120)
                    .map(|seq| mail(seq, "yardmaster", seq < 31))
                    .collect(),
            );
            yard.converse(
                "assembly",
                (121..=123)
                    .map(|seq| mail(seq, "assembly", false))
                    .collect(),
            );
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
        }
        "drawer" => {
            // The yardmaster's conversation naming the bead the drawer
            // shows beside it: yd-zulu travels on the board every state
            // serves, and is read whole for the drawer, as a click on
            // its chip reads it.
            yard.answer(
                "UserInbox",
                &json!({"result": {"mail": [
                    {"seq": 12, "from": "yardmaster", "to": "jo",
                     "text": "The turn (yd-zulu) is still waiting.",
                     "read": false},
                ], "unread": 1, "last": 12, "from": ["yardmaster"]}})
                .to_string(),
            );
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 12, "at": "2026-10-14T08:00:00Z", "from": "yardmaster", "to": "jo",
                     "text": "The turn (yd-zulu) is still waiting.",
                     "read": false},
                    {"seq": 13, "at": "2026-10-14T08:02:00Z", "from": "jo", "to": "yardmaster",
                     "text": "Then it waits on the grant, as ever.",
                     "read": false},
                ]})
                .to_string(),
            );
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
            yard.answer(
                "Bead",
                &bead_zulu("The turn the yard waits on: its tests are green, and the landing waits for the review."),
            );
        }
        "refs" => {
            // The yardmaster's conversation naming an ask and two bead
            // ids: the ask's chip with its preview under the mail, and
            // the beads' chips. The ask is a run ask's, as the grant in
            // the review is; yardr-e9f0 is its own bead, and yd-zulu
            // travels on the board every state serves.
            yard.answer(
                "UserInbox",
                &json!({"result": {"mail": [
                    {"seq": 12, "from": "yardmaster", "to": "jo",
                     "text": "I put the grant to you as a card (ask 6926).",
                     "read": false},
                ], "unread": 1, "last": 12, "from": ["yardmaster"]}})
                .to_string(),
            );
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 12, "at": "2026-10-14T08:00:00Z", "from": "yardmaster", "to": "jo",
                     "text": "I put the grant to you as a card (ask 6926).",
                     "read": false},
                    {"seq": 13, "at": "2026-10-14T08:02:00Z", "from": "jo", "to": "yardmaster",
                     "text": "On it. Is the turn (yd-zulu) still waiting?",
                     "read": false},
                    {"seq": 14, "at": "2026-10-14T08:05:00Z", "from": "yardmaster", "to": "jo",
                     "text": "It waits on the grant. The grant's bead (yardr-e9f0) holds it.",
                     "read": false},
                ]})
                .to_string(),
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [
                    {"bead": "yardr-e9f0", "bead_title": "Docs grant",
                     "ask": 6926, "question": "Grant yardmaster \"replace\" on google-docs?",
                     "asker": "yardmaster", "since": "2026-10-14T07:55:00Z",
                     "run": run_of()},
                ], "decided": []}})
                .to_string(),
            );
        }
        "reactions" | "reactions-hover" | "reactions-zoom" => {
            // The yardmaster's conversation with the parties' reactions
            // under three mails: jo's on the question, both parties' on
            // the answer, and jo's on a last one, with a mail between
            // without any. The last reads under the one above's head. The
            // hover state poses the pointer on it, so its bar of reactions
            // to tap floats on its top edge, with jo's own marked, and no
            // mail moves; the zoom state poses it on one emoji of that
            // bar.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo",
                     "text": "Shall I land it?", "read": true,
                     "reactions": [{"from": "jo", "emoji": "👍"}]},
                    {"seq": 13, "at": "2026-10-08T09:36:00Z", "from": "jo", "to": "yardmaster",
                     "text": "Land it.", "read": true,
                     "reactions": [{"from": "jo", "emoji": "✅"},
                                   {"from": "yardmaster", "emoji": "👀"}]},
                    {"seq": 14, "at": "2026-10-08T09:42:00Z", "from": "yardmaster", "to": "jo",
                     "text": "Landed.", "read": true},
                    {"seq": 15, "at": "2026-10-08T09:43:00Z", "from": "yardmaster", "to": "jo",
                     "text": "On main. The gate passed, the branch is gone and its worktree with it, and no other bead waits on this one. The next release has no blockers: its notes are drafted and wait for you whenever you want them.",
                     "read": true,
                     "reactions": [{"from": "jo", "emoji": "🎉"}]},
                ]})
                .to_string(),
            );
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
        }
        "reply" => {
            // The yardmaster's conversation with two replies among its
            // mails: jo's answer to the question names it, 12, and the
            // yardmaster's to that names jo's, 14. The mail between the
            // question and its follow-up reads under one head, as ever;
            // the last answers none, and heads its own without a mark,
            // since the head above it says one.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo",
                     "text": "The gate passed. Shall I land it?", "read": true},
                    {"seq": 13, "at": "2026-10-08T09:31:00Z", "from": "yardmaster", "to": "jo",
                     "text": "Nothing else waits on it.", "read": true},
                    {"seq": 14, "at": "2026-10-08T09:36:00Z", "from": "jo", "to": "yardmaster",
                     "text": "Land it.", "read": true, "answered": 12},
                    {"seq": 15, "at": "2026-10-08T09:37:00Z", "from": "yardmaster", "to": "jo",
                     "text": "Landed. The branch is gone and its worktree with it.",
                     "read": true, "answered": 14},
                    {"seq": 16, "at": "2026-10-08T09:38:00Z", "from": "yardmaster", "to": "jo",
                     "text": "The next release has no blockers.", "read": true},
                ]})
                .to_string(),
            );
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
        }
        "mail" | "image" | "image-menu" | "mail-refused" | "mail-edge" => {
            // The yardmaster's mail carries a page and an image, as the
            // bead's note does in the page state: the yard holds the
            // image, and the page is never asked for, since Open takes
            // it to the browser, with no run. The image state serves
            // the same mail, and the window opens its image over it; the
            // image-menu state opens the menu of the image in the thread;
            // the mail-refused and mail-edge states serve it where the web
            // view is behind a login, and where it is the edge's address,
            // with the page alone: the row is what they show, and the
            // mockup's own drawn Open would stand beside it.
            let picture = !matches!(state, "mail-refused" | "mail-edge");
            let mut files = vec![
                json!({"hash": MAIL_PAGE_HASH, "type": "text/html; charset=utf-8",
                       "bytes": 18000}),
            ];
            if picture {
                yard.blob(IMAGE_HASH, IMAGE);
                files.push(json!({"hash": IMAGE_HASH, "type": "image/png",
                                  "bytes": IMAGE.len()}));
            }
            yard.answer(
                "UserInbox",
                &json!({"result": {"mail": [
                    {"seq": 12, "from": "yardmaster", "to": "jo",
                     "text": "The mockup is in the mail.", "read": false},
                ], "unread": 1, "last": 12, "from": ["yardmaster"]}})
                .to_string(),
            );
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo",
                     "text": "The mockup is in the mail.",
                     "files": files, "read": false},
                ]})
                .to_string(),
            );
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
        }
        "undelivered" => {
            // The conversation with one of a linked yard whose mail the
            // yard gave up over the hour: its line says what the link
            // takes and when the hour ends, from the mail the yard wrote
            // of it. The failed message and the telling cross as the
            // stream tells of them, past the head the window reads from.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [], "unread": 0, "last": 0, "from": []}}"#,
            );
            yard.converse(
                "clerk@north",
                vec![
                    json!({"seq": 12, "at": "2026-10-13T09:30:00Z", "from": "clerk@north",
                           "to": "jo", "text": "The ledger closes at noon.", "read": true}),
                    json!({"seq": 13, "at": "2026-10-13T09:32:00Z", "from": "jo",
                           "to": "clerk@north", "text": "Is the ledger closed?",
                           "read": false, "delivery": "failed"}),
                ],
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 11, "at": "2026-10-13T09:29:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
            yard.emit(r#"{"seq": 21, "kind": "peer_message_failed", "actor": "yard", "data": {"peer": "north", "kind": "mail", "id": "m1", "attempts": 1, "reason": "peer north: refused by the peer: it takes 30 mails an hour from this yard; send again after 21:00"}}"#);
            yard.emit(r#"{"seq": 22, "kind": "mail", "actor": "yard", "at": "2026-10-13T20:31:00Z", "data": {"to": "jo", "to_user": true, "undelivered": "m1", "limit": 30, "until": "2026-10-13T21:00:00Z", "text": "your mail to clerk@north of 10-13 20:31 was not delivered: refused by the peer: it takes 30 mails an hour from this yard; send again after 21:00"}}"#);
        }
        "setup-host" => {
            // An open card of a setup ask whose account stands, with a
            // host template: the host field above the account's, posed
            // with a host typed, and the yard's host line among the
            // Does lines.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [queued_setup_host()], "decided": []}}).to_string(),
            );
            yard.answer("Bead", &bead_setup_host());
        }
        "secret"
        | "secret-shown"
        | "secret-refused"
        | "secret-harness"
        | "secret-harness-empty" => {
            // An open card of a secret ask, posed with a value typed:
            // hidden, with the toggle to Show, or shown where the yard
            // marks the value no secret, with the toggle to Hide. The
            // refused shot poses the same ask with the yard's refusal
            // of its answer on the card instead, the field empty. The
            // harness shot poses the ask for a harness's API key instead
            // of a source's credential: the yard sends it no reach, and
            // its card says where the value goes in the harness's words.
            // The harness-empty shot poses the same ask with nothing
            // typed, so that the field says what it takes: the harness
            // the key is of, where the yard sends no source to name.
            let unconcealed = state == "secret-shown";
            let harness = matches!(state, "secret-harness" | "secret-harness-empty");
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [queued_secret(unconcealed, harness)], "decided": []}})
                    .to_string(),
            );
            yard.answer("Bead", &bead_secret(unconcealed, harness));
        }
        "decisions" | "decisions-filtered" | "decisions-more" => {
            // One open ask with a question that wraps, and a history of
            // 156 decided: three outcomes asked today, a folded look-alike
            // run of four with a put-aside, a decline and a secret given
            // yesterday, and the older weeks behind them. The history draws
            // its first page of fifty and a row for the rest. The filtered
            // state poses the filter with `docs`, which matches today
            // above the fold and yesterday below it; the more state
            // stands the list at its end, under the page's last rows,
            // where that row is.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {
                    "open": [queued_links()],
                    "decided": decided_history(),
                }})
                .to_string(),
            );
            yard.answer("Bead", &bead_links());
        }
        "waiting" | "waiting-folded" | "waiting-end" | "old-waiting" | "lines-waiting"
        | "lines-waiting-end" | "lines-help" | "lines-panel" | "lines-departures"
        | "lines-siding" => {
            // The Decide list: one open ask, and under it three beads
            // parked for jo's move, the longest waiting first: a bead with
            // a move forward and a move back that wants a note, one with
            // the one move a siding offers, and one a person approves
            // for production, which closes it. Twelve dealt with asks are
            // folded under them. The first parked bead is open on its card,
            // with the stage's route under its title. The folded state
            // folds the section to its head, its card still open. The Lines shots open
            // that bead's details instead, from the board: the same moves
            // under its body, and on a yard from before the call, which
            // serves no `Waiting`, the one line that says who moves its
            // stage. The panel and departures shots open the arrivals pane
            // over the same yard instead: its stats and its banner
            // count the open ask with the three parked beads after it. The
            // siding shot opens plantime's default flow, where the stages'
            // heads stand: the siding's says its one waiting bead against
            // its fold of four. The
            // old-waiting shot is the Decide list on a yard whose yardr
            // refuses the call by name, as one from before the call was a
            // stable one does: no section, the words that the yard is too
            // old in its place, and the open ask's card open.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            let history: Vec<serde_json::Value> = decided_history()
                .as_array()
                .unwrap()
                .iter()
                .take(12)
                .cloned()
                .collect();
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [queued_links()], "decided": history}}).to_string(),
            );
            yard.answer("Bead", &bead_links());
            let midnight = shot_midnight();
            // The yard from before the call answers no such method. One from
            // before it was a stable one refuses the call that names its
            // arguments, with yardr's own sentence.
            if state == "old-waiting" {
                yard.answer(
                    "Waiting",
                    r#"{"error": {"code": "error", "message": "arguments: Waiting takes them as a JSON array: only the calls docs/api.md names take them by name"}}"#,
                );
            } else if state != "lines-help" {
                yard.answer(
                    "Waiting",
                    &json!({"result": [
                    {"bead": "plantime-4f2a", "title": "Release the autumn timetable",
                     "depot": "plantime", "stage": "ready", "group": "people",
                     "since": shot_since(&midnight, 3, 9, 30), "revision": "01KJ8W3C5T0E9XRQ2NAM4PBD7F",
                     "actions": [
                        {"kind": "done", "label": "done: move to build",
                         "outcome": "done", "to": "build"},
                        {"kind": "done", "label": "changes: send back to shape (note required)",
                         "outcome": "changes", "to": "shape", "note_required": true}]},
                    {"bead": "plantime-9c1e", "title": "Fold the October small fixes",
                     "depot": "plantime", "stage": "siding", "group": "people",
                     "since": shot_since(&midnight, 1, 14, 10), "revision": "01KJ9",
                     "actions": [{"kind": "advance", "label": "Move to ready", "to": "ready"}]},
                    {"bead": "plantime-d3a7", "title": "Take the new fares live",
                     "depot": "plantime", "stage": "production-approval", "group": "people",
                     "since": shot_since(&midnight, 0, 8, 45), "revision": "01KJA",
                     "actions": [
                        {"kind": "done", "label": "approved: finish as live (closes the bead)",
                         "outcome": "approved", "to": "live", "terminal": true},
                        {"kind": "done", "label": "changes: send back to ready (note required)",
                         "outcome": "changes", "to": "ready", "note_required": true}]}
                ]})
                .to_string(),
            );
            }
            for (bead, title, stage, note) in [
                (
                    "plantime-4f2a",
                    "Release the autumn timetable",
                    "ready",
                    "assembly: folded 6 beads, tests pass",
                ),
                (
                    "plantime-9c1e",
                    "Fold the October small fixes",
                    "siding",
                    "assembly: 4 small beads wait for company",
                ),
                (
                    "plantime-d3a7",
                    "Take the new fares live",
                    "production-approval",
                    "assembly: the fares check passes on staging",
                ),
            ] {
                yard.answer_for(
                    "Bead",
                    "bead",
                    bead,
                    &json!({"result": {
                        "id": bead, "title": title,
                        "body": "The folded carriers are printed and checked; the branch is green.",
                        "stage": stage, "labels": [],
                        "notes": [{"seq": 4, "kind": "note", "author": "assembly",
                                   "text": note.trim_start_matches("assembly: "),
                                   "at": shot_since(&midnight, 3, 9, 0)}],
                        "asks": []
                    }})
                    .to_string(),
                );
            }
            yard.answer(
                "Flows",
                &json!({"result": {"travelling": 3, "depots": [
                    {"name": "plantime", "kind": "git", "travelling": 3, "flows": [
                        {"name": "default", "type": "", "source": "store", "stages": [
                            {"stage": "shape", "group": "people", "human": true,
                             "lamp": "none", "terminal": false, "beads": []},
                            {"stage": "build", "group": "builders", "human": false,
                             "lamp": "none", "terminal": false, "limit": 2, "taken": 0,
                             "beads": []},
                            {"stage": "ready", "group": "people", "human": true,
                             "lamp": "turn", "terminal": false,
                             "beads": [{"bead": "plantime-4f2a",
                                        "title": "Release the autumn timetable",
                                        "expected": "—", "remark": "waits for jo",
                                        "tone": "amber", "working": false}]},
                            {"stage": "siding", "group": "people", "human": true,
                             "lamp": "turn", "terminal": false, "fold": 4,
                             "beads": [{"bead": "plantime-9c1e",
                                        "title": "Fold the October small fixes",
                                        "expected": "—", "remark": "waits for jo",
                                        "tone": "amber", "working": false}]},
                            {"stage": "production-approval", "group": "people", "human": true,
                             "lamp": "turn", "terminal": false,
                             "beads": [{"bead": "plantime-d3a7",
                                        "title": "Take the new fares live",
                                        "expected": "—", "remark": "waits for jo",
                                        "tone": "amber", "working": false}]},
                            {"stage": "live", "group": "shippers", "human": false,
                             "lamp": "none", "terminal": true, "beads": []}
                        ]}
                    ], "elsewhere": []}
                ], "unassigned": []}})
                .to_string(),
            );
        }
        "waiting-many" | "waiting-many-open" => {
            // The Decide list with one bead parked for jo's move, served
            // ten plain moves as a flow-less bead is: its card stands
            // folded behind the one control, and unfolded in the open
            // state, stood at its end where the control or the moves and
            // the line for their note are.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer("UserAsks", r#"{"result": {"open": [], "decided": []}}"#);
            let midnight = shot_midnight();
            let moves: Vec<serde_json::Value> = [
                "new", "draft", "build", "review", "decide", "fix", "verify", "publish", "sign",
                "merged",
            ]
            .iter()
            .map(|to| json!({"kind": "advance", "label": format!("Move to {to}"), "to": to}))
            .collect();
            yard.answer(
                "Waiting",
                &json!({"result": [
                    {"bead": "lead-5gvp", "title": "A lead with no flow",
                     "stage": "backlog", "group": "people",
                     "since": shot_since(&midnight, 3, 9, 30), "revision": "01KJ8W3C5T0E9XRQ2NAM4PBD7F",
                     "actions": moves}
                ]})
                .to_string(),
            );
            yard.answer(
                "Bead",
                &json!({"result": {
                    "id": "lead-5gvp", "title": "A lead with no flow",
                    "body": "A lead from the site's invite field, with no depot and no flow.",
                    "stage": "backlog", "labels": [],
                    "notes": [{"seq": 4, "kind": "note", "author": "yardmaster",
                               "text": "asked for the team's size",
                               "at": shot_since(&midnight, 3, 9, 0)}],
                    "asks": []
                }})
                .to_string(),
            );
        }
        "weighty" => {
            // A weighty run ask's card beside a routine one's: the
            // weighty card shows its classes' lines first, and where
            // the window has no gate the typed-back number above the
            // control, against the routine card's approval.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [
                    {"bead": "bead-s", "bead_title": "The link with airy", "topic": "",
                     "ask": 7, "question": "Take more mail from airy?",
                     "asker": "yardmaster", "since": "2026-10-10T10:00:00Z",
                     "run": weighed_run_of(),
                     "weight": [{"class": "trust",
                                 "line": "widens who the yard listens to: the link with airy takes 300 mails and asks an hour from it"}]},
                    {"bead": "bead-s", "bead_title": "The link with airy", "topic": "",
                     "ask": 8, "question": "List the yard's peers?",
                     "asker": "yardmaster", "since": "2026-10-10T10:05:00Z",
                     "run": {"command": "yardr peer list", "argv": ["peer", "list"],
                             "does": ["lists the yard's peers, and what each takes and sends"],
                             "back": ""}},
                ], "decided": []}})
                .to_string(),
            );
            yard.answer(
                "Bead",
                &json!({"result": {"id": "bead-s", "title": "The link with airy",
                    "body": "The body.", "stage": "new", "labels": [], "notes": [],
                    "asks": [
                        {"ask": 7, "to": "jo", "asker": "yardmaster",
                         "since": "2026-10-10T10:00:00Z",
                         "question": "Take more mail from airy?",
                         "context": ["airy's yard was refused 4 mails in the last hour."],
                         "options": [], "state": "open", "run": weighed_run_of(),
                         "weight": [{"class": "trust",
                                     "line": "widens who the yard listens to: the link with airy takes 300 mails and asks an hour from it"}]},
                        {"ask": 8, "to": "jo", "asker": "yardmaster",
                         "since": "2026-10-10T10:05:00Z",
                         "question": "List the yard's peers?",
                         "context": ["the board wants the count."],
                         "options": [], "state": "open",
                         "run": {"command": "yardr peer list", "argv": ["peer", "list"],
                                 "does": ["lists the yard's peers, and what each takes and sends"],
                                 "back": ""}},
                    ]}})
                .to_string(),
            );
        }
        "weighty-mapping" => {
            // A weighty mapping ask's card beside a routine one's: the
            // mapping card shows its class's line first, and where the
            // window has no gate the typed-back number above its pick,
            // against the routine card's answer.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [
                    {"bead": "bead-s", "bead_title": "Who is U999?", "topic": "",
                     "ask": 7, "question": "Who is U999 in talk?",
                     "asker": "yardmaster", "since": "2026-10-10T10:00:00Z",
                     "identity": {"source": "talk", "sender": "U999"},
                     "weight": [{"class": "as-user",
                                 "line": "a pick says which user U999 in talk is, and a delivery from them is then heard as that user's"}]},
                    {"bead": "bead-s", "bead_title": "Who is U999?", "topic": "",
                     "ask": 8, "question": "Move the links now?",
                     "asker": "yardmaster", "since": "2026-10-10T10:05:00Z"},
                ], "decided": []}})
                .to_string(),
            );
            yard.answer(
                "Bead",
                &json!({"result": {"id": "bead-s", "title": "Who is U999?",
                    "body": "The body.", "stage": "new", "labels": [], "notes": [],
                    "asks": [
                        {"ask": 7, "to": "jo", "asker": "yardmaster",
                         "since": "2026-10-10T10:00:00Z",
                         "question": "Who is U999 in talk?",
                         "context": ["a delivery arrived from U999, who maps to nobody."],
                         "options": [
                            {"label": "ada", "consequence": "messages from U999 count as ada's"},
                            {"label": "Someone new", "consequence": "pick this and write the new user's name with your answer"},
                            {"label": "Nobody — ignore this sender", "consequence": "ignores U999 in talk: their messages are dropped, and no further card is offered"},
                            {"label": "You decide", "consequence": "leave it to yardmaster"}],
                         "state": "open",
                         "identity": {"source": "talk", "sender": "U999"},
                         "weight": [{"class": "as-user",
                                     "line": "a pick says which user U999 in talk is, and a delivery from them is then heard as that user's"}]},
                        {"ask": 8, "to": "jo", "asker": "yardmaster",
                         "since": "2026-10-10T10:05:00Z",
                         "question": "Move the links now?",
                         "context": ["the links file declares the new source."],
                         "options": [
                            {"label": "Move them now", "consequence": "all 64 links point tonight"},
                            {"label": "You decide", "consequence": "leave it to yardmaster"}],
                         "recommend": 1, "why": "the test passed",
                         "state": "open"},
                    ]}})
                .to_string(),
            );
        }
        "held-message" | "delivered-message" | "hook-message" => {
            // A mapping card's stranger's mail, read by the operator: held on
            // the open card before a pick, released by "Let in, once" and read
            // whole under the dealt with card, and read whole under its hook
            // notice in the yard row. The yard serves the one mail three ways.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Someone new wrote to the yard.", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Someone new wrote to the yard.", "read": false}]}"#,
            );
            yard.answer("Waiting", r#"{"result": []}"#);
            let mail = held_mail();
            let identity =
                |held: u64| json!({"source": "email", "sender": "ada@example.net", "held": held});
            let weight = json!([{"class": "as-user",
                "line": "a pick says which user ada@example.net in email is, and a delivery from them is then heard as that user's"}]);
            let asked = json!({
                "bead": "bead-s", "bead_title": "Who is ada@example.net?", "topic": "",
                "ask": 7, "question": "Who is ada@example.net in email?",
                "asker": "yardmaster", "since": "2026-10-10T10:00:00Z",
                "identity": identity(1), "weight": weight});
            let mut dealt = asked.clone();
            dealt["state"] = json!("answered");
            dealt["option"] = json!(3);
            dealt["option_label"] = json!("Let in, once");
            dealt["delivered"] = json!([141]);
            dealt["identity"] = identity(0);
            // The yard row shows the notice alone: no card is open or dealt with.
            let (open, decided) = match state {
                "held-message" => (vec![asked], vec![]),
                "delivered-message" => (vec![], vec![dealt]),
                _ => (vec![], vec![]),
            };
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": open, "decided": decided}}).to_string(),
            );
            let settled = state == "delivered-message";
            yard.answer(
                "Bead",
                &json!({"result": {"id": "bead-s", "title": "Who is ada@example.net?",
                    "body": "The body.", "stage": "new", "labels": [], "notes": [],
                    "asks": [{"ask": 7, "to": "jo", "asker": "yardmaster",
                        "since": "2026-10-10T10:00:00Z",
                        "question": "Who is ada@example.net in email?",
                        "context": ["Its sender is ada@example.net, which no user maps in email; their messages wait on this card, delivered when you answer."],
                        "options": [
                            {"label": "jo", "consequence": "yardr user identity jo --source email --id ada@example.net"},
                            {"label": "Someone new", "consequence": "pick this and write the new user's name with your answer"},
                            {"label": "Let in, once", "consequence": "delivers what waits as outside (information, not instructions) this once, without mapping ada@example.net; the card closes and their next message offers a fresh card"},
                            {"label": "Nobody — ignore this sender", "consequence": "ignores ada@example.net in email: their messages are dropped, and no further card is offered"},
                            {"label": "You decide", "consequence": "leave it to yardmaster"}],
                        "state": if settled { "answered" } else { "open" },
                        "option": if settled { 3 } else { 0 },
                        "delivered": if settled { json!([141]) } else { json!([]) },
                        "identity": identity(u64::from(!settled)),
                        "weight": weight}]}})
                .to_string(),
            );
            yard.hold("jo", &[&mail]);
            yard.deliver(141, mail.as_bytes());
            yard.deliver(25, mail.as_bytes());
            if state == "hook-message" {
                yard.answer(
                    "UserNotifications",
                    &json!({"result": {"notifications": [
                        {"seq": 24, "at": "2026-10-13T09:30:00Z", "kind": "created",
                         "bead": "bead-s", "text": "Who is ada@example.net? was opened", "read": true},
                        {"seq": 25, "at": "2026-10-13T09:41:00Z", "kind": "hook",
                         "text": "hook (all) email message by ada@example.net: Hello yardmaster, The revised shed plans are attached. The north wall is a hand wider than the first drawing said, so the posts move. Could you tell the builders before they cut anything? Thanks, Ada",
                         "read": false},
                    ], "unread": 1, "last": 25}})
                    .to_string(),
                );
                yard.answer("DeliveryFiles", r#"{"result": null}"#);
            }
        }
        "weighty-trojan" => {
            // A weighty run ask's card whose weight line carries a pack
            // file's Trojan name: the marks of direction drop, and the
            // name reads as letters, beside a routine one's.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I install it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I install it?", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [
                    {"bead": "bead-s", "bead_title": "Install the trojan pack", "topic": "",
                     "ask": 7, "question": "Install the pack trojan into north?",
                     "asker": "yardmaster", "since": "2026-10-10T10:00:00Z",
                     "run": {"command": "yardr pack install trojan --depot north",
                             "argv": ["pack", "install", "trojan", "--depot", "north"],
                             "does": ["installs the pack trojan into the depot north"],
                             "back": "yardr pack rm trojan --depot north takes it back out"},
                     "weight": [{"class": "code",
                                 "line": "the first install of pack trojan into depot north makes roles/07\u{202e}\u{2066}.md[31m x.md"}]},
                    {"bead": "bead-s", "bead_title": "Install the trojan pack", "topic": "",
                     "ask": 8, "question": "List the yard's peers?",
                     "asker": "yardmaster", "since": "2026-10-10T10:05:00Z",
                     "run": {"command": "yardr peer list", "argv": ["peer", "list"],
                             "does": ["lists the yard's peers, and what each takes and sends"],
                             "back": ""}},
                ], "decided": []}})
                .to_string(),
            );
            yard.answer(
                "Bead",
                &json!({"result": {"id": "bead-s", "title": "Install the trojan pack",
                    "body": "The body.", "stage": "new", "labels": [], "notes": [],
                    "asks": [
                        {"ask": 7, "to": "jo", "asker": "yardmaster",
                         "since": "2026-10-10T10:00:00Z",
                         "question": "Install the pack trojan into north?",
                         "context": ["the pack brings a crew of its own."],
                         "options": [], "state": "open",
                         "run": {"command": "yardr pack install trojan --depot north",
                                 "argv": ["pack", "install", "trojan", "--depot", "north"],
                                 "does": ["installs the pack trojan into the depot north"],
                                 "back": "yardr pack rm trojan --depot north takes it back out"},
                         "weight": [{"class": "code",
                                     "line": "the first install of pack trojan into depot north makes roles/07\u{202e}\u{2066}.md[31m x.md"}]},
                        {"ask": 8, "to": "jo", "asker": "yardmaster",
                         "since": "2026-10-10T10:05:00Z",
                         "question": "List the yard's peers?",
                         "context": ["the board wants the count."],
                         "options": [], "state": "open",
                         "run": {"command": "yardr peer list", "argv": ["peer", "list"],
                                 "does": ["lists the yard's peers, and what each takes and sends"],
                                 "back": ""}},
                    ]}})
                .to_string(),
            );
        }
        "frame" | "decide" => {
            // The frame at 960x640: the open card's bead with two asks,
            // a run ask with a short command and its moves row and a
            // pick, and the bead's thread with its image under the
            // cards. Under it in the list, two more beads with three asks,
            // one bead parked for a move, twelve dealt with, and a crew of
            // four, two at work. The yard
            // holds the image, as in the mail state. The decide state
            // serves the open bead's flow besides, for its head's stage
            // dots; the frame state serves none, and its head names the
            // stage alone.
            yard.blob(IMAGE_HASH, IMAGE);
            yard.answer(
                "Crew",
                r#"{"result": [{"name": "yardmaster", "enabled": true, "state": "running", "status": "idle"},
                               {"name": "assembly", "enabled": true, "state": "running", "status": "working"},
                               {"name": "shaper", "enabled": true, "state": "running", "status": "working"},
                               {"name": "brakeman", "enabled": true, "state": "stopped"}]}"#,
            );
            yard.answer(
                "UserInbox",
                &json!({"result": {"mail": [
                    {"seq": 14, "from": "yardmaster", "to": "jo", "text": "Two asks on footplate-q2rd, the landing first.", "read": false},
                    {"seq": 15, "from": "shaper", "to": "jo", "text": "Quiet hours: two questions in Decide.", "read": false},
                    {"seq": 16, "from": "assembly", "to": "jo", "text": "The fares cache works, at a cost.", "read": false}],
                    "unread": 3, "last": 16, "from": ["yardmaster", "shaper", "assembly"]}})
                .to_string(),
            );
            yard.answer(
                "Conversation",
                &json!({"result": [
                    {"seq": 11, "at": landed_at(2, 30), "from": "jo", "to": "yardmaster", "text": "How is the dock badge coming?", "read": true},
                    {"seq": 12, "at": landed_at(2, 25), "from": "yardmaster", "to": "jo", "text": "Built and in review. ada is on it.", "read": true},
                    {"seq": 13, "at": landed_at(0, 40), "from": "yardmaster", "to": "jo", "text": "Review passed, with a picture of the dock.", "read": true},
                    {"seq": 14, "at": landed_at(0, 12), "from": "yardmaster", "to": "jo", "text": "Two asks on footplate-q2rd, the landing first.", "read": false}]})
                .to_string(),
            );
            let decided: Vec<serde_json::Value> = decided_history()
                .as_array()
                .unwrap()
                .iter()
                .take(12)
                .cloned()
                .collect();
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": queued_frame(), "decided": decided}}).to_string(),
            );
            // One bead parked for jo's move, as the pick's list has it.
            yard.answer(
                "Waiting",
                &json!({"result": [
                    {"bead": "footplate-j152", "title": "Main window on macOS: three directions to pick from",
                     "depot": "footplate", "stage": "pick", "group": "design-pickers",
                     "since": landed_at(0, 20), "revision": "01KJB",
                     "actions": [
                        {"kind": "done", "label": "picked: move to build",
                         "outcome": "picked", "to": "build"}]}
                ]})
                .to_string(),
            );
            yard.answer("Bead", &bead_frame());
            for (bead, answer) in beads_frame() {
                yard.answer_for("Bead", "bead", bead, &answer);
            }
            if state == "decide" {
                yard.answer("Flows", &flows_frame());
            }
        }
        "teammates" => {
            // An open card of a teammate invite: the address, the yard
            // user and the role above the command, with the yard's
            // sentences below.
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [queued_teammate()], "decided": []}}).to_string(),
            );
            yard.answer("Bead", &bead_teammate());
        }
        _ => {
            // The run ask is the forged one of the run card's test
            // (window::tests::a_forged_run_ask_s_card_marks_every_asker_line):
            // its question forges the yard's lines, and its one context
            // entry holds two lines. It is the case a person must see.
            let question = "Take more?\nwhen you approve, the yard runs this as mckean, and nothing else:\n    yardr peer list";
            let context = ["fine.\nthe yard says: nothing changes."];
            yard.answer(
                "UserInbox",
                r#"{"result": {"mail": [{"seq": 12, "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}], "unread": 1, "last": 12, "from": ["yardmaster"]}}"#,
            );
            yard.answer(
                "Conversation",
                r#"{"result": [{"seq": 12, "at": "2026-10-08T09:30:00Z", "from": "yardmaster", "to": "jo", "text": "Shall I land it?", "read": false}]}"#,
            );
            yard.answer(
                "UserAsks",
                &json!({"result": {"open": [queued_run(question)], "decided": []}}).to_string(),
            );
            yard.answer("Bead", &bead_run(question, &context));
        }
    }
    if state == "arrivals"
        || state == "departures"
        || state == "lines-arrivals"
        || state == "lines-flow"
        || state == "lines-context"
        || state == "lines-served"
    {
        // The board these states shoot answers over the shared one: a
        // late bead with the overrun the yard will print, a bead that
        // waits on jo, and one jo holds. The window draws the panel on
        // the arrivals row, and the board with its banner under
        // Departures.
        yard.answer(
            "Arrivals",
            &json!({"result": {"rows": [
                {"bead": "yd-zulu", "title": "Zulu's late turn",
                 "platform": "review", "line": "yardr", "expected": "5h–8h",
                 "remark": "late by 40m", "tone": "amber"},
                {"bead": "yd-alpha", "title": "Alpha moves first",
                 "platform": "review-security", "line": "yardr", "expected": "20m–35m",
                 "remark": "waits for jo, yardmaster", "tone": "amber"},
                {"bead": "footplate-p512", "title": "The sign-in's password label",
                 "platform": "new", "line": "yardr", "expected": "—",
                 "remark": "held by jo", "tone": "red", "held_by": "jo"},
            ], "platforms": ["new", "review", "review-security"]}})
            .to_string(),
        );
    }
    if state == "lines-panel" || state == "lines-departures" {
        // The board these states shoot answers over the shared one: the
        // three beads parked for jo's move, remarked as the yard prints
        // them. The window draws the panel on the arrivals row, and the
        // board with its banner under Departures.
        yard.answer(
            "Arrivals",
            &json!({"result": {"rows": [
                {"bead": "plantime-4f2a", "title": "Release the autumn timetable",
                 "platform": "ready", "line": "plantime", "expected": "—",
                 "remark": "waits for jo", "tone": "amber"},
                {"bead": "plantime-9c1e", "title": "Fold the October small fixes",
                 "platform": "siding", "line": "plantime", "expected": "—",
                 "remark": "waits for jo", "tone": "amber"},
                {"bead": "plantime-d3a7", "title": "Take the new fares live",
                 "platform": "production-approval", "line": "plantime", "expected": "—",
                 "remark": "waits for jo", "tone": "amber"},
            ], "platforms": ["ready", "siding", "production-approval"]}})
            .to_string(),
        );
    }
    if state == "arrivals"
        || state == "departures"
        || state == "flow"
        || state == "lines-arrivals"
        || state == "lines-flow"
        || state == "lines-context"
        || state == "lines-served"
    {
        // The flows these states shoot answer over the shared ones:
        // five tracks with a backlog to count, a bead jo holds, one
        // yardmaster holds and a blocked bead under a fault lamp, a
        // late and a worked bead with one more in review's full group,
        // a wait and a person's turn under a turn lamp, footplate's
        // `new` with two beads worked and two waiting for a slot, as
        // yardr.dev's line stood at 16:44 on 2 Nov (its station draws a
        // worked chip and counts the rest), a terminal
        // stage with its end square and two landings under it, an
        // Elsewhere bead, two beads with no flow, and drafts quiet.
        // Each stage carries its terminal flag and its routed group's
        // seats, each claimed bead its worker and its stage's clock.
        // The longest names this yard's own flows have stand among them:
        // `contributor` and `approved` as stages of people, and the flow
        // `contributors.issue · issue`, so a station too narrow for a real
        // name shows in the picture. The lines shots add a quiet flow,
        // standing as one line of its path.
        let served = with_two_at_work(with_the_longest_names(json!({"result": {
            "travelling": 15,
            "depots": [
                {"name": "yardr", "kind": "git", "travelling": 10, "flows": [
                    {"name": "default", "type": "", "source": "store", "stages": [
                        {"stage": "backlog", "group": "mckean", "human": true,
                         "lamp": "none", "terminal": false, "limit": 1, "taken": 0,
                         "beads": [
                            {"bead": "yd-b1", "title": "First to shape",
                             "expected": "—", "remark": "", "tone": "none",
                             "working": false},
                            {"bead": "yd-b2", "title": "Second to shape",
                             "expected": "—", "remark": "", "tone": "none",
                             "working": false}]},
                        {"stage": "new", "group": "muse-builders", "human": false,
                         "lamp": "fault", "terminal": false, "limit": 3, "taken": 2,
                         "beads": [
                            {"bead": "footplate-p512",
                             "title": "The sign-in's password label",
                             "expected": "—", "remark": "held by jo",
                             "tone": "red", "held_by": "jo",
                             "working": false},
                            {"bead": "yd-x2", "title": "Held by another",
                             "expected": "2h–3h",
                             "remark": "held by yardmaster",
                             "tone": "red", "held_by": "yardmaster",
                             "working": false},
                            {"bead": "yd-x1", "title": "Blocked behind it",
                             "expected": "1h–2h", "remark": "blocked by footplate-p512",
                             "tone": "amber", "working": false}]},
                        {"stage": "decide", "group": "decisions", "human": true,
                         "lamp": "none", "terminal": false, "limit": 1, "taken": 0,
                         "beads": []},
                        {"stage": "review", "group": "muse-reviewers", "human": false,
                         "lamp": "none", "terminal": false, "limit": 2, "taken": 2,
                         "beads": [
                            {"bead": "yd-zulu", "title": "Zulu's late turn",
                             "expected": "5h–8h", "remark": "late by 40m",
                             "tone": "amber", "working": false},
                            {"bead": "yd-r2", "title": "Second in review",
                             "expected": "2h–3h", "remark": "", "tone": "none",
                             "working": true, "worker": "herdr yd-r2-review",
                             "entered_stage_at": "2026-10-10T09:30:00Z"},
                            {"bead": "yd-r3", "title": "Third in review",
                             "expected": "3h–4h", "remark": "", "tone": "none",
                             "working": false}]},
                        {"stage": "review-security", "group": "muse-security", "human": false,
                         "lamp": "turn", "terminal": false, "limit": 1, "taken": 0,
                         "beads": [
                            {"bead": "yd-alpha", "title": "Alpha moves first",
                             "expected": "20m–35m", "remark": "waits for jo, yardmaster",
                             "tone": "amber", "working": false},
                            {"bead": "yd-w1", "title": "A person's turn",
                             "expected": "1h–2h", "remark": "",
                             "tone": "none", "working": false,
                             "entered_stage_at": "2026-10-10T09:45:00Z"}]},
                        {"stage": "approved", "group": "shippers", "human": false,
                         "lamp": "none", "terminal": true, "limit": 2, "taken": 0,
                         "beads": []},
                    ],
                    "landings": [
                        {"bead": "yd-r9pn", "title": "Serve the landings",
                         "landed_at": landed_at(2, 15)},
                        {"bead": "yd-points", "title": "Oil the points",
                         "landed_at": landed_at(5, 40)},
                    ]},
                    {"name": "design.design", "type": "design", "source": "store", "stages": [
                        {"stage": "draft", "group": "mckean", "human": true,
                         "lamp": "none", "terminal": false, "limit": 1, "taken": 0,
                         "beads": [
                            {"bead": "yd-d1", "title": "Drafts the follow-up",
                             "expected": "3h–5h", "remark": "",
                             "tone": "none", "working": false}]},
                        {"stage": "critique", "group": "mckean", "human": true,
                         "lamp": "none", "terminal": true, "beads": []},
                    ]},
                ], "elsewhere": [
                    {"bead": "yardr-p9", "title": "A pinned flow's bead",
                     "expected": "—", "remark": "", "tone": "none",
                     "working": false},
                ]},
                {"name": "footplate", "kind": "git", "travelling": 2, "flows": [
                    {"name": "default", "type": "task", "source": "store", "stages": [
                        {"stage": "new", "group": "muse-builders", "human": false,
                         "lamp": "none", "terminal": false, "limit": 2, "taken": 1,
                         "beads": [
                            {"bead": "footplate-f1", "title": "A quiet bead",
                             "expected": "1h–2h", "remark": "",
                             "tone": "none", "working": false}]},
                    ]},
                ], "elsewhere": []},
                {"name": "drafts", "kind": "dir", "travelling": 0,
                 "flows": [], "elsewhere": []},
            ],
            "unassigned": [
                {"bead": "yd-s1", "title": "Travels light",
                 "expected": "2h–3h", "remark": "", "tone": "none",
                 "working": false},
                {"bead": "yd-s2", "title": "Waits with no flow",
                 "expected": "—", "remark": "waits for yardmaster, jo",
                 "tone": "amber", "working": false},
            ],
        }})));
        let served = match state {
            "lines-arrivals" | "lines-flow" => with_quiet_flows(served),
            _ => served,
        };
        yard.answer("Flows", &served.to_string());
    }
    if state == "lines-board" {
        // The board this state shoots: footplate's pull-request flow
        // with one bead past triage, two terminals, and two landings,
        // which stand under merged alone. Compact, so the shot holds
        // every column.
        yard.answer(
            "Flows",
            &json!({"result": {
                "travelling": 1,
                "depots": [
                    {"name": "footplate", "kind": "git", "travelling": 1, "flows": [
                        {"name": "contributors.pr", "type": "pr", "source": "store", "stages": [
                            {"stage": "contributor", "group": "mckean", "human": true,
                             "lamp": "turn", "terminal": false, "limit": 1, "taken": 0,
                             "beads": [
                                {"bead": "footplate-c1",
                                 "title": "A contributor's pull request",
                                 "expected": "1h–2h", "remark": "waits for jo",
                                 "tone": "amber", "working": false}]},
                            {"stage": "merged", "group": "", "human": false,
                             "lamp": "none", "terminal": true, "beads": []},
                            {"stage": "closed", "group": "", "human": false,
                             "lamp": "none", "terminal": true, "beads": []},
                        ],
                        "landings": [
                            {"bead": "yd-r9pn", "title": "Serve the landings",
                             "landed_at": landed_at(2, 15)},
                            {"bead": "yd-points", "title": "Oil the points",
                             "landed_at": landed_at(5, 40)},
                        ]},
                    ], "elsewhere": []},
                ],
                "unassigned": [],
            }})
            .to_string(),
        );
    }
    if state == "lines-arrivals"
        || state == "lines-flow"
        || state == "lines-context"
        || state == "lines-served"
    {
        // The asks the head's pills count: two open on beads of
        // yardr's default flow, and two beads parked for jo's move on
        // yardr's design flow and signalbox's, so the tree reads rust
        // where they stand.
        yard.answer(
            "UserAsks",
            &json!({"result": {"open": [
                {"bead": "footplate-p512", "bead_title": "The sign-in's password label",
                 "ask": 1, "question": "Hold it for the release?",
                 "asker": "yardmaster", "since": "2026-10-10T10:00:00Z"},
                {"bead": "yd-zulu", "bead_title": "Zulu's late turn",
                 "ask": 2, "question": "Land it past the late mark?",
                 "asker": "yardmaster", "since": "2026-10-10T10:05:00Z"},
            ], "decided": []}})
            .to_string(),
        );
        let midnight = shot_midnight();
        yard.answer(
            "Waiting",
            &json!({"result": [
                {"bead": "yd-d1", "title": "Drafts the follow-up",
                 "depot": "yardr", "stage": "draft", "group": "mckean",
                 "since": shot_since(&midnight, 1, 14, 10), "revision": "01KJ9",
                 "actions": []},
                {"bead": "signalbox-i1", "title": "An issue to triage",
                 "depot": "signalbox", "stage": "triage", "group": "mckean",
                 "since": shot_since(&midnight, 0, 8, 45), "revision": "01KJA",
                 "actions": []},
            ]})
            .to_string(),
        );
    }
}

/// The rich flows with three beads more on footplate's `new` stage, two of
/// them worked, so that two work there and two wait for a slot, as yardr.dev's
/// line stood at 16:44 on 2 Nov with its waiting beads listed first: the
/// station draws one chip, a worked one, with "+3" after it, under a
/// track head that reads "2 working". Added to the literal, not written
/// in it: it is long enough as it stands for json! to reach its recursion
/// limit.
fn with_two_at_work(mut flows: serde_json::Value) -> serde_json::Value {
    let bead = |bead: &str, working: bool| {
        json!({"bead": bead, "title": "A bead on the stage", "expected": "1h–2h",
               "remark": "", "tone": "none", "working": working})
    };
    flows["result"]["travelling"] = json!(18);
    flows["result"]["depots"][1]["travelling"] = json!(5);
    flows["result"]["depots"][1]["flows"][0]["stages"][0]["beads"]
        .as_array_mut()
        .unwrap()
        .extend([
            bead("footplate-f2", false),
            bead("footplate-f3", true),
            bead("footplate-f4", true),
        ]);
    flows
}

/// The rich flows with this yard's longest names added: footplate's flow
/// of pull requests, whose stages of people have the longest heads, and
/// signalbox, whose flow has the longest line. They are added to the
/// literal, not written in it: it is long enough as it stands for json!
/// to reach its recursion limit.
fn with_the_longest_names(mut flows: serde_json::Value) -> serde_json::Value {
    let depots = flows["result"]["depots"].as_array_mut().unwrap();
    depots[1]["flows"].as_array_mut().unwrap().push(json!(
        {"name": "contributors.pr", "type": "pr", "source": "store", "stages": [
            {"stage": "triage", "group": "mckean", "human": true,
             "lamp": "none", "terminal": false, "limit": 1, "taken": 0, "beads": []},
            {"stage": "review", "group": "muse-reviewers", "human": false,
             "lamp": "none", "terminal": false, "limit": 2, "taken": 2, "beads": []},
            {"stage": "contributor", "group": "mckean", "human": true,
             "lamp": "turn", "terminal": false, "limit": 1, "taken": 0,
             "beads": [
                {"bead": "footplate-c1", "title": "A contributor's pull request",
                 "expected": "1h–2h", "remark": "waits for jo",
                 "tone": "amber", "working": false}]},
            {"stage": "approved", "group": "mckean", "human": true,
             "lamp": "none", "terminal": false, "limit": 1, "taken": 0, "beads": []},
            {"stage": "decide", "group": "decisions", "human": true,
             "lamp": "none", "terminal": false, "limit": 1, "taken": 0, "beads": []},
            {"stage": "merged", "group": "", "human": false,
             "lamp": "none", "terminal": true, "beads": []},
            {"stage": "closed", "group": "", "human": false,
             "lamp": "none", "terminal": true, "beads": []},
        ]}
    ));
    depots.push(json!(
        {"name": "signalbox", "kind": "git", "travelling": 1, "flows": [
            {"name": "contributors.issue", "type": "issue", "source": "store", "stages": [
                {"stage": "triage", "group": "mckean", "human": true,
                 "lamp": "turn", "terminal": false, "limit": 1, "taken": 0,
                 "beads": [
                    {"bead": "signalbox-i1", "title": "An issue to triage",
                     "expected": "—", "remark": "waits for yardmaster",
                     "tone": "amber", "working": false}]},
                {"stage": "backlog", "group": "mckean", "human": true,
                 "lamp": "none", "terminal": false, "limit": 1, "taken": 0, "beads": []},
                {"stage": "new", "group": "muse-builders", "human": false,
                 "lamp": "none", "terminal": false, "limit": 2, "taken": 1, "beads": []},
                {"stage": "review", "group": "muse-reviewers", "human": false,
                 "lamp": "none", "terminal": false, "limit": 2, "taken": 2, "beads": []},
                {"stage": "approved", "group": "shippers", "human": false,
                 "lamp": "none", "terminal": false, "limit": 2, "taken": 0, "beads": []},
                {"stage": "decide", "group": "decisions", "human": true,
                 "lamp": "none", "terminal": false, "limit": 1, "taken": 0, "beads": []},
                {"stage": "merged", "group": "", "human": false,
                 "lamp": "none", "terminal": true, "beads": []},
                {"stage": "closed", "group": "", "human": false,
                 "lamp": "none", "terminal": true, "beads": []},
            ]},
        ], "elsewhere": []}
    ));
    flows
}

/// The rich flows with a quiet flow on yardr: the release flow with
/// three beads waiting in its backlog and none past it, so the lines
/// shots draw its one line of path with its backlog. Added to the
/// literal, not written in it: it is long enough as it stands for
/// json! to reach its recursion limit.
fn with_quiet_flows(mut flows: serde_json::Value) -> serde_json::Value {
    let waiting = |bead: &str, title: &str| {
        json!({"bead": bead, "title": title, "expected": "—",
               "remark": "", "tone": "none", "working": false})
    };
    let stage = |stage: &str, human: bool, terminal: bool| {
        json!({"stage": stage, "group": "shippers", "human": human,
               "lamp": "none", "terminal": terminal, "beads": []})
    };
    flows["result"]["depots"][0]["flows"]
        .as_array_mut()
        .unwrap()
        .push(json!(
            {"name": "type/release", "type": "release", "source": "store", "stages": [
                {"stage": "backlog", "group": "mckean", "human": true,
                 "lamp": "none", "terminal": false, "beads": [
                    waiting("yd-cut1", "First to cut"),
                    waiting("yd-cut2", "Second to cut"),
                    waiting("yd-cut3", "Third to cut")]},
                stage("build", false, false),
                stage("sign", false, false),
                stage("decide", true, false),
                stage("verify", false, false),
                stage("publish", false, false),
                stage("published", false, true),
            ]}
        ));
    flows
}

/// The command of the run ask: longer than the window is wide.
fn long_command() -> String {
    let crew: Vec<String> = (1..=40).map(|n| format!("--crew builder-{n}")).collect();
    format!("yardr source grant crm read {}", crew.join(" "))
}

const RUN_DOES: &str = "the link with airy (it calls itself yardr, key 6lHks9_z…) takes 300 mails and asks an hour from it. It takes 30 now.";
const RUN_BACK: &str = "yardr peer allow airy --mail-limit 30";
/// The pairing card's trust line, as the yard weighs it.
const PAIR_WEIGHT: &str = "a new device watch of jo speaks as them: whoever holds its token calls as jo until the device is removed";
/// What a pairing approval ran, as the yard keeps it: the safe sentence,
/// with no code in it.
const PAIR_RESULT: &str = "a pairing was opened for the device watch of jo, good until 09:30:00: the code was shown to the approver alone and is kept nowhere";

fn run_of() -> serde_json::Value {
    let command = long_command();
    let argv: Vec<&str> = command.split(' ').skip(1).collect();
    json!({"argv": argv, "command": command, "does": [RUN_DOES], "back": RUN_BACK})
}

/// The frame shot's run: the pick's landing, short enough that its
/// command, its moves row and the thread stand in one 960x640 frame.
fn frame_run() -> serde_json::Value {
    json!({"command": "yardr land footplate-q2rd --onto main",
           "argv": ["land", "footplate-q2rd", "--onto", "main"],
           "does": ["Rebases onto main, runs .yardr/test, then fast-forwards main"],
           "back": ""})
}

/// The frame shot's open asks as the queue lists them, the open card's
/// two first: ask 1, the landing, and ask 2, a pick on the same bead.
/// Two more beads wait under them, one with two asks, and one asked by
/// the assembly. None names a topic, so the list draws no topic's head. Their
/// ages are from now, so the list reads minutes and hours whenever the
/// shot runs. The yardmaster asks the open card's, and can be written to
/// from here, so no unreachable line stands under its moves.
fn queued_frame() -> Vec<serde_json::Value> {
    vec![
        json!({
            "bead": "footplate-q2rd", "bead_title": "Show the open asks on the dock badge",
            "topic": "", "ask": 1, "question": "May I land it?",
            "asker": "yardmaster", "since": landed_at(0, 12), "run": frame_run()
        }),
        json!({
            "bead": "footplate-q2rd", "bead_title": "Show the open asks on the dock badge",
            "topic": "", "ask": 2,
            "question": "Should the badge count parked beads too?",
            "asker": "yardmaster", "since": landed_at(0, 9)
        }),
        json!({
            "bead": "yardr-m4kt", "bead_title": "Quiet hours for the yard's notifications",
            "topic": "", "ask": 4,
            "question": "Hold every notification overnight, or only mail?",
            "asker": "shaper", "since": landed_at(1, 40)
        }),
        json!({
            "bead": "yardr-m4kt", "bead_title": "Quiet hours for the yard's notifications",
            "topic": "", "ask": 5,
            "question": "Which hours count as the night?",
            "asker": "shaper", "since": landed_at(1, 35)
        }),
        json!({
            "bead": "plantime-7wq3", "bead_title": "Fares page loads in under a second",
            "topic": "", "ask": 6,
            "question": "The cache doubles memory on staging. Keep it?",
            "asker": "assembly", "since": landed_at(3, 5)
        }),
    ]
}

/// The bead of the frame's open card, with its two asks and its thread:
/// the builder's note, the review's note with the shot image, and the
/// yardmaster's word on what changed since.
fn bead_frame() -> String {
    json!({"result": {"id": "footplate-q2rd", "title": "Show the open asks on the dock badge",
        "body": "The Dock icon carries the number of open asks, as the tray does.",
        "stage": "decide", "labels": ["eyes"],
        "notes": [
            {"seq": 1, "kind": "note", "author": "muse-wonder",
             "text": "Built: the badge counts open asks and clears as each is decided. Tests pass.",
             "at": landed_at(1, 50)},
            {"seq": 2, "kind": "note", "author": "ada",
             "text": "Review passed. The dock with three open asks.",
             "at": landed_at(0, 40),
             "files": [{"hash": IMAGE_HASH, "type": "image/png",
                        "bytes": IMAGE.len(), "name": "dock-badge.png"}]},
            {"seq": 3, "kind": "note", "author": "yardmaster",
             "text": "main moved two commits since the review; neither touches src/dock.rs.",
             "at": landed_at(0, 14)}],
        "asks": [
            {"ask": 1, "to": "jo", "asker": "yardmaster",
             "since": landed_at(0, 12),
             "question": "May I land it?",
             "context": ["Review passed at 3f1c2ab; main has moved two commits since"],
             "options": [], "state": "open", "run": frame_run()},
            {"ask": 2, "to": "jo", "asker": "yardmaster",
             "since": landed_at(0, 9),
             "question": "Should the badge count parked beads too?",
             "context": ["Three beads wait for your move today; the badge leaves them out."],
             "options": [
                {"label": "Asks only", "consequence": "the badge stays as built"},
                {"label": "Asks and parked beads", "consequence": "a follow-up bead counts both"},
                {"label": "You decide", "consequence": "leave it to yardmaster"}],
             "recommend": 1, "why": "parked beads already show in the list",
             "state": "open"}]}})
    .to_string()
}

/// The frame's other beads, each with its open asks, as the Bead call
/// answers them by id.
fn beads_frame() -> [(&'static str, String); 2] {
    [
        (
            "yardr-m4kt",
            json!({"result": {"id": "yardr-m4kt", "title": "Quiet hours for the yard's notifications",
                "body": "No notification wakes the operator at night.",
                "stage": "shape", "labels": [],
                "notes": [{"seq": 1, "kind": "note", "author": "shaper",
                           "text": "Two questions before I draft the design.",
                           "at": landed_at(1, 45)}],
                "asks": [
                    {"ask": 4, "to": "jo", "asker": "shaper", "since": landed_at(1, 40),
                     "question": "Hold every notification overnight, or only mail?",
                     "options": [
                        {"label": "Everything", "consequence": "asks wait for the morning too"},
                        {"label": "Only mail", "consequence": "asks still reach you at night"}],
                     "state": "open"},
                    {"ask": 5, "to": "jo", "asker": "shaper", "since": landed_at(1, 35),
                     "question": "Which hours count as the night?",
                     "options": [], "state": "open"}]}})
            .to_string(),
        ),
        (
            "plantime-7wq3",
            json!({"result": {"id": "plantime-7wq3", "title": "Fares page loads in under a second",
                "body": "The fares page takes 2.4s on staging.",
                "stage": "build", "labels": [],
                "notes": [{"seq": 1, "kind": "note", "author": "assembly",
                           "text": "With the cache the page loads in 0.6s; memory goes from 210 to 430 MB.",
                           "at": landed_at(3, 10)}],
                "asks": [
                    {"ask": 6, "to": "jo", "asker": "assembly", "since": landed_at(3, 5),
                     "question": "The cache doubles memory on staging. Keep it?",
                     "options": [
                        {"label": "Keep it", "consequence": "ship with the cache"},
                        {"label": "Cap it at 300 MB", "consequence": "slower on a cold start"},
                        {"label": "Drop it", "consequence": "find another way"}],
                     "state": "open"}]}})
            .to_string(),
        ),
    ]
}

/// The decide shot's flows: footplate's own flow with the open bead
/// standing at decide, the second of five stages, so that its head
/// draws a dot behind, its own and three ahead with the stage named.
/// The yardmaster's role as the `crew-context` shot reads it: past
/// thirty lines, so the Role tab folds it behind its count, with the
/// headings, the list and the code the window's Markdown draws.
fn crew_role_text() -> String {
    [
        "# Yardmaster",
        "",
        "You coordinate the work in this yard across its depots. You do not implement: builders do, each bead in its own worktree, and reviewers check their work.",
        "",
        "## First run",
        "",
        "Read `yardr doctor` when you start. Its \"Setup\" part lists the steps below, each `done` or `todo`, and names the next one: while a step is `todo`, your job is the setup, with mckean. Go on from the first step not done, never from the top.",
        "",
        "The onboarding ends in `yardr chat`, which shows mckean the mail between you and nothing of your pane. So when your first prompt asks you to greet them, do it by mail: `yardr mail mckean \"…\"`.",
        "",
        "## The work",
        "",
        "What you do:",
        "",
        "- **Triage.** Read your inbox when nudged. Things that went wrong (gave_up, session_blocked, session_stalled, move_refused, unrouted, stranded…) and beads agents held are yours to understand and resolve, or to bring to mckean.",
        "- **Shape work.** Beads wait in stage `backlog`, where `yardr bead create` puts them. When a bead is specified well enough for a builder to finish it without asking (a clear goal, where to look, what done means, how to test), advance it: `yardr bead advance <id> --to new`.",
        "- **Watch the line.** A bead that has stood past its expected arrival is late: find who holds it and why, and say so before mckean asks.",
        "- **Keep the crew.** The crew's sessions are yours to start and stop; a session that failed to start is yours to understand first.",
        "",
        "## Mail",
        "",
        "Write short declarative sentences that say what a thing *is* or *does*. Address mckean as \"you\" only for things they do; describe the system in the third person. No exclamation marks, no emoji, no hype.",
        "",
        "Numbers first: versions, sizes, counts. Never an adjective where a number says it.",
        "",
        "## Asks",
        "",
        "An ask you file carries its question, what you tried, and what you recommend. Weighty cards go through the operator's unlock; the window takes the ask's number typed back where no unlock is offered.",
        "",
        "Decide in the open: the bead's history shows every move, and your notes say why. A question too small for a bead goes by mail.",
        "",
        "## End of day",
        "",
        "Nothing waits on mckean overnight that can wait on you instead. What waits on them stands in Decide with its question whole, its context marked, and your recommendation.",
    ]
    .join("\n")
        + "\n"
}

/// The yardmaster's prime as the `crew-context` and `crew-prime` shots
/// read it: its role, the operator's style and note, the three newest of
/// seven memory titles, one of them retiring, and the rules files of
/// footplate's checkout, the other depot having none.
fn crew_prime() -> String {
    json!({"result": {
        "role": {"path": "/home/jo/.yardr/crew/yardmaster/roles/yardmaster.md",
                 "text": crew_role_text().trim()},
        "operator": {
            "name": "jo",
            "style": "show",
            "label": "Show me",
            "tells": "Explain briefly what you do. Show the command you run. Ask when there is a decision to make.",
            "note": "I read diffs, not prose"},
        "memories": [
            {"id": "mem-freeze", "title": "The signal box is closed for the freeze",
             "scope": "crew:yardmaster", "expires": "2026-11-27T09:30:00Z",
             "updated_at": "2026-11-23T16:10:00Z"},
            {"id": "mem-lamps", "title": "Lamps are lit at dusk", "scope": "all",
             "updated_at": "2026-11-20T09:30:00Z"},
            {"id": "mem-landing", "title": "Land one bead at a time: the base moves under a second",
             "scope": "all", "updated_at": "2026-11-18T11:45:00Z"}],
        "memories_omitted": 4,
        "depots": [
            {"name": "footplate", "rules": [
                {"path": "/home/jo/projects/footplate/AGENTS.md",
                 "text": "# footplate\n\nA native chat window onto a yardr yard.\n\n## Layout\n\nTop level: `src/` the program.\n"},
                {"path": "/home/jo/projects/footplate/CLAUDE.md", "text": "@AGENTS.md\n"}]},
            {"name": "notes", "rules": []}],
    }})
    .to_string()
}

/// The `Line` answer of the drift shots: yardr's depot as one store flow,
/// `default`, whose stage files the repository's flow of that name holds
/// too, in the shape the yard answers with. Right after `yardr flow
/// copy-repo` (`edited` false) it carries no key, the files being the
/// repository's as copied. After the first stage's file was edited
/// (`edited`) the flow and that stage carry `differs_from_repo`, and its
/// file has the text it was saved over beside the new one; `review` stays as
/// the repository has it, so only the flow's key tells of it.
fn line_drift(edited: bool) -> String {
    let copied = "A new bead waits here until someone has read it and shaped it.\n\nOutcomes: `done` → review\n";
    let saved = "A new bead waits here until someone has read it and shaped it.\n\nCheck that it names a depot and a goal before it moves on.\n\nOutcomes: `done` → review\n";
    let review = "A reviewer reads the change against the bead's goal.\n\nOutcomes: `done` → merged · `changes` → new\n";
    let mut flow = json!({"name": "default", "type": "", "source": "store", "stages": [
        {"stage": "new", "next": ["review"], "outcomes": [{"name": "done", "to": "review"}],
         "terminal": false, "other_routes": [], "open": 0,
         "stage_file": {"path": "/home/jo/.yardr/flows/default/new.md",
           "text": copied, "digest": crate::fake::digest(copied)}},
        {"stage": "review", "next": ["merged"], "outcomes": [{"name": "done", "to": "merged"}],
         "terminal": false, "other_routes": [], "open": 0,
         "stage_file": {"path": "/home/jo/.yardr/flows/default/review.md",
           "text": review, "digest": crate::fake::digest(review)}},
        {"stage": "merged", "next": [], "terminal": true, "other_routes": [], "open": 0}]});
    if edited {
        flow["differs_from_repo"] = json!(true);
        let new = &mut flow["stages"][0];
        new["differs_from_repo"] = json!(true);
        new["stage_file"] = json!({"path": "/home/jo/.yardr/flows/default/new.md",
            "text": saved, "digest": crate::fake::digest(saved),
            "prior_text": copied, "prior_digest": crate::fake::digest(copied)});
    }
    json!({"result": {"depot": "yardr", "kind": "git", "flows": [flow], "crew": []}}).to_string()
}

fn flows_frame() -> String {
    json!({"result": {
        "travelling": 1,
        "depots": [
            {"name": "footplate", "kind": "git", "travelling": 1, "flows": [
                {"name": "default", "type": "task", "source": "store", "stages": [
                    {"stage": "new", "group": "muse-builders", "human": false,
                     "lamp": "none", "terminal": false, "limit": 2, "taken": 1,
                     "beads": []},
                    {"stage": "decide", "group": "decisions", "human": true,
                     "lamp": "turn", "terminal": false, "limit": 1, "taken": 0,
                     "beads": [
                        {"bead": "footplate-q2rd",
                         "title": "Show the open asks on the dock badge",
                         "expected": "—", "remark": "waits for jo",
                         "tone": "amber", "working": false}]},
                    {"stage": "review", "group": "muse-reviewers", "human": false,
                     "lamp": "none", "terminal": false, "limit": 2, "taken": 2,
                     "beads": []},
                    {"stage": "review-security", "group": "muse-security", "human": false,
                     "lamp": "none", "terminal": false, "limit": 1, "taken": 0,
                     "beads": []},
                    {"stage": "approved", "group": "shippers", "human": false,
                     "lamp": "none", "terminal": true, "limit": 2, "taken": 0,
                     "beads": []},
                ]},
            ], "elsewhere": []},
        ],
        "unassigned": [],
    }})
    .to_string()
}

/// The weighty shot's command: the mail limit the trust class is of,
/// short enough that both cards stand in one shot.
fn weighed_run_of() -> serde_json::Value {
    json!({"command": "yardr peer allow airy --mail-limit 300",
           "argv": ["peer", "allow", "airy", "--mail-limit", "300"],
           "does": [RUN_DOES], "back": RUN_BACK})
}

/// The pairing run of the pair shots: the watch of jo, reached at the
/// yard's address, in the yard's words.
fn pairing_run_of() -> serde_json::Value {
    json!({"command": "yardr user device pair jo watch",
           "argv": ["user", "device", "pair", "jo", "watch"],
           "does": ["opens a pairing for the device watch of jo, reached at https://mac.example.net.",
                    "jo has 1 device: phone (never used).",
                    "Approval opens the pairing for that device alone: its code is good for ten minutes and for one device, and whoever has the code within those minutes is answered with that device's token.",
                    "Nothing in the yard keeps the code after it is used or runs out.",
                    "The new device then calls as jo until yardr user device rm jo watch, and jo will then have 2 devices."],
           "back": "yardr user device pair --end ends it while it stands; once the code is used, yardr user device rm jo watch removes the device"})
}

/// The pairing ask 7 on bead-s as the queue lists it, open or, where
/// `decided`, answered with what the approval ran.
fn queued_pairing(decided: bool) -> serde_json::Value {
    let weight = PAIR_WEIGHT;
    let mut queued = json!({
        "bead": "bead-s", "bead_title": "A second device", "topic": "", "ask": 7,
        "question": "Pair the watch?", "asker": "yardmaster",
        "since": "2026-10-10T10:00:00Z", "run": pairing_run_of(),
        "weight": [{"class": "trust", "line": weight}]
    });
    if decided {
        queued["state"] = json!("answered");
        queued["ran"] = json!({"argv": pairing_run_of()["argv"], "result": PAIR_RESULT});
    }
    queued
}

/// The bead of that ask, decided with the queue where `decided`.
fn bead_pairing(decided: bool) -> String {
    let weight = PAIR_WEIGHT;
    let state = match decided {
        true => "answered",
        false => "open",
    };
    json!({"result": {"id": "bead-s", "title": "A second device", "body": "The body.",
        "stage": "new", "labels": [], "notes": [], "asks": [
            {"ask": 7, "to": "jo", "asker": "yardmaster",
             "since": "2026-10-10T10:00:00Z",
             "question": "Pair the watch?",
             "context": ["jo's watch arrived, and wants the yard."],
             "options": [], "state": state, "run": pairing_run_of(),
             "weight": [{"class": "trust", "line": weight}]}]}})
    .to_string()
}

/// The run ask 7 on bead-s as the queue lists it, asking `question`.
fn queued_run(question: &str) -> serde_json::Value {
    json!({
        "bead": "bead-s", "bead_title": "The link with airy", "topic": "", "ask": 7,
        "question": question, "asker": "yardmaster",
        "since": "2026-10-10T10:00:00Z", "run": run_of()
    })
}

/// The arrival yd-zulu whole, as the Bead call answers it.
fn bead_zulu(body: &str) -> String {
    json!({"result": {
        "id": "yd-zulu", "title": "Zulu's late turn",
        "body": body,
        "stage": "draft", "labels": ["size:medium", "eyes"],
        "notes": [
            {"seq": 3, "kind": "note", "author": "assembly",
             "text": "running late: the review has two beads ahead of it",
             "at": "2026-10-10T10:30:00Z"},
            {"seq": 5, "kind": "note", "author": "yardmaster",
             "text": "still late, and still first in line",
             "at": "2026-10-10T11:30:00Z"},
        ],
        "asks": [
            {"ask": 7, "to": "jo", "asker": "yardmaster",
             "since": "2026-10-10T11:00:00Z",
             "question": "Wait for it?", "state": "open"},
            {"ask": 8, "to": "jo", "asker": "assembly",
             "since": "2026-10-10T11:30:00Z",
             "question": "Land it now?", "state": "open"},
            {"ask": 6, "to": "jo", "asker": "yardmaster",
             "since": "2026-10-10T10:00:00Z",
             "question": "Shape it?", "state": "answered"},
        ],
    }})
    .to_string()
}

/// The bead of that ask, asking `question` with `context`.
fn bead_run(question: &str, context: &[&str]) -> String {
    json!({"result": {"id": "bead-s", "title": "The link with airy", "body": "The body.",
        "stage": "new", "labels": [], "notes": [], "asks": [
            {"ask": 7, "to": "jo", "asker": "yardmaster",
             "since": "2026-10-10T10:00:00Z",
             "question": question,
             "context": context,
             "options": [], "state": "open", "run": run_of()}]}})
    .to_string()
}

/// The teammate invite the teammates shot's ask proposes, with the
/// yard's sentences for it.
fn invite_run_of() -> serde_json::Value {
    json!({"command": "yardr user invite eve --email eve@example.net",
           "argv": ["user", "invite", "eve", "--email", "eve@example.net"],
           "does": ["adds the user eve with the role member, and maps eve@example.net to eve in web_jwt_users: signed in at the edge as eve@example.net, they act as eve.",
                   "reports the invite to the edge for the yard mckean/edgeyard (member add): until the report lands the card reads as not yet at the edge, and nobody is admitted meanwhile.",
                   "they set a password from the mail the edge sends to this email, then sign in; nothing the operator holds admits them."],
           "back": "yardr user uninvite eve", "back_by": "only the operator"})
}

/// The teammate invite ask 7 on bead-s as the queue lists it.
fn queued_teammate() -> serde_json::Value {
    json!({
        "bead": "bead-s", "bead_title": "Teammates", "topic": "", "ask": 7,
        "question": "Invite Eve?", "asker": "yardmaster",
        "since": "2026-10-10T10:00:00Z", "run": invite_run_of()
    })
}

/// The bead of that ask.
fn bead_teammate() -> String {
    json!({"result": {"id": "bead-s", "title": "Teammates", "body": "The body.",
        "stage": "new", "labels": [], "notes": [], "asks": [
            {"ask": 7, "to": "jo", "asker": "yardmaster",
             "since": "2026-10-10T10:00:00Z",
             "question": "Invite Eve?",
             "context": ["she works with us daily."],
             "options": [], "state": "open", "run": invite_run_of()}]}})
    .to_string()
}

/// What the secret ask asks for, and what its value reaches, in the yard's
/// words: the lines it gives a source's credential.
fn secret_of() -> serde_json::Value {
    json!({
        "source": "books", "env": ["BOOKS_TOKEN"],
        "line": "a secret: BOOKS_TOKEN, for the source books (https://books.example.com)",
        "reach": [
            "lands in: the source's env_file, or sources/books.env in the yard's home when it has none",
            "read by: declared calls (yardr call books ...) and the source's check, from the file at each call",
            "not reached: thread commands (yardr reply, thread open, thread state), which read the speaker's own file; a crew member's is given with --for-crew",
            "mint: where the source mints, these names are its inputs; a static token beside them is left as it is, and not used while they are set"
        ]
    })
}

/// What the ask for a harness's API key asks for, in the yard's words and
/// as its docs show it (api.md, `UserAsks`): no source and no reach, since
/// its card says where the value goes in the words of its kind.
fn harness_secret_of() -> serde_json::Value {
    json!({
        "source": "", "env": ["ANTHROPIC_API_KEY"], "crew": "yardmaster", "harness": "claude",
        "line": "a secret: ANTHROPIC_API_KEY, the API key Claude Code runs with, asked for crew member yardmaster; it is read by every Claude Code session that burns the operator's login (the crew, groups without login_for, the operator's conversations), from its next start"
    })
}

/// What a secret shot's ask says: its bead's title, its question, the
/// context under it and the yard's secret. The credential of a source, or
/// the API key a harness runs with.
struct SecretAsked {
    title: &'static str,
    question: &'static str,
    context: &'static [&'static str],
    secret: serde_json::Value,
}

fn secret_asked(harness: bool) -> SecretAsked {
    match harness {
        true => SecretAsked {
            title: "Read the CRM",
            question: "The API key Claude Code will use",
            context: &[],
            secret: harness_secret_of(),
        },
        false => SecretAsked {
            title: "The books",
            question: "The token for the books",
            context: &["The books were moved to the declared source."],
            secret: secret_of(),
        },
    }
}

/// The secret ask 7 on bead-s as the queue lists it, marked no secret
/// where the state shows it, and for a harness's key where it is.
fn queued_secret(unconcealed: bool, harness: bool) -> serde_json::Value {
    let asked = secret_asked(harness);
    let mut ask = json!({
        "bead": "bead-s", "bead_title": asked.title, "topic": "", "ask": 7,
        "question": asked.question, "asker": "yardmaster",
        "since": "2026-10-10T10:00:00Z", "secret": asked.secret
    });
    if unconcealed {
        ask["concealed"] = json!(false);
    }
    ask
}

/// The bead of that ask, marked as the queue marks it.
fn bead_secret(unconcealed: bool, harness: bool) -> String {
    let asked = secret_asked(harness);
    let mut ask = json!(
        {"ask": 7, "to": "jo", "asker": "yardmaster",
         "since": "2026-10-10T10:00:00Z",
         "question": asked.question,
         "context": asked.context,
         "options": [], "state": "open", "secret": asked.secret});
    if unconcealed {
        ask["concealed"] = json!(false);
    }
    json!({"result": {"id": "bead-s", "title": asked.title, "body": "The body.",
        "stage": "new", "labels": [], "notes": [], "asks": [ask]}})
    .to_string()
}

/// The setup-host shot's card: the courier pack on a yard where its
/// account stands, with the address a host fills. The yard's lines, as
/// it writes them: the pack, the credential, the call and the mint, the
/// host above the account's, and the grants.
const SETUP_HOST_TEMPLATE: &str = "https://{host}/api";
const SETUP_HOST_PACK: &str = "pack courier (embedded in yardr 1.2.3), of kind source, into the yard: it adds 1 source and copies 1 command file (1 command), and changes nothing else. It makes no hook and sets no credential's file.";
const SETUP_HOST_CREDENTIAL: &str = "source courier: its calls go to https://courier.example.com/api. Its credential is the variable COURIER_TOKEN (the name, never the value), minted from COURIER_ACCOUNT by courier-mint, sent in the header \"Authorization: Bearer {credential}\" on every call.";
const SETUP_HOST_CALL: &str =
    "call ping (read): GET /me. It takes no parameter, and sends no query.";
const SETUP_HOST_MINT: &str = "its mint courier-mint is copied to <home>/sources/courier/courier-mint, sha256 275239824e00e61b0a220e61a41791c7e9b4bd726f8b0c27077a338f8131c9dc mode 0700. It runs with the operator's local logins, and a mint_command the operator sets wins over it.";
const SETUP_HOST_LINE: &str = "the host of source courier: its calls go to https://courier.example.com/api. To send them to a server of your own instead, give its host with --host <host> (a name or an address with an optional port, yours to type): they then go to https://<host>/api, and the account below is given for that host. The account that stands was given for the host in use, so a new host holds it back until you type it again.";
const SETUP_HOST_ACCOUNT: &str = "the account COURIER_ACCOUNT: given, in /home/jo/.yardr/sources/courier.env. Type another unseen to change it, or approve keeping it with a bare preset pick (written to /home/jo/.yardr/sources/courier.env, named in the answer, never the value).";
const SETUP_HOST_GRANTS: &str = "grants to crew yardmaster — pick one:";
const SETUP_HOST_BACK: &str =
    "yardr source rm courier; yardr source revoke courier <what> --crew yardmaster";
const SETUP_HOST_DECLARATION: &str = "name: courier\nbase: https://courier.example.com/api\nbase_template: https://{host}/api\ncredential: COURIER_TOKEN\n";
const SETUP_HOST_MINT_FILE: &str = "#!/bin/sh\n# mint COURIER_TOKEN from COURIER_ACCOUNT\nexec courier-mint \"$COURIER_ACCOUNT\"\n";
const SETUP_HOST_MINT_SHA: &str =
    "275239824e00e61b0a220e61a41791c7e9b4bd726f8b0c27077a338f8131c9dc";

/// What the setup-host shot's ask proposes.
fn setup_host_of() -> serde_json::Value {
    json!({"pack": "courier", "account": ["COURIER_ACCOUNT"],
           "account_file": "/home/jo/.yardr/sources/courier.env",
           "host_template": SETUP_HOST_TEMPLATE, "given": true,
           "candidates": [],
           "does": [SETUP_HOST_PACK, SETUP_HOST_CREDENTIAL, SETUP_HOST_CALL,
                    SETUP_HOST_MINT, SETUP_HOST_LINE, SETUP_HOST_ACCOUNT,
                    SETUP_HOST_GRANTS],
           "back": SETUP_HOST_BACK,
           "carried": [{"name": "courier", "text": SETUP_HOST_DECLARATION,
                         "files": [{"program": "courier-mint", "text": SETUP_HOST_MINT_FILE,
                                    "sha256": SETUP_HOST_MINT_SHA}]}],
           "presets": [{"name": "read", "grants": [{"source": "courier", "calls": ["read"]}]},
                       {"name": "everything",
                        "grants": [{"source": "courier", "calls": ["read", "put"]}]}],
           "to": [{"crew": "yardmaster"}]})
}

/// The setup ask 7 on bead-s as the queue lists it, open.
fn queued_setup_host() -> serde_json::Value {
    json!({
        "bead": "bead-s", "bead_title": "The courier", "topic": "", "ask": 7,
        "question": "Set up courier for the yardmaster?", "asker": "yardmaster",
        "since": "2026-10-10T10:00:00Z", "setup": setup_host_of()
    })
}

/// The bead of that ask, with two presets and `you decide`.
fn bead_setup_host() -> String {
    json!({"result": {"id": "bead-s", "title": "The courier", "body": "The body.",
        "stage": "new", "labels": [], "notes": [], "asks": [
            {"ask": 7, "to": "jo", "asker": "yardmaster",
             "since": "2026-10-10T10:00:00Z",
             "question": "Set up courier for the yardmaster?",
             "context": ["The yardmaster reads the courier to work beads."],
             "options": [{"label": "read", "consequence": "may make every read call of the source courier (https://courier.example.com/api): today that is ping"},
                         {"label": "everything", "consequence": "may make every call of the source courier (https://courier.example.com/api)"},
                         {"label": "You decide", "consequence": "the recommended preset, read, is granted"}],
             "recommend": 1, "why": "reading is the job",
             "state": "open", "setup": setup_host_of()}]}})
    .to_string()
}

/// The open ask of the decisions states: a question that wraps to two
/// lines in the list, with four options on its card.
fn queued_links() -> serde_json::Value {
    json!({
        "bead": "bead-a", "bead_title": "Move the links", "topic": "topic:linear-out",
        "ask": 3,
        "question": "Point this yard's 64 links at the declared source now, before Friday's release?",
        "asker": "yardmaster", "since": "2026-10-10T10:00:00Z"
    })
}

/// The bead of that ask, with its four options.
fn bead_links() -> String {
    json!({"result": {
        "id": "bead-a", "title": "Move the links",
        "body": "The body of bead-a.",
        "stage": "new", "labels": ["topic:linear-out"],
        "notes": [{"seq": 2, "kind": "note", "author": "yardmaster",
                   "text": "the test passed", "at": "2026-10-10T09:30:00Z"}],
        "asks": [{
            "ask": 3, "to": "jo", "asker": "yardmaster",
            "since": "2026-10-10T10:00:00Z",
            "question": "Point this yard's 64 links at the declared source now, before Friday's release?",
            "context": ["The links file declares the new source.",
                        "The move was tested."],
            "options": [
                {"label": "Move them now",
                 "consequence": "all 64 links point tonight"},
                {"label": "After Friday",
                 "consequence": "keep the old source"},
                {"label": "One lane first",
                 "consequence": "move the drafts lane tonight"},
                {"label": "You decide",
                 "consequence": "leave it to yardmaster"}
            ],
            "recommend": 1, "why": "the test passed",
            "state": "open"
        }]
    }})
    .to_string()
}

/// Midnight this morning, in the person shooting's zone: the history's
/// days stand against it, so the shot always shows a Today and a
/// Yesterday whatever day it is taken.
fn shot_midnight() -> DateTime<Local> {
    Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .single()
        .unwrap()
}

/// An asked time `days` days ago at `hour`:`min`, as the yard serves it.
fn shot_since(midnight: &DateTime<Local>, days: i64, hour: u32, min: u32) -> String {
    (*midnight - Duration::days(days)
        + Duration::hours(hour as i64)
        + Duration::minutes(min as i64))
    .to_rfc3339()
}

/// When a landing of the shot's flows landed, `hours` and `minutes`
/// ago, as the yard serves it: from now, not from midnight, so the
/// shot's ages read right whatever hour it runs.
fn landed_at(hours: i64, minutes: i64) -> String {
    (Local::now() - Duration::hours(hours) - Duration::minutes(minutes)).to_rfc3339()
}

/// The decisions states' history, newest first: 156 dealt with asks.
fn decided_history() -> serde_json::Value {
    let midnight = shot_midnight();
    let at = |days: i64, hour: u32, min: u32| shot_since(&midnight, days, hour, min);
    let mut decided = vec![
        json!({"bead": "yd-cal", "bead_title": "The calendar", "topic": "topic:linear-out",
               "ask": 101, "question": "Let me edit Google Docs in place", "asker": "yardmaster",
               "since": at(0, 11, 53), "state": "answered",
               "option": 1, "option_label": "Let them"}),
        json!({"bead": "yd-cal", "bead_title": "The calendar", "topic": "topic:linear-out",
               "ask": 102, "question": "Which inbox set should count as attention?", "asker": "mayor",
               "since": at(0, 10, 5), "state": "answered", "words": "just the direct ones"}),
        json!({"bead": "yd-train", "bead_title": "The release train", "topic": "",
               "ask": 103, "question": "Take the release train tonight?", "asker": "yardmaster",
               "since": at(0, 9, 12), "state": "taken back",
               "by": "yardmaster", "words": "the train left"}),
        json!({"bead": "yd-docs", "bead_title": "The quota", "topic": "topic:linear-out",
               "ask": 107, "question": "Raise Google Docs to 4000 calls", "asker": "mayor",
               "since": at(1, 20, 21), "state": "answered",
               "option": 1, "option_label": "Raise them"}),
        json!({"bead": "yd-docs", "bead_title": "The quota", "topic": "topic:linear-out",
               "ask": 106, "question": "Raise Google Docs to 3000 calls", "asker": "mayor",
               "since": at(1, 20, 19), "state": "answered",
               "option": 1, "option_label": "Raise them"}),
        json!({"bead": "yd-docs", "bead_title": "The quota", "topic": "topic:linear-out",
               "ask": 105, "question": "Raise Google Docs to 2000 calls", "asker": "mayor",
               "since": at(1, 20, 18), "state": "answered",
               "option": 1, "option_label": "Raise them"}),
        json!({"bead": "yd-docs", "bead_title": "The quota", "topic": "topic:linear-out",
               "ask": 104, "question": "Raise Google Docs to 1000 calls", "asker": "mayor",
               "since": at(1, 20, 17), "state": "answered",
               "option": 1, "option_label": "Raise them"}),
        json!({"bead": "yd-index", "bead_title": "The index", "topic": "",
               "ask": 108, "question": "Rebuild the search index tonight?", "asker": "yardmaster",
               "since": at(1, 18, 2), "state": "put aside",
               "return_at": at(-1, 9, 30)}),
        json!({"bead": "yd-ship", "bead_title": "The migration", "topic": "topic:linear-out",
               "ask": 109, "question": "Land the half-written migration?", "asker": "assembly",
               "since": at(1, 14, 44), "state": "declined", "words": "not yet"}),
        json!({"bead": "yd-books", "bead_title": "The books", "topic": "",
               "ask": 110, "question": "The token for the books", "asker": "yardmaster",
               "since": at(1, 9, 30), "state": "answered", "given": ["BOOKS_TOKEN"]}),
    ];
    // The older weeks: 146 asks over eleven days, newest first. The asker
    // turns every ask, so no two neighbours ever fold: the frame's rows
    // stand as written.
    let askers = ["mayor", "yardmaster", "assembly"];
    let services = ["api", "web", "worker", "store", "mail"];
    for (i, n) in (111..=256).enumerate() {
        let days = 2 + (i as i64) / 14;
        let hour = 18 - (i as u32 % 14);
        let min = (i as u32 * 7) % 60;
        let asker = askers[i % askers.len()];
        let mut ask = json!({
            "bead": format!("yd-past-{n}"), "bead_title": "The older weeks",
            "topic": "topic:linear-out", "ask": n,
            "question": format!("Approve the {} {} deploy?", services[i % services.len()], 100 + i),
            "asker": asker, "since": at(days, hour, min),
        });
        if n == 160 || n == 230 {
            ask["state"] = json!("declined");
            ask["words"] = json!("not now");
        } else if i % 13 == 7 {
            ask["state"] = json!("put aside");
        } else if i % 11 == 5 {
            ask["state"] = json!("taken back");
            ask["by"] = json!(asker);
            ask["words"] = json!("changed my mind");
        } else if i % 7 == 3 {
            ask["state"] = json!("answered");
            ask["words"] = json!("on it");
        } else {
            ask["state"] = json!("answered");
            ask["option"] = json!(1);
            ask["option_label"] = json!("Approve it");
        }
        decided.push(ask);
    }
    json!(decided)
}

/// Whether the list just drawn is the one the `switcher-shut` state shuts:
/// true for the first call in that state, false after it and in every
/// other. The state opens as `switcher` does, and the window then calls
/// what the list's listener calls for a press outside it. The shot scripts
/// drive no pointer, and gpui hands a window a mouse event only from its
/// platform, so the press itself is the window test's.
pub(crate) fn shuts_once() -> bool {
    static SHUT: AtomicBool = AtomicBool::new(false);
    std::env::var("FOOTPLATE_SHOT").as_deref() == Ok("switcher-shut")
        && !SHUT.swap(true, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session;
    use std::os::unix::fs::PermissionsExt;

    /// The history a state was loaded with. The shots' yards serve no
    /// `UserOpenAsks`, so each state is read whole, and the window starts
    /// from the history as the screen shows it; a yard that served the call
    /// would load none, and this says so.
    fn history(state: &crate::model::State) -> &[crate::api::DecidedAsk] {
        state
            .asks
            .decided
            .as_deref()
            .expect("a shot's yard serves no UserOpenAsks, so its state is read whole")
    }

    /// Every state reads as the window reads it: the login name is no user
    /// of the yard, so the window is for its operator, jo, and the state
    /// opens where the shot says. A state added to [`STATES`] lands in the
    /// unreachable arm until it is covered here. `stopped`, `refusal`,
    /// `setup`, `setup-terminal`, `setup-other`, `setup-linux`, `setup-copy`
    /// and `hosted` listen nowhere:
    /// their run's socket names a yard that is down.
    #[test]
    fn every_state_loads_as_the_window_reads_it() {
        for state in STATES {
            if matches!(
                *state,
                "stopped"
                    | "refusal"
                    | "setup"
                    | "setup-terminal"
                    | "setup-other"
                    | "setup-linux"
                    | "setup-copy"
                    | "hosted"
            ) {
                let env = Env {
                    socket: PathBuf::from("/none"),
                    ..Env::default()
                };
                let (env, shot) = start_for_state(env, state);
                assert!(shot.is_none(), "{state}");
                assert!(!env.socket.exists(), "{state}");
                if matches!(*state, "setup" | "setup-terminal") {
                    // The canned doctor the second step stands with: two
                    // installed providers with labels and offers, one
                    // recommended each.
                    let doctor = setup_doctor();
                    assert!(doctor.ready(), "{state}");
                    let kinds: Vec<_> = doctor
                        .installed()
                        .iter()
                        .map(|item| item.harness.clone().unwrap())
                        .collect();
                    assert_eq!(kinds, ["claude", "codex"], "{state}");
                    for item in doctor.installed() {
                        assert!(item.label.is_some(), "{state}");
                        assert!(item.recommended_offer().is_some(), "{state}");
                    }
                }
                if *state == "setup-other" {
                    // The canned doctor the needs step stands with: a claude
                    // that is not Claude Code, once as itself and once among
                    // the ones that can be fetched, which is one row; the
                    // fetched claude standing behind it, so that the row
                    // offers no second fetch; and a codex still to fetch.
                    let doctor = other_doctor();
                    assert!(!doctor.ready(), "{state}");
                    let rows = doctor.rows();
                    let programs: Vec<_> = rows.iter().map(|item| item.program.as_str()).collect();
                    assert_eq!(programs, ["git", "herdr", "claude", "codex"], "{state}");
                    assert!(rows[2].answers_as_another(), "{state}");
                    assert!(rows[2].another_said().is_some(), "{state}");
                    assert_eq!(doctor.shadowed(|_| true), ["claude"], "{state}");
                }
                if matches!(*state, "setup-linux" | "setup-copy") {
                    // The canned doctor of a Linux box: herdr to install in
                    // one click where yardr pins its file (setup-linux), and
                    // where it does not, the line to install it by hand with
                    // the command alone to copy (setup-copy); git has a
                    // page and nothing to copy, and no harness is there.
                    let doctor = linux_doctor(*state == "setup-linux");
                    assert!(!doctor.ready(), "{state}");
                    let rows = doctor.rows();
                    let programs: Vec<_> = rows.iter().map(|item| item.program.as_str()).collect();
                    assert_eq!(programs, ["git", "herdr", "a harness"], "{state}");
                    assert_eq!(rows[1].fetch.is_some(), *state == "setup-linux", "{state}");
                    assert_eq!(
                        rows[1].command.as_deref(),
                        Some("curl -fsSL https://herdr.dev/install.sh | sh"),
                        "{state}"
                    );
                    assert!(
                        rows[0].command.is_none() && rows[0].page().is_some(),
                        "{state}"
                    );
                }
                continue;
            }
            let yard = Yard::start();
            populate(&yard, state);
            let loaded = session::load(&yard.client(), "shot").unwrap();
            assert_eq!(loaded.me, "jo", "{state}");
            match *state {
                "decisions" | "decisions-filtered" | "decisions-more" => {
                    assert_eq!(history(&loaded).len(), 156, "{state}")
                }
                "waiting" | "waiting-folded" | "waiting-end" | "old-waiting" | "lines-waiting"
                | "lines-waiting-end" | "lines-help" | "lines-panel" | "lines-departures"
                | "lines-siding" | "frame" | "decide" => {
                    assert_eq!(history(&loaded).len(), 12, "{state}")
                }
                "pair-qr" | "pair-ended" | "delivered-message" => {
                    assert_eq!(history(&loaded).len(), 1, "{state}")
                }
                _ => assert_eq!(history(&loaded).len(), 0, "{state}"),
            }
            match *state {
                "yardmaster"
                | "far"
                | "settings"
                | "settings-appearance"
                | "settings-yards"
                | "settings-yards-rename"
                | "settings-agents"
                | "settings-notifications"
                | "settings-connection"
                | "settings-connection-edge"
                | "settings-voice"
                | "settings-voice-list"
                | "settings-voice-microphone"
                | "settings-telemetry"
                | "settings-telemetry-off"
                | "voice"
                | "voice-ready"
                | "voice-draft"
                | "reconnecting"
                | "answered"
                | "signedout"
                | "away"
                | "switcher"
                | "switcher-shut"
                | "switcher-member"
                | "member-pair"
                | "about"
                | "about-missing"
                | "feedback" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 6, "{state}");
                    assert_eq!(letters[1].from, "jo", "{state}");
                    // The switch the telemetry shots open on: the
                    // section's install id and next payload, or the
                    // consent words and Turn on.
                    if *state == "settings-telemetry" {
                        let preview = loaded.telemetry.as_ref().unwrap();
                        assert_eq!(preview.telemetry, "on", "{state}");
                        assert!(!preview.install_id.is_empty(), "{state}");
                        assert!(preview.request.is_some(), "{state}");
                        assert!(!loaded.telemetry_unserved, "{state}");
                    }
                    if *state == "settings-telemetry-off" {
                        let preview = loaded.telemetry.as_ref().unwrap();
                        assert_eq!(preview.telemetry, "off", "{state}");
                        assert!(preview.request.is_none(), "{state}");
                        assert!(!loaded.telemetry_unserved, "{state}");
                    }
                }
                "arrivals" | "departures" | "arrival" | "flow" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 6, "{state}");
                    assert_eq!(letters[1].from, "jo", "{state}");
                    // The board the window opens: three rows, with an
                    // amber remark, none, and a red one. The arrivals and
                    // departures states shoot their own board over that
                    // one: a late bead with its overrun, a bead that
                    // waits on jo, and one jo holds.
                    assert_eq!(loaded.arrivals.rows.len(), 3, "{state}");
                    assert_eq!(loaded.arrivals.rows[0].bead, "yd-zulu", "{state}");
                    let overrun = *state == "arrivals" || *state == "departures";
                    assert_eq!(
                        (
                            loaded.arrivals.rows[0].remark.as_str(),
                            loaded.arrivals.rows[0].tone.as_str()
                        ),
                        match overrun {
                            true => ("late by 40m", "amber"),
                            false => ("late", "amber"),
                        },
                        "{state}"
                    );
                    assert_eq!(
                        loaded.arrivals.rows[1].remark.as_str(),
                        match overrun {
                            true => "waits for jo, yardmaster",
                            false => "",
                        },
                        "{state}"
                    );
                    assert_eq!(
                        (
                            loaded.arrivals.rows[2].expected.as_str(),
                            loaded.arrivals.rows[2].remark.as_str(),
                            loaded.arrivals.rows[2].tone.as_str(),
                            loaded.arrivals.rows[2].held_by.as_str()
                        ),
                        ("—", "held by jo", "red", "jo"),
                        "{state}"
                    );
                    // The flows view beside the board: three travelling,
                    // yardr's two on a turn and a quiet stage, footplate's
                    // one on a fault stage, and nothing without a line.
                    // The arrivals, departures and flow states shoot the
                    // richer one instead: yardr's two flows with a
                    // backlog, an Elsewhere bead and two beads with no
                    // line, footplate's two, drafts quiet, and signalbox
                    // with the longest line name.
                    let rich = overrun || *state == "flow";
                    assert_eq!(
                        loaded.flows.travelling,
                        match rich {
                            true => 18,
                            false => 3,
                        },
                        "{state}"
                    );
                    assert_eq!(loaded.flows.depots[0].name, "yardr", "{state}");
                    assert_eq!(
                        loaded.flows.depots[0].travelling,
                        match rich {
                            true => 10,
                            false => 2,
                        },
                        "{state}"
                    );
                    assert_eq!(
                        loaded.flows.depots.len(),
                        match rich {
                            true => 4,
                            false => 2,
                        },
                        "{state}"
                    );
                    if rich {
                        let stages = &loaded.flows.depots[0].flows[0].stages;
                        assert_eq!(
                            stages
                                .iter()
                                .map(|s| (s.stage.as_str(), s.human))
                                .collect::<Vec<_>>(),
                            [
                                ("backlog", true),
                                ("new", false),
                                ("decide", true),
                                ("review", false),
                                ("review-security", false),
                                ("approved", false),
                            ],
                            "{state}"
                        );
                        assert_eq!(stages[1].lamp, "fault", "{state}");
                        assert_eq!(stages[4].lamp, "turn", "{state}");
                        assert_eq!(stages[1].beads.len(), 3, "{state}");
                        // The fault stage holds a bead jo holds, one
                        // yardmaster holds, and the blocked bead behind
                        // them.
                        assert_eq!(
                            stages[1]
                                .beads
                                .iter()
                                .map(|b| (b.remark.as_str(), b.held_by.as_str()))
                                .collect::<Vec<_>>(),
                            [
                                ("held by jo", "jo"),
                                ("held by yardmaster", "yardmaster"),
                                ("blocked by footplate-p512", ""),
                            ],
                            "{state}"
                        );
                        assert_eq!(stages[3].beads.len(), 3, "{state}");
                        assert!(!stages[1].terminal, "{state}");
                        assert!(stages[5].terminal, "{state}");
                        assert_eq!(
                            (stages[3].limit, stages[3].taken),
                            (Some(2), Some(2)),
                            "{state}"
                        );
                        assert_eq!(stages[3].beads[1].worker, "herdr yd-r2-review", "{state}");
                        assert_eq!(
                            stages[3].beads[1].entered_stage_at, "2026-10-10T09:30:00Z",
                            "{state}"
                        );
                        assert_eq!(loaded.flows.depots[0].flows.len(), 2, "{state}");
                        // What landed under the landed station, newest
                        // first.
                        let landings = &loaded.flows.depots[0].flows[0].landings;
                        assert_eq!(
                            landings
                                .iter()
                                .map(|l| (l.bead.as_str(), l.title.as_str()))
                                .collect::<Vec<_>>(),
                            [
                                ("yd-r9pn", "Serve the landings"),
                                ("yd-points", "Oil the points"),
                            ],
                            "{state}"
                        );
                        assert_eq!(loaded.flows.depots[0].elsewhere.len(), 1, "{state}");
                        // footplate's `new`: a bead waiting, one more waiting
                        // and two worked, the worked ones listed last.
                        assert_eq!(loaded.flows.depots[1].travelling, 5, "{state}");
                        assert_eq!(
                            loaded.flows.depots[1].flows[0].stages[0]
                                .beads
                                .iter()
                                .map(|b| (b.bead.as_str(), b.working))
                                .collect::<Vec<_>>(),
                            [
                                ("footplate-f1", false),
                                ("footplate-f2", false),
                                ("footplate-f3", true),
                                ("footplate-f4", true),
                            ],
                            "{state}"
                        );
                        assert_eq!(loaded.flows.depots[2].travelling, 0, "{state}");
                        // This yard's longest names: the stages of people
                        // with the longest heads, and the longest line.
                        let contributors = &loaded.flows.depots[1].flows[1];
                        assert_eq!(contributors.name, "contributors.pr", "{state}");
                        assert_eq!(
                            contributors
                                .stages
                                .iter()
                                .filter(|s| s.human)
                                .map(|s| s.stage.as_str())
                                .collect::<Vec<_>>(),
                            ["triage", "contributor", "approved", "decide"],
                            "{state}"
                        );
                        let signalbox = &loaded.flows.depots[3];
                        assert_eq!(signalbox.name, "signalbox", "{state}");
                        assert_eq!(signalbox.flows[0].name, "contributors.issue", "{state}");
                        assert_eq!(signalbox.flows[0].flow_type, "issue", "{state}");
                        assert_eq!(
                            loaded
                                .flows
                                .unassigned
                                .iter()
                                .map(|b| b.bead.as_str())
                                .collect::<Vec<_>>(),
                            ["yd-s1", "yd-s2"],
                            "{state}"
                        );
                    } else {
                        assert!(loaded.flows.unassigned.is_empty(), "{state}");
                    }
                    assert_eq!(
                        loaded.flows.depots[0].flows[0].stages[0].lamp,
                        match rich {
                            true => "none",
                            false => "turn",
                        },
                        "{state}"
                    );
                    // One arrival's bead, which the arrival state opens:
                    // the body, two labels, two notes, and three asks, two
                    // of them open.
                    if *state == "arrival" {
                        let bead = yard.client().bead("yd-zulu").unwrap();
                        assert_eq!(bead.title, "Zulu's late turn", "{state}");
                        assert_eq!(bead.stage, "draft", "{state}");
                        assert_eq!(bead.labels.len(), 2, "{state}");
                        // The body holds the whole-text Markdown sample.
                        assert!(
                            bead.body.contains("| Yard | Waiting | Working |"),
                            "{state}"
                        );
                        assert!(bead.body.contains("> Hold the autumn timetable"), "{state}");
                        assert!(
                            bead.body
                                .contains("[release page](https://yardr.dev/releases)"),
                            "{state}"
                        );
                        assert!(bead.body.ends_with("\n---"), "{state}");
                        assert_eq!(bead.notes.len(), 2, "{state}");
                        assert_eq!(bead.asks.len(), 3, "{state}");
                        assert_eq!(
                            bead.asks.iter().filter(|a| a.state == "open").count(),
                            2,
                            "{state}"
                        );
                    }
                }
                "lines-board" => {
                    // The board's one flow: a contributor's bead past
                    // triage, two terminals, and two landings, which the
                    // window stands under merged alone.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 6, "{state}");
                    assert_eq!(loaded.arrivals.rows.len(), 3, "{state}");
                    assert_eq!(loaded.flows.travelling, 1, "{state}");
                    assert_eq!(loaded.flows.depots.len(), 1, "{state}");
                    assert_eq!(loaded.flows.depots[0].name, "footplate", "{state}");
                    let flow = &loaded.flows.depots[0].flows[0];
                    assert_eq!(flow.name, "contributors.pr", "{state}");
                    assert_eq!(flow.flow_type, "pr", "{state}");
                    assert_eq!(
                        flow.stages
                            .iter()
                            .map(|s| (s.stage.as_str(), s.terminal))
                            .collect::<Vec<_>>(),
                        [("contributor", false), ("merged", true), ("closed", true)],
                        "{state}"
                    );
                    assert_eq!(flow.stages[0].beads.len(), 1, "{state}");
                    assert_eq!(flow.stages[0].beads[0].bead, "footplate-c1", "{state}");
                    assert_eq!(
                        flow.landings
                            .iter()
                            .map(|l| (l.bead.as_str(), l.title.as_str()))
                            .collect::<Vec<_>>(),
                        [
                            ("yd-r9pn", "Serve the landings"),
                            ("yd-points", "Oil the points"),
                        ],
                        "{state}"
                    );
                }
                "lines-arrivals" | "lines-flow" => {
                    // The rich flows with a quiet flow on yardr, the
                    // arrivals board over the shared one, two open asks
                    // on yardr's default flow and two beads parked for
                    // jo's move on footplate's and signalbox's.
                    assert_eq!(
                        loaded
                            .asks
                            .open
                            .iter()
                            .map(|ask| ask.bead.as_str())
                            .collect::<Vec<_>>(),
                        ["footplate-p512", "yd-zulu"],
                        "{state}"
                    );
                    assert!(!loaded.waiting_too_old, "{state}");
                    assert_eq!(
                        loaded
                            .parked
                            .iter()
                            .map(|p| p.bead.as_str())
                            .collect::<Vec<_>>(),
                        ["yd-d1", "signalbox-i1"],
                        "{state}"
                    );
                    assert_eq!(loaded.arrivals.rows.len(), 3, "{state}");
                    assert_eq!(loaded.arrivals.rows[0].bead, "yd-zulu", "{state}");
                    assert_eq!(loaded.flows.travelling, 18, "{state}");
                    assert_eq!(loaded.flows.depots.len(), 4, "{state}");
                    let release = &loaded.flows.depots[0].flows[2];
                    assert_eq!(release.name, "type/release", "{state}");
                    assert_eq!(release.flow_type, "release", "{state}");
                    assert_eq!(
                        release
                            .stages
                            .iter()
                            .filter(|s| s.stage != "backlog")
                            .map(|s| s.beads.len())
                            .sum::<usize>(),
                        0,
                        "{state}"
                    );
                    assert_eq!(
                        release
                            .stages
                            .iter()
                            .find(|s| s.stage == "backlog")
                            .map(|s| s.beads.len()),
                        Some(3),
                        "{state}"
                    );
                }
                "lines-context" => {
                    // The drawer's depot reads the same line: footplate's
                    // default flow at new, with its stage file, its
                    // serving group and the group's prompt, and review
                    // standing unserved.
                    let line = yard.client().line("footplate").unwrap();
                    assert_eq!(line.depot, "footplate", "{state}");
                    let flow = &line.flows[0];
                    assert_eq!(flow.name, "default", "{state}");
                    let new = &flow.stages[0];
                    assert_eq!(
                        new.stage_file.as_ref().unwrap().path,
                        ".yardr/flows/default/new.md",
                        "{state}"
                    );
                    let group = &new.route.as_ref().unwrap().chain[0];
                    assert_eq!(group.name, "muse-builders", "{state}");
                    let prompt = group.prompt.as_ref().unwrap();
                    assert!(prompt.carriable, "{state}");
                    assert_eq!(prompt.digest.len(), 64, "{state}");
                    assert!(flow.stages[1].route.is_none(), "{state}");
                }
                "crew-context" | "crew-prime" => {
                    // The drawer's member reads the first depot's line:
                    // footplate's, with the yardmaster's role prompt
                    // whole and carriable past the fold, and its prime,
                    // which the Prime tab lists: four items, three of
                    // seven memory titles, and one depot with rules.
                    let prime = yard.client().prime("yardmaster").unwrap();
                    assert!(prime.role.is_some(), "{state}");
                    assert_eq!(prime.operator.as_ref().unwrap().style, "show", "{state}");
                    assert_eq!(prime.memories.len(), 3, "{state}");
                    assert_eq!(prime.memories_omitted, 4, "{state}");
                    assert_eq!(prime.depots.len(), 2, "{state}");
                    assert_eq!(prime.depots[0].rules.len(), 2, "{state}");
                    assert!(prime.depots[1].rules.is_empty(), "{state}");
                    let line = yard.client().line("footplate").unwrap();
                    assert_eq!(line.depot, "footplate", "{state}");
                    let member = line
                        .crew
                        .iter()
                        .find(|member| member.name == "yardmaster")
                        .unwrap();
                    let prompt = member.prompt.as_ref().unwrap();
                    assert!(prompt.carriable, "{state}");
                    assert_eq!(prompt.digest.len(), 64, "{state}");
                    assert!(prompt.text.lines().count() > 30, "{state}");
                    assert_eq!(loaded.operator, "jo", "{state}");
                }
                "crew" => {
                    // The sidebar's groups full: three crew with their
                    // states, three of the linked yard, two people, three
                    // unread notices, one open ask, and one of jo's beads
                    // added past the head.
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    let crew: Vec<(&str, &str, &str)> = loaded
                        .crew
                        .iter()
                        .map(|r| (r.name.as_str(), r.state.as_str(), r.status.as_str()))
                        .collect();
                    assert_eq!(
                        crew,
                        [
                            ("assembly", "stopped", ""),
                            ("yardmaster", "running", "working"),
                            ("brakeman", "running", "idle"),
                        ],
                        "{state}"
                    );
                    let roster: Vec<&str> = loaded.peers[0]
                        .said
                        .as_ref()
                        .unwrap()
                        .roster
                        .iter()
                        .map(|e| e.name.as_str())
                        .collect();
                    assert_eq!(roster, ["clerk", "warden", "rue"], "{state}");
                    let users: Vec<&str> = loaded.users.iter().map(|u| u.name.as_str()).collect();
                    assert_eq!(users, ["jo", "ada"], "{state}");
                    assert_eq!(
                        loaded.notifications.iter().filter(|n| !n.read).count(),
                        3,
                        "{state}"
                    );
                    let mut events = yard
                        .client()
                        .events(20, crate::session::WATCH.quiet)
                        .unwrap();
                    let added = events.next().unwrap().unwrap();
                    assert_eq!(added.kind.as_str(), "created", "{state}");
                    assert_eq!(added.text("originator"), "jo", "{state}");
                }
                "plain" | "plain-select" | "lines" | "keys" => {
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    // The card these shots open carries the line
                    // `bead-a · ask 3` beside its disclosure: the bead's id
                    // and the ask's number.
                    assert_eq!(
                        (loaded.asks.open[0].bead.as_str(), loaded.asks.open[0].ask),
                        ("bead-a", 3),
                        "{state}"
                    );
                    let bead = yard.client().bead("bead-a").unwrap();
                    assert_eq!(bead.asks.len(), 1, "{state}");
                    assert_eq!(bead.asks[0].options.len(), 3, "{state}");
                    assert_eq!(bead.asks[0].recommend, 1, "{state}");
                    // The plain-select shot poses its selection over the
                    // first of these lines, past its first word.
                    if *state == "plain-select" {
                        assert_eq!(
                            bead.asks[0].context,
                            [
                                "The links file declares the new source.",
                                "The move was tested."
                            ],
                            "{state}"
                        );
                    }
                    // The lines badge's bead, added past the head: one of
                    // jo's.
                    if *state == "lines" {
                        let mut events = yard
                            .client()
                            .events(20, crate::session::WATCH.quiet)
                            .unwrap();
                        let added = events.next().unwrap().unwrap();
                        assert_eq!(added.kind.as_str(), "created", "{state}");
                        assert_eq!(added.text("originator"), "jo", "{state}");
                    }
                }
                "run" => {
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    assert!(loaded.asks.open[0].run.is_some(), "{state}");
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 1, "{state}");
                    assert_eq!(bead.asks[0].question.lines().count(), 3, "{state}");
                    assert!(bead.asks[0].run.is_some(), "{state}");
                }
                "frame" | "decide" => {
                    assert_eq!(loaded.asks.open.len(), 5, "{state}");
                    let run = loaded.asks.open[0].run.as_ref().unwrap();
                    assert_eq!(
                        run.line().as_str(),
                        "yardr land footplate-q2rd --onto main",
                        "{state}"
                    );
                    let bead = yard.client().bead("footplate-q2rd").unwrap();
                    assert_eq!(bead.asks.len(), 2, "{state}");
                    assert!(bead.asks[0].run.is_some(), "{state}");
                    assert_eq!(bead.asks[1].options.len(), 3, "{state}");
                    assert_eq!(bead.notes.len(), 3, "{state}");
                    assert_eq!(bead.notes[1].files.len(), 1, "{state}");
                    assert_eq!(bead.notes[1].files[0].hash, IMAGE_HASH, "{state}");
                    for (id, asks) in [("yardr-m4kt", 2), ("plantime-7wq3", 1)] {
                        let bead = yard.client().bead(id).unwrap();
                        assert_eq!(bead.asks.len(), asks, "{state} {id}");
                    }
                    // The decide state serves the open bead's flow for its
                    // head's stage dots: five stages with the bead at
                    // decide, the second. The frame state serves none.
                    if *state == "decide" {
                        let stages = &loaded.flows.depots[0].flows[0].stages;
                        let names: Vec<&str> = stages.iter().map(|s| s.stage.as_str()).collect();
                        assert_eq!(
                            names,
                            ["new", "decide", "review", "review-security", "approved"],
                            "{state}"
                        );
                        assert_eq!(stages[1].beads.len(), 1, "{state}");
                        assert_eq!(stages[1].beads[0].bead, "footplate-q2rd", "{state}");
                    } else {
                        // The frame state serves the shared flows alone,
                        // where the open bead stands in no stage: its head
                        // names the stage alone.
                        let holds = loaded.flows.depots.iter().any(|depot| {
                            depot.flows.iter().any(|flow| {
                                flow.stages.iter().any(|stage| {
                                    stage.beads.iter().any(|b| b.bead == "footplate-q2rd")
                                })
                            })
                        });
                        assert!(!holds, "{state}");
                    }
                }
                "teammates" => {
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    let run = loaded.asks.open[0].run.as_ref().unwrap();
                    assert_eq!(
                        run.line().as_str(),
                        "yardr user invite eve --email eve@example.net",
                        "{state}"
                    );
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 1, "{state}");
                    assert!(bead.asks[0].run.is_some(), "{state}");
                }
                "weighty" => {
                    // The weighty run ask beside the routine one: the
                    // first carries its classes, the second none.
                    assert_eq!(loaded.asks.open.len(), 2, "{state}");
                    assert!(loaded.asks.open[0].run.is_some(), "{state}");
                    assert_eq!(loaded.asks.open[0].weight.len(), 1, "{state}");
                    assert_eq!(loaded.asks.open[0].weight[0].class, "trust", "{state}");
                    assert!(loaded.asks.open[1].run.is_some(), "{state}");
                    assert!(loaded.asks.open[1].weight.is_empty(), "{state}");
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 2, "{state}");
                }
                "weighty-mapping" => {
                    // The weighty mapping ask beside the routine one: the
                    // first maps a sender and carries its class, the second
                    // neither.
                    assert_eq!(loaded.asks.open.len(), 2, "{state}");
                    assert!(loaded.asks.open[0].identity.is_some(), "{state}");
                    assert_eq!(loaded.asks.open[0].weight.len(), 1, "{state}");
                    assert_eq!(loaded.asks.open[0].weight[0].class, "as-user", "{state}");
                    assert!(loaded.asks.open[1].identity.is_none(), "{state}");
                    assert!(loaded.asks.open[1].weight.is_empty(), "{state}");
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 2, "{state}");
                    assert!(bead.asks[0].identity.is_some(), "{state}");
                    assert_eq!(bead.asks[0].options.len(), 4, "{state}");
                    assert!(bead.asks[1].identity.is_none(), "{state}");
                }
                "held-message" => {
                    // The mapping card that holds one mail, and the mail
                    // the yard serves for it to jo alone.
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    let held = loaded.asks.open[0].identity.as_ref().unwrap().held;
                    assert_eq!(held, 1, "{state}");
                    let bodies = yard
                        .client()
                        .identity_held_payloads("bead-s", 7, "jo")
                        .unwrap();
                    assert_eq!(bodies, [held_mail()], "{state}");
                    assert!(
                        yard.client()
                            .identity_held_payloads("bead-s", 7, "ada")
                            .is_err(),
                        "{state}"
                    );
                    let read = crate::model::message(bodies[0].as_bytes());
                    assert_eq!(read.heads[0].1, "Ada Okoye <ada@example.net>", "{state}");
                }
                "delivered-message" | "hook-message" => {
                    // The mail the yard stored when the card was answered,
                    // or told of by its notice, served whole by its seq.
                    let seq = if *state == "hook-message" { 25 } else { 141 };
                    let stored = yard.client().hook_payload(seq).unwrap();
                    assert_eq!(stored, held_mail().as_bytes(), "{state}");
                    if *state == "delivered-message" {
                        assert!(loaded.asks.open.is_empty(), "{state}");
                        let decided = loaded.asks.decided.as_deref().unwrap();
                        assert_eq!(decided[0].delivered, [141], "{state}");
                        assert_eq!(decided[0].option_label, "Let in, once", "{state}");
                    } else {
                        assert!(
                            loaded.notifications.iter().any(|n| n.kind == "hook"),
                            "{state}"
                        );
                    }
                }
                "weighty-trojan" => {
                    // The weighty run ask beside the routine one: the
                    // first carries its class with the Trojan name raw,
                    // as the yard stored it, which the card reads as
                    // letters; the second carries none.
                    assert_eq!(loaded.asks.open.len(), 2, "{state}");
                    assert!(loaded.asks.open[0].run.is_some(), "{state}");
                    assert_eq!(loaded.asks.open[0].weight.len(), 1, "{state}");
                    let weight = &loaded.asks.open[0].weight[0];
                    assert_eq!(weight.class, "code", "{state}");
                    assert!(weight.line.contains('\u{202e}'), "{state}");
                    assert_eq!(
                        crate::model::weight_line(&weight.class, &weight.line),
                        "WEIGHTY · code: the first install of pack trojan into depot north makes roles/07.md[31m x.md",
                        "{state}"
                    );
                    assert!(loaded.asks.open[1].run.is_some(), "{state}");
                    assert!(loaded.asks.open[1].weight.is_empty(), "{state}");
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 2, "{state}");
                }
                "pair-ask" => {
                    // The pairing run ask: its command, and its trust
                    // class.
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    let run = loaded.asks.open[0].run.as_ref().unwrap();
                    assert_eq!(
                        run.line().as_str(),
                        "yardr user device pair jo watch",
                        "{state}"
                    );
                    assert_eq!(loaded.asks.open[0].weight.len(), 1, "{state}");
                    assert_eq!(loaded.asks.open[0].weight[0].class, "trust", "{state}");
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 1, "{state}");
                    assert!(bead.asks[0].run.is_some(), "{state}");
                }
                "pair-qr" | "pair-ended" => {
                    // The pairing ask decided, with what its approval
                    // ran: the screen stands on it.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    assert_eq!(history(&loaded).len(), 1, "{state}");
                    assert!(history(&loaded)[0].run.is_some(), "{state}");
                    assert!(history(&loaded)[0].ran.is_some(), "{state}");
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 1, "{state}");
                    assert!(bead.asks[0].run.is_some(), "{state}");
                }
                "setup-host" => {
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    let setup = loaded.asks.open[0].setup.as_ref().unwrap();
                    assert_eq!(setup.host_template, SETUP_HOST_TEMPLATE, "{state}");
                    assert!(setup.given, "{state}");
                    assert_eq!(setup.does.len(), 7, "{state}");
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 1, "{state}");
                    assert_eq!(bead.asks[0].options.len(), 3, "{state}");
                    assert!(bead.asks[0].setup.is_some(), "{state}");
                }
                "secret" | "secret-shown" | "secret-refused" => {
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    // The card is drawn from the queue's secret, which
                    // carries the yard's four lines of what it reaches.
                    let secret = loaded.asks.open[0]
                        .secret
                        .as_ref()
                        .unwrap_or_else(|| panic!("{state}: no secret"));
                    assert_eq!(secret.reach.len(), 4, "{state}");
                    assert!(secret.reach[0].starts_with("lands in: "), "{state}");
                    assert_eq!(
                        loaded.asks.open[0].concealed,
                        *state != "secret-shown",
                        "{state}"
                    );
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 1, "{state}");
                    assert!(bead.asks[0].secret.is_some(), "{state}");
                    assert_eq!(bead.asks[0].concealed, *state != "secret-shown", "{state}");
                }
                "secret-harness" | "secret-harness-empty" => {
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    // The card is drawn from the queue's secret, which
                    // names the harness and carries no reach: its words of
                    // where the value goes stand directly under its line.
                    let secret = loaded.asks.open[0]
                        .secret
                        .as_ref()
                        .unwrap_or_else(|| panic!("{state}: no secret"));
                    assert_eq!(secret.harness, "claude", "{state}");
                    assert!(secret.login.is_empty(), "{state}");
                    assert!(secret.reach.is_empty(), "{state}");
                    // The empty field says what it takes from the same
                    // secret: the name, and the harness it is the key of,
                    // with no source to name. That sentence is pinned in
                    // window.rs, where the window draws it.
                    assert_eq!(secret.env, ["ANTHROPIC_API_KEY"], "{state}");
                    assert!(secret.source.is_empty(), "{state}");
                    assert!(loaded.asks.open[0].concealed, "{state}");
                    let bead = yard.client().bead("bead-s").unwrap();
                    assert_eq!(bead.asks.len(), 1, "{state}");
                    assert!(bead.asks[0].secret.is_some(), "{state}");
                }
                "page" => {
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    let bead = yard.client().bead("bead-a").unwrap();
                    assert_eq!(bead.notes.len(), 1, "{state}");
                    assert_eq!(bead.notes[0].files.len(), 3, "{state}");
                    assert!(crate::model::is_page(&bead.notes[0].files[0]), "{state}");
                    assert!(crate::model::is_image(&bead.notes[0].files[1]), "{state}");
                    // The third row is a file, named; the yard holds the
                    // image, and the page and the file are named only.
                    assert!(!crate::model::is_page(&bead.notes[0].files[2]), "{state}");
                    assert!(!crate::model::is_image(&bead.notes[0].files[2]), "{state}");
                    assert_eq!(bead.notes[0].files[2].name, "figures.pdf", "{state}");
                    let image = yard.client().blob(IMAGE_HASH).unwrap().unwrap();
                    assert_eq!(image.len(), IMAGE.len(), "{state}");
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 1, "{state}");
                    assert_eq!(letters[0].files.len(), 1, "{state}");
                    assert!(crate::model::is_page(&letters[0].files[0]), "{state}");
                }
                "page-unset" => {
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    let bead = yard.client().bead("bead-a").unwrap();
                    assert_eq!(bead.notes.len(), 1, "{state}");
                    assert_eq!(bead.notes[0].files.len(), 1, "{state}");
                    assert!(crate::model::is_page(&bead.notes[0].files[0]), "{state}");
                }
                "page-unset-many" => {
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    let bead = yard.client().bead("bead-a").unwrap();
                    assert_eq!(bead.notes.len(), 1, "{state}");
                    assert_eq!(bead.notes[0].files.len(), 4, "{state}");
                    assert!(crate::model::is_page(&bead.notes[0].files[0]), "{state}");
                    assert!(crate::model::is_page(&bead.notes[0].files[1]), "{state}");
                    assert!(crate::model::is_page(&bead.notes[0].files[2]), "{state}");
                    // The fourth row is a file, named; the pages and the
                    // file are named only.
                    assert!(!crate::model::is_page(&bead.notes[0].files[3]), "{state}");
                    assert!(!crate::model::is_image(&bead.notes[0].files[3]), "{state}");
                    assert_eq!(bead.notes[0].files[3].name, "figures.pdf", "{state}");
                }
                "thread" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    // Eight mails from alternating senders: the newest
                    // holds the whole-text Markdown sample, above the one
                    // with the image the yard holds.
                    let image = yard.client().blob(IMAGE_HASH).unwrap().unwrap();
                    assert_eq!(image.len(), IMAGE.len(), "{state}");
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 8, "{state}");
                    assert_eq!(letters[1].from, "jo", "{state}");
                    assert_eq!(letters[6].files.len(), 1, "{state}");
                    assert!(crate::model::is_image(&letters[6].files[0]), "{state}");
                    assert!(
                        letters[7].text.contains("| Yard | Waiting | Working |"),
                        "{state}"
                    );
                    assert!(
                        letters[7].text.contains("> Hold the autumn timetable"),
                        "{state}"
                    );
                    assert!(
                        letters[7]
                            .text
                            .contains("[release page](https://yardr.dev/releases)"),
                        "{state}"
                    );
                    assert!(letters[7].text.ends_with("\n---"), "{state}");
                }
                "column" | "column-hover" | "column-copy" | "column-top" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    // Twelve mails over three days, grouped by sender,
                    // with a call link, a calendar link, a correction and
                    // a code block of one line last.
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 12, "{state}");
                    assert!(letters[2].text.contains("teams.microsoft.com"), "{state}");
                    assert!(letters[6].text.contains("calendar.google.com"), "{state}");
                    assert!(letters[4].text.starts_with("correction"), "{state}");
                    assert_eq!(
                        letters[11].text, "```\nyardr review -g box-approvers\n```",
                        "{state}"
                    );
                }
                "older-unread" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    // Ninety of the yardmaster's mails are unread, and
                    // three of the assembly's. The newest page the window
                    // asks for stops short of the oldest of the
                    // yardmaster's, which stands on the page before: what
                    // opening the conversation reads back to.
                    let unread_of = |name: &str| -> Vec<u64> {
                        let from = loaded.unread.iter().filter(|l| l.from == name);
                        from.map(|l| l.seq).collect()
                    };
                    let yardmaster = unread_of("yardmaster");
                    assert_eq!(yardmaster.len(), 90, "{state}");
                    assert_eq!(unread_of("assembly").len(), 3, "{state}");
                    let oldest = *yardmaster.iter().min().unwrap();
                    let client = yard.client();
                    let page = crate::model::PAGE;
                    let newest = client
                        .conversation_page("jo", "yardmaster", page, 0)
                        .unwrap();
                    assert!(newest.more, "{state}");
                    let first = newest.letters[0].seq;
                    assert!(oldest < first, "{state}");
                    let before = client
                        .conversation_page("jo", "yardmaster", page, first)
                        .unwrap();
                    assert!(before.letters.iter().any(|l| l.seq == oldest), "{state}");
                    // The assembly's row holds its three, for a click.
                    let assembly = client.conversation("jo", "assembly", 0).unwrap();
                    assert_eq!(assembly.len(), 3, "{state}");
                }
                "mail" | "image" | "image-menu" | "mail-refused" | "mail-edge" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    // The mail carries the page and the image; the yard
                    // holds the image, and the page is named only. The
                    // refused and edge states carry the page alone.
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 1, "{state}");
                    assert!(crate::model::is_page(&letters[0].files[0]), "{state}");
                    if matches!(*state, "mail-refused" | "mail-edge") {
                        assert_eq!(letters[0].files.len(), 1, "{state}");
                        assert!(yard.client().blob(IMAGE_HASH).unwrap().is_none(), "{state}");
                    } else {
                        let image = yard.client().blob(IMAGE_HASH).unwrap().unwrap();
                        assert_eq!(image.len(), IMAGE.len(), "{state}");
                        assert_eq!(letters[0].files.len(), 2, "{state}");
                        assert!(!crate::model::is_page(&letters[0].files[1]), "{state}");
                    }
                }
                "reactions" | "reactions-hover" | "reactions-zoom" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    // Four mails, the first with one reaction and the
                    // second with both parties', the third with none and
                    // the fourth with one.
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 4, "{state}");
                    assert_eq!(letters[0].reactions.len(), 1, "{state}");
                    assert_eq!(letters[1].reactions.len(), 2, "{state}");
                    assert!(letters[2].reactions.is_empty(), "{state}");
                    assert_eq!(letters[3].reactions.len(), 1, "{state}");
                }
                "reply" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    // Five mails: the yardmaster's two, jo's answer to the
                    // first, the yardmaster's to that, and a last that
                    // answers none.
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    let answered: Vec<u64> = letters.iter().map(|l| l.answered).collect();
                    assert_eq!(answered, [0, 0, 12, 14, 0], "{state}");
                }
                "notice" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    // The yardmaster's six mails, and the flows view that
                    // did not answer, as the notice the window says.
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 6, "{state}");
                    assert!(
                        loaded.flows_notice.as_deref().is_some_and(|notice| {
                            notice.starts_with(
                                "the flows could not be read: the yard does not answer (",
                            )
                        }),
                        "{state}: {:?}",
                        loaded.flows_notice
                    );
                    assert_eq!(loaded.arrivals_notice, None, "{state}");
                }
                "drawer" => {
                    // The two mails, the first naming the bead the
                    // drawer shows, which is read whole for it.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 2, "{state}");
                    assert!(letters[0].text.contains("yd-zulu"), "{state}");
                    let bead = yard.client().bead("yd-zulu").unwrap();
                    assert_eq!(bead.title, "Zulu's late turn", "{state}");
                }
                "refs" => {
                    // The run ask the first mail names, and the two bead
                    // ids the mails name: the ask's own, and yd-zulu's
                    // from the board.
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    assert_eq!(loaded.asks.open[0].ask, 6926, "{state}");
                    assert_eq!(loaded.asks.open[0].bead, "yardr-e9f0", "{state}");
                    assert!(loaded.asks.open[0].run.is_some(), "{state}");
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 3, "{state}");
                    assert!(letters[0].text.contains("ask 6926"), "{state}");
                    assert!(letters[1].text.contains("yd-zulu"), "{state}");
                    assert!(letters[2].text.contains("yardr-e9f0"), "{state}");
                }
                "undelivered" => {
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    let letters = yard.client().conversation("jo", "clerk@north", 0).unwrap();
                    assert_eq!(letters.len(), 2, "{state}");
                    assert_eq!(letters[1].delivery, "failed", "{state}");
                    // The failed message and the telling of it cross past
                    // the head, as the stream tells of them.
                    let mut events = yard
                        .client()
                        .events(20, crate::session::WATCH.quiet)
                        .unwrap();
                    let failed = events.next().unwrap().unwrap();
                    assert_eq!(failed.text("peer"), "north", "{state}");
                    assert_eq!(failed.text("id"), "m1", "{state}");
                    let telling = events.next().unwrap().unwrap();
                    assert_eq!(telling.text("undelivered"), "m1", "{state}");
                    assert_eq!(telling.number("limit"), 30, "{state}");
                    assert!(!telling.text("until").is_empty(), "{state}");
                }
                "waiting" | "waiting-folded" | "waiting-end" | "lines-waiting"
                | "lines-waiting-end" | "lines-panel" | "lines-departures" | "lines-siding" => {
                    // One open ask, and three beads parked for jo's move,
                    // the longest waiting first, with the moves the yard
                    // offers each: the first open on its card, where the
                    // flow places it, and the last one's closing the bead.
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    assert!(!loaded.waiting_too_old, "{state}");
                    let parked: Vec<&str> = loaded.parked.iter().map(|p| p.bead.as_str()).collect();
                    assert_eq!(
                        parked,
                        ["plantime-4f2a", "plantime-9c1e", "plantime-d3a7"],
                        "{state}"
                    );
                    assert_eq!(loaded.parked[0].actions.len(), 2, "{state}");
                    assert!(loaded.parked[0].actions[1].note_required, "{state}");
                    assert!(loaded.parked[2].actions[0].terminal, "{state}");
                    let bead = yard.client().bead("plantime-4f2a").unwrap();
                    assert_eq!(bead.stage, "ready", "{state}");
                    assert_eq!(bead.notes.len(), 1, "{state}");
                    assert_eq!(loaded.flows.depots[0].flows[0].stages.len(), 6, "{state}");
                    // The siding's head says its fold: `1 of 4 to fold`.
                    assert_eq!(
                        loaded.flows.depots[0].flows[0].stages[3].fold,
                        Some(4),
                        "{state}"
                    );
                    // The panel and departures shots draw their own board
                    // over the shared one: the three parked beads, in the
                    // same order.
                    if *state == "lines-panel" || *state == "lines-departures" {
                        let rows: Vec<&str> = loaded
                            .arrivals
                            .rows
                            .iter()
                            .map(|row| row.bead.as_str())
                            .collect();
                        assert_eq!(
                            rows,
                            ["plantime-4f2a", "plantime-9c1e", "plantime-d3a7"],
                            "{state}"
                        );
                    }
                }
                "waiting-many" | "waiting-many-open" => {
                    // No ask, and one bead parked for jo's move, served ten
                    // plain moves as a flow-less bead is: the card folds
                    // them behind the one control until it is unfolded.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    assert!(!loaded.waiting_too_old, "{state}");
                    assert_eq!(loaded.parked.len(), 1, "{state}");
                    let row = &loaded.parked[0];
                    assert_eq!(row.bead, "lead-5gvp", "{state}");
                    assert!(row.depot.is_empty(), "{state}");
                    assert_eq!(row.actions.len(), 10, "{state}");
                    assert!(crate::model::moves_fold(&row.actions), "{state}");
                    let bead = yard.client().bead("lead-5gvp").unwrap();
                    assert_eq!(bead.stage, "backlog", "{state}");
                    assert_eq!(bead.notes.len(), 1, "{state}");
                }
                "old-waiting" => {
                    // The same yard, its yardr refusing the call by name as
                    // one from before the call was a stable one does, with
                    // its own sentence and not no such method: no beads
                    // parked, no notice, and the call too old, so that
                    // Decide says so in the section's place.
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    assert!(loaded.waiting_too_old, "{state}");
                    assert!(loaded.parked.is_empty(), "{state}");
                    assert_eq!(loaded.parked_notice, None, "{state}");
                    let refused = yard.client().waiting("jo").unwrap_err();
                    assert!(
                        refused
                            .to_string()
                            .starts_with("arguments: Waiting takes them as a JSON array"),
                        "{state}: {refused}"
                    );
                    assert!(crate::model::waiting_too_old(&refused), "{state}");
                    // The words the window then draws are the model's, with
                    // none of the yard's.
                    let (model, _) = crate::model::Model::new(
                        loaded.clone(),
                        crate::model::Mode::Decisions,
                        None,
                    );
                    assert!(model.waiting_too_old(), "{state}");
                    assert!(
                        crate::model::WAITING_TOO_OLD.contains("older than the beads-to-move view"),
                        "{state}"
                    );
                }
                "lines-help" => {
                    // The same yard from before the call: no beads parked
                    // and the call too old, with the bead in a stage of
                    // people that no session works, so that its details
                    // say who moves the stage.
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    assert!(loaded.waiting_too_old, "{state}");
                    assert!(loaded.parked.is_empty(), "{state}");
                    let stage = &loaded.flows.depots[0].flows[0].stages[2];
                    assert_eq!(
                        (stage.stage.as_str(), stage.human),
                        ("ready", true),
                        "{state}"
                    );
                    assert!(!stage.beads[0].working, "{state}");
                    assert_eq!(stage.beads[0].bead, "plantime-4f2a", "{state}");
                }
                "decisions" | "decisions-filtered" | "decisions-more" => {
                    // One open ask with four options on its card.
                    assert_eq!(loaded.asks.open.len(), 1, "{state}");
                    let bead = yard.client().bead("bead-a").unwrap();
                    assert_eq!(bead.asks.len(), 1, "{state}");
                    assert_eq!(bead.asks[0].options.len(), 4, "{state}");
                    assert_eq!(bead.asks[0].recommend, 1, "{state}");
                    // Three outcomes asked today, then yesterday's folded
                    // run of four, newest first.
                    let asked: Vec<u64> = history(&loaded).iter().map(|ask| ask.ask).collect();
                    assert_eq!(
                        &asked[..10],
                        &[101, 102, 103, 107, 106, 105, 104, 108, 109, 110],
                        "{state}"
                    );
                    assert_eq!(
                        history(&loaded)[0].question,
                        "Let me edit Google Docs in place",
                        "{state}"
                    );
                    // The history the window draws from it: a Today and a
                    // Yesterday first, with the run folded to one row.
                    let (model, _) = crate::model::Model::new(
                        loaded.clone(),
                        crate::model::Mode::Decisions,
                        None,
                    );
                    let groups = model.decided_groups(&Local::now());
                    assert_eq!(groups[0].day, "Today", "{state}");
                    assert_eq!(groups[1].day, "Yesterday", "{state}");
                    assert!(groups.len() > 2, "{state}");
                    let yesterday = &groups[1].rows[0];
                    match yesterday {
                        crate::model::DecidedEntry::Fold(fold) => {
                            assert_eq!(fold.ask, 107, "{state}");
                            assert_eq!(fold.rows.len(), 4, "{state}");
                        }
                        entry => panic!("the run folds: {entry:?}"),
                    }
                    // A history of more than a page: it draws the first
                    // page, and the rest is left for the more row.
                    assert_eq!(
                        model.decided_left(),
                        156 - crate::model::DECIDED_PAGE,
                        "{state}"
                    );
                }
                "old-lines" => {
                    // The yard serves neither Lines call, as one from
                    // before the Lines view: both halves read as empty,
                    // with no notice, and too old instead. Nor the grants
                    // call, as one from before that view too.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 6, "{state}");
                    assert!(loaded.arrivals.rows.is_empty(), "{state}");
                    assert_eq!(loaded.arrivals_notice, None, "{state}");
                    assert!(loaded.arrivals_too_old, "{state}");
                    assert_eq!(loaded.flows.travelling, 0, "{state}");
                    assert!(loaded.flows.depots.is_empty(), "{state}");
                    assert_eq!(loaded.flows_notice, None, "{state}");
                    assert!(loaded.flows_too_old, "{state}");
                    assert!(loaded.grants.is_empty(), "{state}");
                    assert!(loaded.grants_too_old, "{state}");
                }
                "delivery" => {
                    // Two notices, the second a hook delivery whose files
                    // the yard answers: the photo it holds, and a document
                    // it did not take. No asks, so the window opens on the
                    // yard row alone.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    assert_eq!(loaded.notifications.len(), 2, "{state}");
                    assert_eq!(loaded.notifications[0].kind, "created", "{state}");
                    assert_eq!(loaded.notifications[1].kind, "hook", "{state}");
                    let files = yard
                        .client()
                        .delivery_files(loaded.notifications[1].seq)
                        .unwrap();
                    assert_eq!(files.len(), 2, "{state}");
                    assert!(crate::model::is_image(&files[0]), "{state}");
                    assert!(files[0].not_taken.is_empty(), "{state}");
                    assert!(files[1].hash.is_empty(), "{state}");
                    assert!(
                        files[1].not_taken.contains("over the yard's limit"),
                        "{state}"
                    );
                    let image = yard.client().blob(&files[0].hash).unwrap().unwrap();
                    assert_eq!(image.len(), IMAGE.len(), "{state}");
                }
                "teammates-list" => {
                    // The teammates the view opens on: jo's own operator
                    // row with a paired laptop, hal waiting, and sam
                    // paired with a tablet and a spare never used. No
                    // asks, so decisions opens on the teammates.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    assert_eq!(loaded.operator, "jo", "{state}");
                    assert_eq!(loaded.teammates.len(), 3, "{state}");
                    assert_eq!(loaded.teammates[0].name, "jo", "{state}");
                    assert_eq!(loaded.teammates[0].role, "operator", "{state}");
                    assert_eq!(loaded.teammates[1].name, "hal", "{state}");
                    assert!(loaded.teammates[1].devices.is_empty(), "{state}");
                    assert_eq!(loaded.teammates[2].name, "sam", "{state}");
                    assert_eq!(loaded.teammates[2].devices.len(), 2, "{state}");
                    assert_eq!(loaded.teammates[2].devices[0].name, "tablet", "{state}");
                    assert!(
                        loaded.teammates[2].devices[1].last_used.is_empty(),
                        "{state}"
                    );
                    assert_eq!(loaded.teammates_notice, None, "{state}");
                    assert!(!loaded.teammates_too_old, "{state}");
                }
                "grants" => {
                    // The grants the view opens on: a read, an update as a
                    // user, one whose history was not kept, and two named
                    // calls with their kinds. No asks, so decisions opens
                    // on the grants.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    assert_eq!(loaded.operator, "jo", "{state}");
                    assert_eq!(loaded.grants.len(), 4, "{state}");
                    assert_eq!(loaded.grants[0].source, "google-mail", "{state}");
                    assert_eq!(loaded.grants[0].calls, ["read"], "{state}");
                    assert_eq!(loaded.grants[1].as_user, "ada", "{state}");
                    assert_eq!(loaded.grants[1].calls, ["send"], "{state}");
                    assert!(loaded.grants[2].given.is_empty(), "{state}");
                    assert_eq!(loaded.grants[3].source, "crm", "{state}");
                    assert_eq!(loaded.grants[3].kinds["customer"], "read", "{state}");
                    assert_eq!(loaded.grants[3].kinds["note"], "update", "{state}");
                    assert_eq!(loaded.grants_notice, None, "{state}");
                    assert!(!loaded.grants_too_old, "{state}");
                }
                "pairing" => {
                    // The pairing the pane opens on: jo's iphone with its
                    // code, expiry and QR string. No asks, so decisions
                    // opens on the pairing.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    assert_eq!(loaded.operator, "jo", "{state}");
                    let pairing = loaded.pairing.as_ref().unwrap();
                    assert_eq!(pairing.device, "iphone", "{state}");
                    assert_eq!(pairing.code, "K7QM-2XPD", "{state}");
                    assert_eq!(
                        pairing.qr.as_deref(),
                        Some("https://edge.example.com#K7QM-2XPD#jo/demo"),
                        "{state}"
                    );
                    assert_eq!(loaded.pairing_notice, None, "{state}");
                    assert!(!loaded.pairing_too_old, "{state}");
                }
                "screen" => {
                    // The captured pane the overlay opens with: the call
                    // answers it, and it parses into styled lines.
                    let screen = yard.client().session_screen(SCREEN_OPEN).unwrap();
                    assert!(!screen.screen.is_empty(), "{state}");
                    let parsed = crate::ansi::parse(&screen.screen);
                    assert_eq!(parsed.lines.len(), 51, "{state}");
                    assert!(parsed.width > 0, "{state}");
                }
                "screen-hosted" => {
                    // The yardmaster's six mails, as the window opens
                    // them past the edge: through it the state offers the
                    // screen, and the yard, one from before the screen
                    // came down the link, refuses the card the shot
                    // opens in the words that say it is read on its
                    // machine.
                    assert!(loaded.asks.open.is_empty(), "{state}");
                    let letters = yard.client().conversation("jo", "yardmaster", 0).unwrap();
                    assert_eq!(letters.len(), 6, "{state}");
                    assert_eq!(letters[1].from, "jo", "{state}");
                    let south = Yard::start_tcp();
                    populate(&south, state);
                    let edge = Router::start(&south, crate::fake::SESSION);
                    let hosted = session::load(
                        &edge.client_with(crate::fake::TOKEN, crate::fake::SESSION),
                        "shot",
                    )
                    .unwrap();
                    assert_eq!(hosted.me, "jo", "{state}");
                    assert!(!hosted.screen_on_the_yard, "{state}");
                    let Err(err) = edge
                        .client_with(crate::fake::TOKEN, crate::fake::SESSION)
                        .session_screen(SCREEN_OPEN)
                    else {
                        panic!("{state}: the screen was read");
                    };
                    assert!(
                        matches!(&err, crate::api::Error::Refused { message, .. } if message == SCREEN_OVER_THE_LINK),
                        "{state}: {err:?}"
                    );
                    assert!(crate::model::screen_over_the_link(&err), "{state}");
                }
                "screen-wide" => {
                    // The pane the overlay opens with is 200 columns wide,
                    // more than the window the shots open holds (960
                    // pixels), so its card scrolls sideways.
                    let screen = yard.client().session_screen(SCREEN_OPEN).unwrap();
                    let parsed = crate::ansi::parse(&screen.screen);
                    assert_eq!(parsed.lines.len(), 33, "{state}");
                    assert_eq!(parsed.width, 200, "{state}");
                }
                "screen-refused" => {
                    // The yard is asked, and refuses in its own many words:
                    // not the refusal of the link, which the card answers
                    // with where the screen is read, so the card stands
                    // red and the line wraps.
                    assert!(!loaded.screen_on_the_yard, "{state}");
                    let Err(err) = yard.client().session_screen(SCREEN_OPEN) else {
                        panic!("{state}: the screen was read");
                    };
                    assert!(
                        matches!(&err, crate::api::Error::Refused { message, .. } if message == SCREEN_REFUSAL),
                        "{state}: {err:?}"
                    );
                    assert!(!crate::model::screen_over_the_link(&err), "{state}");
                }
                "line-window" | "line-edit" | "line-sent" | "line-picker" | "line-refused"
                | "line-pack" | "line-stage-edit" | "line-stage-review" | "line-stage-copy"
                | "line-stage-copying" | "lines-served" => {
                    // The window's read of yardr's depot: its flows
                    // named by their source, an exec stage's cmd
                    // whole, a prompt's text and a labelled yard-wide
                    // rule under the first stage's Other rules.
                    let line = yard.client().line("yardr").unwrap();
                    assert_eq!(line.depot, "yardr", "{state}");
                    // The first stage's rule serves builders, with
                    // reviewers past it: the served shot's posed mark
                    // stands under this rule for that group.
                    let route = line.flows[0].stages[0].route.as_ref().unwrap();
                    assert_eq!(route.id, "r-1", "{state}");
                    assert_eq!(route.chain[0].name, "builders", "{state}");
                    assert!(
                        crate::model::line_groups(&line)
                            .iter()
                            .any(|(name, _)| name == "reviewers"),
                        "{state}"
                    );
                    let sources: Vec<&str> =
                        line.flows.iter().map(|flow| flow.source.as_str()).collect();
                    assert_eq!(
                        sources,
                        ["store", "pack starter", "repo@abcdef0", "repo (directory)"],
                        "{state}"
                    );
                    assert_eq!(line.flows[3].directory, "/home/jo/notes", "{state}");
                    let first = &line.flows[0].stages[0];
                    assert_eq!(first.other_routes[0].scope, "yard-wide", "{state}");
                    assert_eq!(first.other_routes[0].filters.labels, ["eyes"], "{state}");
                    assert_eq!(
                        line.flows[0].stages[1].route.as_ref().unwrap().chain[0].cmd,
                        "scripts/merge-local --fast",
                        "{state}"
                    );
                    assert_eq!(
                        first.route.as_ref().unwrap().chain[0]
                            .prompt
                            .as_ref()
                            .unwrap()
                            .text,
                        "Build the thing.\nRun the depot's test command before handing on.\n",
                        "{state}"
                    );
                    // The builders' prompt is a file the yard wrote, which
                    // a card can carry: the editor opens over it.
                    let prompt = first.route.as_ref().unwrap().chain[0]
                        .prompt
                        .as_ref()
                        .unwrap();
                    assert!(prompt.carriable, "{state}");
                    assert_eq!(prompt.digest.len(), 64, "{state}");
                    // The pack's `new` stage file is the yard's copy, saved
                    // over once: the text it replaced stands in the read, so
                    // that Restore prior text… is offered, and the digest a
                    // save would name. The repository's has a file too,
                    // which is not edited here.
                    let managed = line.flows[1].stages[0].stage_file.as_ref().unwrap();
                    assert_eq!(managed.digest.len(), 64, "{state}");
                    assert_eq!(managed.prior_digest.len(), 64, "{state}");
                    assert!(!managed.prior_text.is_empty(), "{state}");
                    assert!(
                        managed.warning.ends_with("the ## Ask section is empty"),
                        "{state}"
                    );
                    let repo = &line.flows[2];
                    assert_eq!(repo.source, "repo@abcdef0", "{state}");
                    assert_eq!(
                        repo.stages[0].stage_file.as_ref().unwrap().path,
                        ".yardr/flows/default/new.md",
                        "{state}"
                    );
                }
                "line-stage-copied"
                | "line-stage-drift"
                | "line-stage-drift-flow"
                | "line-stage-drift-drawer" => {
                    // One store flow, read as the yard answers just after
                    // `flow copy-repo` (no key anywhere) or after its first
                    // stage's file was edited: the key stands on the flow and
                    // on that stage alone, so the section says it of the
                    // stage there and of the flow on `review`, and nothing on
                    // the flow just copied.
                    let line = yard.client().line("yardr").unwrap();
                    assert_eq!(line.flows.len(), 1, "{state}");
                    let flow = &line.flows[0];
                    assert_eq!(flow.source, "store", "{state}");
                    let edited = *state != "line-stage-copied";
                    assert_eq!(flow.differs_from_repo, edited, "{state}");
                    assert_eq!(flow.stages[0].differs_from_repo, edited, "{state}");
                    assert!(!flow.stages[1].differs_from_repo, "{state}");
                    for stage in &flow.stages[..2] {
                        assert_eq!(
                            crate::model::line_drift_words(flow, stage).is_some(),
                            edited,
                            "{state}"
                        );
                    }
                    // Both files are the yard's own, so a card can edit
                    // them; the edited one has the text it was saved over.
                    let new = flow.stages[0].stage_file.as_ref().unwrap();
                    assert_eq!(new.digest.len(), 64, "{state}");
                    assert_eq!(!new.prior_text.is_empty(), edited, "{state}");
                    assert_eq!(
                        flow.stages[1].stage_file.as_ref().unwrap().digest.len(),
                        64,
                        "{state}"
                    );
                }
                _ => unreachable!("a state with no coverage: {state}"),
            }
        }
    }

    /// `SHOT_SIZE` names a size as the Linux script takes it: two positive
    /// whole numbers joined by an `x`. Anything else opens the window at its
    /// usual size.
    #[test]
    fn a_shot_s_window_size_is_two_positive_numbers_joined_by_an_x() {
        assert_eq!(size_from("1200x800"), Some((1200., 800.)));
        assert_eq!(size_from("624x360"), Some((624., 360.)));
        for none in [
            "",
            "1200",
            "x800",
            "1200x",
            "0x800",
            "1200x0",
            "1200x800x2",
            "wide",
            "-1x5",
        ] {
            assert_eq!(size_from(none), None, "{none:?}");
        }
    }

    #[test]
    fn screenshot_yards_use_private_directories_and_remove_them_on_drop() {
        let root = tempfile::tempdir().unwrap();
        let (first_socket, first) = ShotYard::start(root.path(), "plain").unwrap();
        let (second_socket, second) = ShotYard::start(root.path(), "run").unwrap();
        let first_directory = first_socket.parent().unwrap().to_path_buf();
        let second_directory = second_socket.parent().unwrap().to_path_buf();

        assert_ne!(first_directory, second_directory);
        assert_eq!(
            fs::metadata(&first_directory).unwrap().permissions().mode() & 0o077,
            0
        );
        assert_eq!(
            fs::metadata(&second_directory)
                .unwrap()
                .permissions()
                .mode()
                & 0o077,
            0
        );
        assert!(first_socket.exists());
        assert!(second_socket.exists());

        drop(first);
        assert!(!first_directory.exists());
        assert!(second_socket.exists());

        drop(second);
        assert!(!second_directory.exists());
    }

    #[test]
    fn the_switcher_shot_s_held_yards_wait_two_and_one_and_twelve() {
        let adventuretime = Yard::start();
        switcher_adventuretime(&adventuretime);
        let loaded = session::load(&adventuretime.client(), "shot").unwrap();
        assert_eq!(loaded.me, "jo");
        assert_eq!(loaded.unread.len(), 2);
        assert_eq!(loaded.asks.open.len(), 1);
        let plantime = Yard::start();
        switcher_plantime(&plantime);
        let loaded = session::load(&plantime.client(), "shot").unwrap();
        assert_eq!(loaded.me, "jo");
        assert_eq!(loaded.unread.len(), 12);
        assert!(loaded.asks.open.is_empty());
    }

    /// The states that write their own settings open in the look the script
    /// asked for in `SHOT_LOOK`: the Mac's script leaves them no settings
    /// file to carry it, since they refuse one that stands.
    #[test]
    fn the_shots_that_write_their_own_settings_open_in_the_look_asked() {
        use crate::settings::Settings;
        use crate::theme::Look;

        assert_eq!(look_named("light"), Look::Light);
        assert_eq!(look_named("dark"), Look::Dark);
        assert_eq!(look_named(""), Look::Auto);
        assert_eq!(look_named("sepia"), Look::Auto);

        for look in [Look::Light, Look::Dark] {
            let away = tempfile::tempdir().unwrap();
            let file = away.path().join("settings.json");
            keep_away_settings(&file, "https://yard.example.net", look);
            let kept = Settings::read(&file);
            assert_eq!(kept.look, look, "away");
            assert_eq!(kept.opened().name, "far", "away");
            assert!(away.path().join("yard-far.token").is_file());

            // The far yard of the page-unset and edge shots: its web view
            // is unset, or the browser's address at the edge.
            for (state, view) in [
                ("page-unset", ""),
                ("mail-edge", EDGE_VIEW),
                ("settings-connection-edge", EDGE_VIEW),
            ] {
                let far = tempfile::tempdir().unwrap();
                let file = far.path().join("settings.json");
                keep_far_settings(&file, "https://yard.example.net", look, state);
                let kept = Settings::read(&file);
                assert_eq!(kept.look, look, "{state}");
                assert_eq!(kept.opened().name, "far", "{state}");
                assert_eq!(kept.yard("far").unwrap().web_view, view, "{state}");
                assert!(far.path().join("yard-far.token").is_file(), "{state}");
            }

            for state in ["signedout", "screen-hosted"] {
                let hosted = tempfile::tempdir().unwrap();
                let file = hosted.path().join("settings.json");
                keep_hosted_settings(&file, "https://edge.example.net", look, state);
                let kept = Settings::read(&file);
                assert_eq!(kept.look, look, "{state}");
                assert_eq!(kept.opened().name, "south", "{state}");
                assert!(hosted.path().join("yard-south.token").is_file(), "{state}");
                assert!(
                    hosted.path().join("yard-south.session").is_file(),
                    "{state}"
                );
            }
        }
    }

    /// The shot scripts take the states served here from `STATES` and keep
    /// no list of their own: scripts/shot-states.sh reads each name of it,
    /// so that a state is refused by neither script once it is added here,
    /// as `weighty-mapping` was on the Mac while each kept a copy. A line of
    /// the list that the read does not take, a trailing comment on it for
    /// one, leaves its state out and fails this.
    #[test]
    fn the_shot_scripts_read_the_states_served_here() {
        let read = std::process::Command::new("sh")
            .args(["-c", r#". scripts/shot-states.sh; printf %s "$states""#])
            .env("shot_rs", "src/shot.rs")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("sh runs");
        assert!(
            read.status.success(),
            "{}",
            String::from_utf8_lossy(&read.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&read.stdout), STATES.join(" "));

        for (name, script) in [
            ("shot-linux.sh", include_str!("../scripts/shot-linux.sh")),
            ("shot-mac.sh", include_str!("../scripts/shot-mac.sh")),
        ] {
            assert!(
                !script.lines().any(|line| line.starts_with("states=\"")),
                "{name} keeps a list of states of its own"
            );
        }
    }

    /// The voice shot's stand-in makes a session as the window starts one,
    /// with the key the window gives it and a moment that is not past, and
    /// the person's words come once it stands: before it, the caption would
    /// have nothing to show; past, the session would end as it began. They
    /// are two asks, a pause between, which the caption holds together.
    #[test]
    fn the_voice_shot_stands_a_session_and_the_person_asks_their_questions() {
        let talking = stand_voice();
        let (tell, updates) = std::sync::mpsc::channel();
        let crate::voice::Start::On(voice) = crate::voice::gated_start(
            true,
            || Ok(VOICE_KEY.to_string()),
            &talking.url,
            "yardmaster",
            "",
            tell,
        ) else {
            panic!("the session stands");
        };
        let next = || {
            updates
                .recv_timeout(std::time::Duration::from_secs(5))
                .expect("an update in five seconds")
        };
        assert_eq!(next(), crate::voice::Update::Started);
        let mut caption = crate::voice::Caption::default();
        for ask in ["What's blocking the release", "And who owns it"] {
            // Each ask begins an utterance, which the session tells of too.
            let fragment = loop {
                match next() {
                    crate::voice::Update::SpeechStarted => {}
                    crate::voice::Update::Words(fragment) => break fragment,
                    other => panic!("the person's words were expected, not {other:?}"),
                }
            };
            assert_eq!(fragment.text, ask);
            caption.heard(fragment);
        }
        assert_eq!(
            caption.text(),
            "What's blocking the release And who owns it"
        );
        drop(voice);
    }
}
