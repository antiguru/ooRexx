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

//! What a program's directives say before any of them is installed.

use crate::plan::ProgramId;
use crate::{GeneratedKind, Loud, dispatch, owned_message};
use rexx_core::ObjRef;
use rexx_parse::{
    AnnotationTarget, AttributeDirective, AttributeStyle, ClassDirective, ClassRef, Directive,
    DirectiveKind, MethodDirective, Program, SymbolId,
};
use std::collections::HashMap;
use std::rc::Rc;

/// Every class this `::CLASS` names: its `SUBCLASS`/`MIXINCLASS` target (one
/// slot, `mixin` telling the two apart), its `METACLASS`, and each entry of
/// its `INHERIT` list.
fn class_references(class: &ClassDirective) -> impl Iterator<Item = &ClassRef> {
    class
        .subclass
        .iter()
        .chain(class.metaclass.iter())
        .chain(class.inherit.iter())
}

// Which stage answers when directives could each refuse a file, all
// measured against the oracle. [`directive_gap`] and
// [`Interp::resolve_directive_library`] are the walks it describes.
// ```text
// ::routine/::method/::attribute EXTERNAL  vs a failing ::CLASS  98.903 rc 158, the EXTERNAL line
// ::routine/::method/::attribute EXTERNAL  vs a ::CLASS cycle    98.903 rc 158, the EXTERNAL line
// ::routine EXTERNAL                       vs ::requires         98.903 rc 158, the EXTERNAL line
// ::annotate routine nosuch                vs a failing ::CLASS  99.945 rc 157, the ::ANNOTATE line
// ::annotate routine nosuch                vs a ::CLASS cycle    99.945 rc 157, the ::ANNOTATE line
// ::annotate routine nosuch                vs ::routine EXTERNAL whichever is FIRST in the file
// ::requires 'nosuch.rex'                  vs a failing ::CLASS  43.901 rc 213, the ::REQUIRES line
// ::requires 'nosuch.rex'                  vs a ::CLASS cycle    98.911 rc 158, the cycle's root
// ::options digits 12                      vs a failing ::CLASS  98.909 rc 158, the ::CLASS line
// ::class q metaclass zzz                  vs a failing ::CLASS  98.908 or 98.909, whichever is first
// ```
// ```text
// ::class a / ::constant kk (1/0) / ::routine r external
//                              'LIBRARY REXX Filespec'                 42.3 rc 214
// ```
// ```text
// ::routine r / ::annotate routine r / ::class a subclass zzznotaclass  98.909 rc 158
// ::class a / ::constant kk (1/0) / ::routine r / ::annotate routine r  42.3 rc 214
// ::routine r / ::annotate routine r / a duplicate ::ROUTINE pair       99.903 rc 157
// ::routine r / ::annotate routine r / a class-less ::constant (1/0)    99.906 rc 157
// ```
/// The gap a `::` directive declares at install time, or `None` for one this
/// crate can install.
pub(super) fn directive_gap(kind: &DirectiveKind) -> Option<Loud> {
    let gap = |name: &str, owner: &'static str| {
        Some(Loud {
            message: owned_message(name, Some(owner)),
        })
    };
    match kind {
        // `REGISTERED` resolves through the RXAPI function registry, and
        // registers the name there before resolving it
        // (`PackageManager::resolveRoutine`, `package/PackageManager.cpp:312`):
        // measured, a later process's `rxfuncquery` answers `0` for a name
        // such a directive failed to resolve.
        DirectiveKind::Routine(routine)
            if routine
                .external
                .as_ref()
                .is_some_and(|spec| spec.registered) =>
        {
            gap("::ROUTINE EXTERNAL naming REGISTERED", "Phase 10")
        }
        // `::OPTIONS` installs (`Interp::install_directives`' own walk): it
        // resolves no name, runs no code, and every setting it writes is one
        // an activation of this package's code starts from.
        DirectiveKind::Annotate(_)
        | DirectiveKind::Attribute(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Constant(_)
        | DirectiveKind::Method(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Requires(_)
        | DirectiveKind::Resource(_)
        | DirectiveKind::Routine(_) => None,
    }
}

/// The `LIBRARY REXX` entry point a directive's `EXTERNAL` names and the
/// `REXX` package does not export -- the one the oracle reports 90.998 for --
/// or `None` for a directive whose `EXTERNAL` resolves and for one carrying
/// none.
pub(super) fn unresolved_external(kind: &DirectiveKind) -> Option<Vec<u8>> {
    let external = match kind {
        DirectiveKind::Method(method) => dispatch::native::method_external(method),
        DirectiveKind::Attribute(attribute) => dispatch::native::attribute_external(attribute),
        _ => None,
    };
    dispatch::native::unresolved_entry(external.as_ref()).map(<[u8]>::to_vec)
}

/// The classes of the file being installed that a `::CLASS`'s own reference
/// can resolve against: the index of every `::CLASS` the file declares, and
/// the object each of the ones installed so far became.
pub(super) struct FileClasses<'a> {
    pub(super) declared: &'a HashMap<Box<[u8]>, usize>,
    pub(super) installed: &'a HashMap<usize, ObjRef>,
}

