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

//! `Raised`: the payload of a real Rexx condition.

use crate::Loud;
use rexx_core::ObjRef;
use rexx_num::{ArithError, FormatError};
use rexx_parse::{DirectiveKind, ParseError};
use std::borrow::Cow;

/// Which activations may trap one raise, walking outward from the one that
/// raised it.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Search {
    /// From the activation that raised, outward level by level. Every
    /// ordinary condition (`say 1/0`, an unset variable under `SIGNAL ON
    /// NOVALUE`), and `RAISE SYNTAX ... RETURN`.
    #[default]
    Here,
    /// From the **caller** of the activation that raised, outward -- the
    /// raising activation's own (inherited) trap is skipped.
    Caller,
    /// The **outermost** activation only; every level it unwinds through
    /// skips its own trap check.
    Top,
    /// No activation at all may trap this -- it is already the condition's
    /// default action, on its way to the report.
    Nobody,
}

/// How one raise in flight must be delivered, beside the payload it carries.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Delivery {
    pub(crate) search: Search,
    /// Render the major line as `Error 42:  ...` rather than
    /// `Error 42 running <path> line 8:  ...`.
    pub(crate) positionless: bool,
    /// Render the major line as `Error 88 running <path>:  ...`: the program
    /// name, and no ` line <n>` after it.
    pub(crate) lineless: bool,
}

/// One value a catalogue message interpolates: **bytes, not text**.
type Substitution = Vec<u8>;

/// The substitutions a sibling crate's error carries, as bytes.
pub(crate) fn into_substitutions(values: Vec<String>) -> Vec<Substitution> {
    values.into_iter().map(String::into_bytes).collect()
}

/// A real Rexx condition raised during evaluation.
#[derive(Clone, Debug)]
pub(crate) struct Raised {
    /// The condition name a trapped Rexx program would see from
    /// `condition('c')`, and the exact bytes an activation's trap table is
    /// keyed by (`Activation::traps`). It is carried as a field rather than
    /// hardcoded at each call site because the spec's own shape includes it
    /// and 4b's `NOVALUE` and `RAISE` do set it to something else.
    pub(crate) condition: Cow<'static, str>,
    pub(crate) number: u16,
    pub(crate) sub: u16,
    /// What `&1`, `&2`, ... in this error's catalogue entry stand for.
    pub(crate) additional: Vec<Substitution>,
    /// What a trapping handler reads back from `RC`, or `None` to leave `RC`
    /// alone.
    /// ```text
    /// signal on syntax  ; say 1/0            -> handler sees RC = 42
    /// signal on syntax  ; raise syntax 40.4  -> handler sees RC = 40  (not 40.4)
    /// signal on novalue ; say zunset         -> handler sees RC = RC  (untouched)
    /// ```
    pub(crate) rc: Option<Vec<u8>>,
    /// `RAISE ... DESCRIPTION expr`'s rendered value, which a trapping
    /// handler reads back through `CONDITION('D')`.
    pub(crate) description: Option<Vec<u8>>,
    pub(crate) delivery: Delivery,
}

/// Which of the two grouped digit notations a validation error is about.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Notation {
    Hex,
    Binary,
}

impl Raised {
    /// A `SYNTAX` condition with an ordinary delivery -- the shape every
    /// raiser outside `RAISE` itself has.
    pub(crate) fn syntax(number: u16, sub: u16, additional: Vec<Substitution>) -> Raised {
        Raised {
            condition: Cow::Borrowed("SYNTAX"),
            number,
            sub,
            additional,
            rc: Some(number.to_string().into_bytes()),
            description: None,
            delivery: Delivery::default(),
        }
    }

    /// A condition that is not `SYNTAX`, carrying no catalogue entry.
    pub(crate) fn condition(name: Cow<'static, str>) -> Raised {
        Raised {
            condition: name,
            number: 0,
            sub: 0,
            additional: Vec::new(),
            rc: None,
            description: None,
            delivery: Delivery::default(),
        }
    }

    /// Whether this condition would *report* if nothing traps it.
    pub(crate) fn reportable(&self) -> bool {
        self.number != 0
    }

    /// 4.1, `HALT`'s own untrapped default action: "Program interrupted with
    /// HALT condition", measured at rc 252 for `raise halt` with no trap
    /// enabled, and measured *again* with `signal on halt` enabled in the
    /// same (top-level) activation -- the tail-less `RAISE` terminates that
    /// activation before its own trap can see the condition, so the trap
    /// does not fire.
    pub(crate) fn halt() -> Raised {
        Raised {
            condition: Cow::Borrowed("HALT"),
            number: 4,
            sub: 1,
            // The `(4, 1)` catalogue entry is "Program interrupted with &1
            // condition.", so the condition's own name is the substitution
            // -- measured, the oracle prints `HALT`, and a version with no
            // substitution prints the literal `&1`.
            additional: vec![b"HALT".to_vec()],
            // `None` rather than `4`: `RC` is measured to carry the major
            // only for `SYNTAX` (42 for `say 1/0`, 40 for `raise syntax
            // 40.4`) and to be left untouched for a trapped `NOVALUE`. A
            // trapped `HALT` is not measured either way, so this follows the
            // non-`SYNTAX` row rather than inventing a third rule.
            rc: None,
            description: None,
            delivery: Delivery::default(),
        }
    }

    /// 41.1: a nonnumeric value used in arithmetic. `value` is the
    /// operand's own text, verbatim -- measured, `say 'abc' + 1` reports
    /// `Nonnumeric value ("abc")`, the operand as it renders, not upcased
    /// or otherwise transformed.
    pub(crate) fn nonnumeric(value: &[u8]) -> Raised {
        Raised::syntax(41, 1, vec![value.to_vec()])
    }

