//! The text of a message, which can be selected with the mouse: gpui draws
//! text and wraps it, and keeps no selection. The talk has one selection at
//! a time, so it is kept beside the texts and not in each of them; a text
//! draws its part of it and changes it with the mouse. A link in the text is
//! drawn as one, and a click on it opens it, as a click on an ask or bead
//! id the model resolved opens the card or the bead.
//!
//! The text is drawn as the blocks its light Markdown makes of it, one
//! under the other and without their marks. What is selected is still a
//! stretch of the text as it was written: a copy has the marks.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Range;

use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;

use gpui::{
    AnyTooltip, AnyView, App, AppContext, Bounds, ClipboardItem, Context, CursorStyle,
    DispatchPhase, Display, Element, ElementId, Entity, FlexDirection, FontStyle, FontWeight,
    GlobalElementId, Hitbox, HitboxBehavior, InspectorElementId, IntoElement, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, Point,
    Render, SharedString, Style, Styled, StyledText, TextAlign, TextLayout, TextRun, TextStyle,
    TextStyleRefinement, UnderlineStyle, Window, div, fill, point, px, rems, size,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::input::next_boundary;
use crate::markdown::{self, Align, Block, Cell, Kind, Marks, NamedLink};
use crate::theme::Palette;

/// What is selected in the talk, if anything is.
#[derive(Default)]
pub struct Selection {
    at: Option<Selected>,
    /// The block whose copy control reads "copied": the last block copied,
    /// by text key and block, until it reads "copy" again.
    copied: Option<(u64, usize)>,
    /// Which wait for "copied" to end is the current one: an older timer
    /// ends nothing.
    copied_generation: u64,
    /// Where each text's copy controls were last drawn, by text key and
    /// block: clicks there copy and select nothing.
    chips: Vec<(u64, Chips)>,
    /// The block a screenshot has the pointer over, by text key and block:
    /// its copy control shows as it does for a pointer there. A debug
    /// build's alone.
    #[cfg(debug_assertions)]
    posed_hover: Option<(u64, usize)>,
}

/// Where the copy controls of one text were last drawn, by block.
type Chips = Vec<(usize, Bounds<Pixels>)>;

/// What a press was on that a release on the same opens: a link, or an
/// ask or bead id.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Clicked {
    Link(SharedString),
    Ref(RefTarget),
}

/// What a press was on: the place in the source, the link or reference
/// under the pointer, and a named link's shape whole where the press
/// was on one's words.
struct Press {
    place: Range<usize>,
    clicked: Option<Clicked>,
    shape: Option<Range<usize>>,
}

struct Selected {
    /// The text it is in, and that text as it was when it was pressed on.
    key: u64,
    text: SharedString,
    /// What the press was on: a word, or a place. A place is empty, or the
    /// marks that stand there and are not drawn. A drag selects from it.
    anchor: Range<usize>,
    word: bool,
    range: Range<usize>,
    /// What the press was on that a release on the same opens, until the
    /// pointer selects something: a drag is no click.
    clicked: Option<Clicked>,
    /// The button is still down.
    selecting: bool,
}

impl Selection {
    /// The selected text, with the line breaks it has.
    pub fn text(&self) -> Option<&str> {
        let at = self.at.as_ref().filter(|at| !at.range.is_empty())?;
        Some(&at.text[at.range.clone()])
    }

    /// Nothing is selected any more.
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        if self.at.take().is_some() {
            cx.notify();
        }
    }

    /// A posed selection, for the screenshots: `range` of the text `key`
    /// stands selected, as a drag over it would. A debug build's alone.
    #[cfg(debug_assertions)]
    pub fn pose(
        &mut self,
        key: u64,
        text: SharedString,
        range: Range<usize>,
        cx: &mut Context<Self>,
    ) {
        self.at = Some(Selected {
            key,
            text,
            anchor: range.clone(),
            word: false,
            range,
            clicked: None,
            selecting: false,
        });
        cx.notify();
    }

    /// A posed pointer, for the screenshots: over the block `block` of the
    /// text `key`, whose copy control shows as it does for a pointer
    /// there. A debug build's alone.
    #[cfg(debug_assertions)]
    pub fn pose_hover(&mut self, key: u64, block: usize, cx: &mut Context<Self>) {
        self.posed_hover = Some((key, block));
        cx.notify();
    }

    /// What of the text `key` is selected.
    fn range_in(&self, key: u64, text: &str) -> Option<Range<usize>> {
        let at = self.at.as_ref()?;
        (at.key == key && at.text.as_ref() == text && !at.range.is_empty())
            .then(|| at.range.clone())
    }

    fn selecting(&self, key: u64) -> bool {
        self.at
            .as_ref()
            .is_some_and(|at| at.key == key && at.selecting)
    }

    /// A press on the text `key` at `at`: what was selected is dropped,
    /// wherever it was. A second press selects the word there, or the
    /// named link's shape whole where it was on one's words.
    fn press(
        &mut self,
        key: u64,
        text: SharedString,
        at: Press,
        clicks: usize,
        cx: &mut Context<Self>,
    ) {
        let word = clicks == 2;
        // The word after the marks that stand there.
        let anchor = if word {
            at.shape.unwrap_or_else(|| word_at(&text, at.place.end))
        } else {
            at.place
        };
        self.at = Some(Selected {
            key,
            text,
            // The first press of the two has opened it already.
            clicked: at.clicked.filter(|_| clicks == 1),
            range: anchor.start..if word { anchor.end } else { anchor.start },
            anchor,
            word,
            selecting: true,
        });
        cx.notify();
    }

    /// The pointer is at `place` with the button down. What is selected
    /// goes from where the press was to there, and has the marks between
    /// the two and none at its ends: bold words selected alone are copied
    /// as the words, and with what is around them as they were written.
    fn drag(&mut self, place: Range<usize>, cx: &mut Context<Self>) {
        let Some(at) = self.at.as_mut() else {
            return;
        };
        let (from, to) = if at.word {
            (at.anchor.start, at.anchor.end)
        } else {
            (at.anchor.end, at.anchor.start)
        };
        let range = if place.start >= at.anchor.end {
            from..place.start
        } else if place.end <= at.anchor.start {
            place.end..to
        } else if at.word {
            at.anchor.clone()
        } else {
            at.anchor.start..at.anchor.start
        };
        if at.range != range {
            at.range = range;
            // A drag selects, also when it comes back to where it began.
            at.clicked = None;
            cx.notify();
        }
    }

    /// The button came up: what was pressed on, if nothing was dragged
    /// since.
    fn release(&mut self) -> Option<Clicked> {
        let at = self.at.as_mut()?;
        at.selecting = false;
        at.clicked.take()
    }

    /// A press that was not on the text `key`: what is selected in it is
    /// dropped.
    fn pressed_outside(&mut self, key: u64, cx: &mut Context<Self>) {
        if self.at.as_ref().is_some_and(|at| at.key == key) {
            self.clear(cx);
        }
    }

    /// The block of the text `key` whose copy control reads "copied", if
    /// one still does.
    #[cfg(test)]
    pub fn copied_in(&self, key: u64) -> Option<usize> {
        self.copied
            .filter(|(at, _)| *at == key)
            .map(|(_, block)| block)
    }

    /// How many copy controls are drawn, in every text.
    #[cfg(test)]
    pub fn chip_count(&self) -> usize {
        self.chips.iter().map(|(_, chips)| chips.len()).sum()
    }

    /// Where the copy controls of the text `key` were last drawn, by
    /// block.
    #[cfg(test)]
    pub fn chips_in(&self, key: u64) -> Chips {
        self.chips
            .iter()
            .find(|(at, _)| *at == key)
            .map(|(_, chips)| chips.clone())
            .unwrap_or_default()
    }

    /// The copy control of the block `block` of the text `key` was
    /// clicked: it reads "copied" for a short while, then "copy" again.
    fn copied_block(&mut self, key: u64, block: usize, cx: &mut Context<Self>) {
        self.copied_generation += 1;
        let generation = self.copied_generation;
        self.copied = Some((key, block));
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            this.update(cx, |selection, cx| {
                if selection.copied_generation == generation {
                    selection.copied = None;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// The copy controls of the text `key` stand at `chips`: clicks there
    /// copy. Noted while painting, so it notifies nothing.
    fn note_chips(&mut self, key: u64, chips: Chips) {
        match self.chips.iter_mut().find(|(at, _)| *at == key) {
            Some((_, at)) => *at = chips,
            None => self.chips.push((key, chips)),
        }
    }

    /// The text and block whose copy control `position` is on, if any.
    fn chip_at(&self, position: Point<Pixels>) -> Option<(u64, usize)> {
        self.chips.iter().find_map(|(key, chips)| {
            chips
                .iter()
                .find(|(_, chip)| chip.contains(&position))
                .map(|(block, _)| (*key, *block))
        })
    }
}

/// The word of `text` that `index` is in.
pub fn word_at(text: &str, index: usize) -> Range<usize> {
    text.split_word_bound_indices()
        .map(|(start, word)| start..start + word.len())
        .find(|word| word.contains(&index))
        .unwrap_or(index..index)
}

/// What ends a sentence or closes a bracket after a link, and is no part of
/// it.
const AFTER_A_LINK: [char; 11] = ['.', ',', ';', ':', '!', '?', ')', ']', '>', '"', '\''];

/// The links in `text`: each starts with `http://` or `https://` and goes to
/// the next whitespace, without what closes the sentence or the bracket it
/// stands in. Nothing else is a link, so only these two are ever opened.
pub fn links(text: &str) -> Vec<Range<usize>> {
    let mut links = Vec::new();
    let mut from = 0;
    while let Some(start) = text[from..].find("http").map(|at| from + at) {
        let rest = &text[start..];
        let Some(scheme) = ["http://", "https://"]
            .into_iter()
            .find(|scheme| rest.starts_with(scheme))
        else {
            from = start + "http".len();
            continue;
        };
        let run = rest.find(char::is_whitespace).unwrap_or(rest.len());
        let link = rest[..run].trim_end_matches(AFTER_A_LINK);
        // The scheme alone names nothing.
        if link.len() > scheme.len() {
            links.push(start..start + link.len());
        }
        from = start + run;
    }
    links
}

/// A link as it is drawn: `from` is where the full link stands in the
/// block's text, or a named link's words, `shown` where its form stands
/// in what gpui lays out, and `url` what a click opens. A long link's
/// form is its short one, and the tooltip names it whole; a short link
/// draws as written, and a named link draws its words with its host
/// beside them.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ShownLink {
    from: Range<usize>,
    shown: Range<usize>,
    url: SharedString,
    /// Whether the link draws in its short form: only then does the
    /// tooltip stand over it, since a short link already shows its
    /// whole URL, and only then do its bounds round out to it. A named
    /// link always does: the tooltip names the address its words hide.
    short: bool,
    /// Where ` · host` stands in what is drawn, for a named link: none
    /// for a bare one, whose short form is the link whole.
    host: Option<Range<usize>>,
}

/// Between a named link's words and its host.
const HOST_SEP: &str = " · ";

/// What of a link draws in the link dress: a bare link's short form
/// whole, or a named link's words without the host beside them.
fn dressed(link: &ShownLink) -> Range<usize> {
    link.shown.start..link.host.as_ref().map_or(link.shown.end, |host| host.start)
}

/// The link under the character `at`: a named link's words are active,
/// its faint host is only a label beside them.
fn link_at(links: &[ShownLink], at: usize) -> Option<&ShownLink> {
    links.iter().find(|link| dressed(link).contains(&at))
}

/// What gpui lays out for the words `text`, whose bare links are `links`
/// and whose named links are `named`: the text with every long link in
/// its short form and every named link's host beside its words, and
/// each link's spans in both. The spans stand in order, and never
/// overlap: a bare link over a named link's words is none, since the
/// words open whole.
fn shrink(text: &str, links: &[Range<usize>], named: &[NamedLink]) -> (String, Vec<ShownLink>) {
    let links: Vec<Range<usize>> = links
        .iter()
        .filter(|link| {
            !named
                .iter()
                .any(|named| link.start < named.shown.end && named.shown.start < link.end)
        })
        .cloned()
        .collect();
    enum Cut<'a> {
        Bare(&'a Range<usize>),
        Named(&'a NamedLink),
    }
    let mut cuts: Vec<(usize, Cut)> = links
        .iter()
        .map(|link| (link.start, Cut::Bare(link)))
        .chain(
            named
                .iter()
                .map(|link| (link.shown.start, Cut::Named(link))),
        )
        .collect();
    cuts.sort_by_key(|(at, _)| *at);
    let mut display = String::with_capacity(text.len());
    let mut shown = Vec::with_capacity(cuts.len());
    let mut at = 0;
    for (_, cut) in cuts {
        match cut {
            Cut::Bare(link) => {
                display.push_str(&text[at..link.start]);
                let url = &text[link.clone()];
                let short = markdown::short_link(url);
                let start = display.len();
                display.push_str(&short);
                shown.push(ShownLink {
                    from: link.clone(),
                    shown: start..display.len(),
                    short: short != url,
                    url: url.to_string().into(),
                    host: None,
                });
                at = link.end;
            }
            Cut::Named(link) => {
                display.push_str(&text[at..link.shown.start]);
                let start = display.len();
                display.push_str(&text[link.shown.clone()]);
                let host_at = display.len();
                display.push_str(HOST_SEP);
                display.push_str(&markdown::link_host(&link.url));
                shown.push(ShownLink {
                    from: link.shown.clone(),
                    shown: start..display.len(),
                    url: link.url.clone().into(),
                    short: true,
                    host: Some(host_at..display.len()),
                });
                at = link.shown.end;
            }
        }
    }
    display.push_str(&text[at..]);
    (display, shown)
}

/// Where the display boundary `at` stands in the block's text. Inside
/// a shortened link at its start, and at its end where the boundary is
/// the link's end, so that selecting the short form selects the whole
/// link; a named link's words with its host stand for the words the
/// same way. Everywhere else the shift the links before it made.
fn to_src(links: &[ShownLink], at: usize) -> usize {
    for link in links {
        // Ahead of the link: the shift the links before it made.
        if at <= link.shown.start {
            return (at as i64 + link.from.start as i64 - link.shown.start as i64) as usize;
        }
        // In a shortened link at its start; in one that draws whole the
        // same character, shifted past the links before it.
        if at < link.shown.end {
            if link.short {
                return link.from.start;
            }
            return (at as i64 + link.from.start as i64 - link.shown.start as i64) as usize;
        }
        // At the link's end on its end.
        if at == link.shown.end {
            return link.from.end;
        }
    }
    match links.last() {
        Some(link) => (at as i64 + link.from.end as i64 - link.shown.end as i64) as usize,
        None => at,
    }
}

/// Where the block-text range stands in what is drawn. A bound inside
/// a shortened link rounds out to the link's short form, or to a named
/// link's words without the host beside them, so that what is selected
/// of a link highlights it whole. An empty range stands where it
/// stands, shifted past the links before it. References never stand
/// inside a link, so theirs only shift.
fn to_display(links: &[ShownLink], range: Range<usize>) -> Range<usize> {
    // The shift before the link `at` stands ahead of: what the
    // shorter links before it took, or what the hosts beside named
    // links added.
    let shift = |at: usize| {
        links.iter().find(|link| at < link.from.end).map_or_else(
            || {
                links
                    .last()
                    .map_or(0, |link| link.from.end as i64 - link.shown.end as i64)
            },
            |link| link.from.start as i64 - link.shown.start as i64,
        )
    };
    if range.is_empty() {
        let shifted = (range.start as i64 - shift(range.start)) as usize;
        return shifted..shifted;
    }
    let start = match links
        .iter()
        .find(|link| link.short && link.from.start <= range.start && range.start < link.from.end)
    {
        Some(link) => link.shown.start,
        None => (range.start as i64 - shift(range.start)) as usize,
    };
    let end = match links
        .iter()
        .find(|link| link.short && link.from.start < range.end && range.end <= link.from.end)
    {
        Some(link) => dressed(link).end,
        None => (range.end as i64 - shift(range.end)) as usize,
    };
    start..end
}

/// What opening a resolved reference shows: an ask's card, or a bead
/// whole.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefTarget {
    /// An ask's card: an open one opens in Decide, a dealt-with one where
    /// it stands.
    Ask { bead: String, ask: u64, open: bool },
    /// A bead whole, as the timetable shows it.
    Bead { bead: String },
}

/// A resolved reference in one of a text's blocks: where it stands in the
/// block's drawn text, and what opening it shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefSpan {
    /// The block it stands in, as `markdown::blocks` numbers them.
    pub block: usize,
    /// The table cell it stands in, row 0 being the head: none elsewhere.
    /// The range is in the cell's words where one stands, else the block's.
    pub cell: Option<(usize, usize)>,
    pub range: Range<usize>,
    pub target: RefTarget,
}

/// What a click on a resolved reference does: the window opens the card
/// or the bead.
pub type RefClick = Rc<dyn Fn(RefTarget, &mut Window, &mut App)>;

/// An ask or bead id named in drawn text, before the model resolves it
/// against what the yard said.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ref {
    /// Where it stands, a qualified ask's bead included.
    pub range: Range<usize>,
    pub kind: RefKind,
}

