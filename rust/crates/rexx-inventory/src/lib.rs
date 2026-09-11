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

//! Tables mechanically derived from the C++ tree, plus the one hand-written
//! list that indexes into them.

pub mod errors {
    include!(concat!(env!("OUT_DIR"), "/errors.rs"));
}

pub mod builtins {
    include!(concat!(env!("OUT_DIR"), "/builtins.rs"));

    /// The builtins `docs/superpowers/plans/phase-4-exclusions.txt` excludes
    /// from Phase 4, whole or in part: its fifteen whole exclusions followed
    /// by its three partial rows.
    pub const EXCLUDED: &[&str] = &[
        // Phase 7, streams and platform.
        "CHARIN",
        "CHAROUT",
        "CHARS",
        "LINEIN",
        "LINEOUT",
        "LINES",
        "STREAM",
        "QUALIFY",
        "USERID",
        "SETLOCAL",
        "ENDLOCAL",
        // Phase 10, RXAPI.
        "RXQUEUE",
        "RXFUNCADD",
        "RXFUNCDROP",
        "RXFUNCQUERY",
        // Partial: in scope in one form, excluded in another.
        "VALUE",
        "ADDRESS",
        "QUEUED",
    ];

    /// The subset of [`EXCLUDED`] that is excluded only in part, and so is
    /// still in scope in its other form. Named rather than written as a
    /// literal `3` at the two places that subtract it, because "in
    /// `EXCLUDED`" and "excluded outright" are different sets and the
    /// difference is exactly these names.
    pub const PARTIALLY_EXCLUDED: &[&str] = &["VALUE", "ADDRESS", "QUEUED"];

    /// The builtins excluded outright: [`EXCLUDED`] less
    /// [`PARTIALLY_EXCLUDED`], in `EXCLUDED` order.
    pub fn wholly_excluded() -> Vec<&'static str> {
        EXCLUDED
            .iter()
            .copied()
            .filter(|name| !PARTIALLY_EXCLUDED.contains(name))
            .collect()
    }

    /// The builtins Phase 4 must be able to run: every name in [`NAMES`] that
    /// is not excluded outright, in `NAMES` order. A partially excluded name
    /// is in here, because its in-scope form still has to work.
    pub fn in_scope() -> Vec<&'static str> {
        let whole = wholly_excluded();
        NAMES
            .iter()
            .copied()
            .filter(|name| !whole.contains(name))
            .collect()
    }
}
