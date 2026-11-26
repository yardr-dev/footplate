# The look

What the window is drawn with: its palette, its type scale, its spacing
and the row and card patterns every screen builds against. The values
here are the built ones, read from `src/theme.rs`,
`src/window/metrics.rs` and the drawing code they name; the mockups in
yardr-design are the direction, never the source of a number. Part of
footplate's manual; the [README](../README.md) is the front door.

## Palette

Fifteen tokens, the design system's whole (`tokens/colors.css` in
yardr-design), in a dark and a light look. The window's look is
`Look::Auto` (the desktop's, as it changes), `Look::Light` or
`Look::Dark`; the palette for it is `Palette::of` in `src/theme.rs`.
No screen names a colour of its own.

| Token | Dark | Light | Stands for |
| --- | --- | --- | --- |
| ground | `#161718` | `#F7F7F6` | the list's and the conversation's ground |
| side | `#101112` | `#EDEDEB` | the rail, a field, a run or setup block, a stranger's message |
| raised | `#1C1D1F` | `#FFFFFF` | a decided card's frame, a secondary button |
| line | `#27282B` | `#E2E2DF` | hairlines: bands, the rules between asks, field borders |
| strong | `#333538` | `#CFCFCB` | a secondary button's edge |
| open | `#2A2B2E` | `#E2E2DF` | the open row, filled |
| text | `#E3E5E8` | `#22242A` | the primary text |
| faint | `#ABADB0` | `#5B5E64` | quiet words, counts, metadata |
| accent | `#D3602F` | `#D3602F` | what waits on the person reading: a state, never a button |
| on accent | `#161718` | `#161718` | text on the accent |
| button | `#B54E22` | `#B54E22` | the one filled button's fill, white on it |
| text accent | `#E2794A` | `#8F3C19` | yard-rust text at contrast: waiting words, the recommended lever's plate and tag |
| ok | `#4CC38A` | `#1F9A60` | the running dot, a working lamp, an approval's mark |
| link | `#7AA2F7` | `#1A56C8` | links, in messages and ask and bead ids; a text control beside what it acts on (`Copy`, `Read whole`, `Show all`) |
| peer | `#B693F5` | `#6440B8` | a linked name's peer suffix (`@airy`) |

A button and a state never share a fill: the filled button is the
deeper rust (`button`, hover `#C0552A`, pressed `#8F3C19`) and the
accent at the brighter step fills only states — the chosen segment,
the checked box, the rail's waiting badge. Green is only the running
dot, a crew member's working lamp and an approval's mark in the
history; red (`#E5675A` dark, `#B3382C` light) is only errors and the
states that say a thing failed.

## Type scale

The design tokens' roles (`tokens/typography.css`), at the window's
one rem, so the text setting scales every role together. At the usual
16-pixel rem: question 22, title 15, section 13, label 13, body 13,
meta 12, code 12. The constants are `TYPE_QUESTION`, `TYPE_TITLE`,
`TYPE_SECTION`, `TYPE_LABEL`, `TYPE_BODY`, `TYPE_META` and
`TYPE_CODE` in `src/theme.rs`. The question is direction 2's `.q`
(22/600), the one text the card owns the pane for.

| Role | Size | Weight | Stands for |
| --- | --- | --- | --- |
| question | 22 | semibold | the current ask's question |
| title | 15 | semibold | another ask's question |
| section | 13 | semibold | a section's or a chat group's head |
| label | 13 | medium | a lever's or a move button's label, a row's head (semibold on an open ask's row), the decision pane's bead title |
| body | 13 | regular | a card's context lines |
| meta | 12 | regular | counts, metadata, a lever's plate and consequence, a move button's key chip, the line over a question |
| code | 12 | mono | commands, and a row's short id |

The mono face is the machine's first of Menlo, Consolas, DejaVu Sans
Mono (`select::mono`); no other face is set anywhere. The window's root
text stands at the person's text setting over this scale; the rail's
labels are fixed at 10 pixels, as the rail carries no reading text.

## Spacing and measures

Fixed in pixels where the frame stands fixed, in rems where the words
in it set the size (`src/window/metrics.rs`).

- The window opens at 960×640; its floor is 624×360 (560 and the
  rail), and the rail is 64 wide with 52-wide tiles.
- The list is 352 wide at the usual text size, capped so the pane
  beside it keeps 290: at 960 the pane is 544, and at the floor the
  list narrows to 270 to keep the pane. A lever label and a list title
  in the shot's content stand whole at 352.
