//! The window's measures: its fixed sizes, and the small functions that
//! fit a pane or a drawer beside them. In pixels where the frame stands
//! fixed, in rems where the words in it set the size.

use gpui::{Div, Pixels, div, prelude::*, px, rems};

use crate::titlebar;

/// How high an image of a mail stands in the conversation, whatever its own
/// size: the room is there before the image is, so nothing moves under the
/// reader when it comes.
pub(crate) const IMAGE_HEIGHT: f32 = 240.;

/// The margin an image open over the window leaves out, each side, in
/// rems: what its fit is short of the window by.
pub(crate) const IMAGE_MARGIN: f32 = 2.;

/// The side of an attachment of the draft, above the line, in rems: its
/// name stands in it where it is no image, so it goes with the text size.
pub(crate) const ATTACHMENT: f32 = 3.5;

/// How wide the column of a pack ask's plan rows that holds the kind chip
/// stands, in rems: the longest of the usual kinds (`pointer`) whole, so
/// every row's name starts at one place. A longer kind (`subscription`)
/// pushes its own row's name past it. In rems because the word in it is.
pub(crate) const PLAN_KIND: f32 = 4.5;

/// How wide the Voice section's labels stand before their fields, in rems:
/// the longest of them (`microphone`) whole, and every field after it at one
/// place. In rems because the words in it are.
pub(crate) const VOICE_LABEL: f32 = 6.;

/// How many chars of a microphone's id its row in the Voice settings' list
/// shows before an ellipsis (`ui::shorten`). Measured as DejaVu Sans draws
/// it (the widest sans of the desktops here, and the font the Linux shots
/// draw in) at the base text size, 14 pixels, with the picker at its widest,
/// 28 rems: the list's border and padding and the row's leave 414 pixels
/// for the row's two texts and the gap between them. The tightest row of
/// the shots' microphones is the Studio Display's, the one chosen and so
/// drawn bold: its name takes 211 of them, which leaves 203 for the id.
/// 18 chars and the ellipsis take 175 of those; 22 would take 211 and cut
/// the ellipsis itself. The Mac's own font is narrower still. Everything in
/// the row is in rems but its hairlines, so this holds at every text size
/// where the picker stands at its widest; below that, the row's
/// `truncate()` cuts where the room ends, as it did.
pub(crate) const INPUT_NOTE: usize = 18;

/// How many chars of a microphone's name its row in the Voice settings' list
/// shows before an ellipsis (`ui::shorten`), measured as [`INPUT_NOTE`] is,
/// in the same 414 pixels of the picker at its widest. The longest name of
/// the shots' microphones, the Studio Display's, is 25 chars and stands
/// whole: drawn bold in the chosen row it takes 211 pixels, which leaves the
/// 203 that [`INPUT_NOTE`] counts on for the id. A longer name is cut there,
/// and its 25 chars and the ellipsis take 226, which leave 188 for the id's
/// 175. Left to the name's own cap, 16 rems or 256 pixels, a long name would
/// leave the id 158, and the id's `truncate()` would clip it mid-glyph, as it
/// did the name. The Mac's own font is narrower still. A name of wider
/// letters than the Studio Display's is cut by its `truncate()` where its
/// room ends, as it was.
pub(crate) const INPUT_NAME: usize = 25;

/// How many chars of a voice's note its row in the Voice settings' list
/// shows before an ellipsis (`ui::shorten`), measured as
/// [`INPUT_NOTE`] is, with the picker at its widest, 22 rems: 318 pixels
/// for the name and the note. Beside the widest name, `meridian`, drawn
/// bold where it is the choice, 248 are left for the note, and 26 chars and
/// the ellipsis take 225 of those. The longest note OpenAI's guide gives,
/// `English, North American`, is 23 chars and stands whole, in the widest
/// row 262 of the 318 pixels, so no note is cut today; a longer one the
/// guide adds is cut where it would not fit, and a test holds the notes
/// to this number.
pub(crate) const VOICE_NOTE: usize = 26;