    /// 26.8: `**`'s right operand is not a whole number, **including not
    /// being a number at all**. Measured: `2 ** 'x'` and `2 ** 2.5` both
    /// give 26.8 ("found \"x\""/"found \"2.5\""), while the identical
    /// failure on the *left* operand is the ordinary 41.1 (`'y' ** 2` is
    /// 41.1, `'y' ** 'x'` is still 41.1 -- the base is checked first).
    /// This is deliberately not routed through `nonnumeric`: the oracle's
    /// own asymmetry between the two operands is the fact being
    /// reproduced, not an implementation shortcut. `found` is the
    /// exponent's own text; used when the exponent does not even parse as
    /// a number, so there is no `Number` for `rexx-num`'s own
    /// `ArithError::PowerExponentNotWhole` to carry.
    pub(crate) fn power_exponent_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(26, 8, vec![found.to_vec()])
    }

    /// 34.901: the prefix `\` operator's operand is not a logical value.
    /// A logical value is *exactly* the one-byte string `0` or `1`, no
    /// coercion -- this is a text check, never a numeric one, which is
    /// why the caller passes the operand's own rendered text rather than
    /// anything from `to_number`. Measured: `say \'abc'` gives 34.901,
    /// `Logical value must be exactly "0" or "1"; found "abc"`.
    pub(crate) fn not_logical(found: &[u8]) -> Raised {
        Raised::syntax(34, 901, vec![found.to_vec()])
    }

    /// 11.1: "Insufficient control stack space" -- D19's evaluation-depth
    /// limit (`eval.rs`'s own `MAX_EVAL_DEPTH`). No substitution: measured
    /// against the oracle's own parse-side 11.1 (nested parens/calls,
    /// `phase-4-exclusions.txt`'s Deviation 2), the catalogue's `(11, 1)`
    /// entry carries none either.
    pub(crate) fn insufficient_stack() -> Raised {
        Raised::syntax(11, 1, Vec::new())
    }

    /// 34.6: one element of a comma-separated logical list
    /// (`ExprKind::Logical`, `if a, b then` and friends) is not a logical
    /// value. A distinct sub-number from `not_logical`'s 34.901, and
    /// deliberately not shared with it even though the underlying check is
    /// identical (exactly `0` or `1`, text not numeric) -- measured, `if 1,
    /// 'x' then` gives 34.6 ("Value of logical list expression element
    /// must be exactly \"0\" or \"1\"; found \"x\""), a different message
    /// from `&`'s 34.901 for the identical bad value. `IF`/`WHEN`/`WHILE`/
    /// `UNTIL`'s own 34.1/34.2/34.3/34.4 are for when the *whole* condition
    /// is a single expression, not a list, and are Tasks 9-11's to raise
    /// when they exist; this crate has no instruction context yet to
    /// prefer one of those over 34.6, so 34.6 is `ExprKind::Logical`'s own
    /// answer regardless of which keyword built the list.
    pub(crate) fn logical_list_element(found: &[u8]) -> Raised {
        Raised::syntax(34, 6, vec![found.to_vec()])
    }

    /// 44.1: an internal routine reached through `ExprKind::Call`'s
    /// expression form (`f(...)`, Task 4, `eval.rs`'s `eval_call`) ran to
    /// completion without a value to hand back -- a bare `RETURN`, measured
    /// against the oracle in a clean directory: `say f(1)` into `f: return`
    /// gives rc 212 and
    /// ```text
    /// Error 44 running .../f.rex line 1:  Function or message did not return data.
    /// Error 44.1:  No data returned from function "F".
    /// ```
    pub(crate) fn no_data_returned(name: &[u8]) -> Raised {
        Raised::syntax(44, 1, vec![name.to_vec()])
    }

    /// 43.1: a named call matched no internal label, no builtin and no
    /// `::ROUTINE` of the running program. `name` is the target **as the
    /// call site spells it** -- upcased for a bare symbol, verbatim for a
    /// quoted literal or a `CALL (expr)` target.
    pub(crate) fn routine_not_found(name: &[u8]) -> Raised {
        Raised::syntax(43, 1, vec![name.to_vec()])
    }

    /// 43.902: a `ns:name(...)` or `CALL ns:name` whose namespace resolved and
    /// whose public routines do not hold `name`. Two substitutions, the
    /// routine name and the namespace, both upcased by the scanner.
    pub(crate) fn namespace_routine_not_found(name: &[u8], namespace: &[u8]) -> Raised {
        Raised::syntax(43, 902, vec![name.to_vec(), namespace.to_vec()])
    }

    /// 98.987: a namespace qualifier no `::REQUIRES ... NAMESPACE` of the
    /// running package registered. Two substitutions, the namespace and the
    /// **package's own path**.
    pub(crate) fn namespace_not_found(namespace: &[u8], package: &str) -> Raised {
        Raised::syntax(
            98,
            987,
            vec![namespace.to_vec(), package.as_bytes().to_vec()],
        )
    }

    /// 98.988: a namespace that resolved and whose public classes do not hold
    /// the name. Two substitutions, the class name and the namespace.
    pub(crate) fn namespace_class_not_found(name: &[u8], namespace: &[u8]) -> Raised {
        Raised::syntax(98, 988, vec![name.to_vec(), namespace.to_vec()])
    }

    /// 90.998: a `::METHOD ... EXTERNAL 'LIBRARY REXX name'` whose entry
    /// point the `REXX` package does not export. `entry` is the name the
    /// directive resolved against -- its third word, or the method's own name
    /// upcased -- and the message quotes it as it was resolved.
    pub(crate) fn external_method_not_found(entry: &[u8]) -> Raised {
        Raised::syntax(90, 998, vec![entry.to_vec()])
    }

    /// 88.922: more arguments than a `LIBRARY REXX` entry point's own
    /// signature declares.
    pub(crate) fn too_many_external_arguments(arity: usize) -> Raised {
        let mut raised = Raised::syntax(88, 922, vec![arity.to_string().into_bytes()]);
        raised.delivery.lineless = true;
        raised
    }

    /// 99.903: two `::ROUTINE` directives of the same name in one program.
    /// No substitutions -- the message names neither.
    pub(crate) fn duplicate_routine() -> Raised {
        Raised::syntax(99, 903, Vec::new())
    }

    /// 99.901: two `::CLASS` directives of one name in one program, and
    /// 99.942: two `::RESOURCE` directives of one name. No substitutions --
    /// neither message names anything.
    pub(crate) fn duplicate_class() -> Raised {
        Raised::syntax(99, 901, Vec::new())
    }

    /// See [`Raised::duplicate_class`], which carries the measurements for
    /// both halves of the pair.
    pub(crate) fn duplicate_resource() -> Raised {
        Raised::syntax(99, 942, Vec::new())
    }

    /// 99.902, 99.931 and 99.932: two member directives of one class writing
    /// the same dictionary key. No substitutions -- none of the messages
    /// names anything.
    pub(crate) fn duplicate_member(kind: &DirectiveKind) -> Raised {
        let sub = match kind {
            DirectiveKind::Attribute(_) => 931,
            DirectiveKind::Constant(_) => 932,
            _ => 902,
        };
        Raised::syntax(99, sub, Vec::new())
    }

    /// 99.905: a `CLASS` keyword on a member directive with no `::CLASS`
    /// above it. No substitutions.
    pub(crate) fn class_keyword_needs_class() -> Raised {
        Raised::syntax(99, 905, Vec::new())
    }

    /// 98.909: a `::CLASS` directive naming a `SUBCLASS` or an `INHERIT`
    /// target no name resolves to. One substitution, the target's upcased
    /// spelling.
    pub(crate) fn class_not_found(name: &[u8]) -> Raised {
        Raised::syntax(98, 909, vec![name.to_vec()])
    }

    /// 98.908: a `::CLASS` directive naming a `METACLASS` target no name
    /// resolves to. One substitution, the target's upcased spelling.
    pub(crate) fn metaclass_not_found(name: &[u8]) -> Raised {
        Raised::syntax(98, 908, vec![name.to_vec()])
    }

    /// 99.927: a `METACLASS` target that resolves to a class which is not a
    /// metaclass. One substitution, the target's `~defaultName`.
    pub(crate) fn bad_metaclass(metaclass: &[u8]) -> Raised {
        Raised::syntax(99, 927, vec![metaclass.to_vec()])
    }

    /// 98.990: `::CLASS ... ABSTRACT` on a class that is a metaclass. One
    /// substitution, the class's own `~id`, and the message quotes nothing
    /// around it.
    pub(crate) fn abstract_metaclass(id: &[u8]) -> Raised {
        Raised::syntax(98, 990, vec![id.to_vec()])
    }

    /// 88.916: an option argument that is not one of the values the method
    /// takes -- `Error_Invalid_argument_list`. It substitutes the argument's
    /// position, the values it may take (already quoted, as the caller writes
    /// them), and the value found.
    pub(crate) fn not_one_of(position: usize, values: &[u8], found: &[u8]) -> Raised {
        Raised::syntax(
            88,
            916,
            vec![
                position.to_string().into_bytes(),
                values.to_vec(),
                found.to_vec(),
            ],
        )
    }

    /// 98.991: `run`, `setMethod` or `unsetMethod` sent from somewhere
    /// `RexxObject::checkRestrictedMethod` does not allow (D66). One
    /// substitution, the method's own name.
    pub(crate) fn restricted_method(name: &[u8]) -> Raised {
        Raised::syntax(98, 991, vec![name.to_vec()])
    }

    /// 98.989: `~new` on an `ABSTRACT` class -- `RexxClass::checkAbstract`
    /// (`classes/ClassClass.cpp:1741`). One substitution, the class's own
    /// `~id`, unquoted.
    pub(crate) fn abstract_class(id: &[u8]) -> Raised {
        Raised::syntax(98, 989, vec![id.to_vec()])
    }

    /// 98.972: an arithmetic operand carrying more digits than the precision
    /// in force, under `::OPTIONS LOSTDIGITS SYNTAX`. One substitution, the
    /// operand's own string value.
    pub(crate) fn lostdigits(operand: &[u8]) -> Raised {
        Raised::syntax(98, 972, vec![operand.to_vec()])
    }

    /// 98.942: an `INHERIT` target that is not a `MIXINCLASS`. One
    /// substitution, the target's `~defaultName`.
    pub(crate) fn inherit_needs_a_mixinclass(mixin: &[u8]) -> Raised {
        Raised::syntax(98, 942, vec![mixin.to_vec()])
    }

    /// 98.943: an `INHERIT` target whose base class the inheriting class does
    /// not already have in scope. Three substitutions: the inheriting class,
    /// the mixin, and the mixin's base class, each a `~defaultName`.
    pub(crate) fn inherit_base_class(class: &[u8], mixin: &[u8], base: &[u8]) -> Raised {
        Raised::syntax(98, 943, vec![class.to_vec(), mixin.to_vec(), base.to_vec()])
    }

    /// 98.944: an `INHERIT` target that is already part of the inheriting
    /// class's own hierarchy, in either direction. Two substitutions, the
    /// inheriting class's `~defaultName` and the mixin's.
    pub(crate) fn recursive_inherit(class: &[u8], mixin: &[u8]) -> Raised {
        Raised::syntax(98, 944, vec![class.to_vec(), mixin.to_vec()])
    }

    /// 98.945: a class named as an `~inherit` position or as an `~uninherit`
    /// target that the receiving class does not inherit. Two substitutions,
    /// the receiver's `~defaultName` and the named class's.
    pub(crate) fn not_inherited(class: &[u8], other: &[u8]) -> Raised {
        Raised::syntax(98, 945, vec![class.to_vec(), other.to_vec()])
    }

    /// 98.985: one of the five class mutators sent to a class the image
    /// itself defines. No substitutions -- the message names neither the
    /// class nor the method.
    pub(crate) fn rexx_defined_class() -> Raised {
        Raised::syntax(98, 985, Vec::new())
    }

    /// 98.981: a `RexxContext` whose activation has ended. No substitutions.
    pub(crate) fn context_not_active() -> Raised {
        Raised::syntax(98, 981, Vec::new())
    }

    /// 98.984: `~addClass` or `~addPublicClass` sent to the package the
    /// primitive classes belong to. No substitutions.
    pub(crate) fn rexx_package_addition() -> Raised {
        Raised::syntax(98, 984, Vec::new())
    }

    /// 98.911: `::CLASS` directives whose declared targets cannot be put in
    /// an order. One substitution, the program's own path.
    pub(crate) fn cyclic_inheritance(path: &str) -> Raised {
        Raised::syntax(98, 911, vec![path.as_bytes().to_vec()])
    }

    /// 43.901: a `::REQUIRES` whose name the four-route search did not find.
    /// One substitution, **the name as the directive wrote it** rather than
    /// anything the search built from it.
    pub(crate) fn requires_file_not_found(name: &[u8]) -> Raised {
        Raised::syntax(43, 901, vec![name.to_vec()])
    }

    /// 3.1: `Method~newFile` or `Routine~newFile` naming a file that cannot
    /// be read. One substitution, **the name as the program wrote it** rather
    /// than anything resolved from it.
    pub(crate) fn executable_file_unreadable(name: &[u8]) -> Raised {
        Raised::syntax(3, 1, vec![name.to_vec()])
    }

    /// 99.917: `loadExternalMethod` or `loadExternalRoutine` given a
    /// descriptor that is not an external name specification. One
    /// substitution, the descriptor.
    pub(crate) fn bad_external_specification(descriptor: &[u8]) -> Raised {
        Raised::syntax(99, 917, vec![descriptor.to_vec()])
    }

    /// 98.952: a `::REQUIRES` naming a package whose own directives are still
    /// installing. One substitution, the **resolved** path.
    pub(crate) fn circular_requires(path: &str) -> Raised {
        Raised::syntax(98, 952, vec![path.as_bytes().to_vec()])
    }

    /// 99.906: a `::CONSTANT` directive's parenthesised expression form with
    /// no `::CLASS` directive anywhere before it in the file. No
    /// substitutions -- the message names neither directive.
    pub(crate) fn constant_needs_class() -> Raised {
        Raised::syntax(99, 906, Vec::new())
    }

    /// 99.945: an `::ANNOTATE` named a target the accumulated package does
    /// not hold. `kind` is the keyword's own lower-case spelling and `name`
    /// the upcased target name, which is how
    /// `Error_Translation_missing_annotation_target` takes them:
    /// `::ANNOTATE target &1 "&2" not found.`
    /// (`messages/RexxErrorMessages.h:715`), with the quotes in the template
    /// and each `syntaxError` call passing a bare C string for `&1`
    /// (`parser/DirectiveParser.cpp:1983`, `:2010`, `:2037`, `:2090`,
    /// `:2172`).
    pub(crate) fn missing_annotation_target(kind: &str, name: &[u8]) -> Raised {
        Raised::syntax(99, 945, vec![kind.as_bytes().to_vec(), name.to_vec()])
    }

    /// 16.1: `SIGNAL`/`SIGNAL VALUE` named a target that matches no label in
    /// the running activation's own body. `name` is the resolved target's
    /// own bytes -- already upcased for a bare symbol, verbatim for a quoted
    /// literal or a `SIGNAL VALUE` expression's rendered text.
    pub(crate) fn label_not_found(name: &[u8]) -> Raised {
        Raised::syntax(16, 1, vec![name.to_vec()])
    }

    /// 17.1: a `PROCEDURE` that is not the first instruction executed after
    /// an internal `CALL` or function invocation. No substitutions.
    pub(crate) fn procedure_out_of_place() -> Raised {
        Raised::syntax(17, 1, Vec::new())
    }

    /// 40.3: `USE STRICT ARG` with fewer arguments than it has targets
    /// without defaults. `routine` is the callee's own resolved name, upcased,
    /// and `minimum` counts the targets that must be supplied.
    pub(crate) fn not_enough_arguments(routine: &[u8], minimum: usize) -> Raised {
        Raised::syntax(
            40,
            3,
            vec![routine.to_vec(), minimum.to_string().into_bytes()],
        )
    }

    /// 40.4: `USE STRICT ARG` with more arguments than it has targets, and
    /// no trailing `...`.
    pub(crate) fn too_many_arguments(routine: &[u8], maximum: usize) -> Raised {
        Raised::syntax(
            40,
            4,
            vec![routine.to_vec(), maximum.to_string().into_bytes()],
        )
    }

    /// 40.5: a required argument was **omitted in place** rather than left
    /// off the end -- `substr('abc',,2)`. `routine` is the callee's own name,
    /// upcased, and `position` is 1-based.
    pub(crate) fn missing_argument(routine: &[u8], position: usize) -> Raised {
        Raised::syntax(
            40,
            5,
            vec![routine.to_vec(), position.to_string().into_bytes()],
        )
    }

    /// 40.12: a builtin argument that has to be a whole number is not one.
    /// `routine` is the builtin's own name as its table row spells it,
    /// `position` is 1-based **in the call's own argument list**, and `found`
    /// is the argument's own **rendered value**.
    /// ```text
    /// numeric digits 3 ; zz = 2 / 3 ; numeric digits 9 ; say left('ab', zz)
    /// ->  40.12  LEFT argument 2 must be a whole number; found "0.667".
    /// ```
    pub(crate) fn argument_not_whole(routine: &[u8], position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            40,
            12,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 40.23: a builtin's pad argument is not exactly one character.
    /// Substitutions as [`argument_not_whole`]'s, and `found` is the
    /// rendered value for the same measured reason.
    pub(crate) fn argument_not_a_pad(routine: &[u8], position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            40,
            23,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 40.26: an argument that must classify as a Rexx symbol (a valid
    /// variable name, a stem, a compound, or a numeric/literal constant)
    /// does not, or classifies as a constant while the call also supplies a
    /// new value for it. `routine` and `position` as [`argument_not_whole`]'s;
    /// `found` is the argument's **own bytes, verbatim -- never upcased**.
    /// ```text
    /// value('*')      Error 40.26:  VALUE argument 1 must be a valid symbol; found "*".
    /// value('5','x')  Error 40.26:  VALUE argument 1 must be a valid symbol; found "5".
    /// ```
    pub(crate) fn argument_not_a_symbol(routine: &[u8], position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            40,
            26,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 40.14: a builtin's argument converted to a whole number but is not
    /// strictly positive. Substituted as the same three slots -- routine,
    /// position, found -- as [`argument_not_whole`]'s.
    pub(crate) fn argument_not_positive(routine: &[u8], position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            40,
            14,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 40.34: `SOURCELINE`'s line number is past the end of the program.
    /// The message names the routine itself, so the only substitutions are
    /// the requested line and the program's own line count.
    pub(crate) fn sourceline_out_of_range(requested: &[u8], lines: usize) -> Raised {
        Raised::syntax(
            40,
            34,
            vec![requested.to_vec(), lines.to_string().into_bytes()],
        )
    }

    /// 40.903: a builtin's argument is outside the fixed range 0-99, which
    /// is the only range this catalogue entry can name -- the bounds are in
    /// the message text, not substituted.
    pub(crate) fn argument_out_of_range(routine: &[u8], position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            40,
            903,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 40.904: a builtin's option argument is not one of the letters that
    /// builtin accepts. `valid` is substituted verbatim, so its own quoting
    /// is the caller's to supply.
    pub(crate) fn argument_not_in_list(
        routine: &[u8],
        position: usize,
        valid: &str,
        found: &[u8],
    ) -> Raised {
        Raised::syntax(
            40,
            904,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                valid.as_bytes().to_vec(),
                found.to_vec(),
            ],
        )
    }

    /// 40.19: `DATE`/`TIME`'s input-conversion argument does not parse under
    /// the input style it was given, or parses but is out of range for it
    /// (`Error_Incorrect_call_format_invalid`, `expression/
    /// BuiltinFunctions.cpp`, both builtins' own `indate`/`intime` blocks).
    /// `found` is the argument's own rendered text and `style` the single
    /// upcased input-style byte, never the whole option string -- both
    /// measured, rc 216: `date(,'','f')` gives `DATE argument 2, "", is not
    /// in the format described by argument 3, "F".` and `date('D','367',
    /// 'D')` (2026 is not a leap year) gives the same shape naming `"D"`.
    pub(crate) fn date_format_invalid(routine: &[u8], found: &[u8], style: u8) -> Raised {
        Raised::syntax(40, 19, vec![routine.to_vec(), found.to_vec(), vec![style]])
    }

    /// 40.29: `TIME`'s elapsed-time output styles (`E`/`R`) refuse an input
    /// conversion argument outright, before the input is even parsed.
    pub(crate) fn invalid_conversion(routine: &[u8], style: u8) -> Raised {
        Raised::syntax(40, 29, vec![routine.to_vec(), vec![style]])
    }

    /// 40.43: a `DATE`/`TIME` separator argument is not exactly one
    /// non-alphanumeric byte and not the null string either.
    pub(crate) fn separator_not_a_char(routine: &[u8], position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            40,
            43,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 40.44: a `DATE`/`TIME` argument's own format is incompatible with a
    /// separator supplied elsewhere in the call. Three call sites share this
    /// one shape, each measured:
    /// ```text
    /// date('b',,,'')                    DATE argument 1, "B", is a format incompatible
    ///                                    with the separator specified in argument 4.
    /// date(,'20070922','w',,'-')        DATE argument 3, "W", is a format incompatible
    ///                                    with the separator specified in argument 5.
    /// date(,'1 May 2022',,,'-')         DATE argument 2, "1 May 2022", is a format
    ///                                    incompatible with the separator specified in
    ///                                    argument 5.
    /// ```
    pub(crate) fn format_incompatible_separator(
        routine: &[u8],
        position: usize,
        value: &[u8],
        other_position: usize,
    ) -> Raised {
        Raised::syntax(
            40,
            44,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                value.to_vec(),
                other_position.to_string().into_bytes(),
            ],
        )
    }

    /// 93.923: a length argument converted to a whole number but is
    /// negative. No routine name and no position in the message, only the
    /// value.
    pub(crate) fn invalid_length(found: &[u8]) -> Raised {
        Raised::syntax(93, 923, vec![found.to_vec()])
    }

    /// 93.924: a position argument converted to a whole number but is zero
    /// or negative. Same shape and same rc 163 as [`invalid_length`], and
    /// `found` is likewise the converted value -- measured,
    /// `substr('abc','0.0')` reports `found "0"`.
    pub(crate) fn invalid_position(found: &[u8]) -> Raised {
        Raised::syntax(93, 924, vec![found.to_vec()])
    }

    /// 93.922: a method's pad argument is a string that is not exactly one
    /// byte. `found` is the argument's own rendered bytes, at the same rc 163
    /// as [`invalid_length`] and [`invalid_position`] -- measured,
    /// `.MutableBuffer~new('abcabc')~substr(1, 2, 'xx')` reports `Incorrect
    /// pad or character argument specified; found "xx".`, and `''` and `12`
    /// report `found ""` and `found "12"`.
    pub(crate) fn incorrect_pad(found: &[u8]) -> Raised {
        Raised::syntax(93, 922, vec![found.to_vec()])
    }

    /// 93.906: a count argument converted to a whole number but is negative.
    /// `position` is 1-based **in the underlying method's argument list**,
    /// which is the builtin's own list less the positions the method takes
    /// as its receiver and its earlier operands -- measured,
    /// `copies('ab',-1)` reports `Method argument 1`, `insert('-','abc',-1)`
    /// reports `Method argument 2` and `changestr('a','banana','X',-1)`
    /// reports `Method argument 3`, from builtin positions 2, 3 and 4.
    pub(crate) fn argument_not_non_negative(position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            93,
            906,
            vec![position.to_string().into_bytes(), found.to_vec()],
        )
    }

    /// 93.915: an option argument's first letter is not one of the ones the
    /// builtin accepts. `valid` is the accepted set as the oracle spells it
    /// in the message, and `found` is the **whole option string**, not the
    /// letter that was rejected.
    pub(crate) fn invalid_option(valid: &str, found: &[u8]) -> Raised {
        Raised::syntax(93, 915, vec![valid.as_bytes().to_vec(), found.to_vec()])
    }

    /// 5: a result string too large to allocate. No sub-number and no
    /// substitution, which is why this is the one raiser here built with a
    /// sub of `0`: measured, `say left('ab','999999999999999999')` prints
    /// ```text
    ///      1 *-* say left('ab','999999999999999999')
    /// Error 5 running /abs/p.rex line 1:  System resources exhausted.
    /// ```
    pub(crate) fn system_resources() -> Raised {
        Raised::syntax(5, 0, Vec::new())
    }

    /// 40.28: an argument that has to be either a character class name or a
    /// single character is neither. Substitutions as [`argument_not_whole`]'s.
    pub(crate) fn argument_not_a_pad_or_class_name(
        routine: &[u8],
        position: usize,
        found: &[u8],
    ) -> Raised {
        Raised::syntax(
            40,
            28,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 93.927: `D2X`/`D2C` were asked to convert a negative value without a
    /// length to hold the sign extension. No substitutions.
    pub(crate) fn length_required_for_negative() -> Raised {
        Raised::syntax(93, 927, Vec::new())
    }

    /// 93.928: `D2X`'s value argument is not a whole number the current
    /// `NUMERIC DIGITS` can hold. `found` is the argument's own **rendered
    /// value**, which is the pair with [`argument_not_whole`]'s measurement:
    /// `numeric digits 3 ; zz = 2 / 3 ; numeric digits 9 ; say d2x(zz)`
    /// reports `found "0.667"`.
    pub(crate) fn d2x_value_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(93, 928, vec![found.to_vec()])
    }

    /// 93.929: [`d2x_value_not_whole`]'s twin for `D2C`, measured to be the
    /// same rule with a different number -- `d2c('abc')` and `d2x('abc')`
    /// differ only in the sub-code and the routine the text names.
    pub(crate) fn d2c_value_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(93, 929, vec![found.to_vec()])
    }

    /// 93.935: `X2D`'s *result* does not fit the current `NUMERIC DIGITS`.
    /// The substitution is the setting itself, not the value.
    pub(crate) fn x2d_result_too_large(digits: u64) -> Raised {
        Raised::syntax(93, 935, vec![digits.to_string().into_bytes()])
    }

    /// 93.936: [`x2d_result_too_large`]'s twin for `C2D`.
    pub(crate) fn c2d_result_too_large(digits: u64) -> Raised {
        Raised::syntax(93, 936, vec![digits.to_string().into_bytes()])
    }

    /// 93.931/93.932: a hexadecimal or binary string carries whitespace where
    /// it may not -- at the very start, or at the very end. `position` is
    /// 1-based.
    pub(crate) fn misplaced_whitespace(notation: Notation, position: usize) -> Raised {
        let sub = match notation {
            Notation::Hex => 931,
            Notation::Binary => 932,
        };
        Raised::syntax(93, sub, vec![position.to_string().into_bytes()])
    }

    /// 93.933/93.934: a byte that is neither a digit of the notation nor one
    /// of the two bytes that may separate its groups. The substitution is the
    /// offending byte itself.
    pub(crate) fn invalid_digit(notation: Notation, character: u8) -> Raised {
        let sub = match notation {
            Notation::Hex => 933,
            Notation::Binary => 934,
        };
        Raised::syntax(93, sub, vec![vec![character]])
    }

    /// 93.976/93.977: the groups of a hexadecimal or binary string are not
    /// sized as the notation requires. No substitutions.
    pub(crate) fn invalid_grouping(notation: Notation) -> Raised {
        let sub = match notation {
            Notation::Hex => 976,
            Notation::Binary => 977,
        };
        Raised::syntax(93, sub, Vec::new())
    }

    /// 40.13: `RANDOM`'s seed is negative. `routine` is the builtin's own
    /// name, `position` is 1-based in the call's argument list, and `found`
    /// is the argument's rendered value.
    pub(crate) fn argument_not_non_negative_call(
        routine: &[u8],
        position: usize,
        found: &[u8],
    ) -> Raised {
        Raised::syntax(
            40,
            13,
            vec![
                routine.to_vec(),
                position.to_string().into_bytes(),
                found.to_vec(),
            ],
        )
    }

    /// 40.32: `RANDOM`'s range is wider than the generator's own limit of
    /// 999,999,999. The two substitutions are the *arguments as written*,
    /// and an omitted one is the null string.
    pub(crate) fn random_range_too_wide(minimum: &[u8], maximum: &[u8]) -> Raised {
        Raised::syntax(40, 32, vec![minimum.to_vec(), maximum.to_vec()])
    }

    /// 40.33: `RANDOM`'s minimum is above its maximum. Substitutions as
    /// [`random_range_too_wide`]'s, including the null string for an omitted
    /// argument -- measured, `random(-1)` reports `argument 1 ("-1")` and
    /// `argument 2 ("")`.
    pub(crate) fn random_bounds_reversed(minimum: &[u8], maximum: &[u8]) -> Raised {
        Raised::syntax(40, 33, vec![minimum.to_vec(), maximum.to_vec()])
    }

    /// 93.903: a `MAX`/`MIN` argument was omitted in place on the path where
    /// the target is an integer object. **`position` is 0-based**, which is
    /// measured rather than mistranscribed: `max(1,,3)` reports `argument 0`,
    /// `max(1,2,,4)` reports `argument 1` and `max(1,2,3,,5)` reports
    /// `argument 2`.
    pub(crate) fn missing_method_argument(position: usize) -> Raised {
        Raised::syntax(93, 903, vec![position.to_string().into_bytes()])
    }

    /// 93.952: a method source array holds something that is not a string.
    pub(crate) fn method_source_not_all_strings(position: &str) -> Raised {
        Raised::syntax(93, 952, vec![position.as_bytes().to_vec()])
    }

    /// 93.902: a message send passed more arguments than the method takes.
    /// `arity` is the count the method **declares**, not the count that
    /// arrived -- measured, `'abc'~length(1)` reports `0 expected` and
    /// `'abc'~hasMethod('a','b')` reports `1 expected`, both at rc 163.
    pub(crate) fn too_many_method_arguments(arity: usize) -> Raised {
        Raised::syntax(93, 902, vec![arity.to_string().into_bytes()])
    }

    /// 93.953: a method argument that has to be one of a set of classes is
    /// none of them. `wanted` is the description the raise site supplies.
    pub(crate) fn argument_not_convertible(position: usize, wanted: &str) -> Raised {
        Raised::syntax(
            93,
            953,
            vec![
                position.to_string().into_bytes(),
                wanted.as_bytes().to_vec(),
            ],
        )
    }

    /// 93.900: `Error_Incorrect_method_user_defined`, whose whole message the
    /// raise site supplies.
    pub(crate) fn method_user_defined(text: &str) -> Raised {
        Raised::syntax(93, 900, vec![text.as_bytes().to_vec()])
    }

    /// 93.905: a method argument that has to be a whole number is not one.
    pub(crate) fn method_argument_not_whole(position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            93,
            905,
            vec![position.to_string().into_bytes(), found.to_vec()],
        )
    }

    /// 93.965: a message resolved to an `ABSTRACT` method.
    pub(crate) fn abstract_method(name: &[u8]) -> Raised {
        Raised::syntax(93, 965, vec![name.to_vec()])
    }

    /// 93.901: a method was given fewer arguments than it needs.
    /// `expected` is the count the oracle names.
    pub(crate) fn not_enough_method_arguments(expected: usize) -> Raised {
        Raised::syntax(93, 901, vec![expected.to_string().into_bytes()])
    }

    /// 93.966: a `Queue` insertion or replacement index is past its last
    /// item.
    pub(crate) fn incorrect_queue_index(position: usize) -> Raised {
        Raised::syntax(93, 966, vec![position.to_string().into_bytes()])
    }

    /// 93.967: a class whose instances come only from native code was sent
    /// `NEW`.
    pub(crate) fn unsupported_new_method(id: &[u8]) -> Raised {
        Raised::syntax(93, 967, vec![id.to_vec()])
    }

    /// 93.918: a `Queue` index names a position it does not hold.
    pub(crate) fn incorrect_list_index(index: &[u8]) -> Raised {
        Raised::syntax(93, 918, vec![index.to_vec()])
    }

    /// 98.975: a sort was asked to order an array with a hole in it.
    pub(crate) fn missing_array_element(position: usize) -> Raised {
        Raised::syntax(98, 975, vec![position.to_string().into_bytes()])
    }

    /// 93.954: a method that only works on a single-dimensional array was
    /// sent to one with more.
    pub(crate) fn single_dimension_only(method: &str) -> Raised {
        Raised::syntax(93, 954, vec![method.as_bytes().to_vec()])
    }

    /// 93.949: a `Set` or a `Bag` was given an index that is not its value.
    pub(crate) fn index_does_not_match() -> Raised {
        Raised::syntax(93, 949, Vec::new())
    }

    /// 26.903: a `COMPARE` method answered something that is not a whole
    /// number.
    pub(crate) fn compare_result_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(26, 903, vec![found.to_vec()])
    }

    /// 26.902: a `COMPARETO` method answered something that is not a whole
    /// number.
    pub(crate) fn compare_to_result_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(26, 902, vec![found.to_vec()])
    }

    /// 93.937: a `Supplier` was asked for a pair it no longer has.
    pub(crate) fn no_more_supplier_items() -> Raised {
        Raised::syntax(93, 937, Vec::new())
    }

    /// 93.926: an array subscript list is longer than the array's dimension.
    pub(crate) fn too_many_subscripts(expected: usize) -> Raised {
        Raised::syntax(93, 926, vec![expected.to_string().into_bytes()])
    }

    /// 93.925: an array subscript list is shorter than the array's dimension.
    pub(crate) fn not_enough_subscripts(expected: usize) -> Raised {
        Raised::syntax(93, 925, vec![expected.to_string().into_bytes()])
    }

    /// 93.959: an array size or dimension product past
    /// `ArrayClass::MaxFixedArraySize`. `max` is that bound, which the
    /// message names.
    pub(crate) fn array_too_big(max: usize) -> Raised {
        Raised::syntax(93, 959, vec![max.to_string().into_bytes()])
    }

    /// 93.907: a method argument is not a positive whole number.
    /// `position` is 1-based and `found` the argument's own rendered bytes.
    pub(crate) fn method_argument_not_positive(position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            93,
            907,
            vec![position.to_string().into_bytes(), found.to_vec()],
        )
    }

    /// 88.914: a `target~name:scope` override whose scope expression did not
    /// evaluate to a class object.
    pub(crate) fn scope_override_not_a_class() -> Raised {
        Raised::syntax(88, 914, vec![b"SCOPE".to_vec(), b"Class".to_vec()])
    }

    /// 93.957: a `target~name:scope` override whose scope is a class object
    /// the receiver's own behaviour was never given.
    pub(crate) fn scope_override_not_a_scope(target: &[u8], scope: &[u8]) -> Raised {
        Raised::syntax(93, 957, vec![target.to_vec(), scope.to_vec()])
    }

    /// 88.914: an argument the method requires to be a class object is not
    /// one. `argument` is the name the raise site substitutes.
    pub(crate) fn argument_not_a_class(argument: &str) -> Raised {
        Raised::syntax(
            88,
            914,
            vec![argument.as_bytes().to_vec(), b"Class".to_vec()],
        )
    }

    /// 88.914 for a class other than `.Class`: `classArgument(routine,
    /// TheRoutineClass, "routine")` and its neighbours, which substitute the
    /// argument's name and the required class's id.
    pub(crate) fn argument_not_an_instance(argument: &str, class: &str) -> Raised {
        Raised::syntax(
            88,
            914,
            vec![argument.as_bytes().to_vec(), class.as_bytes().to_vec()],
        )
    }

    /// 93.914: a method argument that has to name one of a list --
    /// `Error_Incorrect_method_list`. It substitutes the argument's position,
    /// the list as the raise site spells it (its own quoting included), and
    /// the value found.
    pub(crate) fn method_argument_not_in_list(position: usize, list: &str, found: &[u8]) -> Raised {
        Raised::syntax(
            93,
            914,
            vec![
                position.to_string().into_bytes(),
                list.as_bytes().to_vec(),
                found.to_vec(),
            ],
        )
    }

    /// 88.901: a method argument the oracle names rather than numbers was
    /// omitted.
    pub(crate) fn missing_named_argument(argument: &str) -> Raised {
        Raised::syntax(88, 901, vec![argument.as_bytes().to_vec()])
    }

    /// 88.909: a method argument has no string value. `position` is
    /// 1-based in the method's own argument list.
    pub(crate) fn argument_needs_a_string_value(position: usize) -> Raised {
        Raised::syntax(88, 909, vec![position.to_string().into_bytes()])
    }

    /// 88.909 for an argument the oracle names rather than numbers.
    pub(crate) fn named_argument_needs_a_string_value(argument: &str) -> Raised {
        Raised::syntax(88, 909, vec![argument.as_bytes().to_vec()])
    }

    /// 88.910: a pad argument the oracle names rather than numbers is a
    /// string that is not exactly one byte. `found` is the argument's own
    /// rendered bytes.
    pub(crate) fn named_argument_invalid_pad(argument: &str, found: &[u8]) -> Raised {
        Raised::syntax(88, 910, vec![argument.as_bytes().to_vec(), found.to_vec()])
    }

    /// 88.911: a length argument the oracle names rather than numbers did not
    /// convert to a non-negative whole number. `found` is the argument's own
    /// rendered bytes.
    pub(crate) fn named_argument_invalid_length(argument: &str, found: &[u8]) -> Raised {
        Raised::syntax(88, 911, vec![argument.as_bytes().to_vec(), found.to_vec()])
    }

    /// 88.912: a position argument the oracle names rather than numbers did
    /// not convert to a positive whole number. `found` is the argument's own
    /// rendered bytes.
    pub(crate) fn named_argument_invalid_position(argument: &str, found: &[u8]) -> Raised {
        Raised::syntax(88, 912, vec![argument.as_bytes().to_vec(), found.to_vec()])
    }

    /// 93.915: a method's option argument is not one of the letters it
    /// accepts. `options` is the accepted set as the oracle spells it and
    /// `found` is the argument's own rendered bytes.
    pub(crate) fn method_option_not_recognised(options: &str, found: &[u8]) -> Raised {
        Raised::syntax(93, 915, vec![options.as_bytes().to_vec(), found.to_vec()])
    }

    /// 93.970: `Class~copy`, which the oracle answers for no class object.
    /// `object` is the receiver's own string value.
    pub(crate) fn copy_not_supported(object: &[u8]) -> Raised {
        Raised::syntax(93, 970, vec![object.to_vec()])
    }

    /// 93.972: a `~send`/`~start` message name that is neither a string nor
    /// an array. `found` is the value's own string value.
    pub(crate) fn message_name_shape(found: &[u8]) -> Raised {
        Raised::syntax(93, 972, vec![found.to_vec()])
    }

    /// 93.946: a `~send`/`~start` message name that is an array of any shape
    /// but a single dimension of two elements
    /// (`classes/ObjectClass.cpp:2143`-`:2146`). No substitution.
    pub(crate) fn message_array_shape() -> Raised {
        Raised::syntax(93, 946, vec![])
    }

    /// 97.1: the receiver's behaviour answers no method of that name.
    /// `target` is the receiver's own **string value** and `name` the
    /// message as the send spells it, already upcased by the parser.
    pub(crate) fn no_method(target: &[u8], name: &[u8]) -> Raised {
        Raised::syntax(97, 1, vec![target.to_vec(), name.to_vec()])
    }

    /// 97.2: the method is `PRIVATE` and `RexxObject::checkPrivate` refused
    /// the caller. The substitutions are [`Raised::no_method`]'s.
    pub(crate) fn private_method(target: &[u8], name: &[u8]) -> Raised {
        Raised::syntax(97, 2, vec![target.to_vec(), name.to_vec()])
    }

    /// 97.3: the method is `PACKAGE` and the caller is in another package.
    /// The substitutions are [`Raised::no_method`]'s.
    pub(crate) fn package_scope_method(target: &[u8], name: &[u8]) -> Raised {
        Raised::syntax(97, 3, vec![target.to_vec(), name.to_vec()])
    }

    /// 97.4: a `::CONSTANT` accessor whose expression form has not been
    /// evaluated yet. `target` is the receiver's own string value and `name`
    /// the constant, upcased as the directive installed it.
    pub(crate) fn constant_not_initialized(target: &[u8], name: &[u8]) -> Raised {
        Raised::syntax(97, 4, vec![target.to_vec(), name.to_vec()])
    }

    /// The `NOMETHOD` condition a dispatch miss raises, whose untrapped
    /// rendering is [`Raised::no_method`]'s own 97.1.
    /// ```text
    ///                C           D            E    RC
    /// nomethod trap  NOMETHOD    NOSUCHMSG    ''   untouched
    /// syntax   trap  SYNTAX      ''           1    97
    /// ```
    pub(crate) fn nomethod(report: Raised, name: &[u8]) -> Raised {
        Raised {
            condition: Cow::Borrowed("NOMETHOD"),
            rc: None,
            description: Some(name.to_vec()),
            ..report
        }
    }

    /// The `NOSTRING` condition the required-string protocol raises when the
    /// receiver has no string value and something is armed to take it.
    pub(crate) fn nostring(readable: &[u8]) -> Raised {
        Raised {
            description: Some(readable.to_vec()),
            ..Raised::condition(Cow::Borrowed("NOSTRING"))
        }
    }

    /// 98.973, what `::OPTIONS NOSTRING SYNTAX` turns an untrapped NOSTRING
    /// into (`Activity::raiseCondition`, `concurrency/Activity.cpp:615`).
    pub(crate) fn nostring_syntax(readable: &[u8]) -> Raised {
        Raised::syntax(98, 973, vec![readable.to_vec()])
    }

    /// 98.986, what `::OPTIONS NOVALUE SYNTAX` turns an untrapped NOVALUE
    /// into (`RexxActivation::handleNovalueEvent`,
    /// `execution/RexxActivation.cpp:2648`).
    pub(crate) fn unassigned_variable(name: &[u8]) -> Raised {
        Raised::syntax(98, 986, vec![name.to_vec()])
    }

    /// 91.999: a message used where a value was wanted returned none.
    /// `name` is the message as the send spells it, already upcased.
    pub(crate) fn no_result(name: &[u8]) -> Raised {
        Raised::syntax(91, 999, vec![name.to_vec()])
    }

    /// The traceback line a method activation contributes when its package
    /// carries no source -- `RexxActivation::formatSourcelessTraceLine`'s
    /// `isMethod()` arm (`execution/RexxActivation.cpp:5057`), reached from
    /// `PackageClass::traceBack` (`classes/PackageClass.cpp:589`) when
    /// `source->extract` answers nothing.
    pub(crate) fn sourceless_method_line(name: &[u8], scope: &str, package: &[u8]) -> Vec<u8> {
        let substitutions = vec![name.to_vec(), scope.as_bytes().to_vec(), package.to_vec()];
        match rexx_inventory::errors::lookup(101, 24) {
            Some(entry) => substitute(entry.text, &substitutions),
            None => b"<no message 101.24 in the catalogue>".to_vec(),
        }
    }

    /// The same for an activation that is a whole program rather than a
    /// method -- `formatSourcelessTraceLine`'s `else` arm.
    pub(crate) fn sourceless_program_line(package: &[u8]) -> Vec<u8> {
        match rexx_inventory::errors::lookup(101, 26) {
            Some(entry) => substitute(entry.text, &[package.to_vec()]),
            None => b"<no message 101.26 in the catalogue>".to_vec(),
        }
    }

    /// The traceback line a native (C++-implemented, here Rust-implemented)
    /// method activation contributes, rendered whole.
    pub(crate) fn compiled_method_line(name: &[u8], scope: &str) -> Vec<u8> {
        let substitutions = vec![name.to_vec(), scope.as_bytes().to_vec()];
        match rexx_inventory::errors::lookup(101, 20) {
            Some(entry) => substitute(entry.text, &substitutions),
            None => b"<no message 101.20 in the catalogue>".to_vec(),
        }
    }

    /// 93.904: a `MAX`/`MIN` argument is not a number. `position` is 1-based
    /// **in the underlying method's argument list**, so it is one lower than
    /// the call's own numbering -- measured, `max(1,'a',3)` reports `Method
    /// argument 1` for the call's argument 2, and `max(1,2,'a')` reports
    /// `Method argument 2`.
    pub(crate) fn method_argument_not_a_number(position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            93,
            904,
            vec![position.to_string().into_bytes(), found.to_vec()],
        )
    }

    /// 93.943: the value a numeric builtin was handed as its *target* is not
    /// a number. `method` is the name the message uses, which is the
    /// builtin's own; `found` is the value's rendered bytes.
    pub(crate) fn method_target_not_a_number(method: &[u8], found: &[u8]) -> Raised {
        Raised::syntax(93, 943, vec![method.to_vec(), found.to_vec()])
    }

    /// 93.940: `~MODULO`'s target is a number but not a whole one.
    pub(crate) fn method_target_not_whole(method: &[u8], found: &[u8]) -> Raised {
        Raised::syntax(93, 940, vec![method.to_vec(), found.to_vec()])
    }

    /// 93.962: `~decodeBase64`'s receiver is not a Base64 encoding. No
    /// substitutions -- the message names neither the method nor the value.
    pub(crate) fn invalid_base64() -> Raised {
        Raised::syntax(93, 962, Vec::new())
    }

    /// 88.928: `USE ARG >name` where the caller did not pass a variable
    /// reference. `position` is 1-based; `found` is the argument's own
    /// **rendered value**.
    pub(crate) fn not_a_variable_reference(position: usize, found: &[u8]) -> Raised {
        Raised::syntax(
            88,
            928,
            vec![position.to_string().into_bytes(), found.to_vec()],
        )
    }

    /// 88.931: `USE ARG >name` where the caller omitted that position
    /// entirely. `position` is 1-based.
    pub(crate) fn variable_reference_omitted(position: usize) -> Raised {
        Raised::syntax(88, 931, vec![position.to_string().into_bytes()])
    }

    /// 88.929: `USE ARG >name` where the target is a **stem** and the caller
    /// passed a reference to a **simple** variable.
    pub(crate) fn not_a_stem_variable_reference(position: usize, reference: &[u8]) -> Raised {
        Raised::syntax(
            88,
            929,
            vec![position.to_string().into_bytes(), reference.to_vec()],
        )
    }

    /// 88.930: the mirror of [`not_a_stem_variable_reference`] -- the target
    /// is a **simple** variable and the caller passed a **stem** reference.
    pub(crate) fn not_a_simple_variable_reference(position: usize, reference: &[u8]) -> Raised {
        Raised::syntax(
            88,
            930,
            vec![position.to_string().into_bytes(), reference.to_vec()],
        )
    }

    /// 98.995: `USE ARG >name` whose target is not currently unset. `name` is
    /// the target's own spelling.
    pub(crate) fn variable_reference_not_uninitialised(name: &[u8]) -> Raised {
        Raised::syntax(98, 995, vec![name.to_vec()])
    }

    /// 98.992: `EXPOSE` outside a method invocation. No substitutions.
    pub(crate) fn expose_outside_method() -> Raised {
        Raised::syntax(98, 992, Vec::new())
    }

    /// 99.911: `GUARD` outside a method invocation. No substitutions.
    pub(crate) fn guard_outside_method() -> Raised {
        Raised::syntax(99, 911, Vec::new())
    }

    /// 99.919: `REPLY` outside a method invocation. No substitutions.
    pub(crate) fn reply_outside_method() -> Raised {
        Raised::syntax(99, 919, Vec::new())
    }

    /// 98.947: `FORWARD` outside a method invocation. No substitutions.
    pub(crate) fn forward_outside_method() -> Raised {
        Raised::syntax(98, 947, Vec::new())
    }

    /// 98.946: a `FORWARD ARGUMENTS` value `requestArray` cannot answer as a
    /// single-dimensional array. No substitutions.
    pub(crate) fn forward_arguments() -> Raised {
        Raised::syntax(98, 946, Vec::new())
    }

    /// 98.935: a second `REPLY` in one method invocation. No substitutions.
    pub(crate) fn reply_twice() -> Raised {
        Raised::syntax(98, 935, Vec::new())
    }

    /// 98.936: `RETURN` with a value after a `REPLY`. No substitutions.
    pub(crate) fn return_after_reply() -> Raised {
        Raised::syntax(98, 936, Vec::new())
    }

    /// 98.937: `EXIT` with a value after a `REPLY`. No substitutions.
    pub(crate) fn exit_after_reply() -> Raised {
        Raised::syntax(98, 937, Vec::new())
    }

    /// 98.993: `USE LOCAL` as the first instruction executed of a top-level
    /// program. No substitutions.
    pub(crate) fn use_local_outside_method() -> Raised {
        Raised::syntax(98, 993, Vec::new())
    }

    /// 99.910: `USE LOCAL` anywhere other than the first instruction executed
    /// of a top-level program. No substitutions.
    pub(crate) fn use_local_not_first() -> Raised {
        Raised::syntax(99, 910, Vec::new())
    }

    /// 29.1: an `ADDRESS` environment name longer than the platform's limit.
    pub(crate) fn environment_name_too_long(limit: usize, found: &[u8]) -> Raised {
        Raised::syntax(29, 1, vec![limit.to_string().into_bytes(), found.to_vec()])
    }

    /// 88.913: a method argument the oracle names is not a single-dimensional
    /// array. The substitution is that name.
    pub(crate) fn argument_not_single_dimensional(argument: &str) -> Raised {
        Raised::syntax(88, 913, vec![argument.as_bytes().to_vec()])
    }

    /// 98.913: an argument the oracle numbers rather than names is not a
    /// single-dimensional array. The substitution is the value's own string
    /// value.
    pub(crate) fn object_not_single_dimensional(found: &[u8]) -> Raised {
        Raised::syntax(98, 913, vec![found.to_vec()])
    }
}

