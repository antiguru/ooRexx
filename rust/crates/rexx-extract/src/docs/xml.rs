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

//! Text-level DocBook scanning, shared by every `oodocs/` extractor.
//!
//! **Text-level rather than parsed, and the choice is forced.** `provide.xml`
//! uses `apos`, `mdash`, `nbsp` and `quot`, none of which `rexxref.ent`
//! defines; they come only from the external DocBook DTD the DOCTYPE names by
//! `http` URL, and `oodocs/` is a read-only checkout with neither the DTD nor
//! a catalog. A local-entity-only parse dies at `&mdash;` on `provide.xml:59`,
//! long before the first `&nbsp;` at `:849`. So an entity reference reaching
//! these extractors as literal source text is the contract, not an accident:
//! [`split_revision_marker`] strips the four revision entities off a title and
//! `hierarchy`'s indentation rule counts literal `&nbsp;`.
//!
//! **Comments are blanked, not deleted.** Every citation these extractors emit
//! is a `file:line`, so a transform that removed bytes would renumber the
//! document it is citing. [`blank_comments`] overwrites a comment's bytes with
//! spaces and leaves its newlines, so offsets and line numbers are unchanged
//! and a scan of the result cannot see inside a comment.

/// Every byte of every `<!-- ... -->` replaced by a space, newlines kept.
///
/// A comment with no closing marker blanks the rest of the file; that is a
/// malformed document rather than a case to absorb, and it shows up as a
/// collapsed row set rather than as a silently wrong one.
pub fn blank_comments(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"<!--") {
            let end = find_from(src, i + 4, "-->").map_or(bytes.len(), |at| at + 3);
            for &b in &bytes[i..end] {
                out.push(if b == b'\n' { '\n' } else { ' ' });
            }
            i = end;
        } else {
            // Byte-at-a-time is safe here only because the branch above never
            // splits a multi-byte character: `<!--` and `-->` are ASCII, and a
            // UTF-8 continuation byte can never equal one of their bytes.
            let ch = src[i..].chars().next().unwrap_or('\u{fffd}');
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn find_from(src: &str, from: usize, needle: &str) -> Option<usize> {
    src.get(from..)
        .and_then(|tail| tail.find(needle))
        .map(|at| at + from)
}

/// One `<section>` element, with the span a caller needs to key rows by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// The `id` attribute, absent for `utilityclasses.xml:4809`'s
    /// `<section><title>Examples</title>` and for any sibling of that shape.
    pub id: Option<String>,
    /// The section's own `<title>`, cut at the first nested markup and
    /// trimmed: a `mth*` title may carry an `<indexterm>` inside the title
    /// element. Empty when the open tag is not followed by one.
    pub title: String,
    /// Whether only whitespace separates the open tag from its `<title>`.
    /// False also when the section has no title of its own.
    pub title_is_immediate: bool,
    /// Nesting depth within the file, 0 for a section with no section parent.
    pub depth: usize,
    /// `id` of the innermost enclosing section that has one.
    pub parent_id: Option<String>,
    /// 1-based line of the `<section` open tag.
    pub open_line: usize,
    /// 1-based line of the matching `</section>`.
    pub close_line: usize,
    /// Byte offset just past the open tag's `>`.
    pub body_start: usize,
    /// Byte offset of the matching `</section>`.
    pub body_end: usize,
}

impl Section {
    /// The part of this section that precedes its first nested `<section`, or
    /// the whole body when it nests none.
    ///
    /// This is a class section's own prose and its generated class table --
    /// the span the method-row extractor keys a class's methods by, and the
    /// span the reading ledger's four-books row was read to.
    pub fn head<'a>(&self, text: &'a str) -> &'a str {
        let body = &text[self.body_start..self.body_end];
        match body.find("<section") {
            Some(at) => &body[..at],
            None => body,
        }
    }
}

/// Every `<section>` in `text`, in document order.
///
/// `text` must already have been through [`blank_comments`]: a commented-out
/// `<section` would otherwise open a span that never closes.
///
/// Panics on an unbalanced `</section>`, because a scan that absorbed one
/// would report a plausible tree for a document it had misread.
pub fn sections(text: &str) -> Vec<Section> {
    let mut open: Vec<usize> = Vec::new();
    let mut out: Vec<Section> = Vec::new();
    let mut i = 0usize;
    loop {
        let next_open = find_open_tag(text, i);
        let next_close = find_from(text, i, "</section>");
        match (next_open, next_close) {
            (Some(at), close) if close.is_none_or(|c| at < c) => {
                let tag_end = find_from(text, at, ">")
                    .unwrap_or_else(|| panic!("unterminated <section tag at byte {at}"));
                let parent_id = open.iter().rev().find_map(|&ix| out[ix].id.clone());
                let section = Section {
                    id: attribute(&text[at..tag_end], "id"),
                    title: String::new(),
                    title_is_immediate: false,
                    depth: open.len(),
                    parent_id,
                    open_line: line_of(text, at),
                    close_line: 0,
                    body_start: tag_end + 1,
                    body_end: 0,
                };
                out.push(section);
                open.push(out.len() - 1);
                i = tag_end + 1;
            }
            (_, Some(c)) => {
                let ix = open.pop().unwrap_or_else(|| {
                    panic!("</section> at line {} closes nothing", line_of(text, c))
                });
                out[ix].body_end = c;
                out[ix].close_line = line_of(text, c);
                i = c + "</section>".len();
            }
            (None, None) => break,
            // Unreachable: the guard on the first arm only declines when
            // `next_close` is `Some` and earlier, which the second arm takes.
            (Some(_), None) => unreachable!("an open tag with no close was not taken above"),
        }
    }
    assert!(
        open.is_empty(),
        "{} <section> elements never close",
        open.len()
    );
    for section in &mut out {
        let (title, immediate) = own_title(text, section.body_start, section.body_end);
        section.title = title;
        section.title_is_immediate = immediate;
    }
    out
}

