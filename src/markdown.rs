//! The small part of Markdown a message is drawn with, the part an agent's
//! mail uses: fenced code and inline code, bullets and numbered lists one
//! level deep, headings down to the third, tables, quotes, rules, bold,
//! italic and named links. Everything else is text as it was written.
//!
//! A text becomes blocks, one under the other. A block is drawn without its
//! marks and knows where each of its characters stands in the text as it
//! was written, so that what is selected in it is copied with its marks.

use std::ops::Range;

/// What a block is drawn as.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Lines as they were written, line breaks and blank lines included.
    Text,
    /// What stood between two fences, or after one that was never closed.
    Code,
    /// A heading of level 1 to 3.
    Heading(u8),
    /// An item of a list: `marker` is drawn before it, a dot for a bullet
    /// and the number as it was written. `depth` is 0, or 1 for an item
    /// under another.
    Item { marker: String, depth: u8 },
    /// A table: its head row and its body rows, each cell words of its
    /// own, with each column's alignment from the delimiter. The block
    /// carries no text, marks or links of its own: those stand on the
    /// cells.
    Table {
        head: Vec<Cell>,
        rows: Vec<Vec<Cell>>,
        align: Vec<Align>,
    },
    /// Lines quoted with `>`, their markers stripped.
    Quote,
    /// A rule: no drawn text.
    Rule,
}

/// How a table column's words stand in it, from the delimiter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Centre,
    Right,
}

/// One cell of a table's head or body: words of its own, with their
/// marks and named links in cell coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    /// What is drawn: the source without its marks.
    pub text: String,
    /// All of the source the cell stands for, its marks included.
    pub source: Range<usize>,
    /// The stretches of `text` that are not plain, in order.
    pub marks: Vec<(Range<usize>, Marks)>,
    /// The named links in `text`, in order.
    pub links: Vec<NamedLink>,
    pieces: Vec<Piece>,
}

/// A named link `[words](address)`: the words as drawn, the address a
/// click opens, and the shape whole.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedLink {
    /// The words as drawn, in block (or cell) coordinates.
    pub shown: Range<usize>,
    /// The address a click opens, its scheme in lower case.
    pub url: String,
    /// The shape whole, from `[` past `)`.
    pub source: Range<usize>,
}

/// How a stretch of a block is drawn apart from the rest of it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
}

/// A stretch of a block's text that stands unbroken in the source.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Piece {
    shown: usize,
    source: usize,
    len: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: Kind,
    /// What is drawn: the source without its marks.
    pub text: String,
    /// All of the source the block stands for, its marks included.
    pub source: Range<usize>,
    /// The stretches of `text` that are not plain, in order.
    pub marks: Vec<(Range<usize>, Marks)>,
    /// The named links in `text`, in order.
    pub links: Vec<NamedLink>,
    /// A blank line stood between this block and the one before it.
    pub apart: bool,
    pieces: Vec<Piece>,
}

impl Block {
    /// Where in the source the boundary `shown` of the drawn text is. Marks
    /// that are not drawn may stand there: the place is from before them to
    /// after them, and empty where there are none. A named link's words
    /// stand for the shape whole: inside them at its start, and at their
    /// end on its end.
    pub fn source_at(&self, shown: usize) -> Range<usize> {
        source_at(
            self.text.len(),
            &self.source,
            &self.pieces,
            &self.links,
            shown,
        )
    }

    /// What a copy of this block puts on the clipboard: a code block's
    /// is the source between the fences, byte for byte, and no other block
    /// has a copy control.
    pub fn copy_text(&self) -> Option<&str> {
        matches!(self.kind, Kind::Code).then_some(&self.text)
    }

    /// What of the drawn text the stretch `source` of the source covers. A
    /// bound inside a named link's shape rounds out to it, so a selection
    /// over any of the words highlights them whole.
    pub fn shown(&self, source: Range<usize>) -> Range<usize> {
        shown(&self.pieces, &self.links, source)
    }
}

impl Cell {
    /// Where in the source the boundary `shown` of the drawn text is, as
    /// a block's: a named link's words stand for the shape whole.
    pub fn source_at(&self, shown: usize) -> Range<usize> {
        source_at(
            self.text.len(),
            &self.source,
            &self.pieces,
            &self.links,
            shown,
        )
    }

    /// What of the drawn text the stretch `source` of the source covers,
    /// as a block's.
    pub fn shown(&self, source: Range<usize>) -> Range<usize> {
        shown(&self.pieces, &self.links, source)
    }
}

/// Where in `source` the boundary `shown` of drawn text of length `len`
/// is. Marks that are not drawn may stand there: the place is from before
/// them to after them, and empty where there are none. A block's and a
/// cell's alike.
fn source_at(
    len: usize,
    source: &Range<usize>,
    pieces: &[Piece],
    links: &[NamedLink],
    shown: usize,
) -> Range<usize> {
    if let Some(link) = links
        .iter()
        .find(|link| link.shown.start <= shown && shown <= link.shown.end)
    {
        if shown == link.shown.end {
            return link.source.end..link.source.end;
        }
        return link.source.start..link.source.start;
    }
    let mut here = pieces
        .iter()
        .filter(|p| p.shown <= shown && shown <= p.shown + p.len)
        .map(|p| p.source + (shown - p.shown));
    let first = here.next();
    let last = here.next_back().or(first);
    let before = match first {
        Some(at) if shown > 0 => at,
        _ => source.start,
    };
    let after = match last {
        Some(at) if shown < len => at,
        _ => source.end,
    };
    before..after
}

/// What of the drawn text the stretch `source` of the source covers. A
/// block's and a cell's alike.
fn shown(pieces: &[Piece], links: &[NamedLink], mut source: Range<usize>) -> Range<usize> {
    for link in links {
        if link.source.start < source.start && source.start < link.source.end {
            source.start = link.source.start;
        }
        if link.source.start < source.end && source.end < link.source.end {
            source.end = link.source.end;
        }
    }
    let mut shown: Option<Range<usize>> = None;
    for p in pieces {
        let start = source.start.max(p.source);
        let end = source.end.min(p.source + p.len);
        if start >= end {
            continue;
        }
        let (start, end) = (p.shown + start - p.source, p.shown + end - p.source);
        shown = Some(shown.map_or(start..end, |s| s.start.min(start)..s.end.max(end)));
    }
    shown.unwrap_or(0..0)
}

/// A block as it is put together.
#[derive(Default)]
struct Out {
    text: String,
    pieces: Vec<Piece>,
    marks: Vec<(Range<usize>, Marks)>,
    links: Vec<NamedLink>,
}

impl Out {
    /// `text` is drawn next, and stands at `source` in the source.
    fn push(&mut self, text: &str, source: usize, marks: Marks) {
        if text.is_empty() {
            return;
        }
        let shown = self.text.len();
        self.text.push_str(text);
        match self.pieces.last_mut() {
            Some(p) if p.shown + p.len == shown && p.source + p.len == source => {
                p.len += text.len()
            }
            _ => self.pieces.push(Piece {
                shown,
                source,
                len: text.len(),
            }),
        }
        if marks == Marks::default() {
            return;
        }
        match self.marks.last_mut() {
            Some((range, same)) if range.end == shown && *same == marks => {
                range.end = self.text.len()
            }
            _ => self.marks.push((shown..self.text.len(), marks)),
        }
    }

    fn block(self, kind: Kind, source: Range<usize>, apart: bool) -> Block {
        Block {
            kind,
            text: self.text,
            source,
            marks: self.marks,
            links: self.links,
            apart,
            pieces: self.pieces,
        }
    }

    fn cell(self, source: Range<usize>) -> Cell {
        Cell {
            text: self.text,
            source,
            marks: self.marks,
            links: self.links,
            pieces: self.pieces,
        }
    }
}

/// How many times `c` stands at the start of `text`.
fn run_of(text: &str, c: char) -> usize {
    text.chars().take_while(|&x| x == c).count()
}

/// What may close a sentence or a bracket after a link: as in the links a
/// text is searched for.
const AFTER_A_LINK: [char; 11] = ['.', ',', ';', ':', '!', '?', ')', ']', '>', '"', '\''];

/// Whether a `*` or `_` with `before` it and `after` it may open emphasis:
/// at the start of a word, and not in the middle of one or before a dot or
/// a slash, which is a glob or a path.
fn opens(before: Option<char>, after: Option<char>) -> bool {
    let free = before.is_none_or(|c| c.is_whitespace() || "([{\"'".contains(c));
    free && after.is_some_and(|c| c.is_alphanumeric() || "*_`\"'([".contains(c))
}

