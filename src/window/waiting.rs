//! The parked beads' views: a bead's row in the decisions' Waiting section,
//! the card that offers the moves the yard serves it, and what the moves
//! do: pick one, write its note, make it.

use gpui::{
    ClickEvent, Context, Div, Focusable, FontWeight, SharedString, Stateful, Window, div,
    prelude::*, px, rgba,
};

use super::Footplate;
use super::metrics::WAITING_SUB;
use super::ui::{field, keys_hint, shorten};
use crate::api::{Address, Reach};
use crate::model::{self, Effect, Model, Selected, WaitingDetail, WaitingMove, WaitingRow};
use crate::select;
use crate::theme::{Palette, yard_color};

/// What the card says under its moves while the yard is reconnecting, which
/// stand faint: nothing is queued, and the move is made by hand once the
/// yard answers.
pub(crate) const MOVE_HELD: &str = "the yard is not answering; make the move once it does";

/// What the card says while a move is with the yard: no second one goes
/// after it.
pub(crate) const MOVE_UNDERWAY: &str = "the move is with the yard";

/// What a move that wants words says on its button until the line holds
/// some.
pub(crate) const NEEDS_NOTE: &str = "needs a note";

/// What the card says of a bead whose flow does not resolve, under the
/// yard's own words: no move exists to offer until the flow is fixed.
pub(crate) const NO_MOVE_UNTIL_FIXED: &str = "no move is offered until the flow is fixed";

/// What the Lines details of a bead in a stage of people say, where the
/// moves would stand, on a yard from before `Waiting`: the stage's
/// mechanism, to every reader alike. It offers no button and does not say
/// that this reader may move the bead.
pub(crate) const STAGE_HELP: &str =
    "this stage is moved by its people: yardr review on the yard's machine offers them its moves";

/// The keys of a parked bead's card, as they stand under it. Several moves
/// are picked by number; a lone move stands picked.
pub(crate) fn waiting_keys(moves: usize) -> String {
    match moves {
        0 => String::new(),
        1 => "i note · enter makes the move".to_string(),
        n => format!("1-{n} pick a move · i note · enter makes it"),
    }
}

/// The keys of a folded card, as they stand under it: the moves are shown
/// first, then picked as today.
pub(crate) fn waiting_keys_folded() -> String {
    "enter shows the moves · i note".to_string()
}

/// What the folded moves' control says on its face: the yard's words for
/// a move to a stage, with no stage named.
pub(crate) const MOVE_TO: &str = "Move to ...";

/// What the folded moves' control says of how many moves stand behind it,
/// at its right.
pub(crate) fn folded_count(moves: usize) -> String {
    format!("{moves} moves ▸")
}

/// What Enter says when it makes no move because none is picked.
pub(crate) fn pick_a_move(moves: usize) -> String {
    format!("pick a move first: 1-{moves}, or the arrows")
}

/// What Enter says of a move whose kind this build does not know.
pub(crate) fn not_makeable(label: &str, kind: &str) -> String {
    format!("{label} cannot be made from here: footplate knows no move of kind {kind}")
}

/// The row's meta line, shortened before a flex item clips it at its edge.
fn waiting_sub(text: &str) -> String {
    shorten(text, WAITING_SUB)
}

/// What Enter says when the move it would make wants words.
pub(crate) fn note_wanted(label: &str) -> String {
    format!("{label} {NEEDS_NOTE}: say what should change in the line (i)")
}

impl Footplate {
    /// The bead whose moves are in view: a parked bead's card, or the Lines
    /// details of a bead the yard parks for the person once the bead is
    /// read, whose block is the card's own. The settings stand where the
    /// card does, and the Feedback form over it, and the keys are not for
    /// moves that are not in view.
    pub(crate) fn waiting_card(&self) -> Option<String> {
        if self.settings_open || self.feedback.is_some() {
            return None;
        }
        let model = self.model.as_ref()?;
        match model.selected()? {
            Selected::Waiting(bead) => Some(bead.clone()),
            // The details draw "Reading the bead." and no block until the
            // bead is read; a drawer beside another row shuts as a bead
            // opens full width, so none is open here.
            Selected::Bead(bead) if model.is_parked(bead) && model.has_bead(bead) => {
                Some(bead.clone())
            }
            _ => None,
        }
    }

