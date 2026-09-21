/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! The `PARSE` template engine: a byte string and a template in, assignments
//! out.
//! ```text
//! parse value 'abcdefghij' with p 5 q -2 r  ->  [abcd][efghij][cdefghij]
//! parse value 'abcdefghij' with p 5 q <2 r  ->  [abcd][cd]    [cdefghij]
//! ```

use crate::error::{Failure, Raised};
use crate::{Code, Interp, Loud};
use rexx_core::{Bytes, Decoded, InlineText, ObjRef};
use rexx_parse::{ExprKind, Parse, ParseSource, ParseTrigger, TriggerKind};

/// The platform name `PARSE SOURCE`'s first word carries.
pub(crate) const PLATFORM: &[u8] = b"LINUX";

/// The line terminator `.ENDOFLINE` answers -- measured, `c2x(.endOfLine)`
/// is `0A` here. A host whose terminator is not this one is Phase 11's, and
/// this constant is one of the sites that phase's seam has to reach.
pub(crate) const LINE_END: &[u8] = b"\n";

/// `PARSE VERSION`'s string.
pub(crate) const VERSION: &[u8] = b"REXX-ooRexx_5.3.0(MT)_64-bit 6.06 30 Jul 2026";

/// `RexxInfo~version`: `ORX_VER.ORX_REL.ORX_MOD`, which
/// `RexxInfo::initialize` renders with the same `%d.%d.%d` that
/// `Interpreter::getVersionString` (`runtime/Version.cpp:73`) embeds in
/// [`VERSION`].
pub(crate) const VERSION_NUMBER: &[u8] = part(VERSION, UNDERSCORE + 1, PAREN);

/// `RexxInfo~majorVersion`: `ORX_VER`, [`VERSION_NUMBER`]'s first field.
pub(crate) const MAJOR_VERSION: &[u8] = part(VERSION_NUMBER, 0, FIRST_DOT);

/// `RexxInfo~release`: `ORX_REL`, [`VERSION_NUMBER`]'s second field.
pub(crate) const RELEASE: &[u8] = part(VERSION_NUMBER, FIRST_DOT + 1, SECOND_DOT);

/// `RexxInfo~modification`: `ORX_MOD`, [`VERSION_NUMBER`]'s third field.
pub(crate) const MODIFICATION: &[u8] = part(VERSION_NUMBER, SECOND_DOT + 1, VERSION_NUMBER.len());

/// `RexxInfo~languageLevel`: `Interpreter::getLanguageLevelString()`, the
/// word `Version.cpp:73` writes after the `-bit` one.
pub(crate) const LANGUAGE_LEVEL: &[u8] = part(VERSION, BIT_BLANK + 1, LEVEL_BLANK);

/// `RexxInfo~date`: the interpreter's build date, `__DATE__` reformatted as
/// `day month year` and the tail of [`VERSION`].
pub(crate) const BUILD_DATE: &[u8] = part(VERSION, LEVEL_BLANK + 1, VERSION.len());

/// The pointer width `Version.cpp:73` writes in front of `-bit`, from
/// `__REXX64__`. `RexxInfo~architecture` reads `sizeof(void *) * 8` instead
/// and the two must agree, which is what this constant exists to let a test
/// say.
#[cfg(test)]
const BIT_WIDTH: &[u8] = part(VERSION, SECOND_UNDERSCORE + 1, DASH);

const UNDERSCORE: usize = seek(VERSION, b'_', 0);
const PAREN: usize = seek(VERSION, b'(', UNDERSCORE);
#[cfg(test)]
const SECOND_UNDERSCORE: usize = seek(VERSION, b'_', PAREN);
#[cfg(test)]
const DASH: usize = seek(VERSION, b'-', SECOND_UNDERSCORE);
const FIRST_DOT: usize = seek(VERSION_NUMBER, b'.', 0);
const SECOND_DOT: usize = seek(VERSION_NUMBER, b'.', FIRST_DOT + 1);
const BIT_BLANK: usize = seek(VERSION, b' ', 0);
const LEVEL_BLANK: usize = seek(VERSION, b' ', BIT_BLANK + 1);

/// The offset of the first `byte` at or after `from`.
const fn seek(bytes: &[u8], byte: u8, from: usize) -> usize {
    let mut at = from;
    while at < bytes.len() {
        if bytes[at] == byte {
            return at;
        }
        at += 1;
    }
    panic!("VERSION does not carry the delimiter its fields are cut at")
}

