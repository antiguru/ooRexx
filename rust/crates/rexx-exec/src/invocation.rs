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
//! ```text
//! rexx p.rex            ->  arg() = 0, arg(1) omitted
//! rexx p.rex a b c      ->  arg() = 1, arg(1) = "a b c"
//! rexx p.rex "a b c"    ->  arg() = 1, arg(1) = "a b c"
//! rexx p.rex a,b,c      ->  arg() = 1, arg(1) = "a,b,c"   (commas do not split)
//! rexx p.rex ""         ->  arg() = 1, arg(1) = ""        (empty, but PRESENT)
//! ```
//! ```text
//! rexx p.rex     ->  <P>   -- nothing bound, so `p` reads as its own name
//! rexx p.rex ""  ->  <>    -- bound to the null string
//! ```
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

use std::time::Duration;

/// What a command line supplied to the program being run.
pub struct Invocation {
    /// The one argument string, or `None` when the command line supplied no
    /// argument. See the module doc for why the two are different states and
    /// how each is observable.
    argument: Option<Vec<u8>>,
    /// Where `.input` reads its lines from.
    input: ProgramInput,
    /// How long the run may take before the interpreter abandons it, or
    /// `None` for no bound at all -- which is what every shipped caller
    /// passes. See [`Invocation::with_deadline`].
    deadline: Option<Duration>,
}

/// Where `.input` -- the position `PULL`, `PARSE PULL` and `PARSE LINEIN` all
/// advance -- reads its lines from.
pub enum ProgramInput {
    /// Nothing to read: every line read answers the null string.
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
    pub fn none() -> Invocation {
        Invocation {
            argument: None,
            input: ProgramInput::Nothing,
            deadline: None,
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

    /// The same invocation, abandoned if the run is still executing clauses
    /// `deadline` after it starts.
    pub fn with_deadline(self, deadline: Duration) -> Invocation {
        Invocation {
            deadline: Some(deadline),
            ..self
        }
    }

    /// The argument string, if there is one, where `.input` reads from, which
    /// engine runs the bodies, and how long the run may take.
    pub(crate) fn into_parts(self) -> (Option<Vec<u8>>, ProgramInput, Option<Duration>) {
        (self.argument, self.input, self.deadline)
    }
}

/// The one argument string a list of command-line words becomes, or `None`
/// when the list is empty.
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
