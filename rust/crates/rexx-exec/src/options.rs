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

use rexx_num::{Form, Settings, SettingsError};
use rexx_parse::{ConditionOption, OptionsForm, PackageOption};

use crate::error::{Failure, Raised};
use crate::plan::Package;
use crate::trace::{TraceMode, mode_from_setting};
use crate::{Interp, ProgramId};
use rexx_core::ObjRef;

/// The six conditions `::OPTIONS` can turn from a condition into a SYNTAX
/// error, in `subDirectives[]` order.
const ESCALATABLE: [&[u8]; 6] = [
    b"ERROR",
    b"FAILURE",
    b"LOSTDIGITS",
    b"NOSTRING",
    b"NOTREADY",
    b"NOVALUE",
];

/// The settings one file's `::OPTIONS` directives accumulate.
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
    /// Whether the directives named `word`, an [`EXPLICIT_WORDS`] entry.
    fn named(self, word: &str) -> bool {
        EXPLICIT_WORDS
            .iter()
            .position(|candidate| *candidate == word)
            .is_some_and(|which| self.named[which])
    }

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
    pub(crate) fn raises(self, condition: &[u8]) -> bool {
        ESCALATABLE
            .iter()
            .position(|name| *name == condition)
            .is_some_and(|which| self.escalated[which])
    }

    /// `SIGNAL ON`/`OFF` and `CALL ON`/`OFF` of `condition`, each of which
    /// turns off the escalation the package asked for -- for the named
    /// condition, or for all six when the name is `ANY`.
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
    pub(crate) fn apply(&mut self, option: &PackageOption) -> Result<(), SettingsError> {
        let mut names = |word: &str| {
            let which = EXPLICIT_WORDS
                .iter()
                .position(|candidate| *candidate == word)
                .expect("every name passed here is an EXPLICIT_WORDS entry");
            self.explicit.named[which] = true;
            // `PROLOG` and `NOPROLOG` clear each other: the directive parser
            // keeps one of the two flags (`parser/DirectiveParser.cpp:1315`).
            let other = match word {
                "PROLOG" => Some("NOPROLOG"),
                "NOPROLOG" => Some("PROLOG"),
                _ => None,
            };
            if let Some(other) = other
                && let Some(other) = EXPLICIT_WORDS
                    .iter()
                    .position(|candidate| *candidate == other)
            {
                self.explicit.named[other] = false;
            }
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

    /// What `Package~options(name)` answers for `which`: the value a read
    /// returns and a write returns as the previous one. `trace` is the
    /// setting `TRACE` renders.
    pub(crate) fn value_of(&self, which: OptionName, trace: Option<TraceMode>) -> Vec<u8> {
        let condition = |which: &[u8]| {
            if self.syntax.raises(which) {
                "SYNTAX"
            } else {
                "CONDITION"
            }
        };
        let text = match which {
            OptionName::All => {
                return ESCALATABLE
                    .iter()
                    .map(|name| format!("{} {}", String::from_utf8_lossy(name), condition(name)))
                    .collect::<Vec<_>>()
                    .join(" ")
                    .into_bytes();
            }
            OptionName::Digits => return self.numeric.digits().to_string().into_bytes(),
            OptionName::Fuzz => return self.numeric.fuzz().to_string().into_bytes(),
            OptionName::Condition(which) => condition(ESCALATABLE[which]),
            OptionName::Form => form_word(self.numeric.form()),
            OptionName::NumericInherit => {
                if self.numeric_inherit {
                    "INHERIT"
                } else {
                    "NOINHERIT"
                }
            }
            OptionName::Prolog => {
                if self.suppress_prolog {
                    "NOPROLOG"
                } else {
                    "PROLOG"
                }
            }
            OptionName::Initial | OptionName::Reset | OptionName::SetOptions => {
                return self.to_options_string(trace);
            }
            OptionName::Trace => trace.map_or("?n/a?", trace_word),
            OptionName::Explicit => return self.explicit.to_text(),
        };
        text.as_bytes().to_vec()
    }

    /// The setting switch's arms that take a value's first letter --
    /// `PackageClass::options` (`classes/PackageClass.cpp:2418`). `Err` is
    /// the list the 93.914 for any other letter names.
    pub(crate) fn set_by_letter(
        &mut self,
        which: OptionName,
        letter: u8,
    ) -> Result<(), &'static str> {
        let letter = letter.to_ascii_uppercase();
        match which {
            OptionName::All | OptionName::Condition(_) => {
                if !matches!(letter, b'S' | b'C') {
                    return Err("\"C[ondition] or S[yntax]\"");
                }
                let syntax = letter == b'S';
                match which {
                    OptionName::Condition(which) => self.syntax.escalated[which] = syntax,
                    _ => self.syntax.escalated = [syntax; 6],
                }
            }
            OptionName::Form => match letter {
                b'E' => self.numeric.set_form(Form::Engineering),
                b'S' => self.numeric.set_form(Form::Scientific),
                _ => return Err("\"E[ngineering] or S[cientific]\""),
            },
            OptionName::NumericInherit => match letter {
                b'I' => self.numeric_inherit = true,
                b'N' => self.numeric_inherit = false,
                _ => return Err("\"I[nherit] or N[oInherit]\""),
            },
            OptionName::Prolog => match letter {
                b'P' => self.suppress_prolog = false,
                b'N' => self.suppress_prolog = true,
                _ => return Err("\"N[oprolog] or P[rolog]\""),
            },
            _ => unreachable!("only the letter-valued options are set here"),
        }
        Ok(())
    }

    /// `PackageClass::overridePackageSettings`: every setting this package's
    /// own `::OPTIONS` did not name, taken from `defaults`. Only `PROLOG`
    /// guards the prologue setting: a package naming `NOPROLOG` takes it
    /// too, as `isExplicitPrologOption` reads one flag.
    pub(crate) fn take_defaults(&mut self, defaults: &PackageOptions) {
        let named = |word: &str| self.explicit.named(word);
        let pick = |word: &str, own: u64, other: u64| if named(word) { own } else { other };
        let digits = pick("DIGITS", self.numeric.digits(), defaults.numeric.digits());
        let fuzz = pick("FUZZ", self.numeric.fuzz(), defaults.numeric.fuzz());
        let form = if named("FORM") {
            self.numeric.form()
        } else {
            defaults.numeric.form()
        };
        let inherit = named("NUMERIC");
        let prolog = named("PROLOG");
        let trace = named("TRACE");
        let conditions: Vec<bool> = ESCALATABLE
            .iter()
            .map(|name| named(&String::from_utf8_lossy(name)))
            .collect();
        self.numeric = Settings::from_parts(digits, fuzz, form);
        if !inherit {
            self.numeric_inherit = defaults.numeric_inherit;
        }
        for (which, named) in conditions.into_iter().enumerate() {
            if !named {
                self.syntax.escalated[which] = defaults.syntax.escalated[which];
            }
        }
        if !prolog {
            self.suppress_prolog = defaults.suppress_prolog;
        }
        if !trace {
            self.trace = defaults.trace;
        }
    }

    /// `package->packageSettings = newSettings`: every setting `settings`
    /// holds, and not which options this package's own directives named.
    pub(crate) fn take_settings(&mut self, settings: &PackageOptions) {
        *self = PackageOptions {
            explicit: self.explicit,
            ..settings.clone()
        };
    }
}

