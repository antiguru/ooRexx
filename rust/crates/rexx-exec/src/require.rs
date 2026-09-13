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

//! Where a `::REQUIRES` name is looked for: the routes it is searched over,
//! the extensions appended to it, and the name the file is then known by.

/// The extension `RESOLVE_REQUIRES` tries ahead of every other, and the one
/// thing that makes a `::REQUIRES` search differ from an external call's.
const REQUIRES_EXTENSION: &str = ".cls";

/// The extensions `SysInterpreterInstance::initialize` appends, in its order
/// (`platform/unix/SysInterpreterInstance.cpp:73`).
const DEFAULT_EXTENSIONS: [&str; 2] = [".REX", ".rex"];

/// The current directory's entry, which `SysSearchPath` adds as this literal
/// rather than as an expanded path.
const CURRENT_DIRECTORY: &str = ".";

/// The path entries a `::REQUIRES` inside a package at `program_dir` is
/// searched over, in order: the requiring program's own directory, the current
/// directory, `REXX_PATH`, then `PATH`.
pub(crate) fn search_entries(
    program_dir: Option<&str>,
    rexx_path: Option<&str>,
    sys_path: Option<&str>,
) -> Vec<String> {
    let mut entries: Vec<String> = Vec::new();
    let mut push = |entry: &str| {
        if !entry.is_empty() {
            entries.push(entry.to_string());
        }
    };
    if let Some(dir) = program_dir {
        push(dir);
    }
    push(CURRENT_DIRECTORY);
    for path in [rexx_path, sys_path].into_iter().flatten() {
        for entry in path.split(PATH_SEPARATOR) {
            push(entry);
        }
    }
    entries
}

/// The byte separating the entries of `REXX_PATH` and `PATH`.
const PATH_SEPARATOR: char = ':';

/// Every path a `::REQUIRES` of `name` tries, in the order it tries them.
pub(crate) fn candidates(
    name: &str,
    entries: &[String],
    parent_extension: Option<&str>,
    requires: bool,
) -> Vec<Vec<String>> {
    // **ASCII, because `strlower` is.** A name with a non-ASCII letter lowers
    // to itself here and takes one pass, where Unicode lowercasing would
    // invent a second spelling the oracle never tries.
    let lower = name.to_ascii_lowercase();
    // `primitiveSearchName`'s own `iterations`: the second pass exists only
    // where lowercasing changes the name.
    let spellings: &[&str] = if lower == name {
        &[name]
    } else {
        &[name, &lower]
    };

    let mut extensions: Vec<Option<&str>> = Vec::new();
    if has_extension(name) {
        extensions.push(None);
    } else {
        if requires {
            extensions.push(Some(REQUIRES_EXTENSION));
        }
        extensions.extend(parent_extension.map(Some));
        extensions.extend(DEFAULT_EXTENSIONS.iter().map(|ext| Some(*ext)));
        extensions.push(None);
    }

    let mut out: Vec<Vec<String>> = Vec::new();
    for extension in extensions {
        for spelling in spellings {
            let candidate = format!("{spelling}{}", extension.unwrap_or(""));
            // Qualified enough to name a file on its own, so the path is not
            // searched at all -- `SysFileSystem::primitiveSearchName`'s own
            // branch, and `searchPath` repeats the test for the same reason.
            if has_directory(&candidate) {
                out.push(vec![candidate]);
                continue;
            }
            out.push(
                entries
                    .iter()
                    .map(|entry| format!("{}/{candidate}", entry.trim_end_matches('/')))
                    .collect(),
            );
        }
    }
    out
}

/// What one candidate path's `stat` found.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Found {
    /// A regular file. The search ends here.
    Regular,
    /// Something else that exists -- a directory, a device, a socket.
    Other,
    /// Nothing at that path.
    Missing,
}

/// The first regular file among `groups`, in order, or `None` when no group
/// holds one.
///
/// **A group is abandoned whole the moment one of its paths exists and is not
/// a regular file.** `searchPath` returns false there rather than trying its
/// remaining entries, so a directory named like the target hides a real file
/// further along the path -- measured, a `main/zdir.rex` directory makes a
/// call find nothing even though `cwd/zdir.rex` is a file. The next
/// spelling-and-extension group still runs.
pub(crate) fn first_regular(
    groups: &[Vec<String>],
    mut probe: impl FnMut(&str) -> Found,
) -> Option<String> {
    for group in groups {
        for candidate in group {
            match probe(candidate) {
                Found::Regular => return Some(candidate.clone()),
                Found::Other => break,
                Found::Missing => {}
            }
        }
    }
    None
}

/// The directory `path` names, trailing separator included, or `None` for a
/// name with no directory part -- `SysFileSystem::extractDirectory`.
pub(crate) fn program_directory(path: &str) -> Option<&str> {
    path.rfind('/').map(|at| &path[..=at])
}

