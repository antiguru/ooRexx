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

use super::{
    CATALOG_MAGIC, Catalog, CatalogImage, rexx_message, search_candidates, strerror, substitute,
};

/// A catalog as gencat lays one out: `(set, number, text)` entries hashed
/// into `plane_size` buckets, the host-order table, the swapped table, then
/// the messages.
fn catalog_bytes(plane_size: u32, entries: &[(u32, u32, &str)]) -> Vec<u8> {
    let size = plane_size as usize;
    let mut buckets: Vec<Vec<(u32, u32, u32)>> = vec![Vec::new(); size];
    let mut strings = Vec::new();
    for (set, number, text) in entries {
        let offset = u32::try_from(strings.len()).expect("small");
        buckets[((set + 1) * number) as usize % size].push((set + 1, *number, offset));
        strings.extend_from_slice(text.as_bytes());
        strings.push(0);
    }
    let depth = buckets.iter().map(Vec::len).max().unwrap_or(1).max(1);
    let mut table = vec![0u32; size * depth * 3];
    for (bucket, held) in buckets.iter().enumerate() {
        for (level, (set, number, offset)) in held.iter().enumerate() {
            let at = (level * size + bucket) * 3;
            table[at..at + 3].copy_from_slice(&[*set, *number, *offset]);
        }
    }
    let mut bytes = Vec::new();
    for word in [
        CATALOG_MAGIC,
        plane_size,
        u32::try_from(depth).expect("small"),
    ] {
        bytes.extend_from_slice(&word.to_ne_bytes());
    }
    for word in &table {
        bytes.extend_from_slice(&word.to_ne_bytes());
    }
    for word in &table {
        bytes.extend_from_slice(&word.swap_bytes().to_ne_bytes());
    }
    bytes.extend_from_slice(&strings);
    bytes
}

fn image(bytes: Vec<u8>) -> CatalogImage {
    let mapped = bytes.len().div_ceil(4096) * 4096;
    CatalogImage { bytes, mapped }
}

fn message(catalog: &Catalog, set: i32, number: i32) -> Option<String> {
    catalog
        .message(set, number)
        .map(|text| String::from_utf8(text).expect("ASCII"))
}

#[test]
fn a_catalog_answers_each_message_by_set_and_number() {
    let entries = [
        (1, 1, "one"),
        (1, 2, "two"),
        (2, 5, "five"),
        (100, 1, "far"),
    ];
    let catalog = Catalog::new(image(catalog_bytes(2, &entries))).expect("valid");
    assert_eq!(message(&catalog, 1, 1).as_deref(), Some("one"));
    assert_eq!(message(&catalog, 1, 2).as_deref(), Some("two"));
    assert_eq!(message(&catalog, 2, 5).as_deref(), Some("five"));
    assert_eq!(message(&catalog, 100, 1).as_deref(), Some("far"));
    assert_eq!(message(&catalog, 1, 3), None);
    assert_eq!(message(&catalog, 3, 1), None);
    assert_eq!(message(&catalog, i32::MAX, 1), None);
    assert_eq!(message(&catalog, 1, -1), None);
}

#[test]
fn a_catalog_header_in_the_other_byte_order_still_opens() {
    let mut bytes = catalog_bytes(1, &[(1, 1, "one")]);
    for at in [0, 4, 8] {
        bytes[at..at + 4].reverse();
    }
    let catalog = Catalog::new(image(bytes)).expect("valid");
    assert_eq!(message(&catalog, 1, 1).as_deref(), Some("one"));
}

#[test]
fn a_last_message_without_its_nul_ends_in_the_mapping_zeros() {
    let mut bytes = catalog_bytes(1, &[(1, 1, "one")]);
    bytes.pop();
    let catalog = Catalog::new(image(bytes)).expect("valid");
    assert_eq!(message(&catalog, 1, 1).as_deref(), Some("one"));
}

#[test]
fn a_truncated_table_fails_the_size_test() {
    let bytes = catalog_bytes(4, &[(1, 1, "one"), (1, 2, "two")]);
    assert!(Catalog::new(image(bytes[..20].to_vec())).is_none());
}

