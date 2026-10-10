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

//! The word builtins: `DELWORD`, `SUBWORD`, `WORD`, `WORDINDEX`,
//! `WORDLENGTH`, `WORDPOS` and `WORDS`.
//! ```text
//! word/wordindex/wordlength at 1     aa   1  2
//! word/wordindex/wordlength at 3     cc   8  2
//! word/wordindex/wordlength at 4     ''   0  0        one past the last
//! word/wordindex/wordlength at 999999 ''  0  0
//! word/wordindex/wordlength at 0                      93.924, rc 163
//! word/wordindex/wordlength at -1                     93.924, rc 163
//! ```
//! ```rexx
//! numeric digits 12
//! n = words('a b c d e f g h i j')   -- created here, renders as 10
//! numeric digits 1
//! say n                              -- still 10: the pair is captured
//! say n + 0                          -- 1E+1: a new value, new pair
//! ```

use std::ops::Range;

use rexx_core::ObjRef;

use super::{Args, length_of, position_of, required_render, required_string, whole_number};
use crate::Interp;
use crate::error::Failure;

/// The default word count `SUBWORD` and `DELWORD` use when their third
/// argument is omitted.
const ALL_REMAINING_WORDS: usize = 999_999_999_999_999_999;

/// Whether `byte` separates words: blank or horizontal tab, and no other
/// byte -- see the module doc for the C++ and the 256-byte measurement.
fn is_blank(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

/// A scan over the words of a byte string, mirroring
/// `RexxString::WordIterator` (`classes/StringClass.hpp`).
struct Words<'a> {
    text: &'a [u8],
    /// Where the next scan starts. The C++ carries a pointer and a remaining
    /// length; one index says the same thing, since the remaining length is
    /// always `text.len() - next`.
    next: usize,
    /// The word [`step`] last found, as a byte range of `text`.
    word: Range<usize>,
}

impl<'a> Words<'a> {
    fn new(text: &'a [u8]) -> Self {
        Words {
            text,
            next: 0,
            word: 0..0,
        }
    }

    /// Advances to the next word, answering whether there was one.
    fn step(&mut self) -> bool {
        self.skip_blanks();
        if self.next == self.text.len() {
            return false;
        }
        let start = self.next;
        while self.next < self.text.len() && !is_blank(self.text[self.next]) {
            self.next += 1;
        }
        self.word = start..self.next;
        true
    }

    /// Steps `count` times, answering whether every step found a word.
    fn skip(&mut self, count: usize) -> bool {
        (0..count).all(|_| self.step())
    }

    /// Advances past the blanks between the current word and the next.
    fn skip_blanks(&mut self) {
        while self.next < self.text.len() && is_blank(self.text[self.next]) {
            self.next += 1;
        }
    }
}

/// Every word of `text`, in order.
pub(crate) fn word_slices(text: &[u8]) -> Vec<&[u8]> {
    let mut scan = Words::new(text);
    let mut found = Vec::new();
    while scan.step() {
        found.push(&text[scan.word.clone()]);
    }
    found
}

/// `WORDS`'s body: how many words `text` holds.
pub(crate) fn word_count(text: &[u8]) -> usize {
    let mut scan = Words::new(text);
    let mut count = 0usize;
    while scan.step() {
        count += 1;
    }
    count
}

/// `WORD`, `WORDINDEX` and `WORDLENGTH`'s body: where the `position`th
/// (1-based) word of `text` lies, or `None` past the last.
pub(crate) fn word_range(text: &[u8], position: usize) -> Option<Range<usize>> {
    let mut scan = Words::new(text);
    if scan.skip(position) {
        Some(scan.word.clone())
    } else {
        None
    }
}

/// `SUBWORD`'s body: the slice of `text` from the `position`th (1-based) word
/// to the end of the `count`th from there, empty past the last word or for a
/// `count` of zero; a `count` of `None` reaches the last word.
pub(crate) fn subword_range(text: &[u8], position: usize, count: Option<usize>) -> Range<usize> {
    let count = count.unwrap_or(ALL_REMAINING_WORDS);
    let mut scan = Words::new(text);
    if count == 0 || !scan.skip(position) {
        return 0..0;
    }
    let start = scan.word.start;
    // Whether this reaches `count` words or runs out, `scan.word` is the
    // last word taken either way, which is what ends the slice at a word
    // rather than at the end of the string.
    scan.skip(count - 1);
    start..scan.word.end
}

/// `DELWORD`'s body: removes `count` words of `bytes` from the `position`th
/// (1-based) together with the blanks after the last of them; a `count` of
/// `None` removes every remaining word.
pub(crate) fn delword_bytes(bytes: &mut Vec<u8>, position: usize, count: Option<usize>) {
    let count = count.unwrap_or(ALL_REMAINING_WORDS);
    let mut scan = Words::new(bytes);
    if count == 0 || !scan.skip(position) {
        return;
    }
    let front = scan.word.start;
    // The C++ asks for the blanks only when the skip reached its count
    // (`RexxString::delWord`); unconditionally is the same thing here,
    // because a [`Words::step`] that fails leaves the scan at the end of the
    // string and there is nothing left for the skip to consume.
    scan.skip(count - 1);
    scan.skip_blanks();
    let rest = scan.next;
    bytes.drain(front..rest);
}