/// How many chars of its choice a Select's field shows before an ellipsis
/// (`ui::shorten`), measured as [`INPUT_NOTE`] is, in the narrower of the
/// two fields, the voice's, at its widest, 22 rems: its border, padding, gap
/// and chevron leave 302 pixels for the choice, and 32 chars of a long id and
/// the ellipsis take 267 of those. The microphone's field, at 28 rems, has
/// 398 pixels and room over. The choice is a long id before the list is
/// read; where the microphone that was chosen is not plugged in it is
/// [`INPUT_KEPT`] chars of the id and the words that say so.
pub(crate) const SELECT_CHOICE: usize = 32;

/// How many chars of the id of a microphone that is not plugged in its field
/// in the Voice settings keeps before an ellipsis (`ui::shorten`), the words
/// ` (not found)` standing after it. Arithmetic, not measured: the field
/// shows [`SELECT_CHOICE`] chars whole and cuts a longer choice at its end,
/// where these words would be, so the id keeps that less the ellipsis (1),
/// the words (12) and one to spare. An id of as many chars or fewer stands
/// whole.
pub(crate) const INPUT_KEPT: usize = SELECT_CHOICE - 14;

/// How many chars of a Waiting row's meta line stand before its ellipsis
/// (`ui::shorten`). At the narrowest list width, this keeps the longest
/// stage name (`production-approval`), its group and the wait together as
/// they fit in the row; longer stage or group names end cleanly.
pub(crate) const WAITING_SUB: usize = 33;

/// A question line's minimum width in pixels: enough for the asker's
/// source label at the root text size; its words move below the label if
/// both do not fit.
pub(crate) const ASK_QUESTION_MIN: f32 = 200.;

/// The list's width in pixels at the usual text size: direction 1's
/// 352, which at the window's usual 960 wide leaves the pane 544
/// (960 − 64 rail − 352), wide enough that no lever label and no list
/// title in the shot's content is cut. Drawn through rems as the other
/// text-bearing widths are, and capped in [`Footplate::column`] so the
/// pane keeps its 290: at the floor the list narrows to 270 for that,
/// and stands at its full 352 from 706 wide up. [`drawer_fits`] counts
/// with the same rule.
pub(crate) const LIST: f32 = 352.;

/// The list's cap, in pixels, which keeps the reading pane at 290 pixels
/// at the usual text size and grows with the text.
pub(crate) fn list_cap(window: Pixels, rem: f32) -> f32 {
    (f32::from(window) - RAIL - 290. * rem / 16.).max(160.)
}

/// The reading pane's width after the list takes its capped width.
pub(crate) fn pane_width(window: Pixels, rem: f32) -> f32 {
    let list = (LIST * rem / 16.).min(list_cap(window, rem));
    f32::from(window) - RAIL - list
}

/// A list row's lamp, in pixels: the dot before a row's title that says
/// whose turn it is, as the crew's rows have it, and the gap after it,
/// which the lines under the title are indented by so they stand under
/// its first letter.
pub(crate) const ROW_LAMP: f32 = 8.;
pub(crate) const ROW_LAMP_GAP: f32 = 8.;

/// A dealt with row's outcome mark in pixels: Lucide's glyph, at a size
/// that sits in the lamp's room with a gap before the title.
pub(crate) const ROW_MARK: f32 = 12.;

/// The char budgets of a list row's lines, which end in an ellipsis past
/// them (`ui::shorten`): gpui's `truncate()` clips mid-glyph in a row's
/// flex items. At the list's full 352 a title of 34 chars at the label size
/// and a question of 34 at the meta size after its `words 4 ·` fit; a
/// narrower list still clips what is left.
pub(crate) const ROW_TITLE_CHARS: usize = 34;
pub(crate) const ROW_ASK_CHARS: usize = 34;

/// A lever's measures, in pixels where the frame stands fixed and rems
/// where its words do: the key plate's least height is the control's
/// (32, the tokens' `--control-h`), its radius the control's (6), the
/// card's the card's (8). A lever row is one option of an ask card: its
/// plate, its label and its consequence, outlined where it stands
/// chosen or recommended, never filled (the one-fill rule: a card's
/// only fill is its Answer or Approve button). A card's moves are one
/// row of buttons with key chips, at the control's height and radius.
pub(crate) const LEVER_H: f32 = 32.;
pub(crate) const RADIUS_CONTROL: f32 = 6.;
pub(crate) const RADIUS_CARD: f32 = 8.;