/// Converts a `rexx-num` arithmetic failure into a `Raised`.
impl From<ArithError> for Raised {
    fn from(error: ArithError) -> Raised {
        // `additional()` and `sub_code()` both borrow, so either can run
        // first; ordered to match the doc comment's own telling.
        let additional = error.additional();
        let (number, sub) = error.sub_code();
        Raised::syntax(number, sub, into_substitutions(additional))
    }
}

/// Converts a `rexx-num` `FORMAT` failure into a `Raised`, the same way
/// [`From<ArithError>`] converts an arithmetic one.
impl From<FormatError> for Raised {
    fn from(error: FormatError) -> Raised {
        let additional = error.additional();
        let (number, sub) = error.sub_code();
        Raised::syntax(number, sub, into_substitutions(additional))
    }
}

/// Converts a `rexx-parse` translation failure into the condition the oracle
/// raises for it.
/// ```text
///      2 *-* do forever then
///      2 *-* interpret "do forever then"
/// Error 27 running /abs/p1.rex line 2:  Invalid DO or LOOP syntax.
/// Error 27.901:  Incorrect data following FOREVER keyword on the loop; found "THEN".
/// ```
impl From<&ParseError> for Raised {
    fn from(error: &ParseError) -> Raised {
        Raised::syntax(error.code, error.sub, Vec::new())
    }
}