/// `corpus/oracle-crashes.txt` entry 42: the oracle divides by the bucket
/// count and dies of `SIGFPE`.
#[test]
fn a_table_of_no_buckets_holds_no_message() {
    let mut bytes = catalog_bytes(1, &[]);
    bytes.truncate(12);
    bytes[4..8].copy_from_slice(&0u32.to_ne_bytes());
    bytes.push(0);
    let catalog = Catalog::new(image(bytes)).expect("glibc opens it");
    assert_eq!(message(&catalog, 1, 1), None);
}

#[test]
fn a_table_reaching_past_the_mapping_is_rejected() {
    let mut bytes = catalog_bytes(1, &[(1, 1, "one")]);
    bytes[4..8].copy_from_slice(&100_000u32.to_ne_bytes());
    assert!(Catalog::new(image(bytes)).is_none());
}

#[test]
fn the_search_path_expands_each_element() {
    let expand = |path: &str, locale: &str| -> Vec<String> {
        search_candidates(path.as_bytes(), b"m.cat", locale.as_bytes())
            .into_iter()
            .map(|candidate| String::from_utf8(candidate).expect("ASCII"))
            .collect()
    };
    assert_eq!(
        expand("/a/%L/%N:/b/%l/%t/%c/%N:", "de_CH.UTF-8"),
        ["/a/de_CH.UTF-8/m.cat", "/b/de/CH/UTF-8/m.cat"]
    );
    assert_eq!(expand("::/c/%N:", "C"), ["m.cat", "m.cat", "/c/m.cat"]);
    assert_eq!(expand("/d/%q/%N:/e/100%%:", "C"), ["/e/100%"]);
    assert_eq!(expand("/f/%t%c:", "C"), ["/f/"]);
    assert_eq!(expand("/g/%t:", "de_"), ["/g/\0"]);
    assert_eq!(expand("/h/%c:", "de."), ["/h/\0"]);
}

#[test]
fn rexx_messages_are_found_by_message_number() {
    assert_eq!(rexx_message(5), Some("System resources exhausted."));
    assert_eq!(
        rexx_message(678),
        Some("Argument &1 must be in the range &2 to &3; found \"&4\".")
    );
    assert_eq!(rexx_message(0), None);
    assert_eq!(rexx_message(-1), None);
}

#[test]
fn substitution_replaces_ampersand_digits_only() {
    let values = [b"abc".to_vec(), Vec::new(), b"&1".to_vec()];
    assert_eq!(substitute(b"&1 + &2 + &3 + &4", &values), b"abc +  + &1 + ");
    assert_eq!(substitute(b"&0 && &", &values), b"&0 && &");
    assert_eq!(substitute(b"&&1&", &values), b"&abc&");
}

/// The text is the standard library's error display less its own suffix,
/// which therefore has to be there for every number.
#[test]
fn strerror_text_drops_exactly_the_standard_library_suffix() {
    for number in (-300..=300).chain([i32::MIN, i32::MAX]) {
        let rendered = std::io::Error::from_raw_os_error(number).to_string();
        assert!(
            rendered.ends_with(&format!(" (os error {number})")),
            "{rendered}"
        );
    }
    assert_eq!(strerror(2), b"No such file or directory");
    assert_eq!(strerror(-1), b"Unknown error -1");
}

/// `corpus/oracle-crashes.txt` entry 41: the oracle formats the message with
/// the raising value's null pointer and dies. This crate answers the raise,
/// trapped as any raise from a routine is.
#[test]
fn a_raising_value_the_message_refers_to_is_trapped() {
    let source = b"signal on syntax\n\
        say SysFormatMessage('&1', .array~of(.k~new))\n\
        exit\n\
        syntax: say 'trapped' condition('O')~code condition('O')~message\n\
        exit\n\
        ::class k\n\
        ::method string\n\
        raise syntax 93.900 array('boom')\n";
    let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&outcome.stderr), "");
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "trapped 93.900 boom.\n"
    );
}

/// `corpus/oracle-crashes.txt` entry 43: the oracle reads the string through
/// the array layout.
#[test]
fn a_make_array_answering_a_string_is_not_an_array() {
    let source = b"signal on syntax\n\
        say SysFormatMessage('&1', .k~new)\n\
        exit\n\
        syntax: say 'trapped' condition('O')~code\n\
        exit\n\
        ::class k\n\
        ::method makearray\n\
        return 'not an array'\n";
    let outcome = crate::run_program("/t.rex", source.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&outcome.stdout), "trapped 98.913\n");
}