/// What the text names: an ask's number, with its bead where the text
/// says which bead's, or a bead id alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefKind {
    /// `ask 6926`: the number alone.
    Ask(u64),
    /// `ask 18 on yd-crm`: the number with its bead, and where the bead
    /// stands, so the bead alone still opens where the ask is none the
    /// yard said.
    AskOn {
        ask: u64,
        bead: String,
        bead_range: Range<usize>,
    },
    /// `yardr-e9f0`: a bead id alone.
    Bead(String),
}

/// Whether the byte may stand inside a word an ask reference abuts: the
/// ask in `re-ask 5` is none, and neither is one in `asked 5`.
fn word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

/// Whether the byte may stand inside a bead id: an id's own characters
/// with the `@` of another yard's.
fn bead_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'@')
}

/// A bead id at `start`: its own characters with one `@peer` at most,
/// without what closes the sentence after it. None where no bead id
/// starts here: one has a dash, and neither it nor its peer is empty.
fn bead_token(text: &str, start: usize) -> Option<(String, usize)> {
    if !text
        .as_bytes()
        .get(start)
        .is_some_and(|b| b.is_ascii_alphanumeric())
    {
        return None;
    }
    let mut end = start;
    while end < text.len() && bead_byte(text.as_bytes()[end]) {
        end += 1;
    }
    let token = text[start..end].trim_end_matches(['.', '-', '@']);
    if token.is_empty() {
        return None;
    }
    let end = start + token.len();
    let (head, peer) = match token.split_once('@') {
        None => (token, None),
        Some((head, peer)) => (head, Some(peer)),
    };
    if peer.is_some_and(|peer| peer.is_empty() || peer.contains('@')) {
        return None;
    }
    if !head.contains('-') || head.starts_with('-') || head.starts_with('.') {
        return None;
    }
    Some((token.to_string(), end))
}

/// An ask reference at `start`: `ask` in any case, then whitespace, then
/// the number, then `on` with a bead id where one follows. None where no
/// reference starts here.
fn ask_at(text: &str, start: usize) -> Option<Ref> {
    let bytes = text.as_bytes();
    if start > 0 && word_byte(bytes[start - 1]) {
        return None;
    }
    if !text[start..]
        .get(..3)
        .is_some_and(|w| w.eq_ignore_ascii_case("ask"))
    {
        return None;
    }
    let mut at = start + 3;
    let spaces = bytes[at..]
        .iter()
        .take_while(|b| b.is_ascii_whitespace())
        .count();
    if spaces == 0 {
        return None;
    }
    at += spaces;
    let digits = bytes[at..]
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();
    if digits == 0 {
        return None;
    }
    if bytes
        .get(at + digits)
        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
    {
        return None;
    }
    let ask: u64 = text[at..at + digits].parse().ok()?;
    let end = at + digits;
    // `on` with a bead id where one follows; else the number alone.
    let mut after = end;
    after += bytes[after..]
        .iter()
        .take_while(|b| b.is_ascii_whitespace())
        .count();
    if text[after..]
        .get(..2)
        .is_some_and(|w| w.eq_ignore_ascii_case("on"))
        && bytes
            .get(after + 2)
            .is_some_and(|b| b.is_ascii_whitespace())
    {
        after += 2;
        after += bytes[after..]
            .iter()
            .take_while(|b| b.is_ascii_whitespace())
            .count();
        if let Some((bead, bead_end)) = bead_token(text, after) {
            return Some(Ref {
                range: start..bead_end,
                kind: RefKind::AskOn {
                    ask,
                    bead,
                    bead_range: after..bead_end,
                },
            });
        }
    }
    Some(Ref {
        range: start..end,
        kind: RefKind::Ask(ask),
    })
}

/// Whether `range` stands inside one of `ranges` at any character.
fn overlaps(ranges: &[Range<usize>], range: &Range<usize>) -> bool {
    ranges
        .iter()
        .any(|other| range.start < other.end && other.start < range.end)
}

/// The asks and bead ids `text` names: `ask 6926`, `ask 18 on yd-crm`,
/// and bead ids alone, each where they stand. A qualified ask's bead is
/// no bead id of its own, and a link's words are none of these: the link
/// opens whole.
pub fn refs(text: &str) -> Vec<Ref> {
    let links = links(text);
    // Past the character at `at`: every match ends on a boundary, and a
    // step past a miss must too.
    let step = |at: usize| text[at..].chars().next().map_or(1, |c| c.len_utf8());
    let mut out = Vec::new();
    let mut at = 0;
    while at + 3 <= text.len() {
        match ask_at(text, at) {
            Some(found) if !overlaps(&links, &found.range) => {
                at = found.range.end;
                out.push(found);
            }
            _ => at += step(at),
        }
    }
    let asked: Vec<Range<usize>> = out.iter().map(|found| found.range.clone()).collect();
    let mut at = 0;
    while at < text.len() {
        let starts = at == 0 || !bead_byte(text.as_bytes()[at - 1]);
        match starts.then(|| bead_token(text, at)).flatten() {
            Some((bead, end)) => {
                let range = at..end;
                if !overlaps(&links, &range) && !overlaps(&asked, &range) {
                    out.push(Ref {
                        range,
                        kind: RefKind::Bead(bead),
                    });
                }
                at = end;
            }
            None => at += step(at),
        }
    }
    out.sort_by_key(|found| found.range.start);
    out
}

/// The asks and bead ids a block names, each with the table cell it
/// stands in, row 0 being the head: outside any named link's words,
/// which open whole.
pub fn block_refs(block: &Block) -> Vec<(Option<(usize, usize)>, Ref)> {
    match &block.kind {
        Kind::Table { head, rows, .. } => {
            let mut out = Vec::new();
            for (row, cells) in std::iter::once(head).chain(rows).enumerate() {
                for (col, cell) in cells.iter().enumerate() {
                    let skip: Vec<Range<usize>> =
                        cell.links.iter().map(|link| link.shown.clone()).collect();
                    out.extend(
                        refs(&cell.text)
                            .into_iter()
                            .filter(|found| !overlaps(&skip, &found.range))
                            .map(|found| (Some((row, col)), found)),
                    );
                }
            }
            out
        }
        _ => {
            let skip: Vec<Range<usize>> =
                block.links.iter().map(|link| link.shown.clone()).collect();
            refs(&block.text)
                .into_iter()
                .filter(|found| !overlaps(&skip, &found.range))
                .map(|found| (None, found))
                .collect()
        }
    }
}

/// Where in `text` a pointer at `position` is: the boundary between two
/// characters that is closest to it, as in the typed line. Above the text
/// that is its start, below it its end.
fn index_at(layout: &TextLayout, text: &str, position: Point<Pixels>) -> usize {
    let (index, under) = match layout.index_for_position(position) {
        Ok(index) => (index, true),
        Err(index) => (index, false),
    };
    let mut index = index.min(text.len());
    while !text.is_char_boundary(index) {
        index -= 1;
    }
    // gpui says which character the pointer is over: past its middle, the
    // boundary after it is the closer one.
    if under {
        let next = next_boundary(text, index);
        if let (Some(here), Some(there)) = (
            layout.position_for_index(index),
            layout.position_for_index(next),
        ) && here.y == there.y
            && position.x > (here.x + there.x) / 2.
        {
            return next;
        }
    }
    index
}

/// A block of a text, as it is drawn.
struct Shown {
    block: Block,
    /// What gpui lays out: the block's text with every long link in its
    /// short form.
    display: SharedString,
    /// The links in both, each with the URL a click opens.
    links: Vec<ShownLink>,
    /// The resolved references in what is drawn, each with what opening
    /// it shows.
    refs: Vec<(Range<usize>, RefTarget)>,
    /// A table's cells as they are drawn, row 0 being the head: empty
    /// where the block is no table.
    cells: Vec<Vec<CellShown>>,
    /// An exact copy payload for a code block whose displayed form was
    /// sanitized before it reached the markdown reader.
    copy_override: Option<SharedString>,
}

/// One table cell as it is drawn.
struct CellShown {
    /// What gpui lays out: the cell's words with every long link in its
    /// short form and every named link's host beside its words.
    display: SharedString,
    /// The links in both, each with the URL a click opens.
    links: Vec<ShownLink>,
    /// The resolved references in what is drawn, each with what opening
    /// it shows.
    refs: Vec<(Range<usize>, RefTarget)>,
}

/// The table cell at `row` and `col` of `block`, row 0 being the head.
fn table_cell(block: &Block, row: usize, col: usize) -> Option<&Cell> {
    match &block.kind {
        Kind::Table { head, .. } if row == 0 => head.get(col),
        Kind::Table { rows, .. } if row > 0 => rows.get(row - 1)?.get(col),
        _ => None,
    }
}

/// The text `text` of the talk, which `selection` may be in. `key` tells it
/// from the other texts.
pub fn text(key: u64, text: impl Into<SharedString>, selection: &Entity<Selection>) -> Text {
    let text = text.into();
    let blocks = markdown::blocks(&text);
    text_inner(key, text, blocks, None, Vec::new(), None, selection)
}

/// Like [`text`], but read as no Markdown: no fence, list, heading
/// or mark in it draws as one. An asker's marked words, which keep no
/// block.
pub fn verbatim(key: u64, text: impl Into<SharedString>, selection: &Entity<Selection>) -> Text {
    let text = text.into();
    let blocks = vec![markdown::verbatim(&text)];
    text_inner(key, text, blocks, None, Vec::new(), None, selection)
}

/// Like [`text`], but the first code block copies `copy_text` exactly. The
/// displayed text may have had control characters removed before it was
/// fenced; copying still returns the bytes the caller was asked to show.
pub fn text_with_code_copy(
    key: u64,
    text: impl Into<SharedString>,
    copy_text: impl Into<SharedString>,
    selection: &Entity<Selection>,
) -> Text {
    let text = text.into();
    let blocks = markdown::blocks(&text);
    text_inner(
        key,
        text,
        blocks,
        Some(copy_text.into()),
        Vec::new(),
        None,
        selection,
    )
}

/// Like [`text`], with resolved references: each stands in its block as a
/// chip, and a click on one calls `on_ref` with what it opens.
pub fn text_with_refs(
    key: u64,
    text: impl Into<SharedString>,
    refs: Vec<RefSpan>,
    on_ref: RefClick,
    selection: &Entity<Selection>,
) -> Text {
    let text = text.into();
    let blocks = markdown::blocks(&text);
    text_inner(key, text, blocks, None, refs, Some(on_ref), selection)
}

