//! What an ask's card says and takes: its heads and counts, its keys,
//! its buttons, and the texts a run or a setup holds.

use gpui::{
    App, ClickEvent, Context, Div, FontWeight, SharedString, Stateful, Window, div, prelude::*, px,
    rems,
};

use super::metrics::RADIUS_CONTROL;
use super::words::plural;
use super::{APPROVE_KEY, Footplate, small_button};
use crate::api;
use crate::model::{self, DecidedDetail, DecisionAsk};
use crate::select;
use crate::theme::{Palette, TYPE_LABEL, TYPE_META, TYPE_SECTION};

/// A section's head in the left list: its title on the left, its count
/// on the right, quiet. "Decisions" counts the open ones, "Crew" the
/// mail that is not read. At least the section's height, and taller where
/// the text wants it.
pub(crate) fn section_head(
    title: &'static str,
    count: String,
    palette: Palette,
) -> impl IntoElement {
    div()
        .min_h(px(32.))
        .px_3()
        .pt_2()
        .pb_1()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .child(head_title(title, title.to_string()))
        .child(head_count(title, count, palette))
}

/// A section's head that folds its rows: [`section_head`]'s title and
/// count with ▾ or ▸ before the title, in the shape of the Decided toggle,
/// with a hover and a pointer to say it takes a press, which the caller
/// gives it. The margin and padding come to the plain head's offsets, so
/// the title stands where it would without the fold.
pub(crate) fn fold_head(
    id: &'static str,
    title: &'static str,
    count: String,
    open: bool,
    palette: Palette,
) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || format!("{id} toggle"))
        .mx_1()
        .mt_1()
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .hover(|style| style.bg(palette.hover))
        .child(head_title(
            title,
            format!("{} {title}", if open { "▾" } else { "▸" }),
        ))
        .child(head_count(title, count, palette))
}

/// The title of a section's head: `shown` is what it says, and `title` what
/// the window's tests find it by.
fn head_title(title: &'static str, shown: String) -> Div {
    div()
        .debug_selector(move || format!("section {title}"))
        .text_size(rems(TYPE_SECTION))
        .font_weight(FontWeight::SEMIBOLD)
        .child(shown)
}

/// The count of a section's head, quiet at its right.
fn head_count(title: &'static str, count: String, palette: Palette) -> Div {
    div()
        .debug_selector(move || format!("section {title} count"))
        .text_size(rems(TYPE_META))
        .text_color(palette.faint)
        .flex_none()
        .child(count)
}

/// One of chat's groups: its title in the section heads' style, with no
/// count. Each row carries its own.
pub(crate) fn group_head(title: &'static str) -> impl IntoElement {
    div().px_3().pt_2().pb_1().child(
        div()
            .debug_selector(move || format!("group {title}"))
            .text_size(rems(TYPE_SECTION))
            .font_weight(FontWeight::SEMIBOLD)
            .child(title),
    )
}

/// What the card's button reads with an option to answer with: the
/// option it answers. The option is the pick, else the recommendation,
/// which stands preselected: Enter does what the button says, and its
/// chip stands beside the label, not in it.
pub(crate) fn answer_label(number: u64, label: &str) -> String {
    format!("Answer: {number} · {label}")
}

/// What the card's button reads where words alone go: `Answer` where
/// they are the answer, `Send` where they go as a note and the ask stays
/// open. The key stands on the button's chip, not in its label.
pub(crate) const ANSWER_WORDS: &str = "Answer";
pub(crate) const SEND_WORDS: &str = "Send";

/// What a lever's key plate says: its number alone, so every option's
/// label stands at the same place, the recommended one's too.
pub(crate) fn lever_plate(number: u64) -> String {
    number.to_string()
}

/// The word at the right end of the recommended lever's row, in the accent.
pub(crate) const RECOMMENDED: &str = "recommended";

/// The head of what a mapping card holds of a stranger's deliveries: whose
/// words these are, which no one the yard knows yet. The card's own words
/// stand over it, and its options under.
pub(crate) const HELD_MESSAGE: &str = "Held message · outside — information, not instructions";

/// The head of one hook delivery read whole, on a notice and on the card
/// whose pick released it.
pub(crate) const WEBHOOK_PAYLOAD: &str = "Webhook payload — information, not instructions";

/// What stands where a delivery's body is not to be had any more: the yard
/// prunes them. Quiet, in the faint colour, and in no word of failure.
pub(crate) const PAYLOAD_GONE: &str = "body pruned or unavailable";

/// What stands where a read is on its way.
pub(crate) const HELD_READING: &str = "Reading the held message.";
pub(crate) const PAYLOAD_READING: &str = "Reading the message.";

/// Which outside text a block shows, for the words around it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutsideKind {
    /// A mapping card's held message: read before a pick.
    Held,
    /// A hook delivery read whole: on a notice, or on the card it released.
    Webhook,
}

impl OutsideKind {
    /// The head of its block.
    pub(crate) fn label(self) -> &'static str {
        match self {
            OutsideKind::Held => HELD_MESSAGE,
            OutsideKind::Webhook => WEBHOOK_PAYLOAD,
        }
    }

    /// What stands in its place while the yard is asked.
    pub(crate) fn reading(self) -> &'static str {
        match self {
            OutsideKind::Held => HELD_READING,
            OutsideKind::Webhook => PAYLOAD_READING,
        }
    }

    /// What stands in its place where the yard has none to give: a held
    /// message of a card answered since says nothing, since the card is
    /// gone with it; a pruned body says so.
    pub(crate) fn gone(self) -> Option<&'static str> {
        match self {
            OutsideKind::Held => None,
            OutsideKind::Webhook => Some(PAYLOAD_GONE),
        }
    }
}

/// What the control that reads a delivery whole says: on a notice, `Read
/// whole`; on a card whose pick released one or several, `Read message`
/// and `Read 3 messages`. Shown, it says `Hide`.
pub(crate) fn read_label(notice: bool, n: usize, shown: bool) -> String {
    match (shown, notice, n) {
        (true, _, _) => "Hide".to_string(),
        (false, true, _) => "Read whole".to_string(),
        (false, false, 1) => "Read message".to_string(),
        (false, false, n) => format!("Read {n} messages"),
    }
}

/// What a fold's control says: how much it left out, or that it is open.
pub(crate) fn fold_label(left: usize, open: bool) -> String {
    match (open, left) {
        (true, _) => "Fold".to_string(),
        (false, 1) => "Show all · 1 more line".to_string(),
        (false, left) => format!("Show all · {left} more lines"),
    }
}

/// What a lever says beside its key plate: the yard's words for the
/// choice and its consequence. The number lives on the plate alone.
pub(crate) fn lever_words(label: &str, consequence: &str) -> (String, String) {
    (label.to_string(), format!("— {consequence}"))
}

/// The recommended option's mark under its own row, with its why: the one
/// place the card names the recommendation.
pub(crate) fn recommended_line(why: &str) -> String {
    match why.is_empty() {
        true => "recommended".to_string(),
        false => format!("recommended: {why}"),
    }
}

/// How long the ask waits, beside its question: `asked 3d`, `asked now`.
/// Nothing where the yard's time cannot be read.
pub(crate) fn asked_age(waits: &str) -> String {
    match waits.is_empty() {
        true => String::new(),
        false => format!("asked {waits}"),
    }
}

/// What an inline ask's preview says over its question: that it waits on
/// the person reading, which ask, and how long it waits.
pub(crate) fn ask_preview_head(kind: &str, ask: u64, waits: &str) -> String {
    [
        "Waiting on you".to_string(),
        format!("{kind} {ask}"),
        asked_age(waits),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" · ")
}

/// What the line on an open card says, beside its disclosure: the bead's id
/// and the ask's number, the two that `yardr bead answer <bead> <ask> <n>`
/// takes. The number is the yard's, from the one counter of every note, so
/// a decision named by it in mail, chat or a terminal is found by it here.
pub(crate) fn ask_line(bead: &str, ask: u64) -> String {
    format!("{bead} · ask {ask}")
}