    /// How many moves the open parked bead's card offers: the line for the
    /// note stands where there are any.
    pub(crate) fn waiting_moves(&self) -> usize {
        let Some(bead) = self.waiting_card() else {
            return 0;
        };
        self.model
            .as_ref()
            .and_then(|m| m.waiting_detail(&bead, &chrono::Utc::now()))
            .map_or(0, |detail| detail.moves.len())
    }

    /// One bead's row of the Waiting section, in the decisions row's shape:
    /// the title wrapped to two lines (a parked bead has no question), then
    /// its stage, its group and how long it waits, with the bead's short id
    /// at the row's end.
    pub(crate) fn waiting_row(
        &self,
        row: WaitingRow,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let sel = Selected::Waiting(row.bead.clone());
        div()
            .id(SharedString::from(format!("waiting {}", row.bead)))
            .debug_selector(|| format!("waiting row {}", row.bead))
            .mx_1()
            .px_2()
            .py_1p5()
            .rounded_md()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .cursor_pointer()
            .border_1()
            .border_color(rgba(0x00000000))
            .when(row.selected, |el| el.bg(palette.open))
            .when(!row.selected, |el| {
                el.hover(|style| style.border_color(palette.line))
            })
            .on_click(cx.listener(move |this, _, window, cx| this.select(sel.clone(), window, cx)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    // Resolve the available row width before measuring the
                    // head, so it wraps within the row.
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .line_clamp(2)
                            .font_weight(FontWeight::MEDIUM)
                            .debug_selector(|| format!("waiting head {}", row.bead))
                            .child(row.head),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(palette.faint)
                            .truncate()
                            .debug_selector(|| format!("waiting sub {}", row.bead))
                            .child(waiting_sub(&row.sub)),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    .text_xs()
                    .text_color(palette.faint)
                    .debug_selector(|| format!("waiting id {}", row.bead))
                    .child(row.short),
            )
    }

    /// A parked bead's card: the yard, then the bead's title and the one
    /// line that names it, with the route and Open where the web view
    /// answers, as the Lines bead detail draws them; the bead whole under
    /// it; and the moves the yard offers, each in its words, with a line
    /// for the note that goes with the move.
    pub(crate) fn talk_waiting(
        &self,
        model: &Model,
        bead: &str,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let pane = div().flex_1().min_w_0().h_full().flex().flex_col();
        let now = chrono::Utc::now();
        let Some(detail) = model.waiting_detail(bead, &now) else {
            return pane
                .items_center()
                .justify_center()
                .text_color(palette.faint)
                .child("It waits for no move of yours now.");
        };
        // The probe answered at this address, so it parses: the same
        // reading, not another. With no view for the yard there is no Open.
        let url = match (self.view_base(), self.view_up_now()) {
            (Some(base), Some(Reach::Up)) => Address::parse_view(&base)
                .expect("the web view answered at this address")
                .bead_url(&detail.bead),
            _ => None,
        };
        let route = model.route_for(&detail.bead);
        let head = div()
            .p_3()
            .border_b_1()
            .border_color(palette.line)
            .debug_selector(|| format!("in {} {}", self.yard, detail.bead))
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1()
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
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(palette.faint)
                                    .child(self.yard.clone()),
                            ),
                    )
                    .children(url.map(|url| {
                        let open = self.means.open.clone();
                        let bead = detail.bead.clone();
                        div()
                            .id(SharedString::from(format!("waiting open {bead}")))
                            .debug_selector({
                                let bead = bead.clone();
                                move || format!("waiting open {bead}")
                            })
                            .text_sm()
                            .text_color(palette.link)
                            .cursor_pointer()
                            .on_click(move |_, _, cx| (open)(&url, cx))
                            .child("Open")
                    })),
            )
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .debug_selector(|| format!("waiting title {}", detail.bead))
                    .child(detail.title.clone()),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(palette.faint)
                    .debug_selector(|| format!("waiting line {}", detail.bead))
                    .child(detail.line.clone()),
            )
            .children(
                route
                    .as_ref()
                    .map(|route| self.route_strip(route, &detail.bead, palette, "waiting")),
            );
        let list = div()
            .id("waiting card")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.decision)
            .p_4();
        // The waiting card's frame: the bead whole with its moves, as an
        // open card holds its disclosure's bead.
        let card = div()
            .debug_selector(|| format!("waiting card {bead}"))
            .flex()
            .flex_col()
            .gap_2p5()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(palette.line)
            .bg(palette.raised);
        let card = match model.arrival_detail(bead) {
            Some(whole) => card.child(self.bead_body(
                select::key(("waiting", &whole.bead)),
                &whole,
                palette,
                cx,
            )),
            None => card.child(div().text_color(palette.faint).child("Reading the bead.")),
        };
        let list = list.child(card.child(self.moves_block(&detail, palette, cx)));
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

    /// The moves under the bead, on its card and at the end of its Lines
    /// details alike: the yard's error where its flow does not resolve, or
    /// its words for the stage's template that broke and the moves all the
    /// same; one button for each move; the line for the note; why the yard
    /// refused the last move; and the keys.
    pub(crate) fn moves_block(
        &self,
        detail: &WaitingDetail,
        palette: Palette,
        cx: &mut Context<Self>,
    ) -> Div {
        let bead = detail.bead.clone();
        let mut block = div().flex().flex_col().gap_2p5();
        // A flow that does not resolve offers no move: the error stands
        // where the moves would, and the bead stays readable whole.
        if !detail.flow_error.is_empty() {
            return block
                .child(
                    div()
                        .debug_selector(|| format!("waiting flow error {bead}"))
                        .text_sm()
                        .text_color(palette.warn)
                        .child(format!("! {}", detail.flow_error)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(palette.faint)
                        .child(NO_MOVE_UNTIL_FIXED),
                );
        }
        // A stage whose ask did not come: the error stands where it
        // would, and the moves under it.
        if !detail.template_error.is_empty() {
            block = block.child(
                div()
                    .debug_selector(|| format!("waiting template error {bead}"))
                    .text_sm()
                    .text_color(palette.warn)
                    .child(format!("! {}", detail.template_error)),
            );
        }
        if detail.moves.is_empty() {
            return block;
        }
        let words = self.line.read(cx).text().to_string();
        let has_words = !words.trim().is_empty();
        // While the yard is reconnecting the moves are held as a
        // conversation's Send is: faint, with the reason under them.
        let held = self.reconnecting();
        let underway = self.moving.contains(&(self.session, bead.clone()));
        let mut buttons = div().flex().flex_col().gap_1();
        // A long uniform list stands folded behind the one control, which
        // unfolds it: expanding is local, so its click stands even while
        // the moves are held.
        if detail.folded {
            let expand = bead.clone();
            buttons = buttons.child(folded_moves_button(
                &bead,
                detail.moves.len(),
                cx.listener(move |this, _: &ClickEvent, _: &mut Window, cx| {
                    this.expand_moves(expand.clone(), cx)
                }),
                palette,
            ));
        } else {
            for how in &detail.moves {
                let waits = how.note_required && !has_words;
                let click = (!waits && !held && !underway).then(|| {
                    let bead = bead.clone();
                    let number = how.number;
                    cx.listener(move |this, _: &ClickEvent, _: &mut Window, cx| {
                        this.make_move(bead.clone(), number, cx)
                    })
                });
                buttons = buttons.child(move_button(&bead, how, waits, click, palette));
            }
        }
        block = block.child(buttons);
        let focus = self.line.focus_handle(cx);
        block = block.child(
            field(focus, self.line.clone(), palette)
                .debug_selector(|| format!("waiting note {bead}")),
        );
        // Why the yard did not take the last move, whole, or that it did
        // not answer: it is the card's, until the next move is made.
        if let Some(why) = self.refused.get(&(bead.clone(), 0)) {
            block = block.child(
                div()
                    .debug_selector(|| format!("waiting refused {bead}"))
                    .text_sm()
                    .text_color(palette.warn)
                    .child(format!("! {why}")),
            );
        }
        if held {
            block = block.child(
                div()
                    .debug_selector(|| format!("waiting held {bead}"))
                    .text_xs()
                    .text_color(palette.faint)
                    .child(MOVE_HELD),
            );
        } else if underway {
            block = block.child(
                div()
                    .debug_selector(|| format!("waiting underway {bead}"))
                    .text_xs()
                    .text_color(palette.faint)
                    .child(MOVE_UNDERWAY),
            );
        }
        block.child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .items_baseline()
                .gap_1()
                .text_xs()
                .text_color(palette.faint)
                .child(div().child(match detail.folded {
                    true => waiting_keys_folded(),
                    false => waiting_keys(detail.moves.len()),
                }))
                .child(div().child("·"))
                .child(keys_hint("keys hint waiting", palette, cx)),
        )
    }

    /// The one line a yard from before `Waiting` gets where a bead's moves
    /// would stand in its Lines details: who moves its stage. It is a line
    /// and no more: nothing to press, and no claim that this reader may move
    /// the bead.
    pub(crate) fn stage_help_line(&self, bead: &str, palette: Palette) -> Div {
        div()
            .debug_selector({
                let bead = bead.to_string();
                move || format!("stage help {bead}")
            })
            .text_sm()
            .text_color(palette.faint)
            .child(STAGE_HELP)
    }

    /// Enter on a parked bead's card, and a click on a move: the move
    /// the pick names, or the only one offered. A move that wants words and
    /// finds the line empty is not made, and says so; so does Enter when
    /// several are offered and none is picked.
    pub(crate) fn make_picked_move(&mut self, bead: String, cx: &mut Context<Self>) {
        // Folded moves stand unfolded first: expanding is local, so it
        // goes even while the yard is not answering.
        if self.model.as_ref().is_some_and(|m| m.moves_folded(&bead)) {
            self.change(cx, |m| {
                m.expand_moves(&bead);
                Vec::new()
            });
            return;
        }
        // Moves are held as a conversation's sends are: nothing is queued.
        if self.reconnecting() {
            return;
        }
        let session = self.session;
        // One at a time: the revision refuses the same move made twice,
        // but the window does not make it twice.
        if self.moving.contains(&(session, bead.clone())) {
            return;
        }
        let Some(model) = self.model.as_ref().filter(|_| self.client.is_some()) else {
            return;
        };
        let Some((how, revision)) = model.move_to_make(&bead) else {
            let offered = model
                .waiting_detail(&bead, &chrono::Utc::now())
                .map_or(0, |detail| detail.moves.len());
            if offered > 1 {
                self.notice(pick_a_move(offered), cx);
            }
            return;
        };
        // A kind this build does not know is not asked of the yard, and so
        // is not said to be unanswered by it.
        if !how.makeable() {
            return self.notice(not_makeable(&how.label, &how.kind), cx);
        }
        let words = self.line.read(cx).text().trim().to_string();
        if how.note_required && words.is_empty() {
            return self.notice(note_wanted(&how.label), cx);
        }
        self.moving.insert((session, bead.clone()));
        self.refused.remove(&(bead.clone(), 0));
        self.notice(String::new(), cx);
        let call = (bead.clone(), words);
        self.ask_in(
            session,
            cx,
            move |client, me| client.move_bead(&call.0, &how, &call.1, &revision, me),
            move |this, moved, cx| {
                this.moving.remove(&(session, bead.clone()));
                match moved {
                    // The bead is not ours to move any more: its row goes
                    // at once, and its note with it. The yard's `advanced`
                    // event reads the board and the flows.
                    Ok(_) => {
                        this.change_in(session, cx, |m| m.moved(&bead));
                        this.forget_note(session, &bead, cx);
                    }
                    // Whatever the yard said, the bead may stand as it did
                    // not when the row was read: the parked beads and the
                    // bead are read again, and the card says why. A call
                    // that ended without an answer is never said to have
                    // moved the bead, nor not to have.
                    Err(err) => {
                        let refused = (bead.clone(), 0);
                        this.refused_in(session, refused, model::not_moved(&err), cx);
                        this.change_in(session, cx, |_| {
                            vec![Effect::Waiting, Effect::Bead { bead }]
                        });
                    }
                }
            },
        );
    }

    /// A click on a move: it is picked, and made at once.
    pub(crate) fn make_move(&mut self, bead: String, number: usize, cx: &mut Context<Self>) {
        self.change(cx, |m| {
            m.choose_move(&bead, number as u64);
            Vec::new()
        });
        self.make_picked_move(bead, cx);
    }

    /// A click on the folded moves' control: the moves stand unfolded,
    /// with none picked.
    pub(crate) fn expand_moves(&mut self, bead: String, cx: &mut Context<Self>) {
        self.change(cx, |m| {
            m.expand_moves(&bead);
            Vec::new()
        });
    }

    /// The note that went with a move is not kept for the bead: it has
    /// moved, and a note typed for its next stage is not this one. It was
    /// kept where the row was left (the line's text goes with the open
    /// row, and a yard that is not open keeps its own). The Lines details
    /// of the bead stay open on it when it is moved, so no row change
    /// empties the line there: it would hold the note when the bead is
    /// parked for the person again.
    fn forget_note(&mut self, session: u64, bead: &str, cx: &mut Context<Self>) {
        let at = (bead.to_string(), 0);
        if session == self.session {
            self.ask_words.remove(&at);
            let open = self.model.as_ref().and_then(|m| m.selected());
            if open == Some(&Selected::Bead(bead.to_string())) {
                self.line.update(cx, |line, cx| line.set_text("", cx));
            }
            return;
        }
        let yard = self
            .held
            .iter()
            .find(|(_, held)| held.session == session)
            .map(|(yard, _)| yard.clone());
        if let Some(kept) = yard.and_then(|yard| self.kept_words.get_mut(&yard)) {
            kept.remove(&at);
        }
    }

    /// Esc on a parked bead's card, with the window holding the keys: the
    /// pick first, then the words, as an ask's card has it. Esc in the line
    /// leaves the line alone, and what is typed stays.
    pub(crate) fn leave_waiting(
        &mut self,
        bead: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.line.focus_handle(cx).is_focused(window) {
            window.focus(&self.focus.clone());
            return;
        }
        let mut cleared = false;
        self.change(cx, |m| {
            cleared = m.clear_move(&bead);
            Vec::new()
        });
        if !cleared && !self.line.read(cx).text().is_empty() {
            self.line.update(cx, |line, cx| line.set_text("", cx));
            self.ask_words.remove(&(bead, 0));
        }
    }
}

