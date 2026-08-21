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

//! Table C's wiring edge set: the documented superclass-or-mixin edge each
//! class carries, read off the indentation of `provide.xml`'s `chi`
//! `<simplelist>`.
//!
//! The list gives **one** edge per class by construction -- its own prose says
//! "Classes inheriting from multiple mixin classes are only listed below one
//! of these mixin classes" -- so a derived edge is a claim that it is *present
//! in* the child's `~superClasses`, never that it is the whole answer.
//!
//! Three rules, and they are not equally witnessed. Rules 2 and 3 are
//! witnessed by running the whole set against the oracle, because their edges
//! fail there when present: `.Object~superClasses~items` is `0` and
//! `.RexxInfo~superClasses` raises `97.1`. **Rule 1 is not**, and
//! [`edges_from_members`] carries its own assertion for that reason; see the
//! assertion's own comment.

use crate::docs::xml::{Member, members};

/// A level of indentation is five `&nbsp;`, verbatim: the entity has no local
/// definition, so it reaches a text-level extractor unresolved.
const INDENT: &str = "&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;";

/// A level-0 member's parent, from the section's own opening sentence --
/// "Rexx provides the following classes belonging to the Object class". This
/// is what makes the edge set larger than the indented members alone.
const ROOT: &str = "Object";

/// One documented hierarchy edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub child: String,
    pub parent: String,
    pub level: usize,
    pub line: usize,
}

/// The `<simplelist>` members of `provide.xml`'s `chi` section.
///
/// Takes the file text rather than a parsed tree so that the caller decides
/// whether comments have been blanked -- which is what lets rule 1's assertion
/// be seen firing instead of merely asserted to work.
pub fn chi_members(provide_xml: &str) -> Vec<Member> {
    let section = provide_xml
        .find("<section id=\"chi\"")
        .unwrap_or_else(|| panic!("no <section id=\"chi\"> in this file"));
    let open = provide_xml[section..]
        .find("<simplelist>")
        .map(|at| section + at)
        .unwrap_or_else(|| panic!("the chi section has no <simplelist>"));
    let close = provide_xml[open..]
        .find("</simplelist>")
        .map(|at| open + at)
        .unwrap_or_else(|| panic!("the chi <simplelist> never closes"));
    assert!(
        !provide_xml[close + "</simplelist>".len()..section_end(provide_xml, section)]
            .contains("<simplelist>"),
        "the chi section holds more than one <simplelist>; the hierarchy block is ambiguous"
    );
    members(&provide_xml[open..close])
        .into_iter()
        .map(|mut m| {
            m.line += provide_xml[..open].bytes().filter(|&b| b == b'\n').count();
            m
        })
        .collect()
}

fn section_end(text: &str, from: usize) -> usize {
    text[from..]
        .find("</section>")
        .map(|at| from + at)
        .unwrap_or(text.len())
}

/// The edge set, under the three rules.
///
/// `excluded` names are still read for the indentation they establish -- a
/// child indented under one of them still gets the right parent -- but emit no
/// edge of their own.
///
/// **The `ArgUtil` assertion.** Rule 1 -- strip comments before reading
/// `<member>`s -- is the one rule the end-to-end oracle run cannot witness:
/// `.ArgUtil~superClasses` is exactly `The Object class`, so an extractor that
/// skips the stripping emits an extra edge the oracle confirms, and the run
/// stays at zero failures over a wrong member set. The assertion is
/// co-extensive with rule 1's whole observable effect here, because the block
/// holds exactly one XML comment and it is the `ArgUtil` member; a comment
/// added upstream later is caught by the both-directions check instead, since
/// the derived set moves and the committed file does not.
pub fn edges_from_members(members: &[Member], excluded: &[&str]) -> Vec<Edge> {
    let mut by_level: Vec<String> = Vec::new();
    let mut out: Vec<Edge> = Vec::new();
    for member in members {
        let Some(linkend) = member.linkend.as_deref() else {
            continue;
        };
        let Some(child) = linkend.strip_prefix("cls") else {
            panic!("chi member at line {} links to {linkend}", member.line);
        };
        let level = indent_level(&member.inner, member.line);
        by_level.truncate(level);
        assert_eq!(
            by_level.len(),
            level,
            "chi member {child} at line {} is indented {level} levels under a shallower parent",
            member.line
        );
        let parent = by_level.last().cloned().unwrap_or_else(|| ROOT.to_string());
        by_level.push(child.to_string());
        // Rule 2: the root emits no self-edge. `clsObject` is itself a level-0
        // member, and `.Object~superClasses~items` is `0`, so that row can
        // never hold.
        if child == parent {
            continue;
        }
        // Rule 3: an excluded name emits no edge. It is still pushed above, so
        // anything indented under it keeps the parent the book gives it.
        if excluded.contains(&child) {
            continue;
        }
        out.push(Edge {
            child: child.to_string(),
            parent,
            level,
            line: member.line,
        });
    }
    assert!(
        !out.iter().any(|e| e.child == "ArgUtil"),
        "ArgUtil is in the derived edge set: the member at provide.xml:844 sits inside an XML \
         comment, so this derivation read a commented-out member as a live one. The oracle \
         confirms that edge -- .ArgUtil~superClasses is The Object class -- so the end-to-end \
         run cannot see this and does not"
    );
    out
}

