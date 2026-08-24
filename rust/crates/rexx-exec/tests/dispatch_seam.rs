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

//! **D45, site one: dispatch passes through exactly one chokepoint.**
//!
//! # The half the compiler enforces
//!
//! `dispatch::seam::Cleared` is a struct with a private field, and both
//! things a resolved method can be take one by value: `dispatch::NativeMethod`
//! is the signature of a primitive method, and `Interp::enter_method_body` is
//! the one function that runs a `::METHOD` directive's Rexx body. It is
//! neither `Copy` nor `Clone` and has no other constructor. So **neither kind
//! of method can be called at all without a value produced inside `mod
//! seam`** -- measured, not argued: writing `native_length(interp,
//! Cleared(()), ..)` anywhere else in the crate is `error[E0423]: cannot
//! initialize a tuple struct which contains private fields`.
//!
//! That is the whole of the compiler's contribution, and it bounds *whether*
//! the seam is reachable around, not *how many* producers there are.
//!
//! # The half this test enforces, and how it is evadable
//!
//! Everything below is a lexical scan, and the previous version of this
//! comment claimed more: it said "the compiler refuses every other
//! spelling", which is false. A second producer written **inside** `mod
//! seam` -- the one place the private field permits -- evades a needle
//! search entirely:
//!
//! ```ignore
//! pub(super) fn clear_for(..) -> Cleared { let ok = (); Cleared(ok) }
//! ```
//!
//! `Cleared(ok)` is not the text `Cleared(())`, and `seam::clear_for(` does
//! not contain `seam::clear(`. So the producer count is bounded by reading
//! the module rather than by counting two tokens, which is what
//! [`the_seam_module_holds_one_struct_and_one_function`] does: the module
//! body is extracted by brace matching and its item keywords are counted, so
//! **any** second producer -- a second `fn`, an `impl` block, a `const` of
//! that type -- is a second item and fails.
//!
//! **What it counts is two literal token spellings and the items inside one
//! brace-matched region of one file** -- not calls, not producers, not paths.
//! What that leaves it unable to see, stated rather than argued away:
//!
//! * **An import alias**, which is the cheapest evasion of the lot and was
//!   missing from this list until Phase 5a Task 6's review found it at the
//!   directory seam. `use seam::{clear as sneak_clear};` plus a second
//!   invocation path calling `sneak_clear` compiles, adds no item to `mod
//!   seam`, and matches neither needle -- so every count here is satisfied
//!   while two paths reach a native method. Verified at the directory seam,
//!   whose module is shaped the same way; nothing here defends against it.
//!
//! * **An item introduced by a macro expansion inside the module**, or a
//!   second `mod seam` in another file. Neither exists; both would pass.
//! * **A `clear` that hands out more than one clearance per call** -- one
//!   returning a tuple of them, or a `Vec<Cleared>`. That keeps one struct,
//!   one function and one call site, so every count here is satisfied while
//!   two invocation paths are fed from a single trip through the seam. The
//!   token's not being `Copy` or `Clone` does not reach it: the producer is
//!   free to build as many as it likes. Nothing defends against this and
//!   nothing is going to; it is here because the list is what the honest
//!   answer to "what could pass this" consists of.
//! * **A path that invokes a third kind of method.** This entry used to read
//!   "something other than a primitive method", against the day Rexx method
//!   bodies became invocable; they now are, and that half is closed rather
//!   than deferred: `Interp::enter_method_body` takes a `Cleared` by value
//!   just as a `NativeMethod` does, so the compiler binds it too, and
//!   [`the_seam_token_is_named_only_by_the_dispatch_module`] pins that every
//!   function which can take one is written in the file this test reads. What
//!   is **not** closed is a *third* invocable kind added later with no
//!   `Cleared` parameter at all: nothing here can require a signature that
//!   does not exist yet, and the same sentence will be true of the fourth.
//! * **A path in another crate.** The scan reads `rexx-exec/src` only.
//!   Nothing outside this crate can call `Interp::invoke` (it is
//!   `pub(crate)`) or build a `Cleared`, so a second path elsewhere would
//!   have to reimplement the object model rather than reuse it -- which R9
//!   already forbids for a different reason, and which this test does not
//!   check.
//! * **A mention inside a comment or a string** counts toward the needle
//!   tallies, so a stray one fails rather than passes -- the safe direction,
//!   and the reason this file builds its needles with `format!` rather than
//!   spelling them.
//!
//! The scan reads `src/` and never this file, so nothing here can satisfy its
//! own assertion.
//!
//! # What passes through the seam, and what does not
//!
//! `PROTECTED` is the one access scope the oracle asks a security manager
//! about, so it is the one the seam decides:
//! [`the_protected_question_is_asked_only_inside_the_seam`] pins that the
//! function answering "is this method `PROTECTED`" is called from the seam
//! and from nowhere else. With no manager installed the answer is always
//! permission, so **no program can tell that branch from its absence** and
//! the lexical assertion is the whole instrument.
//!
//! `PRIVATE` and `PACKAGE` are decided one step earlier, in
//! `Interp::resolve`, and that is measured rather than chosen: the oracle
//! refuses the *lookup* and falls through to the receiver's own `UNKNOWN`, so
//! a check at the seam would refuse sends the oracle answers. Nothing here
//! bounds where those two are asked; `dispatch.rs`'s own tests are what pin
//! their behaviour.

