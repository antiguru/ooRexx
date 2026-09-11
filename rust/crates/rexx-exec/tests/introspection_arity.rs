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

//! `corpus/introspection-arity.tsv` (Phase 5i Task 0): what each documented
//! introspection method does when it is sent an argument list it could
//! accept.

mod support;

use support::arity;

const HEADER: &str = "\
# What each documented introspection method does when sent an argument list it
# could accept (Phase 5i Task 0). Derived by
# crates/rexx-exec/tests/introspection_arity.rs; edit that, never this.
#
# class<TAB>method<TAB>arm<TAB>verdict<TAB>evidence
#
# THIS TABLE REPLACES corpus/method-bodies.txt's verdict column for these
# classes. That column sends no arguments, so for a method that needs them it
# records agreement about an arity error. No task in Phase 5i may cite it as
# a reason a row needs no work.
#
#   agree          the oracle and the crate give identical descriptors,
#                  the answered value among them
#   send-differs   the receiver was built on both sides and the send differs
#   setup-differs  one side could not build the receiver, so the row says
#                  nothing about its own method
#   exempt         a committed reason instead of an argument list
#   no-value       the send completed the same way on both sides and returned
#                  no result, so nothing beyond the three descriptors was
#                  compared. It is deliberately not `agree`: a reader sizing
#                  work from it learns that the send is reachable and nothing
#                  about what the method answers.
#   unstable       marked UNSTABLE: -- the oracle does not reproduce its own
#                  answer between two runs, so no value comparison could say
#                  anything. The marker is policed by running the oracle
#                  twice, not by this line.
#
# THE ANSWERED VALUE IS COMPARED HERE, unlike corpus/collection-arity.tsv:
# the probe assigns the send's result and prints `vv~string`, so a row both
# sides complete with different answers reads send-differs rather than agree.
# `Class~enhanced` is the row it exists for: the oracle answers `enhanced K`
# at rc 0, so a crate that completes the send and answers something else was
# invisible to an instrument reading only the exit status and the streams. A
# task closing a row still owes it a witness that reads the answer's own
# contents, which `~string` does not.
#
# The argument lists are corpus/introspection-arguments.tsv and the receivers
# are corpus/introspection-receivers.tsv. A list is only real if the ORACLE
# completes the send; the test makes an oracle run that does not a failure for
# that row.
#
# Pointer's and Buffer's INSTANCE arm is deliberately absent: class-set.txt
# gives them no construction expression, because the reference says instances
# come only from native code (utilityclasses.xml:429, :6910), so every
# instance row of theirs would be setup-differs on the oracle's own side.
# Their class arm is here, marked REFUSED: in the argument file -- the oracle
# answers 93.967 and that refusal is what the row measures.
";

fn layout() -> arity::Layout {
    arity::Layout {
        receivers: "introspection-receivers.tsv",
        arguments: "introspection-arguments.tsv",
        scopes: "introspection-scopes.tsv",
        table: "introspection-arity.tsv",
        refresh_env: "REXX_INTROSPECTION_ARITY_REFRESH",
        header: HEADER,
        arm_column: true,
        compare_values: true,
        fixture: true,
        probe_prefix: "rexx-introspection-arity",
    }
}

/// The committed table is what the three sides say today.
#[test]
fn the_table_matches_the_three_sides() {
    let moved = arity::table_disagreements(&layout());
    assert!(
        moved.is_empty(),
        "corpus/introspection-arity.tsv disagrees with the interpreters. A row that moved \
         because a task implemented something is a refresh; a row that moved otherwise is the \
         finding.\n{moved:#?}"
    );
}

/// **The harness rule.** A row whose send the oracle does not complete is a
/// failure of this instrument for that row, never a data point.
#[test]
fn the_oracle_completes_every_send() {
    let unsent = arity::unsent_rows(&layout());
    assert!(
        unsent.is_empty(),
        "these argument lists are not real -- the oracle does not complete the send, so the \
         row's verdict is about the list and not about the method. Fix the list in \
         corpus/introspection-arguments.tsv, or exempt it with a reason.\n{unsent:#?}"
    );
}

/// A row marked `REFUSED:` is one the oracle really does refuse.
#[test]
fn every_refused_row_is_really_refused() {
    let completed = arity::refusals_the_oracle_completes(&layout());
    assert!(completed.is_empty(), "{completed:#?}");
}

/// A row marked `UNSTABLE:` is one the oracle really does not reproduce.
#[test]
fn every_unstable_row_is_really_unstable() {
    let stable = arity::stable_rows_marked_unstable(&layout());
    assert!(stable.is_empty(), "{stable:#?}");
}

/// Every documented row has a list, and a native row whose upstream arity is
/// not zero is sent something.
#[test]
fn every_row_is_sent_something_its_arity_needs() {
    let wrong = arity::rows_missing_arguments(&layout());
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// An exemption carries its reason, so declining a row is a sentence and not
/// a blank.
#[test]
fn every_exemption_says_why() {
    let blank = arity::exemptions_without_a_reason(&layout());
    assert!(blank.is_empty(), "{blank:#?}");
}
