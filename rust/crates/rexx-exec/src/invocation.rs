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

//! What a command line supplies to a top-level program, as
//! [`run_program`](crate::run_program)'s third parameter.
//!
//! # A command line can supply at most ONE argument string
//!
//! Not one per word. `rexx foo.rex a b c` hands the program a single string
//! `a b c`, and `rexx foo.rex "a b c"` hands it the identical string, so the
//! two invocations are indistinguishable to the program. Measured, with
//! `arg()` and `arg(1)` printed together:
//!
//! ```text
//! rexx p.rex            ->  arg() = 0, arg(1) omitted
//! rexx p.rex a b c      ->  arg() = 1, arg(1) = "a b c"
//! rexx p.rex "a b c"    ->  arg() = 1, arg(1) = "a b c"
//! rexx p.rex a,b,c      ->  arg() = 1, arg(1) = "a,b,c"   (commas do not split)
//! rexx p.rex ""         ->  arg() = 1, arg(1) = ""        (empty, but PRESENT)
//! ```
//!
//! `arg(2)` is the null string for every one of those, so a single optional
//! string is the whole model. More than one argument is reachable only by
//! invoking a program as an external routine, which Phase 4 does not do.
//!
//! # Absent and empty are different states
//!
//! The last two rows above are the reason [`Invocation`]'s argument is an
//! `Option` and not a `Vec<u8>` that happens to be empty. `rexx p.rex ""`
//! supplies an argument whose value is the null string; `rexx p.rex` supplies
//! no argument at all. The oracle's own launcher makes exactly that
//! distinction and makes it separately from the joined text
//! (`utilities/rexx/platform/unix/rexx.cpp`): `argCount` counts the non-option
//! command-line words and is then clamped to `(argCount == 0) ? 0 : 1`, while
//! the string is built independently in `arg_buffer`. So an empty
//! `arg_buffer` with `argCount == 1` is representable there and has to be
//! representable here.
//!
//! **It is observable from Phase 4c's own instructions, three ways**, so
//! modelling absence as `Some(Vec::new())` is not a latent defect but a
//! visible wrong answer. Measured, oracle, `use arg p` / `say '<'p'>'`:
//!
//! ```text
//! rexx p.rex     ->  <P>   -- nothing bound, so `p` reads as its own name
//! rexx p.rex ""  ->  <>    -- bound to the null string
//! ```
//!
//! and with `use strict arg p` the absent case is `Error 40.3` (rc 216) where
//! the empty case is rc 0, and with `use strict arg` (no targets at all) the
//! absent case is rc 0 where the empty case is `Error 40.4` (rc 216).
//!
//! # How the words are joined
//!
//! The separating blank goes in **only when what has been accumulated so far
//! is not empty**, which is not the same as "join the words with a blank".
//! `rexx.cpp`'s loop is `if (arg_buffer[0] != '\0') strcat(arg_buffer, " ");`
//! then `strcat(arg_buffer, argv[i])`, so a leading empty word contributes
//! nothing at all -- neither text nor separator -- while a trailing one still
//! contributes its separator. Measured, `length(arg(1))` and `c2x(arg(1))`
//! for each:
//!
//! ```text
//! ""      ""        ->  ""      (0)   -- not " " (1)
//! ""      "x"       ->  "x"     (1)   -- not " x" (2)
//! "x"     ""        ->  "x "    (2)
//! "" "" "x"         ->  "x"     (1)
//! "x" "" "y"        ->  "x  y"  (4)
//! " " "x"           ->  "  x"   (3)   -- a blank word IS non-empty
//! " x"              ->  " x"    (2)   -- nothing is stripped
//! "a  b"  "c"       ->  "a  b c" (6)  -- internal spacing survives
//! ```
//!
//! The last three rows were predicted from the C++ above and then run, rather
//! than being the rows that suggested the rule.
//!
//! [`join_command_line`] is that rule. It has no length ceiling, where
//! `rexx.cpp` `strcat`s into a fixed `char arg_buffer[8192]`; overrunning
//! that is undefined behaviour in the oracle rather than an answer to
//! reproduce.
//!
//! # The other half of an invocation
//!
//! [`ProgramInput`] says where `.input` reads its lines from, and that arm's
//! own doc has why the descriptor is never the default. What reading actually
//! does with it -- the line rule, the `\r\n` collapse, and the position all
//! the input constructs share -- is `input.rs`.

/// What a command line supplied to the program being run.
///
/// Constructed by the caller that read the command line -- `bin/rexx-run.rs`
/// in production, a test that wants to supply an argument otherwise -- and
/// handed to [`run_program`](crate::run_program) as one value rather than as
/// a widening list of parameters.
pub struct Invocation {
    /// The one argument string, or `None` when the command line supplied no
    /// argument. See the module doc for why the two are different states and
    /// how each is observable.
    argument: Option<Vec<u8>>,
    /// Where `.input` reads its lines from.
    input: ProgramInput,
}

/// Where `.input` -- the position `PULL`, `PARSE PULL` and `PARSE LINEIN` all
/// advance -- reads its lines from.
///
/// **[`ProgramInput::Nothing`] is the default, and that asymmetry is what
/// makes it impossible for a test to block.** `run_program`'s in-process
/// callers include every differential and assertion harness in this crate, all
/// of which run inside a `cargo test` process whose own standard input is a
/// terminal on a developer's machine and a pipe on a build agent. A reader
/// that reached the real descriptor from there would hang until someone typed
/// a line, or silently eat bytes belonging to the harness, and neither failure
/// looks like a bug in the construct under test. So the descriptor is not the
/// default and **not reachable by omission**: reaching it requires writing
/// [`ProgramInput::Stdin`], which belongs to a caller that is a process of its
/// own with a stdin of its own.
pub enum ProgramInput {
    /// Nothing to read: every line read answers the null string.
    ///
    /// Byte for byte what the oracle answers with its stdin at `/dev/null`,
    /// which is what the differential harnesses give it -- measured, an empty
    /// queue and `/dev/null` give `PARSE PULL` and `PARSE LINEIN` the null
    /// string, rc 0, no condition and no hang, repeatably.
    Nothing,
    /// The process's own standard input, read one line at a time.
    Stdin,
    /// A fixed buffer of bytes, read one line at a time -- what a test uses to
    /// supply input deterministically.
    Bytes(Vec<u8>),
}