fn text_inner(
    key: u64,
    text: SharedString,
    blocks: Vec<Block>,
    mut code_copy: Option<SharedString>,
    refs: Vec<RefSpan>,
    on_ref: Option<RefClick>,
    selection: &Entity<Selection>,
) -> Text {
    let blocks = blocks
        .into_iter()
        .enumerate()
        .map(|(ix, block)| {
            let cells = match &block.kind {
                Kind::Table { head, rows, .. } => std::iter::once(head)
                    .chain(rows)
                    .enumerate()
                    .map(|(row, cells)| {
                        cells
                            .iter()
                            .enumerate()
                            .map(|(col, cell)| {
                                let (display, links) =
                                    shrink(&cell.text, &links(&cell.text), &cell.links);
                                let refs = refs
                                    .iter()
                                    .filter(|span| {
                                        span.block == ix && span.cell == Some((row, col))
                                    })
                                    .map(|span| {
                                        (
                                            to_display(&links, span.range.clone()),
                                            span.target.clone(),
                                        )
                                    })
                                    .collect();
                                CellShown {
                                    display: display.into(),
                                    links,
                                    refs,
                                }
                            })
                            .collect()
                    })
                    .collect(),
                _ => Vec::new(),
            };
            let (display, links) = shrink(&block.text, &links(&block.text), &block.links);
            Shown {
                copy_override: if block.kind == Kind::Code {
                    code_copy.take()
                } else {
                    None
                },
                display: display.into(),
                refs: refs
                    .iter()
                    .filter(|span| span.block == ix && span.cell.is_none())
                    .map(|span| (to_display(&links, span.range.clone()), span.target.clone()))
                    .collect(),
                cells,
                links,
                block,
            }
        })
        .collect();
    Text {
        key,
        text,
        blocks,
        on_ref,
        selection: selection.clone(),
        tip: None,
    }
}

/// A key for a text that has no number of its own, from what it is: a
/// mail's is its number in the yard.
pub fn key(of: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    of.hash(&mut hasher);
    hasher.finish()
}

/// How far an item's words stand in from the edge of the text, in rems: the
/// marker is in that space. An item under another stands in twice as far.
pub const ITEM_INDENT: f32 = 1.5;
/// Between a marker and its item's words.
const MARKER_GAP: f32 = 0.4;
/// How far a quote's words stand in from the edge of the text, in rems:
/// the bar is in that space.
const QUOTE_INSET: f32 = 0.75;
/// How wide a quote's bar is, in pixels.
const QUOTE_BAR: f32 = 3.;
/// Around a table cell's words: half a rem at each side, and six pixels
/// above and below, as the design gives them.
const CELL_SIDE: f32 = 0.5;
const CELL_ENDS: f32 = 6.;
/// The widest share of the reading column one table column takes: two
/// fifths.
const TABLE_COLUMN_CAP: f32 = 0.4;
/// Around the lines of a code block, inside its ground. The left is wider:
/// the mark of a line that was wrapped stands there.
const CODE_LEFT: f32 = 1.;
const CODE_RIGHT: f32 = 0.625;
const CODE_ABOVE: f32 = 0.375;
/// Above a block that a blank line stood before, and above one that is a
/// heading or beside code.
const APART: f32 = 0.6;
const NEAR: f32 = 0.3;
/// The mark before the rest of a line of code that did not fit the width:
/// code is wrapped and not scrolled, and the mark says the break is not one
/// of the code's own.
const WRAPPED: &str = "↪";
/// How far the copy control of a code block stands in from the top right
/// of its ground, and how large it is, in rems: room for "copied" at any
/// text size.
const COPY_INSET: f32 = 0.25;
const COPY_WIDTH: f32 = 4.;
const COPY_HEIGHT: f32 = 1.5;
/// How long a copy control reads "copied" after it was clicked.
const COPIED_FOR: Duration = Duration::from_secs(2);

/// The size the window's text is drawn at, in rems: the root sets it and
/// what stands under it inherits it. A view that sizes something for its
/// text while it renders reads it here, not from `window.text_style()`,
/// which answers gpui's 1 rem until the root's element lays out.
pub(crate) const ROOT_TEXT_REM: f32 = 0.875;

/// A monospaced face this machine has, for code: the first of these.
const MONO: [&str; 7] = [
    "Menlo",
    "DejaVu Sans Mono",
    "Liberation Mono",
    "Noto Sans Mono",
    "Ubuntu Mono",
    "Consolas",
    "Courier New",
];

/// The face code is drawn in. Without one of them it is the text's face,
/// and its ground alone tells it from the text. The arrivals board's
/// times are drawn in it too.
pub(crate) fn mono(cx: &App) -> Option<SharedString> {
    static FACE: OnceLock<Option<&'static str>> = OnceLock::new();
    FACE.get_or_init(|| {
        let names = cx.text_system().all_font_names();
        MONO.into_iter()
            .find(|face| names.iter().any(|name| name == face))
    })
    .map(SharedString::new_static)
}

/// The face a block of `kind` is drawn in, where the text's is `base`: a
/// heading is bold and a little larger than the text, the first level the
/// most, and code is in `mono`. All of it goes with the text's size.
fn face(kind: &Kind, base: &TextStyle, mono: Option<&SharedString>, rem: Pixels) -> TextStyle {
    let mut face = base.clone();
    match kind {
        Kind::Heading(level) => {
            let larger = match level {
                1 => 1.25,
                2 => 1.15,
                _ => 1.05,
            };
            face.font_size = (base.font_size.to_pixels(rem) * larger).into();
            face.font_weight = FontWeight::BOLD;
        }
        Kind::Code => {
            if let Some(mono) = mono {
                face.font_family = mono.clone();
            }
        }
        Kind::Text | Kind::Item { .. } | Kind::Table { .. } | Kind::Quote | Kind::Rule => {}
    }
    face
}

/// The face a table cell is drawn in: its block's, semibold where the
/// cell is in the head.
fn cell_face(head: bool, face: &TextStyle) -> TextStyle {
    let mut face = face.clone();
    if head {
        face.font_weight = FontWeight::SEMIBOLD;
    }
    face
}

/// Each column's share of the table's width from its cells' measures:
/// weighed by the widest cell up to two fifths of the column, what
/// remains shared. The shares add to one.
fn columns(natural: &[f32]) -> Vec<f32> {
    let mut shares = vec![0.0; natural.len()];
    let mut open: Vec<usize> = (0..natural.len()).collect();
    let mut remaining = 1.0;
    while !open.is_empty() {
        let total: f32 = open.iter().map(|&i| natural[i]).sum();
        let share = |i: usize| {
            if total > 0.0 {
                remaining * natural[i] / total
            } else {
                remaining / open.len() as f32
            }
        };
        let capped: Vec<usize> = open
            .iter()
            .copied()
            .filter(|&i| share(i) > TABLE_COLUMN_CAP)
            .collect();
        if capped.is_empty() {
            for &i in &open {
                shares[i] = share(i);
            }
            return shares;
        }
        for i in capped {
            shares[i] = TABLE_COLUMN_CAP;
            remaining -= TABLE_COLUMN_CAP;
            open.retain(|&j| j != i);
        }
    }
    // Every column capped: what remains is shared equally.
    if remaining > 0.0 {
        let each = remaining / shares.len() as f32;
        for share in shares.iter_mut() {
            *share += each;
        }
    }
    shares
}

/// What runs are drawn with: the words' face, the palette, and the
/// monospaced face where the machine has one.
struct Draw<'a> {
    face: &'a TextStyle,
    palette: &'a Palette,
    mono: Option<&'a SharedString>,
}

/// The runs words are drawn in: bold, italic and inline code as `marks`
/// say, with a quiet ground under the code; a link as one, in the link
/// blue, a named link's host beside its words in the faint colour with
/// neither underline nor selection ground; a reference as a chip, in
/// the code's face on its ground but in the link blue, with no
/// underline; and `selected` of the words on the selection's ground.
/// `marks` and `selected` stand in the words; `refs` and every link's
/// spans stand in `display`, the long links shortened. A selection may
/// begin or end inside a link or a mark: the runs do not overlap.
fn runs(
    display: &str,
    marks: &[(Range<usize>, Marks)],
    links: &[ShownLink],
    refs: &[Range<usize>],
    selected: Option<Range<usize>>,
    draw: &Draw,
) -> Vec<TextRun> {
    let selected = selected
        .map(|selected| to_display(links, selected))
        .filter(|selected| !selected.is_empty());
    let marks: Vec<(Range<usize>, Marks)> = marks
        .iter()
        .map(|(range, marks)| (to_display(links, range.clone()), *marks))
        .collect();
    let dressed: Vec<Range<usize>> = links.iter().map(dressed).collect();
    let hosts: Vec<Range<usize>> = links.iter().filter_map(|link| link.host.clone()).collect();
    let mut cuts = vec![0, display.len()];
    let marked = marks.iter().map(|(range, _)| range);
    for range in marked
        .chain(&dressed)
        .chain(&hosts)
        .chain(refs)
        .chain(&selected)
    {
        cuts.extend([range.start, range.end]);
    }
    cuts.sort_unstable();
    cuts.dedup();
    let within = |range: &Range<usize>, at: usize| range.start <= at && at < range.end;
    let mut runs = Vec::new();
    for cut in cuts.windows(2) {
        let (start, end) = (cut[0], cut[1]);
        let mut run = draw.face.to_run(end - start);
        if let Some((_, marks)) = marks.iter().find(|(range, _)| within(range, start)) {
            if marks.bold {
                run.font.weight = FontWeight::BOLD;
            }
            if marks.italic {
                run.font.style = FontStyle::Italic;
            }
            if marks.code {
                if let Some(mono) = draw.mono {
                    run.font.family = mono.clone();
                }
                run.background_color = Some(draw.palette.code.into());
            }
        }
        if dressed.iter().any(|link| within(link, start)) {
            run.color = draw.palette.link.into();
            run.underline = Some(UnderlineStyle {
                thickness: px(1.),
                color: Some(draw.palette.link.into()),
                wavy: false,
            });
        }
        if refs.iter().any(|reference| within(reference, start)) {
            if let Some(mono) = draw.mono {
                run.font.family = mono.clone();
            }
            run.color = draw.palette.link.into();
            run.background_color = Some(draw.palette.code.into());
        }
        // A named link's host: the faint colour, with neither the
        // link's underline nor the selection's ground over it.
        let hosted = hosts.iter().any(|host| within(host, start));
        if hosted {
            run.color = draw.palette.faint.into();
            run.underline = None;
        }
        if !hosted && selected.as_ref().is_some_and(|s| within(s, start)) {
            run.background_color = Some(draw.palette.selection.into());
        }
        runs.push(run);
    }
    // An empty text is one empty run, as gpui makes it.
    if runs.is_empty() {
        runs.push(draw.face.to_run(0));
    }
    runs
}

/// The space around `block`, which stands under one of kind `before`: the
/// room above it, an item's indent, and the inside of a code block's
/// ground.
fn frame(block: &Block, before: Option<&Kind>) -> Style {
    let mut style = Style::default();
    if let Some(before) = before {
        // The wide room above a block a blank line stood before, above
        // any heading or table, and above and below any rule.
        let above = if block.apart
            || matches!(
                block.kind,
                Kind::Heading(_) | Kind::Table { .. } | Kind::Rule
            )
            || *before == Kind::Rule
        {
            APART
        } else if block.kind == Kind::Code || *before == Kind::Code {
            NEAR
        } else {
            0.
        };
        style.margin.top = rems(above).into();
    }
    match &block.kind {
        Kind::Code => {
            style.padding.left = rems(CODE_LEFT).into();
            style.padding.right = rems(CODE_RIGHT).into();
            style.padding.top = rems(CODE_ABOVE).into();
            style.padding.bottom = rems(CODE_ABOVE).into();
        }
        Kind::Item { depth, .. } => {
            style.padding.left = rems(ITEM_INDENT * f32::from(depth + 1)).into();
        }
        Kind::Quote => {
            style.padding.left = rems(QUOTE_INSET).into();
        }
        Kind::Rule => {
            // The hairline alone: a pixel tall, with the wide room its
            // margins give it.
            style.size.height = px(1.).into();
        }
        Kind::Text | Kind::Heading(_) | Kind::Table { .. } => {}
    }
    style
}

/// Where the rows begin that a line of the code `text` goes on in, after
/// its first: a line that did not fit the width.
fn wrapped_rows(layout: &TextLayout, text: &str) -> Vec<Point<Pixels>> {
    let mut rows = Vec::new();
    let mut start = 0;
    for line in text.split('\n') {
        if let (Some(at), Some(wrapped)) = (
            layout.position_for_index(start),
            layout.line_layout_for_index(start),
        ) {
            for row in 1..=wrapped.wrap_boundaries.len() {
                rows.push(point(at.x, at.y + layout.line_height() * row as f32));
            }
        }
        start += line.len() + 1;
    }
    rows
}

/// The copy control of a code block: a fixed-size chip at the top right
/// of its ground. Fixed so a click lands where the control was drawn,
/// whatever it reads.
fn copy_chip(frame: Bounds<Pixels>, rem: Pixels) -> Bounds<Pixels> {
    let chip = size(rem * COPY_WIDTH, rem * COPY_HEIGHT);
    Bounds::new(
        point(
            frame.right() - rem * COPY_INSET - chip.width,
            frame.top() + rem * COPY_INSET,
        ),
        chip,
    )
}

/// The blocks of a text where they are drawn: what the mouse goes by.
#[derive(Clone)]
struct Drawn {
    blocks: Rc<[Shown]>,
    /// Each block's words where they are drawn: none where the block is
    /// a table, whose cells stand in `cells`.
    layouts: Rc<[Option<TextLayout>]>,
    /// Where each block's ground is drawn.
    frames: Rc<[Bounds<Pixels>]>,
    /// The copy control of each code block, by block.
    chips: Rc<[(usize, Bounds<Pixels>)]>,
    /// Every table cell where it is drawn, by block, row and column.
    cells: Rc<[CellDrawn]>,
}

