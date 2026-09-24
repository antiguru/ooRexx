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

//! The interpreter's version string and the fields `RexxInfo` cuts from it, the
//! platform name and the line terminator.

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
pub(super) const BIT_WIDTH: &[u8] = part(VERSION, SECOND_UNDERSCORE + 1, DASH);

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