impl Invocation {
    /// No argument at all and nothing to read: the state `rexx p.rex` puts a
    /// program in when its stdin is at `/dev/null`.
    ///
    /// Spelled as its own constructor rather than reached through `Default`
    /// because the absence is the interesting half. A caller writing
    /// `Invocation::none()` has said which of the two absent-looking argument
    /// states it means; a caller writing `Default::default()` has not.
    pub fn none() -> Invocation {
        Invocation {
            argument: None,
            input: ProgramInput::Nothing,
        }
    }

    /// One argument string, exactly as supplied -- including the null string,
    /// which is a present argument and not an absent one.
    pub fn with_argument(argument: Vec<u8>) -> Invocation {
        Invocation {
            argument: Some(argument),
            ..Invocation::none()
        }
    }

    /// The same invocation, reading `.input` from `input`.
    pub fn with_input(self, input: ProgramInput) -> Invocation {
        Invocation { input, ..self }
    }

    /// The argument string, if there is one, and where `.input` reads from.
    ///
    /// One accessor consuming the whole value rather than two borrowing
    /// getters: `execute` needs both halves and takes ownership of each, and a
    /// pair of getters would either clone the argument bytes or hand out a
    /// borrow that outlives nothing useful.
    pub(crate) fn into_parts(self) -> (Option<Vec<u8>>, ProgramInput) {
        (self.argument, self.input)
    }
}

/// The one argument string a list of command-line words becomes, or `None`
/// when the list is empty.
///
/// The joining rule and its measurements are in the module doc. `None` for an
/// empty `words` is the `argCount == 0` half of that record: a program run
/// with no words after its name has no argument, which is a different state
/// from having one whose text happens to be empty.
pub fn join_command_line<I, W>(words: I) -> Invocation
where
    I: IntoIterator<Item = W>,
    W: AsRef<[u8]>,
{
    let mut joined: Option<Vec<u8>> = None;
    for word in words {
        let buffer = joined.get_or_insert_with(Vec::new);
        // The blank goes in only when something is already accumulated, which
        // is what makes a leading empty word contribute neither text nor
        // separator. `buffer` having been created by `get_or_insert_with`
        // above is what keeps the *presence* of an argument independent of
        // whether any word had bytes in it.
        if !buffer.is_empty() {
            buffer.push(b' ');
        }
        buffer.extend_from_slice(word.as_ref());
    }
    Invocation {
        argument: joined,
        ..Invocation::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The module doc's own measured table, row for row.
    ///
    /// Every row is a joined byte string measured off the oracle with
    /// `length(arg(1))` and `c2x(arg(1))`, so this is a comparison against
    /// the interpreter rather than against a second copy of this function's
    /// own reasoning. The rows that matter most are the ones where "join with
    /// a blank" gives a different answer -- `["", ""]`, `["", "x"]`,
    /// `["", "", "x"]` -- because that is the implementation this is here to
    /// exclude.
    #[test]
    fn command_line_words_join_the_way_the_oracle_joins_them() {
        let cases: &[(&[&str], Option<&[u8]>)] = &[
            (&[], None),
            (&[""], Some(b"")),
            (&["", ""], Some(b"")),
            (&["", "x"], Some(b"x")),
            (&["x", ""], Some(b"x ")),
            (&["", "", "x"], Some(b"x")),
            (&["x", "", "y"], Some(b"x  y")),
            (&[" ", "x"], Some(b"  x")),
            (&[" x"], Some(b" x")),
            (&["x "], Some(b"x ")),
            (&["a", "b", "c"], Some(b"a b c")),
            (&["a b c"], Some(b"a b c")),
            (&["a  b", "c"], Some(b"a  b c")),
            (&["a,b,c"], Some(b"a,b,c")),
            (&["x ", "y"], Some(b"x  y")),
        ];
        for (words, expected) in cases {
            let joined = join_command_line(*words).into_parts().0;
            assert_eq!(
                joined.as_deref(),
                *expected,
                "joining {words:?} gave {:?}",
                joined.as_ref().map(|b| String::from_utf8_lossy(b))
            );
        }
    }

    /// The absent/empty split, on its own, because it is the one this type
    /// exists for and the table above could satisfy it by accident.
    ///
    /// A `join_command_line` that returned `Some(Vec::new())` for no words
    /// passes every *text* comparison in the table above -- the joined bytes
    /// really are empty either way -- and fails here. That asymmetry is why
    /// this is a separate test rather than one more row.
    #[test]
    fn no_words_is_absent_and_one_empty_word_is_present() {
        assert!(
            join_command_line(Vec::<&str>::new())
                .into_parts()
                .0
                .is_none()
        );
        assert_eq!(
            join_command_line([""]).into_parts().0.as_deref(),
            Some(&b""[..])
        );
        assert!(Invocation::none().into_parts().0.is_none());
        assert_eq!(
            Invocation::with_argument(Vec::new())
                .into_parts()
                .0
                .as_deref(),
            Some(&b""[..])
        );
    }
}