/// The tie between two stage dots of a Decide head, in pixels: the
/// board's 14, at a pixel and a half thick, in the strong colour, as
/// the dots are. The dots themselves are [`ROW_LAMP`].
pub(crate) const STAGE_TIE: f32 = 14.;

/// A thread picture's frame, in pixels: the board's 156 by 96, with the
/// control's radius and the strong border, the image scaled inside it.
pub(crate) const THREAD_PIC_W: f32 = 156.;
pub(crate) const THREAD_PIC_H: f32 = 96.;

/// The rail's width in pixels, fixed: room for a label under each
/// icon, as the review has it.
pub(crate) const RAIL: f32 = 64.;

/// The window's narrowest width in pixels, which main.rs sets as its
/// minimum: the 560 the list (capped at 48 hundredths of the window
/// there) and the pane beside it have between them, and the rail beside
/// those, so that no pane gives up room for the rail. The tests of the
/// narrowest window read this number too.
pub(crate) const WINDOW_MIN: f32 = 560. + RAIL;

/// A rail tile's width in pixels: the glyph with its label under it, in
/// the rail with its padding on both sides, 6 + 52 + 6.
pub(crate) const RAIL_TILE: f32 = 52.;

/// A rail label's size in pixels, fixed with its tile.
pub(crate) const RAIL_LABEL: f32 = 10.;

/// How far in from the list's left edge a head's first item stands where
/// the content reaches up into the title bar, in pixels: the list starts
/// at the rail's edge, and the traffic lights run 5 pixels past it, so the
/// item begins where [`titlebar::CLEAR`] says, from the window's left. In
/// pixels as the lights are, whatever the text.
pub(crate) const HEAD_INSET: f32 = titlebar::CLEAR - RAIL;

/// A track's station at its narrowest, in pixels at the usual text
/// size, drawn through [`grown`]: the board's 82, room for the node
/// with the track through it and one bead chip. Stations share the
/// card's width past that, and the track scrolls sideways past the
/// pane's edge.
pub(crate) const TRACK_STATION: f32 = 82.;

/// A quiet flow's name in its one line, in pixels at the usual text
/// size, drawn through [`grown`]: the board's 150, past which the name
/// ends in an ellipsis and the path takes the rest of the row.
pub(crate) const QUIET_NAME: f32 = 150.;

/// How many chars of a quiet flow's name its one line shows before an
/// ellipsis (`ui::shorten`): 20 at the meta size take 140 of the
/// [`QUIET_NAME`] pixels as DejaVu Sans draws them, with the ellipsis
/// past them. The row's `truncate()` cuts what is left where the pane
/// is narrower.
pub(crate) const QUIET_NAME_CHARS: usize = 20;

/// A flow list row's short id in pixels at the usual text size, drawn
/// through [`grown`]: the board's 44, a four-letter headcode in mono
/// with room past it, so the titles start at one place.
pub(crate) const FLOW_ID: f32 = 44.;

/// The chars before an ellipsis in a flow-list row's title and metadata,
/// sized to the pane the row stands in. The fixed width is its padding,
/// mark, id and gaps; the text widths use conservative character advances
/// for the body and meta roles at the widest desktop sans.
pub(crate) fn flow_row_budgets(
    pane: f32,
    rem: f32,
    title_len: usize,
    meta_len: usize,
) -> (usize, usize) {
    let scale = rem / 16.;
    let fixed = 40. + (FLOW_ID + 36.) * scale;
    let available = (pane - fixed).max(0.);
    let title_advance = 8.5 * scale;
    let meta_advance = 7. * scale;
    let title_reserve = (title_len.min(13) + 1) as f32 * title_advance;
    let meta_visible = ((available - title_reserve).max(0.) / meta_advance).floor() as usize;
    let meta_budget = before_ellipsis(meta_len, meta_visible);
    let shown_meta = shown_chars(meta_len, meta_visible);
    let title_visible =
        ((available - shown_meta as f32 * meta_advance).max(0.) / title_advance).floor() as usize;
    (before_ellipsis(title_len, title_visible), meta_budget)
}