/// One table cell where it is drawn: what the mouse goes by.
#[derive(Clone)]
struct CellDrawn {
    block: usize,
    row: usize,
    col: usize,
    layout: TextLayout,
    /// Where the cell's frame is drawn, its room included.
    frame: Bounds<Pixels>,
    align: TextAlign,
}

/// Where the pointer stands in aligned words: gpui paints a right- or
/// centre-aligned line shifted into its width, and finds positions as
/// if it stood at the left, so the shift comes back off for the mouse.
fn unshift(
    layout: &TextLayout,
    align: TextAlign,
    display: &str,
    position: Point<Pixels>,
) -> Point<Pixels> {
    let shift = align_shift(layout, align, display, position).unwrap_or(px(0.));
    point(position.x - shift, position.y)
}

/// How far the line under the pointer is shifted into its width, where
/// the words are right- or centre-aligned: the slack the line leaves,
/// or half of it. None where the line cannot be found.
fn align_shift(
    layout: &TextLayout,
    align: TextAlign,
    display: &str,
    position: Point<Pixels>,
) -> Option<Pixels> {
    if align == TextAlign::Left {
        return Some(px(0.));
    }
    let mut probe = position;
    probe.x = layout.bounds().left();
    let mut ix = layout.index_for_position(probe).unwrap_or_else(|ix| ix);
    ix = ix.min(display.len());
    while !display.is_char_boundary(ix) {
        ix -= 1;
    }
    let wrapped = layout.line_layout_for_index(ix)?;
    let line = &wrapped.unwrapped_layout;
    let start = display[..ix].rfind('\n').map_or(0, |at| at + 1);
    let top = layout.position_for_index(start)?;
    let row = (f32::from(position.y - top.y) / f32::from(layout.line_height()))
        .round()
        .max(0.) as usize;
    let fragments = wrapped.wrap_boundaries.len() + 1;
    let row = row.min(fragments - 1);
    // The fragments' widths from their glyphs: the first glyph of the
    // next fragment stands where this one ends, and the last fragment
    // ends where the line does.
    let first_x = |fragment: usize| {
        if fragment == 0 {
            return Some(px(0.));
        }
        let bound = wrapped.wrap_boundaries.get(fragment - 1)?;
        let run = line.runs.get(bound.run_ix)?;
        if let Some(glyph) = run.glyphs.get(bound.glyph_ix + 1) {
            Some(glyph.position.x)
        } else {
            Some(line.runs.get(bound.run_ix + 1)?.glyphs.first()?.position.x)
        }
    };
    let start_x = first_x(row)?;
    let end_x = if row + 1 < fragments {
        first_x(row + 1)?
    } else {
        line.width
    };
    let slack = layout.bounds().size.width - (end_x - start_x);
    let slack = if slack > px(0.) { slack } else { px(0.) };
    Some(match align {
        TextAlign::Right => slack,
        TextAlign::Center => slack / 2.,
        TextAlign::Left => px(0.),
    })
}

impl Drawn {
    /// The block the pointer at `position` is in. Between two blocks it is
    /// the lower one, whose start is closest, and under the last the last.
    /// A rule's frame is its hairline; its empty text may stand taller,
    /// and must not catch what is under it.
    fn block_at(&self, position: Point<Pixels>) -> Option<usize> {
        let last = self.blocks.len().checked_sub(1)?;
        let under = |ix: usize| {
            let bottom = match self.layouts[ix].as_ref() {
                Some(layout) => {
                    let (text, frame) = (layout.bounds().bottom(), self.frames[ix].bottom());
                    if text < frame { text } else { frame }
                }
                None => self.frames[ix].bottom(),
            };
            position.y < bottom
        };
        Some((0..self.blocks.len()).position(under).unwrap_or(last))
    }

    /// The table cell of the block `block` the pointer is in: the first
    /// row standing under it, else the last, and within it the first
    /// cell standing past it, else the last.
    fn cell_at(&self, block: usize, position: Point<Pixels>) -> Option<(usize, usize)> {
        let mut row = None;
        let mut last = None;
        for cell in self.cells.iter().filter(|cell| cell.block == block) {
            last = Some(cell.row);
            if row.is_none() && position.y < cell.frame.bottom() {
                row = Some(cell.row);
            }
        }
        let row = row.or(last)?;
        let mut col = None;
        let mut last = None;
        for cell in self
            .cells
            .iter()
            .filter(|cell| cell.block == block && cell.row == row)
        {
            last = Some(cell.col);
            if col.is_none() && position.x < cell.frame.right() {
                col = Some(cell.col);
            }
        }
        Some((row, col.or(last)?))
    }

    /// The table cell drawn at `block`, `row` and `col`.
    fn drawn_cell(&self, block: usize, row: usize, col: usize) -> Option<&CellDrawn> {
        self.cells
            .iter()
            .find(|cell| cell.block == block && cell.row == row && cell.col == col)
    }

    /// Where in the text as it was written the pointer is: a place, with
    /// the marks that stand there and are not drawn. A shortened link's
    /// short form stands for the whole link.
    fn place_at(&self, position: Point<Pixels>) -> Range<usize> {
        let Some(ix) = self.block_at(position) else {
            return 0..0;
        };
        let shown = &self.blocks[ix];
        if matches!(shown.block.kind, Kind::Table { .. }) {
            let Some((row, col)) = self.cell_at(ix, position) else {
                return 0..0;
            };
            let (Some(cell), Some(words)) = (
                self.drawn_cell(ix, row, col),
                table_cell(&shown.block, row, col),
            ) else {
                return 0..0;
            };
            let display = &shown.cells[row][col].display;
            let position = unshift(&cell.layout, cell.align, display, position);
            let index = index_at(&cell.layout, display, position);
            return words.source_at(to_src(&shown.cells[row][col].links, index));
        }
        let Some(layout) = self.layouts[ix].as_ref() else {
            return 0..0;
        };
        let index = index_at(layout, &shown.display, position);
        shown.block.source_at(to_src(&shown.links, index))
    }

    /// The full link under the pointer, where it draws shortened:
    /// what the tooltip names. The character under the pointer counts
    /// here, not the closest boundary.
    fn short_link_at(&self, position: Point<Pixels>) -> Option<SharedString> {
        let ix = self.block_at(position)?;
        let shown = &self.blocks[ix];
        if matches!(shown.block.kind, Kind::Table { .. }) {
            let (row, col) = self.cell_at(ix, position)?;
            let cell = self.drawn_cell(ix, row, col)?;
            let display = &shown.cells[row][col].display;
            let position = unshift(&cell.layout, cell.align, display, position);
            let index = cell.layout.index_for_position(position).ok()?;
            return shown.cells[row][col]
                .links
                .iter()
                .find(|link| link.short && link.shown.contains(&index))
                .map(|link| link.url.clone());
        }
        let layout = self.layouts[ix].as_ref()?;
        let index = layout.index_for_position(position).ok()?;
        shown
            .links
            .iter()
            .find(|link| link.short && link.shown.contains(&index))
            .map(|link| link.url.clone())
    }

    /// The named link's shape under the pointer, where it is over one's
    /// words: a second press selects it whole. The character under the
    /// pointer counts here, not the closest boundary.
    fn shape_at(&self, position: Point<Pixels>) -> Option<Range<usize>> {
        let ix = self.block_at(position)?;
        let shown = &self.blocks[ix];
        if matches!(shown.block.kind, Kind::Table { .. }) {
            let (row, col) = self.cell_at(ix, position)?;
            let cell = self.drawn_cell(ix, row, col)?;
            let display = &shown.cells[row][col].display;
            let position = unshift(&cell.layout, cell.align, display, position);
            let index = cell.layout.index_for_position(position).ok()?;
            let link =
                link_at(&shown.cells[row][col].links, index).filter(|link| link.host.is_some())?;
            let words = table_cell(&shown.block, row, col)?;
            return Some(
                words.source_at(link.from.start).start..words.source_at(link.from.end).end,
            );
        }
        let layout = self.layouts[ix].as_ref()?;
        let index = layout.index_for_position(position).ok()?;
        let link = link_at(&shown.links, index).filter(|link| link.host.is_some())?;
        let block = &shown.block;
        Some(block.source_at(link.from.start).start..block.source_at(link.from.end).end)
    }

    /// The link or reference the pointer is over: the character under
    /// it counts here, not the closest boundary.
    fn clicked_at(&self, position: Point<Pixels>) -> Option<Clicked> {
        let ix = self.block_at(position)?;
        let shown = &self.blocks[ix];
        if matches!(shown.block.kind, Kind::Table { .. }) {
            let (row, col) = self.cell_at(ix, position)?;
            let cell = self.drawn_cell(ix, row, col)?;
            let display = &shown.cells[row][col].display;
            let position = unshift(&cell.layout, cell.align, display, position);
            let index = cell.layout.index_for_position(position).ok()?;
            let links = &shown.cells[row][col].links;
            if let Some(link) = link_at(links, index) {
                return Some(Clicked::Link(link.url.clone()));
            }
            return shown.cells[row][col]
                .refs
                .iter()
                .find(|(range, _)| range.contains(&index))
                .map(|(_, target)| Clicked::Ref(target.clone()));
        }
        let layout = self.layouts[ix].as_ref()?;
        let index = layout.index_for_position(position).ok()?;
        if let Some(link) = link_at(&shown.links, index) {
            return Some(Clicked::Link(link.url.clone()));
        }
        shown
            .refs
            .iter()
            .find(|(range, _)| range.contains(&index))
            .map(|(_, target)| Clicked::Ref(target.clone()))
    }

    /// The code block the pointer is over, if it is over one's ground:
    /// its copy control is drawn.
    fn hovered_code(&self, position: Point<Pixels>) -> Option<usize> {
        self.chips
            .iter()
            .find_map(|(ix, _)| self.frames[*ix].contains(&position).then_some(*ix))
    }

    /// The code block whose copy control the pointer is on, if any.
    fn copy_at(&self, position: Point<Pixels>) -> Option<usize> {
        self.chips
            .iter()
            .find(|(_, chip)| chip.contains(&position))
            .map(|(ix, _)| *ix)
    }
}

/// The full link under the pointer, where it draws shortened: one
/// line in the tooltip's own style, wrapped where it runs longer than
/// the tooltip may.
struct LinkTip(SharedString);

impl Render for LinkTip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = Palette::now(window, cx);
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .border_1()
            .border_color(palette.line)
            .bg(palette.ground)
            .text_sm()
            .text_color(palette.text)
            .max_w(rems(30.))
            .child(self.0.clone())
    }
}

pub struct Text {
    key: u64,
    /// The text as it was written: what is selected is a stretch of this.
    text: SharedString,
    blocks: Rc<[Shown]>,
    on_ref: Option<RefClick>,
    selection: Entity<Selection>,
    /// The tooltip of the link under the pointer, by its URL: made once,
    /// and shown while the pointer stays over that link.
    tip: Option<(SharedString, AnyView)>,
}

/// A block of a text where it is laid out.
pub struct Laid {
    /// The space around the block, and where it is once that is known.
    frame: LayoutId,
    bounds: Bounds<Pixels>,
    text: LaidText,
}

/// A block's words where they are laid out: one text, or a table's rows
/// of cell texts.
enum LaidText {
    One {
        /// gpui's own text: it shapes, wraps and draws.
        styled: StyledText,
        text: LayoutId,
    },
    Table {
        rows: Vec<RowLaid>,
    },
}

/// One table row where it is laid out.
struct RowLaid {
    frame: LayoutId,
    bounds: Bounds<Pixels>,
    cells: Vec<CellLaid>,
}

/// One table cell where it is laid out.
struct CellLaid {
    /// gpui's own text: it shapes, wraps and draws.
    styled: StyledText,
    text: LayoutId,
    frame: LayoutId,
    /// Where the cell's frame is drawn, its room included.
    frame_bounds: Bounds<Pixels>,
    align: TextAlign,
}