use std::fs;
use std::path::{Path, PathBuf};

/// Every `.rs` file under `crates/rexx-exec/src/`, recursively.
fn source_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        for entry in entries {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    walk(&src, &mut out);
    out.sort();
    assert!(
        !out.is_empty(),
        "the scan found no source files at all, so its counts would be a \
         vacuous zero rather than a measurement"
    );
    out
}

/// Every occurrence of `needle` across the crate's own sources, as
/// `(path, line number)` pairs so a failure names where the extra one is.
fn occurrences(needle: &str) -> Vec<String> {
    let mut found = Vec::new();
    for path in source_files() {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for (index, line) in text.lines().enumerate() {
            for _ in line.matches(needle) {
                found.push(format!("{}:{}", path.display(), index + 1));
            }
        }
    }
    found
}

/// Every occurrence of `needle` on a line that is not a whole-line comment,
/// as `(path, line number)` pairs.
///
/// The comment stripping is the same rule [`seam_module_body`] applies for
/// the same reason, one scope wider: `Cleared` is an ordinary English word
/// and appears in two of `run.rs`'s doc comments, neither of which can name
/// a type.
fn code_occurrences(needle: &str) -> Vec<String> {
    let mut found = Vec::new();
    for path in source_files() {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for _ in line.matches(needle) {
                found.push(format!("{}:{}", path.display(), index + 1));
            }
        }
    }
    found
}

/// Every collection in this crate goes through `Interp::collect_now`, which
/// is where the root set is completed before `Heap::collect` is handed it.
///
/// **The root set the collector receives is not `Interp::roots` alone.** An
/// activation's context object is named by nothing in that structure while
/// the activation is running or suspended; `collect_now` sweeps the
/// activation stack for those objects and pushes them as temporaries around
/// the collect. A collection reached by any other route therefore frees a
/// live object: a build whose `GC('Force')` reaches `Heap::collect` directly
/// refuses the next send to `.CONTEXT` at rc 120 against the oracle's rc 0.
///
/// **The corpus does see that particular door.** `class_context_gc.rex` is
/// red against such a build and `class_context_reply.rex` covers the parked
/// route beside it. What no corpus row can see is a door nothing in the
/// corpus reaches, and the door that produced those two rows was found by
/// hand rather than by a gate.
///
/// **This is a lexical assertion because the alternative is a paragraph.**
/// Nothing in the type system stops a new caller of `Heap::collect`, and the
/// rooting hangs off the *call site* rather than off any value a compiler can
/// track. It is the same argument this file's module doc makes for the seam
/// and `rexx-core/tests/unsafe_sites.rs` makes for `unsafe`: where the
/// property is "this is the only place that does X", the text is the only
/// thing there is to check.
///
/// `Heap::collect` is the whole question rather than a proxy for it: the only
/// writes of `Slot::Free` in the workspace are inside that function, so
/// "reaches a collection" and "calls `Heap::collect`" name one set.
///
/// # What this leaves it unable to see
///
/// Stated rather than argued away, in the shape this file's module doc uses.
/// The scan matches a spelling, so a call spelled differently passes it green
/// with the sweep bypassed:
///
/// * **UFCS.** `Heap::collect(&mut self.heap, &self.roots)` names the type
///   rather than the field and matches nothing here.
/// * **A rebinding.** `let h = &mut self.heap;` then `h.collect(..)` moves
///   the receiver's name out of the call.
/// * **`rexx-core`'s own code.** The walker reads
///   `crates/rexx-exec/src/` alone, so a collection introduced inside the
///   crate that defines `Heap` is invisible to it.
///
/// The needle is `heap.collect(` over whitespace-collapsed text rather than
/// `heap.collect(&` per line so that these do **not** escape either: a call
/// whose argument is already a reference (`self.heap.collect(roots)`) and one
/// `rustfmt` wraps across lines both still match.
#[test]
fn heap_collect_is_called_from_collect_now_alone() {
    // Built rather than written literally, so this file's own text does not
    // contribute to the count it takes.
    let call = format!("{}.{}(", "heap", "collect");

    // Whitespace-collapsed, so a wrapped call is still one match. Comment
    // lines go first, so that a comment cannot satisfy the assertion.
    let mut sites = Vec::new();
    for path in source_files() {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let code: String = text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join(" ");
        let collapsed: String = code.split_whitespace().collect::<Vec<_>>().join("");
        for _ in collapsed.matches(&call) {
            sites.push(path.display().to_string());
        }
    }
    assert_eq!(
        sites.len(),
        1,
        "every collection must go through `Interp::collect_now`, which is \
         what completes the root set; {call} appears in {sites:?}"
    );
    assert!(
        sites[0].contains("lib.rs"),
        "the one collection site is outside `lib.rs`, in {:?} -- if \
         `collect_now` moved there too this assertion needs updating, and if \
         it did not, the sweep is being bypassed",
        sites[0]
    );

    // Inside `collect_now` and not merely somewhere in `lib.rs`, which is
    // what makes this stronger than the file check above: the body is read
    // from the signature to the first closing brace at the same indentation.
    let lib = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs"),
    )
    .expect("lib.rs is readable");
    let signature = format!("    fn {}_{}(&mut self) {{", "collect", "now");
    let start = lib
        .find(&signature)
        .unwrap_or_else(|| panic!("no function matching {signature:?}"));
    let body = &lib[start..];
    let end = body
        .find("\n    }\n")
        .expect("the function has a closing brace at its own indentation");
    assert!(
        body[..end].contains(&call),
        "the one `Heap::collect` call in this crate is in `lib.rs` but not \
         inside `collect_now`, so it does not get the swept root set"
    );
}