/// `directive`'s own line number and clause text, as a traceback echoes them.
pub(super) fn directive_clause(program: &Rc<Program>, directive: &Directive) -> (usize, Vec<u8>) {
    let line = program.source.line_of(directive.clause_span.start);
    let text = program
        .source
        .join_span(directive.clause_span.clone())
        .map_or_else(
            || b"<clause span outside the retained source>".to_vec(),
            |bytes| bytes.into_owned(),
        );
    (line, text)
}

/// The directives this one must be installed after: every class it names
/// that this file also declares, unqualified.
fn class_dependencies<'a>(
    class: &'a ClassDirective,
    declared: &'a HashMap<Box<[u8]>, usize>,
) -> impl Iterator<Item = usize> + 'a {
    class_references(class)
        .filter(|target| target.namespace.is_none())
        .filter_map(|target| declared.get(target.name.as_ref()).copied())
}

/// The order a file's `::CLASS` directives install in: each class after every
/// class it names, when that target is declared in the same file.
/// ```text
/// blame the LAST ::CLASS directive in the file:
///     directive_constant_blames_the_last_installed_class.rex differs, 105 of 106
/// blame the FIRST ::CLASS directive in the file:
///     directive_constant_expression_blames_the_last_class.rex differs, 105 of 106
/// ```
pub(super) fn class_install_order(
    program: &Program,
    declared: &HashMap<Box<[u8]>, usize>,
) -> Result<Vec<usize>, usize> {
    /// Where a directive is in the walk. Absent from the map is "not yet
    /// looked at".
    enum Visit {
        OnStack,
        Done,
    }
    let mut state: HashMap<usize, Visit> = HashMap::new();
    let mut order = Vec::new();
    for root in 0..program.directives.len() {
        if !matches!(program.directives[root].kind, DirectiveKind::Class(_)) {
            continue;
        }
        // The walk from this root, innermost pending class last.
        let mut stack = vec![root];
        while let Some(index) = stack.last().copied() {
            if matches!(state.get(&index), Some(Visit::Done)) {
                stack.pop();
                continue;
            }
            let DirectiveKind::Class(class) = &program.directives[index].kind else {
                state.insert(index, Visit::Done);
                stack.pop();
                continue;
            };
            // The classes this one names that the file declares. A target the
            // file does not declare is the registry's and is nobody's
            // predecessor here.
            let mut waiting = false;
            for target in class_dependencies(class, declared) {
                match state.get(&target) {
                    // Already on this walk without having finished, so every
                    // class from it up to here is waiting on it. A class
                    // naming itself reaches this on its second visit, which
                    // is the one-entry case of the same thing.
                    Some(Visit::OnStack) => return Err(root),
                    Some(Visit::Done) => {}
                    None => {
                        stack.push(target);
                        waiting = true;
                    }
                }
            }
            if waiting {
                state.insert(index, Visit::OnStack);
            } else {
                state.insert(index, Visit::Done);
                order.push(index);
                stack.pop();
            }
        }
    }
    Ok(order)
}

/// The dictionary keys a `::METHOD` directive claims, each with what a send
/// reaching it runs.
pub(super) fn method_dictionary_keys(
    method: &MethodDirective,
) -> Vec<(Vec<u8>, Option<GeneratedKind>)> {
    let upper = method.name.to_ascii_uppercase();
    if method.delegate.is_some() {
        let delegate = Some(GeneratedKind::Delegate);
        if method.attribute {
            vec![(accessor_setter_name(&upper), delegate), (upper, delegate)]
        } else {
            vec![(upper, delegate)]
        }
    } else if method.attribute {
        let setter = accessor_setter_name(&upper);
        let (get, set) = if method.abstract_ {
            (Some(GeneratedKind::Abstract), Some(GeneratedKind::Abstract))
        } else if method.external.is_some() {
            (None, None)
        } else {
            (Some(GeneratedKind::Getter), Some(GeneratedKind::Setter))
        };
        vec![(upper, get), (setter, set)]
    } else if method.abstract_ {
        vec![(upper, Some(GeneratedKind::Abstract))]
    } else {
        vec![(upper, None)]
    }
}