/// The chars before an ellipsis in a flow group head's stage and group
/// words. It leaves at least 18 meta chars for the group words, then gives
/// the remaining width to the stage name.
pub(crate) fn flow_group_budgets(
    pane: f32,
    rem: f32,
    stage_len: usize,
    words_len: usize,
) -> (usize, usize) {
    let scale = rem / 16.;
    let fixed = 42. + 20. * scale;
    let title_advance = 8.5 * scale;
    let meta_advance = 7. * scale;
    let stage_visible =
        ((pane - fixed - 18. * meta_advance).max(0.) / title_advance).floor() as usize;
    let shown_stage = shown_chars(stage_len, stage_visible);
    let words_visible = ((pane - fixed - shown_stage as f32 * title_advance).max(0.) / meta_advance)
        .floor() as usize;
    (
        before_ellipsis(stage_len, stage_visible),
        before_ellipsis(words_len, words_visible),
    )
}

/// The chars before an ellipsis in the folded list of empty stages: its
/// count keeps its place, and the remaining width holds their names.
pub(crate) fn flow_empty_budgets(
    pane: f32,
    rem: f32,
    count_len: usize,
    names_len: usize,
) -> (usize, usize) {
    let scale = rem / 16.;
    let fixed = 32. + 36. * scale;
    let title_advance = 8.5 * scale;
    let meta_advance = 7. * scale;
    let count_visible = ((pane - fixed).max(0.) / title_advance).floor() as usize;
    let shown_count = shown_chars(count_len, count_visible);
    let names_visible = ((pane - fixed - shown_count as f32 * title_advance).max(0.) / meta_advance)
        .floor() as usize;
    (
        before_ellipsis(count_len, count_visible),
        before_ellipsis(names_len, names_visible),
    )
}

/// The chars before an ellipsis in one unfolded empty-stage row. The
/// stage has the room past its mark, gaps, padding and "empty" label.
pub(crate) fn flow_empty_stage_budget(pane: f32, rem: f32, stage_len: usize) -> usize {
    let scale = rem / 16.;
    let fixed = 42. + 20. * scale + 5. * 7. * scale;
    let visible = ((pane - fixed).max(0.) / (8.5 * scale)).floor() as usize;
    before_ellipsis(stage_len, visible)
}

/// The chars before an ellipsis in a quiet flow's path on the panel: the
/// room past the panel's padding, the row's padding, the name's box, the
/// row's two gaps and the backlog's words, which stand whole. The path's
/// room moves with the pane, so the budget is figured from its width, as
/// the flow list's are; the row's `truncate()` cuts what is left where the
/// pane is narrower still.
pub(crate) fn flow_quiet_path_budget(
    pane: f32,
    rem: f32,
    path_len: usize,
    backlog_len: usize,
) -> usize {
    let scale = rem / 16.;
    let name = QUIET_NAME.max(QUIET_NAME / 16. * rem);
    let fixed = 2. * rem + 1.5 * rem + name + 1.5 * rem;
    let meta_advance = 7. * scale;
    let visible = ((pane - fixed - backlog_len as f32 * meta_advance).max(0.) / meta_advance)
        .floor() as usize;
    before_ellipsis(path_len, visible)
}

/// The chars before an ellipsis in a directory flow's path on its Line
/// window button: the room past the body's padding (a rem either side)
/// and the button's padding (half a rem either side) and border, less
/// the label's other words, which stand whole. The button takes its own
/// wrapped row at most, so the budget is figured from the pane's width,
/// as the flow list's are; widen the window and the cut path stands
/// whole again.
pub(crate) fn line_flow_dir_budget(pane: f32, rem: f32, words_len: usize, dir_len: usize) -> usize {
    let scale = rem / 16.;
    let fixed = 3. * rem + 2.;
    let advance = 8.5 * scale;
    let visible = ((pane - fixed).max(0.) / advance).floor() as usize;
    before_ellipsis(
        dir_len,
        visible.saturating_sub(words_len.saturating_sub(dir_len)),
    )
}