fn indent_level(inner: &str, line: usize) -> usize {
    let mut rest = inner.trim_start();
    let mut level = 0usize;
    while let Some(tail) = rest.strip_prefix(INDENT) {
        rest = tail;
        level += 1;
    }
    assert!(
        !rest.starts_with("&nbsp;"),
        "chi member at line {line} is indented by a number of &nbsp; that is not a multiple of \
         five, so its level is not the book's"
    );
    level
}

/// `child<TAB>parent<TAB>level<TAB>line`, one per row.
pub fn rows(edges: &[Edge]) -> Vec<String> {
    edges
        .iter()
        .map(|e| format!("{}\t{}\t{}\t{}", e.child, e.parent, e.level, e.line))
        .collect()
}

/// The header the committed file carries, with its D56 stamp.
pub fn header(stamp: &str, excluded: &[&str]) -> Vec<String> {
    vec![
        "Table C's wiring edge set: the documented superclass-or-mixin edge per".into(),
        "class, from the indentation of provide.xml's `chi` <simplelist>.".into(),
        String::new(),
        "One `child<TAB>parent<TAB>level<TAB>line` per line, in document order.".into(),
        "`level` is the member's indentation in units of five &nbsp;; `line` is".into(),
        "its line in provide.xml. Each row claims the parent is PRESENT IN the".into(),
        "child's ~superClasses, never that it is the whole answer: the list".into(),
        "gives one edge per class by construction, and says so in its own".into(),
        "prose.".into(),
        String::new(),
        format!(
            "Three rules. Comments are blanked before the members are read; the \
             root emits no self-edge; and these names emit no edge: {}.",
            excluded.join(", ")
        ),
        String::new(),
        "Derived by `cargo run -p rexx-extract --bin rexx-extract-docs`, whose".into(),
        "module is `src/docs/hierarchy.rs`. Re-derived and compared in both".into(),
        "directions by `tests/extract_docs.rs`.".into(),
        String::new(),
        stamp.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docs::xml::blank_comments;

    const SAMPLE: &str = r#"<section id="chi"><title>The Class Hierarchy</title>
<para>
  <!-- commented ArgUtil (deprecated, won't document) -->
  <simplelist>
  <member><xref linkend="clsAlarm" xrefstyle="template:Alarm class"/></member>
<!--
  <member><xref linkend="clsArgUtil" xrefstyle="template:ArgUtil class"/></member>
-->
  <member><xref linkend="clsCollection" xrefstyle="template:Collection class"/></member>
  <member>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<xref linkend="clsMapCollection" xrefstyle="t"/></member>
  <member>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;<xref linkend="clsBag" xrefstyle="t"/></member>
  <member><xref linkend="clsObject" xrefstyle="template:Object class"/></member>
  <member><xref linkend="clsRexxInfo" xrefstyle="template:RexxInfo class"/></member>
  </simplelist>
</para>
</section>
"#;

    fn derived() -> Vec<Edge> {
        edges_from_members(&chi_members(&blank_comments(SAMPLE)), &["RexxInfo"])
    }

    #[test]
    fn indentation_gives_the_parent_and_level_zero_gives_object() {
        let out = derived();
        let pairs: Vec<(&str, &str)> = out
            .iter()
            .map(|e| (e.child.as_str(), e.parent.as_str()))
            .collect();
        assert_eq!(
            pairs,
            [
                ("Alarm", "Object"),
                ("Collection", "Object"),
                ("MapCollection", "Collection"),
                ("Bag", "MapCollection"),
            ]
        );
        assert_eq!(out[3].level, 2);
    }

    /// Rule 2 and rule 3, each by the row that is absent.
    #[test]
    fn the_root_and_an_excluded_name_emit_no_edge() {
        let out = derived();
        assert!(!out.iter().any(|e| e.child == "Object"));
        assert!(!out.iter().any(|e| e.child == "RexxInfo"));
    }

    /// Rule 1, witnessed rather than asserted: hand the same derivation the
    /// unblanked text and the `ArgUtil` assertion fires. Nothing else about
    /// the run changes, which is the point -- an oracle comparison over this
    /// wrong member set would still report zero failures.
    #[test]
    #[should_panic(expected = "ArgUtil is in the derived edge set")]
    fn skipping_the_comment_blanking_fires_the_argutil_assertion() {
        edges_from_members(&chi_members(SAMPLE), &["RexxInfo"]);
    }

    #[test]
    fn a_line_number_is_the_line_in_the_whole_file() {
        assert_eq!(derived()[0].line, 5);
    }
}
