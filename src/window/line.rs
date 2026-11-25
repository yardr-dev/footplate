//! The Line window: one depot's line read whole through `Line` —
//! its flows named by their source, the stage track in the flow
//! map's words, the stage detail with its Rule line, its chain, its
//! Other rules and its next stages, its stage file and prompts each
//! behind Read, and the crew prompts below. A prompt the read says a
//! card can carry has an Edit, an editor over the text that stands,
//! whose Send mails the yardmaster the proposed text: the card it
//! becomes stands in Decide, and this window approves nothing. A
//! prompt the read says no card can carry has the reason in Edit's
//! place. A stage file is the yard's own copy where its flow is the
//! store's or a pack's, and saves straight to it from its own editor
//! (`stagefile.rs`), after a review of the change and the weighty gate.
//! What serves a stage and its stages are filed by mail: a Served-by
//! picker under every Rule line and the Stages row mail the yardmaster
//! the card or the depot bead each change stands on, and approve
//! nothing here.

use gpui::{
    AnyElement, Context, Div, Entity, Focusable, FontWeight, SharedString, Window, div, prelude::*,
    px, rems,
};

use super::Footplate;
use super::ask::card_button;
use super::metrics::{LINE_TRACK_CELL, line_flow_dir_budget, line_track_overflows};
use super::panel::{buffer_stop, panel_track_line};
use super::stagefile::StageAt;
use super::ui::{field, shorten, small_button};
use crate::api::{Event, LineFlow, LinePrompt, LineStage};
use crate::input;
use crate::model::{self, EditNow, Model, PromptOwner, Selected};
use crate::select;
use crate::theme::{Palette, TYPE_META};

/// What the editor says before Send, which is design yardr-y1cn's two
/// costs of filing by mail. The mail is the filing, so the text stays in
/// the mail log, and the edit waits on the yardmaster's next pass, so
/// the card will stand in Decide and nothing has changed until it is
/// approved there.
const EDITOR_WORDS: &str = "The mail is the filing, so the text stays in the mail log. The yardmaster files a card, which will stand in Decide. Nothing changes until you approve it there, and a session already running keeps the prompt it started with.";

/// What the editor says of a prompt that changed under it. The text
/// typed is over the one that was read, so a card over it is stale.
const EDITOR_MOVED: &str = "This prompt changed since you read it, so Send is held. Copy your text if you want it, cancel, and edit again over the text that stands now.";

/// How many characters of the yardmaster's answer the row shows: the
/// rest is in its mail.
const ANSWER_CHARS: usize = 600;

/// The Line window's prompt editor, while one is open: whose prompt, the
/// prompt as the read named it when the editor opened (what the mail
/// names, and what a later read is compared with) and the field the
/// proposed text is typed in. One stands at a time, and one that holds
/// typed text is never dropped by another's opening.
pub(crate) struct LineEdit {
    pub(crate) depot: String,
    pub(crate) owner: PromptOwner,
    /// The row the editor stands under: the prompt's text key.
    pub(crate) key: String,
    pub(crate) over: LinePrompt,
    pub(crate) field: Entity<input::Line>,
    /// A send is on its way: no second one goes after it.
    pub(crate) sending: bool,
    /// Why the last send did not go, in the yard's words, with what was
    /// typed still in the field.
    pub(crate) said: Option<String>,
}

impl LineEdit {
    /// Whether the field holds more than the text the editor opened over.
    fn changed(&self, cx: &gpui::App) -> bool {
        self.field.read(cx).text() != input::line_breaks(&self.over.text)
    }
}

/// The last prompt edit the Line window sent: whose, over which prompt,
/// the text, and the yardmaster's answer once one has come. It stays,
/// through reads of the line, until another is sent or it is cleared:
/// where the line is read again its prompt says whether the edit is
/// waiting, landed or moved.
pub(crate) struct LineSent {
    pub(crate) depot: String,
    pub(crate) owner: PromptOwner,
    pub(crate) over: LinePrompt,
    pub(crate) proposed: String,
    /// The newest event of the yard when the mail went: only mail after
    /// it can be the answer.
    pub(crate) after: u64,
    pub(crate) answer: Option<String>,
}

/// The last stage or server filing the Line window mailed: what it
/// asked for, in the window's words, and the yardmaster's answer once
/// one has come. It stays until another is mailed or it is cleared:
/// the card it becomes stands in Decide, and this window approves
/// nothing.
pub(crate) struct LineFiled {
    pub(crate) depot: String,
    pub(crate) stage: String,
    pub(crate) words: String,
    /// The newest event of the yard when the mail went: only mail after
    /// it can be the answer.
    pub(crate) after: u64,
    pub(crate) answer: Option<String>,
}

/// The New group form, while it stands open: the name typed for the
/// group and the runner chosen first for its card.
pub(crate) struct LineNewGroup {
    pub(crate) name: Entity<input::Line>,
    pub(crate) runner: String,
}

/// A served pick whose mail is on its way: the key its mark stood
/// under, and the group the mail names.
#[derive(Clone)]
pub(crate) struct LineServedSending {
    pub(crate) key: (String, String, String),
    pub(crate) group: String,
}

/// The key a Served-by picker's mark stands under: its depot, stage
/// and rule token, the rule's id or `none` where no rule serves yet.
fn served_key(
    depot: &str,
    stage: &str,
    rule: Option<(&str, &str, &crate::api::LineFilters)>,
) -> (String, String, String) {
    let token = rule.map(|(id, _, _)| id).unwrap_or("none");
    (depot.to_string(), stage.to_string(), token.to_string())
}

/// The card a served pick goes by, in the picker's words: a depot's
/// own rule by its route card, a yard-wide rule by the set-aside for
/// this depot alone, and a stage with no rule by a new route card.
fn served_card_words(rule: Option<(&str, &str, &crate::api::LineFilters)>) -> &'static str {
    match rule {
        Some((_, "this depot's own", _)) => "route card",
        Some(_) => "set-aside card",
        None => "route card",
    }
}

/// The runners a new group's card offers, as the card names them:
/// the runner is the choice, chosen before the card is filed.
const GROUP_RUNNERS: [&str; 3] = ["herdr", "exec", "manual"];

/// Whether the text typed is something to send: not blank, and not the
/// text that stands.
pub(crate) fn sendable(over: &str, proposed: &str) -> bool {
    !proposed.trim().is_empty() && proposed != input::line_breaks(over)
}

impl Footplate {
    /// Opens the editor over `over`, the prompt as the read named it, with
    /// the text that stands in the field and the focus in it. One editor
    /// stands at a time: Edit on the row that holds it only takes the focus
    /// back, and Edit on another, while the one that stands holds typed
    /// text, says to finish or cancel that first, so that no text typed is
    /// ever dropped for another's opening.
    pub(crate) fn edit_prompt(
        &mut self,
        depot: String,
        owner: PromptOwner,
        key: String,
        over: LinePrompt,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(open) = self.line_edit.as_ref() {
            if open.depot == depot && open.key == key {
                window.focus(&open.field.focus_handle(cx));
                return;
            }
            if open.changed(cx) {
                let words = format!(
                    "finish or cancel the open edit of the prompt of {} {} in {} first",
                    open.owner.kind(),
                    open.owner.name(),
                    open.depot
                );
                return self.notice(words, cx);
            }
        }
        let field = cx.new(input::Line::editor);
        field.update(cx, |line, cx| {
            line.set_text(&over.text, cx);
            // The text is read from its top, not from its end.
            line.set_cursor(0, cx);
        });
        // Send is dim while there is nothing to send: the window is drawn
        // again with what is typed.
        cx.observe(&field, |_, _, cx| cx.notify()).detach();
        window.focus(&field.focus_handle(cx));
        self.line_edit = Some(LineEdit {
            depot,
            owner,
            key,
            over,
            field,
            sending: false,
            said: None,
        });
        self.notice(String::new(), cx);
    }

