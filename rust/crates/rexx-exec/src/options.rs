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
    /// `NOPROLOG`, stored negated so that a package naming no `::OPTIONS` has
    /// its prologue run -- `PackageClass::isPrologEnabled`, which
    /// `runProlog` reads (`classes/PackageClass.cpp:2131`).
    ///
    /// **It selects what a `::REQUIRES` of this file does, not what running
    /// this file does.** Measured, oracle rc 0 both ways: `::options
    /// noprolog` beside `say 'main'` still prints the line when the file is
    /// the program, and suppresses it when another file requires this one --
    /// the directives install either way.
    pub(crate) suppress_prolog: bool,
    /// Which options a `::OPTIONS` directive named, as opposed to which are
    /// in force -- `PackageClass::isExplicit*Option`, which
    /// `Package~options('X')` renders and which the class-level override
    /// mechanism consults before it writes.
    explicit: Explicit,
}

/// The `::OPTIONS` subkeywords a package's directives actually named, in the
/// order `PackageClass::optionsExplicitlySetToString`
/// (`classes/PackageClass.cpp:2910`) writes them.
///
/// `PROLOG` and `NOPROLOG` are two flags there and two here: measured,
/// oracle rc 0, `::options prolog numeric noinherit` renders
/// `NUMERIC PROLOG` and `::options noprolog ...` renders `NOPROLOG`.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
struct Explicit {
    named: [bool; EXPLICIT_WORDS.len()],
}

/// The subkeyword each [`Explicit`] flag stands for, in rendering order.
const EXPLICIT_WORDS: [&str; 13] = [
    "DIGITS",
    "FORM",
    "FUZZ",
    "NUMERIC",
    "ERROR",
    "FAILURE",
    "LOSTDIGITS",
    "NOSTRING",
    "NOTREADY",
    "NOVALUE",
    "NOPROLOG",
    "PROLOG",
    "TRACE",
];

impl Explicit {
    /// The blank-delimited subkeyword list, empty for a package whose
    /// directives named none -- measured, oracle rc 0, a file with no
    /// `::OPTIONS` renders the null string.
    fn to_text(self) -> Vec<u8> {
        let named: Vec<&str> = EXPLICIT_WORDS
            .iter()
            .zip(self.named)
            .filter(|(_, named)| *named)
            .map(|(word, _)| *word)
            .collect();
        named.join(" ").into_bytes()
    }
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
        let mut names = |word: &str| {
            let which = EXPLICIT_WORDS
                .iter()
                .position(|candidate| *candidate == word)
                .expect("every name passed here is an EXPLICIT_WORDS entry");
            self.explicit.named[which] = true;
        };
        match option {
            PackageOption::Digits(_) => names("DIGITS"),
            PackageOption::Fuzz(_) => names("FUZZ"),
            PackageOption::Form(_) => names("FORM"),
            PackageOption::Trace(_) => names("TRACE"),
            PackageOption::Condition { which, .. } => match which {
                ConditionOption::All => {
                    for name in ESCALATABLE {
                        names(&String::from_utf8_lossy(name));
                    }
                }
                ConditionOption::Error => names("ERROR"),
                ConditionOption::Failure => names("FAILURE"),
                ConditionOption::LostDigits => names("LOSTDIGITS"),
                ConditionOption::NoString => names("NOSTRING"),
                ConditionOption::NotReady => names("NOTREADY"),
                ConditionOption::NoValue => names("NOVALUE"),
            },
            PackageOption::Prolog(enabled) => names(if *enabled { "PROLOG" } else { "NOPROLOG" }),
            PackageOption::NumericInherit(_) => names("NUMERIC"),
        }
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
            PackageOption::Prolog(enabled) => self.suppress_prolog = !*enabled,
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

    /// Whether `LOSTDIGITS` is one of them, which `Interp::install_directives`
    /// asks in order to arm the arithmetic path's own gate.
    pub(crate) fn escalates_lostdigits(&self) -> bool {
        self.syntax.raises(b"LOSTDIGITS")
    }

    /// `PackageSetting::toString` (`execution/PackageSetting.hpp:122`): these
    /// settings written back as the `::OPTIONS` directive that would produce
    /// them, which `Package~options` answers when sent no option name.
    ///
    /// `trace` is the setting the package carries, and `None` renders
    /// `?n/a?` -- the oracle's own last arm, reached by a `PackageSetting`
    /// whose trace flags were never defaulted. Measured, oracle rc 0: the
    /// REXX package's `~options` ends `TRACE ?n/a?` where a program
    /// package's ends `TRACE NORMAL`.
    pub(crate) fn to_options_string(&self, trace: Option<TraceMode>) -> Vec<u8> {
        let condition = |which: &[u8]| {
            if self.syntax.raises(which) {
                "SYNTAX"
            } else {
                "CONDITION"
            }
        };
        format!(
            "::OPTIONS DIGITS {} FORM {} FUZZ {} NUMERIC {} ERROR {} FAILURE {} LOSTDIGITS {} \
NOSTRING {} NOTREADY {} NOVALUE {} {} TRACE {}",
            self.numeric.digits(),
            form_word(self.numeric.form()),
            self.numeric.fuzz(),
            if self.numeric_inherit {
                "INHERIT"
            } else {
                "NOINHERIT"
            },
            condition(b"ERROR"),
            condition(b"FAILURE"),
            condition(b"LOSTDIGITS"),
            condition(b"NOSTRING"),
            condition(b"NOTREADY"),
            condition(b"NOVALUE"),
            if self.suppress_prolog {
                "NOPROLOG"
            } else {
                "PROLOG"
            },
            trace.map_or("?n/a?", trace_word),
        )
        .into_bytes()
    }