/// What a clause can produce instead of a value: a construct this crate does
/// not implement (`Loud`), a real Rexx condition (`Raised`), an `EXIT`
/// travelling through an expression (`Exited`), or a harness bound the run
/// outlived (`Deadline`). `step` and everything above it propagate this
/// rather than any one alone, since a clause containing an expression can
/// fail more than one way -- `eval`'s own `ExprKind::Message` arm is `Loud`
/// (not implemented, Phase 5's), its `1 / 0` arm is `Raised` (implemented,
/// and this is what it does).
#[derive(Debug)]
pub(crate) enum Failure {
    /// **Boxed, and that is a measurement rather than a habit.** This variant
    /// is what sets `Failure`'s width, and `Failure` is the error half of
    /// `Result<ObjRef, Failure>`, the shape an expression answers in. Unboxed,
    /// `Loud` is 24 bytes and that `Result` is 24, which travels through a
    /// stack slot; boxed, both are 16, which is two registers. Measured with
    /// `perf stat -e instructions:u` over `bench-programs`: `strings` -1.00%,
    /// `varlookup` -0.77%, `emptyloop` -0.62%, the pinned `rexxcps` -0.58%,
    /// `compound` -0.54%, `arith` -0.18% -- one direction on every axis, which
    /// is what separates this from the +-0.5% a relayout moves things by.
    Loud(Box<Loud>),
    Raised(Box<Raised>),
    /// **Not a failure at all** -- `EXIT` inside a routine reached through
    /// `ExprKind::Call`'s expression form (Task 4), or that routine falling
    /// off its own end, either of which ends the whole program exactly as
    /// the same event does when reached through `CALL` (`resolve_and_run_
    /// call`'s own doc, `run.rs`). `CALL`'s own instruction form carries
    /// this through `Flow::Exit`/`Ended::Exited` instead, entirely through
    /// `Ok` returns, because `step` and `run_activation` both return a
    /// `Flow`/`Ended` that has room for "the program is exiting" as a
    /// successful outcome. `eval`'s own return type is a plain `ObjRef`, with
    /// no such room, so this variant is what lets the same event travel
    /// through an expression instead: constructed once, in `eval_call`
    /// (`eval.rs`), and then propagated by every intervening `?` completely
    /// unremarked -- `Op::Clause`'s region's and `Interp::invoke_call`'s own
    /// generic "an `Err` escaped, record a site and re-throw" paths do not
    /// need to know this variant exists, because sealing a site nothing
    /// prints is harmless (`execute`, `lib.rs`, never calls `Raised::report`
    /// for it) and re-throwing is exactly what unwinding every nested `CALL`
    /// to end the whole program needs regardless of how many levels deep the
    /// `EXIT` was. `execute`'s own top-level match is the one place this is
    /// finally read, and there it is handled exactly like an ordinary
    /// `Ok(value)`: same `exit_code_for`, no stderr report.
    Exited(Option<ObjRef>),
    /// The run outlived the deadline its [`Invocation`](crate::Invocation)
    /// set, and is being abandoned at a clause boundary
    /// ([`Deadline`](crate::clause::Deadline), whose own doc has what that
    /// bound does and does not reach).
    Deadline,
}