    /// Shuts the editor, and what was typed in it goes with it. A send
    /// that is on its way cannot be recalled: keep the editor while it is
    /// sending, so its guard prevents another edit from sending the same
    /// mail again.
    pub(crate) fn cancel_prompt_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.line_edit.as_ref().is_some_and(|edit| edit.sending) {
            return;
        }
        if self.line_edit.take().is_some() {
            window.focus(&self.focus.clone());
        }
        cx.notify();
    }

    /// Whether the line as read says the prompt `edit` was opened over is
    /// another now, or gone. A line not read at the moment (a read is on
    /// its way) says nothing.
    fn line_edit_moved(&self, edit: &LineEdit) -> bool {
        let Some(line) = self
            .model
            .as_ref()
            .and_then(|m| m.line_read_for(&edit.depot))
        else {
            return false;
        };
        model::line_prompt_of(line, &edit.owner)
            .is_none_or(|now| model::line_prompt_moved(&edit.over, now))
    }

    /// Mails the yardmaster the proposed text, once: the mail names the
    /// depot, the group or crew member and the prompt file and digest the
    /// editor was opened over, and carries the text whole. Held while a
    /// send is on its way, while the yard is reconnecting (nothing is
    /// queued, so a mail sent as the connection broke cannot go twice),
    /// while the prompt has moved under the editor, and while there is
    /// nothing to send. A mail that went closes the editor and leaves the
    /// hand-off; one that did not leaves the editor as it was, with why.
    pub(crate) fn send_prompt_edit(&mut self, cx: &mut Context<Self>) {
        if self.reconnecting() {
            return;
        }
        let Some(edit) = self.line_edit.as_ref() else {
            return;
        };
        let proposed = edit.field.read(cx).text().to_string();
        if edit.sending || self.line_edit_moved(edit) || !sendable(&edit.over.text, &proposed) {
            return;
        }
        let (depot, owner, over) = (edit.depot.clone(), edit.owner.clone(), edit.over.clone());
        let editor = edit.field.entity_id();
        // A yard with no yardmaster to write to refuses the mail in the
        // same words: said before it goes, with the text kept.
        let unwritable = self
            .model
            .as_ref()
            .and_then(|m| m.unwritable(model::YARDMASTER));
        if let Some(edit) = self.line_edit.as_mut() {
            edit.sending = unwritable.is_none();
            edit.said = unwritable.map(|why| format!("not sent: {why}"));
            if !edit.sending {
                return cx.notify();
            }
        }
        cx.notify();
        let mail = model::line_edit_mail(&depot, &owner, &over, &proposed);
        let session = self.session;
        self.ask_through(
            cx,
            move |client, me| client.mail(model::YARDMASTER, &mail, me, &[]),
            move |this, sent, cx| {
                // A yard that is not open any more shows nothing of this.
                if this.session != session {
                    return;
                }
                let mine = |this: &Self| {
                    this.line_edit
                        .as_ref()
                        .is_some_and(|edit| edit.field.entity_id() == editor)
                };
                match sent {
                    Ok(()) => {
                        if mine(this) {
                            this.line_edit = None;
                            // The field held the focus and is not drawn
                            // any more: esc is the window's.
                            let (focus, handle) = (this.focus.clone(), this.handle);
                            cx.defer(move |cx| {
                                let _ = handle.update(cx, |_, window, _| window.focus(&focus));
                            });
                        }
                        let after = this.model.as_ref().map_or(0, Model::events_seen);
                        this.line_sent = Some(LineSent {
                            depot,
                            owner,
                            over,
                            proposed,
                            after,
                            answer: None,
                        });
                        this.change_in(session, cx, |m| m.sent(model::YARDMASTER));
                    }
                    Err(err) => {
                        if mine(this)
                            && let Some(edit) = this.line_edit.as_mut()
                        {
                            edit.sending = false;
                            edit.said = Some(format!("not sent: {err}"));
                        }
                        cx.notify();
                    }
                }
            },
        );
    }

    /// The stream told of `event`. Where it is the yardmaster's mail to the
    /// person after a prompt edit went, the hand-off shows it as it is: the
    /// answer, whether a card stands in Decide or the prompt changed since
    /// it was read. Only the first mail after the send is taken.
    pub(crate) fn line_hears(&mut self, event: &Event) {
        let Some(model) = self.model.as_ref() else {
            return;
        };
        if let Some(sent) = self.line_sent.as_mut()
            && sent.answer.is_none()
            && event.seq > sent.after
            && let Some(text) = model.answer_in(event, model::YARDMASTER)
        {
            sent.answer = Some(text.trim().to_string());
        }
        if let Some(filed) = self.line_filed.as_mut()
            && filed.answer.is_none()
            && event.seq > filed.after
            && let Some(text) = model.answer_in(event, model::YARDMASTER)
        {
            filed.answer = Some(text.trim().to_string());
        }
    }

    /// Mails the yardmaster one stage or server filing, once: held
    /// while a filing is on its way and while the yard is
    /// reconnecting, so a mail sent as the connection broke cannot go
    /// twice. A mail that went leaves the hand-off under its stage;
    /// one that did not is said in the notice, with nothing filed.
    /// Where the filing is a served pick, `served` names its mark, which
    /// is taken only once the mail has gone, and only where it still
    /// names the group the mail names: a send that did not go, and a
    /// pick made while one was on its way, keep the mark for another
    /// try.
    pub(crate) fn file_line_mail(
        &mut self,
        depot: String,
        stage: String,
        mail: String,
        words: String,
        served: Option<LineServedSending>,
        cx: &mut Context<Self>,
    ) {
        if self.reconnecting() || self.line_filing {
            return;
        }
        if let Some(why) = self
            .model
            .as_ref()
            .and_then(|m| m.unwritable(model::YARDMASTER))
        {
            return self.notice(format!("not sent: {why}"), cx);
        }
        self.line_filing = true;
        self.line_served_sending = served.clone();
        self.line_refused = None;
        cx.notify();
        let session = self.session;
        self.ask_through(
            cx,
            move |client, me| client.mail(model::YARDMASTER, &mail, me, &[]),
            move |this, sent, cx| {
                if this.session != session {
                    return;
                }
                this.line_filing = false;
                this.line_served_sending = None;
                match sent {
                    Ok(()) => {
                        let after = this.model.as_ref().map_or(0, Model::events_seen);
                        this.line_filed = Some(LineFiled {
                            depot,
                            stage,
                            words,
                            after,
                            answer: None,
                        });
                        this.line_new_stage = None;
                        this.line_new_group = None;
                        if let Some(sent) = served
                            && this
                                .line_served_marks
                                .get(&sent.key)
                                .is_some_and(|marked| marked.as_str() == sent.group.as_str())
                        {
                            this.line_served_marks.remove(&sent.key);
                        }
                        this.change_in(session, cx, |m| m.sent(model::YARDMASTER));
                    }
                    Err(err) => this.notice(format!("not sent: {err}"), cx),
                }
            },
        );
    }

    /// Files the serving pick of one rule, from its picker's Send: a
    /// depot's own rule by its route card, a yard-wide rule by the
    /// set-aside card for this depot alone, and a stage with no rule by
    /// a new rule. No pick names a yard-wide rule itself as the thing
    /// changed. Only the marked group goes: a Send for no mark, or for
    /// another mark, files nothing.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn file_served_pick(
        &mut self,
        depot: &str,
        stage: &str,
        rule: Option<(&str, &str, &crate::api::LineFilters)>,
        group: &str,
        cx: &mut Context<Self>,
    ) {
        let key = served_key(depot, stage, rule);
        if self
            .line_served_marks
            .get(&key)
            .is_none_or(|marked| marked.as_str() != group)
        {
            return;
        }
        let (mail, words) = match rule {
            Some((id, scope, filters)) if scope == "this depot's own" => (
                model::line_route_mail(depot, stage, id, scope, filters, group),
                format!(
                    "Mailed the yardmaster to file the route card for rule {id}. The card will stand in Decide; this window approves nothing."
                ),
            ),
            Some((id, scope, filters)) => (
                model::line_set_aside_mail(depot, stage, id, scope, filters, group),
                format!(
                    "Mailed the yardmaster to file the set-aside card for rule {id} in depot {depot}. The card will stand in Decide; this window approves nothing, and rule {id} itself is not changed."
                ),
            ),
            None => (
                model::line_route_add_mail(depot, stage, group),
                format!(
                    "Mailed the yardmaster to file a route card for the stage {stage}, which no rule serves. The card will stand in Decide; this window approves nothing."
                ),
            ),
        };
        let sending = LineServedSending {
            key,
            group: group.to_string(),
        };
        self.file_line_mail(
            depot.to_string(),
            stage.to_string(),
            mail,
            words,
            Some(sending),
            cx,
        );
    }

    /// Files the stage change `proposed` of `flow`: a store flow by
    /// its flow card, and a pack's flow by the same card, since it is
    /// a store flow its pack brought; a git depot's repository flow by
    /// a bead in the depot. No other flow is filed from here, and a
    /// pack's flow is never filed as a bead of the depot, which has no
    /// file of it.
    pub(crate) fn file_flow_change(
        &mut self,
        depot: &str,
        flow: &LineFlow,
        proposed: &[LineStage],
        removed: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        if flow.source == "store" || flow.source.starts_with("pack ") {
            let mail = model::line_flow_mail(depot, &flow.name, &model::line_flow_json(proposed));
            let words = format!(
                "Mailed the yardmaster to file the flow card for flow {}. The card will stand in Decide; this window approves nothing.",
                flow.name
            );
            return self.file_line_mail(depot.to_string(), String::new(), mail, words, None, cx);
        }
        let flow_file = format!(".yardr/flows/{}.flow", flow.name);
        let mut text = model::line_flow_text(proposed);
        for stage in proposed {
            let stands = text.lines().any(|line| {
                line.strip_prefix(&stage.stage)
                    .is_some_and(|rest| rest.starts_with(" ->") || rest == " terminal")
            });
            if !stands {
                text.push_str(&format!(
                    "# stage {}: added here, with no moves yet\n",
                    stage.stage
                ));
            }
        }
        let beads;
        let removed = match removed {
            Some(stage) => {
                beads = self.model.as_ref().map_or(Vec::new(), |m| {
                    m.line_stage_beads(depot, &flow.name, &flow.flow_type, stage)
                });
                Some((stage, beads.as_slice()))
            }
            None => None,
        };
        let mail = model::line_flow_bead_mail(depot, &flow.name, &flow_file, &text, removed);
        let words = format!(
            "Mailed the yardmaster to file a bead in depot {depot} for flow {}. It travels the depot's own flow and lands through its review; this window approves nothing.",
            flow.name
        );
        self.file_line_mail(depot.to_string(), String::new(), mail, words, None, cx);
    }

    /// The Line window for `depot`: the pickers over the depots
    /// already listed and the depot's flows, the stage track, the
    /// open stage's detail and the crew below. A yard from before
    /// the call says so in the pane, in no red, and the window is
    /// offered no more; a read that failed says so there too, and
    /// opening the window again tries again.
    pub(crate) fn talk_line_window(
        &self,
        model: &Model,
        depot: &str,
        pane_width: f32,
        rem: f32,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let pane = div().flex_1().min_w_0().h_full().flex().flex_col();
        let head = div()
            .p_3()
            .border_b_1()
            .border_color(palette.line)
            .flex()
            .flex_col()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .debug_selector(|| "line head".to_string())
                    .child("Line"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| "line reopen".to_string())
                    .child("Read when the window opens; reopen it after an edit lands."),
            );
        if model.line_unserved() {
            return pane
                .children(self.reconnect_strip(palette))
                .child(head)
                .child(
                    div().flex_1().min_h_0().p_4().child(
                        div()
                            .debug_selector(|| "line too old".to_string())
                            .text_color(palette.faint)
                            .child(model::LINE_TOO_OLD),
                    ),
                )
                .into_any_element();
        }
        if !model.is_operator() {
            return pane
                .children(self.reconnect_strip(palette))
                .child(head)
                .child(
                    div().flex_1().min_h_0().p_4().child(
                        div()
                            .debug_selector(|| "line operator".to_string())
                            .text_color(palette.faint)
                            .child("The Line window is the operator's."),
                    ),
                )
                .into_any_element();
        }
        let mut body = div()
            .id("line window")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.decision)
            .p_4()
            .flex()
            .flex_col()
            .gap_4();
        // The depot picker: the depots already listed, each opening
        // the window on it, which reads its line.
        let depots = div().flex().flex_col().gap_1().child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.faint)
                .child("Depot"),
        );
        let mut depot_row = div().flex().flex_row().flex_wrap().gap_2();
        for listed in model.depots() {
            let name = listed.name.clone();
            let sel = Selected::LineWindow(name.clone());
            depot_row = depot_row.child(
                small_button(name.clone(), palette)
                    .id(SharedString::from(format!("line depot {name}")))
                    .debug_selector({
                        let name = name.clone();
                        move || format!("line depot {name}")
                    })
                    .when(name == depot, |el| el.bg(palette.open))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.line_flow.clear();
                        this.line_stage.clear();
                        this.line_refused = None;
                        this.line_new_stage = None;
                        this.select(sel.clone(), window, cx);
                    })),
            );
        }
        body = body.child(depots.child(depot_row));
        match model.line_read_for(depot) {
            None => {
                let words = match model.line_notice() {
                    Some(notice) => notice.to_string(),
                    None => "Reading the line…".to_string(),
                };
                body = body.child(
                    div()
                        .debug_selector(|| "line reading".to_string())
                        .text_color(palette.faint)
                        .child(words),
                );
            }
            Some(line) => {
                let flow = line
                    .flows
                    .iter()
                    .find(|flow| model::line_flow_key(flow) == self.line_flow)
                    .or_else(|| line.flows.first());
                // The flow picker: each flow named by its source.
                let flows = div().flex().flex_col().gap_1().child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.faint)
                        .child("Flow"),
                );
                let mut flow_row = div().flex().flex_row().flex_wrap().gap_2();
                for named in &line.flows {
                    let key = model::line_flow_key(named);
                    let picked = flow.is_some_and(|flow| model::line_flow_key(flow) == key);
                    let name = named.name.clone();
                    let source = named.source.clone();
                    let selector = match named.flow_type.is_empty() {
                        true => format!("{name} {source}"),
                        false => format!("{} {name} {source}", named.flow_type),
                    };
                    let selector = format!("line flow {selector}");
                    // A directory flow's button ends its path in an
                    // ellipsis past the row's room; the rest stands whole.
                    let words = match named.directory.is_empty() {
                        true => model::line_flow_words(named),
                        false => {
                            let whole = model::line_flow_words(named);
                            let budget = line_flow_dir_budget(
                                pane_width,
                                rem,
                                whole.chars().count(),
                                named.directory.chars().count(),
                            );
                            model::line_flow_dir_words(named, &shorten(&named.directory, budget))
                        }
                    };
                    flow_row = flow_row.child(
                        small_button(words, palette)
                            .id(SharedString::from(selector.clone()))
                            .debug_selector({
                                let selector = selector.clone();
                                move || selector.clone()
                            })
                            .when(picked, |el| el.bg(palette.open))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.line_flow = key.clone();
                                this.line_stage.clear();
                                this.line_refused = None;
                                this.line_new_stage = None;
                                cx.notify();
                            })),
                    );
                }
                body = body.child(flows.child(flow_row));
                if let Some(flow) = flow {
                    body = body.child(self.line_track(flow, pane_width, rem, palette, cx));
                    for (index, warning) in flow.warnings.iter().enumerate() {
                        body = body.child(
                            div()
                                .text_sm()
                                .text_color(palette.faint)
                                .debug_selector({
                                    let flow = flow.name.clone();
                                    move || format!("line flow warning {flow} {index}")
                                })
                                .child(warning.clone()),
                        );
                    }
                    let stage = flow
                        .stages
                        .iter()
                        .find(|stage| stage.stage == self.line_stage)
                        .or_else(|| flow.stages.first());
                    if let Some(stage) = stage {
                        body = body.child(self.line_detail(depot, flow, stage, palette, cx));
                    }
                }
                body = body.child(self.line_crew(line, palette, cx));
            }
        }
        pane.children(self.reconnect_strip(palette))
            .child(head)
            .child(body)
            .into_any_element()
    }

    /// One flow's stage track: each stage a cell with its name, who
    /// serves it in the flow map's words and its open count, over
    /// the panel's track line, with the buffer stop where entering
    /// closes the bead. It scrolls sideways past the pane's edge, where
    /// the track runs on a tie with a › stands at the edge to say so, as
    /// the panel strip's does. A tap opens the stage's detail below.
    fn line_track(
        &self,
        flow: &LineFlow,
        pane_width: f32,
        rem: f32,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let flow_heading = model::line_flow_heading(flow);
        let mut track = div()
            .id(SharedString::from(format!("line track {flow_heading}")))
            .debug_selector({
                let heading = flow_heading.clone();
                move || format!("line track {heading}")
            })
            .flex_1()
            .min_w_0()
            .overflow_x_scroll()
            .flex()
            .flex_row();
        track.style().restrict_scroll_to_axis = Some(true);
        let selected = flow
            .stages
            .iter()
            .find(|stage| stage.stage == self.line_stage)
            .or_else(|| flow.stages.first())
            .map(|stage| stage.stage.as_str());
        for stage in &flow.stages {
            let who = model::line_who(stage);
            let picked = selected == Some(stage.stage.as_str());
            let name = stage.stage.clone();
            let flow_name = flow_heading.clone();
            let mut cell = div()
                .id(SharedString::from(format!(
                    "line stage {} {}",
                    flow_heading, stage.stage
                )))
                .debug_selector({
                    let (flow, stage) = (flow_heading.clone(), stage.stage.clone());
                    move || format!("line stage {flow} {stage}")
                })
                .flex_none()
                .w(px(LINE_TRACK_CELL))
                .px_2()
                .py_1()
                .rounded_md()
                .flex()
                .flex_col()
                .gap_1()
                .cursor_pointer()
                .when(picked, |el| el.bg(palette.open))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.line_stage = name.clone();
                    this.line_refused = None;
                    this.line_new_stage = None;
                    cx.notify();
                }))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(stage.stage.clone()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(palette.faint)
                        .debug_selector({
                            let (flow, stage) = (flow_name.clone(), stage.stage.clone());
                            move || format!("line who {flow} {stage}")
                        })
                        .child(who),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(palette.faint)
                        .child(match stage.terminal {
                            true => "—".to_string(),
                            false => format!("{} open", stage.open),
                        }),
                )
                .child(panel_track_line(who == "people", stage.open > 0, palette));
            if stage.terminal {
                cell = cell.child(buffer_stop(
                    format!("line stop {} {}", flow_heading, stage.stage),
                    palette,
                ));
            }
            track = track.child(cell);
        }
        let marks = line_track_overflows(pane_width, rem, flow.stages.len());
        div()
            .flex()
            .flex_row()
            .items_center()
            .child(track)
            .when(marks, |el| {
                el.child(
                    div()
                        .debug_selector({
                            let flow = flow_heading.clone();
                            move || format!("line track more {flow}")
                        })
                        .flex_none()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .pl_2()
                        .pr_4()
                        .child(div().w(px(20.)).h(px(2.)).flex_none().bg(palette.strong))
                        .child(
                            div()
                                .text_size(rems(TYPE_META))
                                .text_color(palette.faint)
                                .child("›"),
                        ),
                )
            })
    }

    /// One stage's detail under the track: its headline, its Rule
    /// line, its chain, its Other rules, its next stages, its stage
    /// file and its groups' prompts, each behind Read.
    fn line_detail(
        &self,
        depot: &str,
        flow: &LineFlow,
        stage: &LineStage,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let groups = self
            .model
            .as_ref()
            .and_then(|m| m.line_read_for(depot))
            .map(model::line_groups)
            .unwrap_or_default();
        let who = model::line_who(stage);
        let served = match stage.route.as_ref().and_then(|route| route.chain.first()) {
            Some(group) => format!("served by {} ({who})", group.name),
            None if stage.terminal => "entering it closes the bead".to_string(),
            None => format!("served by nobody ({who})"),
        };
        let mut detail = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .debug_selector({
                        let stage = stage.stage.clone();
                        move || format!("line detail {stage}")
                    })
                    .child(format!("Stage {} · {served}", stage.stage)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let stage = stage.stage.clone();
                        move || format!("line open {stage}")
                    })
                    .child(match stage.open {
                        1 => "Open · 1 bead".to_string(),
                        open => format!("Open · {open} beads"),
                    }),
            );
        if let Some(words) = self.model.as_ref().and_then(|m| m.line_last_change(stage)) {
            detail = detail.child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let stage = stage.stage.clone();
                        move || format!("line last {stage}")
                    })
                    .child(words),
            );
        }
        if let Some(route) = &stage.route {
            detail = detail.child(
                div()
                    .debug_selector({
                        let stage = stage.stage.clone();
                        move || format!("line rule {stage}")
                    })
                    .child(format!(
                        "Rule {}",
                        model::line_rule_words(&route.id, &route.scope, &route.filters)
                    )),
            );
            for (at, group) in route.chain.iter().enumerate() {
                let words = match at {
                    0 => format!("{} · {}", group.name, model::line_group_words(group)),
                    _ => format!("then {} · {}", group.name, model::line_group_words(group)),
                };
                let words = match group.when.is_empty() {
                    true => words,
                    false => format!("{words} where {}", group.when),
                };
                detail = detail.child(
                    div()
                        .text_sm()
                        .debug_selector({
                            let (stage, group) = (stage.stage.clone(), group.name.clone());
                            move || format!("line chain {stage} {group}")
                        })
                        .child(words),
                );
            }
            if let Some(limit) = route.limit {
                detail = detail.child(
                    div()
                        .text_sm()
                        .text_color(palette.faint)
                        .debug_selector({
                            let stage = stage.stage.clone();
                            move || format!("line limit {stage}")
                        })
                        .child(format!("limit {limit}")),
                );
            }
            let current = route.chain.first().map(|g| g.name.as_str()).unwrap_or("");
            detail = detail.child(self.line_served(
                depot,
                &stage.stage,
                Some((&route.id, &route.scope, &route.filters)),
                current,
                &groups,
                true,
                palette,
                cx,
            ));
        }
        if stage.route.is_none() && !stage.terminal {
            detail = detail.child(self.line_served(
                depot,
                &stage.stage,
                None,
                "",
                &groups,
                true,
                palette,
                cx,
            ));
        }
        if !stage.other_routes.is_empty() {
            detail = detail.child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.faint)
                    .child("Other rules"),
            );
            for other in &stage.other_routes {
                detail = detail.child(
                    div()
                        .text_sm()
                        .debug_selector({
                            let (stage, id) = (stage.stage.clone(), other.id.clone());
                            move || format!("line other {stage} {id}")
                        })
                        .child(format!(
                            "{} → {}",
                            model::line_rule_words(&other.id, &other.scope, &other.filters),
                            other.group
                        )),
                );
                detail = detail.child(self.line_served(
                    depot,
                    &stage.stage,
                    Some((&other.id, &other.scope, &other.filters)),
                    &other.group,
                    &groups,
                    false,
                    palette,
                    cx,
                ));
            }
        }
        let next = match stage.outcomes.is_empty() {
            false => stage
                .outcomes
                .iter()
                .map(model::line_outcome_words)
                .collect::<Vec<_>>()
                .join(" · "),
            true => stage.next.join(", "),
        };
        if !next.is_empty() {
            detail = detail.child(
                div()
                    .text_sm()
                    .debug_selector({
                        let stage = stage.stage.clone();
                        move || format!("line next {stage}")
                    })
                    .child(format!("Next {next}")),
            );
        }
        for warning in &stage.warnings {
            detail = detail.child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let stage = stage.stage.clone();
                        move || format!("line warning {stage}")
                    })
                    .child(warning.clone()),
            );
        }
        detail = detail.child(self.line_stage_file(depot, flow, stage, palette, cx));
        if let Some(route) = &stage.route {
            for group in &route.chain {
                match &group.prompt {
                    Some(prompt) => {
                        let head = match prompt.source.is_empty() {
                            true => format!("Prompt · {}", model::line_path_words(&prompt.path)),
                            false => format!(
                                "Prompt · {} ({})",
                                model::line_path_words(&prompt.path),
                                prompt.source
                            ),
                        };
                        detail = detail.child(self.line_prompt_row(
                            depot,
                            PromptOwner::Group(group.name.clone()),
                            head,
                            format!(
                                "prompt:{}:{}:{}:{}:{}",
                                flow.flow_type, flow.name, flow.source, stage.stage, group.name
                            ),
                            format!("line prompt {} {}", stage.stage, group.name),
                            prompt,
                            palette,
                            cx,
                        ));
                    }
                    // A group with no prompt has none to edit: the stage
                    // says so, by what serves it, and no editor stands
                    // over a stage file in its place (design yardr-y1cn 3.2).
                    None => {
                        detail = detail.child(
                            div()
                                .text_sm()
                                .text_color(palette.faint)
                                .debug_selector({
                                    let (stage, group) = (stage.stage.clone(), group.name.clone());
                                    move || format!("line prompt {stage} {group} none")
                                })
                                .child(format!(
                                    "Prompt · {} · {}",
                                    group.name,
                                    model::line_no_prompt_words(group)
                                )),
                        );
                    }
                }
            }
        }
        detail = detail.child(self.line_stages(depot, flow, stage, palette, cx));
        if let Some(filed) = self
            .line_filed
            .as_ref()
            .filter(|f| f.depot == depot && (f.stage.is_empty() || f.stage == stage.stage))
        {
            detail = detail.child(self.line_filed_row(filed, palette, cx));
        }
        detail
    }

    /// A stage's file section: its head with Read, under it where its text
    /// comes from and the controls that follow (Edit, or Add stage file
    /// where none stands, Restore prior text…, or Copy flow into yard… on a
    /// repository's flow), and the stage file editor under the text where
    /// one stands open over the stage. Where the stage has no file the head
    /// says so. The context drawer's Stage tab draws the same section over
    /// its own head.
    fn line_stage_file(
        &self,
        depot: &str,
        flow: &LineFlow,
        stage: &LineStage,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let at = StageAt::of(depot, flow, &stage.stage);
        let selector = format!("line stagefile {}", stage.stage);
        let under = self.stage_file_source(&at, flow, stage, &selector, false, palette, cx);
        let editing = self.stage_edit.as_ref().is_some_and(|edit| edit.at == at);
        let mut row = match &stage.stage_file {
            Some(file) => {
                let key = format!(
                    "stagefile:{}:{}:{}:{}",
                    flow.flow_type, flow.name, flow.source, stage.stage
                );
                self.line_text_row(
                    format!("Stage file · {}", file.path),
                    selector.clone(),
                    key,
                    &file.text,
                    None,
                    under,
                    editing,
                    palette,
                    cx,
                )
            }
            None => div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .debug_selector({
                            let selector = selector.clone();
                            move || selector.clone()
                        })
                        .child("Stage file · none"),
                )
                .children(under),
        };
        if let Some(editor) = self.stage_file_editor(&at, &selector, false, palette, cx) {
            row = row.child(editor);
        }
        row
    }

    /// One rule's Served-by picker, standing under the Rule line it
    /// edits: it names the rule's id, scope and filters first, then a
    /// button for each group the line names, with its runner's words,
    /// the group that serves now filled. Picking one only marks it, for
    /// Send to mail that scope's card: looking through the groups, or
    /// changing one's mind, sends nothing. The marked button carries
    /// the accent edge, and Send stands dim until a group is marked;
    /// picking the group that serves now marks nothing. The
    /// representative rule's picker also carries New group…, whose form
    /// stands under it. The context drawer draws the same picker on its
    /// Group tab, with no New group, over the same marks.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn line_served(
        &self,
        depot: &str,
        stage: &str,
        rule: Option<(&str, &str, &crate::api::LineFilters)>,
        current: &str,
        groups: &[(String, String)],
        new_group: bool,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let key = served_key(depot, stage, rule);
        let token = key.2.clone();
        let head = match rule {
            Some((id, scope, filters)) => {
                format!("Served by · {}", model::line_rule_words(id, scope, filters))
            }
            None => "Served by · no rule yet".to_string(),
        };
        // A mark the line has caught up with files nothing: the group
        // it names serves now, so the picker stands as with no mark.
        let marked = match self.line_served_marks.get(&key) {
            Some(marked) if marked.as_str() != current => Some(marked.clone()),
            _ => None,
        };
        let mut picker = div().flex().flex_col().gap_1();
        // The head stands on its own line, never sharing a wrapping row
        // with the buttons: at the drawer's widths the first button would
        // hang on the label's line and read as a mistake.
        picker = picker.child(
            div()
                .text_sm()
                .text_color(palette.faint)
                .debug_selector({
                    let (stage, token) = (stage.to_string(), token.clone());
                    move || format!("line served {stage} {token}")
                })
                .child(head),
        );
        let mut row = div().flex().flex_row().flex_wrap().items_center().gap_2();
        for (name, runner) in groups {
            let label = match runner.is_empty() {
                true => name.clone(),
                false => format!("{name} ({runner})"),
            };
            let (key, current, name) = (key.clone(), current.to_string(), name.clone());
            let (stage, token) = (stage.to_string(), token.clone());
            let picked = name == current;
            let is_marked = marked.as_deref() == Some(name.as_str());
            row = row.child(
                small_button(label, palette)
                    .id(SharedString::from(format!(
                        "line serve {stage} {token} {name}"
                    )))
                    .debug_selector({
                        let (stage, token, name) = (stage.clone(), token.clone(), name.clone());
                        move || format!("line serve {stage} {token} {name}")
                    })
                    .when(picked, |el| el.bg(palette.open))
                    .when(is_marked, |el| el.border_color(palette.accent))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if name == current {
                            this.line_served_marks.remove(&key);
                        } else {
                            this.line_served_marks.insert(key.clone(), name.clone());
                        }
                        cx.notify();
                    })),
            );
        }
        if new_group {
            row = row.child(
                small_button("New group…".to_string(), palette)
                    .id(SharedString::from("line newgroup open"))
                    .debug_selector(|| "line newgroup open".to_string())
                    .on_click(cx.listener(|this, _, window, cx| {
                        if this.line_new_group.is_none() {
                            let name = cx.new(input::Line::new);
                            window.focus(&name.focus_handle(cx));
                            this.line_new_group = Some(LineNewGroup {
                                name,
                                runner: GROUP_RUNNERS[0].to_string(),
                            });
                        }
                        cx.notify();
                    })),
            );
        }
        picker = picker.child(row);
        // Send mails the marked group, and stands dim while none is
        // marked or a filing is on its way; the words name the marked
        // group and the card it goes by.
        let card = served_card_words(rule);
        let words = match marked.as_deref() {
            Some(marked) => format!("Marked {marked}; Send mails the yardmaster the {card}."),
            None => format!("Pick a group to mark it; Send mails the yardmaster the {card}."),
        };
        let sending = self
            .line_served_sending
            .as_ref()
            .is_some_and(|sending| sending.key == key);
        let send = match sending {
            true => "Sending…",
            false => "Send",
        };
        let (owned_depot, owned_stage) = (depot.to_string(), stage.to_string());
        let owned_rule =
            rule.map(|(id, scope, filters)| (id.to_string(), scope.to_string(), filters.clone()));
        let click = (marked.is_some() && !self.line_filing).then(|| {
            cx.listener(move |this, _, _, cx| {
                let rule = owned_rule
                    .as_ref()
                    .map(|(id, scope, f)| (id.as_str(), scope.as_str(), f));
                let Some(marked) = marked.as_deref() else {
                    return;
                };
                this.file_served_pick(&owned_depot, &owned_stage, rule, marked, cx);
            })
        });
        // The row justifies its end, so a wrapped Send stands right
        // as an unwrapped one does, and the words may shrink past
        // their longest word (`min_w_0`), so they wrap inside the
        // picker instead of running out of its left edge.
        picker = picker.child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .justify_end()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_sm()
                        .text_color(palette.faint)
                        .debug_selector({
                            let (stage, token) = (stage.to_string(), token.clone());
                            move || format!("line served {stage} {token} words")
                        })
                        .child(words),
                )
                .child(card_button(
                    format!("line served {stage} {token} send"),
                    send.to_string(),
                    palette,
                    click,
                )),
        );
        if new_group && let Some(form) = &self.line_new_group {
            picker = picker.child(self.line_new_group_form(depot, stage, form, palette, cx));
        }
        picker
    }

    /// The New group form under a Served-by picker: its name, the
    /// runner chosen first (agents, script or people, as the card
    /// names them herdr, exec or manual), and File, which mails the
    /// yardmaster to file its card. It does nothing until a route
    /// sends it work, which the mail says.
    fn line_new_group_form(
        &self,
        depot: &str,
        stage: &str,
        form: &LineNewGroup,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let mut runners = div().flex().flex_row().flex_wrap().gap_2();
        for runner in GROUP_RUNNERS {
            let picked = form.runner == runner;
            runners = runners.child(
                small_button(
                    format!("{runner} ({})", model::line_runner_words(runner)),
                    palette,
                )
                .id(SharedString::from(format!("line newgroup runner {runner}")))
                .debug_selector(move || format!("line newgroup runner {runner}"))
                .when(picked, |el| el.bg(palette.open))
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(form) = this.line_new_group.as_mut() {
                        form.runner = runner.to_string();
                    }
                    cx.notify();
                })),
            );
        }
        let (depot, stage) = (depot.to_string(), stage.to_string());
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.faint)
                    .debug_selector(|| "line newgroup".to_string())
                    .child("New group"),
            )
            .child(
                field(form.name.focus_handle(cx), form.name.clone(), palette)
                    .debug_selector(|| "line newgroup name".to_string()),
            )
            .child(runners)
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        small_button("Cancel".to_string(), palette)
                            .id(SharedString::from("line newgroup cancel"))
                            .debug_selector(|| "line newgroup cancel".to_string())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.line_new_group = None;
                                cx.notify();
                            })),
                    )
                    .child(
                        small_button("File change…".to_string(), palette)
                            .id(SharedString::from("line newgroup file"))
                            .debug_selector(|| "line newgroup file".to_string())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let Some(form) = this.line_new_group.as_ref() else {
                                    return;
                                };
                                let name = form.name.read(cx).text().trim().to_string();
                                if name.is_empty() {
                                    return this.notice(
                                        "a new group wants a name".to_string(),
                                        cx,
                                    );
                                }
                                let mail =
                                    model::line_group_mail(&depot, &name, &form.runner);
                                let words = format!(
                                    "Mailed the yardmaster to file the group card for {name}. The card will stand in Decide; this window approves nothing."
                                );
                                this.file_line_mail(
                                    depot.clone(),
                                    stage.clone(),
                                    mail,
                                    words,
                                    None,
                                    cx,
                                );
                            })),
                    ),
            )
    }

    /// One flow's Stages row under its open stage: Add, Move up,
    /// Move down and Remove for that stage, by the door its source
    /// names. A store flow files its flow card, and so does a pack's
    /// flow, a store flow its pack brought, under the card's own lines
    /// for it: whose it is, what `yardr pack apply` then keeps and the
    /// `--take` that sets the pack's again; a git depot's repository
    /// flow composes the depot bead's mail; a dir depot's directory
    /// flow offers no control and names the terminal; a type's flow is
    /// not edited here. A removal with beads standing files nothing
    /// and names them.
    fn line_stages(
        &self,
        depot: &str,
        flow: &LineFlow,
        stage: &LineStage,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let stage_name = stage.stage.clone();
        let mut row = div().flex().flex_col().gap_1().child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.faint)
                .debug_selector({
                    let stage = stage_name.clone();
                    move || format!("line stages {stage}")
                })
                .child("Stages"),
        );
        if !flow.flow_type.is_empty() {
            return row.child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let stage = stage_name.clone();
                        move || format!("line stages {stage} typed")
                    })
                    .child("A type's flow is not edited here."),
            );
        }
        if flow.source == "repo (directory)" {
            return row.child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let stage = stage_name.clone();
                        move || format!("line stages {stage} terminal")
                    })
                    .child(match flow.directory.is_empty() {
                        true => {
                            "This flow is the folder's file: change it at a terminal, in the folder."
                                .to_string()
                        }
                        false => format!(
                            "This flow is the folder's file: change it at a terminal, in {}.",
                            flow.directory
                        ),
                    }),
            );
        }
        // A pack's flow stands in the store as any flow does, so its change
        // is the same card; the pack's words stand under it.
        let pack = flow.source.strip_prefix("pack ").map(str::to_string);
        let door = match flow.source.as_str() {
            "store" => "On a store flow the change stands on its card in Decide.",
            _ if pack.is_some() => "On a pack's flow the change stands on its card in Decide.",
            _ => {
                "This flow is the depot's file: the change travels as a bead and lands through its review."
            }
        };
        let (depot, flow) = (depot.to_string(), flow.clone());
        let at = flow
            .stages
            .iter()
            .position(|s| s.stage == stage_name)
            .unwrap_or(0);
        let mut buttons = div().flex().flex_row().flex_wrap().gap_2();
        buttons = buttons
            .child(
                small_button("Add stage…".to_string(), palette)
                    .id(SharedString::from(format!("line stages {stage_name} add")))
                    .debug_selector({
                        let stage = stage_name.clone();
                        move || format!("line stages {stage} add")
                    })
                    .on_click(cx.listener(|this, _, window, cx| {
                        if this.line_new_stage.is_none() {
                            let field = cx.new(input::Line::new);
                            window.focus(&field.focus_handle(cx));
                            this.line_new_stage = Some(field);
                        }
                        cx.notify();
                    })),
            )
            .child(
                small_button("Move up".to_string(), palette)
                    .id(SharedString::from(format!("line stages {stage_name} up")))
                    .debug_selector({
                        let stage = stage_name.clone();
                        move || format!("line stages {stage} up")
                    })
                    .when(at == 0, |el| el.text_color(palette.faint))
                    .on_click(cx.listener({
                        let (depot, flow, stage) =
                            (depot.clone(), flow.clone(), stage_name.clone());
                        move |this, _, _, cx| {
                            if at > 0 {
                                let proposed = model::line_stage_moved(&flow.stages, &stage, true);
                                this.file_flow_change(&depot, &flow, &proposed, None, cx);
                            }
                        }
                    })),
            )
            .child(
                small_button("Move down".to_string(), palette)
                    .id(SharedString::from(format!("line stages {stage_name} down")))
                    .debug_selector({
                        let stage = stage_name.clone();
                        move || format!("line stages {stage} down")
                    })
                    .when(at + 1 >= flow.stages.len(), |el| {
                        el.text_color(palette.faint)
                    })
                    .on_click(cx.listener({
                        let (depot, flow, stage) =
                            (depot.clone(), flow.clone(), stage_name.clone());
                        move |this, _, _, cx| {
                            if at + 1 < flow.stages.len() {
                                let proposed = model::line_stage_moved(&flow.stages, &stage, false);
                                this.file_flow_change(&depot, &flow, &proposed, None, cx);
                            }
                        }
                    })),
            )
            .child(
                small_button("Remove…".to_string(), palette)
                    .id(SharedString::from(format!(
                        "line stages {stage_name} remove"
                    )))
                    .debug_selector({
                        let stage = stage_name.clone();
                        move || format!("line stages {stage} remove")
                    })
                    .on_click(cx.listener({
                        let (depot, flow, stage) =
                            (depot.clone(), flow.clone(), stage_name.clone());
                        move |this, _, _, cx| {
                            let open = flow
                                .stages
                                .iter()
                                .find(|s| s.stage == stage)
                                .map_or(0, |s| s.open);
                            if open > 0 {
                                let beads = this.model.as_ref().map_or(Vec::new(), |m| {
                                    m.line_stage_beads(&depot, &flow.name, &flow.flow_type, &stage)
                                });
                                this.line_refused =
                                    Some(model::line_removal_refused(&stage, &beads, open));
                                return cx.notify();
                            }
                            let proposed = model::line_stage_removed(&flow.stages, &stage);
                            this.file_flow_change(&depot, &flow, &proposed, Some(&stage), cx);
                        }
                    })),
            );
        row = row.child(buttons);
        if let Some(input) = &self.line_new_stage {
            let (depot, flow, stage) = (depot.clone(), flow.clone(), stage_name.clone());
            row = row.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        field(input.focus_handle(cx), input.clone(), palette).debug_selector({
                            let stage = stage_name.clone();
                            move || format!("line stages {stage} newname")
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                small_button("Cancel".to_string(), palette)
                                    .id(SharedString::from(format!(
                                        "line stages {stage_name} add cancel"
                                    )))
                                    .debug_selector({
                                        let stage = stage_name.clone();
                                        move || format!("line stages {stage} add cancel")
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.line_new_stage = None;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                small_button("Add".to_string(), palette)
                                    .id(SharedString::from(format!(
                                        "line stages {stage_name} add send"
                                    )))
                                    .debug_selector({
                                        let stage = stage_name.clone();
                                        move || format!("line stages {stage} add send")
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let Some(field) = this.line_new_stage.as_ref() else {
                                            return;
                                        };
                                        let name =
                                            field.read(cx).text().trim().to_string();
                                        if name.is_empty()
                                            || flow.stages.iter().any(|s| s.stage == name)
                                        {
                                            return this.notice(
                                                format!(
                                                    "a stage named {name} cannot be added to flow {}",
                                                    flow.name
                                                ),
                                                cx,
                                            );
                                        }
                                        let proposed = model::line_stage_added(
                                            &flow.stages,
                                            &stage,
                                            &name,
                                        );
                                        this.file_flow_change(
                                            &depot, &flow, &proposed, None, cx,
                                        );
                                    })),
                            ),
                    ),
            );
        }
        row = row.child(
            div()
                .text_sm()
                .text_color(palette.faint)
                .debug_selector({
                    let stage = stage_name.clone();
                    move || format!("line stages {stage} door")
                })
                .child(door),
        );
        if let Some(pack) = &pack {
            let words = model::line_pack_words(pack, &depot, &flow.name);
            for (name, text) in [("whose", words.whose), ("keeps", words.keeps)] {
                row = row.child(
                    div()
                        .text_sm()
                        .text_color(palette.faint)
                        .debug_selector({
                            let stage = stage_name.clone();
                            move || format!("line stages {stage} pack {name}")
                        })
                        .child(text),
                );
            }
            // The command is one a person types at a terminal: drawn as
            // code is, and not run from here.
            let mut command = div()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(palette.code)
                .text_sm()
                .child(words.door);
            if let Some(mono) = select::mono(cx) {
                command = command.font_family(mono);
            }
            row = row.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .debug_selector({
                        let stage = stage_name.clone();
                        move || format!("line stages {stage} pack take")
                    })
                    .child(div().text_sm().text_color(palette.faint).child(words.back))
                    .child(div().flex().flex_row().child(command)),
            );
        }
        if let Some(refused) = &self.line_refused {
            row = row.child(
                div()
                    .text_sm()
                    .text_color(palette.warn)
                    .debug_selector({
                        let stage = stage_name.clone();
                        move || format!("line stages {stage} refused")
                    })
                    .child(refused.clone()),
            );
        }
        row
    }

    /// What stands under a stage after a filing was mailed for it:
    /// that its card will stand in Decide and this window approves
    /// nothing, the yardmaster's answer where one has come, and
    /// Clear.
    pub(crate) fn line_filed_row(
        &self,
        filed: &LineFiled,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let mut row = div().flex().flex_col().gap_1().child(
            div()
                .text_sm()
                .text_color(palette.faint)
                .debug_selector(|| "line filed".to_string())
                .child(filed.words.clone()),
        );
        if let Some(answer) = &filed.answer {
            row = row.child(
                div()
                    .text_sm()
                    .debug_selector(|| "line filed answer".to_string())
                    .child(format!(
                        "The yardmaster wrote: {}",
                        shorten(answer, ANSWER_CHARS)
                    )),
            );
        }
        row.child(
            small_button("Clear".to_string(), palette)
                .id(SharedString::from("line filed clear"))
                .debug_selector(|| "line filed clear".to_string())
                .on_click(cx.listener(|this, _, _, cx| {
                    this.line_filed = None;
                    cx.notify();
                })),
        )
    }

    /// The crew below the flows: one row per member, its prompt
    /// behind Read where its config names one.
    fn line_crew(
        &self,
        line: &crate::api::Line,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut crew = div().flex().flex_col().gap_2().child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.faint)
                .debug_selector(|| "line crew head".to_string())
                .child("Crew"),
        );
        if line.crew.is_empty() {
            crew = crew.child(div().text_sm().text_color(palette.faint).child("No crew."));
        }
        for member in &line.crew {
            crew = crew.child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .debug_selector({
                        let name = member.name.clone();
                        move || format!("line crew {name}")
                    })
                    .child(member.name.clone()),
            );
            match &member.prompt {
                Some(prompt) => {
                    let head = match prompt.source.is_empty() {
                        true => format!("Prompt · {}", model::line_path_words(&prompt.path)),
                        false => format!(
                            "Prompt · {} ({})",
                            model::line_path_words(&prompt.path),
                            prompt.source
                        ),
                    };
                    crew = crew.child(self.line_prompt_row(
                        &line.depot,
                        PromptOwner::Crew(member.name.clone()),
                        head,
                        format!("crew:{}", member.name),
                        format!("line crew prompt {}", member.name),
                        prompt,
                        palette,
                        cx,
                    ));
                }
                None => {
                    crew = crew.child(div().text_sm().text_color(palette.faint).child("no prompt"));
                }
            }
        }
        crew
    }

    /// A prompt's row: its head, Read for its text, and beside Read an
    /// Edit where the read says a card can carry the prompt, or why one
    /// cannot where it says that. The gate is that answer alone, never
    /// the prompt's source. No Edit where the prompt's text could not be
    /// read: its tried paths and error stand in its place. Under the row
    /// stand the editor, where it is open on this prompt, and the
    /// hand-off of an edit sent for the prompt's owner.
    #[allow(clippy::too_many_arguments)]
    fn line_prompt_row(
        &self,
        depot: &str,
        owner: PromptOwner,
        head: String,
        key: String,
        selector: String,
        prompt: &LinePrompt,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let editing = self
            .line_edit
            .as_ref()
            .filter(|edit| edit.depot == depot && edit.key == key);
        let edit = match (prompt.carriable, prompt.why.is_empty()) {
            (true, _) => {
                Some(self.line_edit_control(depot, &owner, &key, &selector, prompt, cx, palette))
            }
            (false, false) => Some(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let selector = selector.clone();
                        move || format!("{selector} edit why")
                    })
                    .child(prompt.why.clone())
                    .into_any_element(),
            ),
            (false, true) => None,
        };
        let mut row = self.line_text_row(
            head,
            selector.clone(),
            key,
            &prompt.text,
            edit,
            None,
            editing.is_some(),
            palette,
            cx,
        );
        if prompt.text.is_empty() && !prompt.error.is_empty() {
            row = row.child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let selector = selector.clone();
                        move || format!("{selector} error")
                    })
                    .child(match prompt.tried.is_empty() {
                        true => prompt.error.clone(),
                        false => format!("{} (tried {})", prompt.error, prompt.tried.join(", ")),
                    }),
            );
        }
        if let Some(edit) = editing {
            row = row.child(self.line_editor(edit, &selector, palette, cx));
        }
        if let Some(sent) = self
            .line_sent
            .as_ref()
            .filter(|sent| sent.depot == depot && sent.owner == owner)
        {
            row = row.child(self.line_hand_off(sent, &selector, palette, false, cx));
        }
        row
    }

    /// The Edit beside a carriable prompt's Read: it opens the editor
    /// over the text the read gave.
    #[allow(clippy::too_many_arguments)]
    fn line_edit_control(
        &self,
        depot: &str,
        owner: &PromptOwner,
        key: &str,
        selector: &str,
        prompt: &LinePrompt,
        cx: &mut Context<Self>,
        palette: Palette,
    ) -> AnyElement {
        let selector = format!("{selector} edit");
        let (depot, owner, key, prompt) = (
            depot.to_string(),
            owner.clone(),
            key.to_string(),
            prompt.clone(),
        );
        small_button("Edit".to_string(), palette)
            .id(SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone())
            .on_click(cx.listener(move |this, _, window, cx| {
                this.edit_prompt(
                    depot.clone(),
                    owner.clone(),
                    key.clone(),
                    prompt.clone(),
                    window,
                    cx,
                )
            }))
            .into_any_element()
    }

    /// The editor under a prompt's row: the field over the text that
    /// stands, what Send does and what comes of it, said before Send, and
    /// Cancel and Send. Send is held while the text is blank or what
    /// stands, while a send is on its way and while the prompt has moved
    /// under the editor, which says so. A send that did not go says why in
    /// red, with the text as it was. The context drawer draws the same
    /// editor under its prompt.
    pub(crate) fn line_editor(
        &self,
        edit: &LineEdit,
        selector: &str,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let selector = format!("{selector} editor");
        let typed = edit.field.read(cx).text().to_string();
        let moved = self.line_edit_moved(edit);
        let enabled = !edit.sending && !moved && sendable(&edit.over.text, &typed);
        let over = match edit.over.digest.is_empty() {
            true => "the prompt as read".to_string(),
            false => format!(
                "the prompt as read (digest {})",
                model::line_short_digest(&edit.over.digest)
            ),
        };
        let mut editor = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                field(edit.field.focus_handle(cx), edit.field.clone(), palette)
                    .min_h(rems(8.))
                    .debug_selector({
                        let selector = selector.clone();
                        move || selector.clone()
                    }),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let selector = selector.clone();
                        move || format!("{selector} words")
                    })
                    .child(format!(
                        "Send mails the yardmaster this text, whole, over {over}. {EDITOR_WORDS}"
                    )),
            );
        if moved {
            editor = editor.child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let selector = selector.clone();
                        move || format!("{selector} moved")
                    })
                    .child(EDITOR_MOVED),
            );
        }
        if let Some(said) = &edit.said {
            editor = editor.child(
                div()
                    .text_sm()
                    .text_color(palette.warn)
                    .debug_selector({
                        let selector = selector.clone();
                        move || format!("{selector} said")
                    })
                    .child(said.clone()),
            );
        }
        let send = match edit.sending {
            true => "Sending…",
            false => "Send",
        };
        let click = enabled.then(|| cx.listener(|this, _, _, cx| this.send_prompt_edit(cx)));
        editor.child(
            div()
                .flex()
                .justify_end()
                .gap_2()
                .child(
                    small_button("Cancel".to_string(), palette)
                        .id(SharedString::from(format!("{selector} cancel")))
                        .debug_selector({
                            let selector = selector.clone();
                            move || format!("{selector} cancel")
                        })
                        .on_click(
                            cx.listener(|this, _, window, cx| this.cancel_prompt_edit(window, cx)),
                        ),
                )
                .child(card_button(
                    format!("{selector} send"),
                    send.to_string(),
                    palette,
                    click,
                )),
        )
    }

    /// What stands under a prompt after an edit was sent for it: that the
    /// card will stand in Decide and that this window approves nothing,
    /// or, once the line is read again, that the prompt now reads as the
    /// text sent or changed since it was read, with the yardmaster's
    /// answer under it when one has come. Read again reads the line again;
    /// Clear takes the hand-off away. The context drawer draws the same
    /// row under its prompt, and there Read again reads the drawer's
    /// depot again instead of opening the Line window.
    pub(crate) fn line_hand_off(
        &self,
        sent: &LineSent,
        selector: &str,
        palette: Palette,
        drawer: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let selector = format!("{selector} sent");
        let now = self
            .model
            .as_ref()
            .and_then(|m| m.line_read_for(&sent.depot))
            .and_then(|line| model::line_prompt_of(line, &sent.owner));
        let (state, words) = match model::line_sent_now(&sent.over, &sent.proposed, now) {
            EditNow::Waiting => (
                "waiting",
                "Sent to the yardmaster. The card will stand in Decide. Nothing changes until you approve it there, and this window approves nothing.".to_string(),
            ),
            EditNow::Landed => (
                "landed",
                "The prompt now reads as the text you sent, so its card was approved.".to_string(),
            ),
            EditNow::Moved => (
                "moved",
                format!(
                    "This prompt changed since it was read. {} The yard refuses to approve a card over the old text as stale, so edit again over the text that stands.",
                    model::line_moved_words(&sent.over, now)
                ),
            ),
        };
        let depot = sent.depot.clone();
        let mut hand_off = div().flex().flex_col().gap_1().child(
            div()
                .text_sm()
                .text_color(palette.faint)
                .debug_selector({
                    let selector = selector.clone();
                    move || format!("{selector} {state}")
                })
                .child(words),
        );
        if let Some(answer) = &sent.answer {
            hand_off = hand_off.child(
                div()
                    .text_sm()
                    .debug_selector({
                        let selector = selector.clone();
                        move || format!("{selector} answer")
                    })
                    .child(format!(
                        "The yardmaster wrote: {}",
                        shorten(answer, ANSWER_CHARS)
                    )),
            );
        }
        hand_off.child(
            div()
                .flex()
                .gap_2()
                .child(
                    small_button("Read again".to_string(), palette)
                        .id(SharedString::from(format!("{selector} read")))
                        .debug_selector({
                            let selector = selector.clone();
                            move || format!("{selector} read")
                        })
                        .on_click(cx.listener(move |this, _, window, cx| match drawer {
                            true => this.reread_context(cx),
                            false => this.select(Selected::LineWindow(depot.clone()), window, cx),
                        })),
                )
                .child(
                    small_button("Clear".to_string(), palette)
                        .id(SharedString::from(format!("{selector} clear")))
                        .debug_selector({
                            let selector = selector.clone();
                            move || format!("{selector} clear")
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.line_sent = None;
                            cx.notify();
                        })),
                ),
        )
    }

    /// One text's row: its head with Read, which stands its text open
    /// below (unless an editor stands over it instead), and, where `edit`
    /// stands, the Edit or the reason none does, beside Read. `under` is
    /// what stands between the head and the text: a stage file's section.
    #[allow(clippy::too_many_arguments)]
    fn line_text_row(
        &self,
        head: String,
        selector: String,
        key: String,
        text: &str,
        edit: Option<AnyElement>,
        under: Option<Div>,
        editing: bool,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let open = self.line_reads.contains(&key) && !editing;
        let mut row = div().flex().flex_col().gap_1().child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .debug_selector({
                            let selector = selector.clone();
                            move || selector.clone()
                        })
                        .child(head),
                )
                .child(
                    small_button("Read".to_string(), palette)
                        .id(SharedString::from(format!("{selector} read")))
                        .debug_selector({
                            let selector = selector.clone();
                            move || format!("{selector} read")
                        })
                        .on_click(cx.listener({
                            let key = key.clone();
                            move |this, _, _, cx| {
                                if !this.line_reads.remove(&key) {
                                    this.line_reads.insert(key.clone());
                                }
                                cx.notify();
                            }
                        })),
                )
                .children(edit),
        );
        if let Some(under) = under {
            row = row.child(under);
        }
        if open {
            row = row.child(
                div()
                    .text_sm()
                    .debug_selector({
                        let selector = selector.clone();
                        move || format!("{selector} text")
                    })
                    .child(text.to_string()),
            );
        }
        row
    }
}

