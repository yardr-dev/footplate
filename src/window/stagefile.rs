//! A stage file edited in place, in the Line window and in the context
//! drawer's Stage tab: where its text comes from, what the yard found
//! wrong with its `## Ask`, and the one editor both draw. A stage file of a
//! store flow or a pack's is the yard's own copy, so the operator's edit
//! saves straight to it (`PutStageFile`), with no mail and no card: the
//! editor's Save… shows the change line by line, Confirm save passes the
//! weighty gate (the system's unlock, or the stage's name typed back where
//! none is offered) and only then writes, over the digest the file was
//! read at, so that a file that moved since is refused and the text typed
//! stays. A stage file of a repository flow is the repository's: nothing
//! edits it here, and Copy flow into yard… (`CopyRepoFlows`, through the
//! same gate) makes the depot's flows the store's, after which its stages
//! edit in place. Restore prior text… puts back the text the file stood as
//! before its last save, through the same review and gate.

use gpui::{
    AnyElement, App, ClickEvent, Context, Div, Entity, Focusable, FontWeight, SharedString,
    Stateful, Window, div, prelude::*, px, rems,
};

use super::Footplate;
use super::ask::card_button;
use super::line::sendable;
use super::ui::{field, small_button};
use crate::api::{Error, LineFlow, LineStage, LineStageFile};
use crate::input;
use crate::model::{self, DiffRow, LineDiff, STAGE_FILE_NONE, StageSource};
use crate::select;
use crate::theme::{Palette, TYPE_CODE, TYPE_LABEL, TYPE_META};
use crate::unlock;

/// How many unchanged lines stand either side of a change in the review.
const DIFF_CONTEXT: usize = 3;

/// What the editor says before Save…: nothing is written by it, and what
/// follows is the review.
const EDITOR_WORDS: &str =
    "Save… shows the change line by line first. Nothing is written until you confirm it there.";

/// What the editor says of a file that changed under it. The text typed is
/// over the file that was read, so the yard would refuse it as stale.
const EDITOR_MOVED: &str = "This stage file changed since you read it, so Save is held. Copy your text if you want it, cancel, and edit again over the text that stands now.";

/// What the review says of what a save changes, as the weighty class the
/// yard words the cards' weight by: a stage file is what each session in
/// the stage reads when it starts.
const SAVE_WEIGHT: &str = "changes what every future session in this stage reads";

/// What the copy says it does, and what it does not: the repository's own
/// flows stay where they are, and the yard says the command that goes back.
const COPY_WORDS: &str = "This copies the flows of the depot's repository into the yard's store, with their stage files, and the yard holds them from then on. A later change in the repository no longer governs them until you switch the depot's flows back, which the yard names when it is done. Every stage then edits in place.";

/// What the copy changes, in the weighty class's words.
const COPY_WEIGHT: &str = "copies the depot's repository flows into the yard, which then holds them instead of the repository";

/// One stage of one flow of one depot: what an editor, a copy and a saved
/// line stand over.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StageAt {
    pub(crate) depot: String,
    pub(crate) flow: String,
    pub(crate) flow_type: String,
    pub(crate) stage: String,
}

impl StageAt {
    pub(crate) fn of(depot: &str, flow: &LineFlow, stage: &str) -> Self {
        StageAt {
            depot: depot.to_string(),
            flow: flow.name.clone(),
            flow_type: flow.flow_type.clone(),
            stage: stage.to_string(),
        }
    }
}

/// Which half of the editor stands: the text typed, or the review of what
/// it changes, which is where the gate stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum StageStep {
    Edit,
    Review,
}

/// What the review shows of a change, worked out once as the review opens:
/// the text cannot change while it stands, and a long file's diff is worth
/// more than a frame's work.
#[derive(Default)]
pub(crate) struct Reviewed {
    rows: Vec<DiffRow>,
    taken_out: usize,
    put_in: usize,
}

impl Reviewed {
    /// The change from the text the editor opened over to `proposed`.
    fn of(over: &str, proposed: &str) -> Self {
        let diff = model::line_diff(&input::line_breaks(over), proposed);
        let count = |keep: fn(&LineDiff) -> bool| diff.iter().filter(|line| keep(line)).count();
        Reviewed {
            taken_out: count(|line| matches!(line, LineDiff::Removed(_))),
            put_in: count(|line| matches!(line, LineDiff::Added(_))),
            rows: model::line_diff_rows(&diff, DIFF_CONTEXT),
        }
    }

    /// What the change is, counted: the lines taken out and put in.
    fn words(&self) -> String {
        let lines = |n: usize| match n {
            1 => "1 line".to_string(),
            n => format!("{n} lines"),
        };
        format!(
            "{} taken out, {} put in",
            lines(self.taken_out),
            lines(self.put_in)
        )
    }
}