/// The extension `path` ends with, leading dot included, or `None` for a name
/// whose last component carries none -- `SysFileSystem::extractExtension`.
pub(crate) fn program_extension(path: &str) -> Option<&str> {
    let last = path.rfind('/').map_or(path, |at| &path[at + 1..]);
    // **The first byte of the component is never examined.**
    // `SysFileSystem::hasExtension` scans backwards while `name < endPtr`, so
    // a name that is nothing but a dotted suffix has no extension: `.hid` is
    // extensionless where `a.b` is qualified.
    last.rfind('.').filter(|at| *at > 0).map(|at| &last[at..])
}

/// Whether `name`'s last component carries an extension --
/// `SysFileSystem::hasExtension`.
fn has_extension(name: &str) -> bool {
    program_extension(name).is_some()
}

/// Whether `name` carries enough directory information to be checked directly
/// rather than searched for along the path --
/// `SysFileSystem::hasDirectory`.
fn has_directory(name: &str) -> bool {
    name.starts_with('~')
        || name.starts_with('/')
        || name.starts_with("./")
        || name.starts_with("../")
}

pub(crate) use crate::paths::normalize;

#[cfg(test)]
mod tests {
    use super::*;

    /// The four routes, in the oracle's own order: the requiring program's
    /// directory first, then the current directory, then `REXX_PATH`, then
    /// `PATH`.
    #[test]
    fn the_four_routes_are_searched_in_the_oracles_order() {
        let entries = search_entries(Some("/prog/"), Some("/rp1:/rp2"), Some("/pp"));
        assert_eq!(entries, vec!["/prog/", ".", "/rp1", "/rp2", "/pp"]);
        // One group: the name carries its own extension, so there is a single
        // spelling and a single extension, and the five entries are the paths
        // that one group tries in turn.
        let tried = candidates("lib.rex", &entries, Some(".rex"), true);
        assert_eq!(
            tried,
            vec![vec![
                "/prog/lib.rex",
                "./lib.rex",
                "/rp1/lib.rex",
                "/rp2/lib.rex",
                "/pp/lib.rex",
            ]]
        );
    }

    /// An absent `REXX_PATH` removes its entries and leaves the other three
    /// routes in place, rather than shifting `PATH` into its position or
    /// dropping the search.
    #[test]
    fn an_absent_environment_entry_removes_only_itself() {
        assert_eq!(
            search_entries(Some("/prog/"), None, Some("/pp")),
            vec!["/prog/", ".", "/pp"]
        );
        assert_eq!(search_entries(None, None, None), vec!["."]);
        // The `::` case `SysFileSystem::searchPath` skips.
        assert_eq!(
            search_entries(None, Some("/a::/b"), None),
            vec![".", "/a", "/b"]
        );
    }

    /// The extension order, and that a whole pass over the path happens per
    /// extension rather than per directory.
    #[test]
    fn the_extension_order_is_cls_then_the_parents_then_the_defaults() {
        let entries = search_entries(Some("/prog/"), None, None);
        assert_eq!(
            candidates("lib", &entries, Some(".rex"), true),
            vec![
                vec!["/prog/lib.cls", "./lib.cls"],
                vec!["/prog/lib.rex", "./lib.rex"],
                vec!["/prog/lib.REX", "./lib.REX"],
                vec!["/prog/lib.rex", "./lib.rex"],
                vec!["/prog/lib", "./lib"],
            ]
        );
        // A requiring program with no extension of its own drops that step
        // and nothing else.
        assert_eq!(
            candidates("lib", &["/prog/".to_string()], None, true),
            vec![
                vec!["/prog/lib.cls"],
                vec!["/prog/lib.REX"],
                vec!["/prog/lib.rex"],
                vec!["/prog/lib"],
            ]
        );
    }

    /// `RESOLVE_DEFAULT` drops the `.cls` step and keeps every other.
    #[test]
    fn the_default_resolve_does_not_try_the_requires_extension() {
        assert_eq!(
            candidates("lib", &["/prog/".to_string()], None, false),
            vec![
                vec!["/prog/lib.REX"],
                vec!["/prog/lib.rex"],
                vec!["/prog/lib"],
            ]
        );
    }

    /// A name that already carries an extension gets none appended.
    #[test]
    fn a_name_with_an_extension_is_searched_once() {
        assert_eq!(
            candidates("lib.rex", &["/prog/".to_string()], Some(".rex"), true),
            vec![vec!["/prog/lib.rex"]]
        );
    }

    /// The lower-case second pass, and that it is the whole name that is
    /// lowered while the extension keeps the case it was appended in.
    #[test]
    fn each_extension_is_tried_in_the_written_case_and_then_in_lower_case() {
        assert_eq!(
            candidates("LIB.REX", &["/prog/".to_string()], None, true),
            vec![vec!["/prog/LIB.REX"], vec!["/prog/lib.rex"]]
        );
        assert_eq!(
            candidates("LIB", &["/prog/".to_string()], None, true),
            vec![
                vec!["/prog/LIB.cls"],
                vec!["/prog/lib.cls"],
                vec!["/prog/LIB.REX"],
                vec!["/prog/lib.REX"],
                vec!["/prog/LIB.rex"],
                vec!["/prog/lib.rex"],
                vec!["/prog/LIB"],
                vec!["/prog/lib"],
            ]
        );
    }