/// Where in `text`, which follows an opening run of `n` times `c`, the run
/// that closes it stands: of the same length, at the end of a word.
fn closer(text: &str, c: char, n: usize) -> Option<usize> {
    let mut at = 0;
    while let Some(found) = text[at..].find(c).map(|i| at + i) {
        let run = run_of(&text[found..], c);
        let before = text[..found].chars().next_back();
        let mut after = text[found + run..].chars();
        let ends = match after.next() {
            None => true,
            // A dot closes a sentence, and stands inside a file's name.
            Some('.') => after.next().is_none_or(|c| !c.is_alphanumeric()),
            Some(c) => c.is_whitespace() || ",;:!?)]}\"'".contains(c),
        };
        if run == n && before.is_some_and(|c| !c.is_whitespace()) && ends {
            return Some(found);
        }
        at = found + run;
    }
    None
}

/// Where in `text`, which follows an opening run of `n` backticks, a run of
/// exactly as many stands.
fn code_end(text: &str, n: usize) -> Option<usize> {
    let mut at = 0;
    while let Some(found) = text[at..].find('`').map(|i| at + i) {
        let run = run_of(&text[found..], '`');
        if run == n {
            return Some(found);
        }
        at = found + run;
    }
    None
}

/// The named link `[words](address)` at the start of `line`, if one
/// stands there: its words in the line, the address a click opens, and
/// the shape's length. The words are one or more characters up to the
/// first unescaped `]`; the address runs to the `)` that balances the
/// opening one, holds no space and no control character, and passes the
/// one check. Anything less stays as written.
fn named_link(line: &str) -> Option<(Range<usize>, String, usize)> {
    let rest = line.strip_prefix('[')?;
    let mut words_end = None;
    let mut i = 0;
    while i < rest.len() {
        let c = rest[i..].chars().next()?;
        if c == '\\' {
            let quoted = rest[i + 1..].chars().next().map_or(0, |n| n.len_utf8());
            i += c.len_utf8() + quoted;
            continue;
        }
        if c == ']' {
            words_end = Some(i);
            break;
        }
        i += c.len_utf8();
    }
    let words_end = words_end?;
    if words_end == 0 {
        return None;
    }
    let address = rest[words_end + 1..].strip_prefix('(')?;
    let mut depth = 1;
    let mut i = 0;
    while i < address.len() {
        let c = address[i..].chars().next()?;
        if c == '\\' {
            let quoted = address[i + 1..].chars().next().map_or(0, |n| n.len_utf8());
            i += c.len_utf8() + quoted;
            continue;
        }
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        i += c.len_utf8();
    }
    if depth != 0 {
        return None;
    }
    let raw = &address[..i];
    if raw.is_empty() || raw.contains(' ') || raw.chars().any(|c| c.is_control()) {
        return None;
    }
    let mut unescaped = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some(n @ ('(' | ')')) => unescaped.push(n),
                Some(n) => {
                    unescaped.push('\\');
                    unescaped.push(n);
                }
                None => unescaped.push('\\'),
            }
        } else {
            unescaped.push(c);
        }
    }
    let url = checked_url(&unescaped)?;
    Some((1..1 + words_end, url, 1 + words_end + 1 + 1 + i + 1))
}

/// The address of a named link, checked: `http` or `https`, letter case
/// aside, its authority carrying no user name or password, its host not
/// empty and written in ASCII, and a port, where one stands, that is
/// empty or digits. What opens is the address as written with its scheme
/// in lower case. A backslash past the authority opens as written; in
/// the authority it hides the host, so it is refused there.
fn checked_url(address: &str) -> Option<String> {
    let (scheme, rest) = address.split_once(':')?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let rest = rest.strip_prefix("//")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains('@') || authority.contains('\\') {
        return None;
    }
    if let Some(bracketed) = authority.strip_prefix('[') {
        let (inside, after) = bracketed.split_once(']')?;
        if inside.contains('[') || !inside.bytes().all(|b| b.is_ascii()) {
            return None;
        }
        match after.strip_prefix(':') {
            None if after.is_empty() => {}
            Some(port) if port.bytes().all(|b| b.is_ascii_digit()) => {}
            _ => return None,
        }
    } else {
        if authority.contains(['[', ']']) {
            return None;
        }
        let mut parts = authority.split(':');
        let host = parts.next()?;
        match parts.next() {
            None => {}
            Some(port) if parts.next().is_none() && port.bytes().all(|b| b.is_ascii_digit()) => {}
            _ => return None,
        }
        if host.is_empty() || !host.bytes().all(|b| b.is_ascii()) {
            return None;
        }
    }
    Some(format!("{}://{rest}", scheme.to_lowercase()))
}

/// One line, or what stands inside emphasis, which begins at `at` in the
/// source: inline code, bold and italic lose their marks, and a mark that
/// opens or closes nothing stays as it was written. A named link's shape
/// is read where it stands whole, its words read again for their marks;
/// inside those words a `\]` stands as `]` and no link nests.
fn inline(out: &mut Out, line: &str, at: usize, marks: Marks, in_link: bool) {
    // What was read and not yet put out: text without a mark in it.
    let mut from = 0;
    let mut i = 0;
    while i < line.len() {
        let rest = &line[i..];
        let Some(c) = rest.chars().next() else {
            break;
        };
        if in_link && rest.starts_with("\\]") {
            out.push(&line[from..i], at + from, marks);
            out.push("]", at + i + 1, marks);
            i += 2;
            from = i;
            continue;
        }
        // A link is no place for a mark: its stars and underscores are its
        // own, except those after its end, which may close what is around it.
        if rest.starts_with("http://") || rest.starts_with("https://") {
            let run = rest.find(char::is_whitespace).unwrap_or(rest.len());
            let link =
                rest[..run].trim_end_matches(|c| AFTER_A_LINK.contains(&c) || "*_`".contains(c));
            i += link.len().max("http".len());
            continue;
        }
        // A named link where the whole shape stands, and a `[` that opens
        // none is written characters. An image's `!` is no part of any
        // link here, so its shape stays as written whole.
        if c == '['
            && !in_link
            && !line[..i].ends_with('!')
            && let Some((words, url, len)) = named_link(rest)
        {
            out.push(&line[from..i], at + from, marks);
            let start = out.text.len();
            inline(
                out,
                &line[i + words.start..i + words.end],
                at + i + words.start,
                marks,
                true,
            );
            out.links.push(NamedLink {
                shown: start..out.text.len(),
                url,
                source: at + i..at + i + len,
            });
            i += len;
            from = i;
            continue;
        }
        if c == '`' {
            let n = run_of(rest, '`');
            if let Some(end) = code_end(&rest[n..], n) {
                out.push(&line[from..i], at + from, marks);
                let code = Marks {
                    code: true,
                    ..marks
                };
                out.push(&rest[n..n + end], at + i + n, code);
                i += n + end + n;
                from = i;
            } else {
                i += n;
            }
            continue;
        }
        if c == '*' || c == '_' {
            let n = run_of(rest, c);
            let before = line[..i].chars().next_back();
            let end = (n <= 3 && opens(before, rest[n..].chars().next()))
                .then(|| closer(&rest[n..], c, n))
                .flatten();
            if let Some(end) = end {
                out.push(&line[from..i], at + from, marks);
                let inner = Marks {
                    bold: marks.bold || n >= 2,
                    italic: marks.italic || n != 2,
                    ..marks
                };
                inline(out, &rest[n..n + end], at + i + n, inner, in_link);
                i += n + end + n;
                from = i;
            } else {
                i += n;
            }
            continue;
        }
        i += c.len_utf8();
    }
    out.push(&line[from..], at + from, marks);
}

/// The backticks of the fence `line` opens, if it opens one: three or more
/// at its start, and after them at most a word for the language.
fn fence(line: &str) -> Option<usize> {
    let line = line.trim_start_matches(' ');
    let n = run_of(line, '`');
    (n >= 3 && !line[n..].trim_end().contains(['`', ' '])).then_some(n)
}

/// Whether `line` closes a fence of `n` backticks: as many or more, alone.
fn closes(line: &str, n: usize) -> bool {
    let line = line.trim_matches(' ');
    line.len() >= n && line.chars().all(|c| c == '`')
}