impl IntoElement for Text {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Text {
    /// The blocks where they are drawn: what the mouse goes by.
    fn drawn(&self, laid: &[Laid], rem: Pixels) -> Drawn {
        Drawn {
            blocks: self.blocks.clone(),
            layouts: laid
                .iter()
                .map(|laid| match &laid.text {
                    LaidText::One { styled, .. } => Some(styled.layout().clone()),
                    LaidText::Table { .. } => None,
                })
                .collect(),
            frames: laid.iter().map(|laid| laid.bounds).collect(),
            chips: laid
                .iter()
                .zip(self.blocks.iter())
                .enumerate()
                .filter(|(_, (_, shown))| shown.block.kind == Kind::Code)
                .map(|(ix, (laid, _))| (ix, copy_chip(laid.bounds, rem)))
                .collect(),
            cells: laid
                .iter()
                .enumerate()
                .flat_map(|(ix, laid)| match &laid.text {
                    LaidText::Table { rows } => rows
                        .iter()
                        .enumerate()
                        .flat_map(|(row, laid)| {
                            laid.cells
                                .iter()
                                .enumerate()
                                .map(move |(col, cell)| CellDrawn {
                                    block: ix,
                                    row,
                                    col,
                                    layout: cell.styled.layout().clone(),
                                    frame: cell.frame_bounds,
                                    align: cell.align,
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                    LaidText::One { .. } => Vec::new(),
                })
                .collect(),
        }
    }
}

impl Element for Text {
    type RequestLayoutState = Vec<Laid>;
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let palette = Palette::now(window, cx);
        let (base, rem, mono) = (window.text_style(), window.rem_size(), mono(cx));
        let selected = self.selection.read(cx).range_in(self.key, &self.text);
        let mut laid = Vec::with_capacity(self.blocks.len());
        let mut before = None;
        for shown in self.blocks.iter() {
            let block = &shown.block;
            if let Kind::Table { head, rows, align } = &block.kind {
                // Each column weighed by its widest cell's words, shaped
                // as one line, with the cell's room at each side.
                let size = base.font_size.to_pixels(rem);
                let natural: Vec<f32> = (0..align.len())
                    .map(|col| {
                        let mut widest = 0f32;
                        for row in 0..shown.cells.len() {
                            let display = &shown.cells[row][col].display;
                            if display.is_empty() {
                                continue;
                            }
                            let mut run = base.to_run(display.len());
                            if row == 0 {
                                run.font.weight = FontWeight::SEMIBOLD;
                            }
                            let line = window.text_system().shape_line(
                                display.clone(),
                                size,
                                &[run],
                                None,
                            );
                            widest = widest.max(f32::from(line.width));
                        }
                        widest + f32::from(rem * 2. * CELL_SIDE)
                    })
                    .collect();
                let shares = columns(&natural);
                let mut row_frames = Vec::with_capacity(shown.cells.len());
                let mut laid_rows = Vec::with_capacity(shown.cells.len());
                for (row, cells) in std::iter::once(head).chain(rows).enumerate() {
                    let mut cell_frames = Vec::with_capacity(cells.len());
                    let mut laid_cells = Vec::with_capacity(cells.len());
                    for (col, words) in cells.iter().enumerate() {
                        let cell = &shown.cells[row][col];
                        let face =
                            cell_face(row == 0, &face(&block.kind, &base, mono.as_ref(), rem));
                        let refs: Vec<Range<usize>> =
                            cell.refs.iter().map(|(range, _)| range.clone()).collect();
                        let selected = selected.clone().map(|selected| words.shown(selected));
                        let draw = Draw {
                            face: &face,
                            palette: &palette,
                            mono: mono.as_ref(),
                        };
                        let runs = runs(
                            &cell.display,
                            &words.marks,
                            &cell.links,
                            &refs,
                            selected,
                            &draw,
                        );
                        let mut styled = StyledText::new(cell.display.clone()).with_runs(runs);
                        let align = match align[col] {
                            Align::Left => TextAlign::Left,
                            Align::Centre => TextAlign::Center,
                            Align::Right => TextAlign::Right,
                        };
                        // gpui takes the size of the lines from the
                        // window, not from the runs, and paints each
                        // line where its alignment says.
                        let size = TextStyleRefinement {
                            font_size: Some(face.font_size),
                            text_align: Some(align),
                            ..Default::default()
                        };
                        let (text, ()) = window.with_text_style(Some(size), |window| {
                            styled.request_layout(None, None, window, cx)
                        });
                        let mut cell_style = Style::default();
                        cell_style.padding.left = rems(CELL_SIDE).into();
                        cell_style.padding.right = rems(CELL_SIDE).into();
                        cell_style.padding.top = px(CELL_ENDS).into();
                        cell_style.padding.bottom = px(CELL_ENDS).into();
                        cell_style.flex_basis = px(0.).into();
                        cell_style.flex_grow = shares[col];
                        let frame = window.request_layout(cell_style, [text], cx);
                        cell_frames.push(frame);
                        laid_cells.push(CellLaid {
                            styled,
                            text,
                            frame,
                            frame_bounds: Bounds::default(),
                            align,
                        });
                    }
                    let row_style = Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        ..Default::default()
                    };
                    let frame = window.request_layout(row_style, cell_frames, cx);
                    row_frames.push(frame);
                    laid_rows.push(RowLaid {
                        frame,
                        bounds: Bounds::default(),
                        cells: laid_cells,
                    });
                }
                let frame = window.request_layout(frame(block, before), row_frames, cx);
                before = Some(&block.kind);
                laid.push(Laid {
                    frame,
                    bounds: Bounds::default(),
                    text: LaidText::Table { rows: laid_rows },
                });
                continue;
            }
            let face = face(&block.kind, &base, mono.as_ref(), rem);
            let refs: Vec<Range<usize>> =
                shown.refs.iter().map(|(range, _)| range.clone()).collect();
            let selected = selected.clone().map(|selected| shown.block.shown(selected));
            let draw = Draw {
                face: &face,
                palette: &palette,
                mono: mono.as_ref(),
            };
            let runs = runs(
                &shown.display,
                &shown.block.marks,
                &shown.links,
                &refs,
                selected,
                &draw,
            );
            let mut styled = StyledText::new(shown.display.clone()).with_runs(runs);
            // gpui takes the size of the lines from the window, not from
            // the runs.
            let size = TextStyleRefinement {
                font_size: Some(face.font_size),
                ..Default::default()
            };
            let (text, ()) = window.with_text_style(Some(size), |window| {
                styled.request_layout(None, None, window, cx)
            });
            let frame = window.request_layout(frame(block, before), [text], cx);
            before = Some(&block.kind);
            laid.push(Laid {
                frame,
                bounds: Bounds::default(),
                text: LaidText::One { styled, text },
            });
        }
        let frames: Vec<LayoutId> = laid.iter().map(|laid| laid.frame).collect();
        (window.request_layout(Style::default(), frames, cx), laid)
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        laid: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        for laid in laid.iter_mut() {
            match &mut laid.text {
                LaidText::One { styled, text } => {
                    let bounds = window.layout_bounds(*text);
                    styled.prepaint(None, None, bounds, &mut (), window, cx);
                }
                LaidText::Table { rows } => {
                    for row in rows.iter_mut() {
                        row.bounds = window.layout_bounds(row.frame);
                        for cell in row.cells.iter_mut() {
                            let bounds = window.layout_bounds(cell.text);
                            cell.styled
                                .prepaint(None, None, bounds, &mut (), window, cx);
                            cell.frame_bounds = window.layout_bounds(cell.frame);
                        }
                    }
                }
            }
            laid.bounds = window.layout_bounds(laid.frame);
        }
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        // The full link under the pointer, where it draws shortened.
        // Asked here and not when painting: the window takes tooltips
        // in prepaint alone.
        let drawn = self.drawn(laid, window.rem_size());
        let selecting = self.selection.read(cx).selecting(self.key);
        if !selecting
            && let Some(url) = hitbox
                .is_hovered(window)
                .then(|| drawn.short_link_at(window.mouse_position()))
                .flatten()
        {
            let view = match &self.tip {
                Some((at, view)) if at == &url => view.clone(),
                _ => {
                    let view: AnyView = cx.new(|_| LinkTip(url.clone())).into();
                    self.tip = Some((url.clone(), view.clone()));
                    view
                }
            };
            let (drawn, hitbox) = (drawn.clone(), hitbox.clone());
            window.set_tooltip(AnyTooltip {
                view,
                mouse_position: window.mouse_position(),
                check_visible_and_update: Rc::new(move |_, window, _| {
                    hitbox.is_hovered(window)
                        && drawn
                            .short_link_at(window.mouse_position())
                            .is_some_and(|at| at == url)
                }),
            });
        }
        hitbox
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        laid: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let palette = Palette::now(window, cx);
        let (base, rem) = (window.text_style(), window.rem_size());
        let size = base.font_size.to_pixels(rem);
        let key = self.key;
        // What stands beside a block's text and is no part of it: it is
        // not selected, and a copy has what was written there.
        let beside = |text: &'static str, color: gpui::Rgba, window: &mut Window| {
            let mut run = base.to_run(text.len());
            run.color = color.into();
            window
                .text_system()
                .shape_line(text.into(), size, &[run], None)
        };
        let drawn = self.drawn(laid, rem);
        // Where the copy controls stand, for clicks anywhere in the talk:
        // a click on one copies and selects nothing, in any text.
        self.selection.update(cx, |selection, _| {
            selection.note_chips(key, drawn.chips.to_vec())
        });
        for (laid, shown) in laid.iter_mut().zip(self.blocks.iter()) {
            let frame = laid.bounds;
            match &mut laid.text {
                // Hairlines divide a table's rows: under the head and
                // between the body rows, with none after the last.
                LaidText::Table { rows } => {
                    let count = rows.len();
                    for (r, row) in rows.iter_mut().enumerate() {
                        if r + 1 < count {
                            let hairline = Bounds::new(
                                point(frame.left(), row.bounds.bottom()),
                                gpui::size(frame.size.width, px(1.)),
                            );
                            window.paint_quad(fill(hairline, palette.line));
                        }
                        for cell in row.cells.iter_mut() {
                            let size = TextStyleRefinement {
                                text_align: Some(cell.align),
                                ..Default::default()
                            };
                            let bounds = cell.styled.layout().bounds();
                            window.with_text_style(Some(size), |window| {
                                cell.styled
                                    .paint(None, None, bounds, &mut (), &mut (), window, cx);
                            });
                        }
                    }
                }
                LaidText::One { styled, .. } => {
                    let layout = styled.layout().clone();
                    let text = layout.bounds();
                    match &shown.block.kind {
                        Kind::Code => {
                            window.paint_quad(fill(frame, palette.code).corner_radii(rem * 0.25));
                            let rows = wrapped_rows(&layout, &shown.display);
                            if !rows.is_empty() {
                                let mark = beside(WRAPPED, palette.faint, window);
                                let left = text.left() - (rem * CODE_LEFT + mark.width) / 2.;
                                for row in rows {
                                    let at = point(left, row.y);
                                    let _ = mark.paint(at, layout.line_height(), window, cx);
                                }
                            }
                        }
                        Kind::Item { marker, .. } => {
                            let mut run = base.to_run(marker.len());
                            run.color = palette.faint.into();
                            let marker = window.text_system().shape_line(
                                marker.clone().into(),
                                size,
                                &[run],
                                None,
                            );
                            let at =
                                point(text.left() - rem * MARKER_GAP - marker.width, text.top());
                            let _ = marker.paint(at, layout.line_height(), window, cx);
                        }
                        Kind::Quote => {
                            let bar = Bounds::new(
                                point(frame.left(), frame.top()),
                                gpui::size(px(QUOTE_BAR), frame.size.height),
                            );
                            window.paint_quad(fill(bar, palette.line));
                        }
                        Kind::Rule => {
                            let line = Bounds::new(
                                point(frame.left(), frame.top()),
                                gpui::size(frame.size.width, px(1.)),
                            );
                            window.paint_quad(fill(line, palette.line));
                        }
                        Kind::Text | Kind::Heading(_) | Kind::Table { .. } => {}
                    }
                    styled.paint(None, None, text, &mut (), &mut (), window, cx);
                }
            }
        }

        // The copy control of a code block shows while the pointer is over
        // the block, and after a click reads "copied" for a short while.
        let copied = self.selection.read(cx).copied;
        let hovered = hitbox
            .is_hovered(window)
            .then(|| drawn.hovered_code(window.mouse_position()))
            .flatten();
        #[cfg(debug_assertions)]
        let hovered = hovered.or(self
            .selection
            .read(cx)
            .posed_hover
            .and_then(|(at, block)| (at == key).then_some(block)));
        for (ix, chip) in drawn.chips.iter() {
            let just_copied = copied == Some((key, *ix));
            if Some(*ix) != hovered && !just_copied {
                continue;
            }
            let Some(layout) = drawn.layouts[*ix].as_ref() else {
                continue;
            };
            let label = if just_copied { "copied" } else { "copy" };
            window.paint_quad(fill(*chip, palette.side).corner_radii(rem * 0.25));
            let mut run = base.to_run(label.len());
            // Said, not waiting: the text colour, never the accent.
            let color = if just_copied {
                palette.text
            } else {
                palette.faint
            };
            run.color = color.into();
            let shaped = window
                .text_system()
                .shape_line(label.into(), size, &[run], None);
            let height = layout.line_height();
            let at = point(
                chip.left() + (chip.size.width - shaped.width) / 2.,
                chip.top() + (chip.size.height - height) / 2.,
            );
            let _ = shaped.paint(at, height, window, cx);
        }

        // The link or reference under the pointer, where the text is
        // not covered.
        let over = {
            let (drawn, hitbox) = (drawn.clone(), hitbox.clone());
            move |position: Point<Pixels>, window: &Window| {
                hitbox
                    .is_hovered(window)
                    .then(|| drawn.clicked_at(position))
                    .flatten()
            }
        };
        let on_open = over(window.mouse_position(), window).is_some();
        let on_copy = drawn.copy_at(window.mouse_position()).is_some();
        let cursor = if on_open || on_copy {
            CursorStyle::PointingHand
        } else {
            CursorStyle::IBeam
        };
        window.set_cursor_style(cursor, hitbox);

        window.on_mouse_event({
            let (selection, text) = (self.selection.clone(), self.text.clone());
            let (drawn, hitbox, over) = (drawn.clone(), hitbox.clone(), over.clone());
            move |event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                    return;
                }
                if hitbox.is_hovered(window) {
                    // A click on a copy control copies the block exactly
                    // as it was written, and starts no selection.
                    if let Some(block) = drawn.copy_at(event.position) {
                        let shown = &drawn.blocks[block];
                        let copy = shown
                            .copy_override
                            .as_ref()
                            .map(ToString::to_string)
                            .or_else(|| shown.block.copy_text().map(str::to_string));
                        if let Some(text) = copy {
                            cx.write_to_clipboard(ClipboardItem::new_string(text));
                            selection
                                .update(cx, |selection, cx| selection.copied_block(key, block, cx));
                        }
                        return;
                    }
                    let pressed = Press {
                        place: drawn.place_at(event.position),
                        clicked: over(event.position, window),
                        shape: (event.click_count == 2)
                            .then(|| drawn.shape_at(event.position))
                            .flatten(),
                    };
                    selection.update(cx, |selection, cx| {
                        selection.press(key, text.clone(), pressed, event.click_count, cx)
                    });
                } else if selection.read(cx).chip_at(event.position).is_none() {
                    // A click on another text's copy control is no press
                    // outside either: it changes no selection.
                    selection.update(cx, |selection, cx| selection.pressed_outside(key, cx));
                }
            }
        });
        // A drag goes on outside the text, so these are the window's events
        // and not those of the place the text is drawn in.
        window.on_mouse_event({
            let (selection, drawn, hitbox, over) =
                (self.selection.clone(), drawn, hitbox.clone(), over.clone());
            move |event: &MouseMoveEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble {
                    return;
                }
                // The pointer came onto a link, a reference, a code
                // block or a copy control, or left one: the cursor and the
                // controls are set when the text is drawn.
                let hovered_now = hitbox
                    .is_hovered(window)
                    .then(|| drawn.hovered_code(event.position))
                    .flatten();
                if over(event.position, window).is_some() != on_open
                    || hovered_now != hovered
                    || drawn.copy_at(event.position).is_some() != on_copy
                {
                    window.refresh();
                }
                if !selection.read(cx).selecting(key) {
                    return;
                }
                // The button came up where the window did not see it, so
                // this is no click.
                if event.pressed_button != Some(MouseButton::Left) {
                    selection.update(cx, |selection, _| selection.release());
                    return;
                }
                let place = drawn.place_at(event.position);
                selection.update(cx, |selection, cx| selection.drag(place, cx));
            }
        });
        window.on_mouse_event({
            let selection = self.selection.clone();
            let on_ref = self.on_ref.clone();
            move |event: &MouseUpEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble
                    || event.button != MouseButton::Left
                    || !selection.read(cx).selecting(key)
                {
                    return;
                }
                // A click on a link or a reference: pressed and released
                // on it, and nothing dragged between the two.
                let pressed = selection.update(cx, |selection, _| selection.release());
                let released = over(event.position, window);
                match (pressed, released) {
                    (Some(clicked), Some(at)) if clicked == at => match clicked {
                        Clicked::Link(url) => cx.open_url(&url),
                        Clicked::Ref(target) => {
                            if let Some(on_ref) = &on_ref {
                                on_ref(target, window, cx);
                            }
                        }
                    },
                    _ => {}
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_press_selects_the_word_under_it() {
        let text = "see the fence, José";
        assert_eq!(word_at(text, 0), 0..3);
        assert_eq!(word_at(text, 5), 4..7);
        // The comma is no part of the word before it.
        assert_eq!(&text[word_at(text, 12)], "fence");
        assert_eq!(&text[word_at(text, 13)], ",");
        assert_eq!(&text[word_at(text, 16)], "José");
        // Past the end there is none.
        assert_eq!(word_at(text, text.len()), text.len()..text.len());
    }

    /// The one block `text` is drawn as.
    fn one(text: &str) -> Block {
        let mut blocks = markdown::blocks(text);
        assert_eq!(blocks.len(), 1, "{text:?}");
        blocks.remove(0)
    }

    fn dark() -> Palette {
        Palette::of(crate::theme::Look::Dark, gpui::WindowAppearance::Dark)
    }

    /// The text's own style, as the window sets it: seven eighths of a rem.
    fn base() -> TextStyle {
        TextStyle {
            font_size: rems(ROOT_TEXT_REM).into(),
            ..Default::default()
        }
    }

    fn mono_face() -> SharedString {
        "Menlo".into()
    }

    /// The runs of the one block of `text`, as (what is drawn, the run).
    fn drawn(text: &str, selected: Option<Range<usize>>) -> Vec<(String, TextRun)> {
        drawn_refs(text, selected, &[])
    }

    /// The block as it is drawn, with no resolved references.
    fn shown_block(block: Block) -> Shown {
        let cells = match &block.kind {
            Kind::Table { head, rows, .. } => std::iter::once(head)
                .chain(rows)
                .map(|cells| {
                    cells
                        .iter()
                        .map(|cell| {
                            let (display, links) =
                                shrink(&cell.text, &links(&cell.text), &cell.links);
                            CellShown {
                                display: display.into(),
                                links,
                                refs: Vec::new(),
                            }
                        })
                        .collect()
                })
                .collect(),
            _ => Vec::new(),
        };
        let (display, links) = shrink(&block.text, &links(&block.text), &block.links);
        Shown {
            block,
            display: display.into(),
            links,
            refs: Vec::new(),
            cells,
            copy_override: None,
        }
    }

    /// The one block of `text` as it is drawn, with no resolved
    /// references.
    fn shown_of(text: &str) -> Shown {
        shown_block(one(text))
    }

    /// The runs of the one block of `text` with resolved references at
    /// `refs`, as (what is drawn, the run). `selected` stands in the
    /// text as it was written, as the selection keeps it.
    fn drawn_refs(
        text: &str,
        selected: Option<Range<usize>>,
        refs: &[Range<usize>],
    ) -> Vec<(String, TextRun)> {
        let shown = shown_of(text);
        let (base, mono, palette) = (base(), mono_face(), dark());
        let face = face(&shown.block.kind, &base, Some(&mono), px(16.));
        let selected = selected.map(|selected| shown.block.shown(selected));
        let draw = Draw {
            face: &face,
            palette: &palette,
            mono: Some(&mono),
        };
        let mut at = 0;
        runs(
            &shown.display,
            &shown.block.marks,
            &shown.links,
            refs,
            selected,
            &draw,
        )
        .into_iter()
        .map(|run| {
            let words = shown.display[at..at + run.len].to_string();
            at += run.len;
            (words, run)
        })
        .collect()
    }

    #[test]
    fn a_text_without_a_mark_has_the_runs_it_had_before_there_were_marks() {
        let text = "see https://example.org/a now";
        let (base, palette) = (base(), dark());
        let shown = shown_of(text);
        let block = &shown.block;
        assert_eq!((&block.kind, block.text.as_str()), (&Kind::Text, text));
        let face = face(&block.kind, &base, Some(&mono_face()), px(16.));
        assert_eq!(face, base);

        // What gpui made of the link and the selection, when the text was
        // handed to it with the two as highlights.
        let link = gpui::HighlightStyle {
            color: Some(palette.link.into()),
            underline: Some(UnderlineStyle {
                thickness: px(1.),
                color: Some(palette.link.into()),
                wavy: false,
            }),
            ..Default::default()
        };
        let selection = gpui::HighlightStyle {
            background_color: Some(palette.selection.into()),
            ..Default::default()
        };
        let before = |highlights: Vec<(Range<usize>, gpui::HighlightStyle)>| {
            let mut runs = Vec::new();
            let mut at = 0;
            for (range, highlight) in gpui::combine_highlights(highlights, None) {
                if at < range.start {
                    runs.push(base.to_run(range.start - at));
                }
                runs.push(base.clone().highlight(highlight).to_run(range.len()));
                at = range.end;
            }
            if at < text.len() {
                runs.push(base.to_run(text.len() - at));
            }
            runs
        };
        let draw = Draw {
            face: &face,
            palette: &palette,
            mono: None,
        };
        let now = |selected: Option<Range<usize>>| {
            let selected = selected.map(|selected| shown.block.shown(selected));
            runs(
                &shown.display,
                &shown.block.marks,
                &shown.links,
                &[],
                selected,
                &draw,
            )
        };

        assert_eq!(now(None), before(vec![(4..25, link)]));
        assert_eq!(now(None).len(), 3);
        // A selection that begins before the link and ends in it.
        assert_eq!(
            now(Some(2..10)),
            before(vec![(4..25, link), (2..10, selection)])
        );
        assert_eq!(now(Some(2..10)).len(), 5);
        // Nothing in it: one run, the text's own.
        let plain = shown_of("Land it today.");
        let draw = Draw {
            face: &base,
            palette: &palette,
            mono: None,
        };
        assert_eq!(
            runs(
                &plain.display,
                &plain.block.marks,
                &plain.links,
                &[],
                None,
                &draw
            ),
            [base.to_run(14)]
        );
        let empty = shown_of("");
        assert_eq!(
            runs(
                &empty.display,
                &empty.block.marks,
                &empty.links,
                &[],
                None,
                &draw
            ),
            [base.to_run(0)]
        );
    }

    #[test]
    fn code_is_in_a_monospaced_face_on_a_quiet_ground() {
        let (base, mono, palette) = (base(), mono_face(), dark());
        let quiet = Some(palette.code.into());

        // A block: the face is the block's, and its ground is drawn under
        // all of it, not under each run.
        let mut blocks = markdown::blocks("Run:\n```sh\n  ls -la\n1. kept\n```");
        let shown = shown_block(blocks.remove(1));
        let code = &shown.block;
        assert_eq!(
            (&code.kind, code.text.as_str()),
            (&Kind::Code, "  ls -la\n1. kept")
        );
        let face = face(&code.kind, &base, Some(&mono), px(16.));
        assert_eq!(face.font_family, mono);
        assert_eq!(face.font_size, base.font_size);
        assert_eq!(face.font_weight, base.font_weight);
        let draw = Draw {
            face: &face,
            palette: &palette,
            mono: Some(&mono),
        };
        let all = runs(
            &shown.display,
            &shown.block.marks,
            &shown.links,
            &[],
            None,
            &draw,
        );
        assert_eq!(all, [face.to_run(code.text.len())]);
        assert_eq!(
            (&all[0].font.family, all[0].background_color),
            (&mono, None)
        );
        let inside = frame(code, Some(&Kind::Text)).padding;
        assert_eq!(inside.left, rems(CODE_LEFT).into());
        assert_eq!(inside.top, rems(CODE_ABOVE).into());

        // Inline: the run alone is in the face and on the ground.
        let all = drawn("run `yardr prime` now", None);
        let words: Vec<&str> = all.iter().map(|(words, _)| words.as_str()).collect();
        assert_eq!(words, ["run ", "yardr prime", " now"]);
        let mut code = base.to_run(11);
        code.font.family = mono.clone();
        code.background_color = quiet;
        assert_eq!(all[0].1, base.to_run(4));
        assert_eq!(all[1].1, code);
        assert_eq!(all[2].1, base.to_run(4));

        // A machine without one of the faces has the ground alone.
        let shown = shown_of("run `yardr prime` now");
        let draw = Draw {
            face: &base,
            palette: &palette,
            mono: None,
        };
        let all = runs(
            &shown.display,
            &shown.block.marks,
            &shown.links,
            &[],
            None,
            &draw,
        );
        assert_eq!(all[1].font, base.to_run(0).font);
        assert_eq!(all[1].background_color, quiet);
    }

    #[test]
    fn a_code_block_s_copy_control_is_a_chip_at_its_ground_s_top_right() {
        let frame = Bounds::new(point(px(10.), px(20.)), size(px(300.), px(100.)));
        let chip = copy_chip(frame, px(16.));
        // Fixed size, whatever the control reads, and inside the ground.
        assert_eq!(chip.size, size(px(64.), px(24.)));
        assert_eq!(chip.origin, point(px(10. + 300. - 4. - 64.), px(20. + 4.)));
        assert!(frame.contains(&chip.origin));
        assert!(frame.contains(&chip.bottom_right()));
    }

    #[test]
    fn a_heading_is_bold_and_a_little_larger_than_the_text() {
        let (base, mono) = (base(), mono_face());
        let mut larger = Vec::new();
        for (text, level) in [("# One", 1), ("## Two", 2), ("### Three", 3)] {
            let shown = shown_of(text);
            let block = &shown.block;
            assert_eq!(block.kind, Kind::Heading(level));
            let face = face(&block.kind, &base, Some(&mono), px(16.));
            assert_eq!(face.font_weight, FontWeight::BOLD);
            assert_eq!(face.font_family, base.font_family);
            let dark = dark();
            let draw = Draw {
                face: &face,
                palette: &dark,
                mono: Some(&mono),
            };
            let all = runs(
                &shown.display,
                &shown.block.marks,
                &shown.links,
                &[],
                None,
                &draw,
            );
            assert_eq!(all, [face.to_run(block.text.len())]);
            assert_eq!(all[0].font.weight, FontWeight::BOLD);
            larger.push(face.font_size.to_pixels(px(16.)));
        }
        // The text is 14 pixels at a rem of 16: never "extremely large".
        let text = px(14.);
        assert_eq!(larger, [text * 1.25, text * 1.15, text * 1.05]);
    }

    #[test]
    fn the_text_size_scales_a_heading_with_the_text() {
        let (base, heading) = (base(), Kind::Heading(1));
        for rem in [px(13.), px(16.), px(20.)] {
            let text = base.font_size.to_pixels(rem);
            let face = face(&heading, &base, None, rem);
            assert_eq!(face.font_size.to_pixels(rem), text * 1.25);
        }
        // What is around a block is in rems, and so goes with it too.
        let item = one("- one");
        let indent = gpui::DefiniteLength::from(rems(ITEM_INDENT));
        assert_eq!(frame(&item, None).padding.left, indent);
    }

    #[test]
    fn bold_and_italic_are_runs_of_the_text_s_face() {
        let base = base();
        let all = drawn("a **bold** and an *italic* and ***both***", None);
        let words: Vec<&str> = all.iter().map(|(words, _)| words.as_str()).collect();
        assert_eq!(words, ["a ", "bold", " and an ", "italic", " and ", "both"]);
        let mut bold = base.to_run(4);
        bold.font.weight = FontWeight::BOLD;
        let mut italic = base.to_run(6);
        italic.font.style = FontStyle::Italic;
        let mut both = bold.clone();
        both.font.style = FontStyle::Italic;
        assert_eq!(all[0].1, base.to_run(2));
        assert_eq!(all[1].1, bold);
        assert_eq!(all[2].1, base.to_run(8));
        assert_eq!(all[3].1, italic);
        assert_eq!(all[5].1, both);
    }

    #[test]
    fn an_item_is_text_set_in_with_its_link_and_its_marks() {
        let (base, palette) = (base(), dark());
        let text = "- see **https://example.org/a** now";
        let block = one(text);
        let marker = "•".to_string();
        assert_eq!(block.kind, Kind::Item { marker, depth: 0 });
        assert_eq!(face(&block.kind, &base, None, px(16.)), base);
        // The link is found in what is drawn, without the stars after it.
        assert_eq!(linked(&block.text), ["https://example.org/a"]);
        let all = drawn(text, None);
        let words: Vec<&str> = all.iter().map(|(words, _)| words.as_str()).collect();
        assert_eq!(words, ["see ", "https://example.org/a", " now"]);
        let link = &all[1].1;
        assert_eq!(link.font.weight, FontWeight::BOLD);
        assert_eq!(link.color, palette.link.into());
        assert!(link.underline.is_some());

        // An item under another stands in twice as far, and the words of
        // both hang: the indent is the block's, not the first line's.
        let blocks = markdown::blocks("1. one\n   - under");
        let left = |ix: usize| frame(&blocks[ix], None).padding.left;
        assert_eq!(left(0), rems(ITEM_INDENT).into());
        assert_eq!(left(1), rems(2. * ITEM_INDENT).into());
    }

    #[test]
    fn what_is_selected_is_drawn_without_the_marks_it_has() {
        let palette = dark();
        // All of it, as it was written: the stars are not drawn.
        let text = "a **bold** b";
        let all = drawn(text, Some(0..text.len()));
        assert!(
            all.iter()
                .all(|(_, run)| run.background_color == Some(palette.selection.into()))
        );
        let words: String = all.iter().map(|(words, _)| words.as_str()).collect();
        assert_eq!(words, "a bold b");
        // From inside the bold word on: the selection's ground is over the
        // code's.
        let all = drawn("`code` b", Some(3..8));
        let grounds: Vec<(&str, Option<gpui::Hsla>)> = all
            .iter()
            .map(|(words, run)| (words.as_str(), run.background_color))
            .collect();
        assert_eq!(
            grounds,
            [
                ("co", Some(palette.code.into())),
                ("de", Some(palette.selection.into())),
                (" b", Some(palette.selection.into())),
            ]
        );
    }

    #[gpui::test]
    fn a_selection_has_the_marks_between_its_ends_and_none_at_them(cx: &mut gpui::TestAppContext) {
        let src = "one\n```sh\nls -la\n```\na **bold** b";
        let blocks = markdown::blocks(src);
        let (code, last) = (&blocks[1], &blocks[2]);
        assert_eq!(last.text, "a bold b");
        use gpui::AppContext as _;
        let selection = cx.new(|_| Selection::default());
        let mut selected =
            |text: &'static str, from: Range<usize>, to: Range<usize>, clicks: usize| {
                selection.update(cx, |selection, cx| {
                    let pressed = Press {
                        place: from,
                        clicked: None,
                        shape: None,
                    };
                    selection.press(1, text.into(), pressed, clicks, cx);
                    selection.drag(to, cx);
                    selection.text().map(str::to_string)
                })
            };

        // The bold word alone, either way: the word.
        let (start, end) = (last.source_at(2), last.source_at(6));
        assert_eq!(
            selected(src, start.clone(), end.clone(), 1).as_deref(),
            Some("bold")
        );
        assert_eq!(
            selected(src, end.clone(), start.clone(), 1).as_deref(),
            Some("bold")
        );
        // With what is around it: as it was written.
        let (first, after) = (last.source_at(0), last.source_at(8));
        assert_eq!(
            selected(src, first.clone(), after.clone(), 1).as_deref(),
            Some("a **bold** b")
        );
        // The code alone is the code, a command as it is typed.
        let (top, bottom) = (code.source_at(0), code.source_at(6));
        assert_eq!(
            selected(src, top.clone(), bottom.clone(), 1).as_deref(),
            Some("ls -la")
        );
        assert_eq!(selected(src, bottom, top, 1).as_deref(), Some("ls -la"));
        // Across the code block, from the text before it to the text after
        // it: the fences and the language are in it.
        assert_eq!(selected(src, 0..0, after.clone(), 1).as_deref(), Some(src));
        assert_eq!(selected(src, after, 0..0, 1).as_deref(), Some(src));
        // A second press on the bold word selects it.
        assert_eq!(
            selected(src, start.clone(), start, 2).as_deref(),
            Some("bold")
        );
        // A quote's drawn words whole: the break and the next line's
        // marker stand in the middle of the stretch, the first marker
        // left off, since the selection begins at the first drawn word.
        let quoted = "> Hold the timetable.\n> It is printed.";
        let quote = &markdown::blocks(quoted)[0];
        assert_eq!(
            selected(quoted, quote.source_at(0), quote.source_at(34), 1).as_deref(),
            Some("Hold the timetable.\n> It is printed.")
        );
        // Either line alone copies without its own marker.
        assert_eq!(
            selected(quoted, quote.source_at(0), quote.source_at(19), 1).as_deref(),
            Some("Hold the timetable.")
        );
        assert_eq!(
            selected(quoted, quote.source_at(20), quote.source_at(34), 1).as_deref(),
            Some("It is printed.")
        );
        // A named link's words: the shape whole.
        let named = "See the [release page](https://yardr.dev/releases) first.";
        let link = &markdown::blocks(named)[0];
        assert_eq!(
            selected(named, link.source_at(10), link.source_at(20), 1).as_deref(),
            Some("[release page](https://yardr.dev/releases)")
        );
        // A second press on the words selects the shape whole.
        let shape = Some(link.links[0].source.clone());
        let doubled = selection.update(cx, |selection, cx| {
            let pressed = Press {
                place: link.source_at(12),
                clicked: None,
                shape,
            };
            selection.press(1, named.into(), pressed, 2, cx);
            selection.text().map(str::to_string)
        });
        assert_eq!(
            doubled.as_deref(),
            Some("[release page](https://yardr.dev/releases)")
        );
        // A press and no drag selects nothing.
        let none = selection.update(cx, |selection, cx| {
            let pressed = Press {
                place: 2..4,
                clicked: None,
                shape: None,
            };
            selection.press(1, src.into(), pressed, 1, cx);
            selection.text().map(str::to_string)
        });
        assert_eq!(none, None);
    }

    fn linked(text: &str) -> Vec<&str> {
        links(text).into_iter().map(|link| &text[link]).collect()
    }

    #[test]
    fn a_link_goes_from_its_scheme_to_the_next_whitespace() {
        assert_eq!(linked("https://a.example/x"), ["https://a.example/x"]);
        assert_eq!(linked("see http://a.example now"), ["http://a.example"]);
        assert_eq!(
            linked("one https://a.example/x?y=1&z=(2)#top\nand https://b.example/, both"),
            ["https://a.example/x?y=1&z=(2)#top", "https://b.example/"]
        );
        // A word before it without a space is no part of it.
        assert_eq!(linked("José:https://a.example"), ["https://a.example"]);
        // In code the named shape is the code's own characters, and both
        // address runs still open as bare addresses do.
        assert_eq!(
            linked("[release page](https://yardr.dev/releases) and https://yardr.dev/releases"),
            ["https://yardr.dev/releases", "https://yardr.dev/releases"]
        );
    }

    #[test]
    fn what_closes_the_sentence_or_the_bracket_is_no_part_of_a_link() {
        assert_eq!(
            linked("It is at https://a.example/x."),
            ["https://a.example/x"]
        );
        assert_eq!(
            linked("(see https://a.example/x)."),
            ["https://a.example/x"]
        );
        assert_eq!(linked("<https://a.example/x>"), ["https://a.example/x"]);
        assert_eq!(linked("'https://a.example/x'!?"), ["https://a.example/x"]);
        assert_eq!(
            linked("[\"https://a.example/x\"]; https://b.example:"),
            ["https://a.example/x", "https://b.example"]
        );
    }

    #[test]
    fn nothing_but_http_and_https_is_a_link() {
        assert!(linked("no link in this, José").is_empty());
        assert!(linked("").is_empty());
        assert!(linked("ftp://x and file:///x").is_empty());
        assert!(linked("www.a.example, jo@a.example, http and https").is_empty());
        // The scheme alone names nothing.
        assert!(linked("https:// and (http://).").is_empty());
        assert_eq!(linked("httpx http://a.example"), ["http://a.example"]);
    }

    /// A long link for the shortening: past the length that draws whole.
    const LONG: &str =
        "https://teams.microsoft.com/l/meetup-join/19%3ameeting_N2Q4YzQefBDwL/0?context=chat";

    #[test]
    fn a_long_link_draws_short_and_maps_to_the_whole_link() {
        let text = format!("call: {LONG} now");
        let (display, spans) = shrink(&text, &links(&text), &[]);
        assert_eq!(spans.len(), 1);
        let link = &spans[0];
        assert_eq!(&text[link.from.clone()], LONG);
        assert_eq!(&display[link.shown.clone()], markdown::short_link(LONG));
        assert_eq!(link.url.as_ref(), LONG);
        assert!(link.short);
        assert_eq!(display, format!("call: {} now", markdown::short_link(LONG)));
        // A short link draws as written, and carries no tooltip.
        let text = "see https://example.org/a now";
        let (display, spans) = shrink(text, &links(text), &[]);
        assert_eq!(display, text);
        assert_eq!(spans.len(), 1);
        assert!(!spans[0].short);
        // None at all: the text draws as it is.
        let (display, spans) = shrink("no link in this", &[], &[]);
        assert_eq!(display, "no link in this");
        assert!(spans.is_empty());
    }

    #[test]
    fn display_and_text_boundaries_map_past_shortened_links() {
        let text = format!("call: {LONG} now, and https://example.org/a too");
        let (display, spans) = shrink(&text, &links(&text), &[]);
        assert_eq!(spans.len(), 2);
        let short = &spans[0];
        assert!(short.short);
        let plain = &spans[1];
        assert!(!plain.short);
        // Before the short link the two stand alike.
        assert_eq!(to_src(&spans, 0), 0);
        assert_eq!(to_src(&spans, short.shown.start), short.from.start);
        // Inside its short form at its start, and at its end on its end:
        // selecting the short form selects the whole link.
        assert_eq!(to_src(&spans, short.shown.start + 1), short.from.start);
        assert_eq!(to_src(&spans, short.shown.end - 1), short.from.start);
        assert_eq!(to_src(&spans, short.shown.end), short.from.end);
        // Past it the shift it made, and the short link after it stands
        // whole in both.
        let after = short.shown.end + 1;
        assert_eq!(
            &display[after..after + 1],
            &text[to_src(&spans, after)..][..1]
        );
        assert_eq!(to_src(&spans, plain.shown.start), plain.from.start);
        assert_eq!(to_src(&spans, plain.shown.end), plain.from.end);
        assert_eq!(to_src(&spans, display.len()), text.len());
        // Inside a link that draws whole the same character, shifted
        // past the shortened one.
        assert_eq!(to_src(&spans, plain.shown.start + 1), plain.from.start + 1);
        assert_eq!(
            to_display(&spans, plain.from.start + 1..plain.from.end - 1),
            plain.shown.start + 1..plain.shown.end - 1
        );
        // The other way: a bound inside the long link rounds out to its
        // short form, and every other bound only shifts.
        assert_eq!(to_display(&spans, short.from.clone()), short.shown.clone());
        assert_eq!(
            to_display(&spans, short.from.start + 1..short.from.end - 1),
            short.shown.clone()
        );
        assert_eq!(
            to_display(&spans, 0..short.from.start),
            0..short.shown.start
        );
        assert_eq!(to_display(&spans, plain.from.clone()), plain.shown.clone());
        assert_eq!(
            to_display(&spans, text.len()..text.len()),
            display.len()..display.len()
        );
        // A round trip past the links stands where it began.
        for at in [
            0,
            short.from.start,
            short.from.end,
            plain.from.start,
            text.len(),
        ] {
            let there = to_display(&spans, at..at);
            assert_eq!(there.start, there.end);
            assert_eq!(to_src(&spans, there.start), at, "{at}");
        }
    }

    /// What `refs` names in `text`: each where it stands, with what it is.
    fn reffed(text: &str) -> Vec<(&str, RefKind)> {
        refs(text)
            .into_iter()
            .map(|found| (&text[found.range], found.kind))
            .collect()
    }

    #[test]
    fn an_ask_is_its_number_after_ask() {
        assert_eq!(reffed("ask 6926"), [("ask 6926", RefKind::Ask(6926))]);
        assert_eq!(
            reffed("its card (ask 6926)."),
            [("ask 6926", RefKind::Ask(6926))]
        );
        // In any case, after any space, with leading zeroes.
        assert_eq!(reffed("Ask 007?"), [("Ask 007", RefKind::Ask(7))]);
        assert_eq!(reffed("ASK\t42"), [("ASK\t42", RefKind::Ask(42))]);
        // Two asks, in order.
        assert_eq!(
            reffed("ask 6893, not ask 6891."),
            [
                ("ask 6893", RefKind::Ask(6893)),
                ("ask 6891", RefKind::Ask(6891)),
            ]
        );
    }

    #[test]
    fn an_ask_with_its_bead_is_one_reference() {
        assert_eq!(
            reffed("ask 18 on yd-crm is declined"),
            [(
                "ask 18 on yd-crm",
                RefKind::AskOn {
                    ask: 18,
                    bead: "yd-crm".to_string(),
                    bead_range: 10..16,
                }
            )]
        );
        // The bead alone still opens where the ask is none the yard said.
        let found = &refs("ask 18 on yd-crm")[0];
        assert_eq!(
            &"ask 18 on yd-crm"[match &found.kind {
                RefKind::AskOn { bead_range, .. } => bead_range.clone(),
                _ => panic!("{found:?}"),
            }],
            "yd-crm"
        );
    }

    #[test]
    fn ask_without_a_number_is_no_ask() {
        assert!(refs("").is_empty());
        assert!(refs("ask yardmaster, ask about this").is_empty());
        assert!(refs("asked 5, asks 5, ask5, task 5").is_empty());
        assert!(refs("ask five, ask 5x, ask 99999999999999999999999").is_empty());
        // `on` without a bead id leaves the number alone.
        assert_eq!(reffed("ask 5 on Friday"), [("ask 5", RefKind::Ask(5))]);
        // `re-ask` is shaped as a bead id is: the model, which knows the
        // yard's beads, leaves it plain.
        assert_eq!(
            reffed("re-ask 5"),
            [("re-ask", RefKind::Bead("re-ask".to_string()))]
        );
    }

    #[test]
    fn a_bead_id_is_an_id_with_a_dash() {
        assert_eq!(
            reffed("over the limit (yardr-e9f0), so run:"),
            [("yardr-e9f0", RefKind::Bead("yardr-e9f0".to_string()))]
        );
        assert_eq!(
            reffed("see footplate-ky46 and yd-zulu."),
            [
                (
                    "footplate-ky46",
                    RefKind::Bead("footplate-ky46".to_string())
                ),
                ("yd-zulu", RefKind::Bead("yd-zulu".to_string())),
            ]
        );
        // Another yard's, with its peer; a short test id; what the
        // sentence puts after it is no part of it.
        assert_eq!(
            reffed("ask 3 on bead-a@north, bead-s, yd-zulu's turn."),
            [
                (
                    "ask 3 on bead-a@north",
                    RefKind::AskOn {
                        ask: 3,
                        bead: "bead-a@north".to_string(),
                        bead_range: 9..21,
                    }
                ),
                ("bead-s", RefKind::Bead("bead-s".to_string())),
                ("yd-zulu", RefKind::Bead("yd-zulu".to_string())),
            ]
        );
    }

    #[test]
    fn what_has_no_dash_is_no_bead_id() {
        assert!(refs("no bead here").is_empty());
        assert!(refs("clerk@north, jo, yardmaster@droplet").is_empty());
        assert!(refs("a@b@c-d, -x, up- -to- -date").is_empty());
        // Whole words: no bead id hides in a longer one.
        assert_eq!(
            reffed("xfootplate-ky46 stands whole"),
            [(
                "xfootplate-ky46",
                RefKind::Bead("xfootplate-ky46".to_string())
            )]
        );
    }

    #[test]
    fn a_link_s_words_are_no_reference() {
        assert!(refs("see https://example.org/ask 5 now").is_empty());
        assert_eq!(
            linked("see https://example.org/a-b now"),
            ["https://example.org/a-b"]
        );
        assert!(refs("see https://example.org/a-b now").is_empty());
        // Beside a link, a reference still stands.
        assert_eq!(
            reffed("ask 5: https://example.org/a"),
            [("ask 5", RefKind::Ask(5))]
        );
    }

    #[test]
    fn a_reference_is_a_chip_in_the_link_blue_on_the_code_s_ground() {
        let (palette, mono) = (dark(), mono_face());
        let text = "its card (ask 6926).";
        let span = [Range { start: 10, end: 18 }];
        let all = drawn_refs(text, None, &span);
        assert_eq!(all.len(), 3);
        let chip = &all[1];
        assert_eq!(chip.0, "ask 6926");
        assert_eq!(chip.1.color, palette.link.into());
        assert_eq!(chip.1.background_color, Some(palette.code.into()));
        assert_eq!(chip.1.font.family, mono);
        assert!(chip.1.underline.is_none());
        // A selection over it keeps the link blue on its own ground.
        let all = drawn_refs(text, Some(12..14), &span);
        let words: Vec<&str> = all.iter().map(|(words, _)| words.as_str()).collect();
        assert_eq!(words, ["its card (", "as", "k ", "6926", ")."]);
        assert_eq!(all[2].1.color, palette.link.into());
        assert_eq!(all[2].1.background_color, Some(palette.selection.into()));
        // And a bold reference keeps its weight.
        let all = drawn_refs("its card (**ask 6926**).", None, &span);
        assert_eq!(all[1].0, "ask 6926");
        assert_eq!(all[1].1.font.weight, FontWeight::BOLD);
        assert_eq!(all[1].1.color, palette.link.into());
    }

    #[test]
    fn a_named_link_draws_its_words_with_its_host_beside_them() {
        let text = "See the [release page](https://yardr.dev/releases) first.";
        let block = one(text);
        assert_eq!(block.links.len(), 1);
        let (display, spans) = shrink(&block.text, &links(&block.text), &block.links);
        assert_eq!(display, "See the release page · yardr.dev first.");
        assert_eq!(spans.len(), 1);
        let link = &spans[0];
        assert_eq!(&block.text[link.from.clone()], "release page");
        assert_eq!(&display[link.shown.clone()], "release page · yardr.dev");
        assert_eq!(link.url.as_ref(), "https://yardr.dev/releases");
        assert!(link.short);
        assert_eq!(
            link.host.clone().map(|host| display[host].to_string()),
            Some(" · yardr.dev".to_string())
        );
        assert_eq!(link_at(&spans, link.shown.start), Some(link));
        assert_eq!(link_at(&spans, dressed(link).end - 1), Some(link));
        let host = link.host.as_ref().unwrap();
        assert!(link_at(&spans, host.start + HOST_SEP.len()).is_none());
        // Selecting any part of the words selects the shape whole:
        // inside at the words' start, at their end with the host on the
        // words' end.
        assert_eq!(to_src(&spans, link.shown.start), link.from.start);
        assert_eq!(to_src(&spans, link.shown.start + 1), link.from.start);
        assert_eq!(to_src(&spans, link.shown.end - 1), link.from.start);
        assert_eq!(to_src(&spans, link.shown.end), link.from.end);
        // The other way the highlight draws over the words alone.
        assert_eq!(to_display(&spans, link.from.clone()), dressed(link));
        assert_eq!(
            dressed(link),
            link.shown.start..link.host.as_ref().unwrap().start
        );
        // An empty range stands where it stands, shifted past the host.
        assert_eq!(to_display(&spans, 0..0), 0..0);
        assert_eq!(
            to_display(&spans, link.from.end..link.from.end),
            link.shown.end..link.shown.end
        );
        // A round trip past the link stands where it began.
        for at in [0, link.from.start, link.from.end, block.text.len()] {
            let there = to_display(&spans, at..at);
            assert_eq!(there.start, there.end);
            assert_eq!(to_src(&spans, there.start), at, "{at}");
        }
    }

    #[test]
    fn a_bare_link_over_a_named_link_s_words_is_none() {
        // A bare address inside the words is the link's words and no
        // link: the tap opens the named address.
        let text = "See [the https://example.org/a page](https://yardr.dev/p).";
        let block = one(text);
        let (_, spans) = shrink(&block.text, &links(&block.text), &block.links);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].url.as_ref(), "https://yardr.dev/p");
        // Beside the words, a bare address still stands.
        let text = "See [the page](https://yardr.dev/p) and https://yardr.dev/q.";
        let block = one(text);
        let (_, spans) = shrink(&block.text, &links(&block.text), &block.links);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].url.as_ref(), "https://yardr.dev/p");
        assert_eq!(spans[1].url.as_ref(), "https://yardr.dev/q");
    }

    #[test]
    fn a_named_link_is_its_words_in_the_link_blue_with_its_host_faint() {
        let palette = dark();
        let text = "See the [release page](https://yardr.dev/releases) first.";
        let all = drawn(text, None);
        let words: Vec<&str> = all.iter().map(|(words, _)| words.as_str()).collect();
        assert_eq!(
            words,
            ["See the ", "release page", " · yardr.dev", " first."]
        );
        let link = &all[1].1;
        assert_eq!(link.color, palette.link.into());
        assert!(link.underline.is_some());
        let host = &all[2].1;
        assert_eq!(host.color, palette.faint.into());
        assert!(host.underline.is_none());
        assert_eq!(host.font.weight, base().font_weight);
        // Bold words keep their weight under the link dress.
        let all = drawn("**[held](https://example.org)**", None);
        assert_eq!(all[0].0, "held");
        assert_eq!(all[0].1.font.weight, FontWeight::BOLD);
        assert_eq!(all[0].1.color, palette.link.into());
        // A selection over the words highlights them alone.
        let all = drawn(text, Some(8..50));
        let grounds: Vec<(&str, Option<gpui::Hsla>)> = all
            .iter()
            .map(|(words, run)| (words.as_str(), run.background_color))
            .collect();
        assert_eq!(
            grounds,
            [
                ("See the ", None),
                ("release page", Some(palette.selection.into())),
                (" · yardr.dev", None),
                (" first.", None),
            ]
        );
    }

    #[test]
    fn a_quote_stands_in_a_rule_takes_no_room_and_a_table_takes_the_wide() {
        let quote = one("> quoted");
        assert_eq!(
            frame(&quote, Some(&Kind::Text)).padding.left,
            rems(QUOTE_INSET).into()
        );
        assert_eq!(face(&quote.kind, &base(), None, px(16.)), base());
        // A rule's frame is its hairline alone, with the wide room above
        // and below it.
        let rule = one("---");
        assert_eq!(
            frame(&rule, Some(&Kind::Text)).margin.top,
            rems(APART).into()
        );
        assert_eq!(frame(&rule, None).size.height, px(1.).into());
        let text = one("words");
        assert_eq!(
            frame(&text, Some(&Kind::Rule)).margin.top,
            rems(APART).into()
        );
        // Above any table, as above a heading.
        let table = one("| a | b |\n| --- | --- |");
        assert_eq!(
            frame(&table, Some(&Kind::Text)).margin.top,
            rems(APART).into()
        );
    }

    #[test]
    fn a_reference_inside_a_named_link_s_words_is_no_chip() {
        let block = one("See [ask 5 on yd-crm](https://yardr.dev/p) and ask 6.");
        let found = block_refs(&block);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, None);
        assert_eq!(&block.text[found[0].1.range.clone()], "ask 6");
        // In a table each cell's references stand with their cell, row 0
        // the head, each range in its cell's words.
        let block = one("| ask 1 | x |\n| --- | --- |\n| yd-zulu | ask 2 |");
        let found = block_refs(&block);
        let cells: Vec<Option<(usize, usize)>> = found.iter().map(|(cell, _)| *cell).collect();
        assert_eq!(cells, [Some((0, 0)), Some((1, 0)), Some((1, 1))]);
        assert_eq!(found[0].1.range, 0..5);
        let block = one("| [ask 9](https://yardr.dev/p) | x |\n| --- | --- |");
        assert!(block_refs(&block).is_empty());
    }

    #[test]
    fn table_columns_are_weighed_by_their_cells_up_to_two_fifths() {
        // Each case as (measures, shares): weighed by the widest cell,
        // capped at two fifths, what remains shared, adding to one.
        for (natural, shares) in [
            (&[1., 1.][..], vec![0.5, 0.5]),
            (&[2., 1., 1.][..], vec![0.4, 0.3, 0.3]),
            (&[1., 1., 1., 1.][..], vec![0.25, 0.25, 0.25, 0.25]),
            (&[3., 1., 1., 1.][..], vec![0.4, 0.2, 0.2, 0.2]),
            (&[5.][..], vec![1.]),
            (&[0., 0.][..], vec![0.5, 0.5]),
        ] {
            let got = columns(natural);
            assert_eq!(got.len(), shares.len(), "{natural:?}");
            for (got, want) in got.iter().zip(shares.iter()) {
                assert!((got - want).abs() < 1e-6, "{natural:?}: {got:?}");
            }
            let sum: f32 = got.iter().sum();
            assert!((sum - 1.).abs() < 1e-6, "{natural:?}: {got:?}");
        }
    }

    #[test]
    fn a_table_s_head_is_set_semibold() {
        let base = base();
        assert_eq!(cell_face(false, &base), base);
        let head = cell_face(true, &base);
        assert_eq!(head.font_weight, FontWeight::SEMIBOLD);
        assert_eq!(head.font_size, base.font_size);
        assert_eq!(head.font_family, base.font_family);
    }

    #[gpui::test]
    fn a_table_s_cells_draw_with_their_links_and_their_references(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext as _;
        let selection = cx.new(|_| Selection::default());
        let text =
            "| see [the page](https://yardr.dev/p) | ask 7 |\n| --- | --- |\n| yd-zulu | x |";
        let blocks = markdown::blocks(text);
        assert_eq!(blocks.len(), 1);
        let refs = vec![
            RefSpan {
                block: 0,
                cell: Some((0, 1)),
                range: 0..5,
                target: RefTarget::Ask {
                    bead: "bead-x".into(),
                    ask: 7,
                    open: true,
                },
            },
            RefSpan {
                block: 0,
                cell: Some((1, 0)),
                range: 0..7,
                target: RefTarget::Bead {
                    bead: "yd-zulu".into(),
                },
            },
        ];
        let made = text_inner(1, text.into(), blocks, None, refs, None, &selection);
        let shown = &made.blocks[0];
        assert_eq!(shown.display.as_ref(), "");
        assert!(shown.links.is_empty());
        assert!(shown.refs.is_empty());
        assert_eq!(shown.cells.len(), 2);
        assert_eq!(shown.cells[0].len(), 2);
        // The head's first cell draws its words with the host beside
        // them, opening the named address.
        assert_eq!(
            shown.cells[0][0].display.as_ref(),
            "see the page · yardr.dev"
        );
        assert_eq!(shown.cells[0][0].links.len(), 1);
        assert_eq!(
            shown.cells[0][0].links[0].url.as_ref(),
            "https://yardr.dev/p"
        );
        assert!(shown.cells[0][0].refs.is_empty());
        // Each cell's references stand in that cell's words.
        assert_eq!(shown.cells[0][1].display.as_ref(), "ask 7");
        assert_eq!(shown.cells[0][1].refs.len(), 1);
        assert_eq!(shown.cells[0][1].refs[0].0, 0..5);
        assert_eq!(shown.cells[1][0].refs.len(), 1);
        assert_eq!(shown.cells[1][0].refs[0].0, 0..7);
        assert!(shown.cells[1][1].refs.is_empty());
    }
}