/// The head of a bead's open asks: the yard and the bead, once for
/// every ask of the bead, then the topic where the bead has one (never
/// the "Everything else" the list groups the topic-less under), and who
/// built and reviewed it where either is known. The yard's dot stands
/// before it, in its colour. No card is read without its yard.
pub(crate) fn decision_head(
    yard: &str,
    bead: &str,
    topic: &str,
    built_by: &str,
    reviewed_by: &str,
) -> String {
    let mut parts = vec![yard.to_string(), bead.to_string()];
    if !topic.is_empty() && topic != model::EVERYTHING_ELSE {
        parts.push(topic.to_string());
    }
    if !built_by.is_empty() {
        parts.push(format!("built by {built_by}"));
    }
    if !reviewed_by.is_empty() {
        parts.push(format!("reviewed by {reviewed_by}"));
    }
    parts.join(" · ")
}

/// What the head's pill counts: the bead's open asks that wait on the
/// person reading. The head stands only where at least one does.
pub(crate) fn asks_on_you(count: usize) -> String {
    match count {
        1 => "1 ask on you".to_string(),
        count => format!("{count} asks on you"),
    }
}

/// What Decline's tooltip says: saying no to every option, so the
/// asker finds another way.
pub(crate) const DECLINE_IS_NO_PICK: &str = "Say no to all options. The asker finds another way.";

/// What the settings say over the list of yards: what a pairing gives that
/// a link between two yards does not.
pub(crate) const PAIRING_GIVES: &str = "A paired yard is called with this device's own token, so your picks and approvals there count as yours, where over a peer link you only see what the link carries and an approval never crosses.";

/// What a secret ask's card says under the yard's line of what is asked
/// for: where the value goes, in the words `yardr chat` has for it. The
/// words are the yard's, by the kind of secret (`cardLines` in yardr's
/// `internal/store/secretreach.go`), and are copied as they stand: a card
/// that gave a harness key or a login's input a file's words would say
/// what the yard does not. These are a source's credential's, and any ask
/// whose kind the yard did not say gets them.
pub(crate) const SECRET_GOES: &str =
    "it goes to a file only the yard's server reads; no agent is shown it";

/// A harness's API key is read by every session of its kind that burns the
/// operator's login, which the yard's line above it names, so its card must
/// not say that no agent is shown it.
pub(crate) const HARNESS_GOES: &str = "the value is kept out of notes, events and logs; each such session receives it in its environment";

/// A waiting login's input is handed to the login in the yard's memory, and
/// kept in no file and no note.
pub(crate) const LOGIN_GOES: &str =
    "it goes to the waiting login only; it is kept nowhere, and no agent is shown it";

/// Which words a secret ask's card says where its value goes in, by its
/// kind, in the order the yard tests it: a harness key's, then a waiting
/// login's, then a file's.
pub(crate) fn secret_goes(secret: &api::Secret) -> &'static str {
    if !secret.harness.is_empty() {
        HARNESS_GOES
    } else if !secret.login.is_empty() {
        LOGIN_GOES
    } else {
        SECRET_GOES
    }
}

/// The keys of a secret ask's card, with the way for a person without the
/// field: the same answer in a terminal on the yard's machine. Where the
/// Mac's unlock stands in front, Enter opens it first; where the card
/// takes its number back, `i` types the number before the value.
pub(crate) fn secret_keys(bead: &str, ask: u64, approval: Approval, writable: bool) -> String {
    let keys = match approval {
        Approval::Direct => format!(
            "i types it · enter sends it to the yard · esc leaves, and nothing is sent · or in a terminal: yardr bead answer {bead} {ask} --secret"
        ),
        Approval::Unlock => format!(
            "i types it · enter sends it to the yard after unlocking · esc leaves, and nothing is sent · or in a terminal: yardr bead answer {bead} {ask} --secret"
        ),
        Approval::TypeBack => format!(
            "i types the number, then it · enter sends it to the yard with its number · esc leaves, and nothing is sent · or in a terminal: yardr bead answer {bead} {ask} --secret"
        ),
        Approval::Nowhere => format!("in a terminal: yardr bead answer {bead} {ask} --secret"),
    };
    if writable {
        format!("{keys} · a ask about this")
    } else {
        keys
    }
}

/// What a run ask's card says over the command, while it can be approved:
/// the words `yardr chat` has for it, led by the yard the command runs in
/// and with the user it runs as, who is the one the approval is called as.
/// With several yards in one window, the sentence over a command is where
/// it must be plain which of them runs it.
pub(crate) fn run_runs(yard: &str, user: &str) -> String {
    format!("in {yard}, when you approve, the yard runs this as {user}, and nothing else:")
}

/// The header of a card: the yard it is of first, then the bead and how
/// long it waits or which ask it is. No card is read without its yard.
/// The yard's dot stands before it, in its colour.
pub(crate) fn card_head(yard: &str, bead: &str, rest: &str) -> String {
    format!("{yard} · {bead} · {rest}")
}

/// What the control that approves a run ask reads: the yard its command
/// runs in, and its key.
pub(crate) fn approve_label(yard: &str) -> String {
    format!("Approve in {yard} ({APPROVE_KEY})")
}

/// What the control that approves a weighty run or setup ask reads where
/// the Mac's unlock stands in front of it: the yard, and that unlocking
/// comes first.
pub(crate) fn approve_unlock_label(yard: &str) -> String {
    format!("Approve in {yard} after unlocking")
}

/// How the card's approval goes: straight to the yard, through the Mac's
/// unlock first, after the ask's number is typed back where no unlock is
/// offered, or nowhere from this window, where the card shows where to
/// approve instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Approval {
    Direct,
    Unlock,
    TypeBack,
    Nowhere,
}

/// How the ask's card approves: nowhere where the yard takes no approval
/// from this window; through the Mac's unlock where a weighty card has
/// one; after the ask's number is typed back where it has none, as a
/// terminal asks it back; straight to the yard otherwise.
pub(crate) fn approval(weighty: bool, gate: bool, elsewhere: bool) -> Approval {
    match (weighty, gate, elsewhere) {
        (_, _, true) => Approval::Nowhere,
        (true, true, false) => Approval::Unlock,
        (true, false, false) => Approval::TypeBack,
        (false, _, false) => Approval::Direct,
    }
}

/// What a run ask's card says over what the command changes: the
/// command's one line from its help, as a terminal says it.
pub(crate) fn description_line(description: &str) -> String {
    format!("what the command does: {description}")
}

/// Who may run the command that takes the change back, as a terminal
/// says it: the yard's words, or anyone where it sends none.
pub(crate) fn take_back_label(back_by: &str) -> String {
    match back_by.is_empty() {
        true => "anyone can take it back:".to_string(),
        false => format!("{back_by} can take it back:"),
    }
}

/// What it says over the command of one that takes no approval any more.
pub(crate) const RUN_PROPOSED: &str = "the command this ask proposed:";

/// What a setup ask's card says over what approving does, while it can be
/// approved: the words `yardr chat` has for it, led by the yard the setup
/// runs in. With several yards in one window, the sentence over a setup
/// is where it must be plain which of them runs it.
pub(crate) fn setup_runs(yard: &str) -> String {
    format!(
        "in {yard}, when you approve, the yard applies the pack, takes the account, and grants the preset you pick, and nothing else:"
    )
}

/// What it says over the setup of one that takes no approval any more.
pub(crate) const SETUP_PROPOSED: &str = "the setup this ask proposed:";

/// What a setup ask's card says over its candidates: a tap takes one, and
/// a filled secret field types another.
pub(crate) const SETUP_TAKE: &str = "take one with a tap, or type another below:";

/// What a run ask's card says in place of the control that approves, where
/// the yard does not have the call or does not serve it to this window.
pub(crate) const APPROVE_ELSEWHERE: &str =
    "this yard takes no approval from footplate. Approve in a terminal on the yard's machine:";