/// The fewest readable chars a track head's flow name keeps: where the
/// budget with the chip falls below this and the name is cut, the chip
/// yields and the name is budgeted without it, since the name identifies
/// where the chip classifies. Four names the flow where an ellipsis
/// alone would not.
pub(crate) const PANEL_HEAD_NAME_FLOOR: usize = 4;

/// The chars before an ellipsis in a track head's flow name: the room past
/// the panel's and the card's padding, the row's gaps, the chip and the
/// meta, which stand whole — the head holds one line, and the name is what
/// clips. `chip_len` is 0 where the head draws no chip, and counts no box
/// then. The row's `truncate()` cuts what is left where the pane is
/// narrower still.
pub(crate) fn panel_head_name_budget(
    pane: f32,
    rem: f32,
    name_len: usize,
    chip_len: usize,
    status_len: usize,
    backlog_len: usize,
) -> usize {
    let scale = rem / 16.;
    // The panel's padding either side, the card's padding and border, and
    // the row's gaps: three where the chip stands, two where it does not.
    let gaps = match chip_len {
        0 => 2.,
        _ => 3.,
    } * 0.5
        * rem;
    let fixed = 2. * rem + 1.5 * rem + 2. + gaps;
    // The chip's border and padding with its words, which stand in mono
    // where the window has one: Menlo and DejaVu Sans Mono advance a hair
    // past 7 pixels a char at the code size, and 7.5 covers them.
    let chip = match chip_len {
        0 => 0.,
        _ => 2. + 0.5 * rem + chip_len as f32 * 7.5 * scale,
    };
    let meta = (status_len + backlog_len) as f32 * 7. * scale;
    let visible = ((pane - fixed - chip - meta).max(0.) / (8.5 * scale)).floor() as usize;
    before_ellipsis(name_len, visible)
}

/// Whether the flow strip's stations run past the pane's edge: their knots
/// and ties with their labels' chars, against the strip's inner width. The
/// labels' advance runs under typical text, with a margin past the edge, so
/// the overflow mark stands only where the track plainly runs on: a strip
/// past the edge by a few pixels shows no mark, and a fitting one never
/// grows one.
pub(crate) fn flow_strip_overflows(pane: f32, rem: f32, labels: &[usize]) -> bool {
    if labels.is_empty() {
        return false;
    }
    let scale = rem / 16.;
    let stations: f32 = labels
        .iter()
        .map(|chars| 10. + 0.375 * rem + *chars as f32 * 6. * scale)
        .sum();
    let ties = (labels.len() - 1) as f32 * (20. + 0.75 * rem);
    stations + ties > pane - 2. * rem + 16. * scale
}

/// One Line window track cell's width in pixels, fixed: the stage's
/// name, who serves it and its open count, whatever the text.
pub(crate) const LINE_TRACK_CELL: f32 = 132.;

/// Whether the Line window's stage track runs past the pane's edge: its
/// cells' fixed widths against the body's inner width. The margin past
/// the edge is the panel strip's, so the overflow mark stands only where
/// the track plainly runs on, never for a few pixels.
pub(crate) fn line_track_overflows(pane: f32, rem: f32, stages: usize) -> bool {
    if stages == 0 {
        return false;
    }
    let scale = rem / 16.;
    stages as f32 * LINE_TRACK_CELL > pane - 2. * rem + 16. * scale
}

fn before_ellipsis(length: usize, visible: usize) -> usize {
    if length > visible {
        visible.saturating_sub(1)
    } else {
        length
    }
}

fn shown_chars(length: usize, visible: usize) -> usize {
    if length > visible {
        visible.max(1)
    } else {
        length
    }
}

/// The switcher's list in rems, drawn once: its narrowest and its widest.
/// They are in rems because the hint line in it is: a list in pixels cut
/// its words at the larger text sizes. Sized for the footer's hint line
/// `↑↓ move · ↵ open · esc close · + add a yard · ? all keys`, as DejaVu
/// Sans draws it (the widest sans of the desktops here, and the font the
/// Linux shots draw in), plus the footer's padding and the list's border,
/// at the usual 16-pixel rem: the line, 340 pixels, between 12 of padding
/// each side with a 1-pixel border. The widest, 480, holds it on one line
/// at the usual size with every row's ctrl+N whole; the narrowest, 320,
/// leaves room to shrink, the hint line eliding, at a larger text size or
/// a narrow window. Where the window's width less the rail is narrower
/// still, the list is capped at that.
///
/// The test beside the switcher's rows holds the hint and every key
/// inside at the usual size and the largest.
pub(crate) const SWITCHER_MIN: f32 = 20.;
pub(crate) const SWITCHER_MAX: f32 = 30.;

