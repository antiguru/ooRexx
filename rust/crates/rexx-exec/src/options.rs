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

//! `::OPTIONS`: what a file's directives leave on its own package.
//!
//! The oracle's `PackageSetting` (`execution/PackageSetting.hpp`), reached
//! through `optionsDirective` (`parser/DirectiveParser.cpp:948`). Every
//! activation of a body this package owns starts from these rather than from
//! the language defaults, which is what makes `::OPTIONS DIGITS 12` reach a
//! `::ROUTINE` and a `::METHOD` as well as the main body.

use rexx_num::{Form, Settings, SettingsError};
use rexx_parse::{ConditionOption, OptionsForm, PackageOption};

use crate::trace::{TraceMode, mode_from_setting};

/// The six conditions `::OPTIONS` can turn from a condition into a SYNTAX
/// error, in `subDirectives[]` order.
///
/// `ALL` is not one of them: it is a spelling that writes all six at once
/// (`DirectiveParser.cpp:1258`-`:1290`), which [`PackageOptions::apply`]
/// does.
const ESCALATABLE: [&[u8]; 6] = [
    b"ERROR",
    b"FAILURE",
    b"LOSTDIGITS",
    b"NOSTRING",
    b"NOTREADY",
    b"NOVALUE",
];

/// The settings one file's `::OPTIONS` directives accumulate.
///
/// Accumulated across every `::OPTIONS` in the file in source order, and the
/// last write to a setting wins -- measured, `::options digits 12` followed
/// by `::options digits 20 fuzz 4` leaves `digits()` 20 and `fuzz()` 4.
#[derive(Clone, Debug, Default)]
pub(crate) struct PackageOptions {
    /// `DIGITS`, `FUZZ` and `FORM`: the numeric settings every activation of
    /// this package's code starts from, and the defaults a bare `NUMERIC
    /// DIGITS`/`FUZZ`/`FORM` resets to.
    pub(crate) numeric: Settings,
    /// `TRACE`, or `None` for a package that named none. `None` is not
    /// `TraceMode::NORMAL`: a package that says nothing leaves the
    /// activation's own starting mode alone.
    pub(crate) trace: Option<TraceMode>,
    /// `NUMERIC INHERIT`, which makes a `::ROUTINE` or `::METHOD` start from
    /// the numeric settings in force at the call site instead of from
    /// [`PackageOptions::numeric`].
    pub(crate) numeric_inherit: bool,
    /// Which of [`ESCALATABLE`] this package raises as a SYNTAX error, which
    /// every activation of its code starts from and can then turn off.
    pub(crate) syntax: ConditionSyntax,
}

/// Which conditions one activation raises as a SYNTAX error rather than as a
/// condition.
///
/// **State of the activation and not of the package**, which is the oracle's
/// own shape: `RexxActivation::trapOn` and `trapOff` turn the flag off for
/// whatever condition a `SIGNAL ON`/`OFF` names, and an activation is where
/// that write lands (`execution/RexxActivation.cpp:1547`, `:1598`). Measured:
/// `::options novalue syntax` with `signal on novalue` then `signal off
/// novalue` reads an unset variable as its own name, where the same file
/// without those two clauses is 98.986.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct ConditionSyntax {
    escalated: [bool; 6],
}

impl ConditionSyntax {
    /// `::OPTIONS <which> SYNTAX`/`CONDITION`, with `ALL` writing all six.
    fn set(&mut self, which: ConditionOption, syntax: bool) {
        match which {
            ConditionOption::All => self.escalated = [syntax; 6],
            ConditionOption::Error => self.escalated[0] = syntax,
            ConditionOption::Failure => self.escalated[1] = syntax,
            ConditionOption::LostDigits => self.escalated[2] = syntax,
            ConditionOption::NoString => self.escalated[3] = syntax,
            ConditionOption::NotReady => self.escalated[4] = syntax,
            ConditionOption::NoValue => self.escalated[5] = syntax,
        }
    }

    /// Whether an untrapped `condition` is a SYNTAX error here.
    ///
    /// `false` for every name outside [`ESCALATABLE`], which is every
    /// condition `::OPTIONS` has no spelling for -- `HALT`, `SYNTAX` and a
    /// `USER` name among them.
    pub(crate) fn raises(self, condition: &[u8]) -> bool {
        ESCALATABLE
            .iter()
            .position(|name| *name == condition)
            .is_some_and(|which| self.escalated[which])
    }