/// What a weighty card says over its number field where this window has
/// no gate: what to type and why, in the terminal's words. `verb` is what
/// the number lets through: approving a run or setup ask, giving a secret.
pub(crate) fn typeback_line(ask: u64, verb: &str) -> String {
    format!("type the ask's number, {ask}, to {verb} this weighty card")
}

/// What the window says where the number field holds anything but the
/// ask's number: nothing went, and the card stays as it is, in the
/// terminal's words.
pub(crate) fn typeback_mismatch(ask: u64) -> String {
    format!("ask {ask}'s number was not typed back, so nothing was done")
}

/// The keys of a run ask's card, beside what its buttons say: the
/// approval and ask-about-this stand on their buttons' chips, so the
/// line names only what no button does. Enter is named because it does
/// nothing here; where the card takes its number back, `i` types it.
pub(crate) fn run_keys(approval: Approval) -> String {
    let mut keys: Vec<String> = Vec::new();
    if !matches!(approval, Approval::Nowhere) {
        keys.push("enter alone approves nothing".to_string());
    }
    if matches!(approval, Approval::TypeBack) {
        keys.push("i types the number".to_string());
    }
    keys.join(" · ")
}

/// The keys of a setup ask's card, beside what its buttons say: the
/// approval and ask-about-this stand on their buttons' chips, so the
/// line names only the field that types the account and the same answer
/// in a terminal on the yard's machine. Where the yard takes no approval
/// from this window, the card says where it does in place of the
/// control, and lists no keys.
pub(crate) fn setup_keys(bead: &str, ask: u64, setup: &api::Setup, approval: Approval) -> String {
    let mut keys: Vec<String> = Vec::new();
    if !matches!(approval, Approval::Nowhere) {
        keys.push("enter alone approves nothing".to_string());
        match approval {
            Approval::TypeBack => keys.push("i types the number, then the account".to_string()),
            _ => keys.push("i types the account".to_string()),
        }
        keys.push(format!(
            "or in a terminal: {}",
            model::setup_approve_command(bead, ask, setup)
        ));
    }
    keys.join(" · ")
}

/// A command as a text that is drawn as one code block: whole, wrapped
/// where the window is narrower and never cut, with the control that copies
/// it exactly. The fence is longer than any run of backticks in the
/// command, so nothing in it ends the block.
pub(crate) fn fenced(command: &str) -> String {
    let (mut longest, mut run) = (0, 0);
    for c in command.chars() {
        run = if c == '`' { run + 1 } else { 0 };
        longest = longest.max(run);
    }
    let fence = "`".repeat((longest + 1).max(3));
    format!("{fence}\n{command}\n{fence}")
}

/// One text a run ask holds behind the tap: its heading, its bytes, and
/// the section its copy control is keyed under.
pub(crate) struct RunText {
    pub(crate) heading: String,
    pub(crate) text: String,
    pub(crate) section: &'static str,
}

/// Every text carried sources hold, in the order a terminal prints them:
/// each declaration, command files, file changes and removed entries'
/// texts. A run ask and a setup ask hold the same: the headings are the
/// terminal's words, so the readings agree. An empty text is no text: a
/// removed entry that is not text stands in the yard's sentences alone.
pub(crate) fn carried_texts(carried: &[api::RunSource]) -> Vec<RunText> {
    let mut texts = Vec::new();
    for carried in carried {
        texts.push(RunText {
            heading: format!(
                "the declaration of source {}, whole as carried:",
                model::plain(&carried.name)
            ),
            text: carried.text.clone(),
            section: "declaration",
        });
        for file in &carried.files {
            texts.push(RunText {
                heading: format!(
                    "the command file {} (sha256 {}), whole as carried:",
                    model::plain(&file.program),
                    model::plain(&file.sha256),
                ),
                text: file.text.clone(),
                section: "command file",
            });
        }
        for change in &carried.old {
            texts.push(RunText {
                heading: format!(
                    "the change to command file {} (sha256 {} -> {}), whole as carried:",
                    model::plain(&change.program),
                    or_none(&change.old_sha256),
                    or_none(&change.new_sha256),
                ),
                text: change.diff.clone(),
                section: "file change",
            });
        }
        for removed in carried.removed.iter().filter(|r| !r.text.is_empty()) {
            texts.push(RunText {
                heading: format!(
                    "the entry {} the copy would remove (sha256 {}), whole as carried:",
                    model::plain(&removed.path),
                    model::plain(&removed.sha256),
                ),
                text: removed.text.clone(),
                section: "removed entry",
            });
        }
    }
    texts
}

/// Every text a run ask holds, in the order a terminal prints them: its
/// carried sources' texts, then each plan file's change and new text. A
/// plan object with neither a change nor a new text stands in the yard's
/// sentences alone.
pub(crate) fn run_texts(run: &api::Run) -> Vec<RunText> {
    let mut texts = carried_texts(&run.carried);
    if let Some(pack) = &run.pack
        && let Some(plan) = &pack.plan
    {
        for object in &plan.objects {
            if !object.diff.is_empty() {
                texts.push(RunText {
                    heading: format!(
                        "the change to file {} (sha256 {} -> {}), whole as carried:",
                        model::plain(&object.name),
                        or_none(&object.file_old_sha256),
                        or_none(&object.file_new_sha256),
                    ),
                    text: object.diff.clone(),
                    section: "plan change",
                });
            }
            if !object.text.is_empty() {
                texts.push(RunText {
                    heading: format!(
                        "the file {} as the pack would install it, whole as carried:",
                        model::plain(&object.name),
                    ),
                    text: object.text.clone(),
                    section: "plan file",
                });
            }
        }
    }
    texts
}

/// How many rows each of a pack ask's plan sections shows before the
/// rest stand behind `Show K more`.
pub(crate) const PLAN_ROWS_SHOWN: usize = 3;

/// The heads of a pack ask's plan sections: what the import would move,
/// what differs and stays as the yard has it, and what already matches.
pub(crate) const PLAN_CHANGES: &str = "What changes";
pub(crate) const PLAN_DIFFERS: &str = "The yard's own version stays";
pub(crate) const PLAN_MATCHES: &str = "Already matches";

/// One row of a pack ask's plan as the card draws it: the kind of thing
/// it is about in the yard's word for it, the thing's name, and what is
/// said of it, a line each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlanRow {
    pub(crate) kind: String,
    pub(crate) subject: String,
    pub(crate) lines: Vec<String>,
}

/// The plan's rows in its three sections, each in the plan's order, the
/// objects before the flows the depot's beads follow: what the import
/// would move (an object it creates, changes or finds dropped from the
/// pack, and a depot whose beads would follow another flow), what
/// differs and stays as the yard has it (`differs`), and what already
/// matches the yard (an object unchanged, kept as the yard has it on an
/// older yard, or taken by a group of the yard, and a depot whose beads
/// already follow the flow). An object of the yard that the pack does
/// not carry (`not_in_pack`, from an older yard) stands in none, as
/// does an action this window has no word for: the yard's sentences say
/// them. `depot` names the depot a flow of no type is the flow of.
pub(crate) fn plan_rows(
    plan: &api::RunPlan,
    depot: &str,
) -> (Vec<PlanRow>, Vec<PlanRow>, Vec<PlanRow>) {
    let mut moves = Vec::new();
    let mut differs = Vec::new();
    let mut matches = Vec::new();
    for object in &plan.objects {
        let row = |lines| PlanRow {
            kind: kind_word(&object.kind),
            subject: model::plain(&object.name),
            lines,
        };
        match object.action.as_str() {
            "create" | "change" | "dropped" => moves.push(row(change_lines(object))),
            "differs" => differs.push(row(differs_lines(object))),
            "unchanged" | "kept" | "takes" => matches.push(row(keep_lines(object))),
            _ => {}
        }
    }
    for govern in &plan.governs {
        let flow = model::plain(&govern.flow);
        let now = model::plain(&govern.now);
        let typed = model::plain(&govern.flow_type);
        let subject = match typed.is_empty() {
            true => own_depot(depot),
            false => typed.clone(),
        };
        let (rows, line) = match (now == flow, typed.is_empty(), now.is_empty()) {
            (true, true, _) => (&mut matches, format!("already follows {flow}")),
            (true, false, _) => (
                &mut matches,
                format!("beads of type {typed} already follow {flow}"),
            ),
            (false, _, true) => (&mut moves, format!("follows {flow}")),
            (false, _, false) => (&mut moves, format!("follows {flow}, was {now}")),
        };
        rows.push(PlanRow {
            kind: "depot".to_string(),
            subject,
            lines: vec![line],
        });
    }
    (moves, differs, matches)
}

