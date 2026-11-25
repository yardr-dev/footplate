//! The decisions' views: the decisions list with its decided history,
//! each open ask's card with its secret, run and setup blocks, and what
//! the cards do: answer, approve, give, decline, put aside.

use std::collections::HashMap;
use std::path::Path;

use gpui::{
    AnyElement, ClipboardItem, Context, Div, Entity, Focusable, FontWeight, MouseButton, ObjectFit,
    SharedString, Window, div, img, prelude::*, px, relative, rems, rgba,
};

use super::Expanded;
use super::Footplate;
use super::ask::{
    ANSWER_WORDS, APPROVE_ELSEWHERE, Approval, DECLINE_IS_NO_PICK, RECOMMENDED, RUN_PROPOSED,
    RunText, SEND_WORDS, SETUP_PROPOSED, SETUP_TAKE, TEAMMATE_CORRECTION, Teammate, answer_label,
    approval, approve_button, approve_move_label, ask_line, asked_age, asks_on_you,
    candidate_choice_line, card_head, carried_texts, decided_keys, decision_head, description_line,
    fenced, fold_head, give_label, give_unlock_label, is_device_pair_run, lever_plate, lever_words,
    move_button, pair_door, recommended_line, run_keys, run_runs, run_texts, secret_goes,
    secret_keys, section_head, send_button, setup_keys, setup_runs, show_button, take_back_label,
    teammate_email, teammate_kind, teammate_of, teammate_role, teammate_user, texts_label,
    typeback_line, typeback_mismatch, words_send,
};
use super::expanded_text_key;
use super::keys::{APPROVE_KEY, SETUP_SECRET_CONTEXT};
use super::metrics::{
    ASK_QUESTION_MIN, LEVER_H, RADIUS_CARD, RADIUS_CONTROL, ROW_ASK_CHARS, ROW_LAMP, ROW_LAMP_GAP,
    ROW_MARK, ROW_TITLE_CHARS, STAGE_TIE, THREAD_PIC_H, THREAD_PIC_W,
};
use super::rail::{Picture, RailTip, UnsentNote, post_note};
use super::state::{Adding, ApprovalPairing};
use super::ui::{field, keys_row, outcome_glyph, send_held, shorten, small_button};
use super::words::{bead_words, decide_words, decided_words, left_words};
use super::{AskAbout, Decline, Expand, FocusFilter, FocusWords};
use crate::api::{self};
use crate::draft::{self, Attachment};
use crate::input::Line;
use crate::model::{
    self, AnswerAction, DecidedEntry, DecidedFold, DecidedRow, DecisionAsk, DecisionDetail,
    DecisionRow, Effect, Mode, Model, Outcome, Selected, Sending, Taken, Target, Width, Write,
};
use crate::select::{self};
use crate::theme::{
    Palette, TYPE_BODY, TYPE_CODE, TYPE_LABEL, TYPE_META, TYPE_QUESTION, TYPE_TITLE, yard_color,
};
use crate::unlock;

impl Footplate {
    /// The line says whom it writes to, or what words on a card do.
    pub(crate) fn shown(&mut self, cx: &mut Context<Self>) {
        let to = match self.model.as_ref().and_then(|m| m.selected().cloned()) {
            // In the row of a user a name of a linked yard is trusted as,
            // that is the name when it wrote last.
            // Before all of such a row is read, who wrote last is not
            // known, and the line names nobody.
            Some(Selected::Conversation(name)) => {
                match self.model.as_ref().and_then(|m| m.target()) {
                    _ if self.model.as_ref().is_some_and(Model::reading) => {
                        format!("Reading {name}")
                    }
                    Some(Target::Peer { name, peer }) => format!("Write to {name}@{peer}"),
                    _ => format!("Write to {name}"),
                }
            }
            Some(Selected::Decision(_)) => {
                let asking = self
                    .model
                    .as_ref()
                    .and_then(|m| m.current_ask())
                    .is_some_and(|current| Some(&current) == self.declining.as_ref());
                if asking {
                    "Why decline? Enter sends, esc leaves".to_string()
                } else if self.current_options_empty() {
                    "Words wanted: Enter sends the answer (i)".to_string()
                } else if self.current_foreign() {
                    // The yard refuses a plain note on another yard's
                    // bead: words go with a pick, and talking to the asker
                    // is "ask about this" below.
                    "Words with the pick (i)".to_string()
                } else {
                    "Words with the pick, or alone as a note (i)".to_string()
                }
            }
            Some(Selected::Decided(_, _)) => "Words with the pick".to_string(),
            Some(Selected::Waiting(_)) => "Note with the move (i)".to_string(),
            Some(Selected::Bead(_)) if self.waiting_card().is_some() => {
                "Note with the move (i)".to_string()
            }
            _ => String::new(),
        };
        self.line
            .update(cx, |line, cx| line.set_placeholder(&to, cx));
        // What is typed of a secret is for the ask it was typed on, and is
        // kept for no other and for no return to this one. The toggle
        // stands as the ask says: shown where the yard marks its value
        // no secret, hidden elsewhere.
        let ask = self.open_ask();
        if ask != self.secret_for {
            self.forget_secret(cx);
            self.secret_for = ask.clone();
            self.secret_left += 1;
            let shown = ask.as_ref().is_some_and(|(bead, ask)| {
                self.model
                    .as_ref()
                    .is_some_and(|m| !m.concealed(bead, *ask))
            });
            self.set_secret_shown(shown, cx);
            // The secret shots pose a value in the field, hidden and
            // shown: a debug build's alone, as the settings and the
            // draft of Footplate::opened. The harness-empty shot poses
            // none, so that the field says what it takes.
            #[cfg(debug_assertions)]
            if matches!(
                std::env::var("FOOTPLATE_SHOT").as_deref(),
                Ok("secret" | "secret-shown" | "secret-harness")
            ) {
                self.secret
                    .update(cx, |field, cx| field.set_text("bk-live-3f9a-7Qx", cx));
            }
            // The refused shot poses the yard's refusal of the answer
            // on the card, the field empty as a refused giving leaves
            // it: a debug build's alone, as above.
            #[cfg(debug_assertions)]
            if std::env::var("FOOTPLATE_SHOT").as_deref() == Ok("secret-refused") {
                let refused = api::Error::Refused {
                    code: "error".to_string(),
                    message: "ask 7 asks for BOOKS_TOKEN: BOOKS_TOKEN was not given".to_string(),
                };
                self.refused.insert(
                    ("bead-s".to_string(), 7),
                    model::not_given(&refused, "bead-s", 7),
                );
            }
        }
        // A bead ID and ask number can recur in another yard. Keep the
        // typed number only for the session and ask it was typed on.
        let typeback_for = ask
            .as_ref()
            .map(|(bead, ask)| (self.session, bead.clone(), *ask));
        if typeback_for != self.typeback_for {
            self.typeback.update(cx, |field, cx| field.set_text("", cx));
            self.typeback_for = typeback_for;
        }
        // What is typed of a host is for the ask it was typed on, and is
        // kept for no other and for no return to this one, as a secret's
        // is. The field names the address it fills, as the web card does.
        // A bead and ask number can recur in another yard. Keep the host
        // only for the session and ask it was typed on.
        let host_for = ask
            .as_ref()
            .map(|(bead, ask)| (self.session, bead.clone(), *ask));
        if host_for != self.host_for {
            self.host.update(cx, |field, cx| field.set_text("", cx));
            self.host_for = host_for;
            // The setup-host shot poses a host in the field: a debug
            // build's alone, as the secret shots pose a value.
            #[cfg(debug_assertions)]
            if std::env::var("FOOTPLATE_SHOT").as_deref() == Ok("setup-host") {
                self.host
                    .update(cx, |field, cx| field.set_text("courier.example.net", cx));
            }
        }
        let hosted = ask.as_ref().and_then(|(bead, ask)| {
            self.model
                .as_ref()
                .and_then(|m| m.setup_approving(bead, *ask))
                .map(|setup| setup.host_template.clone())
                .filter(|template| !template.is_empty())
        });
        let hosted = hosted.map(|template| format!("Host of a server of your own: {template}"));
        self.host.update(cx, |field, cx| {
            field.set_placeholder(hosted.as_deref().unwrap_or_default(), cx)
        });
        // A pairing approval's sitting is for the card it was approved
        // on, and is kept for no other and for no return to this one:
        // leaving the screen drops the transient, and the ended state
        // with it.
        let pairing_card = self.approval_pairing.as_ref().map(ApprovalPairing::card);
        if pairing_card.is_some() && pairing_card != ask {
            self.approval_pairing = None;
            self.selection
                .update(cx, |selection, cx| selection.clear(cx));
        }
        // A secret ask's card draws no line for words, nor does the card
        // of an ask taken back. A focus left in
        // the line by the conversation before it would hold the keys where
        // nothing is drawn to take them: the window has them, and `i` or a
        // click types in the field. Never the field at once: what was being
        // typed for a conversation must not go on as a secret.
        // Nor does a run ask's, but for the words of a decline.
        let no_line = ask.as_ref().is_some_and(|open| {
            let (bead, ask) = open;
            let model = self.model.as_ref();
            model.is_some_and(|m| {
                m.secret(bead, *ask).is_some()
                    || m.taken_back(bead, *ask)
                    || ((m.run(bead, *ask).is_some() || m.setup(bead, *ask).is_some())
                        && self.declining.as_ref() != Some(open))
            })
        });
        if no_line && !self.settings_open {
            let (handle, line, focus) =
                (self.handle, self.line.focus_handle(cx), self.focus.clone());
            // Afterwards: the window is not to be had while a key or click
            // that came through it is still on its way.
            cx.defer(move |cx| {
                // A window that is closed has no focus to move.
                let _ = handle.update(cx, |_, window, _| {
                    if line.is_focused(window) {
                        window.focus(&focus);
                    }
                });
            });
        }
        // The number field stands on a weighty card with no gate alone. A
        // focus left in it by a card that approved since, or by another
        // card, would hold the keys where nothing is drawn to take them:
        // the window has them.
        let typebacked = !self.settings_open
            && ask.as_ref().is_some_and(|(bead, ask)| {
                matches!(self.card_approval(bead, *ask), Approval::TypeBack)
            });
        if !typebacked {
            let (handle, typeback, focus) = (
                self.handle,
                self.typeback.focus_handle(cx),
                self.focus.clone(),
            );
            // Afterwards, as above.
            cx.defer(move |cx| {
                // A window that is closed has no focus to move.
                let _ = handle.update(cx, |_, window, _| {
                    if typeback.is_focused(window) {
                        window.focus(&focus);
                    }
                });
            });
        }
        let wanted = self
            .giving()
            .map(|secret| {
                let at = self.secret_typed.len();
                let name = secret.env.get(at).cloned().unwrap_or_default();
                let of = match secret.env.len() {
                    0 | 1 => String::new(),
                    n => format!(" ({} of {n})", at + 1),
                };
                // What the value is for, in the order `secret_goes` tests
                // the kinds. The yard sends no source on a harness key's
                // ask or a waiting login's input, so each names its own:
                // the harness as the yard writes it, since the card's line
                // above has its label and a table of labels here would
                // fall behind every kind the yard adds.
                if !secret.harness.is_empty() {
                    format!("{name} for the {} key{of}", secret.harness)
                } else if !secret.login.is_empty() {
                    format!("{name} for the waiting login {}{of}", secret.login)
                } else {
                    format!("{name} for the source {}{of}", secret.source)
                }
            })
            .or_else(|| {
                self.setup_account().map(|(names, pack)| {
                    let at = self.secret_typed.len();
                    match names.get(at) {
                        Some(name) => {
                            let of = match names.len() {
                                0 | 1 => String::new(),
                                n => format!(" ({} of {n})", at + 1),
                            };
                            format!("{name} for the setup of {pack}{of}")
                        }
                        None => format!("typed — {APPROVE_KEY} approves"),
                    }
                })
            });
        self.secret.update(cx, |field, cx| {
            field.set_placeholder(&wanted.unwrap_or_default(), cx)
        });
    }

    /// What the open card's field takes, when its ask is a secret ask that
    /// can be given.
    pub(crate) fn giving(&self) -> Option<api::Secret> {
        let (bead, ask) = self.open_ask()?;
        self.model.as_ref()?.giving(&bead, ask).cloned()
    }

    /// The command the control on the open card approves, when its ask is
    /// a run ask that can be approved.
    pub(crate) fn approvable(&self) -> Option<api::Run> {
        let (bead, ask) = self.open_ask()?;
        self.model.as_ref()?.approving(&bead, ask).cloned()
    }

    /// Whether the open card is a run ask's, open or dealt with.
    pub(crate) fn run_card(&self) -> bool {
        self.open_ask().is_some_and(|(bead, ask)| {
            let model = self.model.as_ref();
            model.is_some_and(|m| m.run(&bead, ask).is_some())
        })
    }

    /// The setup the control on the open card approves, when its ask is
    /// a setup ask that can be approved.
    pub(crate) fn setup_approvable(&self) -> Option<api::Setup> {
        let (bead, ask) = self.open_ask()?;
        self.model.as_ref()?.setup_approving(&bead, ask).cloned()
    }

    /// What the open card's field takes beside the preset pick, when its
    /// ask is a setup ask that can be approved: the account names the
    /// answer types — the missing ones where the update newly needs
    /// them — with the pack they set up.
    pub(crate) fn setup_account(&self) -> Option<(Vec<String>, String)> {
        let setup = self.setup_approvable()?;
        Some((model::setup_typed_names(&setup).to_vec(), setup.pack))
    }

    /// Nothing typed of a secret stays in the window: not in the field, and
    /// not among the values of an ask of several names.
    pub(crate) fn forget_secret(&mut self, cx: &mut Context<Self>) {
        self.secret_typed.clear();
        self.secret.update(cx, |field, cx| field.set_text("", cx));
    }

    /// Nothing typed of a host stays in the window once its approval
    /// went: the card leaves with it.
    pub(crate) fn forget_host(&mut self, cx: &mut Context<Self>) {
        self.host.update(cx, |field, cx| field.set_text("", cx));
    }

    /// Draws the secret as typed, for a look, or as marks again: the
    /// card's toggle, for the ask the field is for.
    pub(crate) fn set_secret_shown(&mut self, shown: bool, cx: &mut Context<Self>) {
        self.secret_shown = shown;
        self.secret
            .update(cx, |field, cx| field.set_shown(shown, cx));
    }

    /// Whether the open card's bead is another yard's question.
    pub(crate) fn current_foreign(&self) -> bool {
        self.model
            .as_ref()
            .and_then(|m| m.current_ask())
            .is_some_and(|(bead, _)| model::foreign_bead(&bead))
    }

    /// Whether the open card's current ask wants words: it has no options.
    pub(crate) fn current_options_empty(&self) -> bool {
        let Some(model) = self.model.as_ref() else {
            return false;
        };
        let now = chrono::Utc::now();
        match model.selected() {
            Some(Selected::Decision(bead)) => model
                .decision_detail(bead, &now)
                .and_then(|d| d.asks.into_iter().find(|a| a.current))
                .is_some_and(|a| a.options.is_empty()),
            Some(Selected::Decided(bead, ask)) => model
                .decided_detail(bead, *ask, &chrono::Local)
                .is_some_and(|d| d.options.is_empty()),
            _ => false,
        }
    }

    /// Reads the open yard's queue again when its next put-aside ask comes
    /// back ([`Footplate::schedule_return_in`]).
    pub(crate) fn schedule_return(&mut self, cx: &mut Context<Self>) {
        self.schedule_return_in(self.session, cx);
    }