impl From<Loud> for Failure {
    fn from(loud: Loud) -> Failure {
        Failure::Loud(Box::new(loud))
    }
}

impl From<Raised> for Failure {
    fn from(raised: Raised) -> Failure {
        Failure::Raised(Box::new(raised))
    }
}

/// Where a failing clause was found -- `Interp::failure_site`'s own type
/// (`lib.rs`), and what `run.rs`'s `record_failure_site` fills in.
#[derive(Clone)]
pub(crate) enum FailureSite {
    /// A level whose failing clause is source text, echoed under its own
    /// line number.
    Clause {
        line: usize,
        text: Vec<u8>,
        /// Spaces to prefix `text` with on the echo line, Task 11's own
        /// nesting-depth quantity. **Computed statically from the AST**
        /// (`run.rs`'s `static_indent`), never carried on a running counter:
        /// Task 10's own report concluded the depth is derivable from the
        /// instruction list alone with no runtime block stack, and this
        /// task's own oracle measurements confirm it for the ordinary case
        /// and for one LEAVE/ITERATE error family (28.5) besides -- see
        /// `static_indent`'s own doc comment and the report for the
        /// transcripts. A mutable per-`Interp` counter was the first design
        /// tried here and was abandoned once it became clear it would need
        /// perfect symmetric bookkeeping on every exit path out of every
        /// construct, including the error paths and the `run_bounded`
        /// `Goto`-absorption case `Flow`'s own doc comment warns about --
        /// exactly the class of defect this crate's skipped-`pop_frame`
        /// discussion elsewhere already flags. A pure function of
        /// `(instructions, index)` cannot desync, because there is nothing
        /// stateful to desync.
        indent: usize,
    },
    /// A level whose `running <name> line <n>` span names something other
    /// than the running program's path, `name` being what it names.
    Named {
        line: usize,
        indent: usize,
        text: Vec<u8>,
        name: Vec<u8>,
    },
    /// A level with no source clause of its own: a native method
    /// activation, whose whole echo line is a catalogue entry
    /// ([`Raised::compiled_method_line`]) carrying its own blank
    /// line-number field, `*-*` marker and text.
    Rendered(Vec<u8>),
}

