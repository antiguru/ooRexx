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
    /// from Phase 4, whole or in part, each with the phase that file gives it:
    /// its fifteen whole exclusions followed by its three partial rows. The
    /// owner rides here rather than in the refusal, so a name and the phase
    /// blamed for it cannot drift apart.
    pub const EXCLUDED: &[(&str, &str)] = &[
        ("CHARIN", "Phase 7"),
        ("CHAROUT", "Phase 7"),
        ("CHARS", "Phase 7"),
        ("LINEIN", "Phase 7"),
        ("LINEOUT", "Phase 7"),
        ("LINES", "Phase 7"),
        ("STREAM", "Phase 7"),
        ("QUALIFY", "Phase 7"),
        ("USERID", "Phase 7"),
        ("SETLOCAL", "Phase 7"),
        ("ENDLOCAL", "Phase 7"),
        ("RXQUEUE", "Phase 10"),
        ("RXFUNCADD", "Phase 10"),
        ("RXFUNCDROP", "Phase 10"),
        ("RXFUNCQUERY", "Phase 10"),
        // Partial: in scope in one form, excluded in another. The owner is
        // the excluded form's -- `VALUE`'s external selector and `ADDRESS`'s
        // command issuing are Phase 7's, and `QUEUED`'s cross-process half is
        // the RXAPI daemon's.
        ("VALUE", "Phase 7"),
        ("ADDRESS", "Phase 7"),
        ("QUEUED", "Phase 10"),
    ];

    /// The phase owing `name`'s excluded form, or `None` for a name
    /// [`EXCLUDED`] does not list.
    pub fn owner_of(name: &str) -> Option<&'static str> {
        EXCLUDED
            .iter()
            .find(|(excluded, _)| *excluded == name)
            .map(|(_, owner)| *owner)
    }

    /// Every excluded name, in [`EXCLUDED`] order.
    pub fn excluded_names() -> Vec<&'static str> {
        EXCLUDED.iter().map(|(name, _)| *name).collect()
    }

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
            .map(|(name, _)| *name)
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