    /// Reads the queue of the following `session` again when the next
    /// put-aside ask comes back, which arrives with no event: of a yard
    /// that is held as of the open one, since what waits in it is told of
    /// and counted. One timer waits at a time for a yard; a queue read
    /// since replaces it. The return is found in the history the model
    /// holds (seeded by the read that follows the state), and read open
    /// alone: an ask that is back is open, so the history is not asked for
    /// again. The queue is read once for a return: a yard that
    /// still lists the ask as put aside after its time is not asked again
    /// for it, and what the stream brings says when it is back. The parked
    /// beads are read with it: an ask that is back takes its bead out of
    /// them, and tells no event of its own.
    pub(crate) fn schedule_return_in(&mut self, session: u64, cx: &mut Context<Self>) {
        let held = self.held.values_mut().find(|h| h.session == session);
        let (model, asked, read) = match held {
            _ if session == self.session => (
                self.model.as_ref(),
                &mut self.ask_generation,
                &mut self.returns_read,
            ),
            Some(held) => (
                held.model.as_ref(),
                &mut held.ask_generation,
                &mut held.returns_read,
            ),
            None => return,
        };
        *asked += 1;
        let generation = *asked;
        let Some(at) = model.and_then(|m| m.next_return(*read)) else {
            return;
        };
        let now = chrono::Utc::now();
        let wait = at.timestamp() - now.timestamp();
        if wait <= 0 {
            // Every return up to now is read by this one read.
            *read = Some(at.max(now));
            self.change_in(session, cx, |_| {
                vec![Effect::Asks(Width::Open), Effect::Waiting]
            });
            return;
        }
        let wait = std::time::Duration::from_secs(wait as u64);
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(wait).await;
            this.update(cx, |this, cx| {
                let held = this.held.values_mut().find(|h| h.session == session);
                let (asked, read) = match held {
                    _ if session == this.session => (this.ask_generation, &mut this.returns_read),
                    Some(held) => (held.ask_generation, &mut held.returns_read),
                    None => return,
                };
                if asked == generation {
                    *read = Some(at);
                    this.change_in(session, cx, |_| {
                        vec![Effect::Asks(Width::Open), Effect::Waiting]
                    });
                }
            })
            .ok();
        })
        .detach();
    }

    /// Reads the pairing of the following `session` again when the open
    /// one's code runs out: the yard ends a pairing whose time ran out on
    /// whoever looks first, so the read flips the pane to no pairing with
    /// Open again. Nothing waits where none is open, where its time cannot
    /// be read, or where it ran out already. A timer whose pairing is gone
    /// by then, or is another's (ended, opened again), reads nothing: each
    /// read of a pairing waits its own, and only its own reads.
    pub(crate) fn schedule_pairing_end_in(&mut self, session: u64, cx: &mut Context<Self>) {
        let Some((code, wait)) = self.model_in(session).and_then(|m| {
            let pairing = m.pairing()?;
            let ends: chrono::DateTime<chrono::Utc> = pairing.expires.parse().ok()?;
            let wait = ends.timestamp() - chrono::Utc::now().timestamp();
            (wait > 0).then(|| (pairing.code.clone(), wait as u64 + 5))
        }) else {
            return;
        };
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(wait))
                .await;
            this.update(cx, |this, cx| {
                let held = this
                    .model_in(session)
                    .and_then(|m| m.pairing())
                    .is_some_and(|pairing| pairing.code == code);
                if held {
                    this.run_in(session, vec![Effect::Pairing], cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// Says why the ask's answer did not go on its card, in the yard of
    /// the following `session`: on the open one's card at once, and on
    /// one that is held when the window shows it. A yard no longer
    /// followed has no card to say it on.
    pub(crate) fn refused_in(
        &mut self,
        session: u64,
        at: (String, u64),
        why: String,
        cx: &mut Context<Self>,
    ) {
        if session == self.session {
            self.refused.insert(at, why);
        } else if let Some(held) = self.held.values_mut().find(|h| h.session == session) {
            held.refused.insert(at, why);
        }
        cx.notify();
    }

    /// Words for the ask `at` of `yard` that the yard did not take are
    /// back with what was typed since: in the line while that yard is
    /// still the open one, as it was opened then, which is answered, and
    /// else with the ask they were written for, in its own yard. Why
    /// they are back is the caller's to say.
    pub(crate) fn words_back(
        &mut self,
        (yard, opening): (&str, u64),
        at: (String, u64),
        words: String,
        cx: &mut Context<Self>,
    ) -> bool {
        if opening == self.opening {
            self.line.update(cx, |line, cx| {
                let back = draft::rejoin(words, line.text().to_string());
                line.set_text(&back, cx);
            });
            return true;
        }
        let kept = match yard == self.yard {
            true => &mut self.ask_words,
            false => self.kept_words.entry(yard.to_string()).or_default(),
        };
        let typed = kept.remove(&at).unwrap_or_default();
        kept.insert(at, draft::rejoin(words, typed));
        false
    }

    /// The open card's ask: the current one of a decision, or the dealt
    /// with itself.
    pub(crate) fn card(&self) -> Option<(String, u64)> {
        // The settings stand where the card does, and the Feedback form
        // over it: its keys are not for a card that is not in view, and
        // the arrows in the form's rows are the form's.
        if self.settings_open || self.feedback.is_some() {
            return None;
        }
        self.open_ask()
    }

    /// The ask of the card that is open, in view or under the settings.
    pub(crate) fn open_ask(&self) -> Option<(String, u64)> {
        match self.model.as_ref()?.selected()? {
            Selected::Decision(_) => self.model.as_ref()?.current_ask(),
            Selected::Decided(bead, ask) => Some((bead.clone(), *ask)),
            _ => None,
        }
    }

    /// Whether the system unlock stands in front of a weighty approval:
    /// where the system offers its gate, or where a test fixed it.
    pub(crate) fn unlock_available(&self) -> bool {
        self.test_unlock
            .map(|(offered, _)| offered)
            .unwrap_or_else(unlock::available)
    }

    /// What the system unlock answers where a test fixed it: the gate is
    /// then answered at once, with no prompt. None asks the system.
    fn unlock_answer(&self) -> Option<bool> {
        self.test_unlock.map(|(_, answer)| answer)
    }

    /// How the ask's card approves: straight to the yard, through the
    /// system unlock first, after the ask's number is typed back, or
    /// nowhere from this window.
    pub(crate) fn card_approval(&self, bead: &str, ask: u64) -> Approval {
        let weighty = self
            .model
            .as_ref()
            .is_some_and(|m| !m.weight(bead, ask).is_empty());
        let elsewhere = self.approve_elsewhere.contains(&(bead.to_string(), ask));
        approval(weighty, self.unlock_available(), elsewhere)
    }

    /// Where the ask's card stands with the yard for a write, while it is
    /// with it: the card offers no second write of any kind (an answer, a
    /// note, Later, a decline, a give, an approval) until the yard says what
    /// came of it.
    pub(crate) fn write_sending(&self, bead: &str, ask: u64) -> Option<Sending> {
        self.model.as_ref()?.sending(bead, ask)
    }

    /// What the card says while a write of it is with the yard, faint:
    /// nothing in it is a failure, which stands in warn where the yard
    /// refused, or has not recorded what it was sent. None where no write
    /// is with it.
    pub(crate) fn sending_words(&self, bead: &str, ask: u64, palette: Palette) -> Option<Div> {
        let (write, sending) = self.model.as_ref()?.writing(bead, ask)?;
        Some(
            div()
                .debug_selector(|| format!("{} {} {bead} {ask}", write.name(), sending.name()))
                .text_xs()
                .text_color(palette.faint)
                .child(write.words(sending)),
        )
    }

    /// Whether the number field holds exactly the ask's number: the gate
    /// of a weighty card where no unlock is offered. What is pasted often
    /// ends in a line break, which is trimmed as a value's is; anything
    /// else but the number is no number typed back, as at a terminal.
    pub(crate) fn typeback_ok(&self, ask: u64, cx: &gpui::App) -> bool {
        self.typeback.read(cx).text().trim() == ask.to_string()
    }

    /// The typed-back number is used up: each weighty card asks again, as
    /// the system unlock does.
    pub(crate) fn clear_typeback(&mut self, cx: &mut Context<Self>) {
        self.typeback.update(cx, |field, cx| field.set_text("", cx));
    }

    /// The approval of the following `session` is no longer with the
    /// gate: its control sends again.
    fn clear_approving_in(&mut self, session: u64) {
        if session == self.session {
            self.approving = None;
        } else if let Some(held) = self.held.values_mut().find(|h| h.session == session) {
            held.approving = None;
        }
    }

    /// Opens the system unlock for a weighty approval or giving: `send`
    /// goes where the person passes it, with `weighed`, and `cancel` puts
    /// the card back where they do not, sending nothing. The approval is
    /// the yard's of `session`: what came of it is that yard's, and is
    /// said there, wherever the window is by then.
    pub(crate) fn unlock_then(
        &mut self,
        session: u64,
        reason: String,
        cx: &mut Context<Self>,
        send: impl FnOnce(&mut Self, u64, &mut Context<Self>) + 'static,
        cancel: impl FnOnce(&mut Self, u64, &mut Context<Self>) + 'static,
    ) {
        // A test fixed the gate: answered at once, with no prompt.
        if let Some(passed) = self.unlock_answer() {
            return match passed {
                true => send(self, session, cx),
                false => cancel(self, session, cx),
            };
        }
        let asked = unlock::evaluate(&reason);
        cx.spawn(async move |this, cx| {
            let passed = asked.recv().await.unwrap_or(false);
            this.update(cx, |this, cx| match passed {
                true => send(this, session, cx),
                false => cancel(this, session, cx),
            })
            .ok();
        })
        .detach();
    }

    /// Enter in the field of a secret ask: the value goes to the yard in
    /// its own call and in nothing else. The field is emptied here, before
    /// the yard is asked and whatever it answers, and what the window says
    /// afterwards is the yard's words or footplate's, on the card, never
    /// the value. An ask of several names takes them one after the other
    /// and sends them together. Enter in the field of a setup ask takes
    /// one account name the same way, and the approval sends them with
    /// the preset pick.
    pub(crate) fn give(&mut self, cx: &mut Context<Self>) {
        // Held as a conversation's sends are: the value stays in the
        // field, with the reason beside Give.
        if self.reconnecting() {
            return;
        }
        let Some((bead, ask)) = self.card() else {
            return;
        };
        // A secret ask's names go in their own call; a setup ask's wait
        // for the approval, which sends them with the preset pick.
        let secret = self.giving();
        let setup = self.setup_account();
        // A value on its way to the yard, or not known to have reached it,
        // is given again only when the card says the yard has not recorded
        // it: what is typed stays where it is, in the field.
        if secret.is_some() && self.write_sending(&bead, ask).is_some() {
            return;
        }
        let names = match (&secret, &setup) {
            (Some(secret), _) => secret.env.clone(),
            (None, Some((names, _))) => names.clone(),
            (None, None) => return,
        };
        // A weighty secret with no gate takes its number first, as a
        // terminal asks it before the values: with anything but the ask's
        // number the value stays where it is and nothing is sent. A setup
        // ask's names only wait here; the approval asks the number back.
        if secret.is_some()
            && matches!(self.card_approval(&bead, ask), Approval::TypeBack)
            && !self.typeback_ok(ask, cx)
        {
            return self.notice(typeback_mismatch(ask), cx);
        }
        // What is pasted often ends in a line break, which nobody sees
        // among the marks and which the yard refuses at either end.
        let value = self
            .secret
            .update(cx, |field, cx| field.take_text(cx))
            .trim()
            .to_string();
        let Some(name) = names.get(self.secret_typed.len()) else {
            return self.forget_secret(cx);
        };
        if value.is_empty() {
            return self.notice(format!("nothing is typed for {name}"), cx);
        }
        self.secret_typed.push(value);
        self.notice(String::new(), cx);
        self.refused.remove(&(bead.clone(), ask));
        if self.secret_typed.len() < names.len() {
            return self.shown(cx);
        }
        if secret.is_none() {
            return self.shown(cx);
        }
        let secret = secret.unwrap();
        let reason = unlock::give_reason(&secret.env, &self.yard);
        let values: Vec<(String, String)> = secret
            .env
            .into_iter()
            .zip(std::mem::take(&mut self.secret_typed))
            .collect();
        self.shown(cx);
        let session = self.session;
        match self.card_approval(&bead, ask) {
            // Nowhere: the card shows where to give instead, and offers
            // no control. What was typed is kept, as what was typed since.
            Approval::Nowhere => {
                self.secret_typed = values.into_iter().map(|(_, value)| value).collect();
                self.shown(cx);
            }
            // The number is typed back: the gate above asked it before
            // the values, so the giving sends weighed, and the number is
            // used up.
            Approval::TypeBack => {
                self.clear_typeback(cx);
                self.send_give_in(session, bead, ask, values, true, cx);
            }
            // The system unlock first: its cancel sends nothing, and what
            // was typed is kept for another try: the last value back in
            // the field, the rest typed, so Enter tries again. Where the
            // window left the yard meanwhile, what was typed is dropped
            // instead: it is that yard's, and is not restored into
            // another's field.
            Approval::Unlock => {
                let back: Vec<String> = values.iter().map(|(_, value)| value.clone()).collect();
                self.unlock_then(
                    session,
                    reason,
                    cx,
                    move |this, session, cx| {
                        this.send_give_in(session, bead, ask, values, true, cx)
                    },
                    move |this, session, cx| {
                        if session == this.session {
                            let mut back: Vec<String> = back;
                            if let Some(last) = back.pop() {
                                this.secret
                                    .update(cx, |field, cx| field.set_text(&last, cx));
                            }
                            this.secret_typed = back;
                        }
                        this.shown(cx);
                    },
                );
            }
            Approval::Direct => self.send_give_in(session, bead, ask, values, false, cx),
        }
    }

    /// Gives the typed values to the yard in its own call and in nothing
    /// else. What the window says afterwards is the yard's words or
    /// footplate's, on the card, never the value. `weighed` goes where
    /// the person passed the system unlock for a weighty card, or typed
    /// its number back where no unlock is offered; it is this give's alone,
    /// and a give sent again passes the gate again. While the call is out
    /// the card says the value is with the yard, and offers no second give.
    /// A call that ended with no answer may have stored the value, so the
    /// card waits while the queue is read, and the values come back to the
    /// field they were typed in ([`Footplate::give_kept`]), never to the
    /// mail line and never sent again of themselves: the card ends as given
    /// where the queue says so, and says the yard has not recorded it where
    /// it still lists the ask open, which a yard that wrote the value to its
    /// file but not the note that records it also shows (a give sent again
    /// writes the same bytes).
    pub(crate) fn send_give_in(
        &mut self,
        session: u64,
        bead: String,
        ask: u64,
        values: Vec<(String, String)>,
        weighed: bool,
        cx: &mut Context<Self>,
    ) {
        let call_bead = bead.clone();
        let at = (bead.clone(), ask);
        let left = self.secret_left;
        let names: Vec<String> = values.iter().map(|(name, _)| name.clone()).collect();
        self.write_out_in(
            session,
            &at,
            Write::Give {
                names: names.clone(),
            },
            cx,
        );
        // What the yard answers is of the yard it was given in, and is
        // said there, wherever the window is by then.
        self.ask_in(
            session,
            cx,
            move |client, me| {
                client
                    .give_secret(&call_bead, ask, &values, me, weighed)
                    .map_err(|err| {
                        let lost = model::answer_lost(&err);
                        let why = model::not_given(&err, &call_bead, ask);
                        // The yard repeats no value in a refusal. Should
                        // anything on the way do so, it is not shown.
                        let why = if values.iter().any(|(_, value)| why.contains(value.as_str())) {
                            "not given: the yard refused it".to_string()
                        } else {
                            why
                        };
                        // The values are handed back only where the yard
                        // may have taken them: a refusal leaves the field
                        // empty, as it always did.
                        let values = if lost { values } else { Vec::new() };
                        GiveStopped { why, lost, values }
                    })
            },
            move |this, given, cx| match given {
                Ok(given) => {
                    this.change_in(session, cx, |m| m.gave(&bead, ask, &given.given));
                    this.schedule_return_in(session, cx);
                    this.notice_in(session, model::given_line(&given), cx);
                }
                Err(stopped) => {
                    if stopped.lost {
                        this.give_kept(session, &at, left, stopped.values, cx);
                    }
                    let write = Write::Give { names };
                    let (lost, why) = (stopped.lost, stopped.why);
                    this.write_failed_in(session, at, write, lost, String::new(), why, cx);
                }
            },
        );
    }

    /// The values of a give whose call got no answer come back to the field
    /// of the card they were typed on, as the system unlock's cancel puts
    /// them: the last in the field, the rest typed, so that Enter gives
    /// them again once the card says the yard has not recorded them. Only
    /// there, and only where the card is still the open one of the yard the
    /// value was given in, `left` being how many times a card was left when
    /// it went: leaving the card forgets a secret, and none is kept for a
    /// return, not even to a card opened again before the call ended. A
    /// field typed in since has the person's own words, which are not
    /// overwritten: the values are dropped instead.
    fn give_kept(
        &mut self,
        session: u64,
        at: &(String, u64),
        left: u64,
        values: Vec<(String, String)>,
        cx: &mut Context<Self>,
    ) {
        if session != self.session
            || self.secret_left != left
            || self.secret_for.as_ref() != Some(at)
            || !self.secret_typed.is_empty()
            || !self.secret.read(cx).text().is_empty()
        {
            return;
        }
        let mut back: Vec<String> = values.into_iter().map(|(_, value)| value).collect();
        if let Some(last) = back.pop() {
            self.secret
                .update(cx, |field, cx| field.set_text(&last, cx));
        }
        self.secret_typed = back;
        self.shown(cx);
    }

    /// Approves the open card's run or setup ask: its key and its button.
    /// A run ask's approval names the ask and nothing of the command,
    /// which the yard runs as it stored it; a setup ask's the preset pick
    /// with the account together. A weighty card's approval passes the
    /// system unlock first, or the typed-back number where no unlock is
    /// offered, and a card that approves nowhere has no control: its card
    /// says where to approve instead.
    pub(crate) fn approve(&mut self, cx: &mut Context<Self>) {
        let Some((bead, ask)) = self.card() else {
            return;
        };
        let at = (bead.clone(), ask);
        if self.approve_elsewhere.contains(&at)
            || self.approving.is_some()
            || self.write_sending(&bead, ask).is_some()
            || self.client.is_none()
            || self.reconnecting()
            || matches!(self.card_approval(&bead, ask), Approval::Nowhere)
        {
            return;
        }
        if let Some(run) = self.approvable() {
            match self.card_approval(&bead, ask) {
                Approval::Direct => self.send_approve(bead, ask, cx),
                // The number field holds the ask's number or the approval
                // sends nothing: the card stays as it is, and says so.
                Approval::TypeBack => {
                    if !self.typeback_ok(ask, cx) {
                        return self.notice(typeback_mismatch(ask), cx);
                    }
                    self.clear_typeback(cx);
                    self.send_approve_in(self.session, bead, ask, true, cx);
                }
                // The system unlock first: its cancel sends nothing, and
                // the control sends again. Nowhere is turned away above.
                Approval::Unlock => {
                    let reason = unlock::approve_reason(&run.line(), &self.yard);
                    let session = self.session;
                    self.declining = None;
                    self.approving = Some(at.clone());
                    self.shown(cx);
                    self.notice(String::new(), cx);
                    self.refused.remove(&at);
                    self.unlock_then(
                        session,
                        reason,
                        cx,
                        move |this, session, cx| this.send_approve_in(session, bead, ask, true, cx),
                        move |this, session, cx| {
                            this.clear_approving_in(session);
                            this.shown(cx);
                        },
                    );
                }
                Approval::Nowhere => {}
            }
        } else if self.setup_approvable().is_some() {
            self.approve_setup(bead, ask, cx);
        }
    }

    /// Approves the open card's run ask. What the window says afterwards
    /// is the yard's: the command as it ran with what it printed, or the
    /// refusal whole, on the card. While the call is out the card says it
    /// is with the yard. A call that ended without an answer is never said
    /// to be approved or refused: the card waits while the queue is read
    /// for what the yard did, and ends as approved, or says the yard has
    /// not recorded it ([`Footplate::write_failed_in`]). A yard without
    /// the call for this window has its card say where to approve.
    pub(crate) fn send_approve(&mut self, bead: String, ask: u64, cx: &mut Context<Self>) {
        self.send_approve_in(self.session, bead, ask, false, cx);
    }

    /// Approves a run ask of the following `session`. `weighed` goes
    /// where the person passed the system unlock for a weighty card, or
    /// typed its number back where no unlock is offered.
    fn send_approve_in(
        &mut self,
        session: u64,
        bead: String,
        ask: u64,
        weighed: bool,
        cx: &mut Context<Self>,
    ) {
        let at = (bead.clone(), ask);
        if session == self.session {
            self.declining = None;
            self.approving = Some(at.clone());
            self.shown(cx);
            self.notice(String::new(), cx);
            self.refused.remove(&at);
        } else {
            let Some(held) = self.held.values_mut().find(|h| h.session == session) else {
                return;
            };
            held.approving = Some(at.clone());
            held.refused.remove(&at);
            self.notice_in(session, String::new(), cx);
        }
        // The card says the approval is with the yard, and takes no second
        // one, until the yard says what came of it.
        self.write_out_in(session, &at, Write::Approval, cx);
        // The approval is the yard's it was pressed in: what came of it is
        // that yard's, and is said there, wherever the window is by then.
        self.ask_in(
            session,
            cx,
            move |client, me| client.approve_ask(&bead, ask, me, weighed),
            move |this, approved, cx| {
                let held = this.held.values_mut().find(|h| h.session == session);
                let (approving, elsewhere) = match held {
                    _ if session == this.session => {
                        (&mut this.approving, &mut this.approve_elsewhere)
                    }
                    Some(held) => (&mut held.approving, &mut held.approve_elsewhere),
                    None => return,
                };
                *approving = None;
                let (bead, ask) = at.clone();
                match approved {
                    Ok(approved) => {
                        let line = model::approved_line(&approved);
                        this.change_in(session, cx, |m| m.approved(&bead, ask, &approved));
                        this.schedule_return_in(session, cx);
                        // A pairing approval shows its QR screen in that
                        // sitting alone, instead of the notice line.
                        match approved.pairing {
                            Some(pairing) => this.show_approval_pairing_in(
                                session,
                                bead.clone(),
                                ask,
                                pairing,
                                line,
                                cx,
                            ),
                            None => this.notice_in(session, line, cx),
                        }
                    }
                    Err(err) if model::approves_elsewhere(&err) => {
                        elsewhere.insert(at);
                        this.change_in(session, cx, |m| {
                            m.write_ended(&bead, ask);
                            Vec::new()
                        });
                        this.notice_in(session, String::new(), cx);
                    }
                    Err(err) => {
                        let why = model::not_approved(&err);
                        let lost = model::answer_lost(&err);
                        let at = (bead.clone(), ask);
                        this.write_failed_in(
                            session,
                            at,
                            Write::Approval,
                            lost,
                            String::new(),
                            why,
                            cx,
                        )
                    }
                }
            },
        );
    }

    /// The write of the ask's card goes out to the yard of the following
    /// `session`: the card says so, and takes no second write of any kind,
    /// until the yard says what came of it.
    fn write_out_in(
        &mut self,
        session: u64,
        at: &(String, u64),
        write: Write,
        cx: &mut Context<Self>,
    ) {
        self.change_in(session, cx, |m| {
            m.write_out(&at.0, at.1, write);
            Vec::new()
        });
    }

    /// A write of the following `session`'s yard ended with no answer to
    /// say what came of it ([`model::answer_lost`]). It may have been taken
    /// all the same (a command run, an answer written, a value stored), so
    /// the card says nothing of the sort: it waits, and the queue is read
    /// for what the yard did (the bead's notes, for a note), at once where
    /// the yard answers and else when it is back ([`Model::resumed`] settles
    /// it from the state it reads). Never a failure for a write the yard
    /// took: the read says it was, or that the yard has not recorded it
    /// ([`Footplate::settled_in`]), and the card offers no second write
    /// meanwhile, so that none goes on a guess; never the same one sent
    /// again of itself, since the yard may hold it already. `words` went
    /// with it, and are back where they were typed
    /// ([`Footplate::words_back`]): a read that says the yard took them
    /// takes them out again.
    fn write_lost_in(
        &mut self,
        session: u64,
        at: (String, u64),
        write: Write,
        words: String,
        cx: &mut Context<Self>,
    ) {
        // A yard known to be away is read when it is back, which settles
        // the card from the state read.
        let up = self.up_in(session);
        self.change_in(session, cx, |m| {
            let read = m.write_unanswered(&at.0, at.1, write, words);
            match up {
                true => read,
                false => Vec::new(),
            }
        });
    }

    /// The yard of the following `session` said what came of the write of
    /// the ask's card, or it never went: the card has nothing more to wait
    /// for, and offers its writes again.
    fn write_over_in(&mut self, session: u64, at: &(String, u64), cx: &mut Context<Self>) {
        self.change_in(session, cx, |m| {
            m.write_ended(&at.0, at.1);
            Vec::new()
        });
    }

    /// A write of the following `session`'s yard ended in a refusal, a
    /// connect that gave up, or with no answer (`lost`). The first two the
    /// card says as `refusal`: the yard's words, whole, or that nothing was
    /// sent. A call with no answer is waited for ([`Footplate::write_lost_in`]).
    #[allow(clippy::too_many_arguments)]
    fn write_failed_in(
        &mut self,
        session: u64,
        at: (String, u64),
        write: Write,
        lost: bool,
        words: String,
        refusal: String,
        cx: &mut Context<Self>,
    ) {
        if lost {
            return self.write_lost_in(session, at, write, words, cx);
        }
        self.write_over_in(session, &at, cx);
        self.refused_in(session, at, refusal, cx);
    }

    /// The queue of the following `session`'s yard was read whole, stamped
    /// `stamp`: the writes whose call ended without an answer are settled
    /// by it ([`Model::settle_writes`]), and each the yard still lists as
    /// it was says so on its card.
    pub(crate) fn settled_in(&mut self, session: u64, stamp: u64, cx: &mut Context<Self>) {
        let not_recorded = match self.model_in(session) {
            Some(model) => model.settle_writes(stamp),
            None => Vec::new(),
        };
        self.settled(session, not_recorded, cx);
    }

    /// The bead of the following `session`'s yard was read, stamped
    /// `stamp`: the notes whose call ended without an answer are settled
    /// by what its notes say ([`Model::settle_notes`]).
    pub(crate) fn settled_notes_in(
        &mut self,
        session: u64,
        bead: &str,
        stamp: u64,
        cx: &mut Context<Self>,
    ) {
        let not_recorded = match self.model_in(session) {
            Some(model) => model.settle_notes(bead, stamp),
            None => Vec::new(),
        };
        self.settled(session, not_recorded, cx);
    }

    /// The bead of the following `session` could not be read, and a note
    /// whose call got no answer waits for that read, which only the bead's
    /// notes can answer: it is read again after the window's retry, while
    /// the note waits and the yard answers. A yard that is away is read
    /// when it is back ([`Model::resumed`]).
    pub(crate) fn reread_notes_in(&mut self, session: u64, bead: &str, cx: &mut Context<Self>) {
        if !self
            .model_in(session)
            .is_some_and(|model| model.note_unsure(bead))
        {
            return;
        }
        let (bead, again_in) = (bead.to_string(), self.retry);
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(again_in).await;
            this.update(cx, |this, cx| {
                let waiting = this
                    .model_in(session)
                    .is_some_and(|model| model.note_unsure(&bead));
                if waiting && this.up_in(session) {
                    this.run_in(session, vec![Effect::Bead { bead }], cx);
                }
            })
            .ok();
        })
        .detach();
    }

    /// What a read of the yard of the following `session` said of the
    /// writes that got no answer: those it took have what the window kept
    /// of them taken out ([`Footplate::taken_in`]), and each that it did
    /// not record says so on its card.
    pub(crate) fn settled(
        &mut self,
        session: u64,
        not_recorded: Vec<(String, u64, Write)>,
        cx: &mut Context<Self>,
    ) {
        let taken = match self.model_in(session) {
            Some(model) => model.take_taken(),
            None => Vec::new(),
        };
        for taken in taken {
            self.taken_in(session, taken, cx);
        }
        for (bead, ask, write) in not_recorded {
            self.refused_in(session, (bead, ask), write.not_taken().to_string(), cx);
        }
        // A later queue read may close an ask another way after this card
        // said it was still open. That warning describes only the queue
        // read that made it: do not leave it under a decided card.
        let warnings: Vec<(String, u64)> = if session == self.session {
            self.refused
                .iter()
                .filter(|(_, why)| model::is_not_taken(why))
                .map(|(at, _)| at.clone())
                .collect()
        } else {
            self.held
                .values()
                .find(|held| held.session == session)
                .map(|held| {
                    held.refused
                        .iter()
                        .filter(|(_, why)| model::is_not_taken(why))
                        .map(|(at, _)| at.clone())
                        .collect()
                })
                .unwrap_or_default()
        };
        for at in warnings {
            let still_open = self
                .model_in(session)
                .is_some_and(|model| model.listed_open(&at.0, at.1));
            if !still_open {
                if session == self.session {
                    self.refused.remove(&at);
                } else if let Some(held) = self.held.values_mut().find(|h| h.session == session) {
                    held.refused.remove(&at);
                }
            }
        }
        cx.notify();
    }

    /// The yard of the following `session` took a write whose call got no
    /// answer: nothing is left to say of it, and what the window kept of
    /// it goes. The words that came back to the line (or to the ask's own
    /// words) are taken out again, as what was typed since stays; a note's
    /// files with them; and the value of a give, which only the card it was
    /// typed on ever held, is forgotten, never shown again.
    pub(crate) fn taken_in(&mut self, session: u64, taken: Taken, cx: &mut Context<Self>) {
        let at = (taken.bead.clone(), taken.ask);
        if session == self.session {
            self.refused.remove(&at);
        } else if let Some(held) = self.held.values_mut().find(|h| h.session == session) {
            held.refused.remove(&at);
        }
        if !taken.words.is_empty() {
            self.words_gone(session, &at, &taken.words, cx);
        }
        match taken.write {
            Write::Note { files } => self.files_gone(session, &at, &files),
            Write::Give { .. }
                if session == self.session && self.secret_for.as_ref() == Some(&at) =>
            {
                self.forget_secret(cx)
            }
            _ => {}
        }
        cx.notify();
    }

    /// The words for the ask `at` of the following `session`, which came
    /// back to where they were typed when the call that carried them got
    /// no answer, are gone: the yard took them. They went back as any
    /// failure's do ([`Footplate::words_back`]): into the line of the yard
    /// that is open, whatever card it is on by now (the queue that says the
    /// yard took them also moves the ask on), or with the ask's own words
    /// of the open yard or of the held one. What follows them there, typed
    /// since, stays; words that were edited since stay as they are.
    fn words_gone(
        &mut self,
        session: u64,
        at: &(String, u64),
        words: &str,
        cx: &mut Context<Self>,
    ) {
        if session != self.session {
            let yard = self
                .held
                .iter()
                .find(|(_, held)| held.session == session)
                .map(|(yard, _)| yard.clone());
            if let Some(kept) = yard.and_then(|yard| self.kept_words.get_mut(&yard)) {
                gone_from(kept, at, words);
            }
            return;
        }
        gone_from(&mut self.ask_words, at, words);
        let typed = self.line.read(cx).text().to_string();
        if let Some(rest) = draft::without_sent(&typed, words) {
            self.line.update(cx, |line, cx| line.set_text(&rest, cx));
        }
    }

    /// The files that went with a note the yard took, by id, are gone from
    /// where they were put back.
    fn files_gone(&mut self, session: u64, at: &(String, u64), files: &[u64]) {
        let kept = if session == self.session {
            Some(&mut self.note_files)
        } else {
            let yard = self
                .held
                .iter()
                .find(|(_, held)| held.session == session)
                .map(|(yard, _)| yard.clone());
            yard.and_then(|yard| self.kept_note_files.get_mut(&yard))
        };
        if let Some(kept) = kept
            && let Some(attached) = kept.get_mut(at)
        {
            attached.retain(|file| !files.contains(&file.id));
            if attached.is_empty() {
                kept.remove(at);
            }
        }
    }

    /// A pairing approval's transient goes on its decided card: the QR
    /// screen in that sitting alone, instead of the notice line. The
    /// history unfolds to the card and the card opens, so the screen
    /// shows where the approval was. The transient drops again when the
    /// screen is left or the code runs out. An ask with no decided card
    /// keeps the notice line instead: the screen has no card to stand
    /// on.
    pub(crate) fn show_approval_pairing_in(
        &mut self,
        session: u64,
        bead: String,
        ask: u64,
        pairing: api::AskPairing,
        line: String,
        cx: &mut Context<Self>,
    ) {
        // A result that arrives after its yard was switched away from is
        // no longer on the screen that authorized showing its transient.
        // Keep only the ordinary approval notice with that held yard.
        if session != self.session {
            return self.notice_in(session, line, cx);
        }
        let dealt = self
            .model_in(session)
            .is_some_and(|m| m.decided_detail(&bead, ask, &chrono::Utc).is_some());
        if !dealt {
            return self.notice_in(session, line, cx);
        }
        if pairing
            .expires
            .parse::<chrono::DateTime<chrono::Utc>>()
            .is_err()
        {
            self.approval_pairing = None;
            self.selection
                .update(cx, |selection, cx| selection.clear(cx));
            return self.notice_in(
                session,
                "the yard's pairing expiry could not be read; the code was not shown".into(),
                cx,
            );
        }
        let shown = ApprovalPairing::Shown {
            bead: bead.clone(),
            ask,
            pairing,
            ending: false,
            end_refused: String::new(),
        };
        self.approval_pairing = Some(shown);
        self.change_in(session, cx, |m| {
            let mut effects = m.show_decided(&bead, ask);
            effects.extend(m.select(Selected::Decided(bead.clone(), ask)));
            effects
        });
        self.schedule_approval_pairing_end_in(session, cx);
    }

    /// Drops a pairing approval's transient when its code runs out: the
    /// screen shows its ended state, with no code and no QR in it.
    /// Nothing waits where none is shown. A timer carries the card and
    /// expiry, never the code or QR; a sitting gone by then, or another's
    /// (left, ended, approved again), drops nothing.
    pub(crate) fn schedule_approval_pairing_end_in(
        &mut self,
        session: u64,
        cx: &mut Context<Self>,
    ) {
        if session != self.session {
            return;
        }
        let Some((bead, ask, expires, wait)) = self.approval_pairing.as_ref().and_then(|sitting| {
            let ApprovalPairing::Shown {
                bead, ask, pairing, ..
            } = sitting
            else {
                return None;
            };
            let expires = pairing.expires.clone();
            let wait = expires
                .parse::<chrono::DateTime<chrono::Utc>>()
                .ok()
                .map(|ends| ends.timestamp() - chrono::Utc::now().timestamp())
                .filter(|wait| *wait > 0)
                .map(|wait| wait as u64);
            Some((bead.clone(), *ask, expires, wait))
        }) else {
            return;
        };
        let Some(wait) = wait else {
            return self.expire_approval_pairing_in(session, &bead, ask, &expires, cx);
        };
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(wait))
                .await;
            this.update(cx, |this, cx| {
                this.expire_approval_pairing_in(session, &bead, ask, &expires, cx);
            })
            .ok();
        })
        .detach();
    }

    /// The code the sitting's screen shows ran out: its transient drops
    /// and the screen shows the ended state. A sitting that is gone, or
    /// is another's, is left: only its own drops.
    pub(crate) fn expire_approval_pairing_in(
        &mut self,
        session: u64,
        bead: &str,
        ask: u64,
        expires: &str,
        cx: &mut Context<Self>,
    ) {
        if session != self.session {
            return;
        }
        let sitting = &mut self.approval_pairing;
        let own = matches!(
            sitting,
            Some(ApprovalPairing::Shown {
                bead: shown_bead,
                ask: shown_ask,
                pairing,
                ..
            }) if shown_bead.as_str() == bead
                && *shown_ask == ask
                && pairing.expires.as_str() == expires
        );
        if own {
            if let Some(ApprovalPairing::Shown {
                bead, ask, pairing, ..
            }) = sitting.take()
            {
                *sitting = Some(ApprovalPairing::Ended {
                    bead,
                    ask,
                    device: pairing.device,
                    user: pairing.user,
                    expired: true,
                });
            }
            self.selection
                .update(cx, |selection, cx| selection.clear(cx));
            cx.notify();
        }
    }

    /// Ends the pairing a pairing approval's screen shows: nothing is
    /// added, and the code is good for nothing then. The screen shows
    /// its ended state. A refusal stands as the screen's line, and the
    /// pairing is kept: the yard reads an approval's pairing back to no
    /// one, so ending again is the way. A call that ended without an
    /// answer is never said to have ended or not. Nothing is ended where
    /// none is shown, or while one is away.
    pub(crate) fn end_approval_pairing(&mut self, cx: &mut Context<Self>) {
        let away = match self.approval_pairing.as_mut() {
            Some(ApprovalPairing::Shown {
                ending,
                end_refused,
                ..
            }) if !*ending => {
                *ending = true;
                end_refused.clear();
                true
            }
            _ => false,
        };
        if !away {
            return;
        }
        cx.notify();
        // The ending is the yard's it was pressed in: what came of it is
        // that yard's, and is said there, wherever the window is by then.
        let session = self.session;
        self.ask_in(
            session,
            cx,
            |client, me| client.device_pair_close_self(me),
            move |this, ended, cx| {
                if session != this.session {
                    return;
                }
                let sitting = &mut this.approval_pairing;
                match ended {
                    // The secrets drop with the sitting: the ended
                    // state holds no code and no QR.
                    Ok(()) => {
                        if let Some(ApprovalPairing::Shown {
                            bead, ask, pairing, ..
                        }) = sitting.take()
                        {
                            *sitting = Some(ApprovalPairing::Ended {
                                bead,
                                ask,
                                device: pairing.device,
                                user: pairing.user,
                                expired: false,
                            });
                        }
                        this.selection
                            .update(cx, |selection, cx| selection.clear(cx));
                    }
                    Err(err) => {
                        if let Some(ApprovalPairing::Shown {
                            ending,
                            end_refused,
                            ..
                        }) = sitting
                        {
                            *ending = false;
                            *end_refused = model::approval_not_ended(&err);
                        }
                    }
                }
                cx.notify();
            },
        );
    }

    /// Files a teammate's removal: the run ask `user uninvite <user>` on
    /// a new bead, addressed to the yard's operator. What the window says
    /// afterwards is the yard's: the bead and the ask it filed on, or the
    /// refusal whole, on the pane's line. A call that ended without an
    /// answer is never said to be filed or not: the queue is read again,
    /// and the card says which. Nothing underway is filed twice.
    pub(crate) fn remove_teammate(&mut self, user: String, cx: &mut Context<Self>) {
        let session = self.session;
        if !self.removing.insert((session, user.clone())) {
            return;
        }
        cx.notify();
        // The filing is the yard's it was pressed in: what came of it is
        // that yard's, and is said there, wherever the window is by then.
        let call = user.clone();
        self.ask_in(
            session,
            cx,
            move |client, me| client.uninvite_teammate(&call, me),
            move |this, filed, cx| {
                this.removing.remove(&(session, user.clone()));
                match filed {
                    Ok(filed) => {
                        let line = model::uninvited_line(&user, &filed);
                        this.notice_in(session, line, cx);
                    }
                    Err(err) => {
                        let reread = model::filing_unknown(&err);
                        this.notice_in(session, model::not_filed(&err), cx);
                        if reread {
                            this.run_in(session, vec![Effect::Asks(Width::Open)], cx);
                        }
                    }
                }
            },
        );
    }

    /// Opens a pairing for the device the pane's field names, for the
    /// window's own user: the terminal's `yardr user device pair` in the
    /// window. The opening answers the code and its expiry; the state is
    /// read for the QR string, and the pane shows all three. A refusal,
    /// a second opening while one lives or a name the yard will not
    /// pair, stands as the pane's line. A call that ended without an
    /// answer is never said to be open or not: the state is read again,
    /// and the pane says which. Nothing is opened where the pane is not
    /// offered, while one is away, or with no name typed.
    pub(crate) fn open_phone_pairing(&mut self, cx: &mut Context<Self>) {
        let session = self.session;
        if self.pairing_opening || self.pairing_ending {
            return;
        }
        let offered = self
            .model
            .as_ref()
            .is_some_and(|m| m.pair_offered() && !m.pairing_too_old() && m.pairing().is_none());
        if !offered {
            return;
        }
        let name = self.device_name.read(cx).text().trim().to_string();
        if name.is_empty() {
            return;
        }
        self.pairing_opening = true;
        cx.notify();
        // The opening is the yard's it was pressed in: what came of it is
        // that yard's, and is said there, wherever the window is by then.
        self.ask_in(
            session,
            cx,
            move |client, me| client.device_pair_open(me, &name, me),
            move |this, opened, cx| {
                this.pairing_opening = false;
                match opened {
                    Ok(pairing) => {
                        this.device_name
                            .update(cx, |line, cx| line.set_text("", cx));
                        this.change_in(session, cx, |m| m.pairing_opened(pairing));
                        this.schedule_pairing_end_in(session, cx);
                    }
                    Err(err) => {
                        this.change_in(session, cx, |m| m.pairing_open_failed(&err));
                    }
                }
            },
        );
    }

    /// Ends the yard's open pairing: nothing is added, and the code is
    /// good for nothing then. The pane offers Open again. A refusal
    /// stands as the pane's line. A call that ended without an answer is
    /// never said to be ended or not: the state is read again, and the
    /// pane says which. Nothing is ended where none is open, or while
    /// one is away.
    pub(crate) fn close_phone_pairing(&mut self, cx: &mut Context<Self>) {
        let session = self.session;
        if self.pairing_opening || self.pairing_ending {
            return;
        }
        let open = self
            .model
            .as_ref()
            .is_some_and(|m| m.pair_offered() && m.pairing().is_some());
        if !open {
            return;
        }
        self.pairing_ending = true;
        cx.notify();
        // The ending is the yard's it was pressed in, as an opening is.
        self.ask_in(
            session,
            cx,
            |client, me| client.device_pair_close(me),
            move |this, ended, cx| {
                this.pairing_ending = false;
                match ended {
                    Ok(()) => {
                        this.change_in(session, cx, |m| {
                            m.pairing_closed();
                            Vec::new()
                        });
                    }
                    Err(err) => {
                        this.change_in(session, cx, |m| m.pairing_close_failed(&err));
                    }
                }
            },
        );
    }

    /// Approves the open card's setup ask: the preset pick, the option
    /// chosen or the recommended one, with the account together. A filled
    /// secret field types another; else the tap takes its file; else a
    /// drift card whose standing file holds every name is kept with the
    /// bare preset pick. What the window says afterwards is the yard's:
    /// that the server carries the setup out, or the refusal whole, on
    /// the card. A call that ended without an answer is never said to be
    /// approved or not, and a yard without the call for this window has
    /// its card say where to approve.
    pub(crate) fn approve_setup(&mut self, bead: String, ask: u64, cx: &mut Context<Self>) {
        let at = (bead.clone(), ask);
        let Some(option) = self.model.as_ref().and_then(|m| m.setup_pick(&bead, ask)) else {
            let unread = !self.model.as_ref().is_some_and(|m| m.has_bead(&bead));
            return self.notice(
                match unread {
                    true => "reading the bead: approve once its presets show".to_string(),
                    false => "pick a preset first".to_string(),
                },
                cx,
            );
        };
        let Some(setup) = self.setup_approvable() else {
            return;
        };
        // The host the card asked for above the account, typed in the
        // clear: it goes with the approval, whatever shape the account
        // takes, and where none was typed nothing goes. An ask with no
        // template sends none, whatever the field holds.
        let host = match setup.host_template.is_empty() {
            true => None,
            false => {
                let typed = self.host.read(cx).text().trim().to_string();
                (!typed.is_empty()).then_some(typed)
            }
        };
        // The field's value goes with what Enter took, as the next name;
        // what is pasted often ends in a line break, which nobody sees
        // among the marks and which the yard refuses at either end.
        let field = self.secret.read(cx).text().trim().to_string();
        let mut typed = self.secret_typed.clone();
        if !field.is_empty() {
            typed.push(field);
        }
        let names = model::setup_typed_names(&setup);
        if names.is_empty() {
            return self.notice(
                "the yard asked for no account names: read the card again".to_string(),
                cx,
            );
        }
        if typed.len() > names.len() {
            return self.notice(
                "the account is typed: clear the field to approve it".to_string(),
                cx,
            );
        }
        let have = typed.len();
        let values: Option<Vec<(String, String)>> = match have == names.len() {
            true => Some(names.iter().cloned().zip(typed).collect()),
            false => None,
        };
        let account = match (&values, self.model.as_ref()) {
            (None, Some(model)) => match model.account_choice(&bead, ask) {
                taken if taken.is_empty() => None,
                taken => Some(taken),
            },
            _ => None,
        };
        if values.is_none() && account.is_none() {
            // A drift card whose standing file holds every account name
            // is kept with the bare preset pick: with nothing typed and
            // no candidate taken the approval goes as it stands. What is
            // typed instead changes the account, and is wanted whole.
            let keep = have == 0 && model::setup_keepable(&setup);
            if !keep {
                if have == 0 {
                    return self.notice(
                        match setup.candidates.is_empty() {
                            true => "type the account".to_string(),
                            false => "take a candidate, or type the account".to_string(),
                        },
                        cx,
                    );
                }
                let next = &names[have];
                return self.notice(format!("type {next} for the account"), cx);
            }
        }
        let session = self.session;
        match self.card_approval(&bead, ask) {
            Approval::Direct => self.send_approve_setup_in(
                session, bead, ask, option, values, account, setup, false, host, cx,
            ),
            // The number field holds the ask's number or the approval
            // sends nothing: the card stays as it is, and says so.
            Approval::TypeBack => {
                if !self.typeback_ok(ask, cx) {
                    return self.notice(typeback_mismatch(ask), cx);
                }
                self.clear_typeback(cx);
                self.send_approve_setup_in(
                    session, bead, ask, option, values, account, setup, true, host, cx,
                );
            }
            // The system unlock first: its cancel sends nothing, and the
            // control sends again. `approve` turns a card that approves
            // nowhere away before this.
            Approval::Unlock => {
                let reason = unlock::setup_reason(&setup.pack, &self.yard);
                self.declining = None;
                self.approving = Some(at.clone());
                self.shown(cx);
                self.notice(String::new(), cx);
                self.refused.remove(&at);
                self.unlock_then(
                    session,
                    reason,
                    cx,
                    move |this, session, cx| {
                        this.send_approve_setup_in(
                            session, bead, ask, option, values, account, setup, true, host, cx,
                        )
                    },
                    move |this, session, cx| {
                        this.clear_approving_in(session);
                        this.shown(cx);
                    },
                );
            }
            Approval::Nowhere => {}
        }
    }

    /// Approves a setup ask of the following `session`: the preset pick
    /// with the account together. `weighed` goes where the person passed
    /// the system unlock for a weighty card, or typed its number back where
    /// no unlock is offered. What the window says afterwards is the yard's:
    /// that the server carries the setup out, or the refusal whole, on the
    /// card. While the call is out, and after a call with no answer, the
    /// card waits as a run ask's does ([`Footplate::send_approve`]).
    #[allow(clippy::too_many_arguments)]
    fn send_approve_setup_in(
        &mut self,
        session: u64,
        bead: String,
        ask: u64,
        option: u64,
        values: Option<Vec<(String, String)>>,
        account: Option<String>,
        setup: api::Setup,
        weighed: bool,
        host: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let at = (bead.clone(), ask);
        if session == self.session {
            self.declining = None;
            self.approving = Some(at.clone());
            self.shown(cx);
            self.notice(String::new(), cx);
            self.refused.remove(&at);
        } else {
            let Some(held) = self.held.values_mut().find(|h| h.session == session) else {
                return;
            };
            held.approving = Some(at.clone());
            held.refused.remove(&at);
            self.notice_in(session, String::new(), cx);
        }
        self.write_out_in(session, &at, Write::Approval, cx);
        // The approval is the yard's it was pressed in: what came of it is
        // that yard's, and is said there, wherever the window is by then.
        self.ask_in(
            session,
            cx,
            move |client, me| {
                let approved = client.approve_setup(
                    &bead,
                    ask,
                    option,
                    values.as_deref(),
                    account.as_deref(),
                    me,
                    weighed,
                    host.as_deref(),
                );
                (approved, values)
            },
            move |this, (approved, values), cx| {
                let held = this.held.values_mut().find(|h| h.session == session);
                let (approving, elsewhere) = match held {
                    _ if session == this.session => {
                        (&mut this.approving, &mut this.approve_elsewhere)
                    }
                    Some(held) => (&mut held.approving, &mut held.approve_elsewhere),
                    None => return,
                };
                *approving = None;
                let (bead, ask) = at.clone();
                match approved {
                    Ok(approved) => {
                        let line = model::setup_approved_line(&setup, &approved);
                        this.forget_secret(cx);
                        if this
                            .host_for
                            .as_ref()
                            .is_some_and(|(for_session, for_bead, for_ask)| {
                                *for_session == session && for_bead == &bead && *for_ask == ask
                            })
                        {
                            this.forget_host(cx);
                        }
                        this.change_in(session, cx, |m| {
                            m.setup_approved(&bead, ask, option, &approved)
                        });
                        this.schedule_return_in(session, cx);
                        this.notice_in(session, line, cx);
                    }
                    Err(err) if model::approves_elsewhere(&err) => {
                        elsewhere.insert(at);
                        this.change_in(session, cx, |m| {
                            m.write_ended(&bead, ask);
                            Vec::new()
                        });
                        this.notice_in(session, String::new(), cx);
                    }
                    Err(err) => {
                        let why = model::not_approved(&err);
                        // The yard repeats no value in a refusal. Should
                        // anything on the way do so, it is not shown.
                        let echoes = values.as_ref().is_some_and(|values| {
                            values.iter().any(|(_, value)| why.contains(value.as_str()))
                        });
                        let why = match echoes {
                            true => "not approved: the yard refused it".to_string(),
                            false => why,
                        };
                        let lost = model::answer_lost(&err);
                        let at = (bead.clone(), ask);
                        this.write_failed_in(
                            session,
                            at,
                            Write::Approval,
                            lost,
                            String::new(),
                            why,
                            cx,
                        );
                    }
                }
            },
        );
    }

    /// Takes the setup ask's candidate with a tap: whatever is typed of
    /// another is forgotten, since a filled secret field types another
    /// instead.
    pub(crate) fn take_account_for(
        &mut self,
        bead: String,
        ask: u64,
        source: String,
        cx: &mut Context<Self>,
    ) {
        let approving = self
            .model
            .as_ref()
            .is_some_and(|m| m.setup_approving(&bead, ask).is_some());
        if !approving {
            return;
        }
        self.forget_secret(cx);
        self.change(cx, |m| {
            m.take_account(&bead, ask, &source);
            Vec::new()
        });
        self.shown(cx);
    }

    /// Enter on a card: the pick with the line's words in the one answer,
    /// or the words alone. Enter with nothing chosen and a recommendation
    /// picks the recommended option.
    pub(crate) fn decide(&mut self, cx: &mut Context<Self>) {
        // Answers on a card are held as a conversation's sends are.
        if self.reconnecting() {
            return;
        }
        // On a parked bead's card Enter makes the move its button names.
        if let Some(bead) = self.waiting_card() {
            return self.make_picked_move(bead, cx);
        }
        let Some((bead, ask)) = self.card() else {
            return;
        };
        // A card with a write out, or not known to have reached the yard,
        // offers no second one: the words stay in the line, and the card
        // says what it waits for.
        if self.write_sending(&bead, ask).is_some() {
            return;
        }
        // A secret ask has its own field and its own call, and no words.
        if self.giving().is_some() {
            return self.give(cx);
        }
        // A decline asked for words: they go now.
        if self.declining.as_ref() == Some(&(bead.clone(), ask)) {
            let words = self.line.read(cx).text().trim().to_string();
            if words.is_empty() {
                return self.notice("decline needs words: why you will not decide it".into(), cx);
            }
            if !self.card_files(&bead, ask).is_empty() {
                return self.notice("a decline carries no file: remove it, then send".into(), cx);
            }
            return self.send_decline(bead, ask, words, cx);
        }
        let words = self.line.read(cx).text().to_string();
        let action = self
            .model
            .as_ref()
            .and_then(|m| m.answer_action(&bead, ask, &words));
        // A file attached to the card's note goes with words alone: an
        // answer carries none, and where the words go nowhere as a note,
        // the file stays until it is removed.
        if !self.card_files(&bead, ask).is_empty() {
            return match action {
                Some(AnswerAction::Note { words }) => {
                    match draft::note_unsendable(&words, self.card_files(&bead, ask)) {
                        Some(why) => self.notice(why, cx),
                        None => self.send_note(bead, ask, words, cx),
                    }
                }
                Some(AnswerAction::Answer { .. }) => {
                    self.notice("an answer carries no file: remove it, then send".into(), cx)
                }
                None => match draft::note_unsendable(&words, self.card_files(&bead, ask)) {
                    Some(why) => self.notice(why, cx),
                    None => self.notice(
                        "words alone go nowhere on this ask: remove the file, then send".into(),
                        cx,
                    ),
                },
            };
        }
        match action {
            Some(AnswerAction::Answer { option, words }) => {
                self.send_answer(bead, ask, option, words, cx);
            }
            Some(AnswerAction::Note { words }) => self.send_note(bead, ask, words, cx),
            None => {}
        }
    }

    /// Picks an option with words alongside: the ask moves to "Decided" at
    /// once, and the queue and the bead are read again for what the yard
    /// says. What the yard refuses is back in the line, with its words,
    /// and why is on the card. A mapping pick writes a user's identity, so
    /// it passes the weighty gate like any other weighty approval: the
    /// Mac's unlock where one is offered, the ask's number typed back
    /// where none is, and `weighed` with the pick. Words alone, later and
    /// decline stay ungated.
    pub(crate) fn send_answer(
        &mut self,
        bead: String,
        ask: u64,
        option: u64,
        words: String,
        cx: &mut Context<Self>,
    ) {
        // A pick on a mapping ask writes by definition; words alone write
        // nothing and go straight.
        let mapping = option != 0
            && self
                .model
                .as_ref()
                .is_some_and(|m| m.identity(&bead, ask).is_some());
        // The yard the pick was pressed in: what came of it is that
        // yard's, and is said there, wherever the window is by then.
        let (yard, opening, session) = (self.yard.clone(), self.opening, self.session);
        if !mapping {
            return self
                .send_answer_in(session, bead, ask, option, words, false, yard, opening, cx);
        }
        match self.card_approval(&bead, ask) {
            Approval::Direct => {
                self.send_answer_in(session, bead, ask, option, words, false, yard, opening, cx)
            }
            // The number field holds the ask's number or the pick sends
            // nothing: the card stays as it is, and says so.
            Approval::TypeBack => {
                if !self.typeback_ok(ask, cx) {
                    return self.notice(typeback_mismatch(ask), cx);
                }
                self.clear_typeback(cx);
                self.send_answer_in(session, bead, ask, option, words, true, yard, opening, cx);
            }
            // The Mac's unlock first: its cancel sends nothing, and the
            // words stay in the line for another try. Nowhere is no card
            // an answer sends from: `AnswerAsk` serves every window.
            Approval::Unlock => {
                let reason = self.mapping_reason(&bead, ask);
                self.unlock_then(
                    session,
                    reason,
                    cx,
                    move |this, session, cx| {
                        this.send_answer_in(
                            session, bead, ask, option, words, true, yard, opening, cx,
                        )
                    },
                    move |_, _, _| {},
                );
            }
            Approval::Nowhere => {}
        }
    }

    /// What the Mac's unlock shows with its prompt for a mapping pick: who
    /// the pick says the sender is, and in which yard. Never empty: the
    /// call throws on an empty reason.
    fn mapping_reason(&self, bead: &str, ask: u64) -> String {
        let who = self
            .model
            .as_ref()
            .and_then(|m| m.identity(bead, ask))
            .map(|id| format!("who {} in {} is", id.sender, id.source))
            .unwrap_or_default();
        unlock::approve_reason(&who, &self.yard)
    }

    /// Picks an option of the following `session` with words alongside.
    /// `weighed` goes where the person passed the Mac's unlock for a
    /// weighty mapping pick, or typed its number back where no unlock is
    /// offered. `yard` with `opening` is where the pick was pressed: what
    /// the yard refuses is back there.
    #[allow(clippy::too_many_arguments)]
    fn send_answer_in(
        &mut self,
        session: u64,
        bead: String,
        ask: u64,
        option: u64,
        words: String,
        weighed: bool,
        yard: String,
        opening: u64,
        cx: &mut Context<Self>,
    ) {
        // The line is cleared only where this yard is still open: a pick
        // sent after the window moved on leaves another yard's line alone.
        if session == self.session {
            self.line.update(cx, |line, cx| line.set_text("", cx));
            self.ask_words.remove(&(bead.clone(), ask));
            self.notice(String::new(), cx);
            self.refused.remove(&(bead.clone(), ask));
        } else {
            let Some(held) = self.held.values_mut().find(|h| h.session == session) else {
                return;
            };
            held.refused.remove(&(bead.clone(), ask));
            self.notice_in(session, String::new(), cx);
        }
        let (call_bead, call_words) = (bead.clone(), words.clone());
        let identity = self.yard_identity(&yard);
        // The card says the answer is with the yard, and takes no second
        // write, until the yard says what came of it.
        self.write_out_in(session, &(bead.clone(), ask), Write::Answer { option }, cx);
        self.ask_in(
            session,
            cx,
            move |client, me| client.answer_ask(&call_bead, ask, option, &call_words, me, weighed),
            move |this, answered, cx| match answered {
                // The yard it was answered in has it, wherever the window
                // is by now.
                Ok(()) => {
                    this.change_in(session, cx, |m| m.picked(&bead, ask, option, &words));
                    this.schedule_return_in(session, cx);
                }
                Err(err) => {
                    if this.yard_identity(&yard) != identity {
                        return this.notice(
                            format!(
                                "not answered in {yard}: that yard entry was removed or changed, so its words were not restored"
                            ),
                            cx,
                        );
                    }
                    // The words are back in the line as they are for any
                    // failure; where the call got no answer they stay while
                    // the queue is read, and go again if it says the yard
                    // took them.
                    let at = (bead, ask);
                    let lost = model::answer_lost(&err);
                    this.words_back((&yard, opening), at.clone(), words.clone(), cx);
                    let write = Write::Answer { option };
                    let why = format!("not answered: {err}");
                    this.write_failed_in(session, at, write, lost, words, why, cx);
                }
            },
        );
    }

    /// Words with no option chosen go as a plain note, with the card's
    /// files; the ask stays open. What the yard refuses is back in the
    /// line with its files, and the file that kept it is marked. While the
    /// call is out the card says the note is with the yard; one that ended
    /// with no answer waits while the bead is read for the note, which only
    /// its notes say ([`Model::settle_notes`]): the words and files come
    /// back as for a refusal and stay, never sent again of themselves, and
    /// go again where the bead has the note.
    pub(crate) fn send_note(
        &mut self,
        bead: String,
        ask: u64,
        words: String,
        cx: &mut Context<Self>,
    ) {
        let at = (bead.clone(), ask);
        let files = self.note_files.remove(&at).unwrap_or_default();
        self.line.update(cx, |line, cx| line.set_text("", cx));
        self.ask_words.remove(&at);
        self.refused.remove(&at);
        self.notice(String::new(), cx);
        let (call_bead, call_words) = (bead.clone(), words.clone());
        let (yard, opening, session) = (self.yard.clone(), self.opening, self.session);
        let identity = self.yard_identity(&yard);
        let attached: Vec<u64> = files.iter().map(|file| file.id).collect();
        self.write_out_in(
            session,
            &at,
            Write::Note {
                files: attached.clone(),
            },
            cx,
        );
        self.ask_through(
            cx,
            move |client, me| match post_note(client, me, &call_bead, &call_words, &files) {
                Ok(()) => Ok(()),
                Err((at, why, lost)) => Err(UnsentNote {
                    words: call_words,
                    files,
                    at,
                    why,
                    lost,
                }),
            },
            move |this, noted, cx| match noted {
                Ok(()) => {
                    // The note arrives as a `noted` event, which reads the
                    // queue and the bead again.
                    this.write_over_in(session, &(bead, ask), cx);
                }
                Err(_) if this.yard_identity(&yard) != identity => this.notice(
                    format!(
                        "not noted in {yard}: that yard entry was removed or changed, so its words were not restored"
                    ),
                    cx,
                ),
                Err(unsent) => {
                    let write = Write::Note { files: attached };
                    this.note_back(session, &yard, opening, (bead, ask), write, unsent, cx)
                }
            },
        );
    }

    /// The files attached to the ask's note, in the order they came.
    pub(crate) fn card_files(&self, bead: &str, ask: u64) -> &[Attachment] {
        self.note_files
            .get(&(bead.to_string(), ask))
            .map_or(&[], Vec::as_slice)
    }

    /// Words for the ask `at` of `yard` that the yard did not take are
    /// back with what was typed since, and their files before what was
    /// attached since, the file that kept them marked: in the line while
    /// that yard is still the open one, as it was opened then, and else
    /// with the ask they were written for, in its own yard. A note whose
    /// call got no answer has them back the same, and the card waits for
    /// what the bead says of it ([`Footplate::write_lost_in`]) where a
    /// refusal says why, as `notice`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn note_back(
        &mut self,
        session: u64,
        yard: &str,
        opening: u64,
        at: (String, u64),
        write: Write,
        unsent: UnsentNote,
        cx: &mut Context<Self>,
    ) {
        let kept = match yard == self.yard {
            true => &mut self.note_files,
            false => self.kept_note_files.entry(yard.to_string()).or_default(),
        };
        let files = kept.entry(at.clone()).or_default();
        let mut back = unsent.files;
        back.append(files);
        *files = back;
        if let Some(id) = unsent.at {
            for file in files.iter_mut().filter(|a| a.id == id) {
                file.refused = Some(unsent.why.clone());
            }
        }
        let words = unsent.words.clone();
        let in_line = self.words_back((yard, opening), at.clone(), unsent.words, cx);
        if unsent.lost {
            return self.write_lost_in(session, at, write, words, cx);
        }
        self.write_over_in(session, &at, cx);
        match in_line {
            true => self.notice(format!("not noted: {}", unsent.why), cx),
            false => self.notice(format!("not noted in {yard}: {}", unsent.why), cx),
        }
    }

    /// Puts the ask aside with `x`: it leaves the list and returns as the
    /// yard says. While the call is out the card says so; one that ended
    /// with no answer waits while the queue is read, and the card ends as
    /// put aside or says the yard still lists it open
    /// ([`Footplate::write_lost_in`]). The words typed on the card, which
    /// Later takes none of, come back as any failure's do, and go again
    /// where the yard took it.
    pub(crate) fn send_later(&mut self, bead: String, ask: u64, cx: &mut Context<Self>) {
        // Held with the card's answers: the words stay where they are.
        if self.reconnecting() {
            return;
        }
        let at = (bead.clone(), ask);
        let words = self.line.read(cx).text().trim().to_string();
        self.line.update(cx, |line, cx| line.set_text("", cx));
        self.ask_words.remove(&at);
        self.refused.remove(&at);
        self.declining = None;
        self.notice(String::new(), cx);
        let call_bead = bead.clone();
        let (yard, opening, session) = (self.yard.clone(), self.opening, self.session);
        let identity = self.yard_identity(&yard);
        self.write_out_in(session, &at, Write::Later, cx);
        self.ask_through(
            cx,
            move |client, me| client.defer_ask(&call_bead, ask, me),
            move |this, deferred, cx| match deferred {
                Ok(()) => {
                    this.change_in(session, cx, |m| m.deferred(&bead, ask));
                    this.schedule_return_in(session, cx);
                }
                Err(err) => {
                    // Words are put back only into the yard they were
                    // typed in.
                    let words = match this.yard_identity(&yard) == identity {
                        true => words,
                        false => String::new(),
                    };
                    if !words.is_empty() {
                        this.words_back((&yard, opening), at.clone(), words.clone(), cx);
                    }
                    if model::answer_lost(&err) {
                        return this.write_lost_in(session, at, Write::Later, words, cx);
                    }
                    this.write_over_in(session, &at, cx);
                    this.notice_in(session, format!("not put aside: {err}"), cx);
                }
            },
        );
    }

    /// Declines the ask with words: not a pick and not "you decide". While
    /// the call is out the card says so; one that ended with no answer
    /// waits while the queue is read, the words back in the line and sent
    /// again by nobody, and the card ends as declined or says the yard
    /// still lists it open ([`Footplate::write_lost_in`]).
    pub(crate) fn send_decline(
        &mut self,
        bead: String,
        ask: u64,
        words: String,
        cx: &mut Context<Self>,
    ) {
        // Held with the card's answers.
        if self.reconnecting() {
            return;
        }
        let at = (bead.clone(), ask);
        self.line.update(cx, |line, cx| line.set_text("", cx));
        self.ask_words.remove(&at);
        self.refused.remove(&at);
        self.declining = None;
        self.notice(String::new(), cx);
        let (call_bead, call_words) = (bead.clone(), words.clone());
        let (yard, opening, session) = (self.yard.clone(), self.opening, self.session);
        let identity = self.yard_identity(&yard);
        self.write_out_in(session, &at, Write::Decline, cx);
        self.ask_through(
            cx,
            move |client, me| client.decline_ask(&call_bead, ask, &call_words, me),
            move |this, declined, cx| match declined {
                Ok(()) => {
                    this.change_in(session, cx, |m| m.declined(&bead, ask, &words));
                    this.schedule_return_in(session, cx);
                }
                Err(err) => {
                    if this.yard_identity(&yard) != identity {
                        return this.notice(
                            format!(
                                "not declined in {yard}: that yard entry was removed or changed, so its words were not restored"
                            ),
                            cx,
                        );
                    }
                    let in_line = this.words_back((&yard, opening), at.clone(), words.clone(), cx);
                    if model::answer_lost(&err) {
                        return this.write_lost_in(session, at, Write::Decline, words, cx);
                    }
                    this.write_over_in(session, &at, cx);
                    match in_line {
                        true => this.notice(format!("not declined: {err}"), cx),
                        false => this.notice(format!("not declined in {yard}: {err}"), cx),
                    }
                }
            },
        );
    }

    /// Puts the open card's current ask aside: the `x` key and the Later
    /// button. Later takes no words: only an open ask, never one dealt with.
    pub(crate) fn later_now(&mut self, cx: &mut Context<Self>) {
        let current = self.model.as_ref().and_then(|m| m.current_ask());
        if let Some((bead, ask)) = current
            && self.write_sending(&bead, ask).is_none()
        {
            self.send_later(bead, ask, cx);
        }
    }

    /// A number key chooses an option of the open card.
    pub(crate) fn pick(&mut self, option: u64, cx: &mut Context<Self>) {
        // On a parked bead's card the number is a move's place.
        if let Some(bead) = self.waiting_card() {
            self.change(cx, |m| {
                m.choose_move(&bead, option);
                Vec::new()
            });
            return;
        }
        let Some((bead, ask)) = self.card() else {
            return;
        };
        self.declining = None;
        self.change(cx, |m| {
            m.choose(&bead, ask, option);
            Vec::new()
        });
    }

    /// The arrows choose an option of the open card, around the ends.
    pub(crate) fn choice(&mut self, by: isize, cx: &mut Context<Self>) {
        // In the switcher's list the arrows move among the yards, and stop
        // at its ends.
        if let Some(at) = self.switching {
            let last = self.settings.yards.len().saturating_sub(1);
            self.switching = Some(at.saturating_add_signed(by).min(last));
            return cx.notify();
        }
        // On a parked bead's card the arrows move the pick among its moves.
        if let Some(bead) = self.waiting_card() {
            self.change(cx, |m| {
                m.step_move(&bead, by);
                Vec::new()
            });
            return;
        }
        let Some((bead, ask)) = self.card() else {
            return;
        };
        self.declining = None;
        self.change(cx, |m| {
            m.move_choice(&bead, ask, by);
            Vec::new()
        });
    }

    /// Decline asks for words and sends them: not a pick and not
    /// "you decide". With words already typed they go at once; else the
    /// next Enter sends what is typed, and esc leaves it.
    pub(crate) fn decline(&mut self, _: &Decline, window: &mut Window, cx: &mut Context<Self>) {
        self.decline_now(window, cx);
    }

    /// Declines the open card's current ask: the `d` key and the Decline
    /// button.
    pub(crate) fn decline_now(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current = self.model.as_ref().and_then(|m| m.current_ask());
        let Some((bead, ask)) = current else {
            return;
        };
        // As Enter: no second write while one is with the yard.
        if self.write_sending(&bead, ask).is_some() {
            return;
        }
        // A secret ask is declined with no words: the yard takes none on
        // it, since words there could be the value.
        if self.giving().is_some() {
            // Held with the card's answers: the value stays in the
            // field.
            if self.reconnecting() {
                return;
            }
            self.forget_secret(cx);
            return self.send_decline(bead, ask, String::new(), cx);
        }
        let words = self.line.read(cx).text().trim().to_string();
        if !words.is_empty() {
            return self.send_decline(bead, ask, words, cx);
        }
        self.declining = Some((bead, ask));
        self.shown(cx);
        window.focus(&self.line.focus_handle(cx));
        cx.notify();
    }

    /// The `a` key asks about the open card: the conversation with its
    /// asker, with the card's reference in the draft.
    pub(crate) fn ask_about(&mut self, _: &AskAbout, window: &mut Window, cx: &mut Context<Self>) {
        self.ask_about_now(window, cx);
    }

    /// Opens the conversation with the open card's asker, with the card's
    /// reference first in its draft: `re <bead> "<question>"`, then an
    /// empty line with the caret on it. Nothing is sent; what the draft
    /// held is kept after the reference, which is put once. The ask stays
    /// open among the decisions: its row is one step back. An asker with
    /// no conversation says so instead.
    pub(crate) fn ask_about_now(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.settings_open {
            return;
        }
        let open = self.model.as_ref().and_then(|m| {
            let (bead, ask) = m.current_ask()?;
            let now = chrono::Utc::now();
            let card = m
                .decision_detail(&bead, &now)?
                .asks
                .into_iter()
                .find(|a| a.ask == ask)?;
            let why = m.unwritable(&card.asker);
            // One trusted as a user of this yard is written to in that
            // user's row.
            let with = m
                .conversation_with(&card.asker)
                .unwrap_or(&card.asker)
                .to_string();
            Some((bead, with, card.question, why))
        });
        let Some((bead, asker, question, why)) = open else {
            return;
        };
        if let Some(why) = why {
            return self.notice(why, cx);
        }
        let reference = draft::ask_reference(&bead, &question);
        let existing = self.drafts.take_text(&asker);
        let (text, caret) = draft::prepend_reference(&existing, &reference);
        self.drafts.put_text(&asker, text);
        // The asker is a chat row. Save the decision row in its own mode
        // and move the rail with the conversation before opening it.
        self.set_mode(Mode::Chat, window, cx);
        self.select(Selected::Conversation(asker), window, cx);
        self.line.update(cx, |line, cx| line.set_cursor(caret, cx));
    }

    /// One key expands the card in place to the bead's body, notes and
    /// about-beads, or folds it again.
    pub(crate) fn expand(&mut self, _: &Expand, _: &mut Window, cx: &mut Context<Self>) {
        let Some((bead, ask)) = self.card() else {
            return;
        };
        self.change(cx, |m| {
            m.toggle_expand(&bead, ask);
            Vec::new()
        });
    }

    /// The line has the words: `i` types them in, where a number key picks.
    pub(crate) fn focus_words(
        &mut self,
        _: &FocusWords,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // A parked bead's card has the line for its move's note, where it
        // offers a move.
        if self.waiting_card().is_some() {
            if self.waiting_moves() > 0 {
                window.focus(&self.line.focus_handle(cx));
            }
            return;
        }
        let Some((bead, ask)) = self.card() else {
            return;
        };
        // A taken-back ask's card draws no field to type in.
        if self
            .model
            .as_ref()
            .is_some_and(|m| m.taken_back(&bead, ask))
        {
            return;
        }
        // A weighty card with no gate takes its number first: `i` types
        // it until the field holds the ask's number, then what the card
        // otherwise takes. A decline's words take the line as ever.
        if self.declining.is_none()
            && matches!(self.card_approval(&bead, ask), Approval::TypeBack)
            && !self.typeback_ok(ask, cx)
        {
            window.focus(&self.typeback.focus_handle(cx));
            return;
        }
        // A run ask's card has a line only for the words of a decline.
        if self.run_card() && self.declining.is_none() {
            return;
        }
        // A card that approves nowhere draws no field to type in: its
        // approval goes in a terminal, and a decline's words take the
        // line.
        if self.declining.is_none() && matches!(self.card_approval(&bead, ask), Approval::Nowhere) {
            return;
        }
        // A setup ask's card has the secret field for its account, as a
        // secret ask's has it for its value.
        let field = match self.giving().is_some() || self.setup_approvable().is_some() {
            true => &self.secret,
            false => &self.line,
        };
        window.focus(&field.focus_handle(cx));
    }

    /// `/` types in the history's filter: the folded history stands open
    /// for it. Elsewhere than decisions the key is nothing.
    pub(crate) fn focus_filter(
        &mut self,
        _: &FocusFilter,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.mode() != Mode::Decisions
            || self
                .model
                .as_ref()
                .is_none_or(|model| model.decided_count() == 0)
        {
            return;
        }
        self.change(cx, |m| match m.decided_expanded() {
            false => m.toggle_decided(),
            true => Vec::new(),
        });
        window.focus(&self.filter.focus_handle(cx));
    }

    /// Makes the ask the bead's current one: the keys act on it. What is
    /// typed stays with the ask it was typed in.
    pub(crate) fn select_ask(&mut self, bead: String, ask: u64, cx: &mut Context<Self>) {
        let old = self.model.as_ref().and_then(|m| m.current_for(&bead));
        if old == Some(ask) {
            return;
        }
        let typed = self.line.read(cx).text().to_string();
        if let Some(old) = old {
            self.ask_words.insert((bead.clone(), old), typed);
        }
        self.change(cx, |m| {
            m.select_ask(&bead, ask);
            Vec::new()
        });
        let text = self.ask_words.remove(&(bead, ask)).unwrap_or_default();
        self.line.update(cx, |line, cx| line.set_text(&text, cx));
        self.declining = None;
        self.shown(cx);
    }

    /// Clicks an option of any card of the open bead: it becomes current,
    /// and the option is chosen.
    pub(crate) fn pick_for(&mut self, bead: String, ask: u64, option: u64, cx: &mut Context<Self>) {
        if self.model.as_ref().and_then(|m| m.current_for(&bead)) != Some(ask) {
            self.select_ask(bead.clone(), ask, cx);
        }
        self.pick(option, cx);
    }

    /// Clicks a card's expand: the bead's body, notes and about-beads.
    pub(crate) fn expand_for(&mut self, bead: String, ask: u64, cx: &mut Context<Self>) {
        self.change(cx, |m| {
            m.toggle_expand(&bead, ask);
            Vec::new()
        });
    }

    /// The fields of the yard that is being added, signed in or
    /// paired again, with the button under them, what came of the last
    /// change to the yards under that, and the help under it: under the
    /// yards in the settings, and under the ways to a yard where there
    /// is none. Renaming stands in the renamed card instead, said beside
    /// its form.
    pub(crate) fn yard_asked(&self, palette: Palette, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut asked: Vec<AnyElement> = Vec::new();
        let busy = self.pairing || self.signing_in;
        let pair = match self.pairing {
            true => "asking the yard…",
            false => "pair (enter)",
        };
        let sign = match self.signing_in {
            true => "signing in…",
            false => "sign in (enter)",
        };
        const OPERATOR: &str = "The yard's operator opens a pairing on the yard's own screen. Paste the full QR, including its fourth field, to seal this device. At a hosted yard, its own-screen code, typed or read aloud, seals it through a PAKE the edge cannot read. A code from the edge's own page is known to it, and does not seal this device.";
        // What came of the last change stands directly under the button,
        // where a person looks the moment it is pressed: the help stands
        // under that. Below the help, a refusal falls off a short window
        // while the button stays in it, and the press looks silent.
        let mut help: Option<AnyElement> = None;
        match &self.adding {
            None => {}
            Some(Adding::Near) => {
                asked.extend([
                    yard_field("name", &self.yard_name, palette, cx).into_any_element(),
                    yard_field("socket", &self.socket, palette, cx).into_any_element(),
                    yard_submit("add (enter)", "yard pair", busy, palette, cx).into_any_element(),
                ]);
                help = Some(
                    yard_help(
                        "A yard of this machine is reached by its socket: no token or pairing. This field starts at this run's socket; change it to another home's yard socket (<its home>/.yardr/yard.sock)."
                            .to_string(),
                        palette,
                    )
                    .into_any_element(),
                );
            }
            Some(Adding::Far) => {
                asked.extend([
                    yard_field("name", &self.yard_name, palette, cx).into_any_element(),
                    yard_field("address", &self.address, palette, cx).into_any_element(),
                    yard_field("code", &self.code, palette, cx).into_any_element(),
                    yard_submit(pair, "yard pair", busy, palette, cx).into_any_element(),
                ]);
                help = Some(
                    yard_help(
                        "This form pairs directly with an address. A typed code goes to that address; a QR with a fourth field seals it and pins the yard's key. For a hosted yard, use its sign-in flow instead: a code from the yard's own screen can be typed or read aloud through a PAKE. A member code from the edge's page is known to it and does not seal this device. With no name it is called by its address's host. The device's token is kept in a file of this yard's own beside the settings, for you alone, and never with the settings."
                            .to_string(),
                        palette,
                    )
                    .into_any_element(),
                );
            }
            Some(Adding::Again(name)) => {
                let (whose, address) = match &self.env.yard {
                    Some(address) => ("the yard".to_string(), address.clone()),
                    None => {
                        let listed = self.settings.yard(name);
                        let address = listed.map(|yard| yard.address.clone());
                        (name.clone(), address.unwrap_or_default())
                    }
                };
                let token = match self
                    .env
                    .token_of(&self.settings, name, self.kept.as_deref())
                {
                    Ok(file) => format!(
                        "The device's token is kept in {}, for you alone, and never with the settings.",
                        file.display()
                    ),
                    Err(why) => why,
                };
                // A member pairs from the operator-opened QR on the yard's
                // own screen. The edge's own pairing page can read or
                // rewrite a code, so it cannot establish the pinned key.
                let member = self
                    .settings
                    .yard(name)
                    .filter(|_| self.env.yard.is_none())
                    .filter(|yard| yard.member().is_some());
                let (line, explained) = match member {
                    Some(yard) => (
                        format!("pair this device with {}, at {address}", yard.title()),
                        format!(
                            "Pairing from this page cannot seal the app: the edge reads the code it gives a member, so a typed code does not seal this device. Ask the yard's operator to open a pairing QR on the yard's own screen, then paste the full QR here. {token}"
                        ),
                    ),
                    None => (
                        format!("a code for {whose}, at {address}"),
                        format!("{OPERATOR} {token}"),
                    ),
                };
                asked.extend([
                    div()
                        .debug_selector(|| "yard again for".to_string())
                        .child(line)
                        .into_any_element(),
                    yard_field("code", &self.code, palette, cx).into_any_element(),
                    yard_submit(pair, "yard pair", busy, palette, cx).into_any_element(),
                ]);
                help = Some(yard_help(explained, palette).into_any_element());
            }
            Some(Adding::Hosted) => {
                asked.extend([
                    yard_field("name", &self.yard_name, palette, cx).into_any_element(),
                    yard_field("address", &self.address, palette, cx).into_any_element(),
                    yard_field("yard", &self.edge_yard, palette, cx).into_any_element(),
                    yard_field("email", &self.email, palette, cx).into_any_element(),
                    yard_field("password", &self.password, palette, cx).into_any_element(),
                    yard_submit(sign, "yard sign in", busy, palette, cx).into_any_element(),
                ]);
                help = Some(
                    yard_help(
                        "The edge's one origin, as the ready page shows it, which of the account's yards there, and the Kratos account's email and password. Empty is the account's own yard. Signing in keeps the session in a file of this yard's own beside the settings, for you alone, and never with the settings; pairing the device comes next with the full QR from the yard's own screen."
                            .to_string(),
                        palette,
                    )
                    .into_any_element(),
                );
            }
            Some(Adding::SignInAgain(name)) => {
                let listed = self.settings.yard(name);
                let address = listed.map(|yard| yard.address.clone()).unwrap_or_default();
                let session = match self
                    .env
                    .session_of(&self.settings, name, self.kept.as_deref())
                {
                    Ok(Some(file)) => format!(
                        "The session is kept in {}, for you alone, and never with the settings.",
                        file.display()
                    ),
                    Ok(None) => format!("The yard {name} is no hosted yard."),
                    Err(why) => why,
                };
                asked.extend([
                    div()
                        .debug_selector(|| "yard sign in for".to_string())
                        .child(format!("a sign-in for {name}, at {address}"))
                        .into_any_element(),
                    yard_field("email", &self.email, palette, cx).into_any_element(),
                    yard_field("password", &self.password, palette, cx).into_any_element(),
                    yard_submit(sign, "yard sign in", busy, palette, cx).into_any_element(),
                ]);
                help = Some(
                    yard_help(
                        format!("The Kratos account's email and password. {session}"),
                        palette,
                    )
                    .into_any_element(),
                );
            }
            // Renaming stands in the renamed card, where its menu stood,
            // not here under the cards: nothing of it is asked for here.
            Some(Adding::Rename(_)) => {}
        }
        // What came of the last change, except while a yard is renamed:
        // that is said in its card, beside the form.
        if !matches!(self.adding, Some(Adding::Rename(_)))
            && let Some(said) = &self.yard_said
        {
            asked.push(
                div()
                    .debug_selector(|| "yard said".to_string())
                    .child(said.clone())
                    .into_any_element(),
            );
        }
        asked.extend(help);
        asked
    }

    pub(crate) fn toggle_decided(&mut self, cx: &mut Context<Self>) {
        self.change(cx, |m| m.toggle_decided());
    }

    pub(crate) fn toggle_waiting(&mut self, cx: &mut Context<Self>) {
        self.change(cx, |m| {
            m.toggle_waiting();
            Vec::new()
        });
    }

    pub(crate) fn show_more_decided(&mut self, cx: &mut Context<Self>) {
        self.change(cx, |m| {
            m.show_more_decided();
            Vec::new()
        });
    }

    /// Decisions' list: the open questions, each with what it wants,
    /// grouped by topic where a topic has two asks or more, with the
    /// "Beads to move" section under them (the rows of the beads parked
    /// for the person's move), which folds to its head, or on a yard too
    /// old for the call one quiet line in its place, and the
    /// Decided toggle at the list's end: decided asks are unbounded
    /// history, and without the toggle they would push open decisions
    /// off a short list. Open, the history has its filter with its chips,
    /// and its days with their rows, look-alikes folded, a page of them
    /// and a row for the rest.
    pub(crate) fn list_decisions(
        &self,
        model: &Model,
        palette: Palette,
        top: f32,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let now = chrono::Utc::now();
        let mut body = self.list_body();
        let decisions = model.decisions_count();
        let decided = model.decided_count();
        body = body.child(section_head("Decisions", decide_words(decisions), palette));
        let groups = model.decision_groups(&now);
        let several = groups.len() > 1;
        for group in groups {
            // A topic above a single ask is noise, and so is one above the
            // only group: the head stands only where two asks or more share
            // the topic, beside another topic.
            let asks: usize = group.rows.iter().map(|row| row.asks.len().max(1)).sum();
            if several && asks > 1 {
                body = body.child(
                    div()
                        .debug_selector({
                            let topic = group.topic.clone();
                            move || format!("decisions topic {topic}")
                        })
                        .px_3()
                        .pt_1()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.faint)
                        .child(group.topic),
                );
            }
            for row in group.rows {
                body = body.child(self.decision_row(row, palette, cx));
            }
        }
        // The beads parked for the person's move stand under the asks: a
        // question asks for words, a parked bead for a move. The fold is
        // named for what the person does with them, "Beads to move". The
        // head above it counts what is "to decide", and one word,
        // "waiting", would name both. A yard from before the call has no
        // section: its words say so in its place, faint and in no red, as the
        // Lines pane does, and the yard's own refusal is never shown. Folded,
        // the head stands with its count, and the rows go.
        if !model.waiting_too_old() {
            let folded = model.waiting_folded();
            body = body.child(
                fold_head(
                    "waiting",
                    "Beads to move",
                    bead_words(model.parked_count()),
                    !folded,
                    palette,
                )
                .on_click(cx.listener(|this, _, _, cx| this.toggle_waiting(cx))),
            );
            if !folded {
                for row in model.waiting_rows(&now) {
                    body = body.child(self.waiting_row(row, palette, cx));
                }
            }
        } else {
            body = body.child(
                div()
                    .debug_selector(|| "waiting too old".to_string())
                    .px_3()
                    .py_1()
                    .text_sm()
                    .text_color(palette.faint)
                    .child(model::WAITING_TOO_OLD),
            );
        }
        if decided > 0 {
            let open = model.decided_expanded();
            // The dealt-with stands against what is above it, with no gap:
            // the fold's own top margin is for a head that starts a section.
            body = body.child(
                fold_head("decided", "Decided", decided_words(decided), open, palette)
                    .mt(px(0.))
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_decided(cx))),
            );
            if open {
                body = body.child(self.decided_filter(palette, cx));
                body = body.child(self.decided_chips(model, palette, cx));
                let groups = model.decided_groups(&chrono::Local::now());
                if groups.iter().all(|group| group.rows.is_empty()) {
                    body = body.child(
                        div()
                            .px_3()
                            .py_1()
                            .text_sm()
                            .text_color(palette.faint)
                            .child("Nothing decided matches."),
                    );
                }
                for group in groups {
                    if group.rows.is_empty() {
                        continue;
                    }
                    body = body.child(
                        div()
                            .debug_selector({
                                let day = group.day.clone();
                                move || format!("decided day {day}")
                            })
                            .px_3()
                            .pt_1()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.faint)
                            .child(group.day),
                    );
                    for row in group.rows {
                        match row {
                            DecidedEntry::One(row) => {
                                body = body.child(self.decided_row(row, palette, cx));
                            }
                            DecidedEntry::Fold(fold) => {
                                body = body.child(self.decided_fold(fold.clone(), palette, cx));
                                if fold.open {
                                    for row in fold.rows {
                                        body = body.child(self.decided_row(row, palette, cx));
                                    }
                                }
                            }
                        }
                    }
                }
                let left = model.decided_left();
                if left > 0 {
                    body = body.child(self.decided_more(left, palette, cx));
                }
            }
        }
        if model.is_operator() {
            body = body.child(self.grants_row(model, palette, cx));
            body = body.child(self.teammates_row(model, palette, cx));
        }
        if model.pair_offered() {
            body = body.child(self.pairing_row(model, palette, cx));
        }
        self.column(body, palette, top, window, cx)
    }

    /// The history's last row while asks it finds are not drawn: it says
    /// how many are left, quiet, in the Decided toggle's shape, and a press
    /// draws the next page. It is no row of the keys' list, as the toggle
    /// is none.
    pub(crate) fn decided_more(
        &self,
        left: usize,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id("decided more")
            .debug_selector(|| "decided more".to_string())
            .mx_1()
            .px_2()
            .py_1()
            .rounded_md()
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .hover(|style| style.bg(palette.hover))
            .on_click(cx.listener(|this, _, _, cx| this.show_more_decided(cx)))
            .child(div().text_sm().text_color(palette.faint).child("Show more"))
            .child(
                div()
                    .flex_none()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| "decided more count".to_string())
                    .child(left_words(left)),
            )
    }

    /// The history's filter field: `/` types in it, and every word typed
    /// matches a row's question, asker or outcome.
    pub(crate) fn decided_filter(
        &self,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let input = self.filter.clone();
        let focus = input.focus_handle(cx);
        field(focus, input, palette)
            .id("decided filter")
            .debug_selector(|| "decided filter".to_string())
            .mx_1()
            .mt_1()
            .text_sm()
    }

    /// The history's chips: All with the four outcomes after it, each with
    /// how many rows choosing it would show.
    pub(crate) fn decided_chips(
        &self,
        model: &Model,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut chips = div().mx_1().px_2().py_1().flex().flex_wrap().gap_1();
        for (chip, count) in model.decided_chip_counts() {
            let word = match chip {
                None => "All".to_string(),
                Some(outcome) => outcome.chip().to_string(),
            };
            let current = model.decided_chip() == chip;
            let id = match chip {
                None => "decided chip all".to_string(),
                Some(outcome) => format!("decided chip {}", outcome.word()),
            };
            chips = chips.child(
                div()
                    .id(SharedString::from(id.clone()))
                    .debug_selector({
                        let id = id.clone();
                        move || id.clone()
                    })
                    .px_2()
                    .rounded_full()
                    .border_1()
                    .text_xs()
                    .cursor_pointer()
                    .when(current, |el| {
                        el.bg(palette.accent)
                            .border_color(palette.accent)
                            .text_color(palette.on_accent)
                    })
                    .when(!current, |el| {
                        el.border_color(palette.line)
                            .text_color(palette.faint)
                            .hover(|style| style.bg(palette.hover))
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.change(cx, |m| {
                            m.set_decided_chip(chip);
                            Vec::new()
                        })
                    }))
                    .child(format!("{word} {count}")),
            );
        }
        chips
    }

    /// One bead's open row: a lamp and the bead's title, then a line for
    /// each open ask, what it wants and its number before the question,
    /// and last who asks and how long the oldest ask waits, with the
    /// bead's short id at the right. The lamp is the accent's: every row
    /// here waits on the person.
    pub(crate) fn decision_row(
        &self,
        row: DecisionRow,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let sel = Selected::Decision(row.bead.clone());
        let mono = select::mono(cx);
        let mut col = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .gap(px(2.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(ROW_LAMP_GAP))
                    .min_w_0()
                    .child(
                        div()
                            .debug_selector(|| format!("decision lamp {}", row.bead))
                            .size(px(ROW_LAMP))
                            .rounded_full()
                            .flex_none()
                            .bg(palette.accent),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(rems(TYPE_LABEL))
                            .font_weight(FontWeight::SEMIBOLD)
                            .debug_selector(|| format!("decision head {}", row.bead))
                            .child(shorten(&row.head, ROW_TITLE_CHARS)),
                    ),
            );
        // The asks' lines and the last line stand under the title, past
        // the lamp.
        let indent = px(ROW_LAMP + ROW_LAMP_GAP);
        for ask in &row.asks {
            let seq = ask.ask;
            col = col.child(
                div()
                    .pl(indent)
                    .truncate()
                    .text_xs()
                    .text_color(palette.faint)
                    .debug_selector(|| format!("decision ask {seq} {}", row.bead))
                    .child(
                        div()
                            .flex()
                            .min_w_0()
                            .gap_1()
                            .child(
                                div()
                                    .flex_none()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(palette.text)
                                    .child(format!("{} {seq}", ask.kind)),
                            )
                            .child(div().flex_none().child("·"))
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .child(shorten(&ask.head, ROW_ASK_CHARS)),
                            ),
                    ),
            );
        }
        let mut short = div()
            .flex_none()
            .text_size(rems(TYPE_CODE))
            .text_color(palette.faint)
            .debug_selector(|| format!("decision id {}", row.bead))
            .child(row.short);
        if let Some(mono) = mono {
            short = short.font_family(mono);
        }
        col = col.child(
            div()
                .pl(indent)
                .flex()
                .items_baseline()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .min_w_0()
                        .truncate()
                        .text_xs()
                        .text_color(palette.faint)
                        .debug_selector(|| format!("decision sub {}", row.bead))
                        .child(row.sub),
                )
                .child(short),
        );
        div()
            .id(SharedString::from(format!("decision {}", row.bead)))
            .debug_selector(|| format!("decision row {}", row.bead))
            .mx_1()
            .px_2()
            .py_1p5()
            .rounded_md()
            .flex()
            .cursor_pointer()
            .border_1()
            .border_color(rgba(0x00000000))
            .when(row.selected, |el| el.bg(palette.open))
            .when(!row.selected, |el| {
                el.hover(|style| style.border_color(palette.line))
            })
            .on_click(cx.listener(move |this, _, window, cx| this.select(sel.clone(), window, cx)))
            .child(col)
    }

    /// The grants row, last of decisions where the window is the
    /// operator's: what is granted, in so many calls, or that none is,
    /// or that the yard is too old to say, or that it says them on its
    /// machine alone.
    pub(crate) fn grants_row(
        &self,
        model: &Model,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let selected = model.selected() == Some(&Selected::Grants);
        let sub = match (model.grants_too_old(), model.grants_on_the_yard()) {
            (true, _) => "yard too old".to_string(),
            (false, true) => "read on the yard's machine".to_string(),
            (false, false) => {
                let calls: usize = model.grant_records().iter().map(|g| g.calls.len()).sum();
                match calls {
                    0 => "none granted".to_string(),
                    1 => "1 call granted".to_string(),
                    n => format!("{n} calls granted"),
                }
            }
        };
        div()
            .id("row grants")
            .debug_selector(|| "row grants".to_string())
            .mx_1()
            .px_2()
            .py_1p5()
            .rounded_md()
            .flex()
            .flex_col()
            .cursor_pointer()
            .border_1()
            .border_color(rgba(0x00000000))
            .when(selected, |el| el.bg(palette.open))
            .when(!selected, |el| {
                el.hover(|style| style.border_color(palette.line))
            })
            .on_click(cx.listener(|this, _, window, cx| this.select(Selected::Grants, window, cx)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .debug_selector(|| "grants head".to_string())
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .text_size(rems(TYPE_LABEL))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Grants"),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(palette.faint)
                    .truncate()
                    .debug_selector(|| "grants sub".to_string())
                    .child(sub),
            )
    }

    /// The teammates row, last of decisions where the window is the
    /// operator's: what is invited, in so many teammates, or that none
    /// is, or that the yard is too old to say.
    pub(crate) fn teammates_row(
        &self,
        model: &Model,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let selected = model.selected() == Some(&Selected::Teammates);
        let sub = match model.teammates_too_old() {
            true => "yard too old".to_string(),
            false => match model.teammate_records().len() {
                0 => "no teammates yet".to_string(),
                1 => "1 teammate".to_string(),
                n => format!("{n} teammates"),
            },
        };
        div()
            .id("row teammates")
            .debug_selector(|| "row teammates".to_string())
            .mx_1()
            .px_2()
            .py_1p5()
            .rounded_md()
            .flex()
            .flex_col()
            .cursor_pointer()
            .border_1()
            .border_color(rgba(0x00000000))
            .when(selected, |el| el.bg(palette.open))
            .when(!selected, |el| {
                el.hover(|style| style.border_color(palette.line))
            })
            .on_click(
                cx.listener(|this, _, window, cx| this.select(Selected::Teammates, window, cx)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .debug_selector(|| "teammates head".to_string())
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .text_size(rems(TYPE_LABEL))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Teammates"),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(palette.faint)
                    .truncate()
                    .debug_selector(|| "teammates sub".to_string())
                    .child(sub),
            )
    }

    /// The pairing row, last of decisions where the window runs as
    /// the yard's operator on its machine: the device a pairing is open
    /// for, or that none is, or that the yard is too old to say.
    pub(crate) fn pairing_row(
        &self,
        model: &Model,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let selected = model.selected() == Some(&Selected::Pairing);
        let sub = match (model.pairing_too_old(), model.pairing()) {
            (true, _) => "yard too old".to_string(),
            (false, Some(pairing)) => format!("open for {}", model::plain(&pairing.device)),
            (false, None) => "no pairing open".to_string(),
        };
        div()
            .id("row pairing")
            .debug_selector(|| "row pairing".to_string())
            .mx_1()
            .px_2()
            .py_1p5()
            .rounded_md()
            .flex()
            .flex_col()
            .cursor_pointer()
            .border_1()
            .border_color(rgba(0x00000000))
            .when(selected, |el| el.bg(palette.open))
            .when(!selected, |el| {
                el.hover(|style| style.border_color(palette.line))
            })
            .on_click(cx.listener(|this, _, window, cx| this.select(Selected::Pairing, window, cx)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .debug_selector(|| "pairing head".to_string())
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .text_size(rems(TYPE_LABEL))
                            .font_weight(FontWeight::MEDIUM)
                            .child("Pair a phone"),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(palette.faint)
                    .truncate()
                    .debug_selector(|| "pairing sub".to_string())
                    .child(sub),
            )
    }

    /// One dealt with, in the open rows' shape (`decided_lines`), with the
    /// bead's short id at the right of its outcome's line.
    pub(crate) fn decided_row(
        &self,
        row: DecidedRow,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let sel = Selected::Decided(row.bead.clone(), row.ask);
        let mut short = div()
            .flex_none()
            .text_size(rems(TYPE_CODE))
            .text_color(palette.faint)
            .debug_selector(|| format!("decided id {} {}", row.bead, row.ask))
            .child(row.short.clone());
        if let Some(mono) = select::mono(cx) {
            short = short.font_family(mono);
        }
        let lines = decided_lines(
            row.mark,
            &row.outcome,
            &row.head,
            &row.detail,
            format!("{} {}", row.bead, row.ask),
            "decided",
            short,
            palette,
        );
        lines
            .id(SharedString::from(format!(
                "decided {} {}",
                row.bead, row.ask
            )))
            .when(row.selected, |el| el.bg(palette.open))
            .when(!row.selected, |el| {
                el.hover(|style| style.border_color(palette.line))
            })
            .on_click(cx.listener(move |this, _, window, cx| this.select(sel.clone(), window, cx)))
    }

    /// Several look-alikes folded into one row, in a dealt with row's
    /// shape: the first's question with the shared outcome and asker. A
    /// click opens the fold to show them, or folds it shut again; the
    /// count with ▸ and ▾ stands where one row's bead id does.
    pub(crate) fn decided_fold(
        &self,
        fold: DecidedFold,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let bead = fold.bead.clone();
        let ask = fold.ask;
        let mark = if fold.open { "▾" } else { "▸" };
        let count = div()
            .flex_none()
            .text_xs()
            .text_color(palette.faint)
            .child(format!("×{} {mark}", fold.rows.len()));
        decided_lines(
            fold.mark,
            &fold.outcome,
            &fold.head,
            &fold.detail,
            format!("{} {}", fold.bead, fold.ask),
            "decided fold",
            count,
            palette,
        )
        .id(SharedString::from(format!(
            "decided fold {} {}",
            fold.bead, fold.ask
        )))
        .debug_selector(|| format!("decided fold {} {}", fold.bead, fold.ask))
        .hover(|style| style.border_color(palette.line))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.change(cx, |m| {
                m.toggle_decided_fold(&bead, ask);
                Vec::new()
            })
        }))
    }

    /// A bead's open asks, each as its card: the yard, the bead and the
    /// topic once in the head, then the question, the asker, the context
    /// with the bead whole one disclosure away, the numbered options with
    /// the recommended one preselected, a line for words, and the answer
    /// button with Later, ask-about-this and Decline beside it.
    pub(crate) fn talk_decision(
        &self,
        model: &Model,
        bead: &str,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let pane = div().flex_1().min_w_0().h_full().flex().flex_col();
        let now = chrono::Utc::now();
        let Some(detail) = model.decision_detail(bead, &now) else {
            return pane
                .items_center()
                .justify_center()
                .text_color(palette.faint)
                .child("Its questions are answered.");
        };
        // The head: the yard, the bead, its topic where it has one and
        // who built and reviewed it where either is known, small, over
        // the bead's title, as the pick's breadcrumb stands over it;
        // beside them the stage dots of its flow with the stage named,
        // and how many asks wait on the person reading. The row wraps,
        // so a narrow pane puts the dots and the count under the words
        // and squeezes none.
        let title = match detail.title.is_empty() {
            true => detail.bead.clone(),
            false => detail.title.clone(),
        };
        let head = div()
            .px_4()
            .py_2()
            .border_b_1()
            .border_color(palette.line)
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap_x_4()
            .gap_y_2()
            .child(
                div()
                    .flex_1()
                    .min_w(px(240.))
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .debug_selector(|| format!("in {} {}", self.yard, detail.bead))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .text_size(rems(TYPE_META))
                            .text_color(palette.faint)
                            .child(
                                div()
                                    .debug_selector(|| {
                                        format!("in {} {} dot", self.yard, detail.bead)
                                    })
                                    .flex_none()
                                    .size(px(8.))
                                    .rounded_full()
                                    .bg(yard_color(&self.yard, palette.dark)),
                            )
                            .child(decision_head(
                                &self.yard,
                                &detail.bead,
                                &detail.topic,
                                &detail.built_by,
                                &detail.reviewed_by,
                            )),
                    )
                    .child(
                        div()
                            .debug_selector(|| format!("title {}", detail.bead))
                            .truncate()
                            .text_size(rems(TYPE_LABEL))
                            .font_weight(FontWeight::MEDIUM)
                            .child(title),
                    ),
            )
            .children(self.stage_dots(model, bead, &detail, palette))
            .child(
                div()
                    .debug_selector(|| format!("asks on you {}", detail.bead))
                    .flex_none()
                    .h(px(24.))
                    .px(px(10.))
                    .rounded_full()
                    .border_1()
                    .border_color(palette.accent)
                    .flex()
                    .items_center()
                    .text_size(rems(TYPE_META))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(palette.text_accent)
                    .child(asks_on_you(detail.asks.len())),
            );
        let list = div()
            .id("decision")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.decision)
            .px_4()
            .py_3()
            .flex()
            .flex_col();
        let list = if !model.has_bead(bead) {
            list.child(div().text_color(palette.faint).child("Reading the bead."))
        } else {
            // The cards flat on the pane, each after the first ruled off
            // from the one above with a thin line.
            let mut list = list.children(detail.asks.iter().enumerate().map(|(ix, ask)| {
                div()
                    .pb_4()
                    .when(ix > 0, |el| {
                        el.pt_4().border_t_1().border_color(palette.line)
                    })
                    .child(self.decision_card(&detail, ask, palette, cx))
            }));
            // The bead's thread under the asks, as the pick draws it:
            // its notes in order, each with its files, the images
            // compact, so a short card and its thread stand together
            // at 960x640. The bead whole stays one disclosure away on
            // each card besides.
            if !detail.notes.is_empty() {
                list = list.child(self.decision_thread(&detail, palette, cx));
            }
            list
        };
        pane.children(self.reconnect_strip(palette))
            .child(head)
            .child(list)
            .when(!model.notice.is_empty(), |el| {
                el.child(
                    div()
                        .px_4()
                        .py_1()
                        .text_sm()
                        .text_color(palette.warn)
                        .child(format!("! {}", model.notice)),
                )
            })
    }

    /// The stage dots of the bead's flow with the stage named, in the
    /// Decide head: one dot per stage in travel order, the stages
    /// behind filled strong, the bead's own rust, the stages ahead
    /// hollow, with the stage's name beside them. Where the flows name
    /// no flow for the bead, the stage alone, as its whole reads it;
    /// nothing until that is read either.
    pub(crate) fn stage_dots(
        &self,
        model: &Model,
        bead: &str,
        detail: &DecisionDetail,
        palette: Palette,
    ) -> Option<impl IntoElement + use<>> {
        let route = model.route_for(bead);
        let here = route
            .as_ref()
            .and_then(|route| route.stages.get(route.current));
        let name = here
            .map(|stage| stage.name.clone())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| detail.stage.clone());
        if name.is_empty() {
            return None;
        }
        let track = route.as_ref().filter(|_| here.is_some()).map(|route| {
            let mut track = div().flex().flex_row().items_center();
            for ix in 0..route.stages.len() {
                if ix > 0 {
                    track = track.child(
                        div()
                            .flex_none()
                            .w(px(STAGE_TIE))
                            .h(px(1.5))
                            .bg(palette.strong),
                    );
                }
                let dot = div().flex_none().size(px(ROW_LAMP)).rounded_full();
                track = track.child(if ix < route.current {
                    dot.bg(palette.strong)
                } else if ix == route.current {
                    dot.bg(palette.accent)
                } else {
                    dot.border_1().border_color(palette.strong)
                });
            }
            track
        });
        Some(
            div()
                .debug_selector(|| format!("stops {bead}"))
                .flex_none()
                .flex()
                .flex_row()
                .items_center()
                .gap_1p5()
                .text_size(rems(TYPE_META))
                .text_color(palette.faint)
                .children(track)
                .child(name),
        )
    }

    /// The bead's thread under its open asks: its notes newest first, each
    /// headed by who wrote it, its time and the bead, its words beside
    /// its image where it has one (the pick's thread: words left, the
    /// picture right in its frame), its other files under its words as
    /// one quiet line each, so the thread and a short card stand
    /// together. A note's words select and copy as a card's do, under
    /// their own keys.
    pub(crate) fn decision_thread(
        &self,
        detail: &DecisionDetail,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let bead = detail.bead.clone();
        let bead_key = select::key(("thread", bead.as_str()));
        let mut thread = div()
            .debug_selector(|| format!("thread {bead}"))
            .border_t_1()
            .border_color(palette.line)
            .pt_3()
            .flex()
            .flex_col()
            .gap_3();
        // The newest first, right under the asks: the latest note is most
        // often what the ask is about, and a long thread does not push it
        // out of view. Each keeps its own index, so its keys do not move.
        for (ix, note) in detail.notes.iter().enumerate().rev() {
            let at = model::clock_time(&note.at, &chrono::Local);
            let head = match at.is_empty() {
                true => format!("{} · on {bead}", note.author),
                false => format!("{} · {at} · on {bead}", note.author),
            };
            let images: Vec<_> = note
                .files
                .iter()
                .filter(|file| model::is_image(file) && file.not_taken.is_empty())
                .collect();
            let mut words = div().flex().flex_col().gap_1().child(
                div()
                    .text_size(rems(TYPE_BODY))
                    .debug_selector(|| format!("thread note {bead} {ix}"))
                    .child(select::text(
                        expanded_text_key(bead_key, "note", ix, &note.text),
                        note.text.clone(),
                        &self.selection,
                    )),
            );
            for file in note
                .files
                .iter()
                .filter(|file| !model::is_image(file) || !file.not_taken.is_empty())
            {
                words = words.child(self.thread_file(file, palette, cx));
            }
            let mut row = div().flex().gap_4().child(words.flex_1().min_w_0());
            if !images.is_empty() {
                let mut pics = div()
                    .flex_none()
                    .w(px(THREAD_PIC_W))
                    .flex()
                    .flex_col()
                    .gap_2();
                for file in images {
                    pics = pics.child(self.thread_file(file, palette, cx));
                }
                row = row.child(pics);
            }
            let block = div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .debug_selector(|| format!("thread head {bead} {ix}"))
                        .text_size(rems(TYPE_META))
                        .text_color(palette.faint)
                        .child(head),
                )
                .child(row);
            thread = thread.child(block);
        }
        thread
    }

    /// One file of a thread note: an image in its frame, where
    /// [`Footplate::picture`] stands it at its reading height; a page or
    /// a file as one quiet line of its name and size, its Open and run
    /// on the bead whole behind the card's disclosure, so the thread
    /// names it without drawing its row twice. A click opens an image
    /// full size as there, a right press saves it; while its bytes are
    /// not there the frame says so and a click opens it all the same.
    pub(crate) fn thread_file(
        &self,
        file: &crate::api::BlobRef,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        if !model::is_image(file) || !file.not_taken.is_empty() {
            if !file.not_taken.is_empty() {
                return self
                    .picture(file, false, false, palette, cx)
                    .into_any_element();
            }
            let label = match file.name.is_empty() {
                false => format!("{}, {}", file.name, model::size(file.bytes)),
                true => match model::is_page(file) {
                    true => format!("page, {}", model::size(file.bytes)),
                    false => format!("file, {}", model::size(file.bytes)),
                },
            };
            let hash = file.hash.clone();
            return div()
                .debug_selector(move || format!("thread file {hash}"))
                .text_size(rems(TYPE_META))
                .text_color(palette.faint)
                .child(label)
                .into_any_element();
        }
        // The frame the thread's picture stands in, with its border and
        // ground: the image scaled inside it, or the words for why no
        // image stands there.
        let frame = div()
            .w(px(THREAD_PIC_W))
            .h(px(THREAD_PIC_H))
            .flex_none()
            .rounded(px(RADIUS_CONTROL))
            .border_1()
            .border_color(palette.strong)
            .bg(palette.side)
            .p_2()
            .flex()
            .items_center()
            .justify_center()
            .id(SharedString::from(format!("thread image {}", file.hash)))
            .debug_selector({
                let hash = file.hash.clone();
                move || format!("thread image {hash}")
            })
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default());
        let line = |text: String| {
            div()
                .text_size(rems(TYPE_META))
                .text_color(palette.faint)
                .child(text)
                .into_any_element()
        };
        match self.images.get(&file.hash) {
            Some(Picture::Here(image)) => {
                let hash = file.hash.clone();
                let bytes = file.bytes;
                let name = file.name.clone();
                frame
                    .on_mouse_down(MouseButton::Right, {
                        let (hash, name) = (hash.clone(), name.clone());
                        cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                            window.prevent_default();
                            this.open_picture_menu(hash.clone(), name.clone(), event.position, cx)
                        })
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_image(hash.clone(), bytes, name.clone(), window, cx)
                    }))
                    .child(
                        img(image.clone())
                            .max_w_full()
                            .max_h_full()
                            .object_fit(ObjectFit::ScaleDown),
                    )
                    .into_any_element()
            }
            Some(Picture::Not(why)) => frame.child(line(why.clone())).into_any_element(),
            Some(Picture::Asked) | None => {
                let hash = file.hash.clone();
                let bytes = file.bytes;
                let name = file.name.clone();
                let text = format!("image, {}…", model::size(file.bytes));
                frame
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_image(hash.clone(), bytes, name.clone(), window, cx)
                    }))
                    .child(
                        div()
                            .text_size(rems(TYPE_META))
                            .text_color(palette.faint)
                            .child(text),
                    )
                    .into_any_element()
            }
        }
    }

    /// One card of an open bead: the question, the largest text; the asker
    /// and how long it waits; the context with the bead whole one
    /// disclosure away; the numbered options, the recommended one
    /// preselected and marked with its why under it; a line for words; the
    /// answer button with Later, ask-about-this and Decline beside it, each
    /// key on its control. Only the current one takes the keys, the line
    /// and the buttons.
    pub(crate) fn decision_card(
        &self,
        detail: &DecisionDetail,
        ask: &DecisionAsk,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let bead_id = detail.bead.clone();
        let ask_seq = ask.ask;
        let card_key = select::key((bead_id.clone(), ask_seq));
        let current = ask.current;
        // The option Enter and the button answer: the pick, else the
        // recommendation, which stands preselected.
        let answered = if ask.chosen == 0 {
            ask.recommend
        } else {
            ask.chosen
        };
        // The Ask card's frame: flat on the pane, as the pick draws it. The
        // pane rules one card off from the next; everything the card says
        // stands inside it.
        let mut card = div()
            .debug_selector(|| format!("card {bead_id} {ask_seq}"))
            .flex()
            .flex_col()
            .gap_2p5();
        // How the card approves: nowhere where the yard takes no approval
        // from this window, after the ask's number is typed back where a
        // weighty card has no gate.
        let elsewhere = self.approve_elsewhere.contains(&(bead_id.clone(), ask_seq));
        let approval = approval(!ask.weight.is_empty(), self.unlock_available(), elsewhere);
        let sends = !matches!(approval, Approval::Nowhere);
        // The yard's weight first, each class's line as the terminal
        // prints it, in rust: a weighty card reads apart from a routine
        // one, and never in red.
        for (n, weight) in ask.weight.iter().enumerate() {
            let line = model::weight_line(&weight.class, &weight.line);
            card = card.child(
                div()
                    .debug_selector(|| format!("weight {bead_id} {ask_seq} {n}"))
                    .text_color(palette.text_accent)
                    .child(select::text(
                        expanded_text_key(card_key, "weight", n, &line),
                        line,
                        &self.selection,
                    )),
            );
        }
        // The door a pairing card's approval crosses, beside the yard's
        // weight: the approver reads it before the approval goes.
        let pairs = sends && ask.run.as_ref().is_some_and(is_device_pair_run);
        if pairs && let Some(client) = self.client.as_ref() {
            let line = pair_door(client.sealed(), client.is_remote());
            card = card.child(
                div()
                    .debug_selector(|| format!("door {bead_id} {ask_seq}"))
                    .text_color(palette.text_accent)
                    .child(select::text(
                        expanded_text_key(card_key, "door", 0, line),
                        line.to_string(),
                        &self.selection,
                    )),
            );
        }
        // The question, with how long it waits beside it. The yard and the
        // bead stand once in the head above, not again on every card.
        let asker_marked = ask.run.is_some() || ask.setup.is_some();
        let question = if asker_marked {
            self.marked_question(
                card_key,
                format!("run question {bead_id} {ask_seq}"),
                &ask.question,
                &ask.asker,
                palette,
            )
            .into_any_element()
        } else {
            select::text(
                expanded_text_key(card_key, "question", 0, &ask.question),
                ask.question.clone(),
                &self.selection,
            )
            .into_any_element()
        };
        // Over the question, what waits and on whom: that it waits on the
        // person, what it wants and its number in the accent, then how long
        // and who asks, quiet. At the row's right the bead's id with the
        // ask's number, to select and copy, so a decision named by number
        // is found on its card, and the bead whole one disclosure away.
        // The line is no part of the disclosure: pressing to select it
        // opens nothing. The row wraps, so a card too narrow for both puts
        // them under the words and squeezes neither.
        let mark = if ask.expanded { "▾" } else { "▸" };
        let bead_click = bead_id.clone();
        let line = ask_line(&bead_id, ask_seq);
        let line_key = expanded_text_key(card_key, "ask", 0, &line);
        let asked = [asked_age(&ask.waits), ask.asker.clone()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        card = card.child(
            div()
                .flex()
                .flex_wrap()
                .items_baseline()
                .gap_x_2()
                .text_size(rems(TYPE_META))
                .child(
                    div()
                        .debug_selector(|| format!("waits on {bead_id} {ask_seq}"))
                        .text_color(palette.text_accent)
                        .child(format!("Waiting on you · {} {ask_seq}", ask.kind)),
                )
                .child(
                    div()
                        .debug_selector(|| format!("asked {bead_id} {ask_seq}"))
                        .flex_grow()
                        .text_color(palette.faint)
                        .child(asked),
                )
                // One line to select and copy whole, so the bead's id keeps
                // the line's quiet colour rather than the link's.
                .child(
                    div()
                        .debug_selector(|| format!("ask line {bead_id} {ask_seq}"))
                        .min_w_0()
                        .text_color(palette.faint)
                        .child(select::verbatim(line_key, line, &self.selection)),
                )
                .child(
                    div()
                        .id(SharedString::from(format!("expand {bead_id} {ask_seq}")))
                        .debug_selector(|| format!("expand {bead_id} {ask_seq}"))
                        .text_color(palette.faint)
                        .cursor_pointer()
                        .hover(|style| style.text_color(palette.text))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.expand_for(bead_click.clone(), ask_seq, cx)
                        }))
                        .child(format!("{mark} details (e)")),
                ),
        );
        // The question. The current ask's stands at the question size, the
        // card the pane is for; the rest at the title size, as the pick's
        // second question does.
        card = card.child(
            div()
                .min_w(px(ASK_QUESTION_MIN))
                .debug_selector(|| format!("question {bead_id} {ask_seq}"))
                .text_size(rems(if current { TYPE_QUESTION } else { TYPE_TITLE }))
                .font_weight(FontWeight::SEMIBOLD)
                .child(question),
        );
        // The context lines: text as it was written, and no block in it.
        // A run or setup ask's each begin with who says them, and wrap
        // under the words.
        for (line, text) in ask.context.iter().enumerate() {
            if asker_marked {
                card = card.child(
                    div()
                        .text_size(rems(TYPE_BODY))
                        .child(self.marked_asker_line(
                            card_key,
                            "context",
                            line,
                            format!("run context {bead_id} {ask_seq} {line}"),
                            text,
                            &ask.asker,
                            palette,
                        )),
                );
            } else {
                card = card.child(
                    div()
                        .debug_selector(|| format!("context {bead_id} {ask_seq} {line}"))
                        .text_size(rems(TYPE_BODY))
                        .child(select::text(
                            expanded_text_key(card_key, "context", line, text),
                            text.clone(),
                            &self.selection,
                        )),
                );
            }
        }
        // The bead whole, opened from the row over the question, under
        // the context it adds to.
        if ask.expanded {
            card = card.child(self.expanded_body(
                card_key,
                Expanded {
                    body: &detail.body,
                    notes: &detail.notes,
                    about: &ask.about,
                    doc: &detail.doc,
                },
                palette,
                cx,
            ));
        }
        // The options numbered, each with its consequence, `you decide`
        // last as the yard lists it. A secret ask has none and wants no
        // words: what it asks for is said in the yard's line, as it is.
        // Nor has a run ask: the yard's block is the last of its card, with
        // the control that approves directly under it. A setup ask has the
        // yard's block with its account choice first, and its presets to
        // pick from under it, as an ordinary ask's options.
        if let Some(secret) = &ask.secret {
            card = card.child(self.secret_lines(&bead_id, ask_seq, secret, palette));
        } else if let Some(run) = &ask.run {
            card = card.child(self.run_block(&bead_id, ask_seq, run, true, palette, cx));
        } else {
            if let Some(setup) = &ask.setup {
                card = card.child(self.setup_block(&bead_id, ask_seq, setup, true, palette, cx));
                card = card.child(self.setup_candidates(
                    &bead_id,
                    ask_seq,
                    setup,
                    &ask.account,
                    palette,
                    cx,
                ));
            }
            if ask.options.is_empty() {
                if ask.setup.is_none() {
                    card = card.child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("words wanted"),
                    );
                }
            } else {
                let mut marked = false;
                for option in &ask.options {
                    // A lever: its key plate, its label and its
                    // consequence. The chosen lever alone is outlined in
                    // the accent, never filled: the card's one filled
                    // control is its Answer button below (the one-fill
                    // rule). The recommended one is chosen until another
                    // is picked, so two outlines never stand at once. Every
                    // plate holds its number alone, so the labels line up;
                    // the recommended lever's plate is in the accent, the
                    // word "recommended" stands at its row's right, and its
                    // why under its label, inside the lever.
                    let n = option.number;
                    let recommended = ask.recommend != 0 && n == ask.recommend;
                    let chosen = n == answered;
                    let (label, consequence) = lever_words(&option.label, &option.consequence);
                    let bead_click = bead_id.clone();
                    let why = (recommended && !ask.why.is_empty()).then(|| ask.why.clone());
                    marked |= recommended;
                    card = card.child(
                        div()
                            .id(SharedString::from(format!("pick {bead_id} {ask_seq} {n}")))
                            .debug_selector(|| format!("pick {bead_id} {ask_seq} {n}"))
                            .cursor_pointer()
                            .rounded(px(RADIUS_CONTROL))
                            .border_1()
                            .border_color(match chosen {
                                true => palette.accent,
                                false => rgba(0x00000000),
                            })
                            .px_2()
                            .py_1()
                            .min_h(px(LEVER_H))
                            .flex()
                            .flex_row()
                            .items_start()
                            .gap_2()
                            .when(!chosen, |el| {
                                el.hover(|style| style.border_color(palette.line))
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.pick_for(bead_click.clone(), ask_seq, n, cx)
                            }))
                            .child(
                                div()
                                    .flex_none()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .border_1()
                                    .border_color(match chosen || recommended {
                                        true => palette.accent,
                                        false => palette.line,
                                    })
                                    .text_size(rems(TYPE_META))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(match recommended {
                                        true => palette.text_accent,
                                        false => palette.faint,
                                    })
                                    .child(lever_plate(n)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    // The words wrap on their own, and the
                                    // tag keeps its column at the right.
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_baseline()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .flex()
                                                    .flex_row()
                                                    .flex_wrap()
                                                    .items_baseline()
                                                    .gap_x_2()
                                                    .child(
                                                        div()
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .text_size(rems(TYPE_LABEL))
                                                            .child(label),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(rems(TYPE_META))
                                                            .text_color(palette.faint)
                                                            .child(consequence),
                                                    ),
                                            )
                                            .when(recommended, |el| {
                                                el.child(
                                                    div()
                                                        .debug_selector(|| {
                                                            format!(
                                                                "recommended {bead_id} {ask_seq} {n}"
                                                            )
                                                        })
                                                        .flex_none()
                                                        .text_size(rems(TYPE_META))
                                                        .text_color(palette.text_accent)
                                                        .child(RECOMMENDED),
                                                )
                                            }),
                                    )
                                    .children(why.map(|why| {
                                        div()
                                            .debug_selector(|| {
                                                format!("recommended why {bead_id} {ask_seq}")
                                            })
                                            .text_xs()
                                            .text_color(palette.faint)
                                            .child(why)
                                    })),
                            ),
                    );
                }
                // A recommendation the options do not hold stands past them, as
                // nothing else names it.
                if ask.recommend != 0 && !marked {
                    card = card.child(
                        div()
                            .ml_6()
                            .text_xs()
                            .text_color(palette.faint)
                            .child(recommended_line(&ask.why)),
                    );
                }
            }
        }
        if current {
            // A run or setup ask takes words only with a decline: its card
            // has the line while one is asked for. A setup ask's field
            // takes its account otherwise, as a secret ask's takes its
            // value.
            let declining = self.declining.as_ref() == Some(&(bead_id.clone(), ask_seq));
            // A line for words, or the field of a secret in its place. A
            // setup ask's account is typed in the secret field too. A card
            // that sends nowhere draws no field: its approval goes in a
            // terminal, and a decline's words take the line.
            let secret_field = (ask.secret.is_some() || ask.setup.is_some()) && !declining && sends;
            let input = match secret_field {
                true => self.secret.clone(),
                false => self.line.clone(),
            };
            let focus = input.focus_handle(cx);
            let kind = if secret_field { "secret" } else { "words" };
            // Where this yard takes no approval from this window, the
            // card says where it does, in place of the control. Where a
            // weighty card has no gate, the card takes the ask's number
            // back instead, before its value or account field.
            if (ask.run.is_some() || ask.setup.is_some()) && elsewhere {
                let command = match &ask.setup {
                    Some(setup) => model::setup_approve_command(&bead_id, ask_seq, setup),
                    None => model::approve_command(&bead_id, ask_seq),
                };
                card =
                    card.child(self.approve_elsewhere_lines(&bead_id, ask_seq, command, palette));
            } else if matches!(approval, Approval::TypeBack) {
                let verb = match (&ask.secret, &ask.identity) {
                    (Some(_), _) => "give",
                    (None, Some(_)) => "pick",
                    (None, None) => "approve",
                };
                card = card.child(self.typeback_lines(&bead_id, ask_seq, verb, palette, cx));
            } else if !sends && ask.secret.is_some() {
                card = card.child(self.approve_elsewhere_lines(
                    &bead_id,
                    ask_seq,
                    format!("yardr bead answer {bead_id} {ask_seq} --secret"),
                    palette,
                ));
            }
            // A setup ask whose ask carries a host template takes the
            // host in a field of its own above the account: an address,
            // typed in the clear. Where the card sends nowhere the
            // approval goes in a terminal, and draws no field.
            let hosted = ask
                .setup
                .as_ref()
                .is_some_and(|setup| !setup.host_template.is_empty());
            let host_focus = self.host.focus_handle(cx);
            let host = self.host.clone();
            // A secret's field stands beside its toggle: Show draws what
            // is typed as typed, for a look, Hide draws marks again.
            let toggle = secret_field.then(|| {
                show_button(
                    format!("show {bead_id} {ask_seq}"),
                    self.secret_shown,
                    palette,
                    cx,
                )
            });
            card = card.when(hosted && !declining && sends, |card| {
                card.child(
                    field(host_focus, host, palette)
                        .debug_selector(|| format!("host {bead_id} {ask_seq}"))
                        // A setup ask's host field takes its approval's
                        // key too, with what the field holds, as the
                        // account field does.
                        .key_context(SETUP_SECRET_CONTEXT),
                )
            });
            card = card.when(
                (ask.run.is_none() || ask.setup.is_some() || declining) && (sends || declining),
                |card| {
                    let boxed = field(focus, input, palette)
                        .debug_selector(|| format!("{kind} {bead_id} {ask_seq}"))
                        // A setup ask's account field takes its approval's key
                        // too, with what the field holds.
                        .when(ask.setup.is_some() && secret_field, |el| {
                            el.key_context(SETUP_SECRET_CONTEXT)
                        });
                    match toggle {
                        Some(toggle) => card.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(boxed.flex_1())
                                .child(toggle),
                        ),
                        None => card.child(boxed),
                    }
                },
            );
            // Under the line the files attached to the ask's note, with
            // why a marked one cannot go: what Enter sends with the
            // words. A file is drawn by its name: footplate shows no
            // preview of one, and an image small.
            let files = self.card_files(&bead_id, ask_seq);
            if !files.is_empty() {
                card = card
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_3()
                            .children(files.iter().map(|a| self.attachment(a, palette, cx))),
                    )
                    .children(draft::reasons(files).into_iter().map(|why| {
                        div()
                            .text_sm()
                            .text_color(palette.warn)
                            .child(format!("! {why}"))
                    }));
            }
            // Why the yard did not take the card's answer, in place: its
            // refusal or footplate's words for what else went wrong, as
            // other refusals are said. A new answer clears it.
            if let Some(why) = self.refused.get(&(bead_id.clone(), ask_seq)) {
                card = card.child(
                    div()
                        .debug_selector(|| format!("refused {bead_id} {ask_seq}"))
                        .text_sm()
                        .text_color(palette.warn)
                        .child(format!("! {why}")),
                );
            }
            // Under it the button that answers, exactly as Enter does,
            // with ask-about-this, Later and Decline beside it, each key on
            // its control's chip. An asker with no conversation has the
            // reason in place of that button.
            let foreign = model::foreign_bead(&detail.bead);
            // A secret ask's one button gives, and shows no key: the keys
            // line says what Enter does. A run or setup ask's one button
            // approves, and sends no words.
            let give = match (&ask.secret, self.giving()) {
                (Some(_), Some(secret)) if sends => Some((
                    format!("give {bead_id} {ask_seq}"),
                    match approval {
                        Approval::Unlock => give_unlock_label(&secret, self.secret_typed.len()),
                        _ => give_label(&secret, self.secret_typed.len()),
                    },
                    !self.secret.read(cx).text().is_empty(),
                )),
                _ => None,
            };
            // A pick or words ask's answer, with its key on the button's
            // chip beside it.
            let answer = match ask.secret.is_some() || ask.run.is_some() || ask.setup.is_some() {
                true => None,
                false => ask
                    .options
                    .iter()
                    .find(|o| o.number == answered)
                    .map(|o| {
                        (
                            format!("send {bead_id} {ask_seq}"),
                            answer_label(o.number, &o.label),
                            true,
                        )
                    })
                    .or_else(|| {
                        words_send(ask, foreign).then(|| {
                            let words_answer = ask.options.is_empty();
                            (
                                format!("send {bead_id} {ask_seq}"),
                                if words_answer {
                                    ANSWER_WORDS.to_string()
                                } else {
                                    SEND_WORDS.to_string()
                                },
                                !self.line.read(cx).text().is_empty(),
                            )
                        })
                    }),
            };
            let unwritable = self.model.as_ref().and_then(|m| m.unwritable(&ask.asker));
            let ask_about = match &unwritable {
                None => move_button(
                    format!("ask {bead_id} {ask_seq}"),
                    "Ask about this".to_string(),
                    "a".to_string(),
                    false,
                    palette,
                    cx,
                    Some(cx.listener(|this, _, window, cx| this.ask_about_now(window, cx))),
                )
                .into_any_element(),
                Some(why) => div()
                    .id(SharedString::from(format!(
                        "ask unreachable {bead_id} {ask_seq}"
                    )))
                    .debug_selector(|| format!("ask unreachable {bead_id} {ask_seq}"))
                    .text_sm()
                    .text_color(palette.faint)
                    .child(why.clone())
                    .into_any_element(),
            };
            // While the yard is reconnecting the card's answers are held
            // as the conversation's sends are: the buttons stand
            // disabled, with the reason beside them, and the keys hold
            // the words where they are.
            let held = self.reconnecting();
            // A card with a write out, or not known to have reached the
            // yard, offers no second one: the controls stand disabled (the
            // keys do nothing), and the words under them say what the card
            // waits for.
            let waiting = self.write_sending(&bead_id, ask_seq).is_some();
            if ask.run.is_some() || ask.setup.is_some() {
                // A run or setup card's four moves in one row of buttons
                // with key chips: Approve filled, the rest outlined. One
                // fill holds, and a held or waited card's moves take no
                // second write.
                let approve_enabled = sends && self.approving.is_none() && !held && !waiting;
                let approve_click =
                    approve_enabled.then(|| cx.listener(|this, _, _, cx| this.approve(cx)));
                let ask_click = unwritable
                    .is_none()
                    .then(|| cx.listener(|this, _, window, cx| this.ask_about_now(window, cx)));
                let later_click =
                    (!waiting).then(|| cx.listener(|this, _, _, cx| this.later_now(cx)));
                let decline_click = (!waiting)
                    .then(|| cx.listener(|this, _, window, cx| this.decline_now(window, cx)));
                let moves = div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(2.))
                    .child(move_button(
                        format!("approve {bead_id} {ask_seq}"),
                        approve_move_label(&self.yard, matches!(approval, Approval::Unlock)),
                        APPROVE_KEY.to_string(),
                        true,
                        palette,
                        cx,
                        approve_click,
                    ))
                    .child(move_button(
                        format!("ask {bead_id} {ask_seq}"),
                        "Ask about this".to_string(),
                        "a".to_string(),
                        false,
                        palette,
                        cx,
                        ask_click,
                    ))
                    .child(move_button(
                        format!("later {bead_id} {ask_seq}"),
                        "Later".to_string(),
                        "x".to_string(),
                        false,
                        palette,
                        cx,
                        later_click,
                    ))
                    .child(move_button(
                        format!("decline {bead_id} {ask_seq}"),
                        "Decline".to_string(),
                        "d".to_string(),
                        false,
                        palette,
                        cx,
                        decline_click,
                    ));
                card = card.child(moves);
                if let Some(why) = &unwritable {
                    card = card.child(
                        div()
                            .debug_selector(|| format!("ask unreachable {bead_id} {ask_seq}"))
                            .text_sm()
                            .text_color(palette.faint)
                            .child(why.clone()),
                    );
                }
                // Under the moves, as under the buttons: where the
                // write stands with the yard, until it says.
                card = card
                    .when(held, |el| el.child(send_held(palette)))
                    .when_some(
                        self.sending_words(&bead_id, ask_seq, palette),
                        |el, words| el.child(words),
                    );
            } else {
                // A pick, words or secret card's moves in one row with the
                // run card's: Answer filled with its key on a chip, the
                // rest outlined. Give shows no key, as before.
                card = card.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_2()
                        .when_some(answer, |el, (id, text, enabled)| {
                            let click = (enabled && !held && !waiting)
                                .then(|| cx.listener(|this, _, _, cx| this.decide(cx)));
                            el.child(move_button(
                                id,
                                text,
                                "↵".to_string(),
                                true,
                                palette,
                                cx,
                                click,
                            ))
                        })
                        .when_some(give, |el, (id, text, enabled)| {
                            el.child(send_button(
                                id,
                                text,
                                enabled && !held && !waiting,
                                palette,
                                cx,
                            ))
                        })
                        .when(held, |el| el.child(send_held(palette)))
                        // Under the control, as the held words are: where the
                        // write stands with the yard, until it says.
                        .when_some(
                            self.sending_words(&bead_id, ask_seq, palette),
                            |el, words| el.child(words),
                        )
                        .child(ask_about)
                        .child(
                            move_button(
                                format!("later {bead_id} {ask_seq}"),
                                "Later".to_string(),
                                "x".to_string(),
                                false,
                                palette,
                                cx,
                                Some(cx.listener(|this, _, _, cx| this.later_now(cx))),
                            )
                            .when(waiting, |el| el.opacity(0.5)),
                        )
                        .child(
                            move_button(
                                format!("decline {bead_id} {ask_seq}"),
                                "Decline".to_string(),
                                "d".to_string(),
                                false,
                                palette,
                                cx,
                                Some(
                                    cx.listener(|this, _, window, cx| this.decline_now(window, cx)),
                                ),
                            )
                            .when(waiting, |el| el.opacity(0.5))
                            // Declining is not a pick, and a run ask has no
                            // pick to take it for: only another ask's button
                            // says so.
                            .when(ask.run.is_none(), |el| {
                                el.tooltip(move |_, cx| {
                                    cx.new(|_| RailTip(DECLINE_IS_NO_PICK.to_string())).into()
                                })
                            }),
                        ),
                );
            }
            // The keys of a secret, run or setup ask's card: an
            // ordinary card carries each key on its control, and lists
            // none. Every card carries the cheatsheet's hint.
            let keys = match (&ask.secret, &ask.run, &ask.setup) {
                (Some(_), _, _) => secret_keys(&bead_id, ask_seq, approval, unwritable.is_none()),
                (None, Some(_), _) => run_keys(approval),
                (None, None, Some(setup)) => setup_keys(&bead_id, ask_seq, setup, approval),
                (None, None, None) => String::new(),
            };
            card = card.child(keys_row(
                keys,
                "keys text card",
                "keys hint card",
                palette,
                cx,
            ));
        } else {
            let bead_click = bead_id.clone();
            card = card.child(
                div()
                    .id(SharedString::from(format!("current {bead_id} {ask_seq}")))
                    .text_sm()
                    .text_color(palette.text_accent)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_ask(bead_click.clone(), ask_seq, cx)
                    }))
                    .child("decide this one"),
            );
        }
        card
    }

    /// A run ask's question lines, each with the asker's label beside its
    /// words. When a line wraps, its continuation stays under the words,
    /// where no line of the yard's card begins.
    pub(crate) fn marked_question(
        &self,
        card_key: u64,
        id: String,
        question: &str,
        asker: &str,
        palette: Palette,
    ) -> impl IntoElement + use<> {
        div()
            .flex()
            .flex_col()
            .children(question.lines().enumerate().map(|(line, text)| {
                self.marked_asker_line(
                    card_key,
                    "question",
                    line,
                    format!("{id} {line}"),
                    text,
                    asker,
                    palette,
                )
            }))
    }

    /// One run asker's line with the trusted label beside its words where
    /// they fit, and above them where they do not. The words select and
    /// copy, drawn as they were written with no block in them; the label
    /// stays plain, so a copy of the words does not carry it. The label is
    /// quiet, in the faint colour at the regular weight, so that it marks
    /// every line without reading as a second question: it stands on each
    /// line all the same.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn marked_asker_line(
        &self,
        card_key: u64,
        section: &str,
        index: usize,
        id: String,
        line: &str,
        asker: &str,
        palette: Palette,
    ) -> impl IntoElement + use<> {
        let prefix = format!("{asker} says: ");
        let words = line.strip_prefix(&prefix).unwrap_or(line).to_string();
        let label = prefix;
        let label_id = format!("asker label {id}");
        let words_id = format!("asker words {id}");
        let key = expanded_text_key(card_key, section, index, &words);
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap_1()
            .child(
                div()
                    .flex_none()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(palette.faint)
                    .debug_selector(move || label_id.clone())
                    .child(label),
            )
            .child(
                div()
                    .flex_1()
                    // A label and a too-narrow word column split the text
                    // into letters. Wrap the words below the label while
                    // there is still room for a word at their side.
                    .min_w(px(80.))
                    .debug_selector(move || words_id.clone())
                    .child(select::verbatim(key, words, &self.selection)),
            )
    }

    /// What a secret ask asks for: the yard's line, which is made from the
    /// source's declaration and never from the asker's words, what the
    /// value reaches where the yard says (its lines stand directly under
    /// the line, so that the specifics are read before the generic go-line
    /// that ends them), and where the value goes, in the words of the
    /// ask's kind. Where the yard sends no reach, nothing stands between
    /// the two: a harness key and a waiting login's input never have any.
    pub(crate) fn secret_lines(
        &self,
        bead: &str,
        ask: u64,
        secret: &api::Secret,
        palette: Palette,
    ) -> impl IntoElement + use<> {
        let card_key = select::key((bead, ask));
        let key = expanded_text_key(card_key, "secret", 0, &secret.line);
        let mut lines = div().flex().flex_col().child(
            div()
                .debug_selector(|| secret.line.clone())
                .font_weight(FontWeight::SEMIBOLD)
                .child(select::text(key, secret.line.clone(), &self.selection)),
        );
        if !secret.reach.is_empty() {
            // A gap between the lines tells a new one from a wrapped row.
            let mut reach = div().flex().flex_col().gap_1().my_1();
            for (ix, line) in secret.reach.iter().enumerate() {
                let key = expanded_text_key(card_key, "secret reach", ix, line);
                reach = reach.child(
                    div()
                        .debug_selector(|| line.clone())
                        .text_xs()
                        .child(select::text(key, line.clone(), &self.selection)),
                );
            }
            lines = lines.child(reach);
        }
        let goes = secret_goes(secret);
        lines.child(
            div()
                .debug_selector(|| goes.to_string())
                .text_xs()
                .text_color(palette.faint)
                .child(goes),
        )
    }

    /// The head of a teammate invite or removal ask's block: which it is,
    /// and its fields parsed from the command. The command whole, the
    /// yard's sentences and the take-back stand below as on any run card:
    /// an approval runs exactly what is shown.
    pub(crate) fn teammate_head(
        &self,
        bead: &str,
        ask: u64,
        teammate: &Teammate,
        palette: Palette,
    ) -> impl IntoElement + use<> {
        let card_key = select::key((bead, ask));
        let kind = teammate_kind(teammate).to_string();
        let mut head = div().flex().flex_col().gap_2().child(
            div()
                .debug_selector(|| format!("teammate kind {bead} {ask}"))
                .font_weight(FontWeight::SEMIBOLD)
                .child(select::text(
                    expanded_text_key(card_key, "teammate kind", 0, &kind),
                    kind,
                    &self.selection,
                )),
        );
        let mut fields: Vec<(&str, String)> = Vec::new();
        match teammate {
            Teammate::Invite { name, email, role } => {
                fields.push(("email", teammate_email(email)));
                fields.push(("user", teammate_user(name)));
                fields.push(("role", teammate_role(role)));
            }
            Teammate::Uninvite { name } => {
                fields.push(("user", teammate_user(name)));
            }
        }
        for (field, line) in &fields {
            let section = format!("teammate {field}");
            head = head.child(
                div()
                    .debug_selector(|| format!("teammate {field} {bead} {ask}"))
                    .child(select::text(
                        expanded_text_key(card_key, &section, 0, line),
                        line.clone(),
                        &self.selection,
                    )),
            );
        }
        if matches!(teammate, Teammate::Invite { .. }) {
            head = head.child(
                div()
                    .debug_selector(|| format!("teammate correction {bead} {ask}"))
                    .text_xs()
                    .text_color(palette.faint)
                    .child(select::text(
                        expanded_text_key(card_key, "teammate correction", 0, TEAMMATE_CORRECTION),
                        TEAMMATE_CORRECTION.to_string(),
                        &self.selection,
                    )),
            );
        }
        head
    }

    /// What a run ask proposes, in the one block of its card, which is all
    /// the yard's and none of it the asker's: the command whole, in a code
    /// block that wraps where the window is narrower and is never cut; the
    /// command's one line from its help; the yard's sentences for what it
    /// changes, each on its own line; the texts the ask holds behind one
    /// tap, for checking the yard's reading against them; and the command
    /// that takes it back, with who may run it. `open` is an ask that can
    /// still be approved. The approval is live whether or not the tap was
    /// opened. A teammate invite or removal carries its own head above all
    /// of it, with its fields parsed from the command.
    pub(crate) fn run_block(
        &self,
        bead: &str,
        ask: u64,
        run: &api::Run,
        open: bool,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let card_key = select::key((bead, ask));
        let command = model::plain(&run.line());
        // The user the approval is called as, and so the one it runs as.
        let user = self.model.as_ref().map(|m| m.me()).unwrap_or_default();
        let over = match open {
            true => run_runs(&self.yard, user),
            false => RUN_PROPOSED.to_string(),
        };
        let mut block = div()
            .debug_selector(|| format!("run {bead} {ask}"))
            .px_3()
            .py_2p5()
            .rounded_md()
            .border_1()
            .border_color(palette.line)
            .bg(palette.side)
            .flex()
            .flex_col()
            .gap_2();
        if let Some(teammate) = teammate_of(run) {
            block = block.child(self.teammate_head(bead, ask, &teammate, palette));
        }
        // The command whole, in mono at the code size, wrapping where
        // the window is narrower and never cut, with Copy as a link word
        // beside it that returns the plain command. Drawn verbatim, not
        // fenced: a fence would draw its own copy chip, and the word is
        // the copy control here.
        let mut command_line = div()
            .flex_1()
            .min_w_0()
            .debug_selector(|| format!("run command {bead} {ask}"))
            .text_size(rems(TYPE_CODE))
            .child(select::verbatim(
                expanded_text_key(card_key, "run", 0, &command),
                command.clone(),
                &self.selection,
            ));
        if let Some(mono) = select::mono(cx) {
            command_line = command_line.font_family(mono);
        }
        block = block
            .child(
                div()
                    .debug_selector(|| over.clone())
                    .text_xs()
                    .text_color(palette.faint)
                    .child(over.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_baseline()
                    .gap_3()
                    .child(command_line)
                    .child(
                        div()
                            .id(SharedString::from(format!("run copy {bead} {ask}")))
                            .debug_selector(|| format!("run copy {bead} {ask}"))
                            .flex_none()
                            .text_size(rems(TYPE_META))
                            .text_color(palette.link)
                            .cursor_pointer()
                            .on_click({
                                let command = command.clone();
                                move |_, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        command.clone(),
                                    ))
                                }
                            })
                            .child("Copy"),
                    ),
            );
        let description = model::plain(&run.description);
        if !description.is_empty() {
            let line = description_line(&description);
            let key = expanded_text_key(card_key, "description", 0, &line);
            block = block.child(
                div()
                    .debug_selector(|| format!("run description {bead} {ask}"))
                    .text_xs()
                    .child(select::text(key, line, &self.selection)),
            );
        }
        for (ix, does) in run.does.iter().enumerate() {
            let line = format!("the yard says: {}", model::plain(does));
            let key = expanded_text_key(card_key, "does", ix, &line);
            block = block.child(
                div()
                    .debug_selector(|| format!("run does {bead} {ask} {ix}"))
                    .text_xs()
                    .child(select::text(key, line, &self.selection)),
            );
        }
        block = block.child(self.block_end(
            "run",
            (bead, ask),
            &run_texts(run),
            (&run.back, &run.back_by),
            palette,
            cx,
        ));
        block
    }

    /// The end of a run or setup ask's block: the ask's own texts behind
    /// one tap, for checking the yard's reading against them, and the
    /// commands that take it back, with who may run them. `kind` is whose
    /// block it is, for the texts' selectors. The approval is live whether
    /// or not the tap was opened.
    pub(crate) fn block_end(
        &self,
        kind: &str,
        at: (&str, u64),
        texts: &[RunText],
        back: (&str, &str),
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let (bead, ask) = at;
        let (back, back_by) = back;
        let card_key = select::key((bead, ask));
        let mut end = div().flex().flex_col().gap_2();
        // The ask's own texts, behind one tap: the approval is live
        // whether or not it was opened.
        let texts_shown = self
            .model
            .as_ref()
            .is_some_and(|m| m.texts_shown(bead, ask));
        if !texts.is_empty() {
            let bead_click = bead.to_string();
            end = end.child(
                div()
                    .id(SharedString::from(format!("texts {bead} {ask}")))
                    .debug_selector(|| format!("texts {bead} {ask}"))
                    .text_xs()
                    .text_color(palette.faint)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.texts_for(bead_click.clone(), ask, cx)
                    }))
                    .child(texts_label(texts.len(), texts_shown)),
            );
        }
        if texts_shown {
            for (ix, text) in texts.iter().enumerate() {
                end = end
                    .child(
                        div()
                            .text_xs()
                            .text_color(palette.faint)
                            .child(text.heading.clone()),
                    )
                    .child(
                        div()
                            .debug_selector(|| format!("{kind} text {bead} {ask} {ix}"))
                            .child(select::text_with_code_copy(
                                expanded_text_key(card_key, text.section, ix, &text.text),
                                fenced(&model::plain(&text.text)),
                                text.text.clone(),
                                &self.selection,
                            )),
                    );
            }
        }
        let back = model::plain(back);
        if !back.is_empty() {
            end = end
                .child(
                    div()
                        .text_xs()
                        .text_color(palette.faint)
                        .child(take_back_label(&model::plain(back_by))),
                )
                .child(
                    div()
                        .debug_selector(|| format!("{kind} back {bead} {ask}"))
                        .child(select::text(
                            expanded_text_key(card_key, "back", 0, &back),
                            fenced(&back),
                            &self.selection,
                        )),
                );
        }
        end
    }

    /// What a setup ask proposes, in the one block of its card, which is
    /// all the yard's and none of it the asker's: what approving does, in
    /// one line; the yard's sentences for what it changes, each on its own
    /// line; the texts the ask holds behind one tap, for checking the
    /// yard's reading against them; and the commands that take it back,
    /// with who may run them. `open` is an ask that can still be approved.
    /// The approval is live whether or not the tap was opened.
    pub(crate) fn setup_block(
        &self,
        bead: &str,
        ask: u64,
        setup: &api::Setup,
        open: bool,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let card_key = select::key((bead, ask));
        let over = match open {
            true => setup_runs(&self.yard),
            false => SETUP_PROPOSED.to_string(),
        };
        let summary = model::setup_summary(setup);
        let summary_key = expanded_text_key(card_key, "summary", 0, &summary);
        let mut block = div()
            .debug_selector(|| format!("setup {bead} {ask}"))
            .px_3()
            .py_2p5()
            .rounded_md()
            .border_1()
            .border_color(palette.line)
            .bg(palette.side)
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .debug_selector(|| over.clone())
                    .text_xs()
                    .text_color(palette.faint)
                    .child(over.clone()),
            )
            .child(
                div()
                    .debug_selector(|| format!("setup summary {bead} {ask}"))
                    .text_xs()
                    .child(select::text(summary_key, summary, &self.selection)),
            );
        for (ix, does) in setup.does.iter().enumerate() {
            let line = format!("the yard says: {}", model::plain(does));
            let key = expanded_text_key(card_key, "does", ix, &line);
            block = block.child(
                div()
                    .debug_selector(|| format!("setup does {bead} {ask} {ix}"))
                    .text_xs()
                    .child(select::text(key, line, &self.selection)),
            );
        }
        block = block.child(self.block_end(
            "setup",
            (bead, ask),
            &carried_texts(&setup.carried),
            (&setup.back, &setup.back_by),
            palette,
            cx,
        ));
        block
    }

    /// The setup ask's account choice: each candidate the yard lists with
    /// the account its file holds, to take with a tap. `taken` is the one
    /// taken, by source name, empty where none is.
    pub(crate) fn setup_candidates(
        &self,
        bead: &str,
        ask: u64,
        setup: &api::Setup,
        taken: &str,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let mut choice = div().flex().flex_col().gap_1();
        if setup.candidates.is_empty() {
            return choice.into_any_element();
        }
        choice = choice.child(div().text_xs().text_color(palette.faint).child(SETUP_TAKE));
        for candidate in &setup.candidates {
            let is_taken = candidate.source == taken;
            let text = candidate_choice_line(candidate, taken);
            let source = candidate.source.clone();
            let bead_click = bead.to_string();
            choice = choice.child(
                div()
                    .id(SharedString::from(format!(
                        "account {bead} {ask} {}",
                        candidate.source
                    )))
                    .debug_selector(|| format!("account {bead} {ask} {}", candidate.source))
                    .cursor_pointer()
                    .rounded_md()
                    .border_1()
                    .border_color(rgba(0x00000000))
                    .px_2()
                    .py_1()
                    // A taken account is outlined, never filled: the
                    // card's one fill is its Approve button.
                    .when(is_taken, |el| el.border_color(palette.accent))
                    .when(!is_taken, |el| {
                        el.hover(|style| style.border_color(palette.line))
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.take_account_for(bead_click.clone(), ask, source.clone(), cx)
                    }))
                    .child(text),
            );
        }
        choice.into_any_element()
    }

    /// Where a run or setup ask is approved when this yard takes no
    /// approval from this window: a terminal on the yard's machine, with
    /// the command in a block that copies it.
    pub(crate) fn approve_elsewhere_lines(
        &self,
        bead: &str,
        ask: u64,
        command: String,
        palette: Palette,
    ) -> impl IntoElement + use<> {
        div()
            .debug_selector(|| format!("approve elsewhere {bead} {ask}"))
            .flex()
            .flex_col()
            .gap_1()
            .child(div().text_color(palette.warn).child(APPROVE_ELSEWHERE))
            .child(select::text(
                expanded_text_key(select::key((bead, ask)), "approve", 0, &command),
                fenced(&command),
                &self.selection,
            ))
    }

    /// A weighty card's number where this window has no gate: what to
    /// type and why, with the field that takes it, in the terminal's
    /// words. The approval or the give sends `weighed` only where the
    /// field holds exactly the ask's number. The number is no secret: it
    /// is the ask's own, drawn as typed. The approval's key approves from
    /// the field too, as from a setup ask's account field.
    pub(crate) fn typeback_lines(
        &self,
        bead: &str,
        ask: u64,
        verb: &str,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let input = self.typeback.clone();
        let focus = input.focus_handle(cx);
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(div().child(typeback_line(ask, verb)))
            .child(
                field(focus, input, palette)
                    .debug_selector(|| format!("typeback {bead} {ask}"))
                    .key_context(SETUP_SECRET_CONTEXT),
            )
    }

    /// One dealt with: what was decided, in plain words with who and when;
    /// the picked option marked, with its words under it; the same options
    /// to pick again from there; and the same full bead one key away. One
    /// its asker took back reads only: the yard refuses an answer to it, so
    /// its card has no options to pick and no field.
    pub(crate) fn talk_decided(
        &self,
        model: &Model,
        bead: &str,
        ask: u64,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let pane = div().flex_1().min_w_0().h_full().flex().flex_col();
        let Some(detail) = model.decided_detail(bead, ask, &chrono::Local) else {
            return pane
                .items_center()
                .justify_center()
                .text_color(palette.faint)
                .child("It is gone from the decided.");
        };
        let head = div()
            .p_3()
            .border_b_1()
            .border_color(palette.line)
            .flex()
            .flex_col()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(detail.title.clone()),
            )
            .child(
                div()
                    .debug_selector(|| format!("in {} {bead} {ask}", self.yard))
                    .text_sm()
                    .text_color(palette.faint)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .debug_selector(|| format!("in {} {bead} {ask} dot", self.yard))
                            .flex_none()
                            .size(px(8.))
                            .rounded_full()
                            .bg(yard_color(&self.yard, palette.dark)),
                    )
                    .child(card_head(&self.yard, &detail.bead, &format!("ask {ask}"))),
            );
        // A pairing approval's sitting stands where its decided card
        // does: the QR screen, or its ended state.
        if let Some(sitting) = self
            .approval_pairing
            .as_ref()
            .filter(|sitting| sitting.card() == (bead.to_string(), ask))
        {
            let body = self.approval_pairing_body(bead, ask, sitting, palette, cx);
            return pane
                .children(self.reconnect_strip(palette))
                .child(head)
                .child(
                    div()
                        .id("decided")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .track_scroll(&self.decision)
                        .p_4()
                        .child(body),
                )
                .when(!model.notice.is_empty(), |el| {
                    el.child(
                        div()
                            .px_4()
                            .py_1()
                            .text_sm()
                            .text_color(palette.warn)
                            .child(format!("! {}", model.notice)),
                    )
                });
        }
        // The decided card's frame: raised and bordered, as an open
        // card's. Everything the card says stands inside it.
        let mut body = div()
            .debug_selector(|| format!("decided card {bead} {ask}"))
            .flex()
            .flex_col()
            .gap_2p5()
            .p_4()
            .rounded(px(RADIUS_CARD))
            .border_1()
            .border_color(palette.line)
            .bg(palette.raised);
        // The question, and what was decided with who and when.
        let card_key = select::key((bead, ask));
        let asker_marked = detail.run.is_some() || detail.setup.is_some();
        let question = if asker_marked {
            self.marked_question(
                card_key,
                format!("run decided question {bead} {ask}"),
                &detail.question,
                &detail.asker,
                palette,
            )
            .into_any_element()
        } else {
            select::text(
                expanded_text_key(card_key, "question", 0, &detail.question),
                detail.question.clone(),
                &self.selection,
            )
            .into_any_element()
        };
        // The yard's weight first, each class's line as the terminal
        // prints it, in rust: a weighty card reads apart from a routine
        // one, and never in red.
        for (n, weight) in detail.weight.iter().enumerate() {
            let line = model::weight_line(&weight.class, &weight.line);
            body = body.child(
                div()
                    .debug_selector(|| format!("weight {bead} {ask} {n}"))
                    .text_color(palette.text_accent)
                    .child(select::text(
                        expanded_text_key(card_key, "weight", n, &line),
                        line,
                        &self.selection,
                    )),
            );
        }
        // The door a pairing card's approval crosses, beside the yard's
        // weight: the approver reads it before the approval goes. A card
        // that approves no more has no door to say.
        let pairs = detail.run.as_ref().is_some_and(is_device_pair_run)
            && model.approving(bead, ask).is_some()
            && !self.approve_elsewhere.contains(&(bead.to_string(), ask));
        if pairs && let Some(client) = self.client.as_ref() {
            let line = pair_door(client.sealed(), client.is_remote());
            body = body.child(
                div()
                    .debug_selector(|| format!("door {bead} {ask}"))
                    .text_color(palette.text_accent)
                    .child(select::text(
                        expanded_text_key(card_key, "door", 0, line),
                        line.to_string(),
                        &self.selection,
                    )),
            );
        }
        body = body
            .child(
                div()
                    .debug_selector(|| format!("question {bead} {ask}"))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(question),
            )
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(detail.state_line.clone()),
            );
        // Under an approved run ask's command, what it printed. One
        // the yard has not carried out yet printed nothing: its state
        // line says it is being carried out, and the pending words are
        // not what it printed.
        let printed = detail
            .ran
            .as_ref()
            .and_then(|ran| match ran.result.as_str() {
                model::CARRY_PENDING => None,
                _ => Some(model::plain(&ran.result)),
            });
        if let Some(printed) = printed.filter(|printed| !printed.is_empty()) {
            body = body.child(
                div()
                    .debug_selector(|| format!("ran {bead} {ask}"))
                    .child(printed),
            );
        }
        if !model.has_bead(bead) {
            body = body.child(div().text_color(palette.faint).child("Reading the bead."));
        } else {
            // The context, the options to pick again, the recommendation.
            for (line, text) in detail.context.iter().enumerate() {
                if asker_marked {
                    body = body.child(div().text_size(rems(TYPE_BODY)).child(
                        self.marked_asker_line(
                            card_key,
                            "context",
                            line,
                            format!("run decided context {bead} {ask} {line}"),
                            text,
                            &detail.asker,
                            palette,
                        ),
                    ));
                } else {
                    body = body.child(
                        div()
                            .debug_selector(|| format!("context {bead} {ask} {line}"))
                            .text_size(rems(TYPE_BODY))
                            .child(select::text(
                                expanded_text_key(card_key, "context", line, text),
                                text.clone(),
                                &self.selection,
                            )),
                    );
                }
            }
            // Whether a put-aside run or setup ask can still be approved
            // from here: one approved, declined or taken back takes no
            // approval.
            let approving = model.approving(bead, ask).is_some();
            let setup_approving = model.setup_approving(bead, ask).is_some();
            let elsewhere = self.approve_elsewhere.contains(&(bead.to_string(), ask));
            let approval = approval(
                !detail.weight.is_empty(),
                self.unlock_available(),
                elsewhere,
            );
            let sends = !matches!(approval, Approval::Nowhere);
            let giving = model.giving(bead, ask);
            // An ask with no options shows none: the answer's words are in
            // the state line, and nothing is wanted. Nor does one taken
            // back: there is nothing to pick. Nor a setup ask past its
            // approval: its presets are picked with the account, which a
            // put-aside one still takes below.
            let taken_back = detail.state == "taken back";
            if !detail.options.is_empty()
                && !taken_back
                && (detail.setup.is_none() || setup_approving)
            {
                for option in &detail.options {
                    // The circle marks the pick; a newly chosen option is
                    // the highlighted one, and the button says what it sends.
                    let picked = option.number == detail.picked;
                    let marker = if picked { "●" } else { "○" };
                    let pick = format!("{} {}. {}", marker, option.number, option.label);
                    let consequence = format!("— {}", option.consequence);
                    let n = option.number;
                    body = body.child(
                        div()
                            .id(SharedString::from(format!("pick {bead} {ask} {n}")))
                            .debug_selector(|| format!("pick {bead} {ask} {n}"))
                            .cursor_pointer()
                            .rounded_md()
                            .border_1()
                            .border_color(rgba(0x00000000))
                            .px_2()
                            .py_1()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_baseline()
                            // The chosen lever is outlined, never
                            // filled: a card's one fill is its button.
                            .when(n == detail.chosen, |el| el.border_color(palette.accent))
                            .when(n != detail.chosen, |el| {
                                el.hover(|style| style.border_color(palette.line))
                            })
                            .on_click(cx.listener(move |this, _, _, cx| this.pick(n, cx)))
                            .child(
                                div()
                                    .when(picked, |el| {
                                        el.font_weight(FontWeight::SEMIBOLD)
                                            .text_color(palette.text_accent)
                                    })
                                    .when(!picked, |el| el.font_weight(FontWeight::MEDIUM))
                                    .child(format!("{pick} ")),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(match picked {
                                        true => palette.text_accent,
                                        false => palette.faint,
                                    })
                                    .child(consequence),
                            ),
                    );
                    // The words sent with the pick stand under it.
                    if picked && !detail.words.is_empty() {
                        body = body.child(div().child(detail.words.clone()));
                    }
                }
                if detail.recommend != 0 {
                    body = body.child(
                        div()
                            .ml_6()
                            .text_xs()
                            .text_color(palette.faint)
                            .child(format!("recommended: {}, {}", detail.recommend, detail.why)),
                    );
                }
            }
            // A mapping pick writes, so a weighty one with no gate takes
            // its number back before the later pick, as on an open card.
            if detail.identity.is_some() && !taken_back && matches!(approval, Approval::TypeBack) {
                body = body.child(self.typeback_lines(bead, ask, "pick", palette, cx));
            }
            // A secret ask says what it asks for in the yard's line. A
            // weighty one with no gate takes its number back before its
            // value field below.
            if let Some(secret) = &detail.secret {
                body = body.child(self.secret_lines(bead, ask, secret, palette));
                if giving.is_some() && matches!(approval, Approval::TypeBack) {
                    body = body.child(self.typeback_lines(bead, ask, "give", palette, cx));
                } else if giving.is_some() && !sends {
                    body = body.child(self.approve_elsewhere_lines(
                        bead,
                        ask,
                        format!("yardr bead answer {bead} {ask} --secret"),
                        palette,
                    ));
                }
            }
            // A run ask has the yard's block, and under it the control
            // that approves while it is only put aside: one approved,
            // declined or taken back takes no approval. A weighty one
            // with no gate takes its number back above the control.
            if let Some(run) = &detail.run {
                body = body.child(self.run_block(bead, ask, run, approving, palette, cx));
                if approving && elsewhere {
                    body = body.child(self.approve_elsewhere_lines(
                        bead,
                        ask,
                        model::approve_command(bead, ask),
                        palette,
                    ));
                } else if approving && sends {
                    if matches!(approval, Approval::TypeBack) {
                        body = body.child(self.typeback_lines(bead, ask, "approve", palette, cx));
                    }
                    body = body.child(div().flex().child(approve_button(
                        format!("approve {bead} {ask}"),
                        &self.yard,
                        matches!(approval, Approval::Unlock),
                        self.approving.is_none()
                            && !self.reconnecting()
                            && self.write_sending(bead, ask).is_none(),
                        palette,
                        cx,
                    )));
                }
            }
            // A setup ask has the yard's block with its account choice,
            // and under it the control that approves while it is only put
            // aside. Its presets stand to pick above, and a weighty one
            // with no gate takes its number back above the control.
            if let Some(setup) = &detail.setup {
                body = body.child(self.setup_block(bead, ask, setup, setup_approving, palette, cx));
                if setup_approving {
                    body = body.child(self.setup_candidates(
                        bead,
                        ask,
                        setup,
                        &detail.account,
                        palette,
                        cx,
                    ));
                    if elsewhere {
                        body = body.child(self.approve_elsewhere_lines(
                            bead,
                            ask,
                            model::setup_approve_command(bead, ask, setup),
                            palette,
                        ));
                    } else if sends {
                        if matches!(approval, Approval::TypeBack) {
                            body =
                                body.child(self.typeback_lines(bead, ask, "approve", palette, cx));
                        }
                        body = body.child(div().flex().child(approve_button(
                            format!("approve {bead} {ask}"),
                            &self.yard,
                            matches!(approval, Approval::Unlock),
                            self.approving.is_none()
                                && !self.reconnecting()
                                && self.write_sending(bead, ask).is_none(),
                            palette,
                            cx,
                        )));
                    }
                }
            }
            // Where a write of the card among the decided stands with the
            // yard (an approval put aside and approved from here, a pick
            // changed, a value given again), under its control.
            body = body.when_some(self.sending_words(bead, ask, palette), |body, words| {
                body.child(words)
            });
            // A line for words with the later pick, the field of a secret
            // to give it again, or the field of a put-aside setup for its
            // account: a declined one takes none, one taken back has
            // neither, and no line for words stands where a value could be
            // typed in sight. A card that sends nowhere draws no field.
            let setup_field = setup_approving && detail.setup.is_some() && sends;
            let secret_field = (detail.secret.is_some() && sends) || setup_field;
            let input = match secret_field {
                true => self.secret.clone(),
                false => self.line.clone(),
            };
            let focus = input.focus_handle(cx);
            let kind = if secret_field { "secret" } else { "words" };
            let words = detail.secret.is_none() && detail.run.is_none() && detail.setup.is_none();
            let fielded = !taken_back && sends && (words || giving.is_some() || setup_field);
            // A setup the card can still approve takes the host above
            // the account where its ask carries a template, as on an
            // open card.
            let hosted = setup_field
                && detail
                    .setup
                    .as_ref()
                    .is_some_and(|setup| !setup.host_template.is_empty());
            let host_focus = self.host.focus_handle(cx);
            let host = self.host.clone();
            // A secret's field stands beside its toggle, as on an open
            // card.
            let toggle = secret_field
                .then(|| show_button(format!("show {bead} {ask}"), self.secret_shown, palette, cx));
            body = body.when(hosted, |body| {
                body.child(
                    field(host_focus, host, palette)
                        .debug_selector(|| format!("host {bead} {ask}"))
                        // A setup's host field takes its approval's key
                        // too, as on an open card.
                        .key_context(SETUP_SECRET_CONTEXT),
                )
            });
            body = body.when(fielded, |body| {
                let boxed = field(focus, input, palette)
                    .debug_selector(|| format!("{kind} {bead} {ask}"))
                    // A setup ask's account field takes its approval's key
                    // too, with what the field holds.
                    .when(setup_field, |el| el.key_context(SETUP_SECRET_CONTEXT));
                match toggle {
                    Some(toggle) => body.child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(boxed.flex_1())
                            .child(toggle),
                    ),
                    None => body.child(boxed),
                }
            });
            // Under it the button that sends the field, exactly as Enter
            // does: the value again on a secret, the words where they are
            // the answer, the new pick where one is chosen.
            // Why the yard did not take the card's answer, in place, as on
            // an open card: an approval refused from here is shown whole.
            if let Some(why) = self.refused.get(&(bead.to_string(), ask)) {
                body = body.child(
                    div()
                        .debug_selector(|| format!("refused {bead} {ask}"))
                        .text_sm()
                        .text_color(palette.warn)
                        .child(format!("! {why}")),
                );
            }
            let send = match (&detail.secret, giving) {
                _ if taken_back => None,
                (Some(_), Some(secret)) if sends => Some((
                    format!("give {bead} {ask}"),
                    match approval {
                        Approval::Unlock => give_unlock_label(secret, self.secret_typed.len()),
                        _ => give_label(secret, self.secret_typed.len()),
                    },
                    !self.secret.read(cx).text().is_empty(),
                )),
                (Some(_), _) => None,
                (None, _) if detail.run.is_some() || detail.setup.is_some() => None,
                (None, _) => detail
                    .options
                    .iter()
                    .find(|o| o.number == detail.chosen)
                    .map(|o| {
                        (
                            format!("send {bead} {ask}"),
                            format!("Change to: {} · {}", o.number, o.label),
                            true,
                        )
                    })
                    .or_else(|| {
                        detail.options.is_empty().then(|| {
                            (
                                format!("send {bead} {ask}"),
                                "Send".to_string(),
                                !self.line.read(cx).text().is_empty(),
                            )
                        })
                    }),
            };
            // As an open card's: held while the yard is reconnecting,
            // with the reason beside the button that would send.
            let held = self.reconnecting();
            // As an open card's: no second write while one is with the
            // yard.
            let waiting = self.write_sending(bead, ask).is_some();
            let sendable = send.is_some()
                || (detail.run.is_some() && approving && sends)
                || (detail.setup.is_some() && setup_approving && sends);
            body = body.when_some(send, |body, (id, text, enabled)| {
                body.child(send_button(
                    id,
                    text,
                    enabled && !held && !waiting,
                    palette,
                    cx,
                ))
            });
            body = body.when(held && sendable, |body| body.child(send_held(palette)));
            let run_approval = match approving {
                true => approval,
                false => Approval::Nowhere,
            };
            let setup_approval = match setup_approving {
                true => approval,
                false => Approval::Nowhere,
            };
            let keys = match detail.secret {
                Some(_) if giving.is_some() => secret_keys(bead, ask, approval, false),
                Some(_) => String::new(),
                None if detail.run.is_some() => run_keys(run_approval),
                None => match &detail.setup {
                    Some(setup) => setup_keys(bead, ask, setup, setup_approval),
                    None => decided_keys(&detail),
                },
            };
            body = body.child(keys_row(
                keys,
                "keys text decided",
                "keys hint decided",
                palette,
                cx,
            ));
            let expand_label = if detail.expanded { "fold" } else { "expand" };
            let bead_click = bead.to_string();
            body = body.child(
                div()
                    .id(SharedString::from(format!("expand {bead} {ask}")))
                    .text_sm()
                    .text_color(palette.faint)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.expand_for(bead_click.clone(), ask, cx)
                    }))
                    .child(format!("{expand_label} (e)")),
            );
            if detail.expanded {
                body = body.child(self.expanded_body(
                    select::key((bead, ask)),
                    Expanded {
                        body: &detail.body,
                        notes: &detail.notes,
                        about: &detail.about,
                        doc: &detail.doc,
                    },
                    palette,
                    cx,
                ));
            }
        }
        pane.children(self.reconnect_strip(palette))
            .child(head)
            .child(
                div()
                    .id("decided")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.decision)
                    .p_4()
                    .child(body),
            )
            .when(!model.notice.is_empty(), |el| {
                el.child(
                    div()
                        .px_4()
                        .py_1()
                        .text_sm()
                        .text_color(palette.warn)
                        .child(format!("! {}", model.notice)),
                )
            })
    }

    /// A pairing approval's sitting on its decided card: the QR screen
    /// while the transient shows, its ended state after. A sitting whose
    /// code ran out draws the ended state, whatever the timer has dropped
    /// yet.
    fn approval_pairing_body(
        &self,
        bead: &str,
        ask: u64,
        sitting: &ApprovalPairing,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        match sitting {
            ApprovalPairing::Shown {
                pairing,
                ending,
                end_refused,
                ..
            } if sitting.live() => self
                .approval_pairing_open(bead, ask, pairing, *ending, end_refused, palette, cx)
                .into_any_element(),
            ApprovalPairing::Shown { pairing, .. } => self
                .approval_pairing_ended(bead, ask, &pairing.device, &pairing.user, true, palette)
                .into_any_element(),
            ApprovalPairing::Ended {
                device,
                user,
                expired,
                ..
            } => self
                .approval_pairing_ended(bead, ask, device, user, *expired, palette)
                .into_any_element(),
        }
    }

    /// The sitting's QR screen: its QR for the new device to scan, its
    /// code as text for one with no camera, and the minutes the code is
    /// good for, with End. The QR string is the transient's verbatim.
    #[allow(clippy::too_many_arguments)]
    fn approval_pairing_open(
        &self,
        bead: &str,
        ask: u64,
        pairing: &api::AskPairing,
        ending: bool,
        end_refused: &str,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let mut open = div().flex().flex_col().gap_3();
        open = open.child(
            div()
                .debug_selector(|| format!("pair head {bead} {ask}"))
                .font_weight(FontWeight::SEMIBOLD)
                .child(format!(
                    "Pair {} for {}",
                    model::plain(&pairing.device),
                    model::plain(&pairing.user)
                )),
        );
        match qr_png(&pairing.qr) {
            Some(png) if !pairing.qr.is_empty() => {
                let image =
                    std::sync::Arc::new(gpui::Image::from_bytes(gpui::ImageFormat::Png, png));
                let faint = palette.faint;
                open = open.child(
                    img(image)
                        .w(px(240.))
                        .h(px(240.))
                        .debug_selector(|| format!("pair qr {bead} {ask}"))
                        .with_fallback(move || {
                            div()
                                .text_sm()
                                .text_color(faint)
                                .child("the QR would not draw: type the code")
                                .into_any_element()
                        }),
                );
            }
            _ => {
                open = open.child(
                    div()
                        .text_sm()
                        .text_color(palette.faint)
                        .debug_selector(|| format!("pair qr unreadable {bead} {ask}"))
                        .child("the QR would not draw: type the code"),
                );
            }
        }
        let code = pairing.code.clone();
        open = open.child(
            div()
                .debug_selector(|| format!("pair code {bead} {ask}"))
                .child(select::text(
                    select::key(("approval-pairing", "code", pairing.code.as_str())),
                    fenced(&code),
                    &self.selection,
                )),
        );
        open = open.child(
            div()
                .text_sm()
                .text_color(palette.faint)
                .debug_selector(|| format!("pair left {bead} {ask}"))
                .child(model::pairing_left(&pairing.expires, &chrono::Utc::now())),
        );
        if ending {
            open = open.child(
                div()
                    .flex()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| format!("pair ending {bead} {ask}"))
                    .child("Ending…"),
            );
        } else {
            open = open.child(
                div().flex().child(
                    small_button("End".to_string(), palette)
                        .id("pair end")
                        .debug_selector(|| format!("pair end {bead} {ask}"))
                        .on_click(cx.listener(|this, _, _, cx| this.end_approval_pairing(cx))),
                ),
            );
        }
        if !end_refused.is_empty() {
            open = open.child(
                div()
                    .text_sm()
                    .text_color(palette.warn)
                    .debug_selector(|| format!("pair refused {bead} {ask}"))
                    .child(end_refused.to_string()),
            );
        }
        open
    }

    /// The sitting's ended state: the pairing ended, or its code ran out.
    /// No code and no QR stand in it.
    fn approval_pairing_ended(
        &self,
        bead: &str,
        ask: u64,
        device: &str,
        user: &str,
        expired: bool,
        palette: Palette,
    ) -> impl IntoElement + use<> {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .debug_selector(|| format!("pair head {bead} {ask}"))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!(
                        "Pair {} for {}",
                        model::plain(device),
                        model::plain(user)
                    )),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| format!("pair ended {bead} {ask}"))
                    .child(match expired {
                        true => "the code ran out".to_string(),
                        false => "the pairing ended".to_string(),
                    }),
            )
    }

    /// The grants pane: every source × holder × call the yard serves,
    /// by source, with each call's own time and approver, the weighty
    /// words for update and `--as` rows in rust as on a card, and each
    /// row's revoke line to copy. It changes nothing itself. A yard too
    /// old for the call says so, in plain words, and so does one that
    /// serves it on its own machine alone.
    pub(crate) fn talk_grants(
        &self,
        model: &Model,
        palette: Palette,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let pane = div().flex_1().min_w_0().h_full().flex().flex_col();
        let said = match (model.grants_too_old(), model.grants_on_the_yard()) {
            (true, _) => Some(("grants too old", model::GRANTS_TOO_OLD)),
            (false, true) => Some(("grants on the yard", model::GRANTS_ON_THE_YARD)),
            (false, false) => None,
        };
        if let Some((selector, words)) = said {
            return pane
                .children(self.reconnect_strip(palette))
                .child(
                    div()
                        .p_3()
                        .border_b_1()
                        .border_color(palette.line)
                        .child(div().font_weight(FontWeight::SEMIBOLD).child("Grants")),
                )
                .child(
                    div().flex_1().min_h_0().p_4().child(
                        div()
                            .debug_selector(move || selector.to_string())
                            .text_color(palette.faint)
                            .child(words),
                    ),
                );
        }
        let grants = model.grant_records();
        if grants.is_empty() {
            return pane
                .children(self.reconnect_strip(palette))
                .child(
                    div()
                        .p_3()
                        .border_b_1()
                        .border_color(palette.line)
                        .child(div().font_weight(FontWeight::SEMIBOLD).child("Grants")),
                )
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(palette.faint)
                        .debug_selector(|| "grants empty".to_string())
                        .child("no call of a source is granted"),
                );
        }
        let head = div()
            .p_3()
            .border_b_1()
            .border_color(palette.line)
            .flex()
            .flex_col()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .debug_selector(|| "grants head".to_string())
                    .child("Grants"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| "grants summary".to_string())
                    .child(model::grant_summary(grants)),
            );
        let mut body = div()
            .id("grants")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.decision)
            .p_4()
            .flex()
            .flex_col()
            .gap_4();
        let mut last_source = "";
        for grant in model::ordered_grants(grants) {
            if grant.source.as_str() != last_source {
                last_source = grant.source.as_str();
                body = body.child(
                    div()
                        .debug_selector({
                            let source = grant.source.clone();
                            move || format!("grants source {source}")
                        })
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.faint)
                        .child(model::plain(&grant.source)),
                );
            }
            for call in &grant.calls {
                body = body.child(self.grant_row(grant, call, palette));
            }
        }
        for line in model::grant_unkept(grants) {
            body = body.child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| "grants unkept".to_string())
                    .child(line),
            );
        }
        pane.children(self.reconnect_strip(palette))
            .child(head)
            .child(body)
            .when(!model.notice.is_empty(), |el| {
                el.child(
                    div()
                        .px_4()
                        .py_1()
                        .text_sm()
                        .text_color(palette.warn)
                        .child(format!("! {}", model.notice)),
                )
            })
    }

    /// One grant row: who holds what call, the weighty lines where the
    /// call is an update or the grant is as a user, when it was granted
    /// and by whom, and the line that takes it back, to copy.
    fn grant_row(
        &self,
        grant: &api::SourceGrant,
        call: &str,
        palette: Palette,
    ) -> impl IntoElement + use<> {
        let holder = model::grant_holder(grant);
        let (granted, by) = model::grant_given(grant.given.get(call));
        let granted_words = match granted.as_str() {
            "not kept" => "granted not kept".to_string(),
            granted => format!("granted {granted} by {by}"),
        };
        let revoke = model::grant_revoke(&grant.source, call, grant);
        let key = select::key((
            &grant.source,
            &grant.group,
            &grant.crew,
            &grant.as_user,
            call,
        ));
        let mut row = div()
            .debug_selector({
                let source = grant.source.clone();
                let call = call.to_string();
                move || format!("grants row {source} {call}")
            })
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .debug_selector({
                        let source = grant.source.clone();
                        let call = call.to_string();
                        move || format!("grants holder {source} {call}")
                    })
                    .child(format!("{holder} — {}", model::plain(call))),
            );
        for (n, line) in model::grant_weight_lines(&grant.source, call, grant)
            .iter()
            .enumerate()
        {
            let line = line.clone();
            row = row.child(
                div()
                    .debug_selector({
                        let source = grant.source.clone();
                        let call = call.to_string();
                        move || format!("grants weight {source} {call} {n}")
                    })
                    .text_color(palette.text_accent)
                    .child(select::text(
                        expanded_text_key(key, "weight", n, &line),
                        line,
                        &self.selection,
                    )),
            );
        }
        row = row.child(
            div()
                .text_sm()
                .text_color(palette.faint)
                .debug_selector({
                    let source = grant.source.clone();
                    let call = call.to_string();
                    move || format!("grants given {source} {call}")
                })
                .child(granted_words),
        );
        row.child(select::text(
            expanded_text_key(key, "revoke", 0, &revoke),
            fenced(&revoke),
            &self.selection,
        ))
    }

    /// The teammates pane: every teammate of the yard's mapping, with
    /// their yard name, email, role, who invited them and when, and
    /// their paired-or-waiting state, and each row's Remove, which files
    /// the uninvite run ask. The operator's own row has none: the yard
    /// refuses self-removal. A yard too old for the call says so, in
    /// plain words.
    pub(crate) fn talk_teammates(
        &self,
        model: &Model,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let pane = div().flex_1().min_w_0().h_full().flex().flex_col();
        if model.teammates_too_old() {
            return pane
                .children(self.reconnect_strip(palette))
                .child(
                    div()
                        .p_3()
                        .border_b_1()
                        .border_color(palette.line)
                        .child(div().font_weight(FontWeight::SEMIBOLD).child("Teammates")),
                )
                .child(
                    div().flex_1().min_h_0().p_4().child(
                        div()
                            .debug_selector(|| "teammates too old".to_string())
                            .text_color(palette.faint)
                            .child(model::TEAMMATES_TOO_OLD),
                    ),
                );
        }
        let teammates = model.teammate_records();
        if teammates.is_empty() {
            return pane
                .children(self.reconnect_strip(palette))
                .child(
                    div()
                        .p_3()
                        .border_b_1()
                        .border_color(palette.line)
                        .child(div().font_weight(FontWeight::SEMIBOLD).child("Teammates")),
                )
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(palette.faint)
                        .debug_selector(|| "teammates empty".to_string())
                        .child("no teammate is invited yet"),
                );
        }
        let head = div()
            .p_3()
            .border_b_1()
            .border_color(palette.line)
            .flex()
            .flex_col()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .debug_selector(|| "teammates head".to_string())
                    .child("Teammates"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| "teammates summary".to_string())
                    .child(model::teammates_summary(teammates)),
            );
        let mut body = div()
            .id("teammates")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.decision)
            .p_4()
            .flex()
            .flex_col()
            .gap_4();
        for teammate in teammates {
            body = body.child(
                self.teammate_row(
                    teammate,
                    teammate.name == model.me(),
                    self.removing
                        .contains(&(self.session, teammate.name.clone())),
                    palette,
                    cx,
                ),
            );
        }
        pane.children(self.reconnect_strip(palette))
            .child(head)
            .child(body)
            .when(!model.notice.is_empty(), |el| {
                el.child(
                    div()
                        .px_4()
                        .py_1()
                        .text_sm()
                        .text_color(palette.warn)
                        .child(format!("! {}", model.notice)),
                )
            })
    }

    /// One teammate's row: their yard name with Remove beside it, their
    /// email, their role, who invited them and when, and whether a
    /// device is paired. The operator's own row has no Remove, and a
    /// removal underway says so in its place.
    fn teammate_row(
        &self,
        teammate: &api::Teammate,
        own: bool,
        removing: bool,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let name = teammate.name.clone();
        let mut head = div().flex().items_center().justify_between().gap_2().child(
            div()
                .flex_1()
                .truncate()
                .debug_selector({
                    let name = name.clone();
                    move || format!("teammates name {name}")
                })
                .font_weight(FontWeight::SEMIBOLD)
                .child(model::plain(&teammate.name)),
        );
        if !own {
            if removing {
                head = head.child(
                    div()
                        .text_sm()
                        .text_color(palette.faint)
                        .debug_selector({
                            let name = name.clone();
                            move || format!("teammates removing {name}")
                        })
                        .child("Removing…"),
                );
            } else {
                head = head.child(
                    small_button("Remove".to_string(), palette)
                        .id(SharedString::from(format!("teammates remove {name}")))
                        .debug_selector({
                            let name = name.clone();
                            move || format!("teammates remove {name}")
                        })
                        .on_click({
                            let name = name.clone();
                            cx.listener(move |this, _, _, cx| {
                                this.remove_teammate(name.clone(), cx)
                            })
                        }),
                );
            }
        }
        div()
            .debug_selector({
                let name = name.clone();
                move || format!("teammates row {name}")
            })
            .flex()
            .flex_col()
            .gap_1()
            .child(head)
            .child(
                div()
                    .text_sm()
                    .debug_selector({
                        let name = name.clone();
                        move || format!("teammates email {name}")
                    })
                    .child(model::plain(&teammate.email)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let name = name.clone();
                        move || format!("teammates role {name}")
                    })
                    .child(teammate_role(&teammate.role)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector({
                        let name = name.clone();
                        move || format!("teammates invited {name}")
                    })
                    .child(model::teammate_invited(teammate)),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(move || format!("teammates state {name}"))
                    .child(model::teammate_state(teammate)),
            )
    }

    /// The Pair a phone pane: the device's name with Open where no
    /// pairing is open, and the open pairing's QR, code and minutes
    /// left with End where one is. A yard too old for the calls says
    /// so, in plain words.
    pub(crate) fn talk_pairing(
        &self,
        model: &Model,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let pane = div().flex_1().min_w_0().h_full().flex().flex_col();
        if model.pairing_too_old() {
            return pane
                .children(self.reconnect_strip(palette))
                .child(
                    div().p_3().border_b_1().border_color(palette.line).child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Pair a phone"),
                    ),
                )
                .child(
                    div().flex_1().min_h_0().p_4().child(
                        div()
                            .debug_selector(|| "pairing too old".to_string())
                            .text_color(palette.faint)
                            .child(model::PAIRING_TOO_OLD),
                    ),
                );
        }
        let summary = match model.pairing() {
            Some(pairing) => format!("open for {}", model::plain(&pairing.device)),
            None => "no pairing open".to_string(),
        };
        let head = div()
            .p_3()
            .border_b_1()
            .border_color(palette.line)
            .flex()
            .flex_col()
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .debug_selector(|| "pairing head".to_string())
                    .child("Pair a phone"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| "pairing summary".to_string())
                    .child(summary),
            );
        let mut body = div()
            .id("pairing")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.decision)
            .p_4()
            .flex()
            .flex_col()
            .gap_3();
        match model.pairing() {
            Some(pairing) => body = body.child(self.pairing_open(pairing, palette, cx)),
            None => body = body.child(self.pairing_form(palette, cx)),
        }
        if !model.pairing_line().is_empty() {
            body = body.child(
                div()
                    .text_sm()
                    .text_color(palette.warn)
                    .debug_selector(|| "pairing line".to_string())
                    .child(model.pairing_line().to_string()),
            );
        }
        pane.children(self.reconnect_strip(palette))
            .child(head)
            .child(body)
            .when(!model.notice.is_empty(), |el| {
                el.child(
                    div()
                        .px_4()
                        .py_1()
                        .text_sm()
                        .text_color(palette.warn)
                        .child(format!("! {}", model.notice)),
                )
            })
    }

    /// The open pairing: its QR for a phone to scan, its code as text,
    /// and the minutes the code is good for, with End. Where the yard
    /// names no address the QR is empty, and the code is typed. Where
    /// the state was not read yet the QR is still away.
    fn pairing_open(
        &self,
        pairing: &api::DevicePairing,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let mut open = div().flex().flex_col().gap_3();
        match pairing.qr.as_deref() {
            Some(qr) if !qr.is_empty() => match qr_png(qr) {
                Some(png) => {
                    let image =
                        std::sync::Arc::new(gpui::Image::from_bytes(gpui::ImageFormat::Png, png));
                    let faint = palette.faint;
                    open = open.child(
                        img(image)
                            .w(px(240.))
                            .h(px(240.))
                            .debug_selector(|| "pairing qr".to_string())
                            .with_fallback(move || {
                                div()
                                    .text_sm()
                                    .text_color(faint)
                                    .child("the QR would not draw: type the code")
                                    .into_any_element()
                            }),
                    );
                }
                None => {
                    open = open.child(
                        div()
                            .text_sm()
                            .text_color(palette.faint)
                            .debug_selector(|| "pairing qr unreadable".to_string())
                            .child("the QR would not draw: type the code"),
                    );
                }
            },
            Some(_) => {
                open = open.child(
                    div()
                        .text_sm()
                        .text_color(palette.faint)
                        .debug_selector(|| "pairing no address".to_string())
                        .child("this yard names no address: type the code"),
                );
            }
            None => {
                open = open.child(
                    div()
                        .text_sm()
                        .text_color(palette.faint)
                        .debug_selector(|| "pairing qr waiting".to_string())
                        .child("reading the QR…"),
                );
            }
        }
        let code = pairing.code.clone();
        open = open.child(
            div()
                .debug_selector(|| "pairing code".to_string())
                .child(select::text(
                    select::key(("pairing", "code", pairing.code.as_str())),
                    fenced(&code),
                    &self.selection,
                )),
        );
        open = open.child(
            div()
                .text_sm()
                .text_color(palette.faint)
                .debug_selector(|| "pairing left".to_string())
                .child(model::pairing_left(&pairing.expires, &chrono::Utc::now())),
        );
        if self.pairing_ending {
            open = open.child(
                div()
                    .flex()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| "pairing ending".to_string())
                    .child("Ending…"),
            );
        } else {
            open = open.child(
                div().flex().child(
                    small_button("End".to_string(), palette)
                        .id("pairing end")
                        .debug_selector(|| "pairing end".to_string())
                        .on_click(cx.listener(|this, _, _, cx| this.close_phone_pairing(cx))),
                ),
            );
        }
        open
    }

    /// The form where no pairing is open: the device's name with Open.
    /// Open is dimmed with no name typed, and while an opening is away.
    fn pairing_form(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let focus = self.device_name.focus_handle(cx);
        let mut form = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().w(rems(5.)).flex_none().child("device"))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .max_w(rems(28.))
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .border_1()
                            .border_color(palette.line)
                            .bg(palette.side)
                            .id("pairing name")
                            .debug_selector(|| "pairing name".to_string())
                            .on_mouse_down(MouseButton::Left, move |_, window, _| {
                                window.focus(&focus);
                                window.prevent_default();
                            })
                            .child(self.device_name.clone()),
                    ),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| "pairing help".to_string())
                    .child("names the phone: Open shows its code and QR for 10 minutes"),
            );
        if self.pairing_opening {
            form = form.child(
                div()
                    .flex()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| "pairing opening".to_string())
                    .child("Opening…"),
            );
        } else {
            let empty = self.device_name.read(cx).text().trim().is_empty();
            let click = (!empty).then(|| cx.listener(|this, _, _, cx| this.open_phone_pairing(cx)));
            let mut open = small_button("Open".to_string(), palette)
                .id("pairing open")
                .debug_selector(|| "pairing open".to_string())
                .when(empty, |el| el.opacity(0.5));
            if let Some(click) = click {
                open = open.on_click(click);
            }
            form = form.child(div().flex().child(open));
        }
        form
    }
}