/// One move of a card as a button, in the yard's words and as it served
/// them. The pick is the one to take: the primary button, ending in the key
/// that makes it; the rest are secondaries. A move that wants words says so
/// on its button until the line holds some, and stands quiet, as a held or
/// underway move does. A click is the move made.
fn move_button(
    bead: &str,
    how: &WaitingMove,
    waits: bool,
    click: Option<impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static>,
    palette: Palette,
) -> Stateful<Div> {
    let id = format!("waiting move {bead} {}", how.number);
    let enabled = click.is_some();
    let label = match how.chosen {
        true => format!("{} ↵", how.label),
        false => how.label.clone(),
    };
    div()
        .id(SharedString::from(id.clone()))
        .debug_selector(|| id.clone())
        .min_h(px(32.))
        .px_3()
        .py_1()
        .rounded_md()
        // The label keeps the button's width and the words that wait for a
        // note drop under it where both do not fit: the yard's label is
        // never squeezed to make room for them.
        .flex()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap_x_2()
        .border_1()
        .when(how.chosen && enabled, |el| {
            el.bg(palette.button)
                .border_color(palette.button)
                .text_color(palette.on_button)
                .hover(|style| style.bg(palette.button_hover))
                .active(|style| style.bg(palette.button_pressed))
        })
        .when(!how.chosen && enabled, |el| {
            el.bg(palette.raised)
                .border_color(palette.strong)
                .text_color(palette.text)
                .hover(|style| style.bg(palette.hover))
                .active(|style| style.bg(palette.open))
        })
        .when(!enabled, |el| {
            el.border_color(palette.line).text_color(palette.disabled)
        })
        .when(enabled, |el| el.cursor_pointer())
        .when_some(click, |el, click| el.on_click(click))
        .child(div().min_w_0().font_weight(FontWeight::MEDIUM).child(label))
        .children(waits.then(|| {
            div()
                .flex_none()
                .text_sm()
                .text_color(palette.faint)
                .child(NEEDS_NOTE)
        }))
}