- An open ask's card stands flat on the pane, 10px between its rows;
  each card after the first is ruled off from the one above by a 1px
  `line`, with 16px either side of it. A decided card keeps the raised,
  bordered frame: raised ground, a 1px `line` border, an 8px radius
  (`RADIUS_CARD`) and 16px of padding.
- A list row's lamp is 8 (`ROW_LAMP`) with 8 after it
  (`ROW_LAMP_GAP`); the lines under a row's head are indented by the two,
  so they stand under its first letter. A decided row's mark is 12
  (`ROW_MARK`) in the same room.
- A row's lines end in an ellipsis past a char budget (`ui::shorten`):
  34 for a head (`ROW_TITLE_CHARS`) and 34 for an ask's question
  (`ROW_ASK_CHARS`), which fit the list at 352. gpui's `truncate()` clips
  mid-glyph in a row's flex items, so it is only the backstop where the
  list is narrower.
- A lever: at least 32 high (`LEVER_H`, the control's height), a 6px
  radius (`RADIUS_CONTROL`), 8px of padding at its sides.
- A card's moves stand in one row of buttons with key chips, at the
  control's height and radius; the row wraps where the pane is narrower.
- A Decide head's stage dots are 8 (`ROW_LAMP`) with 14px ties
  (`STAGE_TIE`) between them, and the stage named beside them in the
  meta role.
- A thread picture stands in a 156 by 96 frame (`THREAD_PIC_W`,
  `THREAD_PIC_H`), the control's radius, the strong border, the side
  ground, the image scaled inside it.
- Controls stand at least 32 high; a row's padding is 8 by 6; a
  section head stands at least 32 high.
- The conversation's reading column stands full width up to 880,
  centred in the pane.

## Rows

One shape for every list's row: a lamp dot of 8px where the row carries
a state (green working, rust waiting on you, red failed, grey at rest),
or the room for one; the head in the label role, one line, ended in an
ellipsis; under it, past the lamp's room, a line for each further fact
in the meta role and quiet; and at the end of the last line a short id
in the mono face at the code size, or a count in quiet words.

- An open ask's row is its bead: the rust lamp and the bead's title,
  semibold, then a line for each open ask (what it wants and its number
  in the text colour, `run 1`, `pick 2`, `words 4`, then the question's
  first line), then who asks, each once, and how long the oldest ask
  waits (`yardmaster · 12m`), with the bead's short id.
- A parked bead's row has no lamp, its title under the open rows'
  titles, then its stage, group and age (`pick · design-pickers · 20m`).
- A decided row has its outcome's mark in the lamp's room: Lucide's
  glyph at one stroke (the design system allows no unicode character as
  an icon), `check` approved in green, `message-square` words or a secret
  given, `undo-2` taken back, `clock` put aside, `x` declined, the rest
  quiet. Then the question's first line, and the outcome in the text
  colour with whose and when quiet after it. A fold of look-alikes
  (same day, asker and outcome, questions differing only in numbers)
  stands in the same shape, its count (`×4 ▸`) where the id stands.

The open row is filled with `open`; a row under the pointer is told by
its hairline alone, so exactly one row ever reads open. A section head
is its title in the section role with its count in quiet words at the
right (`2 to decide`, `none waiting`); counts are words, never a
numeral badge, the yard's notices alone excepted. A topic's head
stands over the open rows only where two asks or more share it beside
another topic.

## Cards and the one-fill rule

The decision pane's head is the yard's dot with the yard, the bead,
its topic where it has one and who built and reviewed it where either
is known in the meta role and quiet, over the bead's title in the label
role; beside them the stage dots of its flow with the stage named, and
a pill counting the bead's asks on the reader (`2 asks on you`). Under
it the bead's asks stand as flat cards, ruled apart. Each
card: the yard's weight lines in text accent; a line over the question
in the meta role, `Waiting on you · run 1` in text accent, then how long
and who asks quiet (`asked 12m · yardmaster`), and at its right the ask
line (`footplate-q2rd · ask 1`, to select and copy) and the bead whole's
disclosure (`▸ details (e)`); the question (the current ask's in the
question role, the rest in the title role); the context in the body
role; then the ask's own body, its field, its files and its refusal, the
action row and the keys line. A run or setup ask's question and context
lines each carry the asker's label (`yardmaster says:`), quiet in the
faint colour at the regular weight: every line the asker wrote is marked
as theirs, so none reads as the yard's.

Under all the bead's cards stands its thread: the bead's notes newest
first (as in the card's details and the bead whole in Lines; the crew's
conversation alone reads oldest first), each headed by who wrote it, its
time and the bead, its words beside its image where it has one (156 by
96 in its frame), its other files under its words as one quiet line of
name and size each, their Open and run on the bead whole behind the
disclosure. A short card and its thread stand together at 960×640.

A pick ask's options are levers: a key plate holding its number alone,
the label and its consequence, so every label stands at the same place.
The chosen lever alone is outlined in the accent; the recommended one is
chosen until another is picked, so two outlines never stand at once.
The recommended lever's plate is in text accent with an accent edge, the
word `recommended` keeps a column at its row's right in text accent, and
its why stands under its label inside the lever. No option lever is ever
filled.

One filled control per ask: the card's only fill is its Answer button
(a pick or words ask) or its Approve button (a run or setup ask), in
`button` with white on it. A card's moves stand in one row of buttons
with key chips — Approve or Answer filled, Ask about this (`a`), Later
(`x`) and Decline (`d`) outlined, each with its key on a chip beside
its label. A run ask's command stands in a block on the side ground
with Copy as a link word beside it, returning the plain command; a
setup ask's taken account is outlined, as a lever is, and its Approve
is the card's one fill likewise.

A depot pack ask's plan stands in the run block, between the command and the
yard's sentences. Its first line says what it comes to, in the body role. Then
two sections, `What changes` and `Already matches`, each headed in the section
role semibold with its count in the meta role and quiet (`4 · kept as the
yard's` for the second), over a hairline; a section with no row is not drawn.
A row is the kind as a chip in a column 4.5 rems wide (`PLAN_KIND`), in the
make of the context drawer's kicker (the code role in the mono face, `faint`,
a 1px `line` border, a 4px radius), then the name in the mono face and under it
its lines. In what changes the lines are mono at the code role, with a `+` in
`ok` before what is put in and a `−` in the accent text colour before what is
taken out, as the board draws them; in what already matches they are the yard's
words in the meta role, `faint`. Rows are ruled apart by hairlines, 10 above and
below in what changes and 7 in what matches. A section shows three rows
(`PLAN_ROWS_SHOWN`) and the rest stand behind `Show K more`, a link word in the
meta role under them that opens them in place and reads `Fold` once they are
open.