/// `WORDPOS`'s body: which word of `string`, counting from the `start`th
/// (1-based), begins a run matching `phrase`'s words, or 0.
pub(crate) fn wordpos_bytes(phrase: &[u8], string: &[u8], start: usize) -> usize {
    wordpos_with(phrase, string, start, <[u8]>::eq)
}

/// [`wordpos_bytes`] with ASCII case folded away,
/// `StringUtil::caselessWordPos` (`classes/support/StringUtil.cpp:1651`).
pub(crate) fn caseless_wordpos_bytes(phrase: &[u8], string: &[u8], start: usize) -> usize {
    wordpos_with(phrase, string, start, crate::builtin::string::caseless_eq)
}

/// The word search both spellings run, `matches` the only difference.
fn wordpos_with(
    phrase: &[u8],
    string: &[u8],
    start: usize,
    matches: impl Fn(&[u8], &[u8]) -> bool,
) -> usize {
    let needle = word_slices(phrase);
    let haystack = word_slices(string);
    // Both guards earn their place, and for different reasons. An empty
    // phrase would otherwise match at every position, since the empty slice
    // equals the empty needle -- the oracle answers 0. A phrase with more
    // words than the string is what the subtraction below cannot survive.
    // A `start` past the last candidate needs no guard of its own: the range
    // is then simply empty.
    if needle.is_empty() || needle.len() > haystack.len() {
        0
    } else {
        (start..=haystack.len() - needle.len() + 1)
            .find(|at| {
                haystack[at - 1..at - 1 + needle.len()]
                    .iter()
                    .zip(&needle)
                    .all(|(found, wanted)| matches(found, wanted))
            })
            .unwrap_or(0)
    }
}

/// `WORDS(string)`: how many blank-delimited words the argument holds.
pub(crate) fn words(interp: &mut Interp, _name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let count = word_count(&string);
    Ok(interp.counted(count))
}

/// The word-position argument at `position`, converted but **not yet
/// range-checked**.
fn converted_position(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
    position: usize,
) -> Result<i64, Failure> {
    Ok(whole_number(interp, name, args, position)?
        .expect("check_arity admitted this required argument"))
}

/// `WORD(string, n)`: the `n`th word, or the null string if there is no such
/// word.
pub(crate) fn word(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    // The position is converted first so that the string can be read through
    // `required_render`, which borrows the bytes where they are instead of
    // copying them out; that function's own doc comment carries the ordering
    // rule and why the reordering is not observable. The word is then built
    // in the lent result buffer, so a word short enough to live in the handle
    // costs no allocation at all.
    let position = position_of(converted_position(interp, name, args, 2)?)?;
    let string = required_render(interp, args, 1);
    let found = word_range(string.text(interp), position).unwrap_or(0..0);
    let mut out = interp.take_result_buffer();
    out.extend_from_slice(&string.text(interp)[found]);
    Ok(interp.text_built(out))
}

/// `WORDINDEX(string, n)`: the 1-based byte offset the `n`th word starts at,
/// or 0.
pub(crate) fn word_index(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let position = position_of(converted_position(interp, name, args, 2)?)?;

    let index = word_range(&string, position).map_or(0, |word| word.start + 1);
    Ok(interp.counted(index))
}

/// `WORDLENGTH(string, n)`: how many bytes the `n`th word is, or 0.
pub(crate) fn word_length(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let position = position_of(converted_position(interp, name, args, 2)?)?;

    let length = word_range(&string, position).map_or(0, |word| word.len());
    Ok(interp.counted(length))
}

/// `SUBWORD(string, n [,length])`: `length` words from the `n`th, as a slice
/// of the argument.
pub(crate) fn subword(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let n = converted_position(interp, name, args, 2)?;
    let requested = whole_number(interp, name, args, 3)?;
    let position = position_of(n)?;
    let count = requested.map(length_of).transpose()?;

    let found = subword_range(&string, position, count);
    Ok(interp.text(&string[found]))
}

/// `DELWORD(string, n [,length])`: the argument with `length` words from the
/// `n`th removed.
pub(crate) fn delword(interp: &mut Interp, name: &[u8], args: Args<'_>) -> Result<ObjRef, Failure> {
    let mut string = required_string(interp, args, 1);
    let n = converted_position(interp, name, args, 2)?;
    let requested = whole_number(interp, name, args, 3)?;
    let position = position_of(n)?;
    let count = requested.map(length_of).transpose()?;

    delword_bytes(&mut string, position, count);
    Ok(interp.text_built(string))
}