    /// `SIGNAL ON`/`OFF` and `CALL ON`/`OFF` of `condition`, each of which
    /// turns off the escalation the package asked for -- for the named
    /// condition, or for all six when the name is `ANY`.
    ///
    /// `signal` is false for the two `CALL` forms, which leave `NOVALUE`,
    /// `LOSTDIGITS` and `NOSTRING` escalating: measured, `call on any` in a
    /// file carrying `::options novalue syntax` still reports 98.986 for an
    /// unset read, where `signal on any` reads the derived name. `NOVALUE`
    /// additionally needs `novalue_trapped` false, which is `trapOff`'s own
    /// extra guard -- a `NOVALUE` or `ANY` trap left in the table after the
    /// removal keeps the flag where it was.
    pub(crate) fn disable_for(&mut self, condition: &[u8], signal: bool, novalue_trapped: bool) {
        let any = condition == b"ANY";
        let names = |name: &[u8]| any || condition == name;
        if signal && names(b"NOVALUE") && !novalue_trapped {
            self.escalated[5] = false;
        }
        if names(b"ERROR") {
            self.escalated[0] = false;
        }
        if names(b"FAILURE") {
            self.escalated[1] = false;
        }
        if signal && names(b"LOSTDIGITS") {
            self.escalated[2] = false;
        }
        if signal && names(b"NOSTRING") {
            self.escalated[3] = false;
        }
        if names(b"NOTREADY") {
            self.escalated[4] = false;
        }
    }
}

impl PackageOptions {
    /// Folds one `::OPTIONS` option into these settings.
    ///
    /// The `Err` is the DIGITS/FUZZ cross-check `optionsDirective` makes
    /// against the package's accumulated pair rather than against a single
    /// directive's: measured, `::options fuzz 5` and `::options digits 3` in
    /// separate directives is the same 33.1 the two written together give.
    pub(crate) fn apply(&mut self, option: &PackageOption) -> Result<(), SettingsError> {
        match option {
            PackageOption::Digits(digits) => self.numeric.set_digits(*digits as u64)?,
            PackageOption::Fuzz(fuzz) => self.numeric.set_fuzz(*fuzz as u64)?,
            PackageOption::Form(form) => self.numeric.set_form(match form {
                OptionsForm::Scientific => Form::Scientific,
                OptionsForm::Engineering => Form::Engineering,
            }),
            PackageOption::Trace(setting) => {
                self.trace = Some(
                    mode_from_setting(setting)
                        .expect("rexx-parse's check_trace_setting already validated this byte"),
                );
            }
            PackageOption::Condition { which, syntax } => self.syntax.set(*which, *syntax),
            // Nothing here reads it, and that is a property of `::REQUIRES`
            // rather than an omission: a package's prolog runs when another
            // file requires it, `directive_gap` refuses every `::REQUIRES`,
            // and a program's own prolog runs whichever way this is set --
            // measured, `::options noprolog` beside `say 'main'` prints the
            // line on the oracle. Whoever implements `::REQUIRES` owes this
            // field and the suppression it selects.
            PackageOption::Prolog(_) => {}
            PackageOption::NumericInherit(inherit) => self.numeric_inherit = *inherit,
        }
        Ok(())
    }

    /// Whether `NOSTRING` is one of the escalated ones, which
    /// `Interp::install_directives` asks in order to arm the required-string
    /// protocol.
    pub(crate) fn escalates_nostring(&self) -> bool {
        self.syntax.raises(b"NOSTRING")
    }
}

#[cfg(test)]
mod tests {
    use super::{ConditionSyntax, ESCALATABLE, PackageOptions};
    use rexx_parse::{ConditionOption, PackageOption};

    /// `ALL` writes every one of the six and nothing outside them, asserted
    /// over [`ESCALATABLE`] rather than over a list written here -- a name
    /// added to that array with no arm in `ConditionSyntax::set` fails this
    /// instead of passing unnoticed.
    #[test]
    fn all_writes_every_escalatable_condition_and_no_other_name() {
        let mut options = PackageOptions::default();
        for name in ESCALATABLE {
            assert!(
                !options.syntax.raises(name),
                "a fresh package escalates {}",
                String::from_utf8_lossy(name)
            );
        }
        options
            .apply(&PackageOption::Condition {
                which: ConditionOption::All,
                syntax: true,
            })
            .expect("a condition option makes no numeric check");
        for name in ESCALATABLE {
            assert!(
                options.syntax.raises(name),
                "`::OPTIONS ALL SYNTAX` left {} unescalated",
                String::from_utf8_lossy(name)
            );
        }
        for name in [b"HALT".as_slice(), b"SYNTAX", b"USER MINE", b""] {
            assert!(
                !options.syntax.raises(name),
                "`::OPTIONS ALL SYNTAX` escalated {}, which has no `::OPTIONS` spelling",
                String::from_utf8_lossy(name)
            );
        }
    }

