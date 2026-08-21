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

//! Table C's concept row set: one row per `<section id>` under `provide.xml`'s
//! `provide` chapter, at every nesting level.
//!
//! The chapter is what supplies the concept floor, so the denominator is the
//! chapter's sections and not a hand-made list of the ones anybody thought to
//! name. `unkno` and `reqstr` are rows here for that reason: neither is a
//! directive keyword, so table D cannot see either.

use crate::docs::xml::{Section, blank_comments, sections};

/// One concept row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptSection {
    pub id: String,
    pub depth: usize,
    pub line: usize,
    /// `id` of the enclosing section, empty for a chapter-level section.
    pub parent: String,
    pub title: String,
}

/// Every section of the `provide` chapter, in document order.
///
/// Panics when a section has no `id` or no title of its own: either would
/// produce a row that cannot be keyed or cannot be read, and absorbing one
/// silently is how a denominator shrinks without anybody noticing.
pub fn concept_sections(provide_xml: &str) -> Vec<ConceptSection> {
    let text = blank_comments(provide_xml);
    let chapter = chapter_span(&text, "provide");
    sections(&text)
        .into_iter()
        .filter(|s| s.body_start > chapter.0 && s.body_end < chapter.1)
        .map(|s| row(&s))
        .collect()
}

fn row(s: &Section) -> ConceptSection {
    let id =
        s.id.clone()
            .unwrap_or_else(|| panic!("section at line {} has no id", s.open_line));
    assert!(
        s.title_is_immediate && !s.title.is_empty(),
        "section {id} at line {} has no title of its own",
        s.open_line
    );
    ConceptSection {
        id,
        depth: s.depth,
        line: s.open_line,
        parent: s.parent_id.clone().unwrap_or_default(),
        title: s.title.clone(),
    }
}

/// The byte range strictly inside `<chapter id="...">` ... `</chapter>`.
fn chapter_span(text: &str, id: &str) -> (usize, usize) {
    let open = text
        .find(&format!("<chapter id=\"{id}\""))
        .unwrap_or_else(|| panic!("no <chapter id=\"{id}\"> in this file"));
    let body = text[open..]
        .find('>')
        .map(|at| open + at + 1)
        .unwrap_or(text.len());
    let close = text[body..]
        .find("</chapter>")
        .map(|at| body + at)
        .unwrap_or_else(|| panic!("<chapter id=\"{id}\"> never closes"));
    (body, close)
}

/// `id<TAB>depth<TAB>line<TAB>parent<TAB>title`, one per row.
pub fn rows(sections: &[ConceptSection]) -> Vec<String> {
    sections
        .iter()
        .map(|s| {
            format!(
                "{}\t{}\t{}\t{}\t{}",
                s.id,
                s.depth,
                s.line,
                if s.parent.is_empty() { "-" } else { &s.parent },
                s.title
            )
        })
        .collect()
}

/// The header the committed file carries, with its D56 stamp.
pub fn header(stamp: &str) -> Vec<String> {
    vec![
        "Table C's concept row set: every <section id> under provide.xml's".into(),
        "`provide` chapter, at every nesting level.".into(),
        String::new(),
        "One `id<TAB>depth<TAB>line<TAB>parent<TAB>title` per line, in document".into(),
        "order. `parent` is `-` for a section whose parent is the chapter.".into(),
        String::new(),
        "Derived by `cargo run -p rexx-extract --bin rexx-extract-docs`, whose".into(),
        "module is `src/docs/provide.rs`. Re-derived and compared in both".into(),
        "directions by `tests/extract_docs.rs`: a row that stops being derived".into(),
        "is as red as one that appears.".into(),
        String::new(),
        stamp.to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<chapter id="provide"><title>Objects and Classes</title>
<section id="typcla"><title>Types of Classes</title>
<section id="objcla"><title>Object Classes</title>
</section>
</section>
<!--
<section id="ghost"><title>Never</title></section>
-->
<section id="usesem">
<title>Defining Instance Methods with SETMETHOD or ENHANCED</title>
</section>
</chapter>
<chapter id="other"><title>Other</title>
<section id="elsewhere"><title>Elsewhere</title></section>
</chapter>
"#;

    #[test]
    fn nesting_and_titles_come_out_of_the_chapter_only() {
        let out = concept_sections(SAMPLE);
        let ids: Vec<&str> = out.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, ["typcla", "objcla", "usesem"]);
        assert_eq!(out[1].depth, 1);
        assert_eq!(out[1].parent, "typcla");
        assert_eq!(out[2].depth, 0);
        assert_eq!(out[2].parent, "");
        // A title on its own line is still the section's own title.
        assert_eq!(
            out[2].title,
            "Defining Instance Methods with SETMETHOD or ENHANCED"
        );
    }

    #[test]
    fn a_section_of_another_chapter_is_not_a_row() {
        assert!(concept_sections(SAMPLE).iter().all(|s| s.id != "elsewhere"));
    }
}