/// The next `<section` that opens an element -- the name has to end there, or
/// a hypothetical `<sectioninfo>` would open a span that never closes.
fn find_open_tag(text: &str, from: usize) -> Option<usize> {
    let mut i = from;
    loop {
        let at = find_from(text, i, "<section")?;
        let after = text.as_bytes().get(at + "<section".len()).copied();
        match after {
            Some(b) if b.is_ascii_whitespace() || b == b'>' || b == b'/' => return Some(at),
            _ => i = at + "<section".len(),
        }
    }
}

/// The section's own `<title>`, which has to be the first element in its body:
/// searching further would answer a nested section's title, or a `<table>`'s,
/// for a section that has none of its own.
fn own_title(text: &str, from: usize, to: usize) -> (String, bool) {
    let body = &text[from..to];
    let after_space = body.trim_start();
    let Some(rest) = after_space.strip_prefix("<title>") else {
        return (String::new(), false);
    };
    let Some(end) = rest.find("</title>") else {
        return (String::new(), false);
    };
    let text_only = rest[..end].split('<').next().unwrap_or("");
    (text_only.trim().to_string(), true)
}

/// The value of `name="..."` in an open tag, single or double quoted.
pub fn attribute(tag: &str, name: &str) -> Option<String> {
    let mut rest = tag;
    loop {
        let at = rest.find(name)?;
        let before_ok = at > 0 && rest.as_bytes()[at - 1].is_ascii_whitespace();
        let after = &rest[at + name.len()..];
        let after_trim = after.trim_start();
        if before_ok && after_trim.starts_with('=') {
            let value = after_trim[1..].trim_start();
            let quote = value.chars().next()?;
            if quote == '"' || quote == '\'' {
                let end = value[1..].find(quote)?;
                return Some(value[1..1 + end].to_string());
            }
        }
        rest = &rest[at + name.len()..];
    }
}

/// The 1-based line number of byte offset `at`.
pub fn line_of(text: &str, at: usize) -> usize {
    text[..at].bytes().filter(|&b| b == b'\n').count() + 1
}

/// One `<member>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// 1-based line of the `<member>` open tag.
    pub line: usize,
    /// Everything between `<member>` and `</member>`, verbatim.
    pub inner: String,
    /// The first `<xref linkend="...">`'s target, if the member has one.
    pub linkend: Option<String>,
    /// That same `<xref>`'s `xrefstyle`, which decides what the book
    /// **displays** for the link. `select:title` renders the target section's
    /// own `<title>`; `template:<text>` replaces it with `<text>` outright, so
    /// an extractor that reads only the target's title reads a name the book
    /// does not show.
    pub xrefstyle: Option<String>,
    /// The text after that `<xref .../>`, trimmed. This is where the class
    /// tables spell out the operator methods a group heading documents --
    /// `mthObjectComparisonMethods` is followed by `= == &lt;> >&lt; \= \==`.
    pub trailing: String,
}

/// Every `<member>` in `text`, in document order.
pub fn members(text: &str) -> Vec<Member> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while let Some(at) = find_from(text, i, "<member>") {
        let from = at + "<member>".len();
        let Some(end) = find_from(text, from, "</member>") else {
            break;
        };
        let inner = &text[from..end];
        let (linkend, xrefstyle, trailing) = match inner.find("<xref ") {
            Some(x) => {
                let tag_end = inner[x..]
                    .find("/>")
                    .map(|e| x + e + 2)
                    .unwrap_or(inner.len());
                let tag = &inner[x..tag_end];
                (
                    attribute(tag, "linkend"),
                    attribute(tag, "xrefstyle"),
                    inner[tag_end..].trim().to_string(),
                )
            }
            None => (None, None, String::new()),
        };
        out.push(Member {
            line: line_of(text, at),
            inner: inner.to_string(),
            linkend,
            xrefstyle,
            trailing,
        });
        i = end + "</member>".len();
    }
    out
}