    /// Each of the six is written on its own, and writing one leaves the
    /// other five where they were. Without this an `apply` that indexed one
    /// slot for two conditions would still pass the `ALL` test above.
    #[test]
    fn each_condition_option_writes_only_its_own_slot() {
        for (which, name) in each_condition() {
            let mut options = PackageOptions::default();
            options
                .apply(&PackageOption::Condition {
                    which,
                    syntax: true,
                })
                .expect("a condition option makes no numeric check");
            for other in ESCALATABLE {
                assert_eq!(
                    options.syntax.raises(other),
                    other == name,
                    "setting {} moved {}",
                    String::from_utf8_lossy(name),
                    String::from_utf8_lossy(other)
                );
            }
        }
    }

    /// `SIGNAL ON <c>` turns off exactly `<c>`'s escalation and `SIGNAL ON
    /// ANY` turns off all six, while the `CALL` forms leave the three
    /// conditions no `CALL ON` can resume from.
    ///
    /// The `CALL` half is the measured one that a "clears everything"
    /// implementation would get wrong: `call on any` in a file carrying
    /// `::options novalue syntax` still reports 98.986.
    #[test]
    fn a_trap_clause_disables_the_escalation_its_own_condition_asked_for() {
        let uncallable = [b"LOSTDIGITS".as_slice(), b"NOSTRING", b"NOVALUE"];
        for (_, name) in each_condition() {
            for (spelling, signal) in [(b"ANY".as_slice(), true), (name, true), (b"ANY", false)] {
                let mut syntax = all_escalated();
                syntax.disable_for(spelling, signal, false);
                for other in ESCALATABLE {
                    let cleared = (spelling == b"ANY" || other == name)
                        && (signal || !uncallable.contains(&other));
                    assert_eq!(
                        syntax.raises(other),
                        !cleared,
                        "{} {} left {} wrong",
                        if signal { "SIGNAL ON" } else { "CALL ON" },
                        String::from_utf8_lossy(spelling),
                        String::from_utf8_lossy(other)
                    );
                }
            }
        }
    }

    /// `trapOff`'s own extra guard: a `NOVALUE` or `ANY` trap still in the
    /// table after the removal leaves `NOVALUE`'s escalation alone, where
    /// every other condition's is cleared regardless.
    #[test]
    fn a_remaining_novalue_trap_keeps_that_one_escalation() {
        let mut syntax = all_escalated();
        syntax.disable_for(b"NOVALUE", true, true);
        assert!(
            syntax.raises(b"NOVALUE"),
            "a `NOVALUE` trap left in the table did not keep the escalation"
        );
        let mut syntax = all_escalated();
        syntax.disable_for(b"NOVALUE", true, false);
        assert!(
            !syntax.raises(b"NOVALUE"),
            "with no trap left the escalation must go"
        );
    }

    /// The six, paired with the name [`ESCALATABLE`] holds for each.
    fn each_condition() -> [(ConditionOption, &'static [u8]); 6] {
        [
            (ConditionOption::Error, b"ERROR".as_slice()),
            (ConditionOption::Failure, b"FAILURE"),
            (ConditionOption::LostDigits, b"LOSTDIGITS"),
            (ConditionOption::NoString, b"NOSTRING"),
            (ConditionOption::NotReady, b"NOTREADY"),
            (ConditionOption::NoValue, b"NOVALUE"),
        ]
    }

    /// `::OPTIONS ALL SYNTAX`, as a starting point for the clearing tests.
    fn all_escalated() -> ConditionSyntax {
        let mut options = PackageOptions::default();
        options
            .apply(&PackageOption::Condition {
                which: ConditionOption::All,
                syntax: true,
            })
            .expect("a condition option makes no numeric check");
        options.syntax
    }
}
