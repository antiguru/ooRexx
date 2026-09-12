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
/// still normalises.
pub(crate) fn normalize(path: &str, cwd: &str) -> String {
    let absolute = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("{}/{path}", cwd.trim_end_matches('/'))
    };
    let mut parts: Vec<&str> = Vec::new();
    for part in absolute.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    format!("/{}", parts.join("/"))
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
}