/// The whole of D45's site-one claim: one call to the seam, and one way to
/// build the token it hands out.
#[test]
fn dispatch_passes_through_exactly_one_chokepoint() {
    // Built rather than written literally, so this file's own text does not
    // contribute to the counts it takes.
    let call = format!("{}::{}(", "seam", "clear");
    let construct = format!("{}(())", "Cleared");

    let calls = occurrences(&call);
    assert_eq!(
        calls.len(),
        1,
        "dispatch must pass through exactly one chokepoint; {call} is called at {calls:?}"
    );

    // The tuple struct's own declaration reads the same as a construction of
    // it, so the two are counted apart: one declaration, and one thing that
    // is not the declaration.
    let declaration = format!("struct {construct}");
    let declarations = occurrences(&declaration);
    assert_eq!(
        declarations.len(),
        1,
        "the seam's token must be declared exactly once; {declaration} appears \
         at {declarations:?}"
    );
    let mentions = occurrences(&construct);
    assert_eq!(
        mentions.len(),
        2,
        "the seam's token must have exactly one construction site beside its \
         one declaration, or counting calls to the seam stops bounding the \
         paths that reach a native method; {construct} appears at {mentions:?}"
    );
}

/// **The `PROTECTED` question is asked inside the seam and nowhere else.**
///
/// The oracle routes a protected method through `processProtectedMethod`
/// (`classes/ObjectClass.cpp:886`-`:889`) and every other method straight to
/// `method->run`, so the manager is asked at one point in the send. This
/// asserts the same of this crate: one code mention of the predicate besides
/// its own definition, and both in the file holding the seam.
///
/// **What it cannot see**, since with no manager the branch cannot refuse and
/// no program can observe it at all: that the call is inside `seam::clear`
/// rather than merely inside `dispatch.rs`, that the manager's own function
/// is reached only from there, or that a later invocation path added without
/// a `Cleared` skips it -- the module doc's list already owns that last one.
/// Had the predicate been called from `Interp::invoke` beside the seam
/// instead of inside it, this test would pass.
#[test]
fn the_protected_question_is_asked_only_inside_the_seam() {
    let predicate = format!("{}_is_{}(", "method", "protected");
    let sites = code_occurrences(&predicate);
    assert_eq!(
        sites.len(),
        2,
        "the protected predicate must have exactly one caller beside its own \
         definition; {predicate} appears at {sites:?}"
    );
    let elsewhere: Vec<&String> = sites
        .iter()
        .filter(|site| !site.contains("dispatch.rs"))
        .collect();
    assert!(
        elsewhere.is_empty(),
        "the protected predicate is named outside `dispatch.rs`, at {elsewhere:?}, \
         so the seam is no longer the only place the manager is asked"
    );
}