/// The word a plan row's chip says: the object's kind as the yard names
/// it, and a stage map as the hook it makes.
fn kind_word(kind: &str) -> String {
    match kind {
        "map" => "hook".to_string(),
        kind => model::plain(kind),
    }
}

/// Who a flow of no type is the flow of: the depot, by its name where the
/// pack ask names one.
fn own_depot(depot: &str) -> String {
    match depot.is_empty() {
        true => "the depot".to_string(),
        false => model::plain(depot),
    }
}

/// What a row of what the import would move says: each field it changes
/// (`field: old → new`, and `+ new` where there was nothing before and
/// `− old` where nothing is left), and where an object comes with no
/// field, that it is added (`+ name`) or no longer in the pack.
fn change_lines(object: &api::RunPlanObject) -> Vec<String> {
    let lines: Vec<String> = object
        .changes
        .iter()
        .map(|change| {
            let (field, old, new) = (
                model::plain(&change.field),
                model::plain(&change.old),
                model::plain(&change.new),
            );
            match (old.is_empty(), new.is_empty()) {
                (false, false) => format!("{field}: {old} → {new}"),
                (true, false) => format!("+ {new}"),
                (false, true) => format!("− {old}"),
                (true, true) => field,
            }
        })
        .filter(|line| !line.is_empty())
        .collect();
    match (lines.is_empty(), object.action.as_str()) {
        (true, "create") => vec![format!("+ {}", model::plain(&object.name))],
        (true, "dropped") => vec!["no longer in the pack".to_string()],
        _ => lines,
    }
}

/// What a row of what differs says: each differing field as
/// `<field>: yard <old>, line <new>` (or the side present where one is
/// empty). A `differs` object with no fields has no lines, so the row
/// shows kind and name alone.
fn differs_lines(object: &api::RunPlanObject) -> Vec<String> {
    object
        .changes
        .iter()
        .map(|change| {
            let (field, old, new) = (
                model::plain(&change.field),
                model::plain(&change.old),
                model::plain(&change.new),
            );
            match (old.is_empty(), new.is_empty()) {
                (false, false) => format!("{field}: yard {old}, line {new}"),
                (true, false) => format!("{field}: line {new}"),
                (false, true) => format!("{field}: yard {old}"),
                (true, true) => field,
            }
        })
        .filter(|line| !line.is_empty())
        .collect()
}

/// What a row of what already matches says: what the import leaves as
/// the yard has it though the pack carries it, and why, a line each.
fn keep_lines(object: &api::RunPlanObject) -> Vec<String> {
    object
        .keeps
        .iter()
        .map(|keep| model::plain(keep))
        .filter(|keep| !keep.is_empty())
        .collect()
}

/// A row line's mark and what follows it: `+` before what is put in and
/// `−` before what is taken out, which the card sets apart in their own
/// colours, and the line as it is for any other.
pub(crate) fn plan_mark(line: &str) -> (Option<char>, &str) {
    for mark in ['+', '−'] {
        if let Some(rest) = line
            .strip_prefix(mark)
            .and_then(|rest| rest.strip_prefix(' '))
        {
            return (Some(mark), rest);
        }
    }
    (None, line)
}

/// What the plan comes to, in one line over its sections: how many
/// changes across which kinds, how many differ and stay as the yard has
/// them, and how many more already match the yard. A plan whose objects
/// are all `unchanged` or `differs` changes nothing and says
/// `Nothing changes.`.
pub(crate) fn plan_summary(moves: &[PlanRow], differing: usize, matching: usize) -> String {
    let (mut line, more) = match moves.len() {
        0 => ("Nothing changes.".to_string(), ""),
        n => (
            format!(
                "{} across {}.",
                plural(n, "", "change", "changes"),
                plan_kinds(moves)
            ),
            " more",
        ),
    };
    match differing {
        0 => {}
        1 => line.push_str(" 1 differs and stays as the yard has it."),
        n => line.push_str(&format!(" {n} differ and stay as the yard has them.")),
    }
    match matching {
        0 => line,
        1 => format!("{line} 1{more} already matches the yard and will not move."),
        n => format!("{line} {n}{more} already match the yard and will not move."),
    }
}

/// The kinds of the rows with how many of each, in the order they first
/// stand and as a sentence says them (`1 flow, 1 hook and the crew`): the
/// crew is one body of members, so it has no count.
fn plan_kinds(rows: &[PlanRow]) -> String {
    let mut kinds: Vec<(&str, usize)> = Vec::new();
    for row in rows {
        match kinds.iter_mut().find(|(kind, _)| *kind == row.kind) {
            Some((_, n)) => *n += 1,
            None => kinds.push((&row.kind, 1)),
        }
    }
    let words: Vec<String> = kinds
        .into_iter()
        .map(|(kind, n)| match kind {
            "crew" => "the crew".to_string(),
            kind => plural(n, "", kind, &format!("{kind}s")),
        })
        .collect();
    match words.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    }
}

/// What stands at the right of the head of what already matches: how
/// many, and whose they are.
pub(crate) fn plan_matches_count(matching: usize) -> String {
    format!("{matching} · kept as the yard's")
}

/// What a plan section's control says: how many rows it left out, or that
/// it is open.
pub(crate) fn plan_more_label(left: usize, open: bool) -> String {
    match open {
        true => "Fold".to_string(),
        false => format!("Show {left} more"),
    }
}

/// A teammate invite or removal a run ask proposes, parsed from the
/// command it was given: `user invite <name> --email <address>
/// [--role operator]` or `user uninvite <name>`. The yard parsed the
/// words when the ask was written, and an approval runs them exactly as
/// shown, so the card reads its own fields back from them. Anything
/// else is no teammate ask, and keeps the generic run card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Teammate {
    Invite {
        name: String,
        email: String,
        role: String,
    },
    Uninvite {
        name: String,
    },
}

/// The teammate ask the run proposes, when it proposes one. The role is
/// `member` where the command names none: the yard stores an explicit
/// member as none given, so the two spellings read as one.
pub(crate) fn teammate_of(run: &api::Run) -> Option<Teammate> {
    let line = run.line();
    let mut words: Vec<&str> = line.split_whitespace().collect();
    if words.first() == Some(&"yardr") {
        words.remove(0);
    }
    match words.as_slice() {
        ["user", "invite", name, rest @ ..] => {
            let (email, role) = invite_fields(rest)?;
            Some(Teammate::Invite {
                name: (*name).to_string(),
                email,
                role,
            })
        }
        ["user", "uninvite", name] => Some(Teammate::Uninvite {
            name: (*name).to_string(),
        }),
        _ => None,
    }
}

/// The address and the role of an invite's words past its name: one
/// `--email`, and `--role` at most once. `member` where none is named.
fn invite_fields(words: &[&str]) -> Option<(String, String)> {
    let mut email: Option<String> = None;
    let mut role: Option<String> = None;
    let mut ix = 0;
    while ix < words.len() {
        match (words[ix], words.get(ix + 1)) {
            ("--email", Some(value)) if email.is_none() => {
                email = Some((*value).to_string());
            }
            ("--role", Some(value)) if role.is_none() => {
                role = Some((*value).to_string());
            }
            _ => return None,
        }
        ix += 2;
    }
    Some((email?, role.unwrap_or_else(|| "member".to_string())))
}

