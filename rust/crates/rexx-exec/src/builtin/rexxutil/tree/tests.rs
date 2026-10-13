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

use std::fs;
use std::os::unix::ffi::OsStrExt as _;
use std::path::PathBuf;

use rustix::fs::FileType;

use super::{TreeOptions, entries, entry_line, matches};

/// A directory of this run's own under the system's temporary directory,
/// removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Scratch {
        use std::hash::BuildHasher as _;
        let unique = std::hash::RandomState::new().hash_one(std::process::id());
        let path = std::env::temp_dir().join(format!(
            "rexxutil-tree-{}-{unique:016x}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// **A dangling link is listed as a link, with the link's own size**, and
/// counts as a file for `F` and `D`. The oracle's `stat64` fails on it and
/// the line carries whatever its buffer last held (`corpus/oracle-crashes.txt`
/// entry 40), so this is this crate's own answer. The regular file beside it
/// is the control that the listing reached the directory.
#[test]
fn a_dangling_link_is_listed_by_its_own_status() {
    let scratch = Scratch::new();
    fs::write(scratch.0.join("plain"), b"abc").expect("a file");
    std::os::unix::fs::symlink("nowhere", scratch.0.join("dangling")).expect("a link");
    let mut directory = scratch.0.as_os_str().as_bytes().to_vec();
    directory.push(b'/');

    let found = entries(&directory);
    let kind = |wanted: &[u8]| {
        let (_, stat) = found
            .iter()
            .find(|(name, _)| name == wanted)
            .unwrap_or_else(|| panic!("{} is listed", String::from_utf8_lossy(wanted)));
        (FileType::from_raw_mode(stat.st_mode), stat.st_size)
    };
    assert_eq!(kind(b"plain"), (FileType::RegularFile, 3));
    assert_eq!(kind(b"dangling"), (FileType::Symlink, 7));

    let (_, stat) = found
        .iter()
        .find(|(name, _)| name == b"dangling")
        .expect("listed");
    let options = TreeOptions::parse(b"F").expect("valid");
    let zone = tz::TimeZone::utc();
    let line = entry_line(options, &zone, b"/x/dangling".to_vec(), stat).expect("a file");
    assert!(
        line.ends_with(b"           7  lrwxrwxrwx  /x/dangling"),
        "{}",
        String::from_utf8_lossy(&line)
    );
    let directories = TreeOptions::parse(b"D").expect("valid");
    assert!(entry_line(directories, &zone, b"/x/dangling".to_vec(), stat).is_none());
}

/// Bracket forms the corpus witness does not reach: `^` negation, a `]`
/// first, an unterminated `[`, a class, an unknown class, and a folded range
/// whose ends straddle the letters, which `FNM_CASEFOLD` folds to lower case.
#[test]
fn bracket_expressions_match_as_fnmatch_does() {
    assert!(matches(b"[^a]x", b"bx", false));
    assert!(!matches(b"[^a]x", b"ax", false));
    assert!(matches(b"[]x]", b"]", false));
    assert!(matches(b"a[", b"a[", false));
    assert!(!matches(b"a[", b"ab", false));
    assert!(matches(b"[[:digit:]]*", b"7up", false));
    assert!(!matches(b"[[:digit:]]*", b"up", false));
    assert!(!matches(b"[[:nosuch:]a]", b"a", false));
    assert!(matches(b"[_-a]", b"`", true));
    assert!(matches(b"[_-a]", b"A", true));
    assert!(!matches(b"[_-a]", b"B", true));
}