/// The level of the heading `line` is and where its words begin, if it is
/// one: one to three `#` at its start and a space.
fn heading(line: &str) -> Option<(u8, usize)> {
    let n = run_of(line, '#');
    let rest = &line[n..];
    let words = rest.trim_start_matches(' ');
    ((1..=3).contains(&n) && rest.starts_with(' ') && !words.is_empty())
        .then(|| (n as u8, line.len() - words.len()))
}

/// The item `line` is, if it is one: how far it is indented, its marker,
/// and where its words begin. A marker is `-`, `*` or a number of up to
/// three digits with a dot or a bracket, and a space follows it.
fn item(line: &str) -> Option<(usize, Range<usize>, usize)> {
    let indent = run_of(line, ' ');
    let rest = &line[indent..];
    let marker = if rest.starts_with(['-', '*']) {
        1
    } else {
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        if !(1..=3).contains(&digits) || !rest[digits..].starts_with(['.', ')']) {
            return None;
        }
        digits + 1
    };
    let after = &rest[marker..];
    let words = after.trim_start_matches(' ');
    (after.starts_with(' ') && !words.is_empty())
        .then(|| (indent, indent..indent + marker, line.len() - words.len()))
}

/// Whether the whole line is a rule: three or more of one of `-`, `*` or
/// `_`, with a single space allowed between any two, after up to three
/// leading spaces and trailing spaces left off.
fn rule(line: &str) -> bool {
    let rest = line[run_of(line, ' ').min(3)..].trim_end();
    let mut mark = None;
    let mut count = 0;
    let mut spaced = false;
    for c in rest.chars() {
        if c == ' ' {
            if spaced || count == 0 {
                return false;
            }
            spaced = true;
            continue;
        }
        if !matches!(c, '-' | '*' | '_') {
            return false;
        }
        match mark {
            Some(m) if m != c => return false,
            _ => mark = Some(c),
        }
        spaced = false;
        count += 1;
    }
    count >= 3
}

/// Where the words of quote line `line` begin, if it is one: `>` past no
/// more than three spaces, with at most one space after it stripped.
fn quote_marker(line: &str) -> Option<usize> {
    if run_of(line, ' ') > 3 {
        return None;
    }
    let rest = line.trim_start_matches(' ').strip_prefix('>')?;
    Some(line.len() - rest.strip_prefix(' ').unwrap_or(rest).len())
}

/// One cell as cut from its row: contiguous source segments with the
/// escaped pipes between them, relative to the line.
struct CutCell {
    segments: Vec<Range<usize>>,
    /// The `|` of each `\|` between the segments, relative to the row.
    pipes: Vec<usize>,
}

/// How the row `line` cuts into cells: on a `|` that is neither escaped
/// nor inside an inline code span, with a `|` at the row's very start or
/// end framing it. Each cell's segments stand trimmed of the spaces at
/// the cell's ends.
fn cut_row(line: &str) -> Vec<CutCell> {
    let row = line.trim();
    let offset = line.len() - line.trim_start().len();
    // The cuts and the escaped pipes, relative to the row.
    let mut cuts = Vec::new();
    let mut escaped = Vec::new();
    let mut i = 0;
    while i < row.len() {
        let rest = &row[i..];
        if rest.starts_with("\\|") {
            escaped.push(i + 1);
            i += 2;
            continue;
        }
        if rest.starts_with('`') {
            let n = run_of(rest, '`');
            match code_end(&rest[n..], n) {
                Some(end) => i += n + end + n,
                None => i += n,
            }
            continue;
        }
        if rest.starts_with('|') {
            cuts.push(i);
            i += 1;
            continue;
        }
        i += rest.chars().next().map_or(1, |c| c.len_utf8());
    }
    let mut bounds = Vec::new();
    let mut at = 0;
    for cut in cuts {
        bounds.push(at..cut);
        at = cut + 1;
    }
    bounds.push(at..row.len());
    if row.starts_with('|') {
        bounds.remove(0);
    }
    if row.ends_with('|') && !row[..row.len() - 1].ends_with('\\') {
        bounds.pop();
    }
    bounds
        .into_iter()
        .map(|bound| {
            let mut segments = Vec::new();
            let mut pipes = Vec::new();
            let mut at = bound.start;
            for pipe in escaped.iter().copied().filter(|p| bound.contains(p)) {
                segments.push(at..pipe - 1);
                pipes.push(pipe);
                at = pipe + 1;
            }
            segments.push(at..bound.end);
            if let Some(first) = segments.first_mut() {
                let words = row[first.clone()].trim_start();
                first.start = first.end - words.len();
            }
            if let Some(last) = segments.last_mut() {
                let words = row[last.clone()].trim_end();
                last.end = last.start + words.len();
            }
            CutCell {
                segments: segments
                    .into_iter()
                    .map(|seg| seg.start + offset..seg.end + offset)
                    .collect(),
                pipes: pipes.into_iter().map(|pipe| pipe + offset).collect(),
            }
        })
        .collect()
}

/// The alignments the delimiter row `line` gives, if it is one: as many
/// cells as `cells`, each an optional `:`, three or more `-`, and an
/// optional `:`. None alone is left, a trailing colon alone is right, a
/// leading colon alone is left, both is centre.
fn delimiter(line: &str, cells: usize) -> Option<Vec<Align>> {
    let cuts = cut_row(line);
    if cuts.len() != cells {
        return None;
    }
    cuts.iter()
        .map(|cell| {
            let mut words = String::new();
            for (k, seg) in cell.segments.iter().enumerate() {
                if k > 0 {
                    words.push('|');
                }
                words.push_str(&line[seg.clone()]);
            }
            let (left, rest) = match words.strip_prefix(':') {
                Some(rest) => (true, rest),
                None => (false, words.as_str()),
            };
            let (right, middle) = match rest.strip_suffix(':') {
                Some(middle) => (true, middle),
                None => (false, rest),
            };
            if middle.len() < 3 || !middle.bytes().all(|b| b == b'-') {
                return None;
            }
            Some(match (left, right) {
                (true, true) => Align::Centre,
                (false, true) => Align::Right,
                _ => Align::Left,
            })
        })
        .collect()
}

/// The cells of the row `line`, which begins at `at` in the source: each
/// cut read as words of its own, a short row filled out with empty cells
/// and a long row cut to `columns`.
fn table_cells(line: &str, at: usize, cuts: Vec<CutCell>, columns: usize) -> Vec<Cell> {
    let mut cells: Vec<Cell> = cuts
        .into_iter()
        .take(columns)
        .map(|cut| {
            let mut out = Out::default();
            for (k, seg) in cut.segments.iter().enumerate() {
                if k > 0 {
                    out.push("|", at + cut.pipes[k - 1], Marks::default());
                }
                inline(
                    &mut out,
                    &line[seg.clone()],
                    at + seg.start,
                    Marks::default(),
                    false,
                );
            }
            let source = match (cut.segments.first(), cut.segments.last()) {
                (Some(first), Some(last)) => at + first.start..at + last.end,
                _ => at + line.len()..at + line.len(),
            };
            out.cell(source)
        })
        .collect();
    while cells.len() < columns {
        let end = at + line.len();
        cells.push(Cell {
            text: String::new(),
            source: end..end,
            marks: Vec::new(),
            links: Vec::new(),
            pieces: Vec::new(),
        });
    }
    cells
}

/// The table whose head is `lines[i]`, if one stands there: its kind,
/// its source, and the line after its last. None where no delimiter
/// stands under the head with as many cells, or the head holds fewer
/// than two.
fn table(lines: &[(usize, &str)], i: usize) -> Option<(Kind, Range<usize>, usize)> {
    let head = cut_row(lines[i].1);
    if head.len() < 2 {
        return None;
    }
    let columns = head.len();
    let align = delimiter(lines.get(i + 1)?.1, columns)?;
    let mut last = i + 1;
    let mut rows = Vec::new();
    while let Some(&(at, body)) = lines.get(last + 1) {
        if body.trim().is_empty() || !body.contains('|') {
            break;
        }
        // A line another reading claims ends the table.
        if fence(body).is_some()
            || heading(body).is_some()
            || rule(body)
            || item(body).is_some()
            || quote_marker(body).is_some()
        {
            break;
        }
        rows.push(table_cells(body, at, cut_row(body), columns));
        last += 1;
    }
    let head = table_cells(lines[i].1, lines[i].0, head, columns);
    let source = lines[i].0..lines[last].0 + lines[last].1.len();
    let kind = Kind::Table { head, rows, align };
    Some((kind, source, last + 1))
}