/// A give that ended in no yard's answer: why, as the card says it, and
/// whether the call got no answer at all, so that the yard may have stored
/// the values all the same. They are handed back from the call then, never
/// to be shown, and nothing else holds a second copy meanwhile.
struct GiveStopped {
    why: String,
    lost: bool,
    values: Vec<(String, String)>,
}

/// Takes the words that went from the start of what is kept for the ask
/// `at`, where a read of the yard says it took them: what follows them,
/// typed since, stays, and nothing is kept for an ask whose words are all
/// gone.
fn gone_from(kept: &mut HashMap<(String, u64), String>, at: &(String, u64), words: &str) {
    let Some(rest) = kept
        .get(at)
        .and_then(|typed| draft::without_sent(typed, words))
    else {
        return;
    };
    match rest.is_empty() {
        true => kept.remove(at),
        false => kept.insert(at.clone(), rest),
    };
}

/// A field of a yard that is asked for, with its name beside it: the
/// focus moves to it on a click. Under the yards with the other
/// fields, or in the renamed card with its form.
pub(crate) fn yard_field(
    name: &'static str,
    line: &Entity<Line>,
    palette: Palette,
    cx: &mut Context<Footplate>,
) -> Div {
    let focus = line.focus_handle(cx);
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(div().w(rems(5.)).flex_none().child(name))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .max_w(rems(28.))
                .px_3()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(palette.line)
                .bg(palette.side)
                .debug_selector(move || format!("yard {name}"))
                .on_mouse_down(MouseButton::Left, move |_, window, _| {
                    window.focus(&focus);
                    window.prevent_default();
                })
                .child(line.clone()),
        )
}