/// Whether the run pairs a device: `user device pair <user> <device>`,
/// with `--url <address>` where the command names one. As the yard's
/// `IsDevicePairRun` reads it: the user and the device stand, and the
/// first three words are the row's.
pub(crate) fn is_device_pair_run(run: &api::Run) -> bool {
    let argv = &run.argv;
    argv.len() >= 5 && argv[0] == "user" && argv[1] == "device" && argv[2] == "pair"
}

/// What a pairing card says of the door its approval crosses, beside
/// the card's weight and before the approval goes. On this machine's
/// socket no edge carries the code; sealed to the device the edge
/// carries it unread; with the token in the clear whoever carries the
/// approval, the edge included, could spend the code first.
pub(crate) fn pair_door(sealed: bool, remote: bool) -> &'static str {
    match (sealed, remote) {
        (true, _) => {
            "approving answers the pairing's code sealed to this device: the edge carries it unread"
        }
        (false, true) => {
            "approving shows the pairing's code on this screen: show it to the device and to nobody else, since whoever has it within its minutes, the carrier of this approval included, could spend it first"
        }
        (false, false) => {
            "approving shows the pairing's code on this screen alone: no edge carries it"
        }
    }
}

/// The head of a teammate ask's card: which of the two it is.
pub(crate) fn teammate_kind(teammate: &Teammate) -> &'static str {
    match teammate {
        Teammate::Invite { .. } => "Teammate invite",
        Teammate::Uninvite { .. } => "Teammate removal",
    }
}

/// The address an invite admits.
pub(crate) fn teammate_email(email: &str) -> String {
    format!("email: {}", model::plain(email))
}

/// The yard user an invite makes, or a removal removes.
pub(crate) fn teammate_user(name: &str) -> String {
    format!("yard user: {}", model::plain(name))
}

/// The role an invite holds: `member` by default, and an operator named
/// as one that can approve and administer the yard, never silently.
pub(crate) fn teammate_role(role: &str) -> String {
    match role {
        "operator" => "role: operator — can approve and administer this yard".to_string(),
        role => format!("role: {}", model::plain(role)),
    }
}

/// What an invite card says beside its fields: the name and the role are
/// the proposer's, and an approval runs them as shown, so a correction
/// goes by declining with words that name the corrected invite, which is
/// then asked again.
pub(crate) const TEAMMATE_CORRECTION: &str = "approval runs this command as shown: to change the name or role, decline with words naming the corrected invite";

/// A hash the card names, or that there is none, as the terminal says it.
pub(crate) fn or_none(sha: &str) -> String {
    match sha.is_empty() {
        true => "(none)".to_string(),
        false => model::plain(sha),
    }
}

/// What the tap behind a run ask's sentences reads: how many texts it
/// holds, and whether they stand shown.
pub(crate) fn texts_label(count: usize, shown: bool) -> String {
    let texts = match count {
        1 => "1 text".to_string(),
        count => format!("{count} texts"),
    };
    match shown {
        true => format!("hide {texts}"),
        false => format!("show {texts}"),
    }
}

/// The keys of a dealt with: nothing is chosen by default and Enter alone
/// sends nothing; a number then Enter replaces the pick. One taken back has
/// no keys: nothing is sent from its card.
pub(crate) fn decided_keys(detail: &DecidedDetail) -> String {
    if detail.state == "taken back" {
        return String::new();
    }
    if detail.options.is_empty() {
        return "i words · enter sends".to_string();
    }
    format!("1-{} change your pick", detail.options.len())
}

/// The button that sends a card, exactly as Enter does: `Give NAME` on a
/// secret ask, `Answer ↵` or `Send ↵` where words alone go, `Answer: 1 ·
/// Move them now ↵` with an option (`Change to: 2 · …` on a dealt with).
/// Disabled while its field is empty: drawn faint, and no click. A pick
/// button is never disabled: a pick alone sends. The option is the pick,
/// else the recommendation, which stands preselected.
pub(crate) fn send_button(
    id: String,
    text: String,
    enabled: bool,
    palette: Palette,
    cx: &mut Context<Footplate>,
) -> impl IntoElement {
    let click = enabled.then(|| cx.listener(|this, _, _, cx| this.decide(cx)));
    card_button(id, text, palette, click)
}

/// The button that approves a run ask, exactly as its key does, which it
/// names with the yard that runs the command: `Approve in plantime (⌘↵)`.
/// Where the Mac's unlock stands in front, it says so instead:
/// `Approve in plantime after unlocking`. Disabled while an approval is
/// with the yard.
pub(crate) fn approve_button(
    id: String,
    yard: &str,
    unlock: bool,
    enabled: bool,
    palette: Palette,
    cx: &mut Context<Footplate>,
) -> impl IntoElement {
    let click = enabled.then(|| cx.listener(|this, _, _, cx| this.approve(cx)));
    let label = match unlock {
        true => approve_unlock_label(yard),
        false => approve_label(yard),
    };
    card_button(id, label, palette, click)
}

/// What a run or setup card's Approve move reads beside its key chip:
/// the yard that runs the command, and unlocking first where the Mac's
/// unlock stands in front. The chip carries the key, so the label names
/// no key: `Approve in home`, chipped `⌘↵`.
pub(crate) fn approve_move_label(yard: &str, unlock: bool) -> String {
    match unlock {
        true => format!("Approve in {yard} after unlocking"),
        false => format!("Approve in {yard}"),
    }
}

/// One move of an open card's row: its label with its key on a chip
/// beside it. Filled is the card's one fill (Approve, Answer); the rest
/// are outlined. Without a click it stands disabled and takes none, as a
/// held or waited card's moves do.
pub(crate) fn move_button(
    id: String,
    label: String,
    key: String,
    filled: bool,
    palette: Palette,
    cx: &mut Context<Footplate>,
    click: Option<impl Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
) -> Stateful<Div> {
    let enabled = click.is_some();
    let mut chip = div()
        .px(px(2.))
        .py(px(1.))
        .rounded(px(4.))
        .border_1()
        .border_color(match filled {
            true => palette.on_button,
            false => palette.strong,
        })
        .text_size(rems(TYPE_META))
        .text_color(match filled {
            true => palette.on_button,
            false => palette.faint,
        })
        .child(key);
    if let Some(mono) = select::mono(cx) {
        chip = chip.font_family(mono);
    }
    div()
        .id(SharedString::from(id.clone()))
        .debug_selector(|| id.clone())
        .min_h(px(32.))
        .px_1()
        .py_1()
        .rounded(px(RADIUS_CONTROL))
        .border_1()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap_1()
        .when(filled && enabled, |el| {
            el.bg(palette.button)
                .border_color(palette.button)
                .text_color(palette.on_button)
                .cursor_pointer()
                .hover(|style| style.bg(palette.button_hover))
                .active(|style| style.bg(palette.button_pressed))
        })
        .when(filled && !enabled, |el| {
            el.bg(palette.button)
                .border_color(palette.button)
                .text_color(palette.on_button)
                .opacity(0.5)
        })
        .when(!filled && enabled, |el| {
            el.bg(palette.raised)
                .border_color(palette.strong)
                .text_color(palette.text)
                .cursor_pointer()
                .hover(|style| style.bg(palette.hover))
        })
        .when(!filled && !enabled, |el| {
            el.bg(palette.raised)
                .border_color(palette.line)
                .text_color(palette.disabled)
                .opacity(0.5)
        })
        .when_some(click, |el, click| el.on_click(click))
        .child(
            div()
                .text_size(rems(TYPE_LABEL))
                .font_weight(FontWeight::MEDIUM)
                .child(label),
        )
        .child(chip)
}

