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

//! The phase a refusal names, and why Phase 10 owes what it owns (spec
//! `2026-10-10-phase-9-core-conformance.md` D3).

/// Why a refusal is Phase 10's: D3's closed set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Deferral {
    /// It needs the RXAPI daemon.
    Rxapi,
    /// It needs an extension library.
    Extension,
    /// It belongs to a row 10 native-API group.
    Row10NativeApi,
    /// It is the embedding API (ruling R4).
    Embedding,
}

impl Deferral {
    pub const ALL: [Deferral; 4] = [
        Deferral::Rxapi,
        Deferral::Extension,
        Deferral::Row10NativeApi,
        Deferral::Embedding,
    ];

    /// The variant's name, as a refusal and the row files spell it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Deferral::Rxapi => "Rxapi",
            Deferral::Extension => "Extension",
            Deferral::Row10NativeApi => "Row10NativeApi",
            Deferral::Embedding => "Embedding",
        }
    }

    /// The variant [`Deferral::name`] spells `name`, or `None`.
    #[must_use]
    pub fn named(name: &str) -> Option<Deferral> {
        Deferral::ALL.into_iter().find(|known| known.name() == name)
    }
}

/// The open phase that owes a refusal's work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Owner {
    Phase9,
    Phase10(Deferral),
}

impl Owner {
    /// What a refusal prints in its parentheses: the phase's name, then a
    /// Phase 10 owner's reason.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Owner::Phase9 => "Phase 9",
            Owner::Phase10(Deferral::Rxapi) => "Phase 10 Rxapi",
            Owner::Phase10(Deferral::Extension) => "Phase 10 Extension",
            Owner::Phase10(Deferral::Row10NativeApi) => "Phase 10 Row10NativeApi",
            Owner::Phase10(Deferral::Embedding) => "Phase 10 Embedding",
        }
    }

    /// The owner a table row spells as its phase and, for Phase 10, its
    /// reason's name, or `None` for a spelling that names neither.
    #[must_use]
    pub fn named(phase: &str, reason: Option<&str>) -> Option<Owner> {
        match (phase, reason) {
            ("Phase 9", None) => Some(Owner::Phase9),
            ("Phase 10", Some(reason)) => Deferral::named(reason).map(Owner::Phase10),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Deferral, Owner};

    #[test]
    fn a_phase_10_label_is_the_phase_then_the_reason() {
        for reason in Deferral::ALL {
            let owner = Owner::Phase10(reason);
            assert_eq!(owner.label(), format!("Phase 10 {}", reason.name()));
            assert_eq!(Owner::named("Phase 10", Some(reason.name())), Some(owner));
        }
        assert_eq!(Owner::Phase9.label(), "Phase 9");
        assert_eq!(Owner::named("Phase 9", None), Some(Owner::Phase9));
        assert_eq!(Owner::named("Phase 10", None), None);
        assert_eq!(Owner::named("Phase 10", Some("Queue")), None);
        assert_eq!(Owner::named("Phase 9", Some("Rxapi")), None);
    }
}