impl FailureSite {
    /// The source line this site echoes under, or `None` for a site that has
    /// no clause of its own.
    pub(crate) fn line(&self) -> Option<usize> {
        match self {
            FailureSite::Clause { line, .. } | FailureSite::Named { line, .. } => Some(*line),
            FailureSite::Rendered(_) => None,
        }
    }

    /// What a [`FailureSite::Named`] reports in place of the program's path,
    /// or `None` for a site whose level reports that path.
    pub(crate) fn reported_name(&self) -> Option<&[u8]> {
        match self {
            FailureSite::Named { name, .. } => Some(name),
            FailureSite::Clause { .. } | FailureSite::Rendered(_) => None,
        }
    }

    /// The clause text a [`FailureSite::Clause`] echoes, or the whole
    /// rendered line of a [`FailureSite::Rendered`].
    #[cfg(test)]
    pub(crate) fn text(&self) -> &[u8] {
        match self {
            FailureSite::Clause { text, .. } | FailureSite::Named { text, .. } => text,
            FailureSite::Rendered(bytes) => bytes,
        }
    }

    /// The spaces a [`FailureSite::Clause`]'s text is prefixed with, or
    /// `None` for a rendered site, whose line carries its own leading blanks.
    #[cfg(test)]
    pub(crate) fn indent(&self) -> Option<usize> {
        match self {
            FailureSite::Clause { indent, .. } | FailureSite::Named { indent, .. } => Some(*indent),
            FailureSite::Rendered(_) => None,
        }
    }
}