/// A line view's stage column in pixels at the usual text size, and no
/// narrower: the panel's track straightened, one column per stage in
/// travel order. Drawn through [`grown`], so it stands 176 wide at the
/// usual size and grows with the text above it.
pub(crate) const LINE_COLUMN: f32 = 176.;

/// A line view's column head in rems: its name with three lines of group
/// words under it, bottom-aligned, so a column whose group wraps onto a
/// second or third line puts its rule level with a shorter group. The
/// row's rules make one straight track, as the panel's heads have it.
/// A name line with three group lines at gpui's line height,
/// 1.42 with 3 × 1.21, rounded up. A group that wants a fourth line
/// makes its own column taller, as a panel head does.
pub(crate) const LINE_HEAD: f32 = 5.2;

/// The departures board's columns in pixels at the usual text size,
/// drawn once: the expected arrival in mono, the bead's title with its
/// headcode, the platform, the line, and the remark. Drawn through
/// [`grown`], so each stands its width at the usual size and grows with
/// the text above it. Past the pane's edge the board scrolls sideways;
/// the panel fits instead. The line holds this yard's longest depot
/// names: 88 cut `yardr-hosted` even at the usual size, so 92.
pub(crate) const BOARD_EXPECTED: f32 = 80.;
pub(crate) const BOARD_BEAD: f32 = 280.;
pub(crate) const BOARD_PLATFORM: f32 = 120.;
pub(crate) const BOARD_LINE: f32 = 92.;
pub(crate) const BOARD_REMARK: f32 = 220.;

/// A column `usual` pixels wide at the usual text size: that wide at
/// smaller sizes too, and growing with the text above it, since the
/// words in it stand in rems. A column in pixels alone cut its words at
/// the larger text sizes; in rems alone it would narrow past what the
/// design draws. `usual` over the usual 16-pixel rem is its width.
pub(crate) fn grown(usual: f32) -> Div {
    div().w(rems(usual / 16.)).min_w(px(usual))
}

/// The bead drawer's width in pixels at the usual text size, drawn
/// through [`grown`]: fixed, and growing with the text above it, as a
/// line view's columns do.
pub(crate) const DRAWER_WIDTH: f32 = 320.;

/// A context drawer's width in pixels at the usual text size, drawn
/// through [`grown`]: the board's 440, room for a stage file's lines
/// and a prompt's beside the pane.
pub(crate) const CONTEXT_WIDTH: f32 = 440.;

/// Whether the window `window` pixels wide holds a context drawer
/// beside the reading pane: the pane keeps at least the drawer's own
/// width past it, as with the bead drawer. Where it does not, the
/// drawer stands over the pane's right instead: at 960 the pane is 544
/// and the drawer stands over it, at 1440 it stands beside it. `rem`
/// is the window's rem in pixels.
pub(crate) fn context_beside(window: Pixels, rem: f32) -> bool {
    let drawer = CONTEXT_WIDTH.max(CONTEXT_WIDTH / 16. * rem);
    let list = (LIST * rem / 16.).min((f32::from(window) - RAIL - 290. * rem / 16.).max(0.));
    f32::from(window) - RAIL - list - drawer >= drawer
}

/// Whether the window `window` pixels wide holds the bead drawer beside
/// the reading pane: the pane keeps at least the drawer's own width. The
/// rail is [`RAIL`] pixels, the list [`LIST`] at the usual text size and
/// capped as [`Footplate::column`] caps it (the pane keeps its 290,
/// grown), and the drawer its grown width; `rem` is the window's rem in
/// pixels.
pub(crate) fn drawer_fits(window: Pixels, rem: f32) -> bool {
    let drawer = DRAWER_WIDTH.max(DRAWER_WIDTH / 16. * rem);
    let list = (LIST * rem / 16.).min((f32::from(window) - RAIL - 290. * rem / 16.).max(0.));
    f32::from(window) - RAIL - list - drawer >= drawer
}