/// The dictionary keys an `::ATTRIBUTE` directive claims -- the plain name
/// for a getter, the name with `=` appended for a setter, both for the
/// default (neither `GET` nor `SET`) style.
pub(super) fn attribute_dictionary_keys(
    attribute: &AttributeDirective,
) -> Vec<(Vec<u8>, Option<GeneratedKind>)> {
    let upper = attribute.name.to_ascii_uppercase();
    let setter_name = accessor_setter_name(&upper);
    let generated = if attribute.abstract_ {
        Some(GeneratedKind::Abstract)
    } else if attribute.delegate.is_some() {
        Some(GeneratedKind::Delegate)
    } else if attribute.external.is_some() || attribute.body.is_some() {
        None
    } else {
        // The one place what an accessor is depends on which half of the pair
        // it is.
        Some(GeneratedKind::Getter)
    };
    // The setter's half of that one place.
    let setter = match generated {
        Some(GeneratedKind::Getter) => Some(GeneratedKind::Setter),
        other => other,
    };
    match attribute.style {
        AttributeStyle::Both => vec![(upper, generated), (setter_name, setter)],
        AttributeStyle::Get => vec![(upper, generated)],
        AttributeStyle::Set => vec![(setter_name, setter)],
    }
}

/// The dictionary keys one member directive claims, each with the side it
/// claims them on -- `true` for the class dictionary.
pub(super) fn member_dictionary_keys(kind: &DirectiveKind) -> Vec<(Vec<u8>, bool)> {
    match kind {
        DirectiveKind::Method(method) => method_dictionary_keys(method)
            .into_iter()
            .map(|(name, _)| (name, method.class_method))
            .collect(),
        DirectiveKind::Attribute(attribute) => attribute_dictionary_keys(attribute)
            .into_iter()
            .map(|(name, _)| (name, attribute.class_method))
            .collect(),
        DirectiveKind::Constant(constant) => {
            let upper = constant.name.to_ascii_uppercase();
            vec![(upper.clone(), false), (upper, true)]
        }
        _ => Vec::new(),
    }
}

/// Which directives each `::CLASS` in `program` owns, keyed by that
/// `::CLASS`'s own index and in source order within a class.
pub(super) fn class_members(program: &Program) -> HashMap<usize, Vec<usize>> {
    let mut members: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut current: Option<usize> = None;
    for (index, directive) in program.directives.iter().enumerate() {
        // A synthetic directive belongs to no class -- see the skip in
        // `Interp::install_directives`, which carries why an empty clause
        // span is what tells one from a written directive. This one is what
        // keeps a loaded file's main section out of that file's last class.
        if directive.clause_span.is_empty() {
            continue;
        }
        match &directive.kind {
            DirectiveKind::Class(_) => current = Some(index),
            DirectiveKind::Method(_) | DirectiveKind::Attribute(_) | DirectiveKind::Constant(_) => {
                if let Some(class) = current {
                    members.entry(class).or_default().push(index);
                }
            }
            _ => {}
        }
    }
    members
}

/// The [`rexx_core::RootSet::add_global`] key one `::CONSTANT`'s value is
/// held under.
pub(super) fn constant_root_key(ProgramId(program): ProgramId, directive: usize) -> String {
    format!("constant:{program}:{directive}")
}

/// What an `::ANNOTATE` names, as the first install walk can express it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(super) enum AnnotatedSite {
    /// `::ANNOTATE PACKAGE`, which names the running package and no
    /// directive.
    Package,
    /// The `::CLASS` or `::ROUTINE` directive named.
    Directive(usize),
    /// One dictionary entry of a method-shaped directive: the directive that
    /// declares it and the name it is filed under.
    Member(usize, Box<[u8]>),
}

/// An `::ANNOTATE` target the accumulated package does not hold: the
/// keyword's own spelling for the message, and the name that resolved to
/// nothing.
pub(super) struct MissingTarget<'a> {
    pub(super) kind: &'static str,
    pub(super) name: &'a [u8],
}

/// Whether a member directive's methods are the *attribute* methods
/// `::ANNOTATE ATTRIBUTE` will accept -- `MethodClass::isAttribute()`.
fn is_attribute_method(kind: &DirectiveKind) -> bool {
    match kind {
        DirectiveKind::Attribute(_) => true,
        DirectiveKind::Method(method) => method.attribute,
        _ => false,
    }
}