/// The stage file editor, while one is open: which stage, the file as the
/// read named it when the editor opened (its digest, `none` where the
/// stage had no file, and its text) and the field the proposed text is
/// typed in. One stands at a time, and one that holds typed text is never
/// dropped by another's opening.
pub(crate) struct StageEdit {
    pub(crate) at: StageAt,
    /// What the save names as the file it is written over.
    pub(crate) over_digest: String,
    pub(crate) over_text: String,
    pub(crate) field: Entity<input::Line>,
    pub(crate) step: StageStep,
    /// The change the review shows, over the text as it stood when the
    /// review opened; empty while the text is edited.
    pub(crate) review: Reviewed,
    /// The stage's name typed back, where this window has no gate.
    pub(crate) typeback: Entity<input::Line>,
    /// A save is on its way: no second one goes after it.
    pub(crate) sending: bool,
    /// Why the last save did not go, in the yard's words, with what was
    /// typed still in the field.
    pub(crate) said: Option<String>,
}

impl StageEdit {
    /// Whether the field holds more than the text the editor opened over.
    fn changed(&self, cx: &App) -> bool {
        self.field.read(cx).text() != input::line_breaks(&self.over_text)
    }
}

/// The copy of a depot's repository flows into the yard, while its
/// confirmation stands open under the stage it was asked from.
pub(crate) struct StageCopy {
    pub(crate) at: StageAt,
    /// The Line window's pick of the flow it was asked from.
    pub(crate) flow_key: String,
    /// Whether it was asked in the Line window: its pick of the flow then
    /// goes to the copy, as the drawer's does.
    pub(crate) from_window: bool,
    /// The depot's name typed back, where this window has no gate.
    pub(crate) typeback: Entity<input::Line>,
    pub(crate) sending: bool,
    pub(crate) said: Option<String>,
}

/// What the last save or copy left under its stage, in words: that it went
/// and what the yard said of the file, or what the copy made. It stands
/// while the file is the one that was saved.
pub(crate) struct StageSaved {
    pub(crate) at: StageAt,
    /// The digest of the file as saved; none for a copy, whose line
    /// stands until another is made.
    pub(crate) digest: Option<String>,
    pub(crate) words: String,
    /// The yard's warning of the file saved, which the line says in warn.
    pub(crate) warning: String,
}

/// What the review says over its field where this window has no gate: what
/// to type and why, in the terminal's words.
pub(crate) fn typeback_stage_line(stage: &str) -> String {
    format!("type the stage's name, {stage}, to save this weighty change")
}

/// What the editor says where the field holds anything but the stage's
/// name: nothing was written.
pub(crate) fn typeback_stage_mismatch(stage: &str) -> String {
    format!("the stage's name, {stage}, was not typed back, so nothing was saved")
}

/// What the copy says over its field where this window has no gate.
pub(crate) fn typeback_depot_line(depot: &str) -> String {
    format!("type the depot's name, {depot}, to copy its flows into the yard")
}

/// What the copy says where the field holds anything but the depot's name.
pub(crate) fn typeback_depot_mismatch(depot: &str) -> String {
    format!("the depot's name, {depot}, was not typed back, so nothing was copied")
}

/// How a write that failed is said: a yard that refused says so in its
/// words, whole, and a call that ended with no answer says what the yard
/// does not know, never that nothing was written.
fn failed_words(verb: &str, err: &Error) -> String {
    match err {
        Error::Refused { .. } | Error::Slow { sent: false, .. } => format!("not {verb}: {err}"),
        _ => err.to_string(),
    }
}

impl Footplate {
    /// The editor, while it stands over `at`.
    fn stage_edit_at(&self, at: &StageAt) -> Option<&StageEdit> {
        self.stage_edit.as_ref().filter(|edit| edit.at == *at)
    }

    /// Whether the focus is in a field of the stage file's editor or of the
    /// copy's confirmation: their keys are text, as the prompt editor's are.
    pub(crate) fn stage_field_focused(&self, window: &Window, cx: &App) -> bool {
        self.stage_edit.as_ref().is_some_and(|edit| {
            edit.field.focus_handle(cx).is_focused(window)
                || edit.typeback.focus_handle(cx).is_focused(window)
        }) || self
            .stage_copy
            .as_ref()
            .is_some_and(|copy| copy.typeback.focus_handle(cx).is_focused(window))
    }

    /// The stage as the line reads it now, by the name and type of its
    /// flow. None where the line is not read at the moment, or the flow or
    /// the stage is gone from it.
    fn stage_read(&self, at: &StageAt) -> Option<&LineStage> {
        self.model
            .as_ref()?
            .line_read_for(&at.depot)?
            .flows
            .iter()
            .find(|flow| flow.name == at.flow && flow.flow_type == at.flow_type)?
            .stages
            .iter()
            .find(|stage| stage.stage == at.stage)
    }

    /// Whether the line as read says the file `edit` was opened over is
    /// another now, or gone. A line not read at the moment (a read is on
    /// its way) says nothing.
    fn stage_edit_moved(&self, edit: &StageEdit) -> bool {
        let read = self
            .model
            .as_ref()
            .is_some_and(|m| m.line_read_for(&edit.at.depot).is_some());
        read && model::line_stage_file_moved(&edit.over_digest, self.stage_read(&edit.at))
    }