/// Where the failing clause is, which is everything the report needs from
/// outside this module.
pub(crate) struct ClauseSite<'a> {
    /// The program's path **as the oracle prints it**, absolute. Measured:
    /// the major line carries the full path, and `rexx-oracle`'s `normalize`
    /// masks the cwd, so an absolute path is comparable across machines.
    pub(crate) path: &'a str,
    /// One entry per activation-like level the condition escaped through,
    /// **innermost first** -- 4b's Task 2, and the whole reason this is a
    /// slice rather than the single site 4a carried.
    /// ```text
    ///      2 *-* say 2 & 1
    ///      2 *-* interpret "say 2 & 1"
    /// ```
    pub(crate) sites: &'a [FailureSite],
}

impl Raised {
    /// `256 - major`, the whole rule.
    pub(crate) fn exit_code(&self) -> i32 {
        256 - i32::from(self.number)
    }

    /// The exact bytes the oracle writes to stderr for this condition.
    /// ```text
    ///      4 *-* end
    /// Error 7 running /abs/path/f.rex line 4:  WHEN or OTHERWISE expected.
    /// Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.
    /// ```
    pub(crate) fn report(&self, site: &ClauseSite<'_>) -> Vec<u8> {
        let mut out = Vec::new();
        // `trace::push_clause` rather than a second copy of the same four
        // lines: the two used to be written out separately and documented as
        // byte-identical, and 4b's Task 2 needed the 40-column clamp on both.
        // Calling the one formatter is what makes "one quantity" true in the
        // code rather than only in a comment -- `push_clause` owns the
        // clamp, the six-wide line field and the indent, and this loop owns
        // only the order.
        for entry in site.sites {
            match entry {
                FailureSite::Clause { line, text, indent }
                | FailureSite::Named {
                    line, text, indent, ..
                } => {
                    crate::trace::push_clause(&mut out, *line, *indent, text);
                }
                // Already a whole line, `*-*` marker and blank line-number
                // field included, straight from the catalogue.
                FailureSite::Rendered(bytes) => {
                    out.extend_from_slice(bytes);
                    out.push(b'\n');
                }
            }
        }
        // The innermost *clause* entry's line, or `0` when nothing was
        // recorded at all -- `execute`'s own guard already substitutes a
        // visible placeholder entry for that case, so this fallback is
        // unreachable from there and exists so this function has no panic on
        // the error path. A `Rendered` entry is skipped rather than counted:
        // it has no line of its own, and the oracle reports the sending
        // clause's line above it.
        // **The innermost line-bearing entry decides the name as well as the
        // number.** A frame in a package with no source reports that
        // package where a program reports its path -- measured, `say
        // .Validate~number('LENGTH', 'abc')` is `Error 88 running REXX line
        // 3700` on the oracle where the same failure in a program's own
        // clause names the program's file. A `Rendered` entry decides
        // neither, for the reason its own doc gives.
        let innermost = site.sites.iter().find(|entry| entry.line().is_some());
        let line = innermost.and_then(FailureSite::line).unwrap_or(0);
        let named = match innermost.and_then(FailureSite::reported_name) {
            Some(package) => String::from_utf8_lossy(package).into_owned(),
            None => site.path.to_string(),
        };
        let site = &ClauseSite {
            path: &named,
            sites: site.sites,
        };
        // `RAISE PROPAGATE` drops the position span and nothing else
        // (`Delivery::positionless`). Measured against the same program with
        // and without the `raise propagate`: the echo lines, the sub line and
        // the exit code are identical, and only ` running <path> line <n>`
        // goes.
        let position = if self.delivery.positionless {
            String::new()
        } else if self.delivery.lineless {
            format!(" running {}", site.path)
        } else {
            format!(" running {} line {line}", site.path)
        };
        out.extend_from_slice(format!("Error {}{}:  ", self.number, position).as_bytes());
        out.extend_from_slice(&self.message(self.number, 0));
        out.push(b'\n');
        // **Sub `0` prints no second line at all**, measured: `raise syntax
        // 40` gives the major line and stops, where `raise syntax 40.4`
        // gives both. Reachable only through `RAISE` -- every raiser in the
        // crate names a real sub -- which is why 4a never had to know.
        if self.sub != 0 {
            out.extend_from_slice(format!("Error {}.{}:  ", self.number, self.sub).as_bytes());
            out.extend_from_slice(&self.message(self.number, self.sub));
            out.push(b'\n');
        }
        // **Applied once, to the whole report, and that is the oracle's own
        // shape rather than a shortcut.** `Activity::display` sends each
        // traceback echo and each `Error ...` line through
        // `displayUsingTraceOutput`, which sanitises the line it is handed;
        // the rule is per byte and leaves `\n` alone, so sanitising the
        // concatenation is the same bytes as sanitising each line. It
        // therefore covers the clause echoes too, which carry the program's
        // own source and can hold any byte -- measured, a raw `0x01` inside a
        // source literal echoes as `?`.
        displayable(&mut out);
        out
    }

    /// One catalogue entry with this error's substitutions applied.
    fn message(&self, major: u16, sub: u16) -> Vec<u8> {
        match rexx_inventory::errors::lookup(major, sub) {
            Some(entry) => substitute(entry.text, &self.additional),
            None => format!("<no message {major}.{sub} in the catalogue>").into_bytes(),
        }
    }
}