/// `annotateDirective`'s target resolution (`parser/DirectiveParser.cpp:1940`):
/// which directives an `::ANNOTATE` annotates, or the target it could not
/// find.
pub(super) fn annotation_target<'a>(
    program: &Program,
    target: &'a AnnotationTarget,
    current_class: Option<usize>,
    claimed: &HashMap<(Option<usize>, bool, Vec<u8>), usize>,
    declared_classes: &HashMap<Vec<u8>, usize>,
    declared_routines: &HashMap<Vec<u8>, usize>,
) -> Result<Vec<AnnotatedSite>, MissingTarget<'a>> {
    // `findMethod`'s own order, instance dictionary before class.
    let member = |name: &[u8]| -> Option<usize> {
        [false, true]
            .into_iter()
            .find_map(|side| claimed.get(&(current_class, side, name.to_vec())).copied())
    };
    // One half of an accessor pair, on the side asked and only where the
    // directive that claimed the name is an attribute directive.
    let accessor = |name: &[u8], side: bool| -> Option<usize> {
        let directive = claimed
            .get(&(current_class, side, name.to_vec()))
            .copied()?;
        is_attribute_method(&program.directives[directive].kind).then_some(directive)
    };
    match target {
        AnnotationTarget::Package => Ok(vec![AnnotatedSite::Package]),
        AnnotationTarget::Class(name) => declared_classes
            .get(&name.to_vec())
            .map(|index| vec![AnnotatedSite::Directive(*index)])
            .ok_or(MissingTarget {
                kind: "class",
                name,
            }),
        AnnotationTarget::Routine(name) => declared_routines
            .get(&name.to_vec())
            .map(|index| vec![AnnotatedSite::Directive(*index)])
            .ok_or(MissingTarget {
                kind: "routine",
                name,
            }),
        AnnotationTarget::Method(name) => member(name)
            .map(|index| vec![AnnotatedSite::Member(index, name.clone())])
            .ok_or(MissingTarget {
                kind: "method",
                name,
            }),
        AnnotationTarget::Constant(name) => member(name)
            .filter(|index| matches!(program.directives[*index].kind, DirectiveKind::Constant(_)))
            .map(|index| vec![AnnotatedSite::Member(index, name.clone())])
            .ok_or(MissingTarget {
                kind: "constant",
                name,
            }),
        AnnotationTarget::Attribute(name) => {
            let setter: Box<[u8]> = accessor_setter_name(name).into();
            let mut found = Vec::new();
            for side in [false, true] {
                for half in [name, &setter] {
                    if let Some(index) = accessor(half, side) {
                        found.push(AnnotatedSite::Member(index, half.clone()));
                    }
                }
                if !found.is_empty() {
                    break;
                }
            }
            if found.is_empty() {
                return Err(MissingTarget {
                    kind: "attribute",
                    name,
                });
            }
            Ok(found)
        }
    }
}

/// Why a resolved method's directive cannot be entered, or `None` when it
/// can -- the gate `Interp::enter_method_body` (`dispatch.rs`) takes before
/// it pushes anything.
pub(super) fn method_body_gap(kind: &DirectiveKind) -> Option<Loud> {
    match kind {
        DirectiveKind::Method(method) => {
            if method.body.is_none() {
                Some(Loud::method_body("a ::METHOD with no body of its own"))
            } else {
                None
            }
        }
        DirectiveKind::Attribute(attribute) => {
            if attribute.body.is_none() {
                Some(Loud::method_body("a ::ATTRIBUTE with no body of its own"))
            } else {
                None
            }
        }
        other => Some(Loud::method_body(&format!(
            "a method installed by ::{}",
            other.keyword()
        ))),
    }
}

/// The dictionary key of a generated setter: the getter's key with `=`
/// appended.
pub(crate) fn accessor_setter_name(upper: &[u8]) -> Vec<u8> {
    let mut setter = upper.to_vec();
    setter.push(b'=');
    setter
}

/// The variable a generated accessor reads and writes: the directive's name
/// **as written**, which is not the accessor's own dictionary key.
pub(super) fn accessor_variable(kind: &DirectiveKind) -> Option<&[u8]> {
    match kind {
        DirectiveKind::Attribute(attribute) => Some(&attribute.name),
        DirectiveKind::Method(method) => Some(&method.name),
        _ => None,
    }
}

/// The variable a `DELEGATE` method reads to find its target: the directive's
/// `DELEGATE` symbol, **not** the directive's own name.
pub(super) fn delegate_variable(kind: &DirectiveKind) -> Option<SymbolId> {
    match kind {
        DirectiveKind::Attribute(attribute) => attribute.delegate,
        DirectiveKind::Method(method) => method.delegate,
        _ => None,
    }
}
