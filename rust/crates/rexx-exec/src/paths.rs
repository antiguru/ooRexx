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

//! Turning a name a program wrote into the one path this interpreter opens.
//! Every consumer -- the streams, `.File`, the `Sys*` routines, `QUALIFY`, the
//! external-routine search and a command's working directory -- goes through
//! here, so all of them answer the same question the same way and none reads
//! the process's own directory.

/// `path` made absolute against `cwd` and reduced to one spelling:
/// `SysFileSystem::canonicalizeName` followed by `normalizePathName`
/// (`platform/unix/SysFileSystem.cpp:628`, `:687`). Purely lexical -- the
/// oracle avoids `realpath` here precisely so that a path that does not exist
/// still normalises. Each `..` removes the component before it, which the
/// oracle's loop does not do after a doubled slash or after a component
/// starting with `.` (Deviation 34).
pub(crate) fn normalize(path: &str, cwd: &str) -> String {
    String::from_utf8(normalize_bytes(path.as_bytes(), cwd.as_bytes()))
        .expect("removing whole components keeps UTF-8 valid")
}

/// [`normalize`] over bytes, for a name that need not be UTF-8.
pub(crate) fn normalize_bytes(path: &[u8], cwd: &[u8]) -> Vec<u8> {
    let mut absolute = Vec::with_capacity(cwd.len() + path.len() + 1);
    if path.first() != Some(&b'/') {
        absolute.extend_from_slice(cwd);
        absolute.push(b'/');
    }
    absolute.extend_from_slice(path);
    let mut parts: Vec<&[u8]> = Vec::new();
    for part in absolute.split(|byte| *byte == b'/') {
        match part {
            b"" | b"." => {}
            b".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    let mut out = Vec::with_capacity(absolute.len());
    for part in &parts {
        out.push(b'/');
        out.extend_from_slice(part);
    }
    if out.is_empty() {
        out.push(b'/');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `normalizePathName`'s reductions, each against the C++ loop's own
    /// answer.
    #[test]
    fn normalize_makes_one_spelling_of_a_path() {
        assert_eq!(normalize("lib.rex", "/a/b"), "/a/b/lib.rex");
        assert_eq!(normalize("./lib.rex", "/a/b"), "/a/b/lib.rex");
        assert_eq!(normalize("../lib.rex", "/a/b"), "/a/lib.rex");
        assert_eq!(normalize("/a//b/./c/../d", "/x"), "/a/b/d");
        assert_eq!(normalize("/a/b/", "/x"), "/a/b");
        assert_eq!(normalize("/..", "/x"), "/");
        // `..b` is a file name, not a parent reference.
        assert_eq!(normalize("/a/..b", "/x"), "/a/..b");
    }

    /// Deviation 34: a `..` removes the component before it, after a doubled
    /// slash and after a component starting with `.` too. The oracle's
    /// `normalizePathName` answers the second column, measured through
    /// `QUALIFY`, which names a different file.
    #[test]
    fn a_parent_reference_removes_one_component_where_the_oracle_does_not() {
        for (path, oracle, this_crate) in [
            ("/a/b//../c", "/a/b/c", "/a/c"),
            ("/a/b///../c", "/a/b/c", "/a/c"),
            ("/a/b/.//../c", "/a/b/c", "/a/c"),
            ("/a/b//..", "/a/b", "/a"),
            ("/a/b/c/..//../d", "/a/b/d", "/a/d"),
            ("/a/b/c//../../d", "/a/b/d", "/a/d"),
            ("/a/.hidden/../c", "/c", "/a/c"),
            ("/a/.config/../x", "/x", "/a/x"),
            ("/a/b/..x/../c", "/a/c", "/a/b/c"),
            ("/a/b/.../../c", "/a/c", "/a/b/c"),
        ] {
            assert_ne!(oracle, this_crate);
            assert_eq!(normalize(path, "/x"), this_crate, "{path}");
        }
    }

    /// A name that is not UTF-8 keeps its bytes.
    #[test]
    fn normalize_bytes_keeps_bytes_that_are_not_utf8() {
        assert_eq!(normalize_bytes(b"a/\xff/../\xfe", b"/x/"), b"/x/a/\xfe");
        assert_eq!(normalize_bytes(b"/\xff/", b"/x"), b"/\xff");
    }
}
