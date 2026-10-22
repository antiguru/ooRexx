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

//! The compiled program image `rexxc` writes: the oracle's `#!` line and
//! `ProgramMetaData` header layout (`classes/support/ProgramMetaData.cpp`),
//! with this build's own magic number and format version, then the name of
//! the routine the image holds, as a 4-byte little-endian length and its
//! bytes, then the checked source. No compatibility is promised between
//! versions of this crate.

use rexx_inventory::errors::{E3_903, E3_905};
use rexx_parse::{ProgramSource, SourceKind};

use crate::builtin::convert::{base64_digit, encode_base64_bytes};
use crate::error::Raised;

/// The courtesy line every image starts with.
const SHEBANG: &[u8] = b"#!/usr/bin/env rexx\n";

/// The `fileTag` the binary form carries, compared as `strcmp` does: up to
/// and including its terminator.
const TAG: &[u8] = b"/**/@REXX\0";

/// What the encoded form carries in place of the header.
const ENCODED_TAG: &[u8] = b"/**/@REXX@\n";

/// The second line an array of lines holds when it is an encoded image
/// (`LanguageParser`'s `ENCODED_NEEDLE`).
const ENCODED_LINE: &[u8] = b"/**/@REXX@";

/// `ProgramMetaData`'s 64-bit size, up to `imageData`.
const HEADER_BYTES: usize = 48;

/// Where the source length sits in the header.
const LENGTH_AT: usize = 40;

/// The oracle's is 11111, so the oracle's `validate` refuses every image
/// this crate writes with 3.903 rather than reading the source as its own
/// flattened heap.
const MAGIC: u16 = 0x7273;

/// This crate's image format.
const FORMAT_VERSION: u16 = 2;

/// The width of the routine name's length.
const NAME_LENGTH_BYTES: usize = 4;

/// The base64 line length, `ProgramMetaData::encodingChunkLength`.
const CHUNK: usize = 72;

/// What a program file's bytes turn out to hold.
#[derive(Debug, PartialEq, Eq)]
pub enum Restore {
    /// Program source: after an optional `#!` line, neither tag.
    Source,
    /// An image this build writes: the source it holds and the name it
    /// gives that source's routine.
    Compiled { source: Vec<u8>, name: Vec<u8> },
    /// A header this build does not write, or a damaged image: 3.903.
    Rejected,
    /// The encoded tag over text the oracle's base64 decoder refuses: 3.905.
    Undecodable,
}

/// The header this build writes, up to its source length field.
fn header_prefix() -> [u8; LENGTH_AT] {
    let mut header = [0u8; LENGTH_AT];
    header[..TAG.len()].copy_from_slice(TAG);
    header[16..18].copy_from_slice(&MAGIC.to_le_bytes());
    header[18..20].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
    // `wordSize`, then `bigEndian` left 0.
    header[20..22].copy_from_slice(&64u16.to_le_bytes());
    // `requiredLevel`, the oracle's language level 6.06.
    header[24..28].copy_from_slice(&60600u32.to_le_bytes());
    header
}

/// The image of `source`, whose routine is called `name`: the oracle names
/// a translated program's routine for the file it was read from and keeps
/// that name in the image (`RoutineClass::flatten`,
/// `classes/RoutineClass.cpp:148`). Binary, or with `encode` the encoded tag
/// and the base64 of the image in lines of [`CHUNK`] digits.
///
/// # Panics
/// If `name` is longer than a `u32` counts.
pub fn write_image(source: &[u8], name: &[u8], encode: bool) -> Vec<u8> {
    let mut image =
        Vec::with_capacity(HEADER_BYTES + NAME_LENGTH_BYTES + name.len() + source.len());
    image.extend_from_slice(&header_prefix());
    let length = u64::try_from(source.len()).expect("a source length fits a u64");
    image.extend_from_slice(&length.to_le_bytes());
    let name_length = u32::try_from(name.len()).expect("a program name fits a u32");
    image.extend_from_slice(&name_length.to_le_bytes());
    image.extend_from_slice(name);
    image.extend_from_slice(source);
    let mut out = SHEBANG.to_vec();
    if encode {
        out.extend_from_slice(ENCODED_TAG);
        for line in encode_base64_bytes(&image).chunks(CHUNK) {
            out.extend_from_slice(line);
            out.push(b'\n');
        }
    } else {
        out.extend_from_slice(&image);
    }
    out
}