/// The primary button: one filled button, standing as the one to take,
/// with what a click on it does: a card's that sends or approves, or the
/// away screen's that sets a yard up. Without that it is drawn quiet, and
/// takes no click. The stateful element itself is returned, so a caller
/// with more to say (the switcher's no-shrink, the first run's kept keys)
/// can say it.
pub(crate) fn card_button(
    id: String,
    text: String,
    palette: Palette,
    click: Option<impl Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
) -> Stateful<Div> {
    let enabled = click.is_some();
    div()
        .id(SharedString::from(id.clone()))
        .debug_selector(|| id.clone())
        .min_h(px(32.))
        .flex()
        .items_center()
        .justify_center()
        .px_3()
        .py_1()
        .text_size(rems(TYPE_LABEL))
        .rounded_md()
        .when(enabled, |el| {
            el.bg(palette.button)
                .text_color(palette.on_button)
                .font_weight(FontWeight::MEDIUM)
                .cursor_pointer()
                .hover(|style| style.bg(palette.button_hover))
                .active(|style| style.bg(palette.button_pressed))
        })
        .when(!enabled, |el| {
            el.text_color(palette.disabled)
                .border_1()
                .border_color(palette.line)
        })
        .when_some(click, |el, click| el.on_click(click))
        .child(text)
}

/// What the button on a secret ask's card says: `Give` with the name the
/// field takes now (`Give PUSHOVER_TOKEN`), where the ask names it.
pub(crate) fn give_label(secret: &api::Secret, typed: usize) -> String {
    match secret.env.get(typed) {
        Some(name) => format!("Give {name}"),
        None => "Give".to_string(),
    }
}

/// What the button on a weighty secret ask's card says where the Mac's
/// unlock stands in front of it: the name, and that unlocking comes
/// first.
pub(crate) fn give_unlock_label(secret: &api::Secret, typed: usize) -> String {
    match secret.env.get(typed) {
        Some(name) => format!("Give {name} after unlocking"),
        None => "Give after unlocking".to_string(),
    }
}

/// One candidate of a setup ask's account choice: the source with the
/// account its file holds, in the yard's own words (`setupAccountLine` in
/// internal/store/runask_setup.go of the yardr repository).
pub(crate) fn candidate_line(candidate: &api::SetupCandidate) -> String {
    let pairs: Vec<String> = candidate
        .values
        .iter()
        .map(|(name, value)| format!("{}={}", model::plain(name), model::plain(value)))
        .collect();
    format!(
        "{} — {} (in {})",
        model::plain(&candidate.source),
        pairs.join(", "),
        model::plain(&candidate.file)
    )
}

/// A setup candidate the card took, named in words beside its account.
/// Selection is also shown by the raised row; it needs no glyph.
pub(crate) fn candidate_choice_line(candidate: &api::SetupCandidate, taken: &str) -> String {
    let line = candidate_line(candidate);
    if candidate.source == taken {
        format!("taken: {line}")
    } else {
        line
    }
}

/// The toggle beside a secret ask's field: `Show` draws what is typed as
/// typed, for a look, `Hide` draws marks again. The value is unchanged
/// either way: it goes in its call as ever.
pub(crate) fn show_button(
    id: String,
    shown: bool,
    palette: Palette,
    cx: &mut Context<Footplate>,
) -> impl IntoElement {
    let text = if shown { "Hide" } else { "Show" }.to_string();
    small_button(text, palette)
        .id(SharedString::from(id.clone()))
        .debug_selector(|| id.clone())
        .on_click(cx.listener(|this, _, _, cx| this.toggle_secret(cx)))
}