/// One `Package~options` option name, by the letters `PackageClass::options`
/// switches on.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum OptionName {
    /// `A[ll]`, which only a write accepts.
    All,
    Digits,
    /// One of [`ESCALATABLE`], by its index there.
    Condition(usize),
    Form,
    Fuzz,
    /// `I[nitialOptions]`.
    Initial,
    /// `NU[meric]`.
    NumericInherit,
    Prolog,
    /// `R[esetOptions]`.
    Reset,
    /// `S[etOptions]`, whose value is an `::OPTIONS` directive.
    SetOptions,
    Trace,
    /// `X`, the options the package's directives named.
    Explicit,
}

impl OptionName {
    /// The option `name` names, or `None` for one none of the arms accept.
    pub(crate) fn parse(name: &[u8]) -> Option<OptionName> {
        let upper: Vec<u8> = name.to_ascii_uppercase();
        Some(match (upper.first()?, upper.get(1), upper.get(2)) {
            (b'A', _, _) => OptionName::All,
            (b'D', _, _) => OptionName::Digits,
            (b'E', _, _) => OptionName::Condition(0),
            (b'F', Some(b'A'), _) => OptionName::Condition(1),
            (b'F', Some(b'O'), _) => OptionName::Form,
            (b'F', Some(b'U'), _) => OptionName::Fuzz,
            (b'I', _, _) => OptionName::Initial,
            (b'L', _, _) => OptionName::Condition(2),
            (b'N', Some(b'U'), _) => OptionName::NumericInherit,
            (b'N', Some(b'O'), Some(b'S')) => OptionName::Condition(3),
            (b'N', Some(b'O'), Some(b'T')) => OptionName::Condition(4),
            (b'N', Some(b'O'), Some(b'V')) => OptionName::Condition(5),
            (b'P', _, _) => OptionName::Prolog,
            (b'R', _, _) => OptionName::Reset,
            (b'S', _, _) => OptionName::SetOptions,
            (b'T', _, _) => OptionName::Trace,
            (b'X', _, _) => OptionName::Explicit,
            _ => return None,
        })
    }
}