/// What `bytes` hold, decided as `ProgramMetaData::processRestoreData` and
/// `validate` decide it.
pub fn restore(bytes: &[u8]) -> Restore {
    let mut rest = bytes;
    if rest.starts_with(b"#!") {
        let Some(end) = rest.iter().position(|&byte| byte == b'\n') else {
            return Restore::Source;
        };
        rest = &rest[end + 1..];
    }
    if rest.starts_with(TAG) {
        return image_source(rest);
    }
    if rest.len() > ENCODED_TAG.len() && rest.starts_with(ENCODED_TAG) {
        return match decode(&rest[ENCODED_TAG.len()..]) {
            Some(image) => image_source(&image),
            None => Restore::Undecodable,
        };
    }
    Restore::Source
}

/// The source `text`, a program buffer's bytes, holds: its own, or the
/// source of the image it is (`ProgramMetaData::restore`). A damaged or
/// foreign image raises 3.903 and undecodable base64 3.905, naming `name`.
pub(crate) fn program_source(text: Vec<u8>, name: &[u8]) -> Result<ProgramSource, Raised> {
    match restore(&text) {
        Restore::Source => Ok(ProgramSource::new(text, SourceKind::Program)),
        image => restored(image, name),
    }
}

/// The source an array of lines holds when its second line is the encoded
/// tag, joined with line feeds and restored (`LanguageParser::
/// createProgram(name, array, context)`), or `None` where the lines are
/// source.
pub(crate) fn lines_source(lines: &[&[u8]], name: &[u8]) -> Option<Result<ProgramSource, Raised>> {
    if !encoded_lines(lines) {
        return None;
    }
    joined_source(lines, name)
}

/// Whether the second of `lines` is the encoded tag, which is what makes
/// the oracle try to restore an array of lines.
pub(crate) fn encoded_lines(lines: &[&[u8]]) -> bool {
    lines.get(1) == Some(&ENCODED_LINE)
}

#[cold]
#[inline(never)]
fn joined_source(lines: &[&[u8]], name: &[u8]) -> Option<Result<ProgramSource, Raised>> {
    match restore(&lines.join(&b'\n')) {
        Restore::Source => None,
        image => Some(restored(image, name)),
    }
}

/// What a buffer that is not source restores to.
#[cold]
#[inline(never)]
fn restored(image: Restore, name: &[u8]) -> Result<ProgramSource, Raised> {
    match image {
        Restore::Compiled { source, name } => Ok(ProgramSource::compiled(source, name.into())),
        Restore::Rejected => Err(Raised::raise(E3_903, [name.to_vec()])),
        Restore::Undecodable => Err(Raised::raise(E3_905, [name.to_vec()])),
        Restore::Source => unreachable!("both callers answer source themselves"),
    }
}

/// The name and source behind a header this build writes, with the source's
/// length exact.
fn image_source(image: &[u8]) -> Restore {
    let Some((header, rest)) = image.split_at_checked(HEADER_BYTES) else {
        return Restore::Rejected;
    };
    let (prefix, length) = header.split_at(LENGTH_AT);
    let length = u64::from_le_bytes(length.try_into().expect("an 8-byte length field"));
    if prefix != header_prefix() {
        return Restore::Rejected;
    }
    let Some((name_length, rest)) = rest.split_at_checked(NAME_LENGTH_BYTES) else {
        return Restore::Rejected;
    };
    let name_length = u32::from_le_bytes(name_length.try_into().expect("a 4-byte length field"));
    let Some((name, source)) = usize::try_from(name_length)
        .ok()
        .and_then(|name_length| rest.split_at_checked(name_length))
    else {
        return Restore::Rejected;
    };
    if u64::try_from(source.len()).ok() != Some(length) {
        return Restore::Rejected;
    }
    Restore::Compiled {
        source: source.to_vec(),
        name: name.to_vec(),
    }
}