    /// Opens the editor over the stage file `file` as the read named it
    /// (none where the stage has no file: the editor then makes one), with
    /// the text that stands in the field and the focus in it. With
    /// `restore` the editor opens in its review over the text the file
    /// stood as before its last save. One editor stands at a time: Edit on
    /// the stage that holds it only takes the focus back, and Edit
    /// elsewhere, while the one that stands holds typed text, says to
    /// finish or cancel that first, so that no text typed is ever dropped
    /// for another's opening.
    pub(crate) fn edit_stage_file(
        &mut self,
        at: StageAt,
        file: Option<&LineStageFile>,
        restore: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(open) = self.stage_edit.as_ref() {
            if open.at == at && !restore {
                window.focus(&open.field.focus_handle(cx));
                return;
            }
            if open.changed(cx) || open.sending {
                let words = format!(
                    "finish or cancel the open edit of the stage file of {} in {} first",
                    open.at.stage, open.at.depot
                );
                return self.notice(words, cx);
            }
        }
        let (over_digest, over_text, seed) = match (file, restore) {
            (Some(file), true) => (
                file.digest.clone(),
                file.text.clone(),
                file.prior_text.clone(),
            ),
            (Some(file), false) => (file.digest.clone(), file.text.clone(), file.text.clone()),
            (None, _) => (STAGE_FILE_NONE.to_string(), String::new(), String::new()),
        };
        let field = cx.new(input::Line::editor);
        field.update(cx, |line, cx| {
            line.set_text(&seed, cx);
            // The text is read from its top, not from its end.
            line.set_cursor(0, cx);
        });
        // Save is dim while there is nothing to save: the window is drawn
        // again with what is typed.
        cx.observe(&field, |_, _, cx| cx.notify()).detach();
        let typeback = cx.new(input::Line::new);
        cx.subscribe(&typeback, |this, _, _: &input::Submitted, cx| {
            this.confirm_stage_edit(cx)
        })
        .detach();
        let (step, review) = match restore {
            true => (StageStep::Review, Reviewed::of(&over_text, &seed)),
            false => (StageStep::Edit, Reviewed::default()),
        };
        let opens_in_review = restore;
        self.stage_edit = Some(StageEdit {
            at: at.clone(),
            over_digest,
            over_text,
            field,
            step,
            review,
            typeback,
            sending: false,
            said: None,
        });
        self.stage_saved = None;
        if opens_in_review {
            self.show_stage_review(&at);
        }
        self.focus_stage_step(window, cx);
        self.notice(String::new(), cx);
    }

    /// The review is the last of the drawer's tab, with its buttons under
    /// the change: where the drawer stands on the stage, they come into
    /// view with it.
    fn show_stage_review(&self, at: &StageAt) {
        if self
            .context
            .as_ref()
            .is_some_and(|open| open.stands_on(&at.depot, &at.flow, &at.flow_type, &at.stage))
        {
            self.context_scroll.scroll_to_bottom();
        }
    }

    /// Puts the focus where the step is typed: in the field while the text
    /// is edited, in the stage's name where the review takes it back, and
    /// in the window otherwise.
    fn focus_stage_step(&self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(edit) = self.stage_edit.as_ref() else {
            return;
        };
        match (edit.step, self.unlock_available()) {
            (StageStep::Edit, _) => window.focus(&edit.field.focus_handle(cx)),
            (StageStep::Review, false) => window.focus(&edit.typeback.focus_handle(cx)),
            (StageStep::Review, true) => window.focus(&self.focus.clone()),
        }
    }