/// Every `xi:include`'s `href`, with the 1-based line it sits on.
pub fn xincludes(text: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while let Some(at) = find_from(text, i, "<xi:include") {
        let tag_end = find_from(text, at, ">").unwrap_or(text.len());
        if let Some(href) = attribute(&text[at..tag_end], "href") {
            out.push((href, line_of(text, at)));
        }
        i = tag_end + 1;
    }
    out
}

/// The four revision-marker entities `rexxref.ent` defines as the empty
/// string, split off the front of a title.
///
/// A text-level extractor sees them verbatim, glued to the name with no
/// space -- `&added50;size`, `&changed50;send`. Deriving `&ADDED50;SIZE` gives
/// a name the oracle has on neither arm.
pub fn split_revision_marker(title: &str) -> (Option<&str>, &str) {
    const MARKERS: &[&str] = &["&added50;", "&added51;", "&added52;", "&changed50;"];
    for marker in MARKERS {
        if let Some(rest) = title.strip_prefix(marker) {
            return (Some(marker), rest);
        }
    }
    (None, title)
}

/// The five predefined XML entities, resolved. Nothing else is touched --
/// a DocBook entity such as `&nbsp;` has no local definition and stays
/// verbatim, which is what the hierarchy extractor counts.
pub fn decode_predefined(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_comment_is_blanked_in_place_so_line_numbers_survive() {
        let src = "a\n<!-- two\nthree -->\nb\n";
        let out = blank_comments(src);
        assert_eq!(out.len(), src.len());
        assert_eq!(out.lines().count(), src.lines().count());
        assert_eq!(out.lines().nth(1), Some("        "));
        assert_eq!(out.lines().nth(3), Some("b"));
    }

    #[test]
    fn a_section_inside_a_comment_is_invisible() {
        let src = "<section id=\"a\"><title>A</title>\n<!--\n<section id=\"b\"><title>B</title></section>\n-->\n</section>";
        let out = sections(&blank_comments(src));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].id.as_deref(), Some("a"));
    }

    #[test]
    fn nesting_gives_depth_and_a_parent() {
        let src = "<section id=\"p\"><title>P</title>\n<section id=\"c\"><title>C</title></section>\n</section>";
        let out = sections(src);
        assert_eq!(out.len(), 2);
        assert_eq!((out[0].depth, out[0].parent_id.as_deref()), (0, None));
        assert_eq!((out[1].depth, out[1].parent_id.as_deref()), (1, Some("p")));
        assert_eq!(out[1].open_line, 2);
    }

    #[test]
    fn a_title_carrying_markup_is_cut_at_the_first_tag() {
        let src = "<section id=\"m\"><title>&added50;size\n<indexterm><primary>size</primary></indexterm>\n</title></section>";
        let out = sections(src);
        assert_eq!(out[0].title, "&added50;size");
        assert!(out[0].title_is_immediate);
        assert_eq!(
            split_revision_marker(&out[0].title),
            (Some("&added50;"), "size")
        );
    }

    #[test]
    fn a_head_stops_at_the_first_nested_section() {
        let src = "<section id=\"p\"><title>P</title>OWN<section id=\"c\"><title>C</title>NESTED</section></section>";
        let out = sections(src);
        assert!(out[0].head(src).contains("OWN"));
        assert!(!out[0].head(src).contains("NESTED"));
    }

    #[test]
    fn a_member_keeps_its_trailing_operator_text() {
        let src = r#"<member><xref linkend="mthObjectComparisonMethods" xrefstyle="select:title"/> = == &lt;> >&lt; \= \==</member>"#;
        let out = members(src);
        assert_eq!(
            out[0].linkend.as_deref(),
            Some("mthObjectComparisonMethods")
        );
        assert_eq!(out[0].trailing, r"= == &lt;> >&lt; \= \==");
        assert_eq!(decode_predefined(&out[0].trailing), r"= == <> >< \= \==");
        assert_eq!(out[0].xrefstyle.as_deref(), Some("select:title"));
    }

    /// The attribute that decides what the book displays for the link, read
    /// off the same `<xref>` as the target.
    #[test]
    fn a_member_keeps_the_xrefstyle_that_overrides_the_displayed_name() {
        let src = r#"<member><xref linkend="mthDateTimeInit" xrefstyle="template:new (Inherited Class Method)"/></member>"#;
        let out = members(src);
        assert_eq!(out[0].linkend.as_deref(), Some("mthDateTimeInit"));
        assert_eq!(
            out[0].xrefstyle.as_deref(),
            Some("template:new (Inherited Class Method)")
        );
    }

    #[test]
    fn an_attribute_is_found_whatever_its_position_or_quote() {
        assert_eq!(
            attribute(r#"<section revisionflag="added" id='x'"#, "id"),
            Some("x".into())
        );
        // `linkend` must not be answered by `xreflinkend`, and `id` must not
        // be answered by the `id` inside `linkend`.
        assert_eq!(attribute(r#"<xref linkend="clsA"/>"#, "id"), None);
    }
}