/// What `.Package~defaultOptions` writes keep: the statics `PackageClass`
/// holds (`classes/PackageClass.cpp:70`).
#[derive(Default)]
pub(crate) struct OptionWrites {
    /// `psOverridePackageSettings`.
    pub(crate) defaults: PackageOptions,
    /// `overrideCount`: a package compiled while it is not zero takes
    /// [`OptionWrites::defaults`] and moves it one down.
    pub(crate) count: i64,
}

/// `NUMERIC FORM`'s two words, as `PackageSetting::toString` writes them.
fn form_word(form: Form) -> &'static str {
    match form {
        Form::Scientific => "SCIENTIFIC",
        Form::Engineering => "ENGINEERING",
    }
}

/// The word `PackageSetting::toString` writes for one trace setting.
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

impl Interp {
    /// `package`'s settings for a change, whose first change enters what
    /// `~options('I')` answers from then on.
    pub(crate) fn package_settings_mut(&mut self, package: ObjRef) -> &mut PackageOptions {
        self.options_armed = true;
        let data = self
            .package_data_mut(package)
            .expect("a package object holds package fields");
        let settings = data.options.get_or_insert_default();
        data.initial_options.get_or_insert_with(|| settings.clone());
        settings
    }

    /// The latches `Interp::install_directives` arms for an escalated
    /// `NOSTRING` or `LOSTDIGITS`, armed for a change that escalates one.
    pub(crate) fn arm_option_latches(&mut self, package: ObjRef) {
        let Some(settings) = self
            .package_data(package)
            .and_then(|data| data.options.as_ref())
        else {
            return;
        };
        let (nostring, lostdigits) = (
            settings.escalates_nostring(),
            settings.escalates_lostdigits(),
        );
        self.reqstr_armed |= nostring;
        self.lostdigits_armed |= lostdigits;
    }

    /// `PackageClass::overridePackageSettings`, which every compile of a
    /// package runs (`parser/LanguageParser.cpp:658`, `:856`) and an
    /// `INTERPRET` does not.
    pub(crate) fn override_package_settings(&mut self, id: ProgramId) -> Result<(), Failure> {
        if self.option_writes.count == 0 {
            return Ok(());
        }
        // No latch to arm: the defaults are an `::OPTIONS` directive's,
        // whose own install armed any they escalate.
        let defaults = self.option_writes.defaults.clone();
        let package = self.package_object(Package::Program(id))?;
        self.package_settings_mut(package).take_defaults(&defaults);
        self.option_writes.count = self.option_writes.count.wrapping_sub(1);
        Ok(())
    }

    /// `PackageClass::setPackageSettings`: the settings `text`, a single
    /// `::OPTIONS` directive, gives a routine compiled from it. That compile
    /// is a package's like any other, so it takes the override too.
    pub(crate) fn settings_from_directive(
        &mut self,
        text: &[u8],
    ) -> Result<PackageOptions, Failure> {
        let mut colon = false;
        for &byte in text.iter().take_while(|byte| **byte != 0) {
            if matches!(byte, b' ' | b'\t') {
                continue;
            }
            if !colon {
                if byte != b':' {
                    return Err(Raised::method_user_defined(
                        "argument is not an ::OPTIONS directive",
                    )
                    .into());
                }
                colon = true;
                continue;
            }
            let hint = match byte {
                b';' => "\";\" (end of clause)",
                b'\r' => "\"0d\"x (CR)",
                b'\n' => "\"0a\"x (LF)",
                _ => continue,
            };
            return Err(Raised::method_user_defined(&format!(
                "argument must not contain a semi-colon, CR, or LF, found: {hint}"
            ))
            .into());
        }
        let name = b"PackageSettingsFromOptionsDirectiveString";
        let parsed = self.parse_file(text.to_vec(), name)?;
        let mut compiled = None;
        self.install_executable(parsed, name, true, None, |_, id| compiled = Some(id))?;
        Ok(compiled
            .and_then(|id| self.program_data(id)?.options.clone())
            .unwrap_or_default())
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