/// Whether the words of an open ordinary card want their own button:
/// they are the answer where there are no options, or a note where there
/// is no pick and none recommended. With a pick the pick's button sends
/// them, and with a recommendation and no pick the recommendation's does;
/// on another yard's bead words alone go as no note.
pub(crate) fn words_send(ask: &DecisionAsk, foreign: bool) -> bool {
    if ask.secret.is_some() || ask.run.is_some() || ask.setup.is_some() || ask.chosen != 0 {
        return false;
    }
    if ask.options.is_empty() {
        return true;
    }
    ask.recommend == 0 && !foreign
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A run proposing `command`, as the yard serves it.
    fn run_of(command: &str) -> api::Run {
        let argv: Vec<String> = command
            .split_whitespace()
            .skip(1)
            .map(str::to_string)
            .collect();
        api::Run {
            argv,
            command: command.to_string(),
            ..api::Run::default()
        }
    }

    /// An invite reads its name, address and role from the command, a
    /// member's where none is named; a removal reads its user. Anything
    /// else is no teammate ask, and keeps the generic run card.
    #[test]
    fn teammate_invites_and_removals_parse_from_the_command() {
        let eve = |role: &str| {
            Some(Teammate::Invite {
                name: "eve".to_string(),
                email: "eve@example.net".to_string(),
                role: role.to_string(),
            })
        };
        let cases: &[(&str, Option<Teammate>)] = &[
            (
                "yardr user invite eve --email eve@example.net",
                eve("member"),
            ),
            (
                "yardr user invite eve --email eve@example.net --role operator",
                eve("operator"),
            ),
            (
                "yardr user invite eve --email eve@example.net --role member",
                eve("member"),
            ),
            (
                "yardr user invite eve --role operator --email eve@example.net",
                eve("operator"),
            ),
            (
                "yardr user uninvite eve",
                Some(Teammate::Uninvite {
                    name: "eve".to_string(),
                }),
            ),
            ("yardr user invite eve", None),
            ("yardr user invite", None),
            (
                "yardr user invite eve --email eve@example.net --email jo@example.net",
                None,
            ),
            (
                "yardr user invite eve --email eve@example.net --role operator --role member",
                None,
            ),
            ("yardr user invite eve --email", None),
            ("yardr user uninvite", None),
            ("yardr user uninvite eve now", None),
            ("yardr user add eve", None),
            ("yardr user rm eve", None),
            ("yardr source grant crm read --crew builder-1", None),
            ("", None),
        ];
        for (command, wanted) in cases {
            assert_eq!(teammate_of(&run_of(command)), *wanted, "{command}");
        }
        // A yard that sent the words alone has them read the same.
        let mut words_only = run_of("yardr user invite eve --email eve@example.net");
        words_only.command.clear();
        assert_eq!(teammate_of(&words_only), eve("member"));
    }

    /// The head of a teammate card: which it is, its fields, and that a
    /// correction goes by declining with words.
    #[test]
    fn teammate_cards_say_which_they_are_with_their_fields() {
        let invite = Teammate::Invite {
            name: "eve".to_string(),
            email: "eve@example.net".to_string(),
            role: "member".to_string(),
        };
        let removal = Teammate::Uninvite {
            name: "eve".to_string(),
        };
        assert_eq!(teammate_kind(&invite), "Teammate invite");
        assert_eq!(teammate_kind(&removal), "Teammate removal");
        assert_eq!(teammate_email("eve@example.net"), "email: eve@example.net");
        assert_eq!(teammate_user("eve"), "yard user: eve");
        assert_eq!(teammate_role("member"), "role: member");
        assert_eq!(
            teammate_role("operator"),
            "role: operator — can approve and administer this yard"
        );
        assert_eq!(
            TEAMMATE_CORRECTION,
            "approval runs this command as shown: to change the name or role, decline with words naming the corrected invite"
        );
    }

    /// A pairing row is `user device pair` with its user and device, and
    /// `--url` where it names one; every shorter or other row is not.
    #[test]
    fn device_pair_rows_read_from_their_words() {
        let cases: &[(&str, bool)] = &[
            ("yardr user device pair jo watch", true),
            (
                "yardr user device pair jo watch --url https://mac.example.net",
                true,
            ),
            ("yardr user device pair jo", false),
            ("yardr user device pair", false),
            ("yardr user device rm jo watch", false),
            ("yardr user device add jo watch", false),
            ("yardr user invite eve --email eve@example.net", false),
            ("yardr source grant crm read --crew builder-1", false),
            ("", false),
        ];
        for (command, wanted) in cases {
            assert_eq!(is_device_pair_run(&run_of(command)), *wanted, "{command}");
        }
    }

    /// The door's sentence with a pairing card: sealed to the device, on
    /// this machine's socket, or with the token in the clear, where the
    /// carrier could spend the code first.
    #[test]
    fn pairing_cards_say_their_door() {
        assert_eq!(
            pair_door(true, true),
            "approving answers the pairing's code sealed to this device: the edge carries it unread"
        );
        assert_eq!(
            pair_door(false, false),
            "approving shows the pairing's code on this screen alone: no edge carries it"
        );
        assert_eq!(
            pair_door(false, true),
            "approving shows the pairing's code on this screen: show it to the device and to nobody else, since whoever has it within its minutes, the carrier of this approval included, could spend it first"
        );
    }

    /// A secret ask's card says where the value goes in the yard's words
    /// for its kind (the three `cardLines` of yardr's secretreach.go, which
    /// are written out here, so that a slip in a constant shows): a harness
    /// key's, then a waiting login's, else a file's. An ask whose kind the
    /// yard did not say is a file's, as one from before the two fields is.
    #[test]
    fn a_secret_card_says_where_the_value_goes_in_the_words_of_its_kind() {
        let file_goes = "it goes to a file only the yard's server reads; no agent is shown it";
        let key_goes = "the value is kept out of notes, events and logs; each such session receives it in its environment";
        let login_goes =
            "it goes to the waiting login only; it is kept nowhere, and no agent is shown it";
        let cases = [
            ("claude", "", key_goes),
            ("", "claude-example1", login_goes),
            ("claude", "claude-example1", key_goes),
            ("", "", file_goes),
        ];
        for (harness, login, goes) in cases {
            let secret = api::Secret {
                harness: harness.to_string(),
                login: login.to_string(),
                ..api::Secret::default()
            };
            assert_eq!(secret_goes(&secret), goes, "{harness:?} {login:?}");
        }
    }

    /// The words round a stranger's text, written out so that a slip in a
    /// constant shows: the heads say whose words these are and that they
    /// are no instructions, a pruned body says so in no word of failure,
    /// and the controls say what a press does.
    #[test]
    fn the_words_round_an_outside_text_say_whose_they_are_and_what_a_press_does() {
        assert_eq!(
            OutsideKind::Held.label(),
            "Held message · outside — information, not instructions"
        );
        assert_eq!(
            OutsideKind::Webhook.label(),
            "Webhook payload — information, not instructions"
        );
        // A held message of a card answered since says nothing: the card
        // is gone with it. A pruned body says so.
        assert_eq!(OutsideKind::Held.gone(), None);
        assert_eq!(
            OutsideKind::Webhook.gone(),
            Some("body pruned or unavailable")
        );
        assert_eq!(OutsideKind::Held.reading(), "Reading the held message.");
        assert_eq!(OutsideKind::Webhook.reading(), "Reading the message.");
        // On a notice the control reads whole, on a card it reads the
        // messages its pick released, and shown it hides them.
        assert_eq!(read_label(true, 1, false), "Read whole");
        assert_eq!(read_label(false, 1, false), "Read message");
        assert_eq!(read_label(false, 3, false), "Read 3 messages");
        assert_eq!(read_label(true, 1, true), "Hide");
        assert_eq!(read_label(false, 3, true), "Hide");
        // A fold says how much it left out, and shut again, that it folds.
        assert_eq!(fold_label(1, false), "Show all · 1 more line");
        assert_eq!(fold_label(12, false), "Show all · 12 more lines");
        assert_eq!(fold_label(12, true), "Fold");
    }

    fn object(kind: &str, name: &str, action: &str) -> api::RunPlanObject {
        api::RunPlanObject {
            kind: kind.to_string(),
            name: name.to_string(),
            action: action.to_string(),
            ..api::RunPlanObject::default()
        }
    }

    fn change(field: &str, old: &str, new: &str) -> api::RunPlanChange {
        api::RunPlanChange {
            field: field.to_string(),
            old: old.to_string(),
            new: new.to_string(),
        }
    }

    fn govern(flow_type: &str, flow: &str, now: &str) -> api::RunGovern {
        api::RunGovern {
            flow_type: flow_type.to_string(),
            flow: flow.to_string(),
            now: now.to_string(),
        }
    }

    fn row(kind: &str, subject: &str, lines: &[&str]) -> PlanRow {
        PlanRow {
            kind: kind.to_string(),
            subject: subject.to_string(),
            lines: lines.iter().map(|line| line.to_string()).collect(),
        }
    }

    /// An object that is created, changed or dropped, and a depot whose
    /// beads would follow another flow, are what the import moves; one
    /// that differs stays as the yard has it in its own section; one
    /// unchanged, kept as the yard has it on an older yard, or taken by
    /// a group of the yard, and a depot whose beads already follow the
    /// flow, already match; what the pack does not carry, or says
    /// nothing of, is in none. A stage map reads as the hook it makes.
    #[test]
    fn a_plan_s_objects_and_flows_stand_in_what_changes_or_what_already_matches() {
        let mut keeper = object("group", "reviewers", "kept");
        keeper.keeps = vec!["limit 4: the group is the yard's".to_string()];
        let mut mapped = object("map", "gh", "change");
        mapped.changes = vec![change("map", "", "pull_request synchronize → pushed")];
        let mut differing_flow = object("flow", "starter.default", "differs");
        differing_flow.changes = vec![change("stages", "new, triage, review", "new, review")];
        let plan = api::RunPlan {
            objects: vec![
                object("file", "roles/pr-reviewer.md", "create"),
                object("flow", "contributors.pr", "change"),
                object("file", "roles/old.md", "dropped"),
                differing_flow,
                object("file", "flows/default/new.md", "differs"),
                object("crew", "builders", "unchanged"),
                keeper,
                object("group", "assembly", "takes"),
                object("file", "notes.md", "not_in_pack"),
                object("file", "later.md", "an action of a newer yard"),
                // A yard from before the rows sends neither kind nor action.
                object("", "flows/default.md", ""),
                mapped,
            ],
            governs: vec![
                govern("pr", "contributors.pr", "none"),
                govern("bug", "contributors.bug", "contributors.bug"),
            ],
        };
        let (moves, differs, matches) = plan_rows(&plan, "signalbox");
        let moved: Vec<(&str, &str)> = moves
            .iter()
            .map(|row| (row.kind.as_str(), row.subject.as_str()))
            .collect();
        assert_eq!(
            moved,
            [
                ("file", "roles/pr-reviewer.md"),
                ("flow", "contributors.pr"),
                ("file", "roles/old.md"),
                ("hook", "gh"),
                ("depot", "pr"),
            ]
        );
        let differing: Vec<(&str, &str)> = differs
            .iter()
            .map(|row| (row.kind.as_str(), row.subject.as_str()))
            .collect();
        assert_eq!(
            differing,
            [
                ("flow", "starter.default"),
                ("file", "flows/default/new.md"),
            ]
        );
        let matching: Vec<(&str, &str)> = matches
            .iter()
            .map(|row| (row.kind.as_str(), row.subject.as_str()))
            .collect();
        assert_eq!(
            matching,
            [
                ("crew", "builders"),
                ("group", "reviewers"),
                ("group", "assembly"),
                ("depot", "bug"),
            ]
        );
    }

    /// A row says what it moves, a line a field; a differing row says
    /// yard against line for each field, or kind and name alone with no
    /// fields; a kept row says what the yard keeps; a depot's beads say
    /// which flow they follow now.
    #[test]
    fn a_plan_row_says_what_it_moves_or_keeps() {
        let mut flow = object("flow", "contributors.pr", "change");
        flow.changes = vec![
            change("stages", "new, review", "new, triage, review"),
            change("stage triage", "", "triage on pushed"),
            change("stage ciwait", "ciwait on pushed", ""),
            change("scope", "", ""),
        ];
        let mut differing_flow = object("flow", "starter.default", "differs");
        differing_flow.changes = vec![
            change("stages", "new, triage, review", "new, review"),
            change("stage triage", "triage -> review", ""),
            change("stage check", "", "check -> review"),
            change("scope", "", ""),
        ];
        let mut differing_group = object("group", "web-builders", "differs");
        differing_group.changes = vec![change("limit", "3", "2")];
        let mut kept = object("group", "reviewers", "kept");
        kept.keeps = vec![
            "limit 4: the group is the yard's".to_string(),
            String::new(),
        ];
        let mut taken = object("group", "assembly", "takes");
        taken.keeps = vec!["its cmd: set by hand".to_string()];
        let plan = api::RunPlan {
            objects: vec![
                flow,
                object("file", "roles/pr-reviewer.md", "create"),
                object("group", "new-group", "create"),
                object("file", "roles/old.md", "dropped"),
                differing_flow,
                differing_group,
                object("file", "flows/default/new.md", "differs"),
                kept,
                taken,
                object("crew", "builders", "unchanged"),
            ],
            governs: vec![
                govern("pr", "contributors.pr", "none"),
                govern("", "starter", ""),
                govern("bug", "contributors.bug", "contributors.bug"),
                govern("", "starter", "starter"),
            ],
        };
        let (moves, differs, matches) = plan_rows(&plan, "signalbox");
        assert_eq!(
            moves,
            [
                row(
                    "flow",
                    "contributors.pr",
                    &[
                        "stages: new, review → new, triage, review",
                        "+ triage on pushed",
                        "− ciwait on pushed",
                        "scope",
                    ]
                ),
                // Created, with no field to say: the name, as added.
                row("file", "roles/pr-reviewer.md", &["+ roles/pr-reviewer.md"]),
                row("group", "new-group", &["+ new-group"]),
                row("file", "roles/old.md", &["no longer in the pack"]),
                row("depot", "pr", &["follows contributors.pr, was none"]),
                // A flow of no type is the depot's own, and where the yard
                // says nothing of the one it follows now, no was.
                row("depot", "signalbox", &["follows starter"]),
            ]
        );
        assert_eq!(
            differs,
            [
                row(
                    "flow",
                    "starter.default",
                    &[
                        "stages: yard new, triage, review, line new, review",
                        "stage triage: yard triage -> review",
                        "stage check: line check -> review",
                        "scope",
                    ]
                ),
                row("group", "web-builders", &["limit: yard 3, line 2"]),
                // Differs, with no fields: kind and name alone.
                row("file", "flows/default/new.md", &[]),
            ]
        );
        assert_eq!(
            matches,
            [
                row("group", "reviewers", &["limit 4: the group is the yard's"]),
                row("group", "assembly", &["its cmd: set by hand"]),
                row("crew", "builders", &[]),
                row(
                    "depot",
                    "bug",
                    &["beads of type bug already follow contributors.bug"]
                ),
                row("depot", "signalbox", &["already follows starter"]),
            ]
        );
        // No depot named: the flow of no type is still the depot's.
        let (moves, _, _) = plan_rows(
            &api::RunPlan {
                governs: vec![govern("", "starter", "none")],
                ..api::RunPlan::default()
            },
            "",
        );
        assert_eq!(
            moves,
            [row("depot", "the depot", &["follows starter, was none"])]
        );
    }

    /// What the yard sent is drawn as letters: a control character in a
    /// name, a field or a value never reaches a row.
    #[test]
    fn a_plan_row_is_made_of_plain_text() {
        let mut flow = object("fl\u{1b}ow", "con\u{7}tributors", "change");
        flow.changes = vec![change("sta\u{1b}ge", "o\u{7}ld", "ne\u{2028}w")];
        let mut diff = object("gro\u{1b}up", "bui\u{7}lders", "differs");
        diff.changes = vec![change("li\u{1b}mit", "3\u{7}", "2\u{2028}")];
        let (moves, differs, _) = plan_rows(
            &api::RunPlan {
                objects: vec![flow, diff],
                governs: vec![
                    govern("p\u{1b}r", "flo\u{7}w", "no\u{1b}w"),
                    govern("", "flo\u{7}w", "no\u{1b}w"),
                ],
            },
            "de\u{7}pot",
        );
        assert_eq!(
            moves,
            [
                row("flow", "contributors", &["stage: old → new"]),
                row("depot", "pr", &["follows flow, was now"]),
                row("depot", "depot", &["follows flow, was now"]),
            ]
        );
        assert_eq!(
            differs,
            [row("group", "builders", &["limit: yard 3, line 2"])]
        );
    }

    /// The line over the sections counts what changes by kind, in the
    /// order the kinds first stand, the crew as one body, says how many
    /// differ and stay as the yard has them, and how many more already
    /// match the yard, where any do.
    #[test]
    fn a_plan_s_summary_counts_what_changes_by_kind_and_what_already_matches() {
        let moves = [
            row("flow", "contributors.pr", &[]),
            row("hook", "gh", &[]),
            row("crew", "yardmaster", &[]),
        ];
        assert_eq!(
            plan_summary(&moves, 0, 6),
            "3 changes across 1 flow, 1 hook and the crew. 6 more already match the yard and will not move."
        );
        assert_eq!(
            plan_summary(&moves, 0, 1),
            "3 changes across 1 flow, 1 hook and the crew. 1 more already matches the yard and will not move."
        );
        assert_eq!(
            plan_summary(&moves, 0, 0),
            "3 changes across 1 flow, 1 hook and the crew."
        );
        assert_eq!(
            plan_summary(&moves, 2, 4),
            "3 changes across 1 flow, 1 hook and the crew. 2 differ and stay as the yard has them. 4 more already match the yard and will not move."
        );
        // Plurals are the kind's word and an s; the crew stays the crew.
        let many = [
            row("file", "a", &[]),
            row("file", "b", &[]),
            row("group", "c", &[]),
            row("crew", "d", &[]),
            row("crew", "e", &[]),
        ];
        assert_eq!(
            plan_summary(&many, 0, 0),
            "5 changes across 2 files, 1 group and the crew."
        );
        assert_eq!(
            plan_summary(&[row("flow", "a", &[])], 1, 2),
            "1 change across 1 flow. 1 differs and stays as the yard has it. 2 more already match the yard and will not move."
        );
        // Nothing moves: unchanged or differs alone still says Nothing changes.
        assert_eq!(
            plan_summary(&[], 0, 4),
            "Nothing changes. 4 already match the yard and will not move."
        );
        assert_eq!(
            plan_summary(&[], 0, 1),
            "Nothing changes. 1 already matches the yard and will not move."
        );
        assert_eq!(
            plan_summary(&[], 2, 0),
            "Nothing changes. 2 differ and stay as the yard has them."
        );
        assert_eq!(
            plan_summary(&[], 1, 0),
            "Nothing changes. 1 differs and stays as the yard has it."
        );
        assert_eq!(
            plan_summary(&[], 2, 4),
            "Nothing changes. 2 differ and stay as the yard has them. 4 already match the yard and will not move."
        );
        assert_eq!(PLAN_DIFFERS, "The yard's own version stays");
        assert_eq!(plan_matches_count(6), "6 · kept as the yard's");
    }

    /// A section's control says how many rows it holds back, and shut
    /// again, that it folds; the marks of what is put in and what is taken
    /// out are told apart from the line.
    #[test]
    fn a_plan_section_s_control_and_a_line_s_mark_are_said_apart() {
        assert_eq!(plan_more_label(3, false), "Show 3 more");
        assert_eq!(plan_more_label(1, false), "Show 1 more");
        assert_eq!(plan_more_label(3, true), "Fold");
        assert_eq!(
            plan_mark("+ triage on pushed"),
            (Some('+'), "triage on pushed")
        );
        assert_eq!(
            plan_mark("− ciwait on pushed"),
            (Some('−'), "ciwait on pushed")
        );
        assert_eq!(plan_mark("stages: a → b"), (None, "stages: a → b"));
        // A sign that is part of a word is no mark.
        assert_eq!(plan_mark("+x"), (None, "+x"));
        assert_eq!(plan_mark("-x"), (None, "-x"));
    }
}