    /// What `Package~options(name)` answers for one option name.
    ///
    /// The oracle switches on the first byte and, for `F` and the `NO`
    /// spellings, on the second -- `PackageClass::options`
    /// (`classes/PackageClass.cpp:2190`), whose own list of accepted names is
    /// what [`OptionQuery::Unknown`] reports. Measured at rc 0 in one program
    /// carrying `::options digits 13 novalue syntax`, one send per name:
    /// `EXPLICITLYDEFINED` answers `CONDITION` because its `E` reaches the
    /// ERROR arm first, `INITIALOPTIONS` and `RESETOPTIONS` both answer the
    /// whole `::OPTIONS` string, and `SETOPTIONS` and `ALL` are argument
    /// errors rather than answers because each needs the second argument.
    ///
    /// **`I` and `R` answer these settings and not the language defaults.**
    /// `saveInitialPackageSettings` snapshots the package's own settings, and
    /// nothing here can move them afterwards -- the setting form of
    /// `~options` is refused -- so the snapshot and the current settings are
    /// the same string. Measured: in the program above both answer
    /// `DIGITS 13 ... NOVALUE SYNTAX`, not `DIGITS 9 ... NOVALUE CONDITION`.
    pub(crate) fn option_query(&self, name: &[u8], trace: Option<TraceMode>) -> OptionQuery {
        let upper: Vec<u8> = name.to_ascii_uppercase();
        let condition = |which: &[u8]| {
            OptionQuery::Value(
                if self.syntax.raises(which) {
                    "SYNTAX"
                } else {
                    "CONDITION"
                }
                .as_bytes()
                .to_vec(),
            )
        };
        let text = |value: &str| OptionQuery::Value(value.as_bytes().to_vec());
        let Some(first) = upper.first() else {
            return OptionQuery::Unknown;
        };
        match (first, upper.get(1), upper.get(2)) {
            (b'A', _, _) => OptionQuery::NeedsValue,
            (b'D', _, _) => text(&self.numeric.digits().to_string()),
            (b'E', _, _) => condition(b"ERROR"),
            (b'F', Some(b'A'), _) => condition(b"FAILURE"),
            (b'F', Some(b'O'), _) => text(form_word(self.numeric.form())),
            (b'F', Some(b'U'), _) => text(&self.numeric.fuzz().to_string()),
            (b'I' | b'R', _, _) => OptionQuery::ReadOnly1(self.to_options_string(trace)),
            (b'L', _, _) => condition(b"LOSTDIGITS"),
            (b'N', Some(b'U'), _) => text(if self.numeric_inherit {
                "INHERIT"
            } else {
                "NOINHERIT"
            }),
            (b'N', Some(b'O'), Some(b'S')) => condition(b"NOSTRING"),
            (b'N', Some(b'O'), Some(b'T')) => condition(b"NOTREADY"),
            (b'N', Some(b'O'), Some(b'V')) => condition(b"NOVALUE"),
            (b'P', _, _) => text(if self.suppress_prolog {
                "NOPROLOG"
            } else {
                "PROLOG"
            }),
            (b'S', _, _) => OptionQuery::MissingSecond,
            (b'T', _, _) => text(trace.map_or("?n/a?", trace_word)),
            (b'X', _, _) => OptionQuery::ReadOnly1(self.explicit.to_text()),
            _ => OptionQuery::Unknown,
        }
    }
}

/// What one `Package~options(name)` send answers, or which refusal it earns.
///
/// The three refusing arms are separate because the oracle raises three
/// different errors and the caller substitutes the name into one of them:
/// measured at rc 0 for the answers and rc 163 for each refusal, `~options('S')`
/// is 93.901, `~options('A')` is 93.903 and `~options('N')` is 93.914.
pub(crate) enum OptionQuery {
    /// The option's current value.
    Value(Vec<u8>),
    /// An option that answers a value and refuses to be set: `I`, `R` and
    /// `X`, each of which is 93.902 `1 expected` when a second argument
    /// arrives. Measured at rc 163, one send each.
    ReadOnly1(Vec<u8>),
    /// `S[etOptions]`, which reads nothing and needs the second argument --
    /// `stringArgument(strNewValue, ARG_TWO)`, 93.901.
    MissingSecond,
    /// `A[ll]`, whose own arm raises before the switch -- 93.903.
    NeedsValue,
    /// A name none of the arms accept -- 93.914, naming the whole list.
    Unknown,
}

/// `NUMERIC FORM`'s two words, as `PackageSetting::toString` writes them.
fn form_word(form: Form) -> &'static str {
    match form {
        Form::Scientific => "SCIENTIFIC",
        Form::Engineering => "ENGINEERING",
    }
}

/// The word `PackageSetting::toString` writes for one trace setting.
///
/// Keyed off [`TraceMode::letter`] rather than off the flags, because the C++
/// tests the flags in a fixed order where the letters are already one per
/// setting. Measured, oracle rc 0, one `::OPTIONS TRACE <letter>` per run:
/// the nine letters answer these nine words.
fn trace_word(mode: TraceMode) -> &'static str {
    match mode.letter {
        b'A' => "ALL",
        b'C' => "COMMANDS",
        b'E' => "ERROR",
        b'F' => "FAILURE",
        b'I' => "INTERMEDIATES",
        b'L' => "LABELS",
        b'O' => "OFF",
        b'R' => "RESULTS",
        _ => "NORMAL",
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