/// **Every consumer of the seam's token is written in `dispatch.rs`**, which
/// is what lets the item read below bound the producers *and* the file above
/// bound the consumers.
///
/// The token's type is `pub(super)` inside `mod seam`, so `dispatch.rs` is
/// the widest scope that can name it -- but "widest scope" is a fact about
/// the module tree, and this asserts the fact about the tree as it stands:
/// no other file in the crate names `Cleared` in code. (`run.rs` uses the
/// word in prose, which is why whole-line comments are stripped.) A second
/// invocation path funded by a clearance would therefore have to be written
/// alongside the ones that exist, in the one file this test already reads.
///
/// It does not, and cannot, stop a path that takes **no** clearance; the
/// module doc's own list says so.
#[test]
fn the_seam_token_is_named_only_by_the_dispatch_module() {
    let token = format!("{}{}", "Clea", "red");
    let sites = code_occurrences(&token);
    let elsewhere: Vec<&String> = sites
        .iter()
        .filter(|site| !site.contains("dispatch.rs"))
        .collect();
    assert!(
        elsewhere.is_empty(),
        "the seam's token is named in code outside `dispatch.rs`, at {elsewhere:?}, \
         so the functions that can consume a clearance are no longer bounded by \
         reading that one file"
    );
    assert!(
        sites.len() >= 2,
        "the scan found fewer than two code mentions of the token inside \
         dispatch.rs, so the filter above is passing by finding nothing"
    );
}

/// The negative control for the scan itself: a token this crate does not
/// contain is found nowhere, and one it plainly does contain is found.
///
/// Without this, a `source_files` that silently read the wrong directory, or
/// an `occurrences` that never matched anything, would make the assertions
/// above pass by finding nothing at all -- and "exactly one" would then be
/// the one thing it could not report.
#[test]
fn the_scan_can_tell_a_present_token_from_an_absent_one() {
    assert!(
        occurrences("Interp::invoke").len() >= 2,
        "the scan found fewer than two mentions of a name the dispatch module \
         defines and documents, so it is not reading the crate's sources"
    );
    assert!(
        occurrences("no_such_token_exists_in_this_crate").is_empty(),
        "the scan matched a token that is not in the crate, so its counts mean nothing"
    );
}

/// The seam module's own body, by brace matching from its `mod` line.
///
/// Read rather than counted from the outside, because the private field puts
/// every possible producer of the token inside these lines and nowhere else.
fn seam_module_body() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/dispatch.rs");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let header = format!("mod {} {{", "seam");
    let start = text
        .find(&header)
        .unwrap_or_else(|| panic!("{} declares no {header}", path.display()))
        + header.len();
    let mut depth = 1usize;
    for (offset, byte) in text[start..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return text[start..start + offset].to_string();
                }
            }
            _ => {}
        }
    }
    panic!(
        "{}: the seam module's braces do not balance",
        path.display()
    );
}

/// **The producer bound: the seam module holds one struct and one function,
/// and no other item at all.**
///
/// The private field means every producer of the token must be written here;
/// counting the items here therefore counts the producers, where counting two
/// token spellings does not -- the module doc has the evasion that motivated
/// this.
///
/// Had a second producer been added -- `fn clear_for(..) -> Cleared { let ok
/// = (); Cleared(ok) }`, the exact shape the needle tallies miss -- the `fn`
/// count below is 2 and this fails.
#[test]
fn the_seam_module_holds_one_struct_and_one_function() {
    let body = seam_module_body();
    // Comments and doc comments are stripped first, so a keyword inside the
    // module's own prose is not counted as an item.
    let code: String = body
        .lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for (keyword, expected) in [
        ("struct ", 1usize),
        ("fn ", 1),
        ("impl ", 0),
        ("const ", 0),
        ("static ", 0),
        ("mod ", 0),
        ("macro_rules!", 0),
        ("trait ", 0),
        ("union ", 0),
        ("enum ", 0),
    ] {
        assert_eq!(
            code.matches(keyword).count(),
            expected,
            "the seam module holds an unexpected number of `{}` items, so the producers of \
             its token are no longer bounded by reading it:\n{code}",
            keyword.trim()
        );
    }
    // Neither derive can be present: a `Copy` or `Clone` token would let one
    // clearance reach two invocations, which is the same defect the item
    // count is here to bound.
    assert!(
        !code.contains("derive"),
        "the seam token derives something; `Copy` or `Clone` on it would let one clearance \
         serve more than one invocation:\n{code}"
    );
}