/// The button under a yard's fields: Enter in a field presses it too.
/// Dimmed while a pairing or a sign-in is away.
pub(crate) fn yard_submit(
    text: &str,
    id: &'static str,
    busy: bool,
    palette: Palette,
    cx: &mut Context<Footplate>,
) -> Div {
    div().flex().child(
        small_button(text.to_string(), palette)
            .id(id)
            .debug_selector(move || id.to_string())
            .when(busy, |el| el.opacity(0.5))
            .on_click(cx.listener(|this, _, _, cx| this.submit_yard(cx))),
    )
}

/// The help under a yard's fields: one faint line.
pub(crate) fn yard_help(text: String, palette: Palette) -> Div {
    div()
        .debug_selector(|| "yard help".to_string())
        .text_sm()
        .text_color(palette.faint)
        .child(text)
}

/// The QR string `qr` as a PNG of dark modules on a light ground:
/// what the Pair a phone pane draws for a phone to scan. Scanners read
/// dark on light, so the colours are black and white in both themes.
/// None where the string is no QR code's.
pub(crate) fn qr_png(qr: &str) -> Option<Vec<u8>> {
    use image::ImageEncoder;
    let code = qrcode::QrCode::new(qr.as_bytes()).ok()?;
    let image = code
        .render::<image::Luma<u8>>()
        .dark_color(image::Luma([0]))
        .light_color(image::Luma([255]))
        .max_dimensions(240, 240)
        .build();
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &image,
            image.width(),
            image.height(),
            image::ExtendedColorType::L8,
        )
        .ok()?;
    (!png.is_empty()).then_some(png)
}