/// `StringUtil::decodeBase64` (`classes/support/StringUtil.cpp:1743`): line
/// ends only between quartets, `=` only in a quartet's last two digits, and
/// anything after the padding a line end. `None` is its refusal.
fn decode(text: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut digit = 0;
    let mut held = 0u8;
    let mut index = 0;
    while index < text.len() {
        let byte = text[index];
        index += 1;
        let Some(value) = base64_digit(byte) else {
            match (byte, digit) {
                (b'=', 2) => {
                    if text.get(index) != Some(&b'=') {
                        return None;
                    }
                    index += 1;
                    break;
                }
                (b'=', 3) => break,
                (b'\n' | b'\r', 0) => continue,
                _ => return None,
            }
        };
        match digit {
            0 => held = value << 2,
            1 => {
                out.push(held | value >> 4);
                held = value << 4;
            }
            2 => {
                out.push(held | value >> 2);
                held = value << 6;
            }
            _ => out.push(held | value),
        }
        digit = (digit + 1) % 4;
    }
    text[index..]
        .iter()
        .all(|byte| matches!(byte, b'\n' | b'\r'))
        .then_some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &[u8] = b"say 'hello'\n::routine r\nreturn 1\n";

    const NAME: &[u8] = b"/d/p.rex";

    fn compiled(source: &[u8], name: &[u8]) -> Restore {
        Restore::Compiled {
            source: source.to_vec(),
            name: name.to_vec(),
        }
    }

    #[test]
    fn both_forms_round_trip() {
        for encode in [false, true] {
            let image = write_image(SOURCE, NAME, encode);
            assert!(image.starts_with(b"#!/usr/bin/env rexx\n/**/@REXX"));
            assert_eq!(restore(&image), compiled(SOURCE, NAME));
        }
        assert_eq!(restore(&write_image(b"", b"", true)), compiled(b"", b""));
    }

    #[test]
    fn the_binary_form_is_the_oracle_layout() {
        let image = write_image(SOURCE, NAME, false);
        let name_at = SHEBANG.len() + HEADER_BYTES + NAME_LENGTH_BYTES;
        assert_eq!(image.len(), name_at + NAME.len() + SOURCE.len());
        assert_eq!(&image[20..36], b"/**/@REXX\0\0\0\0\0\0\0");
        assert_ne!(u16::from_le_bytes([image[36], image[37]]), 11111);
        assert_eq!(&image[name_at - NAME_LENGTH_BYTES..name_at], &[8, 0, 0, 0]);
        assert_eq!(&image[name_at..name_at + NAME.len()], NAME);
        assert_eq!(&image[name_at + NAME.len()..], SOURCE);
    }

    #[test]
    fn the_encoded_form_has_full_lines_and_ends_in_a_line_end() {
        let image = write_image(&[b'x'; 500], NAME, true);
        let body = &image[SHEBANG.len() + ENCODED_TAG.len()..];
        let lines: Vec<&[u8]> = body.split(|&byte| byte == b'\n').collect();
        assert_eq!(lines.last(), Some(&&b""[..]));
        let lines = &lines[..lines.len() - 1];
        assert!(
            lines[..lines.len() - 1]
                .iter()
                .all(|line| line.len() == CHUNK)
        );
        assert!(lines[lines.len() - 1].len() <= CHUNK);
    }

    #[test]
    fn a_foreign_or_damaged_header_is_rejected() {
        let image = write_image(SOURCE, NAME, false);
        let mut oracle = image.clone();
        oracle[36..38].copy_from_slice(&11111u16.to_le_bytes());
        let mut version = image.clone();
        version[38] = version[38].wrapping_add(1);
        let mut past = image.clone();
        past[SHEBANG.len() + LENGTH_AT] += 1;
        let mut short = image.clone();
        short[SHEBANG.len() + LENGTH_AT] -= 1;
        let name_length = SHEBANG.len() + HEADER_BYTES;
        let mut long_name = image.clone();
        long_name[name_length] += 1;
        let mut short_name = image.clone();
        short_name[name_length] -= 1;
        let mut huge_name = image.clone();
        huge_name[name_length + 3] = 0xff;
        for damaged in [
            oracle, version, past, short, long_name, short_name, huge_name,
        ] {
            assert_eq!(restore(&damaged), Restore::Rejected);
        }
        for cut in [
            SHEBANG.len() + 10,
            SHEBANG.len() + 30,
            SHEBANG.len() + HEADER_BYTES + 2,
            SHEBANG.len() + HEADER_BYTES + NAME_LENGTH_BYTES + 3,
            image.len() - 1,
        ] {
            assert_eq!(restore(&image[..cut]), Restore::Rejected, "cut at {cut}");
        }
        let encoded = write_image(SOURCE, NAME, true);
        let mut foreign = b"#!/usr/bin/env rexx\n/**/@REXX@\n".to_vec();
        foreign.extend_from_slice(b"QUJD\n");
        assert_eq!(restore(&foreign), Restore::Rejected);
        assert!(matches!(restore(&encoded), Restore::Compiled { .. }));
    }

    #[test]
    fn broken_base64_is_undecodable() {
        let encoded = write_image(SOURCE, NAME, true);
        let at = SHEBANG.len() + ENCODED_TAG.len() + 5;
        let mut broken = encoded.clone();
        broken[at] = b'!';
        assert_eq!(restore(&broken), Restore::Undecodable);
        // Measured, oracle `rexx`: each of these is 3.905.
        assert_eq!(restore(b"/**/@REXX@\nsay 1\n"), Restore::Undecodable);
        assert_eq!(
            restore(b"#!/usr/bin/env rexx\n/**/@REXX@\n!!!!\n"),
            Restore::Undecodable
        );
        // A line end inside a quartet.
        let mut split = encoded.clone();
        split.insert(at, b'\n');
        assert_eq!(restore(&split), Restore::Undecodable);
    }

    #[test]
    fn source_that_merely_starts_like_a_tag_is_source() {
        // Measured, oracle `rexx`: both are parsed, 13.1 on the `@`.
        assert_eq!(restore(b"/**/@REXX say 1\n"), Restore::Source);
        assert_eq!(restore(b"#!/x\n/**/@REXX say 1\n"), Restore::Source);
        assert_eq!(restore(b"#!/usr/bin/env rexx"), Restore::Source);
        assert_eq!(restore(b"/**/@REXX"), Restore::Source);
        assert_eq!(restore(SOURCE), Restore::Source);
    }

    #[test]
    fn the_decoder_follows_the_oracle_rules() {
        assert_eq!(decode(b"QUJD\nREVG\r\n"), Some(b"ABCDEF".to_vec()));
        assert_eq!(decode(b"QQ==\n"), Some(b"A".to_vec()));
        assert_eq!(decode(b"QUI=\n\n"), Some(b"AB".to_vec()));
        assert_eq!(decode(b"QQ=x"), None);
        assert_eq!(decode(b"QQ==x"), None);
        assert_eq!(decode(b"Q==="), None);
        assert_eq!(decode(b"QU\nJD"), None);
        assert_eq!(decode(b""), Some(Vec::new()));
    }

    #[test]
    fn a_restored_source_is_not_traceable() {
        let source = program_source(write_image(SOURCE, NAME, false), b"p").expect("restored");
        assert!(!source.is_traceable());
        assert_eq!(source.image_name(), Some(NAME));
        assert_eq!(source.line(1), Some(&b"say 'hello'"[..]));
        let plain = program_source(SOURCE.to_vec(), b"p").expect("source");
        assert!(plain.is_traceable());
        assert_eq!(plain.image_name(), None);
        let refused = program_source(b"/**/@REXX@\n!!!!\n".to_vec(), b"p").err();
        assert_eq!(
            refused.map(|raised| (raised.number, raised.sub)),
            Some((3, 905))
        );
    }

    #[test]
    fn only_lines_whose_second_is_the_encoded_tag_are_restored() {
        let image = write_image(SOURCE, NAME, true);
        let lines: Vec<&[u8]> = image.split(|&byte| byte == b'\n').collect();
        let restored = lines_source(&lines, b"p")
            .expect("an image")
            .expect("restored");
        assert!(!restored.is_traceable());
        assert!(lines_source(&[b"say 1".as_slice(), b"say 2"], b"p").is_none());
        // The tag as the second line of what is not an image: source.
        assert!(lines_source(&[b"say 1".as_slice(), ENCODED_LINE, b"QUJD"], b"p").is_none());
        let refused = lines_source(&[b"#!x".as_slice(), ENCODED_LINE, b"QUJD"], b"p")
            .expect("an image")
            .err();
        assert_eq!(
            refused.map(|raised| (raised.number, raised.sub)),
            Some((3, 903))
        );
    }
}