/// A link this long or shorter is drawn as it was written. A longer
/// one draws shortened, as [`short_link`] has it: raw calendar and call
/// links run to two lines in the reading column, and the shortened form
/// stays on one.
const LONG_LINK: usize = 60;

/// The last characters of a shortened link's path: its end, after the
/// ellipsis.
const LINK_TAIL: usize = 24;

/// How a link is drawn: a short one as it was written, and a longer one
/// as its domain and the end of its path, the scheme gone
/// (`teams.microsoft.com/…QefBDwL`). The end is the last characters of
/// whatever follows the host, the query and the fragment included. The
/// full link still opens on a click, and stands in the tooltip.
pub fn short_link(url: &str) -> String {
    if url.len() <= LONG_LINK {
        return url.to_string();
    }
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return url.to_string();
    };
    // A valid URL can put its query or fragment directly after the host,
    // with no slash for an empty path. Keep those suffixes with the path
    // tail so a long root URL shortens just like one with a path.
    let (host, suffix) = match rest.find(['/', '?', '#']) {
        Some(at) => rest.split_at(at),
        None => return rest.to_string(),
    };
    let chars = suffix.chars().count();
    if chars <= LINK_TAIL + 1 {
        return rest.to_string();
    }
    let tail: String = suffix.chars().skip(chars - LINK_TAIL).collect();
    format!("{host}/…{}", tail.trim_start_matches('/'))
}

/// The host a named link shows beside its words: the checked address's
/// host as written, lower-cased, without port, path or scheme. Empty
/// where no host stands, which a checked address never gives.
pub fn link_host(url: &str) -> String {
    let authority = url
        .split("://")
        .nth(1)
        .unwrap_or("")
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("");
    let host = match authority.strip_prefix('[') {
        Some(bracketed) => match bracketed.split_once(']') {
            Some((inside, _)) => format!("[{inside}]"),
            None => String::new(),
        },
        None => authority.split(':').next().unwrap_or("").to_string(),
    };
    host.to_lowercase()
}

/// The whole `text` as one block, read as no Markdown: what is drawn
/// is what was written, byte for byte, with no marks and no block in
/// it. For words that stay as they were written wherever they stand.
pub fn verbatim(text: &str) -> Block {
    let mut out = Out::default();
    out.push(text, 0, Marks::default());
    out.block(Kind::Text, 0..text.len(), false)
}