#[cfg(debug_assertions)]
impl Footplate {
    /// The editor's and the hand-off's shots, once the line is read:
    /// `line-edit` opens the editor on the first stage's first prompt a card
    /// can carry, with a change typed in it, and `line-sent` stands the
    /// hand-off of an edit written over that prompt as it stood before it
    /// changed, with the yardmaster's answer; `line-pack` stands on the
    /// pack's flow, at its middle stage. `line-stage-review` opens the stage
    /// file editor of the pack's flow, in the drawer standing on it, at the
    /// review of a change typed over its text (`line-stage-edit` is that
    /// drawer before any edit, with the file's words and controls);
    /// `line-stage-copy` stands on the repository's flow, where its stage
    /// file offers to be copied into the yard, and `line-stage-copying` has
    /// that copy's confirmation open. Once; a debug build's alone.
    pub(crate) fn shot_line_window(&mut self, cx: &mut Context<Self>) {
        let shot = std::env::var("FOOTPLATE_SHOT").unwrap_or_default();
        if !matches!(
            shot.as_str(),
            "line-edit"
                | "line-sent"
                | "line-picker"
                | "line-refused"
                | "line-pack"
                | "line-stage-review"
                | "line-stage-copy"
                | "line-stage-copying"
        ) || self.line_edit.is_some()
            || self.line_sent.is_some()
            || self.stage_edit.is_some()
            || self.stage_copy.is_some()
        {
            return;
        }
        let handle = self.handle;
        cx.spawn(async move |view, cx| {
            let _ = handle.update(cx, |_, window, cx| {
                view.update(cx, |this, cx| this.shot_line_edit(&shot, window, cx))
            });
        })
        .detach();
    }