/// The directory `dir` as a `file:` URL, for the settings footer's
/// reveal: what the opener is handed. A space in it (macOS's
/// "Application Support") is encoded, so it still opens.
pub(crate) fn dir_url(dir: &Path) -> String {
    file_url(&dir.to_string_lossy(), cfg!(windows))
}

/// `path` as a `file:` URL. On Windows, `windows`, a path is
/// `C:\Users\jo` and its URL `file:///C:/Users/jo`: the backslashes are
/// slashes and the drive stands after three of them, as a share's
/// `\\host\share` is `file://host/share`. Elsewhere a backslash is a
/// character of a name, and is encoded as any other.
pub(crate) fn file_url(path: &str, windows: bool) -> String {
    let drive =
        windows && matches!(path.as_bytes(), [letter, b':', ..] if letter.is_ascii_alphabetic());
    let share = windows && path.starts_with(r"\\");
    let path = match windows {
        true => path.replace('\\', "/"),
        false => path.to_string(),
    };
    let mut url = match (drive, share) {
        (true, _) => "file:///",
        (_, true) => "file:",
        _ => "file://",
    }
    .to_string();
    for (at, byte) in path.bytes().enumerate() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                url.push(byte as char)
            }
            // The drive's colon stands as it is.
            b':' if drive && at == 1 => url.push(':'),
            _ => url.push_str(&format!("%{byte:02X}")),
        }
    }
    url
}