/// The blocks `text` is drawn as, from top to bottom. A text without a mark
/// is one block with the text as it is.
pub fn blocks(text: &str) -> Vec<Block> {
    let mut lines = Vec::new();
    let mut start = 0;
    for line in text.split('\n') {
        lines.push((start, line));
        start += line.len() + 1;
    }
    let end_of = |i: usize| lines[i].0 + lines[i].1.len();
    let blank = |i: usize| lines[i].1.trim().is_empty();

    let mut blocks: Vec<Block> = Vec::new();
    // A blank line was passed since the last block.
    let mut apart = false;
    // The lines since the last block that are text and nothing else.
    let mut plain: Option<Range<usize>> = None;
    // They become a block. The blank lines between it and its neighbours
    // are the space between blocks, and no lines of it.
    let flush = |plain: &mut Option<Range<usize>>,
                 blocks: &mut Vec<Block>,
                 apart: &mut bool,
                 more: bool| {
        let Some(mut run) = plain.take() else {
            return;
        };
        while !blocks.is_empty() && !run.is_empty() && blank(run.start) {
            run.start += 1;
            *apart = true;
        }
        let mut after = false;
        while more && !run.is_empty() && blank(run.end - 1) {
            run.end -= 1;
            after = true;
        }
        if !run.is_empty() {
            let mut out = Out::default();
            for i in run.clone() {
                let (at, line) = lines[i];
                if i > run.start {
                    out.push("\n", at - 1, Marks::default());
                }
                inline(&mut out, line, at, Marks::default(), false);
            }
            let source = lines[run.start].0..end_of(run.end - 1);
            blocks.push(out.block(Kind::Text, source, *apart));
            *apart = false;
        }
        *apart |= after;
    };

    let mut i = 0;
    while i < lines.len() {
        let (at, line) = lines[i];
        if let Some(n) = fence(line) {
            flush(&mut plain, &mut blocks, &mut apart, true);
            // A fence that is never closed has the rest of the text.
            let close = (i + 1..lines.len()).find(|&j| closes(lines[j].1, n));
            let last = close.unwrap_or(lines.len());
            let mut out = Out::default();
            if last > i + 1 {
                let code = lines[i + 1].0..end_of(last - 1);
                out.push(&text[code.clone()], code.start, Marks::default());
            }
            let end = end_of(close.unwrap_or(lines.len() - 1));
            blocks.push(out.block(Kind::Code, at..end, apart));
            apart = false;
            i = last + 1;
            continue;
        }
        if let Some((level, words)) = heading(line) {
            flush(&mut plain, &mut blocks, &mut apart, true);
            let mut out = Out::default();
            inline(
                &mut out,
                line[words..].trim_end(),
                at + words,
                Marks::default(),
                false,
            );
            blocks.push(out.block(Kind::Heading(level), at..at + line.len(), apart));
            apart = false;
            i += 1;
            continue;
        }
        if rule(line) {
            flush(&mut plain, &mut blocks, &mut apart, true);
            blocks.push(Out::default().block(Kind::Rule, at..at + line.len(), apart));
            apart = false;
            i += 1;
            continue;
        }
        if let Some((indent, marker, words)) = item(line) {
            flush(&mut plain, &mut blocks, &mut apart, true);
            let under = matches!(blocks.last(), Some(b) if matches!(b.kind, Kind::Item { .. }));
            let marker = match &line[marker] {
                "-" | "*" => "•".to_string(),
                number => number.to_string(),
            };
            let mut out = Out::default();
            inline(
                &mut out,
                &line[words..],
                at + words,
                Marks::default(),
                false,
            );
            // An indented line under an item goes on with it.
            let mut last = i;
            while let Some(&(next_at, next)) = lines.get(last + 1) {
                let goes_on = next.starts_with(' ')
                    && !next.trim().is_empty()
                    && fence(next).is_none()
                    && item(next).is_none();
                if !goes_on {
                    break;
                }
                out.push("\n", next_at - 1, Marks::default());
                let words = next.len() - next.trim_start_matches(' ').len();
                inline(
                    &mut out,
                    &next[words..],
                    next_at + words,
                    Marks::default(),
                    false,
                );
                last += 1;
            }
            let kind = Kind::Item {
                marker,
                depth: (indent >= 2 && under) as u8,
            };
            blocks.push(out.block(kind, at..end_of(last), apart));
            apart = false;
            i = last + 1;
            continue;
        }
        if quote_marker(line).is_some() {
            let mut last = i;
            while let Some(&(_, next)) = lines.get(last + 1) {
                if quote_marker(next).is_none() {
                    break;
                }
                last += 1;
            }
            // A run in which no line carries words governs nothing, so
            // its lines stay text as they were written.
            let carries = (i..=last).any(|j| {
                quote_marker(lines[j].1)
                    .is_some_and(|marker| !lines[j].1[marker..].trim().is_empty())
            });
            if carries {
                flush(&mut plain, &mut blocks, &mut apart, true);
                let mut out = Out::default();
                for (k, &(at, quoted)) in lines[i..=last].iter().enumerate() {
                    if let Some(marker) = quote_marker(quoted) {
                        if k > 0 {
                            out.push("\n", at - 1, Marks::default());
                        }
                        inline(
                            &mut out,
                            &quoted[marker..],
                            at + marker,
                            Marks::default(),
                            false,
                        );
                    }
                }
                blocks.push(out.block(Kind::Quote, at..end_of(last), apart));
                apart = false;
                i = last + 1;
                continue;
            }
        }
        if line.contains('|')
            && let Some((kind, source, next)) = table(&lines, i)
        {
            flush(&mut plain, &mut blocks, &mut apart, true);
            blocks.push(Out::default().block(kind, source, apart));
            apart = false;
            i = next;
            continue;
        }
        plain = Some(plain.map_or(i..i + 1, |run| run.start..i + 1));
        i += 1;
    }
    flush(&mut plain, &mut blocks, &mut apart, false);
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOLD: Marks = Marks {
        bold: true,
        italic: false,
        code: false,
    };
    const ITALIC: Marks = Marks {
        bold: false,
        italic: true,
        code: false,
    };
    const CODE: Marks = Marks {
        bold: false,
        italic: false,
        code: true,
    };

    /// The blocks of `text` as their kind and what is drawn.
    fn drawn(text: &str) -> Vec<(Kind, String)> {
        blocks(text).into_iter().map(|b| (b.kind, b.text)).collect()
    }

    fn text(words: &str) -> (Kind, String) {
        (Kind::Text, words.to_string())
    }

    fn bullet(depth: u8, words: &str) -> (Kind, String) {
        let marker = "•".to_string();
        (Kind::Item { marker, depth }, words.to_string())
    }

    fn quote(words: &str) -> (Kind, String) {
        (Kind::Quote, words.to_string())
    }

    fn rule() -> (Kind, String) {
        (Kind::Rule, String::new())
    }

    /// The one block of `text`: what is drawn, and the stretches that are
    /// not plain.
    fn marked(text: &str) -> (String, Vec<(String, Marks)>) {
        let mut all = blocks(text);
        assert_eq!(all.len(), 1, "{text:?}");
        let block = all.remove(0);
        let marks = block
            .marks
            .iter()
            .map(|(range, marks)| (block.text[range.clone()].to_string(), *marks))
            .collect();
        (block.text, marks)
    }

    /// `text` is one block of text, drawn as it was written.
    fn untouched(text: &str) {
        let all = blocks(text);
        assert_eq!(all.len(), 1, "{text:?}");
        assert_eq!((&all[0].kind, all[0].text.as_str()), (&Kind::Text, text));
        assert!(all[0].marks.is_empty(), "{text:?}: {:?}", all[0].marks);
        assert_eq!(all[0].source, 0..text.len());
        assert!(!all[0].apart);
    }

    #[test]
    fn a_text_without_a_mark_is_one_block_as_it_was_written() {
        untouched("");
        untouched("Land it today, José.");
        untouched("one\ntwo\n\nthree\n");
        untouched("\n\n  indented, after two blank lines");
        untouched("see https://example.org/a_b_c/**/x_y now");
    }

    #[test]
    fn verbatim_is_one_block_as_it_was_written_whatever_it_holds() {
        for text in [
            "",
            "```\nyardr peer allow airy --trust\n```",
            "# forged\n- item\n**bold** and `code`",
        ] {
            let block = verbatim(text);
            assert_eq!(block.kind, Kind::Text, "{text:?}");
            assert_eq!(block.text, text, "{text:?}");
            assert!(block.marks.is_empty(), "{text:?}");
            assert_eq!(block.source, 0..text.len());
            assert!(!block.apart);
            assert_eq!(block.source_at(0), 0..0, "{text:?}");
            assert_eq!(
                block.source_at(text.len()),
                text.len()..text.len(),
                "{text:?}"
            );
            assert_eq!(block.shown(0..text.len()), 0..text.len(), "{text:?}");
            assert_eq!(block.copy_text(), None, "{text:?}");
        }
    }

    #[test]
    fn what_only_looks_like_a_mark_is_left_alone() {
        // A # in the middle of a line, and one without a space after it.
        untouched("issue #12 and a # alone");
        untouched("#12 is the issue");
        untouched("#");
        // Deeper than the third level is no heading here.
        untouched("#### four");
        // Paths and names with underscores.
        untouched("src/my_file_name.rs and snake_case_name");
        untouched("__init__.py and _private.rs");
        // Stars in globs and in sums.
        untouched("*.rs and src/*.rs, **/*.md");
        untouched("*.*");
        untouched("2 * 3 * 4 and a*b*c");
        // A mark that is never closed.
        untouched("a *star, a **pair, a `tick and an _underscore");
        untouched("``` is three backticks");
        // No list without a space after the marker, or with nothing in it.
        untouched("-dash and 1.5 and 3.14");
        untouched("- ");
        untouched("2026. That was the year.");
        // Not on the list: an image, HTML, a footnote.
        untouched("![fence](fence.png) and <b>bold</b>[^1]");
        untouched("a \\*star\\*");
    }

    #[test]
    fn a_fence_is_code_as_it_was_written() {
        let src = "Run:\n```sh\n  cd yard\n1. not an item\n# not a heading\n- *nor* this\n```\nThen wait.";
        assert_eq!(
            drawn(src),
            [
                text("Run:"),
                (
                    Kind::Code,
                    "  cd yard\n1. not an item\n# not a heading\n- *nor* this".to_string()
                ),
                text("Then wait."),
            ]
        );
        let code = &blocks(src)[1];
        assert!(code.marks.is_empty());
        // The block stands for its fences too.
        assert_eq!(
            &src[code.source.clone()],
            &src[5..src.len() - "\nThen wait.".len()]
        );

        // Without a language word, and with more backticks than three.
        assert_eq!(drawn("```\nls\n```"), [(Kind::Code, "ls".to_string())]);
        assert_eq!(
            drawn("````\n```\nls\n```\n````"),
            [(Kind::Code, "```\nls\n```".to_string())]
        );
        // Blank lines in it are its own.
        assert_eq!(
            drawn("```\n\na\n\nb\n\n```"),
            [(Kind::Code, "\na\n\nb\n".to_string())]
        );
        // An empty one is still a block.
        assert_eq!(drawn("```\n```"), [(Kind::Code, String::new())]);
    }

    #[test]
    fn a_copy_of_a_code_block_is_the_source_between_the_fences() {
        let code = format!(
            "  indented\n\ttabbed\n\n{}",
            "echo a very long line that wraps in the window ".repeat(20)
        );
        let src = format!("Run:\n```sh\n{code}\n```\nThen wait.");
        let all = blocks(&src);
        assert_eq!(all.len(), 3);
        assert_eq!(all[1].copy_text(), Some(code.as_str()));
        // The fences and the language word are not in it, and no other
        // block has a copy control.
        assert_eq!(all[0].copy_text(), None);
        assert_eq!(all[2].copy_text(), None);
        assert!(!all[1].copy_text().unwrap().contains("```"));
    }

    #[test]
    fn an_unclosed_fence_has_the_rest_as_code_and_hides_nothing() {
        let src = "Look:\n```\nfirst\n\n- second\n**third**";
        assert_eq!(
            drawn(src),
            [
                text("Look:"),
                (Kind::Code, "first\n\n- second\n**third**".to_string()),
            ]
        );
        assert_eq!(blocks(src)[1].source, 6..src.len());
        assert_eq!(drawn("```"), [(Kind::Code, String::new())]);
    }

    #[test]
    fn inline_code_bold_and_italic_lose_their_marks() {
        assert_eq!(
            marked("run `yardr prime` now"),
            (
                "run yardr prime now".to_string(),
                vec![("yardr prime".to_string(), CODE)]
            )
        );
        assert_eq!(
            marked("a **bold** and an *italic* and an _other_ and __more__."),
            (
                "a bold and an italic and an other and more.".to_string(),
                vec![
                    ("bold".to_string(), BOLD),
                    ("italic".to_string(), ITALIC),
                    ("other".to_string(), ITALIC),
                    ("more".to_string(), BOLD),
                ]
            )
        );
        let both = Marks {
            bold: true,
            italic: true,
            code: false,
        };
        assert_eq!(
            marked("***both*** and **bold with _italic_ in it**"),
            (
                "both and bold with italic in it".to_string(),
                vec![
                    ("both".to_string(), both),
                    ("bold with ".to_string(), BOLD),
                    ("italic".to_string(), both),
                    (" in it".to_string(), BOLD),
                ]
            )
        );
        // What is in code is code: no mark in it is read.
        assert_eq!(
            marked("`**/*.rs` and ``a ` b``"),
            (
                "**/*.rs and a ` b".to_string(),
                vec![("**/*.rs".to_string(), CODE), ("a ` b".to_string(), CODE)]
            )
        );
        let bold_code = Marks {
            bold: true,
            italic: false,
            code: true,
        };
        assert_eq!(
            marked("**`cargo test`**"),
            (
                "cargo test".to_string(),
                vec![("cargo test".to_string(), bold_code)]
            )
        );
        // A link keeps its own stars and underscores, and what closes after
        // it closes.
        assert_eq!(
            marked("**https://example.org/a_b_** (_see https://example.org/*x*_)"),
            (
                "https://example.org/a_b_ (see https://example.org/*x*)".to_string(),
                vec![
                    ("https://example.org/a_b_".to_string(), BOLD),
                    ("see https://example.org/*x*".to_string(), ITALIC),
                ]
            )
        );
        // Emphasis does not go over a line break.
        untouched("*one\ntwo*");
    }

    #[test]
    fn a_long_link_draws_as_its_domain_and_the_end_of_its_path() {
        // A short link draws as it was written, the scheme included.
        assert_eq!(short_link("https://example.org/a"), "https://example.org/a");
        assert_eq!(short_link("http://a.example"), "http://a.example");
        let sixty = format!("https://example.org/{}", "a".repeat(40));
        assert_eq!(sixty.len(), 60);
        assert_eq!(short_link(&sixty), sixty);
        // A longer one draws as its domain and the end of its path, the
        // scheme gone.
        let long = format!("https://example.org/{}", "a".repeat(41));
        assert_eq!(
            short_link(&long),
            format!("example.org/…{}", "a".repeat(24))
        );
        // The end is the last characters of whatever follows the host: a
        // call link keeps its id, a calendar link its query.
        assert_eq!(
            short_link(
                "https://teams.microsoft.com/l/meetup-join/19%3ameeting_N2Q4YzQefBDwL/0?context=chat"
            ),
            "teams.microsoft.com/…YzQefBDwL/0?context=chat"
        );
        assert_eq!(
            short_link(
                "https://calendar.google.com/calendar/event?action=TEMPLATE&tmeid=abc123&dates=20261019"
            ),
            "calendar.google.com/…id=abc123&dates=20261019"
        );
        // A root URL may put its query or fragment directly after the
        // host, with no slash to mark the path.
        assert_eq!(
            short_link("https://calendar.google.com?event_id=12345678901234567890123456789012"),
            "calendar.google.com/…901234567890123456789012"
        );
        assert_eq!(
            short_link("https://example.org#release-20261019-123456789012345678901234"),
            "example.org/…123456789012345678901234"
        );
        // A long host with no path, or with a short one, loses the scheme
        // alone.
        let host = format!("https://{}.example.org", "a".repeat(50));
        assert_eq!(short_link(&host), host.trim_start_matches("https://"));
        // What is no link draws as it was written.
        assert_eq!(short_link("ftp://example.org/a"), "ftp://example.org/a");
    }

    #[test]
    fn bullets_and_numbers_are_items_one_level_deep() {
        let src = "To do:\n- one\n* two\n  - under two\n      - and deeper\n1. first\n10) tenth";
        let number = |marker: &str, words: &str| {
            let marker = marker.to_string();
            (Kind::Item { marker, depth: 0 }, words.to_string())
        };
        assert_eq!(
            drawn(src),
            [
                text("To do:"),
                bullet(0, "one"),
                bullet(0, "two"),
                bullet(1, "under two"),
                bullet(1, "and deeper"),
                number("1.", "first"),
                number("10)", "tenth"),
            ]
        );
        // An indented line goes on with its item, without its indent; one
        // that is not indented is text again.
        assert_eq!(
            drawn("- one\n  goes on\nand ends\n\n- two"),
            [
                bullet(0, "one\ngoes on"),
                text("and ends"),
                bullet(0, "two")
            ]
        );
        // An item's words are read as a line's are.
        let item = &blocks("- **Done**: see `x`")[0];
        assert_eq!(item.text, "Done: see x");
        assert_eq!(item.marks, [(0..4, BOLD), (10..11, CODE)]);
        // An indented item under no other is at the first level.
        assert_eq!(drawn("  - alone"), [bullet(0, "alone")]);
    }

    #[test]
    fn headings_go_to_the_third_level() {
        assert_eq!(
            drawn("# One\n## Two\n### Three ###\ntext"),
            [
                (Kind::Heading(1), "One".to_string()),
                (Kind::Heading(2), "Two".to_string()),
                (Kind::Heading(3), "Three ###".to_string()),
                text("text"),
            ]
        );
        let heading = &blocks("## The `new` stage")[0];
        assert_eq!(heading.text, "The new stage");
        assert_eq!(heading.marks, [(4..7, CODE)]);
    }

    #[test]
    fn blank_lines_between_blocks_are_space_and_no_lines() {
        let all = blocks("# Head\n\nwords\n\n\n- item\n- next\n\n```\ncode\n```\n\n");
        let apart: Vec<(&str, bool)> = all.iter().map(|b| (b.text.as_str(), b.apart)).collect();
        assert_eq!(
            apart,
            [
                ("Head", false),
                ("words", true),
                ("item", true),
                ("next", false),
                ("code", true),
            ]
        );
        // Inside text they stay lines.
        assert_eq!(
            drawn("one\n\ntwo\n# Head"),
            [text("one\n\ntwo"), (Kind::Heading(1), "Head".to_string())]
        );
    }

    #[test]
    fn a_place_in_a_block_is_a_place_in_the_source() {
        let src = "a **bold** b";
        let block = &blocks(src)[0];
        assert_eq!(block.text, "a bold b");
        assert_eq!(block.source_at(0), 0..0);
        assert_eq!(block.source_at(1), 1..1);
        // Before the first letter of the bold word the stars stand, and
        // after its last.
        assert_eq!(block.source_at(2), 2..4);
        assert_eq!(block.source_at(4), 6..6);
        assert_eq!(block.source_at(6), 8..10);
        assert_eq!(block.source_at(8), 12..12);
        // What is selected of the source, as it is drawn.
        assert_eq!(block.shown(0..src.len()), 0..8);
        assert_eq!(block.shown(4..8), 2..6);
        assert_eq!(block.shown(2..5), 2..3);
        assert_eq!(block.shown(2..4), 0..0);
        assert_eq!(block.shown(11..40), 7..8);

        // The start and the end of a block are where its marks begin and
        // end.
        let src = "one\n```sh\nls -la\n```\n## Two";
        let all = blocks(src);
        let (code, heading) = (&all[1], &all[2]);
        assert_eq!(code.source_at(0), 4..10);
        assert_eq!(code.source_at(3), 13..13);
        assert_eq!(code.source_at(6), 16..20);
        assert_eq!(heading.source_at(0), 21..24);
        assert_eq!(heading.source_at(3), 27..27);
        assert_eq!(code.shown(0..src.len()), 0..6);
        assert_eq!(all[0].shown(4..src.len()), 0..0);

        // An item that goes on: the indent of its second line is not drawn.
        let src = "- one\n  two";
        let item = &blocks(src)[0];
        assert_eq!(item.text, "one\ntwo");
        assert_eq!(item.source_at(0), 0..2);
        assert_eq!(item.source_at(3), 5..5);
        assert_eq!(item.source_at(4), 6..8);
        assert_eq!(item.source_at(7), 11..11);
    }

    /// The table `text` is drawn as: its head, its rows and its alignments.
    fn tabled(text: &str) -> (Vec<String>, Vec<Vec<String>>, Vec<Align>) {
        let mut all = blocks(text);
        assert_eq!(all.len(), 1, "{text:?}");
        match all.remove(0).kind {
            Kind::Table { head, rows, align } => {
                let words =
                    |cells: Vec<Cell>| cells.into_iter().map(|cell| cell.text).collect::<Vec<_>>();
                (words(head), rows.into_iter().map(words).collect(), align)
            }
            kind => panic!("{text:?}: {kind:?}"),
        }
    }

    /// The one block `text` is drawn as: what is drawn, and its named
    /// links as (words, address, shape).
    fn linked(text: &str) -> (String, Vec<(String, String, String)>) {
        let mut all = blocks(text);
        assert_eq!(all.len(), 1, "{text:?}");
        let block = all.remove(0);
        let links = block
            .links
            .iter()
            .map(|link| {
                (
                    block.text[link.shown.clone()].to_string(),
                    link.url.clone(),
                    text[link.source.clone()].to_string(),
                )
            })
            .collect();
        (block.text, links)
    }

    #[test]
    fn a_table_is_its_head_and_its_rows_with_the_delimiter_s_alignments() {
        let src = "| Yard | Waiting |\n| :--- | ------: |\n| home | none |\n| airy | 2 |";
        assert_eq!(
            tabled(src),
            (
                vec!["Yard".to_string(), "Waiting".to_string()],
                vec![
                    vec!["home".to_string(), "none".to_string()],
                    vec!["airy".to_string(), "2".to_string()],
                ],
                vec![Align::Left, Align::Right],
            )
        );
        let all = blocks(src);
        let block = &all[0];
        assert_eq!(block.source, 0..src.len());
        assert!(block.marks.is_empty());
        assert!(block.links.is_empty());
        assert!(!block.apart);
        // Each cell knows its words in the source: a selection inside one
        // copies those words with their marks, and one across cells the
        // written lines whole.
        let (head, rows) = match &block.kind {
            Kind::Table { head, rows, .. } => (head, rows),
            kind => panic!("{src:?}: {kind:?}"),
        };
        assert_eq!(&src[head[0].source.clone()], "Yard");
        assert_eq!(&src[rows[1][1].source.clone()], "2");
        assert_eq!(head[0].shown(2..3), 0..1);
        assert_eq!(head[0].source_at(1), 3..3);
        assert_eq!(rows[1][1].source_at(0), 63..63);
        for cell in head.iter().chain(rows.iter().flatten()) {
            assert_eq!(cell.shown(cell.source.clone()), 0..cell.text.len());
        }
    }

    #[test]
    fn a_cell_holds_marks_and_links_and_pipes_of_its_own() {
        // Bold, code and a named link in cells.
        let src = "| **Yard** | `up` [page](https://yardr.dev/p) |\n| --- | --- |";
        let (head, rows, align) = tabled(src);
        assert_eq!(head, ["Yard", "up page"]);
        assert!(rows.is_empty());
        assert_eq!(align, [Align::Left, Align::Left]);
        let all = blocks(src);
        let head = match &all[0].kind {
            Kind::Table { head, .. } => head,
            kind => panic!("{src:?}: {kind:?}"),
        };
        assert_eq!(head[0].marks, [(0..4, BOLD)]);
        assert_eq!(head[1].marks, [(0..2, CODE)]);
        assert_eq!(head[1].links.len(), 1);
        assert_eq!(head[1].links[0].shown, 3..7);
        assert_eq!(head[1].links[0].url, "https://yardr.dev/p");
        assert_eq!(
            &src[head[1].links[0].source.clone()],
            "[page](https://yardr.dev/p)"
        );
        // A code span's pipe cuts no cell, and an escaped pipe stands in it.
        let src = "| `a|b` | c\\|d |\n| --- | --- |";
        assert_eq!(tabled(src).0, ["a|b", "c|d"]);
        let all = blocks(src);
        let head = match &all[0].kind {
            Kind::Table { head, .. } => head,
            kind => panic!("{src:?}: {kind:?}"),
        };
        assert_eq!(head[0].marks, [(0..3, CODE)]);
        assert_eq!(&src[head[1].source.clone()], "c\\|d");
        assert_eq!(head[1].source_at(0), 10..10);
        assert_eq!(head[1].source_at(1), 11..12);
        assert_eq!(head[1].source_at(2), 13..13);
        assert_eq!(head[1].source_at(3), 14..14);
        assert_eq!(head[1].shown(10..14), 0..3);
        assert_eq!(head[1].shown(11..12), 0..0);
    }

    #[test]
    fn a_short_row_is_filled_a_long_row_cut_and_a_head_alone_is_a_table() {
        assert_eq!(
            tabled("| a | b |\n| --- | --- |\n| 1 |"),
            (
                vec!["a".to_string(), "b".to_string()],
                vec![vec!["1".to_string(), String::new()]],
                vec![Align::Left, Align::Left],
            )
        );
        // The filled cell is empty, at its line's end.
        let all = blocks("| a | b |\n| --- | --- |\n| 1 |");
        let rows = match &all[0].kind {
            Kind::Table { rows, .. } => rows,
            kind => panic!("{kind:?}"),
        };
        assert_eq!(rows[0][1].text, "");
        assert_eq!(rows[0][1].source, 29..29);
        // A long row has its extra cells left off.
        assert_eq!(
            tabled("| a | b |\n| --- | --- |\n| 1 | 2 | 3 |").1,
            [["1".to_string(), "2".to_string()]]
        );
        // A header and delimiter with no body: drawn as its head alone.
        let src = "| a | b |\n|---|---|";
        assert_eq!(
            tabled(src),
            (
                vec!["a".to_string(), "b".to_string()],
                Vec::new(),
                vec![Align::Left, Align::Left],
            )
        );
        assert_eq!(blocks(src)[0].source, 0..src.len());
    }

    #[test]
    fn a_table_claimed_only_as_a_whole_falls_back_to_text() {
        // No delimiter under the head.
        untouched("| a | b |\n| 1 | 2 |");
        // A delimiter with fewer cells is none.
        untouched("| a | b |\n| --- |");
        // One column is none.
        untouched("| a |\n| --- |");
        // A body line with no `|` ends the table; the line stands alone.
        let all = blocks("| a | b |\n| --- | --- |\n| 1 | 2 |\nno pipe");
        assert_eq!(all.len(), 2);
        assert!(matches!(all[0].kind, Kind::Table { .. }));
        assert_eq!(
            (&all[1].kind, all[1].text.as_str()),
            (&Kind::Text, "no pipe")
        );
        // As a line another reading claims does.
        for body in ["```", "# head", "- item", "> quoted"] {
            let src = format!("| a | b |\n| --- | --- |\n{body}");
            let all = blocks(&src);
            assert_eq!(all.len(), 2, "{src:?}");
            assert!(matches!(all[0].kind, Kind::Table { .. }), "{src:?}");
        }
    }

    #[test]
    fn a_quote_is_its_lines_without_their_markers() {
        let src = "> Hold the timetable.\n> It is printed.";
        let all = blocks(src);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].kind, Kind::Quote);
        assert_eq!(all[0].text, "Hold the timetable.\nIt is printed.");
        assert_eq!(all[0].source, 0..src.len());
        assert!(all[0].marks.is_empty());
        // The markers stand in the source and in no piece: at the first
        // drawn word the first stands before it, and between the lines the
        // next stands in the middle of the stretch.
        let quote = &all[0];
        assert_eq!(quote.source_at(0), 0..2);
        assert_eq!(quote.source_at(19), 21..21);
        assert_eq!(quote.source_at(20), 22..24);
        assert_eq!(quote.source_at(34), 38..38);
        assert_eq!(quote.shown(0..src.len()), 0..34);
        assert_eq!(quote.shown(2..21), 0..19);
        assert_eq!(quote.shown(24..38), 20..34);
    }

    #[test]
    fn a_quote_holds_marks_and_links_and_blank_lines_of_its_own() {
        let src = "> **Held** `now` [page](https://yardr.dev/p)\n>\n> kept";
        let all = blocks(src);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].text, "Held now page\n\nkept");
        assert_eq!(all[0].marks, [(0..4, BOLD), (5..8, CODE)]);
        assert_eq!(all[0].links.len(), 1);
        assert_eq!(all[0].links[0].shown, 9..13);
        assert_eq!(all[0].links[0].url, "https://yardr.dev/p");
        // A blank quoted line at the run's edge stands at the quote's edge.
        assert_eq!(blocks(">\n> words")[0].text, "\nwords");
        assert_eq!(blocks("> words\n>")[0].text, "words\n");
        // One level is stripped.
        assert_eq!(blocks(">> nested")[0].text, "> nested");
        assert_eq!(blocks(">> nested")[0].kind, Kind::Quote);
        // No lazy continuation: a wrapped sentence left unmarked stands
        // outside the quote as its own text.
        assert_eq!(
            drawn("> Held.\nStanding outside."),
            [quote("Held."), text("Standing outside.")]
        );
        // A marker run governing no words is no quote.
        untouched(">");
        untouched("> \n>");
        // Past three spaces is no quote line.
        untouched("    > quoted");
    }

    #[test]
    fn a_rule_is_a_line_of_three_or_more_of_one_mark() {
        for line in ["---", "***", "___", "- - -", "* * *", "  ---  "] {
            let all = blocks(line);
            assert_eq!(all.len(), 1, "{line:?}");
            assert_eq!(all[0].kind, Kind::Rule, "{line:?}");
            assert!(all[0].text.is_empty());
            assert!(all[0].marks.is_empty());
            assert_eq!(all[0].source, 0..line.len(), "{line:?}");
        }
        // Two is none, and an item's words stay what they are.
        untouched("--");
        assert_eq!(drawn("- -"), [bullet(0, "-")]);
        // A rule under a line of text makes no heading of it.
        assert_eq!(drawn("Title\n---"), [text("Title"), rule()]);
        // A rule between two paragraphs, for its room.
        let all = blocks("above\n\n---\n\nbelow");
        assert_eq!(all.len(), 3);
        assert_eq!(all[1].kind, Kind::Rule);
        assert!(all[1].apart);
        assert!(all[2].apart);
    }

    #[test]
    fn quotes_tables_and_rules_stand_apart_by_the_blank_line_rule() {
        let all = blocks("# Head\n> quoted\n\n| a | b |\n| --- | --- |\n---\n\ntext");
        assert_eq!(all.len(), 5);
        assert!(matches!(all[1].kind, Kind::Quote));
        assert!(matches!(all[2].kind, Kind::Table { .. }));
        assert_eq!(all[3].kind, Kind::Rule);
        let apart: Vec<bool> = all.iter().map(|block| block.apart).collect();
        assert_eq!(apart, [false, false, true, false, true]);
    }

    #[test]
    fn a_named_link_is_its_words_with_its_address_carried() {
        let (text, links) = linked("See the [release page](https://yardr.dev/releases) first.");
        assert_eq!(text, "See the release page first.");
        assert_eq!(
            links,
            [(
                "release page".to_string(),
                "https://yardr.dev/releases".to_string(),
                "[release page](https://yardr.dev/releases)".to_string()
            )]
        );
        // A port, a query and a fragment are a link, opened whole.
        let (text, links) = linked("[yard](https://example.org:8443/a?b=c#d)");
        assert_eq!(text, "yard");
        assert_eq!(links[0].1, "https://example.org:8443/a?b=c#d");
        // Bold and code stand in the words, and around them.
        let all = blocks("[**held** and `kept`](https://example.org/x)");
        assert_eq!(all[0].text, "held and kept");
        assert_eq!(all[0].marks, [(0..4, BOLD), (9..13, CODE)]);
        assert_eq!(all[0].links[0].shown, 0..13);
        let all = blocks("**[held](https://example.org)**");
        assert_eq!(all[0].text, "held");
        assert_eq!(all[0].marks, [(0..4, BOLD)]);
        assert_eq!(all[0].links[0].shown, 0..4);
        // An escaped bracket stands in the words.
        let (text, links) = linked("[a\\]b](https://example.org/x)");
        assert_eq!(text, "a]b");
        assert_eq!(links[0].0, "a]b");
    }

    #[test]
    fn a_shape_that_is_no_link_stays_as_written() {
        untouched("[mail](mailto:a@example.org)");
        untouched("[page](/releases)");
        untouched("[broken](https://example.org");
        untouched("[empty]()");
        untouched("![alt](https://example.org/x.png)");
        untouched("[words](https://example.org \"title\")");
        // The authority's host is evil.example: the disguise stays written.
        untouched("[yardr](https://yardr.dev@evil.example/login)");
        untouched("[yardr](https://user:pass@yardr.dev/login)");
        untouched("[yardr](https://münchen.example/login)");
        // A port that is not digits is no link; an empty one and a
        // bracketed host are, as the phones read them.
        untouched("[yardr](https://yardr.dev:abc/)");
        let (text, links) = linked("[yardr](https://yardr.dev:/x)");
        assert_eq!(
            (text.as_str(), links[0].1.as_str()),
            ("yardr", "https://yardr.dev:/x")
        );
        let (text, links) = linked("[here](https://[::1]/x)");
        assert_eq!(
            (text.as_str(), links[0].1.as_str()),
            ("here", "https://[::1]/x")
        );
        // A well-formed address whose host the words do not describe is
        // still claimed: the check polices the address, not its description.
        let (text, links) = linked("[release page](https://evil.example/login)");
        assert_eq!(
            (text.as_str(), links[0].1.as_str()),
            ("release page", "https://evil.example/login")
        );
    }

    #[test]
    fn a_named_link_stands_beside_a_bare_one_and_inside_quotes_and_cells() {
        let (text, links) = linked("See [the page](https://yardr.dev/p) and https://yardr.dev/q.");
        assert_eq!(text, "See the page and https://yardr.dev/q.");
        assert_eq!(links.len(), 1);
        let all = blocks("> see [the page](https://yardr.dev/p)");
        assert_eq!(all[0].kind, Kind::Quote);
        assert_eq!(all[0].links.len(), 1);
        let all = blocks("| see [the page](https://yardr.dev/p) | x |\n| --- | --- |");
        let head = match &all[0].kind {
            Kind::Table { head, .. } => head,
            kind => panic!("{kind:?}"),
        };
        assert_eq!(head[0].links.len(), 1);
        assert_eq!(head[0].links[0].url, "https://yardr.dev/p");
        // In code the shape is the code's own characters: no named link
        // is carried, while the bare addresses in it still open.
        let all = blocks(
            "```\n[release page](https://yardr.dev/releases) and https://yardr.dev/releases\n```",
        );
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].kind, Kind::Code);
        assert!(all[0].links.is_empty());
        let all = blocks("`[release page](https://yardr.dev/releases)`");
        assert!(all[0].links.is_empty());
        assert_eq!(all[0].marks, [(0..all[0].text.len(), CODE)]);
    }

    #[test]
    fn a_named_link_s_words_stand_for_the_shape_whole() {
        let src = "See the [release page](https://yardr.dev/releases) first.";
        let block = &blocks(src)[0];
        assert_eq!(block.links.len(), 1);
        let link = &block.links[0];
        assert_eq!(link.shown, 8..20);
        assert_eq!(
            &src[link.source.clone()],
            "[release page](https://yardr.dev/releases)"
        );
        // Inside the words at the shape's start, at their end on its end.
        assert_eq!(block.source_at(8), link.source.start..link.source.start);
        assert_eq!(block.source_at(12), link.source.start..link.source.start);
        assert_eq!(block.source_at(20), link.source.end..link.source.end);
        // A bound inside the shape rounds out to it.
        assert_eq!(block.shown(link.source.clone()), 8..20);
        assert_eq!(
            block.shown(link.source.start + 1..link.source.end - 1),
            8..20
        );
    }

    #[test]
    fn a_named_link_shows_its_host_without_port_path_or_scheme() {
        assert_eq!(link_host("https://yardr.dev/releases"), "yardr.dev");
        assert_eq!(link_host("https://example.org:8443/a?b=c#d"), "example.org");
        assert_eq!(link_host("https://YARDR.DEV/Up"), "yardr.dev");
        assert_eq!(link_host("https://[::1]/x"), "[::1]");
        assert_eq!(link_host("https://[::1]:8080/x"), "[::1]");
        assert_eq!(link_host("https://yardr.dev:/x"), "yardr.dev");
        assert_eq!(link_host("http://a.example"), "a.example");
    }

    #[test]
    fn the_forms_stand_together_as_a_reader_meets_them() {
        let src = "Two yards answer tonight:\n\n| Yard | Waiting | Working |\n| :--- | ------: | ------: |\n| home | none | 3 |\n| airy | 2 | none |\n\n> Hold the autumn timetable until the folded\n> carriers are printed.\n\nSee the [release page](https://yardr.dev/releases) before you start.\n\n---";
        let all = blocks(src);
        assert_eq!(all.len(), 5);
        assert_eq!(all[0].kind, Kind::Text);
        assert!(matches!(all[1].kind, Kind::Table { .. }));
        assert_eq!(all[2].kind, Kind::Quote);
        assert_eq!(all[3].kind, Kind::Text);
        assert_eq!(all[4].kind, Kind::Rule);
        assert_eq!(all[0].text, "Two yards answer tonight:");
        assert_eq!(
            all[2].text,
            "Hold the autumn timetable until the folded\ncarriers are printed."
        );
        assert_eq!(all[3].links.len(), 1);
        assert_eq!(all[3].links[0].url, "https://yardr.dev/releases");
        // The table's head and rows, with the numbers at the right.
        match &all[1].kind {
            Kind::Table { head, rows, align } => {
                let words: Vec<&str> = head.iter().map(|cell| cell.text.as_str()).collect();
                assert_eq!(words, ["Yard", "Waiting", "Working"]);
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0][2].text, "3");
                assert_eq!(*align, [Align::Left, Align::Right, Align::Right]);
            }
            kind => panic!("{kind:?}"),
        }
        // The table stands for its head to its last body line, the
        // delimiter inside it.
        assert_eq!(
            &src[all[1].source.clone()],
            "| Yard | Waiting | Working |\n| :--- | ------: | ------: |\n| home | none | 3 |\n| airy | 2 | none |"
        );
        let apart: Vec<bool> = all.iter().map(|block| block.apart).collect();
        assert_eq!(apart, [false, true, true, true, true]);
    }
}
