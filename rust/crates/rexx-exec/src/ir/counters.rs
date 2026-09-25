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

//! The driver's test-only counters: per-thread counts of what
//! [`super::drive`] did, which its tests read.

// Test-only instrumentation: how many chunks this thread has driven.
#[cfg(test)]
thread_local! {
    static RUN_CHUNK_ENTRIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn count_run_chunk_entry() {
    if !counting() {
        return;
    }
    RUN_CHUNK_ENTRIES.with(|entries| entries.set(entries.get() + 1));
}

#[cfg(test)]
pub(super) fn run_chunk_entries() -> usize {
    RUN_CHUNK_ENTRIES.with(std::cell::Cell::get)
}

// Whether the counters above and below are recording.
#[cfg(test)]
thread_local! {
    static COUNTING: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

#[cfg(test)]
fn counting() -> bool {
    COUNTING.with(std::cell::Cell::get)
}

/// Stops the counters recording until [`resume_counters`]. `Interp::
/// bootstrap_library` is the only caller and both are `#[cfg(test)]`, so
/// neither exists in a release build.
#[cfg(test)]
pub(crate) fn suspend_counters() {
    COUNTING.with(|on| on.set(false));
}

/// The other half of [`suspend_counters`].
#[cfg(test)]
pub(crate) fn resume_counters() {
    COUNTING.with(|on| on.set(true));
}

// Test-only instrumentation: how many clauses this thread has stepped from a
// compiled stream.
#[cfg(test)]
thread_local! {
    static CLAUSE_OP_ENTRIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn count_clause_op_entry() {
    if !counting() {
        return;
    }
    CLAUSE_OP_ENTRIES.with(|entries| entries.set(entries.get() + 1));
}

#[cfg(test)]
pub(super) fn clause_op_entries() -> usize {
    CLAUSE_OP_ENTRIES.with(std::cell::Cell::get)
}

// Test-only instrumentation: how many clause echoes this thread has emitted
// from a chunk's own `Op::TraceClause`.
#[cfg(test)]
thread_local! {
    static TRACE_OP_ECHOES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn count_trace_op_echo() {
    if !counting() {
        return;
    }
    TRACE_OP_ECHOES.with(|echoes| echoes.set(echoes.get() + 1));
}

#[cfg(test)]
pub(super) fn trace_op_echoes() -> usize {
    TRACE_OP_ECHOES.with(std::cell::Cell::get)
}

// Test-only instrumentation: how many times this thread has skipped the
// small-integer path because a site's hint said it had already fallen through.
#[cfg(test)]
thread_local! {
    static ARITH_HINT_SKIPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn count_arith_hint_skip() {
    if !counting() {
        return;
    }
    ARITH_HINT_SKIPS.with(|skips| skips.set(skips.get() + 1));
}

#[cfg(test)]
pub(super) fn arith_hint_skips() -> usize {
    ARITH_HINT_SKIPS.with(std::cell::Cell::get)
}

// Test-only instrumentation: how many times this thread has run a compiled call
// from the resolution its site had already kept.
#[cfg(test)]
thread_local! {
    static CALL_SITE_HITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn count_call_site_hit() {
    if !counting() {
        return;
    }
    CALL_SITE_HITS.with(|hits| hits.set(hits.get() + 1));
}

#[cfg(test)]
pub(super) fn call_site_hits() -> usize {
    CALL_SITE_HITS.with(std::cell::Cell::get)
}

// Test-only instrumentation: how many times an [`super::Op::Const`] on this
// thread has had to build its value rather than read an interned one.
#[cfg(test)]
thread_local! {
    static CONST_BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn count_const_build() {
    if !counting() {
        return;
    }
    CONST_BUILDS.with(|builds| builds.set(builds.get() + 1));
}

#[cfg(test)]
pub(super) fn const_builds() -> usize {
    CONST_BUILDS.with(std::cell::Cell::get)
}

// The same for [`super::Op::LoadConstant`]; see `CONST_BUILDS` for why the two
// are counted apart.
#[cfg(test)]
thread_local! {
    static LOAD_CONSTANT_BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn count_load_constant_build() {
    if !counting() {
        return;
    }
    LOAD_CONSTANT_BUILDS.with(|builds| builds.set(builds.get() + 1));
}

#[cfg(test)]
pub(super) fn load_constant_builds() -> usize {
    LOAD_CONSTANT_BUILDS.with(std::cell::Cell::get)
}

// Test-only instrumentation: the deepest frame stack any [`Interp::run_ops`]
// entry on this thread has found already open.
#[cfg(test)]
thread_local! {
    static FRAME_FLOOR_HIGH_WATER: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(super) fn record_frame_floor(base: usize) {
    if !counting() {
        return;
    }
    FRAME_FLOOR_HIGH_WATER.with(|floor| floor.set(floor.get().max(base)));
}

#[cfg(test)]
pub(super) fn frame_floor_high_water() -> usize {
    FRAME_FLOOR_HIGH_WATER.with(std::cell::Cell::get)
}