/// Replaces `&1`, `&2`, ... with the raiser's substitution values.
fn substitute(text: &str, values: &[Substitution]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut bytes = text.as_bytes().iter().copied().peekable();
    while let Some(byte) = bytes.next() {
        if byte != b'&' {
            out.push(byte);
            continue;
        }
        match bytes.peek().copied() {
            Some(digit @ b'1'..=b'9') => {
                bytes.next();
                match values.get(usize::from(digit - b'1')) {
                    Some(value) => out.extend_from_slice(value),
                    None => out.extend_from_slice(&[b'&', digit]),
                }
            }
            _ => out.push(b'&'),
        }
    }
    out
}

/// The oracle's own rule for putting arbitrary Rexx bytes on a report line.
/// ```text
/// rendered as ?  :  00-08  0b-0c  0e-1f
/// rendered raw   :  09-0a  0d     20-ff
/// ```
pub(crate) fn displayable(bytes: &mut [u8]) {
    for byte in bytes {
        if *byte < 0x20 && !matches!(*byte, b'\t' | b'\n' | b'\r') {
            *byte = b'?';
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One `FailureSite`, for the tests that predate the stack.
    fn one(line: usize, text: &[u8], indent: usize) -> Vec<FailureSite> {
        vec![FailureSite::Clause {
            line,
            text: text.to_vec(),
            indent,
        }]
    }

    /// The 7.3 transcript, captured from `build/bin/rexx` with `cat -A` so the
    /// trailing bytes are the oracle's and not a guess.
    #[test]
    fn the_7_3_report_matches_the_oracle_byte_for_byte() {
        let raised = Raised::syntax(7, 3, vec![]);
        let sites = one(4, b"end", 0);
        let site = ClauseSite {
            path: "/abs/path/f.rex",
            sites: &sites,
        };
        assert_eq!(
            String::from_utf8(raised.report(&site)).unwrap(),
            "     4 *-* end\n\
             Error 7 running /abs/path/f.rex line 4:  WHEN or OTHERWISE expected.\n\
             Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.\n"
        );
        assert_eq!(raised.exit_code(), 249);
    }

    /// A substituted message, and a clause echo that keeps its trailing space.
    #[test]
    fn a_substituted_message_and_a_clause_echo_that_keeps_its_trailing_space() {
        let raised = Raised::not_logical(b"x");
        let sites = one(12, b"if 'x' ", 0);
        let site = ClauseSite {
            path: "/abs/w.rex",
            sites: &sites,
        };
        let report = String::from_utf8(raised.report(&site)).unwrap();
        assert_eq!(
            report,
            "    12 *-* if 'x' \n\
             Error 34 running /abs/w.rex line 12:  Logical value not 0 or 1.\n\
             Error 34.901:  Logical value must be exactly \"0\" or \"1\"; found \"x\".\n"
        );
        assert_eq!(raised.exit_code(), 222);
    }

    /// The line number is right-aligned in a six-character field, measured at
    /// one, two and three digits against the oracle: `     4`, `    12`,
    /// `   105`.
    #[test]
    fn the_line_number_field_is_six_wide() {
        for (line, expected) in [(4usize, "     4"), (12, "    12"), (105, "   105")] {
            let sites = one(line, b"nop", 0);
            let site = ClauseSite {
                path: "/p",
                sites: &sites,
            };
            let report = Raised::syntax(7, 3, vec![]).report(&site);
            let first = String::from_utf8(report).unwrap();
            let first = first.lines().next().unwrap().to_string();
            assert_eq!(&first[..6], expected, "line {line}");
        }
    }

    /// `256 - major`, over every major 4a is measured to raise.
    #[test]
    fn the_exit_code_is_256_minus_the_major() {
        for (major, sub, rc) in [
            (7u16, 3u16, 249i32),
            (11, 1, 245),
            (24, 901, 232),
            (25, 11, 231),
            (26, 5, 230),
            (28, 3, 228),
            (33, 1, 223),
            (34, 1, 222),
            (41, 1, 215),
            (42, 3, 214),
            (98, 913, 158),
        ] {
            assert_eq!(
                Raised::syntax(major, sub, vec![]).exit_code(),
                rc,
                "{major}"
            );
        }
    }

    /// Every raiser family 4a is measured to produce has catalogue text for
    /// both its lines.
    #[test]
    fn every_measured_family_has_catalogue_text() {
        for (major, sub) in [
            (7u16, 3u16),
            (11, 1),
            (24, 1),
            (24, 901),
            (25, 11),
            (26, 2),
            (26, 3),
            (26, 5),
            (26, 6),
            (26, 8),
            (33, 1),
            (34, 1),
            (34, 2),
            (34, 3),
            (34, 4),
            (28, 1),
            (28, 2),
            (28, 3),
            (28, 4),
            (28, 5),
            (34, 6),
            (34, 901),
            (41, 1),
            (42, 3),
            (42, 901),
            (98, 913),
        ] {
            for (m, s) in [(major, 0), (major, sub)] {
                let entry = rexx_inventory::errors::lookup(m, s)
                    .unwrap_or_else(|| panic!("no catalogue entry for {m}.{s}"));
                assert!(!entry.text.is_empty(), "{m}.{s} has empty text");
            }
        }
    }

    /// A substitution value containing `&1` is not re-substituted.
    #[test]
    fn a_substitution_value_containing_an_ampersand_digit_is_left_alone() {
        let raised = Raised::nonnumeric(b"&1");
        assert_eq!(
            raised.message(41, 1),
            b"Nonnumeric value (\"&1\") used in arithmetic operation."
        );
    }

    /// An `&` that is not a substitution, and a missing value, both pass
    /// through rather than being swallowed.
    #[test]
    fn a_bare_ampersand_and_a_missing_value_pass_through() {
        assert_eq!(substitute("a & b", &[]), b"a & b");
        assert_eq!(substitute("x &1 y", &[]), b"x &1 y");
        assert_eq!(substitute("&1 and &2", &[b"one".to_vec()]), b"one and &2");
    }

    /// A catalogue miss renders visibly instead of panicking or rendering
    /// empty: the error path is the worst place to abort, since it would turn
    /// a reportable condition into a crash.
    #[test]
    fn a_catalogue_miss_is_visible_rather_than_silent() {
        let raised = Raised::syntax(999, 999, vec![]);
        assert_eq!(
            raised.message(999, 999),
            b"<no message 999.999 in the catalogue>"
        );
    }

    /// Task 11's own addition: `site.indent` prefixes the clause echo with
    /// that many spaces, and nothing else on the report moves.
    #[test]
    fn the_indent_field_prefixes_the_clause_echo_with_that_many_spaces() {
        let raised = Raised::syntax(42, 3, vec![]);
        let sites = one(2, b"say 1/0", 2);
        let site = ClauseSite {
            path: "/abs/do1.rex",
            sites: &sites,
        };
        let report = String::from_utf8(raised.report(&site)).unwrap();
        assert_eq!(
            report.lines().next().unwrap(),
            "     2 *-*   say 1/0",
            "two spaces before the clause text, none anywhere else on the line"
        );
    }

    /// 4b Task 2: one echo line per entry, innermost first, each carrying its
    /// own line and its own absolute indent -- and the major line naming the
    /// **innermost** entry's line, not the outermost.
    #[test]
    fn the_report_echoes_one_line_per_level_innermost_first() {
        let raised = Raised::syntax(42, 3, vec![]);
        let sites = vec![
            FailureSite::Clause {
                line: 8,
                text: b"say 1/0".to_vec(),
                indent: 6,
            },
            FailureSite::Clause {
                line: 3,
                text: b"call sub1".to_vec(),
                indent: 4,
            },
        ];
        let site = ClauseSite {
            path: "/abs/c2.rex",
            sites: &sites,
        };
        assert_eq!(
            String::from_utf8(raised.report(&site)).unwrap(),
            concat!(
                "     8 *-*       say 1/0\n",
                "     3 *-*     call sub1\n",
                "Error 42 running /abs/c2.rex line 8:  Arithmetic overflow/underflow.\n",
                "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
            )
        );
    }

    /// The clause echo saturates at 40 columns, and the two error lines do
    /// not move when it does.
    #[test]
    fn the_clause_echo_saturates_at_forty_columns() {
        for (indent, expected) in [(36usize, 36usize), (38, 38), (40, 40), (42, 40), (50, 40)] {
            let sites = one(9, b"say 1/0", indent);
            let site = ClauseSite {
                path: "/p",
                sites: &sites,
            };
            let report = String::from_utf8(Raised::syntax(42, 3, vec![]).report(&site)).unwrap();
            let echo = report.lines().next().unwrap();
            let after_field = &echo[11..];
            assert_eq!(
                after_field.len() - after_field.trim_start().len(),
                expected,
                "indent {indent}"
            );
            assert!(
                report.contains("Error 42 running /p line 9:  "),
                "the clamp moved something other than the echo's indent: {report:?}"
            );
        }
    }

    /// A `ParseError` becomes the SYNTAX condition the oracle raises for it,
    /// with the parser's own major and sub and the matching `256 - major`
    /// exit code.
    #[test]
    fn a_parse_error_becomes_the_condition_the_oracle_raises() {
        for (code, sub, rc) in [(27u16, 901u16, 229i32), (35, 929, 221)] {
            let raised: Raised = (&ParseError::new(code, sub, 0)).into();
            assert_eq!((raised.number, raised.sub), (code, sub));
            assert_eq!(raised.exit_code(), rc);
        }
    }

    // The new Task 11 raisers themselves -- 26.2/26.3/28.1-28.5/34.3/34.4 --
    // live in `run.rs` as local `fn raised_*` free functions, matching that
    // file's own established convention for every other instruction-
    // specific raiser (`raised_if_not_logical`, `raised_select_no_when`,
    // `raised_symbol_expected`, ...), not as `Raised::` methods here: this
    // module holds only the raisers `eval.rs` also needs (cross-module), and
    // `insufficient_stack` is the one member of this task's own set that
    // qualifies. Their wording is exercised end to end by `run.rs`'s own
    // tests (`run_source` against a real program, checking `raised.number`/
    // `.sub`/`.additional`), not spot-checked again here -- `Raised::message`
    // is private to this module and `every_measured_family_has_catalogue_text`
    // above already proves every one of their catalogue entries exists.
}