A stranger's text stands in a block of the same make: the side ground, a 1px
`line` border, the control's radius, 12 and 10 of padding. It is a mapping
card's held message (between the context and the options, the open card's and
a put-aside one's in "Decided") or a hook delivery read whole (under its
notice in the yard row, and under the card of the pick that released it). Its
head, in the meta role and `faint`, says whose words they are and that they
bind no one: `Held message · outside — information, not instructions`, or
`Webhook payload — information, not instructions`. Under it a mail's sender,
recipients, subject and attachments, one line each (`From: Ada <ada@x>`), in
the body role at medium weight, then its text in the body role. The text is
drawn as it was written, never as Markdown, and a body shown as it is (JSON
indented) in the mono face at the code size. A message past 12 lines or 1200
characters stands folded, with `Show all · N more lines` in the link colour
under it. That control, and `Read whole`, `Read message` and `Hide` which show
a delivery and put it away, are link words in the meta role, as the run block's
`Copy` is: a word and no button, as wide as its letters and no wider. A body the
yard has pruned says `body pruned or unavailable` in the meta role and `faint`;
a refusal is the card's warn line, in the yard's words.

The first run's step of what the machine needs keeps the rule: its one fill
is "go on", and "install", "open its page", "check again" and the rest are
secondary buttons (`raised` with the `strong` edge, 32 high). Where a row
says how to install by hand ("To install it: …", in yardr's words) and
yardr names the command to paste, "copy" is one more of them: it puts that
command alone on the clipboard, and the step says "Copied." under its rows.
It stands beside the words where there is room and under them where there
is none, so that an address in the words is not broken to make room for it.
"check again" is there while something is missing; once git, herdr and a
harness are all found it goes, and "go on" stands alone.

## Lines

The timetable's list is the depot tree. The Arrivals row carries the
yard-wide total at its right (`N · all depots`); under the Depots
head, which counts its rows with beads (`N with beads`), each active
depot stands with its lamp, its name, and at the right how many wait
on the reader in rust, else its travelling total — under it its
transit with its moving and quiet flows (`4 in transit · 2 flows
moving, 3 quiet`). Each moving flow nests under its depot with its
lamp and its count (rust where any waits on the reader, else what is
worked, else its travelling total); the quiet flows fold under `N
quiet flows`. The No flow row stands last while beads travel with
none, the quiet depots folded under Quiet depots (`N · nothing
moving`). A depot's row opens the depot's board, a flow's its list.

The arrivals head is `Arrivals` in the title role with `N in transit
across all flows` under it and the Panel/Departures segmented control
at its right, then the stats, each a 28-high pill only where any
stands: what is worked (the running dot before it, split by kind
where two or more work, the working groups with their seats on
hover), what waits to decide in rust with the accent edge and to move
after it, each opening Decide. The departures board under the toggle
keeps its columns: Expected, Bead, Platform, Line, Remark.

The panel opens with the legend of the five nodes, then one section
per depot with beads in transit. Each moving flow is a track: its
head above its stations — the flow's name with its type (`type X`)
or source chip, who it waits on with its backlog counted apart — one
station per stage but the backlog's in travel order, each a node on
the track's line with its name and its beads' chips under it (two,
then `+N`). The card reads rust-edged where any of its beads waits
on the reader. Each quiet flow is one line of its path (`a › b › T1
/ T2`, the backlog left out) with its backlog; the Elsewhere line
stands where beads stand in no stage of the depot's flows. The beads
travelling with no flow stand in their own section with its line,
which opens their list. Stations share the card's width down to 82
(`TRACK_STATION`), past which the track scrolls sideways.

A station's node is 14 wide: the plain crew dot, the hollow people
ring, the working ring, the rust fill waiting on the reader, the end
square. A bead's chip is a 20-high pill with its short handle in
mono: rust filled where it waits on the reader, semibold where
worked, red where held or stalled. A quiet flow's name holds 150
(`QUIET_NAME`) with its path past it.

A flow's pane opens with the depot, the flow under it and the flow's
source, with the List/Board toggle at the right. Under it the
stages' strip — each but the backlog's a 10-wide knot on the track
with its name and the stages with beads named with their count —
scrolling sideways past the pane's edge; `Grouped by stage · moving a
bead goes through a Decide card`; then one group per stage with beads,
each head with its count and its group words (the group's name, then
what it waits for: `waiting on a person`, a siding's `1 of 4 to fold`
— this depot's beads against its fold — or a served group's `2 of 3
seats`; `paused` and `no route matches` say why a stage cannot move),
the backlog's rows with the stage past them (`→ next`), and the served
landing rows under the flow's landed end with their elapsed time. Empty
stages fold to one row naming them. A row is 40 high: the
mark (the working dot, the rust dot, the backlog's ring, the landed
check), the short id in mono (`FLOW_ID`), the title, and at the
right who works it with how long it has stood. The board under the
toggle is the depot board's columns for the one flow. In a narrow pane,
long titles, stage names, group words and metadata end with an ellipsis;
their character budgets grow with the pane so the text stays inside it.

A press on a station opens the stage's context drawer at the window's
right, over the Lines views. It is 440 wide (`CONTEXT_WIDTH`), grown
with the text: beside the pane where the pane keeps 440 past it (at
1440), over the pane's right where it does not (at 960), the list
staying in reach either way. Its head is 52 high: the `stage` kicker
chip in the code role, the depot, the flow and the stage in the label
role semibold (the type standing between depot and flow where the flow
is typed), and the drawers' ×. Under it the three tabs stand 34 high
in the meta role medium and quiet; the open one reads in the text
colour over a 2px accent edge.

Each tab opens with the file line: the path in the code role over one
meta line and quiet — the depot and the flow's source for a stage
file, the source, the lines counted and whose sessions for a prompt —
ruled off below. The file renders as the window's Markdown in the
body role, folded past 30 lines (`FOLDED_LINES`) behind `N more lines
· Show the whole file`, the link word in `link`. The Group tab's facts
stand under 96-wide quiet labels: the runner with its flow-map word
(`herdr · agents`), the harness with its model, a script's command, a
manual group's members or another yard's name, the seats (`1 · 0
taken`, from the pressed station's own stage, left out where the yard
serves neither), further members, and the prompt's path in code (or
why the group reads none). The chain past its first group stands one
meta line per group (`then …`, `where …` past it). Served by… stands
as a link word past the serving line where a group serves the stage,
and as a button with its note where none does; the pick marks the
group and Send mails its route card, or the depot's set-aside card for
a yard-wide rule, as the Line window's picker does. No route reads as a warn
with the accent edge, and a stage that closes the bead as the same
shape muted, with no pick. The foot holds Copy path as a link word
that returns the path whole; on a prompt, Propose an edit stands
before it with the note of what an edit does, and the Line window's
editor and hand-off stand under the prompt, the foot going while the
editor stands.

The Stage tab, and the Line window's stage file under its own head, share
one section for where the text comes from and what can be done with it
(`stagefile.rs`). Each line is quiet in the meta role (the Line window's
small text) except two. The yard's warning of a broken `## Ask` stands
first, right under the file line, in `warn`: its words are the yard's.
Then one line says where the text comes from (`store flow`, `from pack X:
you change the yard's copy, applying the pack again keeps it`, `from
repository (repo@abcdef0): copy this flow into the yard before editing its
stages`). Where the yard says the store flow differs from the
repository's, one line more follows it, in the accent text colour and
never in `warn`, since a difference is no error (`differs from the
repository's file: …` on the stage whose file differs, `this flow
differs from the repository's: …` on any other stage of the flow). Then
the controls as small buttons, 32 high with the strong
edge: Edit, or Add stage file where none stands, and Restore prior text…
where the yard kept the text before the last save; Copy flow into yard…
in place of them on a repository's stage. While the editor stands over
the stage the controls give way to it, and the file's rendered text with
them.

The editor is the prompt editor's field (the `side` ground, a hairline,
8 rems high at the least) with one quiet line of what Save… does, and
Cancel and Save…, the button held (outlined, in the disabled text) while
there is nothing to save. Save… opens the review in its place: `Review change` in the label
role semibold, a quiet count (`1 line taken out, 2 lines put in`), the
change in a box on the `raised` ground with a hairline and a 6px radius,
and under it the weight in the accent text colour, as a card's head
says it (`WEIGHTY · code: …`), never in red. The change is set in the
code role, one row a line with a 10-wide marker before it: `-` and the
line in `warn` for what is taken out, `+` and the line in `ok` for what is
put in, and what stands round them in quiet text, three lines either
side of a change and a longer run as one quiet row (`12 unchanged
lines`). Where the system offers no unlock, the stage's name is typed
back in a field under the weight, with the words that ask for it above
it. Cancel, Back to edit and Confirm save stand under it, the last the
filled button. A refusal and a name that was not typed back stand over
the buttons in `warn`; a file that moved says so in quiet text.
What the last save or copy left stands under the section as one quiet
line, or in `warn` where the yard warned of the saved file's `## Ask`.
Copy flow into yard… opens its confirmation the same way: what it copies,
the weight, the depot's name where no unlock is offered, and Cancel and
Confirm copy.

