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
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};

use super::{EEXIST, move_file};

fn bytes(path: &Path) -> &[u8] {
    path.as_os_str().as_bytes()
}

/// A fresh empty directory at `path`.
fn fresh(path: PathBuf) -> PathBuf {
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));
    path
}

/// **A move across file systems copies and then unlinks**, keeping the
/// data, the mode and the modification time of a file and keeping a link a
/// link, and a target that is a dangling link answers `EEXIST`. The oracle answers an
/// uninitialised errno here (`corpus/oracle-crashes.txt` entry 39), so this is
/// this crate's own answer, the one this checkout's `interpreter/` gives. The
/// two directories are this crate's target directory and the system's
/// temporary directory, and the test first checks that a rename between them
/// is refused with `EXDEV`.
#[test]
fn a_move_across_file_systems_copies_and_unlinks() {
    let here = fresh(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp/rexxutil-move"));
    let there = fresh(std::env::temp_dir().join(format!("rexxutil-move-{}", std::process::id())));

    let probe = here.join("probe");
    fs::write(&probe, b"").expect("a probe file");
    assert_eq!(
        rustix::fs::rename(bytes(&probe), bytes(&there.join("probe"))),
        Err(rustix::io::Errno::XDEV),
        "this test needs {} and {} on different file systems",
        here.display(),
        there.display()
    );

    let from = here.join("f.txt");
    fs::write(&from, b"data").expect("the source");
    fs::set_permissions(&from, fs::Permissions::from_mode(0o640)).expect("its mode");
    let modified = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
    fs::File::options()
        .write(true)
        .open(&from)
        .and_then(|file| file.set_modified(modified))
        .expect("its time");
    let to = there.join("f.txt");
    assert_eq!(move_file(bytes(&from), bytes(&to)), 0);
    assert!(!from.exists(), "the source is unlinked");
    assert_eq!(fs::read(&to).expect("the target"), b"data");
    let meta = fs::metadata(&to).expect("the target");
    assert_eq!(meta.permissions().mode() & 0o777, 0o640);
    assert_eq!(meta.mtime(), 1_000_000_000);

    let link = here.join("link");
    std::os::unix::fs::symlink("f.txt", &link).expect("a link");
    let moved = there.join("link");
    assert_eq!(move_file(bytes(&link), bytes(&moved)), 0);
    assert!(fs::symlink_metadata(&link).is_err(), "the link is unlinked");
    assert_eq!(
        fs::read_link(&moved).expect("still a link"),
        Path::new("f.txt")
    );

    let second = here.join("g.txt");
    fs::write(&second, b"more").expect("a second source");
    let dangling = there.join("dangling");
    std::os::unix::fs::symlink("nowhere", &dangling).expect("a dangling link");
    assert_eq!(move_file(bytes(&second), bytes(&dangling)), EEXIST);
    assert!(
        !there.join("nowhere").exists(),
        "nothing is written through the link"
    );
    assert_eq!(fs::read(&second).expect("the source stays"), b"more");

    let _ = fs::remove_dir_all(&here);
    let _ = fs::remove_dir_all(&there);
}