    /// The second spelling is the ASCII lowering, so a name whose only
    /// upper-case letter is outside ASCII lowers to itself and takes one pass.
    /// Unicode lowercasing would add a spelling `strlower` never produces.
    #[test]
    fn the_lower_case_pass_is_ascii_only() {
        assert_eq!(
            candidates("LIB\u{00c9}", &["/prog/".to_string()], None, false),
            vec![
                vec!["/prog/LIB\u{00c9}.REX"],
                vec!["/prog/lib\u{00c9}.REX"],
                vec!["/prog/LIB\u{00c9}.rex"],
                vec!["/prog/lib\u{00c9}.rex"],
                vec!["/prog/LIB\u{00c9}"],
                vec!["/prog/lib\u{00c9}"],
            ]
        );
        assert_eq!(
            candidates("\u{00c9}", &["/prog/".to_string()], None, false),
            vec![
                vec!["/prog/\u{00c9}.REX"],
                vec!["/prog/\u{00c9}.rex"],
                vec!["/prog/\u{00c9}"],
            ]
        );
    }

    /// A name qualified enough to stand on its own is checked directly and the
    /// path is not searched, where a name merely carrying a directory is
    /// searched over every entry.
    #[test]
    fn a_directory_qualified_name_bypasses_the_path() {
        let entries = search_entries(Some("/prog/"), None, None);
        assert_eq!(
            candidates("./sub/lib.rex", &entries, None, true),
            vec![vec!["./sub/lib.rex"]]
        );
        assert_eq!(
            candidates("/abs/lib.rex", &entries, None, true),
            vec![vec!["/abs/lib.rex"]]
        );
        assert_eq!(
            candidates("sub/lib.rex", &entries, None, true),
            vec![vec!["/prog/sub/lib.rex", "./sub/lib.rex"]]
        );
    }

    /// A path that exists and is not a regular file abandons the rest of its
    /// own spelling and extension, and only that group.
    #[test]
    fn a_non_regular_hit_abandons_the_rest_of_its_group() {
        let groups = vec![
            vec!["/main/zdir.rex".to_string(), "/cwd/zdir.rex".to_string()],
            vec!["/main/zdir".to_string(), "/cwd/zdir".to_string()],
        ];
        // `/main/zdir.rex` is a directory, so the file at `/cwd/zdir.rex` is
        // never reached -- but the extensionless group still runs.
        assert_eq!(
            first_regular(&groups, |path| match path {
                "/main/zdir.rex" => Found::Other,
                "/cwd/zdir.rex" | "/cwd/zdir" => Found::Regular,
                _ => Found::Missing,
            })
            .as_deref(),
            Some("/cwd/zdir")
        );
        // The control: with nothing in the way, the same file wins.
        assert_eq!(
            first_regular(&groups, |path| match path {
                "/cwd/zdir.rex" | "/cwd/zdir" => Found::Regular,
                _ => Found::Missing,
            })
            .as_deref(),
            Some("/cwd/zdir.rex")
        );
        assert_eq!(first_regular(&groups, |_| Found::Missing), None);
    }

    /// `sub/lib` has no extension, because the scan backwards for a dot stops
    /// at the first directory separator.
    #[test]
    fn an_extension_is_looked_for_in_the_last_component_alone() {
        assert!(has_extension("lib.rex"));
        assert!(!has_extension("lib"));
        assert!(!has_extension("/a.b/lib"));
        assert!(has_extension("/a.b/lib.cls"));
        assert_eq!(program_extension("/a/b/main.rex"), Some(".rex"));
        assert_eq!(program_extension("/a.b/main"), None);
        assert_eq!(program_directory("/a/b/main.rex"), Some("/a/b/"));
        assert_eq!(program_directory("main.rex"), None);
    }

    /// A leading dot is not an extension: the backwards scan never reaches
    /// the component's first byte, so a dotfile is extensionless and takes
    /// the whole extension list, where `a.b` is qualified and takes none.
    #[test]
    fn a_leading_dot_is_not_an_extension() {
        assert!(!has_extension(".hid"));
        assert!(!has_extension("/a/b/.hid"));
        assert!(!has_extension("./.hid"));
        assert_eq!(program_extension(".hid"), None);
        assert_eq!(program_extension("/a/b/.hid"), None);
        assert!(has_extension("a.b"));
        assert_eq!(program_extension("a.b"), Some(".b"));
        assert_eq!(program_extension(".hid.rex"), Some(".rex"));
    }
}
