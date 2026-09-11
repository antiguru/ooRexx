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

//! `corpus/collection-arity.tsv` (Phase 5g Task 0b): what each documented
//! collection method does when it is sent an argument list it could accept.
//!
//! The probe, the verdicts and the harness rule are `support::arity`; this
//! file is the four files, the header and the refresh variable.
//!
//! # Why this exists at all
//!
//! `corpus/method-bodies.txt` classifies a row by sending its name with NO
//! arguments, and its own header says what that costs: for a method that
//! needs arguments the two sides agree about an *arity error*. Measured, that
//! is most of these classes -- `Table~at` reads `answers` and
//! `.Table~new; t['k'] = 'v'` refuses at rc 120; `.Bag~new~put('x')` reads
//! `answers` and is rc 168 against the oracle's rc 0. **No task in this phase
//! may cite that column as a reason a row needs no work.** This table is what
//! it reads instead.
//!
//! This table carries no `arm` column and covers the instance arm only.
//! Phase 5i's `corpus/introspection-arity.tsv` carries one; widening this
//! file would move bytes no task asked to move, and its header's sentence
//! about `method-bodies.txt` is scoped to these classes. Value comparison is
//! off here for the same reason, so `agree` says the three descriptors
//! matched and not that the two sides answered the same value; the property
//! is `support::arity`'s.
//!
//! Refresh with
//!   `REXX_COLLECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec \
//!        --test collection_arity`

mod support;

use support::arity;

const HEADER: &str = "\
# What each documented collection method does when sent an argument list it
# could accept (Phase 5g Task 0b). Derived by
# crates/rexx-exec/tests/collection_arity.rs; edit that, never this.
#
# class<TAB>method<TAB>verdict<TAB>evidence
#
# THIS TABLE REPLACES corpus/method-bodies.txt's verdict column for these
# classes. That column sends no arguments, so for a method that needs them it
# records agreement about an arity error. No task in this phase may cite it as
# a reason a row needs no work.
#
#   agree          the oracle and the crate give identical descriptors
#   send-differs   the receiver was built on both sides and the send differs
#   setup-differs  one side could not build the receiver, so the row says
#                  nothing about its own method -- the class has no store
#   exempt         a committed reason instead of an argument list
#
# The argument lists are corpus/collection-arguments.tsv and the receivers are
# corpus/collection-receivers.tsv. A list is only real if the ORACLE completes
# the send; the test makes an oracle run that does not a failure for that row.
";

fn layout() -> arity::Layout {
    arity::Layout {
        receivers: "collection-receivers.tsv",
        arguments: "collection-arguments.tsv",
        scopes: "collection-scopes.tsv",
        table: "collection-arity.tsv",
        refresh_env: "REXX_COLLECTION_ARITY_REFRESH",
        header: HEADER,
        arm_column: false,
        compare_values: false,
        fixture: false,
        probe_prefix: "rexx-collection-arity",
    }
}

/// The committed table is what the three sides say today.
#[test]
fn the_table_matches_the_three_sides() {
    let moved = arity::table_disagreements(&layout());
    assert!(
        moved.is_empty(),
        "corpus/collection-arity.tsv disagrees with the interpreters. A row that moved because \
         a task implemented something is a refresh; a row that moved otherwise is the finding.\n\
         {moved:#?}"
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
         corpus/collection-arguments.tsv, or exempt it with a reason.\n{unsent:#?}"
    );
}

/// Every documented instance row has a list, and a native row whose upstream
/// arity is not zero is sent something.
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
