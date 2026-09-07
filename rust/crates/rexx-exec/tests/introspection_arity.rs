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
//!
//! The probe, the verdicts and the harness rule are `support::arity`; this
//! file is the four files, the header and the refresh variable.
//!
//! # Why this exists at all
//!
//! Before it, every class in Phase 5i had no arity row anywhere, and the only
//! instrument that had run them was `corpus/method-bodies.txt` -- whose own
//! header says a zero-argument send to a method needing arguments records
//! agreement about an *arity error*. **No task in this phase may cite that
//! column as a reason a row needs no work.** This table is what it reads
//! instead.
//!
//! # What it carries that `collection-arity.tsv` does not
//!
//! An `arm` column. Five of this phase's own `loud` rows are class methods --
//! `Package~defaultOptions`, `Method~loadExternalMethod`, `Method~newFile`,
//! `Routine~loadExternalRoutine`, `Routine~newFile` -- so an instance-only
//! table would leave them unsized.
//!
//! Refresh with
//!   `REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec \
//!        --test introspection_arity`

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
#   agree          the oracle and both engines give identical descriptors
#   send-differs   the receiver was built on both sides and the send differs
#   setup-differs  one side could not build the receiver, so the row says
#                  nothing about its own method
#   engine-differs the two engines disagree with each other
#   exempt         a committed reason instead of an argument list
#
# `agree` SAYS THE SEND COMPLETED THE SAME WAY, NOT THAT THE TWO SIDES
# ANSWERED THE SAME VALUE: the probe sends a bare statement and never prints
# the result. A task closing a row still owes it a witness that reads what it
# answered.
#
# The argument lists are corpus/introspection-arguments.tsv and the receivers
# are corpus/introspection-receivers.tsv. A list is only real if the ORACLE
# completes the send; the test makes an oracle run that does not a failure for
# that row.
#
# Pointer and Buffer are deliberately absent: class-set.txt gives them no
# construction expression, because the reference says instances come only from
# native code (utilityclasses.xml:429, :6910), so every instance row of theirs
# would be setup-differs on the oracle's own side.
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

/// Every documented row has a list, and a native row whose upstream arity is
/// not zero is sent something.
///
/// This is what defeats "fill in the two control rows and leave the rest
/// empty": an empty list on a method that takes arguments is caught here
/// rather than read as agreement.
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

/// The two engines never disagree with each other.
#[test]
fn the_engines_agree_with_one_another() {
    let split = arity::engine_splits(&layout());
    assert!(
        split.is_empty(),
        "the compiled and tree-walking engines answer differently: {split:#?}"
    );
}