const fn part(bytes: &'static [u8], from: usize, to: usize) -> &'static [u8] {
    bytes.split_at(to).0.split_at(from).1
}

/// The two bytes `PARSE` treats as whitespace when carving a section into
/// words: blank and tab, and nothing else (`RexxTarget::getWord`,
/// `instructions/ParseTarget.cpp:423`, `*scan == ' ' || *scan == '\t'`).
fn is_blank(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

/// Where one `PARSE` clause's templates get their strings.
enum ParseStrings {
    /// A single-string source. The slot empties on the first take, so a
    /// template past the first parses the null string -- which is the rule
    /// [`Interp::next_template`] wants and gets here for free.
    One(Option<SourceText>),
    /// `PARSE ARG`: the index of the argument the next template reads.
    Arg(usize),
}

/// Where one template's parse string lives while its triggers walk it.
enum SourceText {
    /// Bytes this walk owns, from [`Interp::parse_buffers`].
    Owned(Vec<u8>),
    /// Bytes carried in the handle, which is a `Copy` value this enum holds,
    /// so nothing in the arena reaches them.
    Inline(InlineText),
    /// A rooted `Body::Text`, read where it already is rather than copied,
    /// with the length it had when the walk started.
    ///
    /// **The slice is derived at each use and never held across a call that
    /// can allocate**: an allocation can grow the arena's slot vector and move
    /// the object's inline bytes with it. The borrow of `Interp` that
    /// [`SourceText::bytes`] hands back is what enforces that, since a `&mut
    /// self` call ends it. The length is the one thing kept rather than
    /// re-derived, and [`SourceText::bytes`] asserts it against the object.
    Text { value: ObjRef, length: usize },
}

impl SourceText {
    /// The parse string's bytes, wherever they live.
    ///
    /// # Panics
    ///
    /// When [`SourceText::Text`] no longer names a live `Body::Text`, which is
    /// a parse source that lost its root rather than anything a program can
    /// write. `Interp::source_text` is the one constructor of that arm and it
    /// roots what it borrows from.
    fn bytes<'a>(&'a self, interp: &'a Interp) -> &'a [u8] {
        match self {
            SourceText::Owned(buffer) => buffer,
            SourceText::Inline(inline) => inline,
            SourceText::Text { value, length } => match interp.heap.body_text(*value) {
                Some(bytes) => {
                    debug_assert_eq!(bytes.len(), *length, "the parse source changed length");
                    bytes
                }
                None => unreachable!("the parse source was collected mid-template"),
            },
        }
    }

    /// How long the parse string is, without reaching the arena for it.
    fn len(&self) -> usize {
        match self {
            SourceText::Owned(buffer) => buffer.len(),
            SourceText::Inline(inline) => inline.len(),
            SourceText::Text { length, .. } => *length,
        }
    }

    /// The pool buffer this source took, for a caller handing it back.
    fn into_buffer(self) -> Option<Vec<u8>> {
        match self {
            SourceText::Owned(buffer) => Some(buffer),
            SourceText::Inline(_) | SourceText::Text { .. } => None,
        }
    }
}

/// The positions a template's triggers move over its parse string.
pub(crate) struct Cursor {
    length: usize,
    /// The start of the section this trigger's targets are carved from.
    start: usize,
    /// One past its end.
    end: usize,
    /// Where the last match began. Every *relative* trigger measures from
    /// here.
    pattern_start: usize,
    /// One past where the last match ended -- the same position for a numeric
    /// trigger, `needle.len()` further on for a string one. Where the next
    /// *section* and the next string search begin.
    pattern_end: usize,
    /// How far into `start..end` the word carving has got.
    subcurrent: usize,
}

impl Cursor {
    /// A fresh cursor over a parse string of `length` bytes, every position at
    /// the origin (`RexxTarget::next`'s own reset).
    pub(crate) fn new(length: usize) -> Cursor {
        Cursor {
            length,
            start: 0,
            end: 0,
            pattern_start: 0,
            pattern_end: 0,
            subcurrent: 0,
        }
    }

    /// The implicit trailing trigger: the section runs from the end of the
    /// last match to the end of the string.
    fn move_to_end(&mut self) {
        self.start = self.pattern_end;
        self.pattern_end = self.length;
        self.pattern_start = self.length;
        self.end = self.length;
        self.subcurrent = self.start;
    }

    /// `+n`. Forward from the last match's start, **with the Rexx
    /// no-movement rule**: if the new position is not past the old one the
    /// section runs to the end of the string instead and the match position
    /// does not move. Measured: `p +0 q` gives both the whole string.
    fn forward(&mut self, offset: usize) {
        self.start = self.pattern_start;
        self.end = self.start.saturating_add(offset).min(self.length);
        if self.end <= self.start {
            self.end = self.length;
            self.pattern_start = self.start;
        } else {
            self.pattern_start = self.end;
        }
        self.pattern_end = self.pattern_start;
        self.subcurrent = self.start;
    }

    /// `>n`. The same movement as `+n` **without** the no-movement rule, so
    /// the section is an exact slice and an offset of zero gives the null
    /// string. Measured: `p >0 q` gives `p = ''`.
    fn forward_length(&mut self, offset: usize) {
        self.start = self.pattern_start;
        self.end = self.start.saturating_add(offset).min(self.length);
        self.pattern_start = self.end;
        self.pattern_end = self.pattern_start;
        self.subcurrent = self.start;
    }

    /// `=n`, and a bare numeric symbol. Column `n` is origin one, so a zero
    /// and a one are the same position -- measured, `p 0 q` and `p 1 q` both
    /// give the whole string twice.
    fn absolute(&mut self, column: usize) {
        let offset = column.saturating_sub(1);
        self.start = self.pattern_end;
        if offset <= self.start {
            self.end = self.length;
            self.pattern_start = offset;
        } else {
            self.end = offset.min(self.length);
            self.pattern_start = self.end;
        }
        self.pattern_end = self.pattern_start;
        self.subcurrent = self.start;
    }

    /// `-n`. The match position moves back, clamped at the origin, and the
    /// section runs from the *old* match position to the end of the string.
    /// So backward movement never assigns the null string -- measured, `p 5 q
    /// -99 r` gives `r` the whole string.
    fn backward(&mut self, offset: usize) {
        self.start = self.pattern_start;
        self.end = self.length;
        self.pattern_start = self.pattern_start.saturating_sub(offset);
        self.pattern_end = self.pattern_start;
        self.subcurrent = self.start;
    }

    /// `<n`. The `n` bytes *ending at* the match position, clamped at the
    /// origin. Not `-n` with a different sign: this is an exact slice and
    /// `p <0 q` gives `p = ''`.
    fn backward_length(&mut self, offset: usize) {
        self.start = self.pattern_start.saturating_sub(offset);
        self.end = self.pattern_start;
        self.pattern_start = self.start;
        self.pattern_end = self.start;
        self.subcurrent = self.start;
    }

    /// A literal or `(expr)` pattern. Searches from the end of the last
    /// match, so searches are non-overlapping.
    fn search(&mut self, string: &[u8], needle: &[u8]) {
        self.match_at(find(string, needle, self.pattern_end, false), needle.len());
    }

    /// [`search`] under `PARSE CASELESS`.
    fn caseless_search(&mut self, string: &[u8], needle: &[u8]) {
        self.match_at(find(string, needle, self.pattern_end, true), needle.len());
    }

    /// The half of a string search that is not the comparison: where the two
    /// searches put the five positions once they know whether they matched.
    fn match_at(&mut self, found: Option<usize>, needle_length: usize) {
        self.start = self.pattern_end;
        match found {
            Some(at) => {
                self.end = at;
                self.pattern_start = at;
                self.pattern_end = at + needle_length;
            }
            None => {
                self.end = self.length;
                self.pattern_start = self.length;
                self.pattern_end = self.length;
            }
        }
        self.subcurrent = self.start;
    }

    /// The next blank-delimited word of the current section, as a range into
    /// `string`, or an empty range once the section is used up.
    fn next_word(&mut self, string: &[u8]) -> std::ops::Range<usize> {
        debug_assert_eq!(
            string.len(),
            self.length,
            "the parse string changed length under the cursor"
        );
        if self.subcurrent >= self.end {
            return 0..0;
        }
        let mut scan = self.subcurrent;
        while scan < self.length && is_blank(string[scan]) {
            scan += 1;
        }
        self.subcurrent = scan;
        if self.subcurrent >= self.end {
            return 0..0;
        }
        let word_end = (self.subcurrent..self.end).find(|&i| is_blank(string[i]));
        match word_end {
            // No blank before the section's end: the rest of the section is
            // the word.
            None => {
                let word = self.subcurrent..self.end;
                self.subcurrent = self.end;
                word
            }
            // The terminating blank is consumed as well, so the next scan
            // starts past it.
            Some(at) => {
                let word = self.subcurrent..at;
                self.subcurrent = at + 1;
                word
            }
        }
    }

    /// Everything left in the current section, leading blanks included -- what
    /// the *last* target of a trigger gets. Measured: `p q r` on `'a  b  c'`
    /// gives `r = ' c'`, so only the final target keeps them.
    fn remainder(&mut self) -> std::ops::Range<usize> {
        if self.subcurrent >= self.end {
            return 0..0;
        }
        let rest = self.subcurrent..self.end;
        self.subcurrent = self.end;
        rest
    }
}

/// The first offset at or after `from` where `needle` occurs in `haystack`,
/// or `None`.
fn find(haystack: &[u8], needle: &[u8], from: usize, caseless: bool) -> Option<usize> {
    if needle.is_empty() || from > haystack.len() {
        return None;
    }
    // Searched as one slice from `from` rather than by indexing `haystack` at
    // each candidate offset: `windows` and `iter` both carry their own bound,
    // where `&haystack[at..at + needle.len()]` is a range check per position.
    let tail = &haystack[from..];
    if needle.len() > tail.len() {
        return None;
    }
    let at = match (needle, caseless) {
        // A one-byte pattern is the common case and the one a byte scan can
        // do without building a window per position -- `samples/rexxcps.rex`
        // searches for a single `b` on four of its inner loop's clauses.
        // `find_byte` is `builtin::string`'s own scan, a word at a time; its
        // doc has the argument for why the lowest flagged byte is the first
        // match, and the test that runs that argument rather than resting on
        // it.
        ([byte], false) => crate::builtin::string::find_byte(tail, *byte),
        ([byte], true) => {
            let folded = byte.to_ascii_lowercase();
            tail.iter()
                .position(|candidate| candidate.to_ascii_lowercase() == folded)
        }
        (_, false) => tail.windows(needle.len()).position(|w| w == needle),
        (_, true) => tail
            .windows(needle.len())
            .position(|w| w.eq_ignore_ascii_case(needle)),
    };
    at.map(|at| at + from)
}

/// How many source buffers [`Interp::parse_buffers`] parks between `PARSE`
/// instructions, and the widest one it will park.
const PARSE_BUFFERS_KEPT: usize = 8;
const PARSE_BUFFER_BYTES_KEPT: usize = 4096;

impl Interp {
    /// An empty buffer for a `PARSE` source string. See
    /// [`Interp::parse_buffers`].
    fn take_parse_buffer(&mut self) -> Vec<u8> {
        self.parse_buffers.pop().unwrap_or_default()
    }

    /// `value`'s text in a buffer from the pool.
    fn rendered_into_parse_buffer(&mut self, value: ObjRef) -> Vec<u8> {
        let mut buffer = self.take_parse_buffer();
        buffer.extend_from_slice(&self.to_text(value));
        buffer
    }

    /// Hands a source buffer back, if it is one worth keeping.
    fn give_parse_buffer(&mut self, mut buffer: Vec<u8>) {
        if buffer.capacity() == 0
            || buffer.capacity() > PARSE_BUFFER_BYTES_KEPT
            || self.parse_buffers.len() >= PARSE_BUFFERS_KEPT
        {
            return;
        }
        buffer.clear();
        self.parse_buffers.push(buffer);
    }

    /// Hands back whatever a spent source borrowed from the pool.
    fn give_source(&mut self, source: SourceText) {
        if let Some(buffer) = source.into_buffer() {
            self.give_parse_buffer(buffer);
        }
    }

    /// Hands back the string a template walk did not consume.
    fn give_parse_strings(&mut self, strings: ParseStrings) {
        match strings {
            ParseStrings::One(Some(source)) => self.give_source(source),
            ParseStrings::One(None) | ParseStrings::Arg(_) => {}
        }
    }

    /// `value` as a parse string: its own bytes where nothing can move them
    /// out from under the walk, and a copy in a pool buffer where they can.
    ///
    /// `Body::Text`'s bytes are written once, by the `alloc` that builds the
    /// object, and no other body in the value model holds bytes that are
    /// immutable for a whole clause -- a `MutableBuffer`'s contents are not.
    /// A folding `PARSE` always copies, because it rewrites the parse string
    /// in place.
    fn source_text(&mut self, value: ObjRef, parse: &Parse) -> SourceText {
        if !parse.upper && !parse.lower {
            match value.decode() {
                Decoded::Text(inline) => return SourceText::Inline(inline),
                Decoded::Heap { .. } => {
                    if let Some(length) = self.heap.body_text(value).map(<[u8]>::len) {
                        // Rooted where the borrow is taken, since from here
                        // the walk reads the object rather than a copy of it.
                        // The temporary lives until this clause ends, which
                        // outlasts the walk.
                        self.roots.push_temp(value);
                        return SourceText::Text { value, length };
                    }
                }
                _ => {}
            }
        }
        SourceText::Owned(self.rendered_into_parse_buffer(value))
    }

    /// One piece of the parse string in a buffer from the pool, for a caller
    /// that needs the bytes across a `&mut self` call.
    fn copied_piece(&mut self, source: &SourceText, piece: std::ops::Range<usize>) -> Vec<u8> {
        let mut buffer = self.take_parse_buffer();
        buffer.extend_from_slice(&source.bytes(self)[piece]);
        buffer
    }

    /// Carves one target's piece out of the parse string and builds its text
    /// value, **under a single borrow of the source**: the carving indexes the
    /// bytes and so does the value, so one derivation serves both.
    fn take_piece(
        &mut self,
        source: &SourceText,
        cursor: &mut Cursor,
        last: bool,
    ) -> (std::ops::Range<usize>, ObjRef) {
        let string = source.bytes(self);
        let piece = if last {
            cursor.remainder()
        } else {
            cursor.next_word(string)
        };
        let bytes = &string[piece.start..piece.end];
        if let Some(inline) = ObjRef::inline_text(bytes) {
            return (piece, inline);
        }
        let bytes = Bytes::from_slice(bytes);
        (piece, self.text_bytes(bytes))
    }

    /// `PARSE ARG`'s string for the template at argument position `at`.
    fn argument_text(&mut self, at: usize, parse: &Parse) -> Result<SourceText, Failure> {
        let argument = match self.call_context.arguments.get(at) {
            Some(Some(argument)) => Some(*argument),
            Some(None) | None => None,
        };
        match argument {
            // `PARSE ARG` converts each argument through the same
            // `ParseTarget::init` the other sources reach, and traces no
            // `>K>` of its own to disagree with.
            Some(value) => {
                let value = self.required_string_value(value)?;
                Ok(self.source_text(value, parse))
            }
            None => Ok(SourceText::Owned(self.take_parse_buffer())),
        }
    }

    /// One `PARSE` instruction: resolve the source, then walk the template.
    pub(crate) fn exec_parse(
        &mut self,
        code: &Code<'_>,
        parse: &Parse,
        evaluated: Option<ObjRef>,
    ) -> Result<(), Failure> {
        let indent = self.clause_state.current_value_indent;
        let mut strings = self.parse_strings(code, parse, indent, evaluated)?;
        let (mut source, mut cursor) = self.next_template(&mut strings, parse, indent)?;

        for entry in &parse.template {
            let Some(trigger) = entry else {
                // The comma fence is a template boundary and not a trigger:
                // it advances to the next parse string, which for `PARSE ARG`
                // is the next argument and for every other source is the null
                // string (`RexxTarget::next`'s own `next_argument != 1` arm).
                let (next_source, next_cursor) = self.next_template(&mut strings, parse, indent)?;
                cursor = next_cursor;
                let spent = std::mem::replace(&mut source, next_source);
                self.give_source(spent);
                continue;
            };
            self.apply_trigger(code, trigger, &source, &mut cursor, indent)?;
            self.assign_targets(code, trigger, &source, &mut cursor, indent)?;
        }
        // Only on the way out through the bottom: a template that leaves
        // through `?` above drops its buffers instead, which is the same
        // trade every lender in this interpreter makes.
        self.give_source(source);
        self.give_parse_strings(strings);
        Ok(())
    }

    /// Steps to the next parse string, applying `UPPER`/`LOWER` and tracing
    /// the result.
    fn next_template(
        &mut self,
        strings: &mut ParseStrings,
        parse: &Parse,
        indent: usize,
    ) -> Result<(SourceText, Cursor), Failure> {
        let mut source = match strings {
            // A template past the single string parses the null string, and
            // an empty `Vec` is that with nothing taken from the pool -- one
            // of zero capacity is what `give_parse_buffer` declines to park
            // anyway.
            ParseStrings::One(source) => source.take().unwrap_or(SourceText::Owned(Vec::new())),
            ParseStrings::Arg(index) => {
                let at = *index;
                *index += 1;
                self.argument_text(at, parse)?
            }
        };
        if parse.upper || parse.lower {
            // A folding source owns its bytes; `Interp::source_text` is where
            // that is decided.
            let SourceText::Owned(buffer) = &mut source else {
                unreachable!("a folding PARSE reached a borrowed parse string")
            };
            if parse.upper {
                buffer.make_ascii_uppercase();
            } else {
                buffer.make_ascii_lowercase();
            }
        }
        let cursor = Cursor::new(source.len());
        if self.traced_mode().results {
            let whole = self.copied_piece(&source, 0..cursor.length);
            self.trace_result(indent, &whole);
            self.give_parse_buffer(whole);
        }
        Ok((source, cursor))
    }

    /// The strings this `PARSE` will consume, in template order, and the
    /// source's own `>K>` line.
    fn parse_strings(
        &mut self,
        code: &Code<'_>,
        parse: &Parse,
        indent: usize,
        evaluated: Option<ObjRef>,
    ) -> Result<ParseStrings, Failure> {
        // Which of the two the source produces is what decides whether the
        // required-string protocol runs at all: a source that builds its own
        // bytes never had an object to convert.
        enum Subject {
            Bytes(Vec<u8>),
            Value(ObjRef),
        }
        let (keyword, value) = match &parse.source {
            // `PARSE VALUE WITH template`, with no expression at all, is
            // legal and parses the null string.
            ParseSource::Value(None) => {
                self.trace_literal(indent, b"");
                ("VALUE", Subject::Bytes(Vec::new()))
            }
            // **Rooted by whichever side produced it.** A value handed in is
            // already held by the register the compiled stream evaluated it
            // into, so a second temp here would root it against a frame this
            // clause does not own; a value evaluated here has nothing else
            // holding it across the `&mut self` calls the template walk makes.
            ParseSource::Value(Some(expression)) => {
                let value = match evaluated {
                    Some(value) => value,
                    None => {
                        let value = self.eval(code, expression)?;
                        self.roots.push_temp(value);
                        value
                    }
                };
                ("VALUE", Subject::Value(value))
            }
            // An ordinary variable read, with everything that implies: `>C>`
            // and `>V>` for a compound, and `NOVALUE` for an unset name --
            // measured, `signal on novalue` traps on `parse var zzunset t`.
            ParseSource::Var(id) => {
                let value = self.read_parse_var(code, *id, indent)?;
                self.roots.push_temp(value);
                ("VAR", Subject::Value(value))
            }
            // The second word is the *calling context* rather than the call
            // depth, and it is the running activation's rather than this
            // clause's: `crate::activation::CallType` carries the measured
            // table and is set where each activation is built.
            ParseSource::Source => {
                let mut source = self.take_parse_buffer();
                source.extend_from_slice(PLATFORM);
                source.push(b' ');
                source.extend_from_slice(self.activation().call_type.token());
                source.push(b' ');
                // The third word is the executable's own name, which is the
                // program's path for a program and the method's name for a
                // method compiled from source text -- measured, oracle rc 0,
                // `parse source` inside a `setMethod` body answers `LINUX
                // METHOD MM`.
                let program = self.activation().program_id;
                source.extend_from_slice(self.program_display_name(program));
                ("SOURCE", Subject::Bytes(source))
            }
            ParseSource::Version => {
                let mut version = self.take_parse_buffer();
                version.extend_from_slice(VERSION);
                ("VERSION", Subject::Bytes(version))
            }
            // No `>K>` line of any kind, and the one source with more than
            // one string (`RexxInstructionParse::execute`'s own `SUBKEY_ARG`
            // arm, which is the only one that does not call
            // `traceKeywordResult`).
            ParseSource::Arg => return Ok(ParseStrings::Arg(0)),
            // The two sources that read a line rather than evaluating a value.
            // The split between them is entirely in which reader is called --
            // `PULL` takes the queue's head when there is one, `LINEIN` never
            // consults the queue at all -- and `input.rs` owns that rule and
            // the measurements behind it.
            ParseSource::Pull => ("PULL", Subject::Bytes(self.pull_line()?)),
            ParseSource::LineIn => ("LINEIN", Subject::Bytes(self.linein_line()?)),
        };
        let value = match value {
            Subject::Bytes(bytes) => {
                self.trace_keyword(indent, keyword, &bytes);
                SourceText::Owned(bytes)
            }
            // **`>K>` names the object and the template walks the
            // conversion.** `RexxInstructionParse::execute` traces `value`
            // and hands the same object to the target, whose own `init` calls
            // `string->requestString()` (`instructions/ParseTarget.cpp:113`).
            // Measured, `trace r` over `parse value .K with a b` with a
            // class-side `makeString` returning `'p q'`:
            // `>K>   "VALUE" => "The K class"` and then `>>>   "p q"`.
            Subject::Value(value) => {
                // The rendering exists for that line and for nothing else, so
                // it is taken under the line's own gate.
                if self.traced_mode().results {
                    let traced = self.rendered_into_parse_buffer(value);
                    self.trace_keyword(indent, keyword, &traced);
                    self.give_parse_buffer(traced);
                }
                let converted = self.required_string_value(value)?;
                self.source_text(converted, parse)
            }
        };
        Ok(ParseStrings::One(Some(value)))
    }

    /// `PARSE VAR`'s source: the variable read, in all three name shapes.
    fn read_parse_var(
        &mut self,
        code: &Code<'_>,
        id: rexx_parse::SymbolId,
        indent: usize,
    ) -> Result<ObjRef, Failure> {
        // Borrowed: `SymbolTable::name` answers with the `Code`'s lifetime,
        // which is the program and not this `Interp`, so it survives the
        // `&mut self` calls below. The renderings beneath are taken only when
        // a line will print them -- `trace_variable` and
        // `trace_compound_name` both return at once unless intermediates are
        // on, and this function is on the `PARSE` path, which
        // `samples/rexxcps.rex` runs on every clause of its inner loop.
        let name = code.symbols.name(id).as_bytes();
        match crate::run::shape_of(name) {
            crate::run::NameShape::Simple => {
                let (value, novalue) = self.read(code, id);
                self.novalue_check(novalue, value)?;
                if self.tracing_intermediates() {
                    let text = self.to_text(value).to_vec();
                    self.trace_variable(indent, name, &text);
                }
                Ok(value)
            }
            // A bare stem read raises no `NOVALUE` (`eval_node`'s own arm has
            // the measured pair) and announces no resolved name.
            crate::run::NameShape::Stem => {
                let value = self.read_stem(name);
                if self.tracing_intermediates() {
                    let text = self.to_text(value).to_vec();
                    self.trace_variable(indent, name, &text);
                }
                Ok(value)
            }
            crate::run::NameShape::Compound => {
                let (stem_name, stem_at) = code.stem(id);
                let mut key = self.take_key_buffer();
                if let Err(failure) = self.tail_key_into(code, id, &mut key) {
                    self.give_key_buffer(key);
                    return Err(failure);
                }
                if self.tracing_intermediates() {
                    let mut resolved = stem_name.to_vec();
                    resolved.extend_from_slice(&key);
                    self.trace_compound_name(indent, name, &resolved);
                }
                let (value, novalue) = self.stem_get_at(stem_name, stem_at, &key);
                self.give_key_buffer(key);
                self.novalue_check(novalue, value)?;
                if self.tracing_intermediates() {
                    let text = self.to_text(value).to_vec();
                    self.trace_variable(indent, name, &text);
                }
                Ok(value)
            }
        }
    }

    /// Evaluates one trigger's operand, if it has one, and moves the cursor.
    fn apply_trigger(
        &mut self,
        code: &Code<'_>,
        trigger: &ParseTrigger,
        source: &SourceText,
        cursor: &mut Cursor,
        indent: usize,
    ) -> Result<(), Failure> {
        // `End` is the only kind with no operand, so every other arm below
        // can ask for one. A missing operand on a kind that needs one is not
        // representable from a program that parsed -- `parse_template`
        // (`rexx-parse`) fills `value` on every trigger it builds except
        // `End`, and refuses a trigger with nothing after it (38.901) -- and
        // it is reported as the gap it would be rather than defaulted to a
        // position.
        let operand = match trigger.kind {
            TriggerKind::End => {
                cursor.move_to_end();
                return Ok(());
            }
            _ => match &trigger.value {
                Some(operand) => operand,
                None => return Err(Loud::parse_trigger_operand().into()),
            },
        };
        let value = self.eval(code, operand)?;
        self.roots.push_temp(value);
        // `>>>` for the operand, after `eval`'s own `>L>`/`>V>` and before
        // the conversion that can fail: `integerTrigger`
        // (`ParseTrigger.cpp:143`-`153`) traces and only then converts, so
        // the operand's own value line is emitted even on the 26.4 path.
        // **Copied only where a copy is needed**, which is neither of the two
        // arms below. `trace_result` returns at once unless `results` is on,
        // and a search reads the bytes where they already are: `Cursor` is a
        // local of `exec_parse` rather than a field, so a shared borrow of
        // the value can be live while it is written to. The unconditional
        // `to_vec` here was 5.2% of every allocation `samples/rexxcps.rex`
        // made -- one per trigger, on a program whose templates are mostly
        // numeric triggers that never look at the bytes at all.
        if self.trace_mode().results {
            let rendered = self.to_text(value).to_vec();
            self.trace_result(indent, &rendered);
        }
        match trigger.kind {
            // **The needle is rendered before the source slice is taken**, not
            // after: `render` can allocate, and the borrow below may not
            // outlive an allocation. `Rendered` re-derives its own bytes for
            // the same reason [`SourceText`] does.
            TriggerKind::String => {
                let needle = self.render(value);
                cursor.search(source.bytes(self), needle.text(self));
                Ok(())
            }
            TriggerKind::Mixed => {
                let needle = self.render(value);
                cursor.caseless_search(source.bytes(self), needle.text(self));
                Ok(())
            }
            _ => {
                // 26.4, and the conversion is bounded by the **active**
                // `NUMERIC DIGITS` rather than by a fixed width
                // (`integerTrigger`'s own `requestUnsignedNumber(result,
                // number_digits())`). Measured: `numeric digits 2; parse
                // value d with p +(100) q` is 26.4 on the oracle, as are a
                // fractional operand, a non-numeric one and a negative one.
                // The substitution is the operand's own rendering -- measured
                // `found "1E2"` for `+(1e2)`, which is D15's rule showing
                // through rather than a re-rendering here.
                let Some(offset) = self.whole_nonneg(value) else {
                    // The one place the operand's own rendering is still
                    // owned, and it is the raise's substitution.
                    return Err(Raised::syntax(26, 4, vec![self.to_text(value).to_vec()]).into());
                };
                // Clamped rather than checked: an offset past `usize` is
                // indistinguishable from one past the string's own end, and
                // every operation below treats those the same way.
                let offset = usize::try_from(offset).unwrap_or(usize::MAX);
                match trigger.kind {
                    TriggerKind::Plus => cursor.forward(offset),
                    TriggerKind::Minus => cursor.backward(offset),
                    TriggerKind::Absolute => cursor.absolute(offset),
                    TriggerKind::PlusLength => cursor.forward_length(offset),
                    TriggerKind::MinusLength => cursor.backward_length(offset),
                    // Handled above; the operand kinds and the movement
                    // kinds partition `TriggerKind`, and this arm is what
                    // keeps that a compile-time exhaustive match rather than
                    // an assumption.
                    TriggerKind::End | TriggerKind::String | TriggerKind::Mixed => {}
                }
                Ok(())
            }
        }
    }

    /// Carves the current section into this trigger's targets: a blank
    /// delimited word each, and the whole remainder for the last one.
    fn assign_targets(
        &mut self,
        code: &Code<'_>,
        trigger: &ParseTrigger,
        source: &SourceText,
        cursor: &mut Cursor,
        indent: usize,
    ) -> Result<(), Failure> {
        let last = trigger.targets.len().saturating_sub(1);
        // **Decided once for the whole trigger**, which is where
        // `ParseTrigger::parse` decides it: it keeps two copies of this loop
        // and chooses between them on `context->tracingResults()`
        // (`ParseTrigger.cpp:248` and `:290`). Nothing the loop below reaches
        // runs Rexx code, so the setting cannot move while it runs, and the
        // assertion inside is what says so rather than leaving it argued.
        let traced = self.traced_mode();
        for (index, target) in trigger.targets.iter().enumerate() {
            debug_assert_eq!(
                traced,
                self.traced_mode(),
                "the TRACE setting moved while one trigger's targets were assigned"
            );
            match target {
                Some(target) => {
                    // **The value is built inside this arm because the other
                    // arm has nothing to assign it to.** A `.` consumes its
                    // field and stores nowhere, so a value built ahead of the
                    // match is created, rooted and dropped unread.
                    let (piece, value) = self.take_piece(source, cursor, index == last);
                    self.roots.push_temp(value);
                    // **The slot the upfront pass already bound this target
                    // to.** No compiler resolves a `PARSE` target -- nothing
                    // promotes the instruction -- but `Plan::build` walks
                    // every instruction of the body and binds the names each
                    // one writes, `note_parse` included, so the answer exists
                    // here and used to be recomputed from the name on every
                    // firing. Measured on `samples/rexxcps.rex` before this
                    // changed: of the `slot_of` calls that program makes,
                    // 2,800,003 were `PARSE` targets and every one of them
                    // resolved to the slot `by_symbol` already held.
                    let bound = match &target.kind {
                        ExprKind::Variable(id) => code.slot_for(*id).map(|slot| (*id, slot)),
                        _ => None,
                    };
                    // One copy of the piece for whichever of the two lines
                    // below will print it, taken only where one of them will
                    // -- the parse string itself may not be borrowed across
                    // the assignment, which is a `&mut self` call.
                    let shown = (traced.intermediates || traced.results)
                        .then(|| self.copied_piece(source, piece));
                    let rendered = if traced.intermediates {
                        shown.as_deref()
                    } else {
                        None
                    };
                    if let Some((id, slot)) = bound {
                        self.assign_bound_variable(code, id, slot, value, rendered, indent);
                    } else {
                        self.assign_expr_target(code, target, value, rendered, indent, None)?;
                    }
                    // The `TRACE R` half of the pair -- see this module's own
                    // `exec_parse` doc for why it is a choice of prefix and
                    // not a second, independent line.
                    if traced.results && !traced.intermediates {
                        let shown = shown
                            .as_deref()
                            .expect("a results line is one of the two the copy above is for");
                        self.trace_result(indent, shown);
                    }
                    if let Some(buffer) = shown {
                        self.give_parse_buffer(buffer);
                    }
                }
                // The `.` placeholder consumes a field and assigns nothing,
                // and its own line is emitted **even when it consumed
                // nothing** -- measured, `parse value 'one two' with p . q`
                // traces `>.>   ""` between `>=> P` and `>=> Q`.
                None => {
                    let piece = if index == last {
                        cursor.remainder()
                    } else {
                        cursor.next_word(source.bytes(self))
                    };
                    if traced.intermediates {
                        let shown = self.copied_piece(source, piece);
                        self.trace_dummy(indent, &shown);
                        self.give_parse_buffer(shown);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The measured oracle answers for one template, as `(template, pieces)`.
    /// Every row was taken from the oracle on 2026-08-05 (this task's report
    /// carries the transcripts); the driver below replays only the *movement*,
    /// so a row here is a claim about [`Cursor`] alone.
    fn pieces(source: &str, template: &[Trigger]) -> Vec<String> {
        let string = source.as_bytes();
        let mut cursor = Cursor::new(string.len());
        let mut out = Vec::new();
        for step in template {
            match step {
                Trigger::End => cursor.move_to_end(),
                Trigger::Plus(n) => cursor.forward(*n),
                Trigger::Minus(n) => cursor.backward(*n),
                Trigger::Absolute(n) => cursor.absolute(*n),
                Trigger::PlusLength(n) => cursor.forward_length(*n),
                Trigger::MinusLength(n) => cursor.backward_length(*n),
                Trigger::Search(needle) => cursor.search(string, needle.as_bytes()),
                Trigger::Caseless(needle) => cursor.caseless_search(string, needle.as_bytes()),
                Trigger::Targets(count) => {
                    for index in 0..*count {
                        let piece = if index + 1 == *count {
                            cursor.remainder()
                        } else {
                            cursor.next_word(string)
                        };
                        out.push(String::from_utf8_lossy(&string[piece]).into_owned());
                    }
                }
            }
        }
        out
    }

    /// A template step, for [`pieces`]. `Targets(n)` is the assignment loop
    /// belonging to the trigger before it, which is where a template's
    /// targets attach (this module's own doc).
    enum Trigger {
        End,
        Plus(usize),
        Minus(usize),
        Absolute(usize),
        PlusLength(usize),
        MinusLength(usize),
        Search(&'static str),
        Caseless(&'static str),
        Targets(usize),
    }
    use Trigger::*;

    const D: &str = "abcdefghij";

    /// `-n` and `<n` are unrelated operations. The pair this crate was warned
    /// would be conflated, and the reason the two match positions exist.
    #[test]
    fn minus_and_minus_length_differ_on_the_same_movement() {
        assert_eq!(
            pieces(
                D,
                &[
                    Absolute(5),
                    Targets(1),
                    Minus(2),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abcd", "efghij", "cdefghij"]
        );
        assert_eq!(
            pieces(
                D,
                &[
                    Absolute(5),
                    Targets(1),
                    MinusLength(2),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abcd", "cd", "cdefghij"]
        );
    }

    /// `=n` and `+n`/`-n` share one rule: forward of the current position
    /// gives `[current, new)`, and anything else -- including *equal* -- gives
    /// `[current, END]`.
    #[test]
    fn absolute_and_relative_share_the_backward_rule() {
        assert_eq!(
            pieces(D, &[Absolute(5), Targets(1), End, Targets(1)]),
            ["abcd", "efghij"]
        );
        // 1 is not greater than 1, so the second target is the remainder and
        // not the null string.
        assert_eq!(
            pieces(D, &[Absolute(1), Targets(1), End, Targets(1)]),
            ["abcdefghij", "abcdefghij"]
        );
        assert_eq!(
            pieces(D, &[Absolute(11), Targets(1), End, Targets(1)]),
            ["abcdefghij", ""]
        );
        assert_eq!(
            pieces(
                D,
                &[
                    Absolute(5),
                    Targets(1),
                    Absolute(5),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abcd", "efghij", "efghij"]
        );
        assert_eq!(
            pieces(
                D,
                &[
                    Absolute(5),
                    Targets(1),
                    Minus(99),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abcd", "efghij", "abcdefghij"]
        );
        assert_eq!(
            pieces(D, &[Plus(0), Targets(1), End, Targets(1)]),
            ["abcdefghij", "abcdefghij"]
        );
    }

    /// `>n`/`<n` do **not** share it: they are exact slices, clamped at the
    /// ends, and an offset of zero gives the null string where `+0` gives the
    /// whole remainder.
    #[test]
    fn length_triggers_are_exact_slices() {
        assert_eq!(
            pieces(D, &[PlusLength(0), Targets(1), End, Targets(1)]),
            ["", "abcdefghij"]
        );
        assert_eq!(
            pieces(D, &[MinusLength(0), Targets(1), End, Targets(1)]),
            ["", "abcdefghij"]
        );
        assert_eq!(
            pieces(
                D,
                &[
                    PlusLength(3),
                    Targets(1),
                    MinusLength(2),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["abc", "bc", "bcdefghij"]
        );
        assert_eq!(
            pieces(D, &[PlusLength(20), Targets(1), End, Targets(1)]),
            ["abcdefghij", ""]
        );
        assert_eq!(
            pieces(D, &[MinusLength(20), Targets(1), End, Targets(1)]),
            ["", "abcdefghij"]
        );
    }

    /// An absent pattern matches at END, the empty pattern behaves as absent,
    /// and a pattern at position 1 gives the null string. Searches are
    /// non-overlapping and the next one starts after the previous match.
    #[test]
    fn string_patterns_match_at_end_when_absent() {
        assert_eq!(
            pieces(D, &[Search("z"), Targets(1), End, Targets(1)]),
            ["abcdefghij", ""]
        );
        assert_eq!(
            pieces(D, &[Search(""), Targets(1), End, Targets(1)]),
            ["abcdefghij", ""]
        );
        assert_eq!(
            pieces(D, &[Search("a"), Targets(1), End, Targets(1)]),
            ["", "bcdefghij"]
        );
        assert_eq!(
            pieces(
                "aXbXc",
                &[
                    Search("X"),
                    Targets(1),
                    Search("X"),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["a", "b", "c"]
        );
        assert_eq!(
            pieces(
                "aXXb",
                &[
                    Search("XX"),
                    Targets(1),
                    Search("X"),
                    Targets(1),
                    End,
                    Targets(1)
                ]
            ),
            ["a", "b", ""]
        );
    }

    /// After a string pattern the next *target* starts past the match, but a
    /// following *relative* trigger measures from the match's START -- the
    /// subtle one, and the reason `pattern_start` and `pattern_end` are two
    /// fields.
    #[test]
    fn a_relative_trigger_after_a_pattern_measures_from_the_match_start() {
        let plain = pieces(D, &[Search("c"), Targets(1), End, Targets(1)]);
        assert_eq!(plain, ["ab", "defghij"]);
        // `+1` moves to exactly where the match ended, so the End trigger's
        // own section is the same one it would have been with no trigger at
        // all.
        assert_eq!(
            pieces(D, &[Search("c"), Targets(1), Plus(1), End, Targets(1)]),
            plain
        );
        assert_eq!(
            pieces(D, &[Search("c"), Targets(1), Minus(1), End, Targets(1)]),
            ["ab", "bcdefghij"]
        );
        assert_eq!(
            pieces(
                D,
                &[Search("c"), Targets(1), MinusLength(1), End, Targets(1)]
            ),
            ["ab", "bcdefghij"]
        );
    }

    /// `CASELESS` folds ASCII letters and nothing else. The three
    /// non-matching rows are byte pairs exactly `0x20` apart that are not
    /// letters, plus a high byte in each direction.
    #[test]
    fn caseless_folds_ascii_letters_only() {
        assert_eq!(
            pieces("aZb", &[Caseless("z"), Targets(1), End, Targets(1)]),
            ["a", "b"]
        );
        assert_eq!(
            pieces(
                "a\u{7b}b",
                &[Caseless("\u{5b}"), Targets(1), End, Targets(1)]
            ),
            ["a{b", ""]
        );
        assert_eq!(
            pieces(
                "a\u{5f}b",
                &[Caseless("\u{3f}"), Targets(1), End, Targets(1)]
            ),
            ["a_b", ""]
        );
        // A byte alphabet, so the two high bytes are compared as bytes and
        // not as text: 0xc9 and 0xe9 are 0x20 apart and match only
        // themselves.
        let string = [b'a', 0xc9, b'b'];
        let mut high = Cursor::new(string.len());
        high.caseless_search(&string, &[0xe9]);
        let piece = high.remainder();
        assert_eq!(string[piece], [b'a', 0xc9, b'b']);
        let mut same = Cursor::new(string.len());
        same.caseless_search(&string, &[0xc9]);
        let piece = same.remainder();
        assert_eq!(string[piece], [b'a']);
    }

    /// Word carving: only the final target keeps its leading blanks, extra
    /// targets get the null string, and a tab is whitespace.
    #[test]
    fn word_carving_keeps_leading_blanks_only_on_the_last_target() {
        assert_eq!(pieces("a  b  c", &[End, Targets(3)]), ["a", "b", " c"]);
        assert_eq!(pieces("a  b  c", &[End, Targets(2)]), ["a", " b  c"]);
        assert_eq!(pieces("  a b  ", &[End, Targets(2)]), ["a", "b  "]);
        assert_eq!(pieces("a b", &[End, Targets(4)]), ["a", "b", "", ""]);
        assert_eq!(pieces("a\tb", &[End, Targets(2)]), ["a", "b"]);
    }

    /// Every `TriggerKind` reaches the [`Cursor`] operation that belongs to
    /// it, and every source reaches its own string -- run through
    /// `run_program`, so the `step` arm and [`Interp::apply_trigger`]'s own
    /// dispatch are inside what is being tested.
    #[test]
    fn every_trigger_kind_and_source_reaches_its_own_operation() {
        let d = "d = 'abcdefghij'\n";
        let show3 = "say '['||p||']['||q||']['||r||']'\n";
        let show2 = "say '['||p||']['||q||']'\n";
        let rows: &[(&str, &str)] = &[
            // `+n` and `>n` agree here; `-n` and `<n` do not agree with
            // either or with each other.
            ("parse value d with p 5 q +2 r\n", "[abcd][ef][ghij]\n"),
            (
                "parse value d with p 5 q -2 r\n",
                "[abcd][efghij][cdefghij]\n",
            ),
            ("parse value d with p 5 q <2 r\n", "[abcd][cd][cdefghij]\n"),
            ("parse value d with p 5 q >2 r\n", "[abcd][ef][ghij]\n"),
        ];
        for (template, expected) in rows {
            let source = format!("{d}{template}{show3}");
            let outcome = crate::run_program(
                "/tmp/parse-dispatch.rex",
                source.into_bytes(),
                crate::Invocation::none(),
            );
            assert_eq!(
                String::from_utf8_lossy(&outcome.stdout),
                *expected,
                "{template}"
            );
        }

        let pairs: &[(&str, &str)] = &[
            // The offset that separates `+n` from `>n`: the no-movement rule
            // applies to one and not the other.
            ("parse value d with p +0 q\n", "[abcdefghij][abcdefghij]\n"),
            ("parse value d with p >0 q\n", "[][abcdefghij]\n"),
            // A bare numeric symbol is the same absolute column `=n` is.
            ("parse value d with p 2 q\n", "[a][bcdefghij]\n"),
            ("parse value d with p =2 q\n", "[a][bcdefghij]\n"),
            // `String` against `Mixed`: the same needle, and only the
            // caseless one matches.
            ("parse value 'aXb' with p 'x' q\n", "[aXb][]\n"),
            ("parse caseless value 'aXb' with p 'x' q\n", "[a][b]\n"),
            // `End`, which has no operand at all.
            ("parse value 'w1 w2 w3' with p q\n", "[w1][w2 w3]\n"),
            // The sources: each has to reach its own string rather than
            // another source's.
            ("vv = 'v1 v2'; parse var vv p q\n", "[v1][v2]\n"),
            ("parse value 'e1 e2' with p q\n", "[e1][e2]\n"),
        ];
        for (template, expected) in pairs {
            let source = format!("{d}{template}{show2}");
            let outcome = crate::run_program(
                "/tmp/parse-dispatch.rex",
                source.into_bytes(),
                crate::Invocation::none(),
            );
            assert_eq!(
                String::from_utf8_lossy(&outcome.stdout),
                *expected,
                "{template}"
            );
        }
    }

    /// A `PARSE ARG` template past the first reads an argument that every
    /// earlier template's allocations have had a chance to collect.
    #[test]
    fn a_late_parse_arg_template_survives_collection() {
        // **The arguments are built by concatenation rather than written as
        // literals**, and that is what gives this test its teeth: a literal is
        // interned where the collector cannot take it, so a template reading
        // one would answer correctly however badly it was rooted.
        let source = concat!(
            "call sub 'first' 'argument here', 'second' 'argument here'\n",
            "exit\n",
            "sub:\n",
            "parse arg a1 a2 a3, b1 b2 b3\n",
            "say '['a1']['a2']['a3']['b1']['b2']['b3']'\n",
            "return\n",
        );
        let expected = "[first][argument][here][second][argument][here]\n";
        let stressed = crate::run_program_collect_every_alloc(
            "/tmp/parse-arg-collect.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(String::from_utf8_lossy(&stressed.stdout), expected);
        assert_eq!(stressed.exit_code, 0);
        assert!(
            stressed.collections > 0,
            "the stress mode collected nothing, so this proves nothing"
        );
        let ordinary = crate::run_program(
            "/tmp/parse-arg-collect.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(String::from_utf8_lossy(&ordinary.stdout), expected);
    }

    /// A template walk whose own allocations collect, over a parse string the
    /// walk reads where it lives rather than out of a copy.
    ///
    /// **Every piece below is longer than `INLINE_TEXT`**, so each assignment
    /// allocates and, under the stress mode, collects -- which is what puts a
    /// collection between one target and the next. **Every parse string is
    /// built by concatenation** rather than written as a literal, for the
    /// reason [`a_late_parse_arg_template_survives_collection`] gives: a
    /// literal is interned where the collector cannot take it.
    ///
    /// **The last template writes over the compound it is reading**, and the
    /// object was built inside `fill` so no live register still holds it:
    /// from the second target on, the walk's own temporary is what keeps it.
    /// Measured with every root on the path deleted -- `Interp::source_text`'s,
    /// the `PARSE VAR` arm's and `required_string_value`'s -- the stress run
    /// sweeps the source between two of this template's targets and
    /// `SourceText::bytes` panics here. Any one of them left in place keeps it
    /// alive, so this asserts that the source is rooted and not that a
    /// particular line roots it.
    #[test]
    fn a_template_walk_survives_a_collection_between_its_targets() {
        let source = concat!(
            "vr = 'alphabetic' 'bookkeeper' 'cannonball' 'dreadnought'\n",
            "parse var vr w1 w2 w3 w4\n",
            "say '['w1']['w2']['w3']['w4']'\n",
            "pat = '=' || '='\n",
            "st = 'leftmostpiece' || pat || 'rightmostpiece'\n",
            "parse var st a1 (pat) a2\n",
            "say '['a1']['a2']'\n",
            "parse value 'valuepiece' 'secondpiece' 'thirdpiece' with v1 . v3\n",
            "say '['v1']['v3']'\n",
            "parse value 'columnarpiece' || 'tailingpiece' with c1 14 c2\n",
            "say '['c1']['c2']'\n",
            "call fill\n",
            "parse var sm.1 s1 sm.1 s3 s4\n",
            "say '['s1']['sm.1']['s3']['s4']'\n",
            "exit\n",
            "fill:\n",
            "sm.1 = 'firstliteral' 'secondliteral' 'thirdliteral' 'fourthliteral'\n",
            "return\n",
        );
        let expected = concat!(
            "[alphabetic][bookkeeper][cannonball][dreadnought]\n",
            "[leftmostpiece][rightmostpiece]\n",
            "[valuepiece][thirdpiece]\n",
            "[columnarpiece][tailingpiece]\n",
            "[firstliteral][secondliteral][thirdliteral][fourthliteral]\n",
        );
        let stressed = crate::run_program_collect_every_alloc(
            "/tmp/parse-in-place-collect.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(String::from_utf8_lossy(&stressed.stdout), expected);
        assert_eq!(stressed.exit_code, 0);
        assert!(
            stressed.collections > 0,
            "the stress mode collected nothing, so this proves nothing"
        );
        let ordinary = crate::run_program(
            "/tmp/parse-in-place-collect.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(String::from_utf8_lossy(&ordinary.stdout), expected);
        assert_eq!(ordinary.exit_code, 0);
    }

    /// `PARSE SOURCE` and `PARSE VERSION` reach their own strings, which no
    /// other source's answer can be mistaken for.
    #[test]
    fn source_and_version_carry_their_own_strings() {
        let outcome = crate::run_program(
            "/tmp/parse-source.rex",
            b"parse source s\nsay s\nparse version v\nsay v\n".to_vec(),
            crate::Invocation::none(),
        );
        let mut expected = b"LINUX COMMAND /tmp/parse-source.rex\n".to_vec();
        expected.extend_from_slice(VERSION);
        expected.push(b'\n');
        assert_eq!(outcome.stdout, expected);
    }

    /// The `RexxInfo` fields reassemble into [`VERSION`] exactly as
    /// `runtime/Version.cpp:73` assembles it.
    #[test]
    fn the_version_fields_reassemble_into_the_constant() {
        let mut rebuilt = b"REXX-ooRexx_".to_vec();
        rebuilt.extend_from_slice(MAJOR_VERSION);
        rebuilt.push(b'.');
        rebuilt.extend_from_slice(RELEASE);
        rebuilt.push(b'.');
        rebuilt.extend_from_slice(MODIFICATION);
        rebuilt.extend_from_slice(b"(MT)_");
        rebuilt.extend_from_slice(BIT_WIDTH);
        rebuilt.extend_from_slice(b"-bit ");
        rebuilt.extend_from_slice(LANGUAGE_LEVEL);
        rebuilt.push(b' ');
        rebuilt.extend_from_slice(BUILD_DATE);
        assert_eq!(rebuilt, VERSION);
        let mut number = MAJOR_VERSION.to_vec();
        number.push(b'.');
        number.extend_from_slice(RELEASE);
        number.push(b'.');
        number.extend_from_slice(MODIFICATION);
        assert_eq!(number, VERSION_NUMBER);
        assert_eq!(
            BIT_WIDTH,
            (size_of::<*const ()>() * 8).to_string().as_bytes(),
            "`__REXX64__` in Version.cpp:73 and `sizeof(void *) * 8` in \
             RexxInfo::getArchitecture are the same fact, so a VERSION whose \
             width disagrees with this build would make `RexxInfo~architecture` \
             and `RexxInfo~name` contradict each other"
        );
    }

    /// A fractional, non-numeric or negative positional operand is 26.4, and
    /// the conversion is bounded by the **active** `NUMERIC DIGITS` rather
    /// than by a fixed width -- measured, `numeric digits 2` makes `+(100)`
    /// fail where the default 9 digits accept it.
    #[test]
    fn a_positional_operand_must_be_a_whole_number_within_the_active_digits() {
        let refused: &[&str] = &[
            "parse value 'abc' with p 2.5 q\n",
            "parse value 'abc' with p +('x') q\n",
            "parse value 'abc' with p +(-1) q\n",
            "numeric digits 2\nparse value 'abc' with p +(100) q\n",
        ];
        for source in refused {
            let outcome = crate::run_program(
                "/tmp/parse-26-4.rex",
                source.as_bytes().to_vec(),
                crate::Invocation::none(),
            );
            assert_eq!(outcome.exit_code, 230, "{source}");
            assert!(
                String::from_utf8_lossy(&outcome.stderr).contains(
                    "Error 26.4:  Positional pattern of PARSE template must be a whole number"
                ),
                "{source}: {}",
                String::from_utf8_lossy(&outcome.stderr)
            );
        }
        let accepted = crate::run_program(
            "/tmp/parse-26-4.rex",
            b"parse value 'abcdefghij' with p +(100) q\nsay '['||p||']'\n".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(accepted.exit_code, 0);
        assert_eq!(
            String::from_utf8_lossy(&accepted.stdout),
            "[abcdefghij]\n",
            "an operand past the string's own end is a clamp, not an error"
        );
    }
}