/// `WORDPOS(phrase, string [,start])`: which word of `string` begins a run
/// matching `phrase`'s words, or 0.
pub(crate) fn word_pos(
    interp: &mut Interp,
    name: &[u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let phrase = required_string(interp, args, 1);
    let string = required_string(interp, args, 2);
    let start = match whole_number(interp, name, args, 3)? {
        Some(value) => position_of(value)?,
        None => 1,
    };

    let found = wordpos_bytes(&phrase, &string, start);
    Ok(interp.counted(found))
}

#[cfg(test)]
mod tests {
    use super::super::dispatch;
    use crate::error::Failure;
    use crate::{Interp, error::Raised};

    /// Runs `name` over `arguments`, each `None` standing for an omitted
    /// interior position, and answers the result's own bytes.
    fn call(name: &[u8], arguments: &[Option<&[u8]>]) -> Result<Vec<u8>, Failure> {
        let mut interp = Interp::new();
        let args: Vec<_> = arguments
            .iter()
            .map(|argument| argument.map(|bytes| interp.text(bytes)))
            .collect();
        let result = dispatch(&mut interp, name, &args).expect("a builtin name")?;
        Ok(interp.to_text(result).into_owned())
    }

    /// [`call`], for the cases whose answer is the bytes and nothing else.
    fn answer(name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
        call(name, &arguments).expect("this call succeeds")
    }

    /// The `(major, sub)` and substitutions of the condition `name` raises.
    fn raised(name: &[u8], arguments: &[Option<&[u8]>]) -> (u16, u16, Vec<Vec<u8>>) {
        let failure = call(name, arguments).expect_err("this call raises");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        (raised.number, raised.sub, raised.additional)
    }

    /// Blank and horizontal tab separate words; no other byte does.
    #[test]
    fn only_blank_and_tab_separate_words() {
        for byte in 0..=u8::MAX {
            let subject = [b'a', byte, b'b'];
            let separates = byte == b' ' || byte == b'\t';
            assert_eq!(
                answer(b"WORDS", &[&subject]),
                if separates { b"2" } else { b"1" },
                "byte {byte:#04x} is on the wrong side of the word boundary"
            );
        }
        // The three that make the rule narrower than any of its neighbours,
        // spelled out so a reader sees them without running the loop.
        assert_eq!(answer(b"WORDS", &[b"a\nb"]), b"1");
        assert_eq!(answer(b"WORDS", &[b"a\rb"]), b"1");
        assert_eq!(answer(b"WORDS", &[&[b'a', 0xa0, b'b']]), b"1");
    }

    /// Leading, trailing and repeated separators, and the strings with no
    /// words at all.
    #[test]
    fn separators_outside_the_words_do_not_make_words() {
        assert_eq!(answer(b"WORDS", &[b""]), b"0");
        assert_eq!(answer(b"WORDS", &[b"   "]), b"0");
        assert_eq!(answer(b"WORDS", &[b"\t\t\t\t"]), b"0");
        assert_eq!(answer(b"WORDS", &[b"  a b  "]), b"2");
        assert_eq!(answer(b"WORDS", &[b"a    b"]), b"2");
        assert_eq!(answer(b"WORDS", &[b"a\t \tb"]), b"2");
        assert_eq!(answer(b"WORDS", &[b"a b c d"]), b"4");
    }

    /// The positional trio at every boundary a position can sit on: the
    /// first word, an interior one, the last, one past the last, and far
    /// past it.
    #[test]
    fn the_positional_builtins_answer_the_oracles_own_bytes() {
        assert_eq!(answer(b"WORD", &[b"aa bb  cc", b"1"]), b"aa");
        assert_eq!(answer(b"WORD", &[b"aa bb  cc", b"2"]), b"bb");
        assert_eq!(answer(b"WORD", &[b"aa bb  cc", b"3"]), b"cc");
        assert_eq!(answer(b"WORD", &[b"aa bb  cc", b"4"]), b"");
        assert_eq!(answer(b"WORD", &[b"aa bb  cc", b"999999"]), b"");
        assert_eq!(answer(b"WORD", &[b"", b"1"]), b"");
        assert_eq!(answer(b"WORD", &[b"   ", b"1"]), b"");

        assert_eq!(answer(b"WORDINDEX", &[b"aa bb  cc", b"1"]), b"1");
        assert_eq!(answer(b"WORDINDEX", &[b"aa bb  cc", b"2"]), b"4");
        assert_eq!(answer(b"WORDINDEX", &[b"aa bb  cc", b"3"]), b"8");
        assert_eq!(answer(b"WORDINDEX", &[b"aa bb  cc", b"4"]), b"0");
        assert_eq!(answer(b"WORDINDEX", &[b"aa bb  cc", b"999999"]), b"0");
        assert_eq!(answer(b"WORDINDEX", &[b"", b"1"]), b"0");
        assert_eq!(answer(b"WORDINDEX", &[b"   ", b"1"]), b"0");

        assert_eq!(answer(b"WORDLENGTH", &[b"aa bb  cc", b"1"]), b"2");
        assert_eq!(answer(b"WORDLENGTH", &[b"aa bb  cc", b"3"]), b"2");
        assert_eq!(answer(b"WORDLENGTH", &[b"aa bb  cc", b"4"]), b"0");
        assert_eq!(answer(b"WORDLENGTH", &[b"", b"1"]), b"0");
        assert_eq!(answer(b"WORDLENGTH", &[b"   ", b"1"]), b"0");

        // Leading separators shift every offset, which is the half a string
        // starting at its first word cannot show.
        assert_eq!(answer(b"WORD", &[b"  aa\tbb  ", b"1"]), b"aa");
        assert_eq!(answer(b"WORD", &[b"  aa\tbb  ", b"2"]), b"bb");
        assert_eq!(answer(b"WORD", &[b"  aa\tbb  ", b"3"]), b"");
        assert_eq!(answer(b"WORDINDEX", &[b"  aa\tbb  ", b"1"]), b"3");
        assert_eq!(answer(b"WORDINDEX", &[b"  aa\tbb  ", b"2"]), b"6");
        assert_eq!(answer(b"WORDINDEX", &[b"  aa\tbb  ", b"3"]), b"0");
    }

    /// `SUBWORD` keeps the separators between the words it takes and drops
    /// the ones outside them.
    #[test]
    fn subword_answers_a_slice_of_its_argument() {
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"1"]), b"aa bb  cc");
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"2"]), b"bb  cc");
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"3"]), b"cc");
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"4"]), b"");
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"999"]), b"");
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"2", b"1"]), b"bb");
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"1", b"2"]), b"aa bb");
        assert_eq!(
            answer(b"SUBWORD", &[b"aa bb  cc", b"1", b"99"]),
            b"aa bb  cc"
        );
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"3", b"99"]), b"cc");
        // A count of zero, which a scan skipping `count - 1` words would
        // wrap past and answer the whole remainder for.
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"2", b"0"]), b"");
        assert_eq!(answer(b"SUBWORD", &[b"aa bb  cc", b"1", b"0"]), b"");

        assert_eq!(answer(b"SUBWORD", &[b"aa bb  ", b"1"]), b"aa bb");
        assert_eq!(answer(b"SUBWORD", &[b"  aa bb", b"1"]), b"aa bb");
        assert_eq!(
            answer(b"SUBWORD", &[b"aa\t\tbb cc", b"1", b"2"]),
            b"aa\t\tbb"
        );
        assert_eq!(answer(b"SUBWORD", &[b"", b"1"]), b"");
        assert_eq!(answer(b"SUBWORD", &[b"   ", b"1"]), b"");
        assert_eq!(
            call(b"SUBWORD", &[Some(b"aa bb  cc"), Some(b"2"), None])
                .expect("an omitted length is legal past the minimum"),
            b"bb  cc"
        );
    }

    /// `DELWORD` takes the separators after the last deleted word and leaves
    /// the ones before the first.
    #[test]
    fn delword_answers_the_oracles_own_bytes() {
        assert_eq!(answer(b"DELWORD", &[b"aa bb  cc", b"1"]), b"");
        assert_eq!(answer(b"DELWORD", &[b"aa bb  cc", b"2"]), b"aa ");
        assert_eq!(answer(b"DELWORD", &[b"aa bb  cc", b"3"]), b"aa bb  ");
        assert_eq!(answer(b"DELWORD", &[b"aa bb  cc", b"4"]), b"aa bb  cc");
        assert_eq!(answer(b"DELWORD", &[b"aa bb  cc", b"2", b"1"]), b"aa cc");
        assert_eq!(answer(b"DELWORD", &[b"aa bb  cc", b"1", b"1"]), b"bb  cc");
        assert_eq!(answer(b"DELWORD", &[b"aa bb  cc", b"1", b"2"]), b"cc");
        assert_eq!(answer(b"DELWORD", &[b"aa bb  cc", b"2", b"99"]), b"aa ");
        // A count of zero is the argument unchanged, which the same wrapping
        // mistake would turn into a deletion to the end.
        assert_eq!(
            answer(b"DELWORD", &[b"aa bb  cc", b"2", b"0"]),
            b"aa bb  cc"
        );

        // The surviving separator is the argument's own byte, not a blank
        // the result was rebuilt with. A tab before the deleted word stays a
        // tab; a tab *after* it goes with the word. Asserted in the oracle's
        // own suite at `ootest/ooRexx/base/source.file/whiteSpace.testGroup`
        // rather than in `DELWORD.testGroup`.
        assert_eq!(
            answer(b"DELWORD", &[b"hey\tis-this  you", b"2", b"1"]),
            b"hey\tyou"
        );
        assert_eq!(
            answer(b"DELWORD", &[b"hey  is-this\tyou", b"2", b"1"]),
            b"hey  you"
        );

        assert_eq!(answer(b"DELWORD", &[b"  aa bb", b"1", b"1"]), b"  bb");
        assert_eq!(answer(b"DELWORD", &[b"aa bb   ", b"2"]), b"aa ");
        assert_eq!(answer(b"DELWORD", &[b"aa bb   ", b"2", b"1"]), b"aa ");
        assert_eq!(answer(b"DELWORD", &[b"aa\tbb cc", b"2", b"1"]), b"aa\tcc");
        assert_eq!(answer(b"DELWORD", &[b"", b"1"]), b"");
        assert_eq!(answer(b"DELWORD", &[b"", b"999"]), b"");
        assert_eq!(answer(b"DELWORD", &[b"   ", b"1"]), b"   ");
        assert_eq!(
            call(b"DELWORD", &[Some(b"aa bb  cc"), Some(b"2"), None])
                .expect("an omitted length is legal past the minimum"),
            b"aa "
        );
    }

    /// `WORDPOS` matches word by word, so the separators on either side are
    /// not part of the comparison.
    #[test]
    fn wordpos_matches_words_and_not_their_separators() {
        let hay: &[u8] = b"now is the time for all good men";
        assert_eq!(answer(b"WORDPOS", &[b"the", hay]), b"3");
        assert_eq!(answer(b"WORDPOS", &[b"the time", hay]), b"3");
        assert_eq!(answer(b"WORDPOS", &[b"the   time", hay]), b"3");
        assert_eq!(answer(b"WORDPOS", &[b"the\ttime", hay]), b"3");
        assert_eq!(answer(b"WORDPOS", &[b"  the  ", hay]), b"3");
        assert_eq!(answer(b"WORDPOS", &[b"good men", hay]), b"7");
        assert_eq!(answer(b"WORDPOS", &[hay, hay]), b"1");
        // The refusals: a wrong case, a prefix of a word, a phrase running
        // past the end, and a phrase with no words at all.
        assert_eq!(answer(b"WORDPOS", &[b"The", hay]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"th", hay]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"good men xx", hay]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"", hay]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"   ", hay]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"a", b""]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"", b""]), b"0");
        // A phrase with more words than the string has: the shape the
        // candidate-count subtraction cannot be asked to compute.
        assert_eq!(answer(b"WORDPOS", &[b"a b c", b"x"]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"a b c", b""]), b"0");
        assert_eq!(answer(b"WORDPOS", &[hay, b"now is"]), b"0");

        // The start position, at each boundary it has.
        assert_eq!(answer(b"WORDPOS", &[b"the", hay, b"3"]), b"3");
        assert_eq!(answer(b"WORDPOS", &[b"the", hay, b"4"]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"the", hay, b"99"]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"men", hay, b"8"]), b"8");
        assert_eq!(answer(b"WORDPOS", &[b"men", hay, b"9"]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"good men", hay, b"7"]), b"7");
        assert_eq!(answer(b"WORDPOS", &[b"good men", hay, b"8"]), b"0");
        assert_eq!(answer(b"WORDPOS", &[b"aa", b"aa bb aa", b"1"]), b"1");
        assert_eq!(answer(b"WORDPOS", &[b"aa", b"aa bb aa", b"2"]), b"3");
        assert_eq!(
            call(b"WORDPOS", &[Some(b"the"), Some(hay), None]).expect("the start defaults to 1"),
            b"3"
        );
    }

    /// Bytes at or above `0x80`, control bytes and the null string, crossed
    /// with each position a word can be asked about.
    #[test]
    fn a_byte_string_alphabet_crosses_every_position() {
        let subject: &[u8] = b"a\xe9 \x00b\tc\nd";
        assert_eq!(answer(b"WORDS", &[subject]), b"3");
        assert_eq!(answer(b"WORD", &[subject, b"1"]), b"a\xe9");
        assert_eq!(answer(b"WORD", &[subject, b"2"]), b"\x00b");
        assert_eq!(answer(b"WORD", &[subject, b"3"]), b"c\nd");
        assert_eq!(answer(b"WORD", &[subject, b"4"]), b"");
        assert_eq!(answer(b"WORDINDEX", &[subject, b"2"]), b"4");
        assert_eq!(answer(b"WORDINDEX", &[subject, b"3"]), b"7");
        assert_eq!(answer(b"WORDLENGTH", &[subject, b"1"]), b"2");
        assert_eq!(answer(b"WORDLENGTH", &[subject, b"3"]), b"3");
        assert_eq!(answer(b"SUBWORD", &[subject, b"2"]), b"\x00b\tc\nd");
        assert_eq!(answer(b"SUBWORD", &[subject, b"2", b"1"]), b"\x00b");
        assert_eq!(answer(b"DELWORD", &[subject, b"2", b"1"]), b"a\xe9 c\nd");
        assert_eq!(answer(b"WORDPOS", &[b"\x00b", subject]), b"2");
        assert_eq!(answer(b"WORDPOS", &[b"c\nd", subject]), b"3");
        // The oracle's own committed case for a NUL inside a word:
        // `DELWORD.testGroup` test19 asserts that
        // `asdf zxcv<NUL x5>ASDF ZXCV` is three words whose second is 13
        // bytes long, and that deleting from word 3 keeps the blank in front
        // of it. Measured here as well as read: `words` is 3, `wordlength`
        // at 2 is 13.
        let nuls: &[u8] = b"asdf zxcv\x00\x00\x00\x00\x00ASDF ZXCV";
        assert_eq!(answer(b"WORDS", &[nuls]), b"3");
        assert_eq!(answer(b"WORDLENGTH", &[nuls, b"2"]), b"13");
        assert_eq!(
            answer(b"WORD", &[nuls, b"2"]),
            b"zxcv\x00\x00\x00\x00\x00ASDF"
        );
        assert_eq!(
            answer(b"DELWORD", &[nuls, b"3", b"2"]),
            b"asdf zxcv\x00\x00\x00\x00\x00ASDF "
        );

        // The null string as the subject, at every one of the seven.
        assert_eq!(answer(b"WORDS", &[b""]), b"0");
        assert_eq!(answer(b"WORD", &[b"", b"1"]), b"");
        assert_eq!(answer(b"WORDINDEX", &[b"", b"1"]), b"0");
        assert_eq!(answer(b"WORDLENGTH", &[b"", b"1"]), b"0");
        assert_eq!(answer(b"SUBWORD", &[b"", b"1"]), b"");
        assert_eq!(answer(b"DELWORD", &[b"", b"1"]), b"");
        assert_eq!(answer(b"WORDPOS", &[b"", b""]), b"0");
    }

    /// A position of zero or below is 93.924, and it is the same answer for
    /// every builtin that takes one.
    #[test]
    fn a_non_positive_position_is_the_operation_layers_own_error() {
        for name in [
            b"WORD".as_slice(),
            b"WORDINDEX",
            b"WORDLENGTH",
            b"SUBWORD",
            b"DELWORD",
        ] {
            assert_eq!(
                raised(name, &[Some(b"a b"), Some(b"0")]),
                (93, 924, vec![b"0".to_vec()]),
                "{} did not refuse position 0",
                String::from_utf8_lossy(name)
            );
            assert_eq!(
                raised(name, &[Some(b"a b"), Some(b"-1")]),
                (93, 924, vec![b"-1".to_vec()]),
                "{} did not refuse a negative position",
                String::from_utf8_lossy(name)
            );
            call(name, &[Some(b"a b"), Some(b"1")]).expect("position 1 is legal");
        }
        // `WORDPOS`'s start is the third argument and optional, and the same
        // refusal reaches it.
        assert_eq!(
            raised(b"WORDPOS", &[Some(b"a"), Some(b"a b"), Some(b"0")]),
            (93, 924, vec![b"0".to_vec()])
        );
        assert_eq!(
            raised(b"WORDPOS", &[Some(b"a"), Some(b"a b"), Some(b"-1")]),
            (93, 924, vec![b"-1".to_vec()])
        );
        // A position argument the string is far too short for is not an
        // error at all, which is what separates 93.924 from "out of range".
        assert_eq!(answer(b"WORD", &[b"a b", b"999999999999999999"]), b"");
        assert_eq!(
            answer(b"WORDPOS", &[b"a", b"a b", b"999999999999999999"]),
            b"0"
        );
    }

    /// A negative count is 93.923, the *length* member of the operation-layer
    /// family rather than the position one.
    #[test]
    fn a_negative_count_is_an_invalid_length() {
        assert_eq!(
            raised(b"SUBWORD", &[Some(b"a b"), Some(b"1"), Some(b"-1")]),
            (93, 923, vec![b"-1".to_vec()])
        );
        assert_eq!(
            raised(b"DELWORD", &[Some(b"a b"), Some(b"1"), Some(b"-1")]),
            (93, 923, vec![b"-1".to_vec()])
        );
        // Zero is the adjacent legal value, and it is not the same answer as
        // omitting the argument.
        assert_eq!(answer(b"SUBWORD", &[b"a b", b"1", b"0"]), b"");
        assert_eq!(answer(b"SUBWORD", &[b"a b", b"1"]), b"a b");
        assert_eq!(answer(b"DELWORD", &[b"a b", b"1", b"0"]), b"a b");
        assert_eq!(answer(b"DELWORD", &[b"a b", b"1"]), b"");
    }

    /// An omitted count reaches every remaining word, however many there
    /// are.
    #[test]
    fn an_omitted_count_reaches_every_remaining_word() {
        let ten: &[u8] = b"a b c d e f g h i j";
        assert_eq!(answer(b"SUBWORD", &[ten, b"1"]), ten);
        assert_eq!(answer(b"SUBWORD", &[ten, b"2"]), b"b c d e f g h i j");
        assert_eq!(answer(b"SUBWORD", &[ten, b"9"]), b"i j");
        assert_eq!(answer(b"DELWORD", &[ten, b"1"]), b"");
        assert_eq!(answer(b"DELWORD", &[ten, b"2"]), b"a ");
        assert_eq!(answer(b"DELWORD", &[ten, b"9"]), b"a b c d e f g h ");
        // And the explicit spelling of that default, which is the oracle's
        // own `MAX_WHOLENUMBER` and the largest the conversion admits.
        assert_eq!(answer(b"SUBWORD", &[ten, b"1", b"999999999999999999"]), ten);
        assert_eq!(answer(b"DELWORD", &[ten, b"1", b"999999999999999999"]), b"");
    }

    /// The zero-count shortcut does not run before the position check, and
    /// the position's range check comes before the count's.
    #[test]
    fn the_position_is_range_checked_before_the_count_is_honoured() {
        assert_eq!(
            raised(b"SUBWORD", &[Some(b"SUBWORD"), Some(b"0"), Some(b"0")]),
            (93, 924, vec![b"0".to_vec()])
        );
        assert_eq!(
            raised(b"DELWORD", &[Some(b"delWord"), Some(b"0"), Some(b"0")]),
            (93, 924, vec![b"0".to_vec()])
        );
        assert_eq!(
            raised(b"SUBWORD", &[Some(b"a b"), Some(b"0"), Some(b"-1")]),
            (93, 924, vec![b"0".to_vec()])
        );
        assert_eq!(
            raised(b"DELWORD", &[Some(b"a b"), Some(b"0"), Some(b"-1")]),
            (93, 924, vec![b"0".to_vec()])
        );
        // The adjacent successes: with the position made legal, the zero
        // count is honoured rather than raising anything.
        assert_eq!(answer(b"SUBWORD", &[b"SUBWORD", b"1", b"0"]), b"");
        assert_eq!(answer(b"DELWORD", &[b"delWord", b"1", b"0"]), b"delWord");
    }

    /// A bad argument *kind* is 40.12, naming the routine and the call's own
    /// argument position -- including `WORDPOS`, whose numeric argument is
    /// third because its phrase comes first.
    #[test]
    fn a_bad_argument_kind_names_the_routine_and_the_call_position() {
        for name in [
            b"WORD".as_slice(),
            b"WORDINDEX",
            b"WORDLENGTH",
            b"SUBWORD",
            b"DELWORD",
        ] {
            assert_eq!(
                raised(name, &[Some(b"a b"), Some(b"x")]),
                (40, 12, vec![name.to_vec(), b"2".to_vec(), b"x".to_vec()]),
                "{} named the wrong argument",
                String::from_utf8_lossy(name)
            );
        }
        assert_eq!(
            raised(b"WORDPOS", &[Some(b"a"), Some(b"a b"), Some(b"q")]),
            (
                40,
                12,
                vec![b"WORDPOS".to_vec(), b"3".to_vec(), b"q".to_vec()]
            )
        );
        assert_eq!(
            raised(b"SUBWORD", &[Some(b"a b"), Some(b"1"), Some(b"q")]),
            (
                40,
                12,
                vec![b"SUBWORD".to_vec(), b"3".to_vec(), b"q".to_vec()]
            )
        );
        assert_eq!(
            raised(b"DELWORD", &[Some(b"a b"), Some(b"1"), Some(b"q")]),
            (
                40,
                12,
                vec![b"DELWORD".to_vec(), b"3".to_vec(), b"q".to_vec()]
            )
        );
        // The null string is not a whole number, and a value needing more
        // than the argument precision is not one either.
        assert_eq!(
            raised(b"WORD", &[Some(b"a b"), Some(b"")]),
            (40, 12, vec![b"WORD".to_vec(), b"2".to_vec(), Vec::new()])
        );
        assert_eq!(
            raised(
                b"SUBWORD",
                &[Some(b"a b"), Some(b"1"), Some(b"1000000000000000000")]
            ),
            (
                40,
                12,
                vec![
                    b"SUBWORD".to_vec(),
                    b"3".to_vec(),
                    b"1000000000000000000".to_vec()
                ]
            )
        );
        // The generous spellings the conversion accepts, so the refusals
        // above are pinned to the value and not to its formatting.
        assert_eq!(answer(b"WORD", &[b"a b c", b" 2 "]), b"b");
        assert_eq!(answer(b"WORD", &[b"a b c", b"+2"]), b"b");
        assert_eq!(answer(b"WORD", &[b"a b c", b"007"]), b"");
        assert_eq!(answer(b"WORD", &[b"a b c", b"1e1"]), b"");
    }

    /// A substitution carries the argument's own bytes, and the report
    /// applies the oracle's display rule to them.
    #[test]
    fn a_substitution_carries_bytes_and_the_report_makes_them_displayable() {
        assert_eq!(
            raised(b"WORD", &[Some(b"a b"), Some(&[0x01, b'q'])]),
            (
                40,
                12,
                vec![b"WORD".to_vec(), b"2".to_vec(), vec![0x01, b'q']]
            )
        );
        assert_eq!(
            raised(b"WORD", &[Some(b"a b"), Some(&[0xe9])]),
            (40, 12, vec![b"WORD".to_vec(), b"2".to_vec(), vec![0xe9]])
        );

        let site = crate::error::ClauseSite {
            sites: &[],
            path: "/p.rex",
        };
        let control = Raised::argument_not_whole(b"WORD", 2, &[0x01, b'q']);
        assert!(
            control.report(&site).windows(4).any(|w| w == *b"\"?q\""),
            "a control byte must reach the report as a question mark"
        );
        let high = Raised::argument_not_whole(b"WORD", 2, &[0xe9]);
        assert!(
            high.report(&site)
                .windows(3)
                .any(|w| w == [b'"', 0xe9, b'"']),
            "a byte at or above 0x80 must reach the report unchanged"
        );
    }

    /// The call layer runs before the operation layer here too, at the one
    /// shape in this family that can tell them apart.
    #[test]
    fn the_call_layer_is_checked_before_the_operation_layer() {
        assert_eq!(
            raised(b"SUBWORD", &[Some(b"a b"), Some(b"0"), Some(b"q")]),
            (
                40,
                12,
                vec![b"SUBWORD".to_vec(), b"3".to_vec(), b"q".to_vec()]
            )
        );
        assert_eq!(
            raised(b"SUBWORD", &[Some(b"a b"), Some(b"0"), Some(b"1")]),
            (93, 924, vec![b"0".to_vec()])
        );
        assert_eq!(
            raised(b"DELWORD", &[Some(b"a b"), Some(b"0"), Some(b"q")]),
            (
                40,
                12,
                vec![b"DELWORD".to_vec(), b"3".to_vec(), b"q".to_vec()]
            )
        );
        assert_eq!(
            raised(b"DELWORD", &[Some(b"a b"), Some(b"0"), Some(b"1")]),
            (93, 924, vec![b"0".to_vec()])
        );
    }

    /// The arity rows, at both ends and at the interior omission each one
    /// admits.
    #[test]
    fn the_arity_rows_are_the_oracles_own() {
        for (name, min, max) in [
            (b"DELWORD".as_slice(), 2usize, 3usize),
            (b"SUBWORD", 2, 3),
            (b"WORD", 2, 2),
            (b"WORDINDEX", 2, 2),
            (b"WORDLENGTH", 2, 2),
            (b"WORDPOS", 2, 3),
            (b"WORDS", 1, 1),
        ] {
            let short: Vec<Option<&[u8]>> = vec![Some(b"a b"); min - 1];
            assert_eq!(
                raised(name, &short),
                (40, 3, vec![name.to_vec(), min.to_string().into_bytes()]),
                "{} did not name its minimum",
                String::from_utf8_lossy(name)
            );
            let long: Vec<Option<&[u8]>> = vec![Some(b"1"); max + 1];
            assert_eq!(
                raised(name, &long),
                (40, 4, vec![name.to_vec(), max.to_string().into_bytes()]),
                "{} did not name its maximum",
                String::from_utf8_lossy(name)
            );
        }
        // An omission interior to the required positions is 40.5, and every
        // shape in this family that can reach it does.
        assert_eq!(
            raised(b"WORD", &[None, Some(b"1")]),
            (40, 5, vec![b"WORD".to_vec(), b"1".to_vec()])
        );
        for name in [b"DELWORD".as_slice(), b"SUBWORD", b"WORDPOS"] {
            assert_eq!(
                raised(name, &[Some(b"a b"), None, Some(b"1")]),
                (40, 5, vec![name.to_vec(), b"2".to_vec()]),
                "{} did not name its missing second argument",
                String::from_utf8_lossy(name)
            );
        }
    }

    /// A count or an offset is created as text, so `NUMERIC DIGITS` cannot
    /// reach it.
    #[test]
    fn a_counting_builtin_answers_text_that_no_digits_setting_reshapes() {
        assert_eq!(answer(b"WORDS", &[b"a b c d e f g h i j"]), b"10");
        assert_eq!(
            answer(b"WORDINDEX", &[b"a b c d e f g h i j", b"10"]),
            b"19"
        );
        assert_eq!(answer(b"WORDLENGTH", &[b"aaaaaaaaaa bb", b"1"]), b"10");
        assert_eq!(answer(b"WORDPOS", &[b"j", b"a b c d e f g h i j"]), b"10");
    }
}