/// A dealt with row's shape, one ask's or a fold's, as the open rows stand:
/// the outcome's mark where their lamp stands (an approval's in the go
/// colour, the rest quiet), the question's first line, then the outcome in
/// the text colour with `detail` quiet after it, and `right` at the end of
/// that line.
/// The debug selectors are `<kind> head <key>`, `<kind> outcome <key>` and
/// `<kind> icon <key>`.
#[allow(clippy::too_many_arguments)]
fn decided_lines(
    mark: Outcome,
    outcome: &str,
    head: &str,
    detail: &str,
    key: String,
    kind: &'static str,
    right: Div,
    palette: Palette,
) -> Div {
    let icon_color = match mark {
        Outcome::Approved => palette.ok,
        _ => palette.faint,
    };
    let words = outcome.to_string();
    let (icon_key, head_key, outcome_key) = (key.clone(), key.clone(), key);
    div()
        .mx_1()
        .px_2()
        .py_1p5()
        .rounded_md()
        .flex()
        .items_start()
        .cursor_pointer()
        .border_1()
        .border_color(rgba(0x00000000))
        .child(
            // The mark stands in the lamp's room, on the head's first line.
            div()
                .flex_none()
                .w(px(ROW_LAMP + ROW_LAMP_GAP))
                .h(rems(TYPE_LABEL * 1.5))
                .flex()
                .items_center()
                .debug_selector(move || format!("{kind} icon {icon_key}"))
                .child(outcome_glyph(mark, ROW_MARK, icon_color)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .child(
                    div()
                        .truncate()
                        .text_size(rems(TYPE_LABEL))
                        .font_weight(FontWeight::MEDIUM)
                        .debug_selector(move || format!("{kind} head {head_key}"))
                        .child(shorten(head, ROW_TITLE_CHARS)),
                )
                .child(
                    div()
                        .flex()
                        .items_baseline()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_baseline()
                                .gap_1()
                                .min_w_0()
                                .child(
                                    div()
                                        .flex_none()
                                        .max_w(relative(0.6))
                                        .text_xs()
                                        .truncate()
                                        .debug_selector(move || {
                                            format!("{kind} outcome {outcome_key}")
                                        })
                                        .child(words),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_xs()
                                        .text_color(palette.faint)
                                        .truncate()
                                        .child(format!("· {detail}")),
                                ),
                        )
                        .child(right),
                ),
        )
}
