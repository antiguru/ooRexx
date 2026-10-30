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

use super::{EEXIST, finish_move, move_file};

fn bytes(path: &Path) -> &[u8] {
    path.as_os_str().as_bytes()
}

/// A directory of this run's own under `parent`, removed when dropped, so a
/// failed assertion leaves nothing behind and two runs never share one.
struct Scratch(PathBuf);

impl Scratch {
    fn new(parent: &Path) -> Scratch {
        use std::hash::BuildHasher as _;
        let unique = std::hash::RandomState::new().hash_one(std::process::id());
        let path = parent.join(format!(
            "rexxutil-move-{}-{unique:016x}",
            std::process::id()
        ));
        fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("cannot create {}: {e}", path.display()));
        Scratch(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// **A move across file systems copies and then unlinks**, keeping the
/// data, the mode and the modification time of a file and keeping a link a
/// link, and a target that is a dangling link answers `EEXIST`. The oracle answers an
/// uninitialised errno here (`corpus/oracle-crashes.txt` entry 39), so this is
/// this crate's own answer, the one this checkout's `interpreter/` gives. The
/// initial rename is refused with `EXDEV`, then the production fallback runs
/// against real files without requiring a second mounted file system.
#[test]
fn a_move_across_file_systems_copies_and_unlinks() {
    let here = Scratch::new(&std::env::temp_dir());
    let there = Scratch::new(&std::env::temp_dir());

    let cross_device_move = |from: &Path, to: &Path| {
        finish_move(bytes(from), bytes(to), Err(rustix::io::Errno::XDEV))
    };

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
    assert_eq!(cross_device_move(&from, &to), 0);
    assert!(!from.exists(), "the source is unlinked");
    assert_eq!(fs::read(&to).expect("the target"), b"data");
    let meta = fs::metadata(&to).expect("the target");
    assert_eq!(meta.permissions().mode() & 0o777, 0o640);
    assert_eq!(meta.mtime(), 1_000_000_000);

    let link = here.join("link");
    std::os::unix::fs::symlink("f.txt", &link).expect("a link");
    let moved = there.join("link");
    assert_eq!(cross_device_move(&link, &moved), 0);
    assert!(fs::symlink_metadata(&link).is_err(), "the link is unlinked");
    assert_eq!(
        fs::read_link(&moved).expect("still a link"),
        Path::new("f.txt")
    );

    let second = here.join("g.txt");
    fs::write(&second, b"more").expect("a second source");
    let dangling = there.join("dangling");
    std::os::unix::fs::symlink("nowhere", &dangling).expect("a dangling link");
    assert_eq!(cross_device_move(&second, &dangling), EEXIST);
    assert!(
        !there.join("nowhere").exists(),
        "nothing is written through the link"
    );
    assert_eq!(fs::read(&second).expect("the source stays"), b"more");
}

#[test]
fn a_move_on_one_file_system_renames_without_copying() {
    let scratch = Scratch::new(&std::env::temp_dir());
    let from = scratch.join("from");
    let to = scratch.join("to");
    fs::write(&from, b"renamed").expect("source");
    let before = fs::metadata(&from).expect("source metadata");
    assert_eq!(move_file(bytes(&from), bytes(&to)), 0);
    assert!(!from.exists());
    assert_eq!(fs::read(&to).expect("target"), b"renamed");
    assert_eq!(fs::metadata(&to).expect("target metadata").ino(), before.ino());
    fs::write(&from, b"kept").expect("second source");
    assert_eq!(move_file(bytes(&from), bytes(&to)), EEXIST);
    assert_eq!(fs::read(&from).expect("source remains"), b"kept");
    assert_eq!(finish_move(bytes(&from), bytes(&to), Err(rustix::io::Errno::ACCESS)), 13);
}