## Crew

A crew member's conversation head carries a Context button beside its
screen, on the operator's window alone: 28 high, 10 of padding at its
sides, a 6px radius, the strong edge, the file-text glyph at 14 with
`Context` in the meta role medium and quiet. While the drawer stands
on the conversation the button reads in the text colour over the
accent's edge.

The press opens the member's context drawer: the stage drawer's shell
with two tabs. Its head is the `crew` kicker chip with `{member}'s
context`; its tabs are Role and Prime, the Prime tab with how many
items it lists once the yard has answered (`Prime · 4`, `Prime · 3`
where the prime carries none of the operator's words; the word alone
while the prime is read, and where it is not).

The Role tab opens with the file line — the path in the code role, a
digest in a file name said by its first twelve, over `depot X · N
lines · read when its session starts` — and the role as the window's
Markdown, folded past 30 lines behind `N more lines · Show the whole
file`. Where the read has no role prompt it says `No role prompt.`
The foot holds Propose an edit with the note of what an edit does and
Copy path; the Line window's editor and hand-off stand under the role,
the foot going while the editor stands.

The Prime tab opens with what the member's session reads when it
starts, in order, with the yard's command for the whole (`yardr prime
--crew X`); under it the yard's Prime answer stands as numbered items —
the 18-wide number in the code role and quiet, its title in the meta
role medium, its value in code, what it says of and whose in quiet,
the numbers running over the items that stand. First the role, with
its path and `N lines · as its session reads it` (`no prompt` where
the member's config names none). Then how to work with you, where the
prime carries the operator's words: the style with the answer it stands
for (`show · Show me`), what the style asks of an agent, their note in
code, and `jo's settings · every session reads both`; no item where it
has neither. Then the member's memories, a title each in the meta role
with `the yard's` or `{member}'s own` beneath, and `until` the time
where one retires; `N more` for those the yard left out, and `no
memories` where it has none. Last each depot's rules: the depot's name
in code, and under it each rules file's path with its lines (`no
rules`, and `no depots` where the yard has none). A title is one line
of letters, whoever wrote it.

Where the tab has no prime to list it says why, in one quiet line in
place of the items and in no red: `The context drawer is the operator's.`
where the window is not the operator's, `the prime is read on the yard's
machine` (a yard at an address serves the call to no device), that the
yard's yardr is older than the Prime read, the yard's own refusal, which
names the depot and the file it could not read, over `Press Context
again to try again.`, or `Reading the prime…` while it is read. The tab
stands whether or not the Role tab's line was read. It lists, and
carries no foot.