    fn shot_line_edit(&mut self, shot: &str, window: &mut Window, cx: &mut Context<Self>) {
        // The picker shot stands on the yard-wide rule's stage, the
        // refused shot on the stage with a bead standing in it, and the
        // pack's on its flow's middle stage, where every control of the
        // Stages row is live: none needs a prompt.
        match shot {
            "line-stage-review" => {
                // The drawer stands on the pack's flow at new: the editor
                // opens over its stage file with a change typed (one line
                // of the second bullet made two), and Save… is pressed, so
                // that the review shows the change and its weight under the
                // pack's words.
                let found = self
                    .model
                    .as_ref()
                    .and_then(|m| m.line_read_for("yardr"))
                    .and_then(|line| {
                        let flow = line.flows.iter().find(|flow| flow.name == "managed")?;
                        let file = flow.stages.first()?.stage_file.clone()?;
                        Some((StageAt::of("yardr", flow, "new"), file))
                    });
                if let Some((at, file)) = found {
                    self.edit_stage_file(at, Some(&file), false, window, cx);
                    let typed = file.text.replace(
                        "- a note on the bead says what changed and what was verified.",
                        "- a note on the bead says what changed and what was verified, and ends with\n  the test line: `test: <command> @ <commit>: pass` or `fail`.",
                    );
                    if let Some(edit) = self.stage_edit.as_ref() {
                        edit.field.update(cx, |line, cx| line.set_text(&typed, cx));
                    }
                    self.review_stage_edit(window, cx);
                }
                cx.notify();
                return;
            }
            "line-stage-copy" | "line-stage-copying" => {
                // The window stands on the repository's flow at new, where
                // the stage file says it is the repository's and offers
                // Copy flow into yard…; the copying shot has pressed it.
                let found = self
                    .model
                    .as_ref()
                    .and_then(|m| m.line_read_for("yardr"))
                    .and_then(|line| line.flows.iter().find(|flow| flow.name == "repo"))
                    .map(|flow| {
                        (
                            StageAt::of("yardr", flow, "new"),
                            model::line_flow_key(flow),
                        )
                    });
                if let Some((at, key)) = found {
                    self.line_flow = key.clone();
                    self.line_stage = "new".to_string();
                    if shot == "line-stage-copying" {
                        self.ask_copy_flow(at, key, true, window, cx);
                    }
                }
                self.decision.scroll_to_bottom();
                cx.notify();
                return;
            }
            "line-pack" => {
                let key = self
                    .model
                    .as_ref()
                    .and_then(|m| m.line_read_for("yardr"))
                    .and_then(|line| {
                        line.flows
                            .iter()
                            .find(|flow| flow.source.starts_with("pack "))
                    })
                    .map(model::line_flow_key);
                if let Some(key) = key {
                    self.line_flow = key;
                }
                self.line_stage = "review".to_string();
                // The Stages row stands at the foot of the detail, with
                // the pack's words under its controls.
                self.decision.scroll_to_bottom();
                cx.notify();
                return;
            }
            "line-picker" => {
                self.line_stage = "review".to_string();
                // The serving picker stands with a group marked and Send
                // live, as a pick leaves it: the first group the line
                // names past the one that serves now. Read live, so the
                // shot follows the data it is served.
                if let Some(line) = self.model.as_ref().and_then(|m| m.line_read_for("yardr")) {
                    let stage = line
                        .flows
                        .iter()
                        .flat_map(|flow| &flow.stages)
                        .find(|stage| stage.stage == "review");
                    if let Some(stage) = stage {
                        let route = stage.route.as_ref();
                        let current = route
                            .and_then(|route| route.chain.first())
                            .map(|group| group.name.as_str())
                            .unwrap_or("");
                        let rule = route
                            .map(|route| (route.id.as_str(), route.scope.as_str(), &route.filters));
                        if let Some(marked) = model::line_groups(line)
                            .into_iter()
                            .map(|(name, _)| name)
                            .find(|name| name != current)
                        {
                            self.line_served_marks
                                .insert(served_key("yardr", "review", rule), marked);
                        }
                    }
                }
                cx.notify();
                return;
            }
            "line-refused" => {
                self.line_stage = "new".to_string();
                let (beads, open) = self
                    .model
                    .as_ref()
                    .and_then(|m| m.line_read_for("yardr"))
                    .and_then(|line| line.flows.first())
                    .and_then(|flow| flow.stages.first())
                    .map_or((Vec::new(), 0), |stage| {
                        (
                            self.model.as_ref().map_or(Vec::new(), |m| {
                                m.line_stage_beads("yardr", "default", "", &stage.stage)
                            }),
                            stage.open,
                        )
                    });
                self.line_refused = Some(model::line_removal_refused("new", &beads, open));
                // The Stages row stands at the foot of the detail:
                // the refusal under it is what the shot shows.
                self.decision.scroll_to_bottom();
                cx.notify();
                return;
            }
            _ => {}
        }
        let found = self
            .model
            .as_ref()
            .and_then(|m| m.line_read_for("yardr"))
            .and_then(|line| {
                let flow = line.flows.first()?;
                let stage = flow.stages.first()?;
                stage.route.as_ref()?.chain.iter().find_map(|group| {
                    let prompt = group.prompt.as_ref().filter(|prompt| prompt.carriable)?;
                    let key = format!(
                        "prompt:{}:{}:{}:{}:{}",
                        flow.flow_type, flow.name, flow.source, stage.stage, group.name
                    );
                    Some((key, group.name.clone(), prompt.clone()))
                })
            });
        let Some((key, group, prompt)) = found else {
            return;
        };
        let owner = PromptOwner::Group(group);
        match shot {
            "line-edit" => {
                self.edit_prompt("yardr".into(), owner, key, prompt.clone(), window, cx);
                if let Some(edit) = self.line_edit.as_ref() {
                    let typed = format!(
                        "{}Leave the branch with one commit per piece.\n",
                        prompt.text
                    );
                    edit.field.update(cx, |line, cx| line.set_text(&typed, cx));
                }
            }
            "line-sent" => {
                let digest = "9d01c7a2e5f3b84610ad3c2f5e7b9a0c1d4e6f8a2b3c5d7e9f0a1b2c3d4e5f60";
                let over = LinePrompt {
                    path: format!("/home/jo/.yardr/roles/builders.{digest}.md"),
                    found: format!("/home/jo/.yardr/roles/builders.{digest}.md"),
                    digest: digest.into(),
                    text: "Build the thing.\n".into(),
                    ..prompt
                };
                self.line_sent = Some(LineSent {
                    depot: "yardr".into(),
                    owner,
                    over,
                    proposed: "Build the thing.\nLeave the branch with one commit per piece.\n"
                        .into(),
                    after: 0,
                    answer: Some(
                        "No card was filed: the prompt changed since it was read. Read the line again, then send the edit over the text that stands."
                            .into(),
                    ),
                });
            }
            _ => {}
        }
        // The prompt's row stands near the foot of the pane at the shot's
        // size: the editor's words and its Send, or the hand-off, are below.
        self.decision.scroll_to_bottom();
        cx.notify();
    }
}