/// A folded list's one control: the yard's "Move to ..." words with how
/// many moves stand behind it. A secondary, since it stands where the moves
/// do; a click stands them unfolded.
fn folded_moves_button(
    bead: &str,
    count: usize,
    click: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static,
    palette: Palette,
) -> Stateful<Div> {
    let id = format!("waiting moves {bead}");
    div()
        .id(SharedString::from(id.clone()))
        .debug_selector(|| id.clone())
        .min_h(px(32.))
        .px_3()
        .py_1()
        .rounded_md()
        .flex()
        .items_center()
        .justify_between()
        .gap_x_2()
        .border_1()
        .bg(palette.raised)
        .border_color(palette.strong)
        .text_color(palette.text)
        .hover(|style| style.bg(palette.hover))
        .active(|style| style.bg(palette.open))
        .cursor_pointer()
        .on_click(click)
        .child(
            div()
                .min_w_0()
                .font_weight(FontWeight::MEDIUM)
                .child(MOVE_TO),
        )
        .child(
            div()
                .flex_none()
                .text_sm()
                .text_color(palette.faint)
                .child(folded_count(count)),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words under a card and what Enter says when it makes no move.
    #[test]
    fn the_card_s_keys_and_refusals_are_said_in_plain_words() {
        let fitting = "production-approval · people · 3d";
        assert_eq!(waiting_sub(fitting), fitting);
        let long = format!("{fitting} · {}", "group".repeat(8));
        let shortened = waiting_sub(&long);
        assert_eq!(shortened.chars().count(), WAITING_SUB + 1);
        assert!(shortened.ends_with('…'));
        assert_eq!(waiting_keys(0), "");
        assert_eq!(waiting_keys(1), "i note · enter makes the move");
        assert_eq!(waiting_keys(3), "1-3 pick a move · i note · enter makes it");
        assert_eq!(waiting_keys_folded(), "enter shows the moves · i note");
        assert_eq!(MOVE_TO, "Move to ...");
        assert_eq!(folded_count(12), "12 moves ▸");
        assert_eq!(pick_a_move(3), "pick a move first: 1-3, or the arrows");
        assert_eq!(
            note_wanted("changes: send back to shape (note required)"),
            "changes: send back to shape (note required) needs a note: say what should change in the line (i)"
        );
        assert_eq!(
            not_makeable("close it", "close"),
            "close it cannot be made from here: footplate knows no move of kind close"
        );
    }
}