/// How many backlog titles a line view's backlog column lists before
/// "+N more".
pub(crate) const BACKLOG_TITLES: usize = 6;

/// The conversation's reading width: the mail, the notice and the
/// composer stand full width up to 880 pixels, centred in the pane.
pub(crate) const READING_WIDTH: f32 = 880.;

/// How far above and below its view the conversation's list measures the
/// rows it does not draw, in pixels: once measured, a row's height is kept,
/// so a view that moves a page finds the rows it moves over measured, and
/// what the page keys and the place a view is put back to count in heights
/// is exact over that distance. More than a half of the tallest view the
/// window has (`reach_top` reasons from it) and a page of it.
pub(crate) const MAIL_OVERDRAW: f32 = 1500.;

/// How many rows above the view still count as near the top of the
/// conversation, where its older mail is asked for: the view is within
/// half its height of the top and no more rows than these stand above it
/// (rows that are not measured count no height, and the rows above a view
/// that stands far down are not all measured).
pub(crate) const MAIL_NEAR_TOP: usize = 12;

/// How far a letter's body stands in from the reading column's left,
/// in rems: past the head's clock, so the clock's place is a gutter
/// the letter's own time stands in while the pointer is on it. The
/// clock says five characters at the text size; 3.5 rems clears the
/// widest of them with room past it, and the words stand roughly
/// under the sender's name. In rems because the clock in it is: a
/// gutter in pixels cut the time at the larger text sizes.
pub(crate) const LETTER_GUTTER: f32 = 3.5;

#[cfg(test)]
mod tests {
    use super::super::ui::shorten;
    use super::*;

    #[test]
    fn flow_pane_width_tracks_the_list_cap() {
        assert_eq!(list_cap(px(624.), 16.), 270.);
        assert_eq!(pane_width(px(624.), 16.), 290.);
        assert_eq!(pane_width(px(960.), 16.), 544.);
    }

    #[test]
    fn a_context_drawer_stands_beside_a_wide_pane_and_over_a_narrow_one() {
        assert!(context_beside(px(1440.), 16.));
        assert!(!context_beside(px(960.), 16.));
        assert!(!context_beside(px(624.), 16.));
    }

    #[test]
    fn flow_rows_shorten_to_the_room_and_keep_full_text_when_wide() {
        let title = "The sign-in's password label";
        let meta = "moving · 3 hours";
        let (title_budget, meta_budget) = flow_row_budgets(
            pane_width(px(624.), 16.),
            16.,
            title.chars().count(),
            meta.chars().count(),
        );
        assert_eq!(shorten(title, title_budget), "The sign-in's…");
        assert!(shorten(meta, meta_budget).ends_with('…'));

        let (title_budget, meta_budget) = flow_row_budgets(
            pane_width(px(960.), 16.),
            16.,
            title.chars().count(),
            meta.chars().count(),
        );
        assert_eq!(shorten(title, title_budget), title);
        assert_eq!(shorten(meta, meta_budget), meta);
    }

    #[test]
    fn flow_group_and_empty_stage_names_shorten_at_the_pane_edge() {
        let stage = "review-security";
        let words = "2 · mckean · waiting on a person · 0 of 1 seats";
        let (stage_budget, words_budget) = flow_group_budgets(290., 16., stage.len(), words.len());
        assert_eq!(shorten(stage, stage_budget), "review-secu…");
        assert!(shorten(words, words_budget).ends_with('…'));

        let names = "critique · ready · review-security · approved";
        let (count_budget, names_budget) = flow_empty_budgets(290., 16., 14, names.len());
        assert_eq!(shorten("6 empty stages", count_budget), "6 empty stages");
        assert!(shorten(names, names_budget).ends_with('…'));
        assert_eq!(
            shorten(stage, flow_empty_stage_budget(290., 16., stage.len())),
            stage
        );
    }