    /// Save…: the review of what the text typed changes, over the file as
    /// it was read. Held while a save is on its way, while the file has
    /// moved under the editor, and while there is nothing to save.
    pub(crate) fn review_stage_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(edit) = self.stage_edit.as_ref() else {
            return;
        };
        let proposed = edit.field.read(cx).text().to_string();
        if edit.sending || self.stage_edit_moved(edit) || !sendable(&edit.over_text, &proposed) {
            return;
        }
        let (at, review) = (edit.at.clone(), Reviewed::of(&edit.over_text, &proposed));
        if let Some(edit) = self.stage_edit.as_mut() {
            edit.step = StageStep::Review;
            edit.review = review;
            edit.said = None;
        }
        self.show_stage_review(&at);
        self.focus_stage_step(window, cx);
        cx.notify();
    }

    /// Back to edit: the review left for the field, with what was typed.
    pub(crate) fn back_to_stage_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(edit) = self.stage_edit.as_mut() {
            if edit.sending {
                return;
            }
            edit.step = StageStep::Edit;
            edit.review = Reviewed::default();
            edit.said = None;
        }
        self.focus_stage_step(window, cx);
        cx.notify();
    }

    /// Shuts the editor, and what was typed in it goes with it. A save
    /// that is on its way cannot be recalled: the editor stays while it is
    /// sending, so its guard keeps another confirm from writing again.
    pub(crate) fn cancel_stage_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.stage_edit.as_ref().is_some_and(|edit| edit.sending) {
            return;
        }
        if self.stage_edit.take().is_some() {
            window.focus(&self.focus.clone());
        }
        cx.notify();
    }

    /// Confirm save: writes the text typed over the file the editor was
    /// opened over, once, through the weighty gate. Held while a save is on
    /// its way, while the yard is reconnecting (nothing is queued, so a
    /// write sent as the connection broke cannot go twice), while the file
    /// has moved under the editor, and while there is nothing to save.
    /// Where the system offers no unlock the stage's name is typed back
    /// first, and anything else sends nothing. A save the yard took closes
    /// the editor and reads the line again; one it refused, or that got no
    /// answer, leaves the editor with the text and the yard's words.
    pub(crate) fn confirm_stage_edit(&mut self, cx: &mut Context<Self>) {
        if self.reconnecting() {
            return;
        }
        if let Some(depot) = self
            .stage_copy
            .as_ref()
            .filter(|copy| copy.sending)
            .and_then(|copy| {
                self.stage_edit
                    .as_ref()
                    .filter(|edit| edit.at.depot == copy.at.depot)
                    .map(|_| copy.at.depot.clone())
            })
        {
            if let Some(edit) = self.stage_edit.as_mut() {
                edit.said = Some(format!(
                    "wait for the repository flow copy in {depot} to finish before saving this stage file"
                ));
            }
            return cx.notify();
        }
        let Some(edit) = self.stage_edit.as_ref() else {
            return;
        };
        let proposed = edit.field.read(cx).text().to_string();
        if edit.step != StageStep::Review
            || edit.sending
            || self.stage_edit_moved(edit)
            || !sendable(&edit.over_text, &proposed)
        {
            return;
        }
        let typed_back = !self.unlock_available();
        if typed_back && edit.typeback.read(cx).text().trim() != edit.at.stage {
            let said = typeback_stage_mismatch(&edit.at.stage);
            if let Some(edit) = self.stage_edit.as_mut() {
                edit.said = Some(said);
            }
            return cx.notify();
        }
        let (at, over_digest) = (edit.at.clone(), edit.over_digest.clone());
        let (editor, typeback) = (edit.field.entity_id(), edit.typeback.clone());
        if let Some(edit) = self.stage_edit.as_mut() {
            edit.sending = true;
            edit.said = None;
        }
        cx.notify();
        let session = self.session;
        let reason = unlock::stage_file_reason(&at.stage, &self.yard);
        let go = move |this: &mut Self, session: u64, cx: &mut Context<Self>| {
            this.put_stage_file_in(session, at, over_digest, proposed, editor, cx)
        };
        match typed_back {
            // The name is used up: each weighty change asks again, as the
            // system's unlock does.
            true => {
                typeback.update(cx, |field, cx| field.set_text("", cx));
                go(self, session, cx);
            }
            // The system's unlock first: its cancel sends nothing, and the
            // review stands as it was for another try.
            false => self.unlock_then(session, reason, cx, go, move |this, session, cx| {
                if session == this.session
                    && let Some(edit) = this.stage_edit.as_mut()
                    && edit.field.entity_id() == editor
                {
                    edit.sending = false;
                }
                cx.notify();
            }),
        }
    }

    /// Writes the file: the call is `PutStageFile` with the gate passed
    /// (`weighed`), over the digest the editor read it at. What the yard
    /// answers is the file as it now stands, with its warning where its
    /// `## Ask` is broken: the save went all the same.
    fn put_stage_file_in(
        &mut self,
        session: u64,
        at: StageAt,
        over_digest: String,
        proposed: String,
        editor: gpui::EntityId,
        cx: &mut Context<Self>,
    ) {
        let (flow, stage) = (at.flow.clone(), at.stage.clone());
        self.ask_in(
            session,
            cx,
            move |client, me| {
                client.put_stage_file(&flow, &stage, &proposed, &over_digest, me, true)
            },
            move |this, saved, cx| {
                // A yard that is not open any more shows nothing of this.
                if this.session != session {
                    return;
                }
                let mine = |this: &Self| {
                    this.stage_edit
                        .as_ref()
                        .is_some_and(|edit| edit.field.entity_id() == editor)
                };
                match saved {
                    Ok(file) => {
                        if mine(this) {
                            this.stage_edit = None;
                            // The field held the focus and is not drawn any
                            // more: esc is the window's.
                            let (focus, handle) = (this.focus.clone(), this.handle);
                            cx.defer(move |cx| {
                                let _ = handle.update(cx, |_, window, _| window.focus(&focus));
                            });
                        }
                        let words = match file.warning.is_empty() {
                            true => "Saved. Restore prior text… puts back the text it replaced."
                                .to_string(),
                            false => format!(
                                "Saved with a broken ## Ask, which the yard keeps all the same: {}",
                                file.warning
                            ),
                        };
                        this.stage_saved = Some(StageSaved {
                            digest: Some(file.digest),
                            warning: file.warning,
                            words,
                            at: at.clone(),
                        });
                        // The line as the yard holds it now: the file, its
                        // digest, the text it replaced and its warning.
                        this.change_in(session, cx, |m| m.reread_line(&at.depot));
                    }
                    Err(err) => {
                        if mine(this)
                            && let Some(edit) = this.stage_edit.as_mut()
                        {
                            edit.sending = false;
                            edit.said = Some(failed_words("saved", &err));
                        }
                        cx.notify();
                    }
                }
            },
        );
    }

    /// Copy flow into yard…: opens the confirmation of copying the depot's
    /// repository flows, under the stage it was asked from. One stands at a
    /// time.
    pub(crate) fn ask_copy_flow(
        &mut self,
        at: StageAt,
        flow_key: String,
        from_window: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.stage_copy.as_ref().is_some_and(|copy| copy.sending) {
            return;
        }
        let typeback = cx.new(input::Line::new);
        cx.subscribe(&typeback, |this, _, _: &input::Submitted, cx| {
            this.confirm_copy_flow(cx)
        })
        .detach();
        if !self.unlock_available() {
            window.focus(&typeback.focus_handle(cx));
        }
        self.stage_copy = Some(StageCopy {
            at,
            flow_key,
            from_window,
            typeback,
            sending: false,
            said: None,
        });
        self.stage_saved = None;
        cx.notify();
    }

    /// Cancel: the confirmation shut, and nothing copied. A copy that is
    /// on its way cannot be recalled.
    pub(crate) fn cancel_copy_flow(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.stage_copy.as_ref().is_some_and(|copy| copy.sending) {
            return;
        }
        if self.stage_copy.take().is_some() {
            window.focus(&self.focus.clone());
        }
        cx.notify();
    }

    /// Confirm copy: the depot's repository flows copied into the yard's
    /// store, once, through the weighty gate, with the depot's name typed
    /// back first where the system offers no unlock. A copy the yard made
    /// moves the Line window's and the drawer's pick to the store flow it
    /// made of the flow they stood on, and reads the line again, so that
    /// its stages edit in place; one it refused leaves the confirmation
    /// with the yard's words.
    pub(crate) fn confirm_copy_flow(&mut self, cx: &mut Context<Self>) {
        if self.reconnecting() {
            return;
        }
        let Some(copy) = self.stage_copy.as_ref() else {
            return;
        };
        if copy.sending {
            return;
        }
        let busy_stage = self
            .stage_edit
            .as_ref()
            .filter(|edit| edit.sending && edit.at.depot == copy.at.depot)
            .map(|edit| edit.at.stage.clone());
        if let Some(stage) = busy_stage {
            if let Some(copy) = self.stage_copy.as_mut() {
                copy.said = Some(format!(
                    "wait for the stage file save of {stage} in {} to finish before copying repository flows",
                    copy.at.depot
                ));
            }
            return cx.notify();
        }
        let typed_back = !self.unlock_available();
        if typed_back && copy.typeback.read(cx).text().trim() != copy.at.depot {
            let said = typeback_depot_mismatch(&copy.at.depot);
            if let Some(copy) = self.stage_copy.as_mut() {
                copy.said = Some(said);
            }
            return cx.notify();
        }
        let (at, typeback) = (copy.at.clone(), copy.typeback.clone());
        if let Some(copy) = self.stage_copy.as_mut() {
            copy.sending = true;
            copy.said = None;
        }
        cx.notify();
        let session = self.session;
        let reason = unlock::copy_flows_reason(&at.depot, &self.yard);
        let go = move |this: &mut Self, session: u64, cx: &mut Context<Self>| {
            this.copy_repo_flows_in(session, at, cx)
        };
        match typed_back {
            true => {
                typeback.update(cx, |field, cx| field.set_text("", cx));
                go(self, session, cx);
            }
            false => self.unlock_then(session, reason, cx, go, |this, session, cx| {
                if session == this.session
                    && let Some(copy) = this.stage_copy.as_mut()
                {
                    copy.sending = false;
                }
                cx.notify();
            }),
        }
    }

    /// Copies the flows: the call is `CopyRepoFlows` with the gate passed
    /// (`weighed`). The yard answers each flow it copied by the bead type
    /// it governs and the store flow it is now.
    fn copy_repo_flows_in(&mut self, session: u64, at: StageAt, cx: &mut Context<Self>) {
        let depot = at.depot.clone();
        self.ask_in(
            session,
            cx,
            move |client, me| client.copy_repo_flows(&depot, me, true),
            move |this, copied, cx| {
                if this.session != session {
                    return;
                }
                match copied {
                    Ok(copied) => {
                        let copy = this.stage_copy.take();
                        // The flow the person stood on is now the store flow
                        // of the same bead type: the pick and the drawer
                        // follow it, and the line is read again to show it.
                        let made = copied
                            .flows
                            .iter()
                            .find(|made| made.flow_type == at.flow_type);
                        let mut now = at.clone();
                        if let Some(made) = made {
                            now.flow = made.flow.clone();
                            let key = model::line_flow_key(&LineFlow {
                                name: made.flow.clone(),
                                flow_type: at.flow_type.clone(),
                                source: "store".to_string(),
                                ..Default::default()
                            });
                            if copy
                                .as_ref()
                                .is_some_and(|copy| copy.from_window || this.line_flow == copy.flow_key)
                            {
                                this.line_flow = key;
                            }
                            if let Some(open) = this.context.as_mut()
                                && let model::ContextSubject::Stage {
                                    depot,
                                    flow,
                                    flow_type,
                                    ..
                                } = &mut open.subject
                                && *depot == at.depot
                                && *flow_type == at.flow_type
                                && *flow == at.flow
                            {
                                *flow = made.flow.clone();
                            }
                        }
                        let made_words = copied
                            .flows
                            .iter()
                            .map(|made| match made.back.is_empty() {
                                true => made.flow.clone(),
                                false => format!("{} (back: {})", made.flow, made.back),
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        let words = match copied.flows.len() {
                            0 => "The yard had no repository flows to copy.".to_string(),
                            _ => format!(
                                "Copied into the yard as {made_words}. Its stages edit in place now."
                            ),
                        };
                        this.stage_saved = Some(StageSaved {
                            at: now.clone(),
                            digest: None,
                            words,
                            warning: String::new(),
                        });
                        let (depot, flow_type, from, to) = (
                            at.depot.clone(),
                            at.flow_type.clone(),
                            at.flow.clone(),
                            now.flow.clone(),
                        );
                        this.change_in(session, cx, move |m| {
                            m.rename_context_flow(&depot, &flow_type, &from, &to);
                            m.reread_line(&depot)
                        });
                    }
                    Err(err) => {
                        if let Some(copy) = this.stage_copy.as_mut() {
                            copy.sending = false;
                            copy.said = Some(failed_words("copied", &err));
                        }
                        cx.notify();
                    }
                }
            },
        );
    }

    /// One control of a stage file's section: a small button the test finds
    /// by `selector`.
    fn stage_control(
        selector: String,
        text: &str,
        palette: Palette,
        click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Stateful<Div> {
        small_button(text.to_string(), palette)
            .id(SharedString::from(selector.clone()))
            .debug_selector(move || selector.clone())
            .on_click(click)
    }

    /// A line of the section's words, in the size of the surface it is
    /// drawn on: the Line window's small text, the drawer's meta.
    fn stage_words(
        words: impl IntoElement,
        selector: String,
        color: gpui::Rgba,
        drawer: bool,
    ) -> Div {
        let line = div()
            .debug_selector(move || selector.clone())
            .text_color(color);
        match drawer {
            true => line.text_size(rems(TYPE_META)),
            false => line.text_sm(),
        }
        .child(words)
    }

    /// Where a stage's text comes from and what can be done with it: the
    /// yard's warning of its `## Ask`, where it has one, in warn; the
    /// provenance line; where the yard says the store flow differs from the
    /// repository's, a line saying so; the controls that follow from the
    /// provenance — Edit, or Add stage file where none stands, and Restore
    /// prior text… where the yard kept the text before the last save, on a
    /// flow of the yard's store or a pack's; Copy flow into yard… on a
    /// repository's, which is not edited here — with the copy's
    /// confirmation under them where it stands, and what the last save or
    /// copy left. `selector` is where the section's parts are found, and
    /// `drawer` the text size of the surface it stands on. The Line window
    /// and the context drawer draw it alike, each over its own head and
    /// text.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn stage_file_source(
        &self,
        at: &StageAt,
        flow: &LineFlow,
        stage: &LineStage,
        selector: &str,
        drawer: bool,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Option<Div> {
        let source = model::line_stage_source(flow);
        let file = stage.stage_file.as_ref();
        let saved = self.stage_saved.as_ref().filter(|saved| {
            saved.at == *at
                && match (&saved.digest, file) {
                    (Some(digest), Some(file)) => *digest == file.digest,
                    (Some(_), None) => false,
                    (None, _) => true,
                }
        });
        let mut section: Vec<AnyElement> = Vec::new();
        // The warning right under the head, unless the line of the save
        // that was just made says it already.
        if let Some(file) = file
            && !file.warning.is_empty()
            && !saved.is_some_and(|saved| saved.warning == file.warning)
        {
            section.push(
                Self::stage_words(
                    file.warning.clone(),
                    format!("{selector} warning"),
                    palette.warn,
                    drawer,
                )
                .into_any_element(),
            );
        }
        if let Some(words) = source.words() {
            section.push(
                Self::stage_words(words, format!("{selector} source"), palette.faint, drawer)
                    .into_any_element(),
            );
        }
        // In the accent text and not in warn: a difference is no error, and
        // the line offers nothing to do about it here.
        if let Some(words) = model::line_drift_words(flow, stage) {
            section.push(
                Self::stage_words(
                    words,
                    format!("{selector} drift"),
                    palette.text_accent,
                    drawer,
                )
                .into_any_element(),
            );
        }
        let editing = self.stage_edit_at(at).is_some();
        let mut controls = div().flex().flex_row().flex_wrap().gap_2();
        let mut offered = false;
        if source.editable() && !editing {
            offered = true;
            let (opens, shown) = (at.clone(), file.cloned());
            match file {
                Some(_) => {
                    controls = controls.child(Self::stage_control(
                        format!("{selector} edit"),
                        "Edit",
                        palette,
                        cx.listener(move |this, _, window, cx| {
                            this.edit_stage_file(opens.clone(), shown.as_ref(), false, window, cx)
                        }),
                    ));
                }
                None => {
                    controls = controls.child(Self::stage_control(
                        format!("{selector} add"),
                        "Add stage file",
                        palette,
                        cx.listener(move |this, _, window, cx| {
                            this.edit_stage_file(opens.clone(), None, false, window, cx)
                        }),
                    ));
                }
            }
            if let Some(file) = file
                && !file.prior_text.is_empty()
                && file.prior_text != file.text
            {
                let (opens, shown) = (at.clone(), file.clone());
                controls = controls.child(Self::stage_control(
                    format!("{selector} restore"),
                    "Restore prior text…",
                    palette,
                    cx.listener(move |this, _, window, cx| {
                        this.edit_stage_file(opens.clone(), Some(&shown), true, window, cx)
                    }),
                ));
            }
        }
        let copying = self
            .stage_copy
            .as_ref()
            .filter(|copy| copy.at.depot == at.depot);
        if matches!(source, StageSource::Repo(_)) && copying.is_none() {
            offered = true;
            let (asks, key) = (at.clone(), model::line_flow_key(flow));
            controls = controls.child(Self::stage_control(
                format!("{selector} copy"),
                "Copy flow into yard…",
                palette,
                cx.listener(move |this, _, window, cx| {
                    this.ask_copy_flow(asks.clone(), key.clone(), !drawer, window, cx)
                }),
            ));
        }
        if offered {
            section.push(controls.into_any_element());
        }
        if let Some(copy) = copying.filter(|_| matches!(source, StageSource::Repo(_))) {
            section.push(
                self.stage_copy_block(copy, selector, drawer, palette, cx)
                    .into_any_element(),
            );
        }
        if let Some(saved) = saved {
            let color = match saved.warning.is_empty() {
                true => palette.faint,
                false => palette.warn,
            };
            section.push(
                Self::stage_words(
                    saved.words.clone(),
                    format!("{selector} saved"),
                    color,
                    drawer,
                )
                .into_any_element(),
            );
        }
        (!section.is_empty()).then(|| div().flex().flex_col().gap_1().children(section))
    }

    /// The confirmation of Copy flow into yard…: what moves, what the yard
    /// does not do, the weighty line, the depot's name to type back where
    /// no unlock is offered, why the last try did not go, and Cancel and
    /// Confirm copy.
    fn stage_copy_block(
        &self,
        copy: &StageCopy,
        selector: &str,
        drawer: bool,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let selector = format!("{selector} copy");
        let mut block = div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::stage_words(
                COPY_WORDS,
                format!("{selector} words"),
                palette.faint,
                drawer,
            ))
            .child(Self::stage_words(
                model::weight_line("code", COPY_WEIGHT),
                format!("{selector} weight"),
                palette.text_accent,
                drawer,
            ));
        if !self.unlock_available() {
            block = block.child(self.stage_typeback(
                &copy.typeback,
                typeback_depot_line(&copy.at.depot),
                &selector,
                drawer,
                palette,
                cx,
            ));
        }
        if let Some(said) = &copy.said {
            block = block.child(Self::stage_words(
                said.clone(),
                format!("{selector} said"),
                palette.warn,
                drawer,
            ));
        }
        let label = match copy.sending {
            true => "Copying…",
            false => "Confirm copy",
        };
        let click =
            (!copy.sending).then(|| cx.listener(|this, _, _, cx| this.confirm_copy_flow(cx)));
        block.child(
            div()
                .flex()
                .justify_end()
                .gap_2()
                .child(Self::stage_control(
                    format!("{selector} cancel"),
                    "Cancel",
                    palette,
                    cx.listener(|this, _, window, cx| this.cancel_copy_flow(window, cx)),
                ))
                .child(card_button(
                    format!("{selector} confirm"),
                    label.to_string(),
                    palette,
                    click,
                )),
        )
    }

    /// The name typed back where this window has no gate: what to type and
    /// why, and the field that takes it. The name is no secret: it is the
    /// stage's or the depot's own, drawn as typed.
    fn stage_typeback(
        &self,
        input: &Entity<input::Line>,
        words: String,
        selector: &str,
        drawer: bool,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(Self::stage_words(
                words,
                format!("{selector} typeback words"),
                palette.faint,
                drawer,
            ))
            .child(
                field(input.focus_handle(cx), input.clone(), palette).debug_selector({
                    let selector = format!("{selector} typeback");
                    move || selector.clone()
                }),
            )
    }

    /// The editor under a stage file's section, where it stands over this
    /// stage: while the text is edited, the field over the file as it was
    /// read, what Save… does, why Save is held where the file moved, why
    /// the last save did not go, and Cancel and Save…; in the review, what
    /// changes line by line, the weighty line, the stage's name to type
    /// back where no unlock is offered, and Cancel, Back to edit and
    /// Confirm save. Save and Confirm save are held while the text is blank
    /// or is what stands, while a save is on its way and while the file has
    /// moved under the editor. None where no editor stands over the stage.
    pub(crate) fn stage_file_editor(
        &self,
        at: &StageAt,
        selector: &str,
        drawer: bool,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Option<Div> {
        let edit = self.stage_edit_at(at)?;
        let selector = format!("{selector} editor");
        let proposed = edit.field.read(cx).text().to_string();
        let moved = self.stage_edit_moved(edit);
        let enabled = !edit.sending && !moved && sendable(&edit.over_text, &proposed);
        let mut editor = div().flex().flex_col().gap_2();
        let mut buttons = div()
            .flex()
            .justify_end()
            .gap_2()
            .child(Self::stage_control(
                format!("{selector} cancel"),
                "Cancel",
                palette,
                cx.listener(|this, _, window, cx| this.cancel_stage_edit(window, cx)),
            ));
        match edit.step {
            StageStep::Edit => {
                let over = match edit.over_digest.as_str() {
                    STAGE_FILE_NONE => "No file stands for this stage yet.".to_string(),
                    "" => "Over the file as read.".to_string(),
                    digest => format!(
                        "Over the file as read (digest {}).",
                        model::line_short_digest(digest)
                    ),
                };
                editor = editor
                    .child(
                        field(edit.field.focus_handle(cx), edit.field.clone(), palette)
                            .min_h(rems(8.))
                            .debug_selector({
                                let selector = selector.clone();
                                move || selector.clone()
                            }),
                    )
                    .child(Self::stage_words(
                        format!("{over} {EDITOR_WORDS}"),
                        format!("{selector} words"),
                        palette.faint,
                        drawer,
                    ));
                let click = enabled
                    .then(|| cx.listener(|this, _, window, cx| this.review_stage_edit(window, cx)));
                buttons = buttons.child(card_button(
                    format!("{selector} save"),
                    "Save…".to_string(),
                    palette,
                    click,
                ));
            }
            StageStep::Review => {
                let restores = !proposed.is_empty()
                    && self
                        .stage_read(at)
                        .and_then(|stage| stage.stage_file.as_ref())
                        .is_some_and(|file| file.prior_text == proposed);
                editor = editor
                    .child(
                        div()
                            .text_size(rems(TYPE_LABEL))
                            .font_weight(FontWeight::SEMIBOLD)
                            .debug_selector({
                                let selector = selector.clone();
                                move || format!("{selector} review")
                            })
                            .child("Review change"),
                    )
                    .child(Self::stage_words(
                        match restores {
                            true => format!(
                                "Puts back the text the file stood as before its last save: {}",
                                edit.review.words()
                            ),
                            false => edit.review.words(),
                        },
                        format!("{selector} review count"),
                        palette.faint,
                        drawer,
                    ))
                    .child(self.stage_diff(&edit.review, &selector, palette, cx))
                    .child(Self::stage_words(
                        model::weight_line("code", SAVE_WEIGHT),
                        format!("{selector} review weight"),
                        palette.text_accent,
                        drawer,
                    ));
                if !self.unlock_available() {
                    editor = editor.child(self.stage_typeback(
                        &edit.typeback,
                        typeback_stage_line(&at.stage),
                        &format!("{selector} review"),
                        drawer,
                        palette,
                        cx,
                    ));
                }
                let label = match edit.sending {
                    true => "Saving…",
                    false => "Confirm save",
                };
                let back = !edit.sending;
                buttons = buttons
                    .child(
                        Self::stage_control(
                            format!("{selector} back"),
                            "Back to edit",
                            palette,
                            cx.listener(|this, _, window, cx| this.back_to_stage_edit(window, cx)),
                        )
                        .when(!back, |el| el.opacity(0.5)),
                    )
                    .child(card_button(
                        format!("{selector} confirm"),
                        label.to_string(),
                        palette,
                        enabled.then(|| cx.listener(|this, _, _, cx| this.confirm_stage_edit(cx))),
                    ));
            }
        }
        if moved {
            editor = editor.child(Self::stage_words(
                EDITOR_MOVED,
                format!("{selector} moved"),
                palette.faint,
                drawer,
            ));
        }
        if let Some(said) = &edit.said {
            editor = editor.child(Self::stage_words(
                said.clone(),
                format!("{selector} said"),
                palette.warn,
                drawer,
            ));
        }
        Some(editor.child(buttons))
    }

    /// The change line by line, as the review draws it: lines taken out in
    /// warn behind a `-`, lines put in in ok behind a `+`, the lines
    /// around them as they stand, and a run of lines left out counted. A
    /// last line with no break says so, since it differs from one with it.
    fn stage_diff(
        &self,
        review: &Reviewed,
        selector: &str,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let mono = select::mono(cx);
        let mut rows = div()
            .flex()
            .flex_col()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(palette.line)
            .bg(palette.raised)
            .text_size(rems(TYPE_CODE))
            .debug_selector({
                let selector = format!("{selector} review diff");
                move || selector.clone()
            });
        for (n, row) in review.rows.iter().cloned().enumerate() {
            let (kind, mark, color, text) = match row {
                DiffRow::Folded(count) => {
                    let words = match count {
                        1 => "1 unchanged line".to_string(),
                        count => format!("{count} unchanged lines"),
                    };
                    rows = rows.child(
                        div()
                            .px_2()
                            .text_color(palette.faint)
                            .debug_selector({
                                let selector = format!("{selector} review folded {n}");
                                move || selector.clone()
                            })
                            .child(words),
                    );
                    continue;
                }
                DiffRow::Line(LineDiff::Same(text)) => ("same", " ", palette.faint, text),
                DiffRow::Line(LineDiff::Removed(text)) => ("removed", "-", palette.warn, text),
                DiffRow::Line(LineDiff::Added(text)) => ("added", "+", palette.ok, text),
            };
            let (body, ended) = match text.strip_suffix('\n') {
                Some(body) => (body.to_string(), true),
                None => (text, false),
            };
            let body = match (body.is_empty(), ended) {
                (true, true) => " ".to_string(),
                (_, true) => body,
                (_, false) => format!("{body}  (no line break at the end)"),
            };
            let mut line = div()
                .flex()
                .flex_row()
                .gap_2()
                .px_2()
                .text_color(color)
                .debug_selector({
                    let selector = format!("{selector} review {kind} {n}");
                    move || selector.clone()
                });
            if let Some(mono) = &mono {
                line = line.font_family(mono.clone());
            }
            rows = rows.child(
                line.child(div().flex_none().w(px(10.)).child(mark))
                    .child(div().flex_1().min_w_0().child(body)),
            );
        }
        rows
    }
}