    #[test]
    fn narrow_panes_cut_the_quiet_path_and_the_head_and_mark_the_strip() {
        // Arrivals at 960: the quiet release path ends with an ellipsis,
        // and stands whole where the pane holds it.
        let path = "build › sign › decide › verify › publish › published";
        let budget = flow_quiet_path_budget(
            pane_width(px(960.), 16.),
            16.,
            path.chars().count(),
            "3 backlog".len(),
        );
        assert_eq!(shorten(path, budget), "build › sign › decide › verify › p…");
        let budget = flow_quiet_path_budget(
            pane_width(px(1440.), 16.),
            16.,
            path.chars().count(),
            "3 backlog".len(),
        );
        assert_eq!(shorten(path, budget), path);
        // Arrivals at 624: the head's name with the chip falls below
        // its floor, so the chip yields and the name is budgeted without
        // it, holding the floor; the meta stands whole throughout.
        let budget = panel_head_name_budget(
            pane_width(px(624.), 16.),
            16.,
            "default".len(),
            "store".len(),
            "2 wait on you".len(),
            "· 2 backlog".chars().count(),
        );
        assert!(budget < PANEL_HEAD_NAME_FLOOR);
        let budget = panel_head_name_budget(
            pane_width(px(624.), 16.),
            16.,
            "default".len(),
            0,
            "2 wait on you".len(),
            "· 2 backlog".chars().count(),
        );
        assert!(budget >= PANEL_HEAD_NAME_FLOOR);
        assert_eq!(shorten("default", budget), "defa…");
        // At 960 the budget with the chip holds the name whole, so the
        // chip stands.
        let budget = panel_head_name_budget(
            pane_width(px(960.), 16.),
            16.,
            "default".len(),
            "store".len(),
            "2 wait on you".len(),
            "· 2 backlog".chars().count(),
        );
        assert!(budget >= PANEL_HEAD_NAME_FLOOR);
        assert_eq!(shorten("default", budget), "default");
        // The flow list: the nine-stage track runs past the narrow pane
        // and the one-stage one stands whole; at 960 the five-stage track
        // fits, and at 624 it runs past.
        let release = [8, 4, 6, 6, 7, 9, 4, 4, 6];
        assert!(flow_strip_overflows(
            pane_width(px(624.), 16.),
            16.,
            &release
        ));
        assert!(!flow_strip_overflows(pane_width(px(624.), 16.), 16., &[7]));
        let default = [7, 6, 10, 19, 8];
        assert!(!flow_strip_overflows(
            pane_width(px(960.), 16.),
            16.,
            &default
        ));
        assert!(flow_strip_overflows(
            pane_width(px(624.), 16.),
            16.,
            &default
        ));
        assert!(!flow_strip_overflows(pane_width(px(624.), 16.), 16., &[]));
        // The Line window: the dirflow button's path ends in an ellipsis
        // at 624 and stands whole at 960; the three-cell track runs past
        // the narrow pane and fits the wide one, and one cell, or none,
        // never marks.
        let words = "dirflow (repo (directory) /home/jo/notes)";
        let dir = "/home/jo/notes";
        let budget = line_flow_dir_budget(
            pane_width(px(624.), 16.),
            16.,
            words.chars().count(),
            dir.chars().count(),
        );
        assert_eq!(budget, 0);
        assert_eq!(shorten(dir, budget), "…");
        let budget = line_flow_dir_budget(
            pane_width(px(960.), 16.),
            16.,
            words.chars().count(),
            dir.chars().count(),
        );
        assert_eq!(shorten(dir, budget), dir);
        assert!(line_track_overflows(pane_width(px(624.), 16.), 16., 3));
        assert!(!line_track_overflows(pane_width(px(960.), 16.), 16., 3));
        assert!(!line_track_overflows(pane_width(px(624.), 16.), 16., 1));
        assert!(!line_track_overflows(pane_width(px(624.), 16.), 16., 0));
    }

    #[test]
    fn flow_budgets_account_for_the_ellipsis_at_large_text_sizes() {
        let (stage_budget, words_budget) = flow_group_budgets(400., 36., 15, 48);
        assert!(stage_budget <= 15);
        assert!(words_budget < 48);

        let (title_budget, meta_budget) = flow_row_budgets(400., 36., 28, 16);
        assert!(title_budget < 28);
        assert!(meta_budget < 16);
    }
}
