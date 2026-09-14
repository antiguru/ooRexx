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

//! The `LIBRARY REXX` entry-point registry (D37): the method names the `REXX`
//! package exports, and what a send to a `::METHOD` bound to one of them
//! runs.
//! ```text
//! ::routine r external 'LIBRARY REXX Filespec'     rc 0, and the routine
//!                                                  runs: it resolves
//!                                                  against a routine table
//!                                                  this registry is not
//! ::routine r external 'LIBRARY zzznolib zzzr'     98.903 rc 158
//! ::attribute at external 'LIBRARY zzznolib zzz'   98.903 rc 158
//! ::method m external 'LIBRARY zzznolib zzzr'      98.903 rc 158
//! ```

use rexx_parse::{
    AttributeDirective, AttributeStyle, ExternalSpec, MethodDirective, RoutineDirective,
};

use super::{Arity, NativeMethod, native_file_path_separator, native_file_separator};
use crate::{Loud, accessor_setter_name};

/// Which part of the interpreter an entry point belongs to, and through that
/// which phase owes it a body.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Family {
    /// `platform/unix/TimeSupport.cpp`: the timers `.Alarm` and `.Ticker`
    /// drive.
    Timer,
    /// `streamLibrary/StreamNative.cpp`: `.Stream`'s body.
    Stream,
    /// `classes/RexxQueueMethods.cpp`: `.RexxQueue`'s body.
    Queue,
    /// `streamLibrary/FileNative.cpp`: `.File`'s body.
    File,
}

impl Family {
    /// The family's short name, used to name its corpus program and to report
    /// it. Lower case, because it names a group of entry points and not a
    /// class.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Family::Timer => "timer",
            Family::Stream => "stream",
            Family::Queue => "queue",
            Family::File => "file",
        }
    }
}

/// One entry point the `REXX` package exports to `::METHOD ... EXTERNAL`.
pub(crate) struct NativeExternal {
    /// The name, spelled as `interpreter/runtime/NativeMethods.h` spells it.
    /// Matched caselessly, so the spelling is the C++'s for a reader's sake
    /// and never for the lookup's.
    pub(crate) name: &'static str,
    pub(crate) family: Family,
    /// **Not `pub(crate)`**: `dispatch::Interp::invoke` is the only reader,
    /// and a body is a [`NativeMethod`], which takes the seam's own
    /// `Cleared` and cannot be named outside this module's parent.
    pub(in crate::dispatch) body: ExternalBody,
}

/// What a send to a method bound to an entry point runs.
pub(in crate::dispatch) enum ExternalBody {
    /// A body this phase runs, with the parameter count the C++ entry
    /// declares.
    Implemented { arity: Arity, run: NativeMethod },
    /// A body some later phase owes, named here. The bind still succeeds;
    /// the send is loud.
    Deferred { owner: &'static str },
}

const fn implemented(
    name: &'static str,
    family: Family,
    arity: Arity,
    run: NativeMethod,
) -> NativeExternal {
    NativeExternal {
        name,
        family,
        body: ExternalBody::Implemented { arity, run },
    }
}

/// A deferred entry, which names the phase that owes it a body.
///
/// **Named per row rather than per family**, because the two came apart:
/// `handle_set` needs `from_raw_fd` and so stays refused after the stream
/// family is built, and a refusal naming a closed phase would be a lie.
/// It is owed by the phase that widens D-U1 past this crate's two granted
/// files, which is Phase 10's -- adopting a descriptor the interpreter did
/// not open is the same grant RXAPI and the external queues need. It never
/// needed the loader.
const fn deferred(name: &'static str, family: Family, owner: &'static str) -> NativeExternal {
    NativeExternal {
        name,
        family,
        body: ExternalBody::Deferred { owner },
    }
}

/// The `REXX` package's exported method table.
static LIBRARY_REXX_METHODS: &[NativeExternal] = &[
    deferred("alarm_startTimer", Family::Timer, "Phase 6"),
    deferred("alarm_stopTimer", Family::Timer, "Phase 6"),
    deferred("ticker_createTimer", Family::Timer, "Phase 6"),
    deferred("ticker_waitTimer", Family::Timer, "Phase 6"),
    deferred("ticker_stopTimer", Family::Timer, "Phase 6"),
    implemented(
        "stream_init",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::init,
    ),
    implemented(
        "stream_open",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::open,
    ),
    implemented(
        "stream_chars",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::chars,
    ),
    implemented(
        "stream_lines",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::lines,
    ),
    implemented(
        "stream_position",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::position,
    ),
    implemented(
        "stream_state",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::state,
    ),
    implemented(
        "stream_description",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::description,
    ),
    implemented(
        "stream_query_position",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::query_position,
    ),
    implemented(
        "stream_charout",
        Family::Stream,
        Arity::Fixed(2),
        super::stream::charout,
    ),
    implemented(
        "stream_charin",
        Family::Stream,
        Arity::Fixed(2),
        super::stream::charin,
    ),
    implemented(
        "stream_linein",
        Family::Stream,
        Arity::Fixed(2),
        super::stream::linein,
    ),
    implemented(
        "stream_lineout",
        Family::Stream,
        Arity::Fixed(2),
        super::stream::lineout,
    ),
    implemented(
        "stream_arrayin",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::arrayin,
    ),
    implemented(
        "qualify",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::qualify,
    ),
    implemented(
        "query_exists",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_exists,
    ),
    implemented(
        "query_size",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_size,
    ),
    implemented(
        "query_time",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_time,
    ),
    deferred("handle_set", Family::Stream, "Phase 10"),
    implemented(
        "std_set",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::std_set,
    ),
    implemented(
        "stream_flush",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::flush,
    ),
    implemented(
        "query_handle",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_handle,
    ),
    implemented(
        "query_streamtype",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_streamtype,
    ),
    implemented(
        "stream_close",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::close,
    ),
    implemented(
        "stream_uninit",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::uninit,
    ),
    deferred("rexx_create_queue", Family::Queue, "Phase 10"),
    deferred("rexx_open_queue", Family::Queue, "Phase 10"),
    deferred("rexx_queue_exists", Family::Queue, "Phase 10"),
    deferred("rexx_delete_queue", Family::Queue, "Phase 10"),
    deferred("rexx_query_queue", Family::Queue, "Phase 10"),
    deferred("rexx_push_queue", Family::Queue, "Phase 10"),
    deferred("rexx_queue_queue", Family::Queue, "Phase 10"),
    deferred("rexx_pull_queue", Family::Queue, "Phase 10"),
    deferred("rexx_linein_queue", Family::Queue, "Phase 10"),
    deferred("rexx_clear_queue", Family::Queue, "Phase 10"),
    implemented(
        "file_separator",
        Family::File,
        Arity::Fixed(0),
        native_file_separator,
    ),
    implemented(
        "file_path_separator",
        Family::File,
        Arity::Fixed(0),
        native_file_path_separator,
    ),
    implemented(
        "file_case_sensitive",
        Family::File,
        Arity::Fixed(0),
        super::files::case_sensitive,
    ),
    implemented(
        "file_list_roots",
        Family::File,
        Arity::Fixed(0),
        super::files::list_roots,
    ),
    implemented(
        "file_qualify",
        Family::File,
        Arity::Fixed(1),
        super::files::qualify,
    ),
    implemented(
        "file_exists",
        Family::File,
        Arity::Fixed(1),
        super::files::exists,
    ),
    implemented(
        "file_delete_file",
        Family::File,
        Arity::Fixed(1),
        super::files::delete_file,
    ),
    implemented(
        "file_delete_directory",
        Family::File,
        Arity::Fixed(1),
        super::files::delete_directory,
    ),
    implemented(
        "file_isDirectory",
        Family::File,
        Arity::Fixed(1),
        super::files::is_directory,
    ),
    implemented(
        "file_isFile",
        Family::File,
        Arity::Fixed(1),
        super::files::is_file,
    ),
    implemented(
        "file_isHidden",
        Family::File,
        Arity::Fixed(1),
        super::files::is_hidden,
    ),
    implemented(
        "file_get_last_modified",
        Family::File,
        Arity::Fixed(1),
        super::files::get_last_modified,
    ),
    implemented(
        "file_set_last_modified",
        Family::File,
        Arity::Fixed(2),
        super::files::set_last_modified,
    ),
    implemented(
        "file_set_read_only",
        Family::File,
        Arity::Fixed(1),
        super::files::set_read_only,
    ),
    implemented(
        "file_length",
        Family::File,
        Arity::Fixed(1),
        super::files::length,
    ),
    implemented(
        "file_list",
        Family::File,
        Arity::Fixed(1),
        super::files::list,
    ),
    implemented(
        "file_make_dir",
        Family::File,
        Arity::Fixed(1),
        super::files::make_dir,
    ),
    implemented(
        "file_can_read",
        Family::File,
        Arity::Fixed(1),
        super::files::can_read,
    ),
    implemented(
        "file_can_write",
        Family::File,
        Arity::Fixed(1),
        super::files::can_write,
    ),
    implemented(
        "file_rename",
        Family::File,
        Arity::Fixed(2),
        super::files::rename,
    ),
    implemented(
        "this_file_case_sensitive",
        Family::File,
        Arity::Fixed(1),
        super::files::this_file_case_sensitive,
    ),
    implemented(
        "file_get_last_accessed",
        Family::File,
        Arity::Fixed(1),
        super::files::get_last_accessed,
    ),
    implemented(
        "file_set_last_accessed",
        Family::File,
        Arity::Fixed(2),
        super::files::set_last_accessed,
    ),
    implemented(
        "file_set_writable",
        Family::File,
        Arity::Fixed(1),
        super::files::set_writable,
    ),
    implemented(
        "file_temporary_path",
        Family::File,
        Arity::Fixed(0),
        super::files::temporary_path,
    ),
    implemented(
        "file_search_path_impl",
        Family::File,
        Arity::Fixed(2),
        super::files::search_path_impl,
    ),
];

/// Every row of the registry, flattened for a caller outside this module --
/// `crate::native_entry_points` is the one, and its own doc says why the
/// registry's types stay in here.
pub(crate) fn entry_points() -> impl Iterator<Item = crate::NativeEntryPoint> {
    LIBRARY_REXX_METHODS
        .iter()
        .map(|entry| crate::NativeEntryPoint {
            entry: entry.name,
            family: entry.family.label(),
            owner: match entry.body {
                ExternalBody::Deferred { owner } => Some(owner),
                ExternalBody::Implemented { .. } => None,
            },
            implemented: matches!(entry.body, ExternalBody::Implemented { .. }),
        })
}

/// The entry point the `REXX` package exports under `name`, or `None` for a
/// name it does not export.
pub(crate) fn entry_point(name: &[u8]) -> Option<&'static NativeExternal> {
    LIBRARY_REXX_METHODS
        .iter()
        .find(|entry| entry.name.as_bytes().eq_ignore_ascii_case(name))
}

/// What a `::METHOD` or `::ATTRIBUTE` directive's `EXTERNAL` names, or `None`
/// for a directive that carries no `EXTERNAL` at all.
pub(crate) enum MethodExternal {
    /// `'LIBRARY REXX [entry]'` on a `::METHOD` with no `ATTRIBUTE` keyword:
    /// one method bound to one entry point. `Ok` is the entry point the `REXX`
    /// package exports; `Err` is the name it does not export, which the
    /// install reports as 90.998.
    LibraryRexx(Result<&'static NativeExternal, Vec<u8>>),
    /// `'LIBRARY REXX [entry]'` on a `::ATTRIBUTE`, or on a `::METHOD`
    /// carrying `ATTRIBUTE`: the accessors [`attribute_binds`] derives, in the
    /// order the oracle creates them.
    Attribute(Vec<AttributeBind>),
    /// `'LIBRARY <name>'` for any name but `REXX`: a shared library to load,
    /// with the dictionary key each of its procedures fills.
    OtherLibrary {
        /// The library name as written. Matched byte for byte: measured,
        /// oracle, `LIBRARY RXREGEXP` is `98.903 Unable to load library
        /// "RXREGEXP"` where `LIBRARY rxregexp` loads.
        library: Vec<u8>,
        binds: Vec<LibraryBind>,
    },
}

/// One procedure a library-backed `EXTERNAL` names.
pub(crate) struct LibraryBind {
    /// The dictionary key this procedure fills, upcased, as
    /// [`AttributeBind::key`] is.
    pub(crate) key: Vec<u8>,
    /// The procedure to look up in the library's own table.
    pub(crate) procedure: Vec<u8>,
}

/// One accessor an attribute-shaped `EXTERNAL` binds.
pub(crate) struct AttributeBind {
    /// The dictionary key this accessor fills, upcased -- the directive's own
    /// name for a getter, that name with `=` appended for a setter. An
    /// installer pairs a key with its bind by this name rather than by
    /// position.
    pub(crate) key: Vec<u8>,
    /// The entry point the `REXX` package exports under the procedure this
    /// accessor names, or that procedure where it exports none.
    pub(crate) entry: Result<&'static NativeExternal, Vec<u8>>,
}

/// The accessors an attribute-shaped `EXTERNAL` binds, in the order the oracle
/// creates them: the getter, then the setter.
/// ```text
/// ::attribute at external 'LIBRARY REXX zzz_no_entry'      90.998 rc 166 on "GETzzz_no_entry"
/// ::attribute at external 'LIBRARY REXX'                   90.998 rc 166 on "GETAT"
/// ::attribute at get external 'LIBRARY REXX zzz_no_entry'  90.998 rc 166 on "zzz_no_entry"
/// ::attribute at get external 'LIBRARY REXX'               90.998 rc 166 on "GETAT"
/// ::attribute at get external 'LIBRARY REXX AT'            90.998 rc 166 on "GETAT"
/// ::attribute at set external 'LIBRARY REXX'               90.998 rc 166 on "SETAT"
/// ::attribute file_separator get
///                     external 'LIBRARY REXX FILE_SEPARATOR'
///                                                          90.998 rc 166 on "GETFILE_SEPARATOR"
/// ::attribute file_separator get
///                     external 'LIBRARY REXX file_separator'          rc 0
/// ::attribute at get external 'LIBRARY REXX file_separator'           rc 0
/// ::attribute at set external 'LIBRARY REXX file_separator'           rc 0
/// ::method m attribute external 'LIBRARY REXX'             90.998 rc 166 on "GETM"
/// ```
fn attribute_binds(upper: &[u8], style: AttributeStyle, spec: &ExternalSpec) -> Vec<AttributeBind> {
    attribute_accessors(upper, style, spec)
        .into_iter()
        .map(|(key, procedure)| AttributeBind {
            key,
            entry: entry_point(&procedure).ok_or(procedure),
        })
        .collect()
}

/// The dictionary key and the procedure name of each accessor, which is the
/// half of [`attribute_binds`] that does not depend on the `REXX` package.
fn attribute_accessors(
    upper: &[u8],
    style: AttributeStyle,
    spec: &ExternalSpec,
) -> Vec<(Vec<u8>, Vec<u8>)> {
    let procedure: Vec<u8> = match &spec.entry {
        Some(entry) => entry.to_vec(),
        None => upper.to_vec(),
    };
    let prefixed = |prefix: &str| [prefix.as_bytes(), &procedure].concat();
    let defaulted = *procedure == *upper;
    match style {
        AttributeStyle::Both => vec![
            (upper.to_vec(), prefixed("GET")),
            (accessor_setter_name(upper), prefixed("SET")),
        ],
        AttributeStyle::Get => vec![(
            upper.to_vec(),
            if defaulted {
                prefixed("GET")
            } else {
                procedure.clone()
            },
        )],
        AttributeStyle::Set => vec![(
            accessor_setter_name(upper),
            if defaulted {
                prefixed("SET")
            } else {
                procedure.clone()
            },
        )],
    }
}

/// [`MethodExternal`] for one `::METHOD`. Half of the boundary between the
/// forms this phase binds and the forms it refuses; [`attribute_external`] is
/// the other half and the library test is shared.
pub(crate) fn method_external(method: &MethodDirective) -> Option<MethodExternal> {
    let spec = method.external.as_ref()?;
    let upper = method.name.to_ascii_uppercase();
    // Before the `ATTRIBUTE` question, because `resolveMethod` loads the
    // library before it looks for either accessor's procedure in it. Measured,
    // oracle: `::method m attribute external 'LIBRARY nosuchlib x'` is
    // `98.903 Unable to load library "nosuchlib"` at rc 158 and not 90.998.
    if *spec.library != *b"REXX" {
        let style = method.attribute.then_some(AttributeStyle::Both);
        return Some(other_library(&upper, style, spec));
    }
    if method.attribute {
        return Some(MethodExternal::Attribute(attribute_binds(
            &upper,
            AttributeStyle::Both,
            spec,
        )));
    }
    let entry = match &spec.entry {
        Some(entry) => entry.to_vec(),
        None => upper,
    };
    Some(MethodExternal::LibraryRexx(
        entry_point(&entry).ok_or(entry),
    ))
}

/// [`MethodExternal`] for one `::ATTRIBUTE`, whose accessors are the same
/// mechanism [`method_external`] gives a `::METHOD ... ATTRIBUTE`, with the
/// `GET` and `SET` styles [`attribute_binds`] describes on top of it.
pub(crate) fn attribute_external(attribute: &AttributeDirective) -> Option<MethodExternal> {
    let spec = attribute.external.as_ref()?;
    let upper = attribute.name.to_ascii_uppercase();
    if *spec.library != *b"REXX" {
        return Some(other_library(&upper, Some(attribute.style), spec));
    }
    Some(MethodExternal::Attribute(attribute_binds(
        &upper,
        attribute.style,
        spec,
    )))
}

/// [`MethodExternal`] for one `::ROUTINE`, or `None` for the forms this phase
/// still refuses: `REGISTERED`, and the `REXX` package, whose routine table
/// is `rexx_routines[]` and not the method registry above
/// (`runtime/InternalPackage.cpp:230`).
///
/// The entry defaults to the routine's own name upcased -- measured, oracle:
/// `::routine zzz external "LIBRARY rxmath"` is `90.999 Unable to find
/// external routine "ZZZ"`.
pub(crate) fn routine_external(routine: &RoutineDirective) -> Option<MethodExternal> {
    let spec = routine.external.as_ref()?;
    if spec.registered || *spec.library == *b"REXX" {
        return None;
    }
    Some(other_library(
        &routine.name.to_ascii_uppercase(),
        None,
        spec,
    ))
}

/// [`MethodExternal::OtherLibrary`] for one directive: the accessors of an
/// attribute-shaped `EXTERNAL`, or the single procedure of every other shape.
fn other_library(
    upper: &[u8],
    style: Option<AttributeStyle>,
    spec: &ExternalSpec,
) -> MethodExternal {
    let accessors = match style {
        Some(style) => attribute_accessors(upper, style, spec),
        None => {
            let procedure = spec
                .entry
                .as_ref()
                .map_or_else(|| upper.to_vec(), |entry| entry.to_vec());
            vec![(upper.to_vec(), procedure)]
        }
    };
    MethodExternal::OtherLibrary {
        library: spec.library.to_vec(),
        binds: accessors
            .into_iter()
            .map(|(key, procedure)| LibraryBind { key, procedure })
            .collect(),
    }
}

/// The entry point one dictionary key is bound to, or `None` for a key this
/// `EXTERNAL` does not bind, for a procedure the `REXX` package does not
/// export, and for a directive carrying no `EXTERNAL`.
pub(crate) fn bound_entry(
    external: Option<&MethodExternal>,
    key: &[u8],
) -> Option<&'static NativeExternal> {
    match external? {
        MethodExternal::LibraryRexx(entry) => entry.as_ref().ok().copied(),
        MethodExternal::Attribute(binds) => binds
            .iter()
            .find(|bind| *bind.key == *key)
            .and_then(|bind| bind.entry.as_ref().ok().copied()),
        MethodExternal::OtherLibrary { .. } => None,
    }
}

/// The first procedure an `EXTERNAL` names that the `REXX` package does not
/// export -- the one the oracle reports 90.998 for -- or `None` when every
/// accessor resolved.
pub(crate) fn unresolved_entry(external: Option<&MethodExternal>) -> Option<&[u8]> {
    match external? {
        MethodExternal::LibraryRexx(entry) => entry.as_ref().err().map(Vec::as_slice),
        MethodExternal::Attribute(binds) => binds
            .iter()
            .find_map(|bind| bind.entry.as_ref().err().map(Vec::as_slice)),
        MethodExternal::OtherLibrary { .. } => None,
    }
}

/// The refusal a send to an entry point this phase does not implement gets.
pub(crate) fn deferred_send(entry: &NativeExternal, owner: &'static str) -> Loud {
    Loud {
        message: crate::owned_message(
            &format!("the LIBRARY REXX entry point \"{}\"", entry.name),
            Some(owner),
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    use rexx_parse::DirectiveKind;

    use super::*;

    /// The interpreter's own bootstrap Rexx, at the paths the C++ tree keeps
    /// it at.
    fn bootstrap_files() -> Vec<PathBuf> {
        let root = PathBuf::from("/home/moritz/dev/repos/ooRexx/interpreter");
        vec![
            root.join("RexxClasses/CoreClasses.orx"),
            root.join("RexxClasses/StreamClasses.orx"),
            root.join("platform/unix/PlatformObjects.orx"),
        ]
    }

    /// Each bootstrap file's directives, parsed by this crate's own parser
    /// rather than scanned.
    fn bootstrap_directives() -> Vec<(PathBuf, rexx_parse::Program)> {
        bootstrap_files()
            .into_iter()
            .map(|path| {
                let text = std::fs::read(&path).unwrap_or_else(|e| {
                    panic!(
                        "cannot read {}: {e}. This crate is checked against that C++ \
                         tree; without it there is nothing to resolve against",
                        path.display()
                    )
                });
                let program = rexx_parse::parse_program(text)
                    .unwrap_or_else(|e| panic!("{} does not parse: {e:?}", path.display()));
                (path, program)
            })
            .collect()
    }

    /// Every `EXTERNAL` the bootstrap files declare is the form this phase
    /// binds, and every one of them resolves.
    #[test]
    fn every_bootstrap_external_binds_to_an_entry_point_this_registry_holds() {
        let mut reached = 0usize;
        for (path, program) in bootstrap_directives() {
            for directive in &program.directives {
                match &directive.kind {
                    DirectiveKind::Method(method) => {
                        let Some(external) = method_external(method) else {
                            continue;
                        };
                        reached += 1;
                        check_binds(&path, "::METHOD", &external);
                    }
                    DirectiveKind::Attribute(attribute) => {
                        let Some(external) = attribute_external(attribute) else {
                            continue;
                        };
                        reached += 1;
                        check_binds(&path, "::ATTRIBUTE", &external);
                    }
                    // `::ROUTINE` carries an `EXTERNAL` too and this phase
                    // refuses every spelling of it, so a bootstrap file
                    // declaring one would be refused at install whatever this
                    // registry holds. Asserted rather than written down,
                    // because "the bootstrap declares no `::ROUTINE
                    // EXTERNAL`" is a premise this registry's scope rests on.
                    DirectiveKind::Routine(routine) => assert!(
                        routine.external.is_none(),
                        "{} declares a ::ROUTINE EXTERNAL, which this phase refuses",
                        path.display()
                    ),
                    _ => {}
                }
            }
        }
        assert!(
            reached > 0,
            "no bootstrap file declared a single ::METHOD EXTERNAL, so this test \
             compared nothing"
        );
    }

    /// One bootstrap directive's `EXTERNAL`: it names `LIBRARY REXX` and every
    /// entry point it resolves against is one the registry holds. Panics with
    /// what a reader needs to place the failure otherwise.
    fn check_binds(path: &Path, keyword: &str, external: &MethodExternal) {
        let missing = match external {
            MethodExternal::LibraryRexx(entry) => entry.as_ref().err(),
            MethodExternal::Attribute(binds) => {
                binds.iter().find_map(|bind| bind.entry.as_ref().err())
            }
            MethodExternal::OtherLibrary { library, .. } => panic!(
                "{} declares a {keyword} whose EXTERNAL names the library {:?}, which \
                 is not the REXX package the bootstrap files are meant to bind against",
                path.display(),
                String::from_utf8_lossy(library)
            ),
        };
        if let Some(missing) = missing {
            panic!(
                "{} declares a {keyword} binding the entry point {:?}, which the REXX \
                 package does not export -- so the whole file is refused at install",
                path.display(),
                String::from_utf8_lossy(missing)
            );
        }
    }

    /// The registry and the bootstrap files name the same entry points.
    #[test]
    fn the_bootstrap_files_and_the_registry_name_the_same_entry_points() {
        let mut declared: BTreeSet<String> = BTreeSet::new();
        for (_, program) in bootstrap_directives() {
            for directive in &program.directives {
                if let DirectiveKind::Method(method) = &directive.kind
                    && let Some(MethodExternal::LibraryRexx(Ok(entry))) = method_external(method)
                {
                    declared.insert(entry.name.to_ascii_lowercase());
                }
            }
        }
        let held: BTreeSet<String> = LIBRARY_REXX_METHODS
            .iter()
            .map(|entry| entry.name.to_ascii_lowercase())
            .collect();
        assert_eq!(
            declared, held,
            "the entry points the bootstrap files declare and the entry points \
             this registry holds are not the same set, compared caselessly"
        );
    }

    /// Every family has at least one entry point this phase defers, so a
    /// program pinning that family's refusal can be written at all.
    #[test]
    fn every_deferred_entry_point_names_an_open_phase() {
        // The phases that still owe entry points, and no more: a fourth
        // spelling is either a typo or work nobody has been assigned, and a
        // phase that has closed cannot owe anything -- a refusal naming one
        // is a lie a program can read.
        const OPEN: &[&str] = &["Phase 6", "Phase 8", "Phase 10"];
        let mut deferred = 0usize;
        for entry in LIBRARY_REXX_METHODS {
            let ExternalBody::Deferred { owner } = entry.body else {
                continue;
            };
            deferred += 1;
            assert!(
                OPEN.contains(&owner),
                "{} defers to {owner:?}, which is not one of {OPEN:?}",
                entry.name
            );
        }
        assert!(
            deferred > 0,
            "the registry defers nothing at all, so this test is passing over an empty set"
        );
    }

    /// The lookup is caseless and the spelling the table carries is not what
    /// makes a name resolve.
    #[test]
    fn an_entry_point_resolves_whatever_case_it_is_asked_for() {
        for spelling in [
            b"file_temporary_path".to_vec(),
            b"FILE_TEMPORARY_PATH".to_vec(),
            b"File_Temporary_Path".to_vec(),
        ] {
            let found = entry_point(&spelling).unwrap_or_else(|| {
                panic!("{} did not resolve", String::from_utf8_lossy(&spelling))
            });
            assert_eq!(found.name, "file_temporary_path");
        }
        assert!(entry_point(b"file_temporary_path ").is_none());
        assert!(entry_point(b"no_such_entry_point_xyz").is_none());
    }

    /// The `EXTERNAL` of the last directive of `source`, decided by whichever
    /// half of the boundary owns that directive's keyword.
    fn last_external(source: &str) -> MethodExternal {
        let program = rexx_parse::parse_program(source.as_bytes().to_vec())
            .unwrap_or_else(|e| panic!("{source:?} does not parse: {e:?}"));
        let last = program
            .directives
            .last()
            .unwrap_or_else(|| panic!("{source:?} declares no directive"));
        let external = match &last.kind {
            DirectiveKind::Method(method) => method_external(method),
            DirectiveKind::Attribute(attribute) => attribute_external(attribute),
            other => panic!("{source:?} ends with a ::{} directive", other.keyword()),
        };
        external.unwrap_or_else(|| panic!("{source:?} carries no EXTERNAL"))
    }

    /// Each accessor of `external`, as the dictionary key it fills paired with
    /// the entry point it resolved or the procedure it did not.
    fn accessors(external: &MethodExternal) -> Vec<(String, Result<&'static str, String>)> {
        let MethodExternal::Attribute(binds) = external else {
            panic!("not an attribute-shaped EXTERNAL");
        };
        binds
            .iter()
            .map(|bind| {
                let entry = match &bind.entry {
                    Ok(entry) => Ok(entry.name),
                    Err(procedure) => Err(String::from_utf8_lossy(procedure).into_owned()),
                };
                (String::from_utf8_lossy(&bind.key).into_owned(), entry)
            })
            .collect()
    }

    /// Which procedure each accessor of an attribute-shaped `EXTERNAL`
    /// resolves against, per style and per third word.
    #[test]
    fn an_attribute_external_resolves_the_procedure_the_oracle_names() {
        let cases: &[(&str, &[(&str, Result<&str, &str>)])] = &[
            // The BOTH style prepends whatever the third word is, and the
            // getter's key is the attribute's own name.
            (
                "::class k\n::attribute at external 'LIBRARY REXX zzz_no_entry'\n",
                &[
                    ("AT", Err("GETzzz_no_entry")),
                    ("AT=", Err("SETzzz_no_entry")),
                ],
            ),
            // No third word, so the procedure is the upcased name and the
            // prefix goes in front of that.
            (
                "::class k\n::attribute at external 'LIBRARY REXX'\n",
                &[("AT", Err("GETAT")), ("AT=", Err("SETAT"))],
            ),
            // A GET or SET style takes an explicit third word unchanged.
            (
                "::class k\n::attribute at get external 'LIBRARY REXX zzz_no_entry'\n",
                &[("AT", Err("zzz_no_entry"))],
            ),
            (
                "::class k\n::attribute at set external 'LIBRARY REXX zzz_no_entry'\n",
                &[("AT=", Err("zzz_no_entry"))],
            ),
            // and prefixes only the default one, which a third word spelled
            // like the upcased name also is.
            (
                "::class k\n::attribute at get external 'LIBRARY REXX'\n",
                &[("AT", Err("GETAT"))],
            ),
            (
                "::class k\n::attribute at get external 'LIBRARY REXX AT'\n",
                &[("AT", Err("GETAT"))],
            ),
            (
                "::class k\n::attribute at set external 'LIBRARY REXX AT'\n",
                &[("AT=", Err("SETAT"))],
            ),
            // The comparison is on the spelling, so a third word differing
            // only in case is not the default and resolves unchanged.
            (
                "::class k\n::attribute file_separator get external \
                 'LIBRARY REXX file_separator'\n",
                &[("FILE_SEPARATOR", Ok("file_separator"))],
            ),
            (
                "::class k\n::attribute file_separator get external \
                 'LIBRARY REXX FILE_SEPARATOR'\n",
                &[("FILE_SEPARATOR", Err("GETFILE_SEPARATOR"))],
            ),
            // An entry point the package exports, reached through each style.
            (
                "::class k\n::attribute at get external 'LIBRARY REXX file_separator'\n",
                &[("AT", Ok("file_separator"))],
            ),
            (
                "::class k\n::attribute at set external 'LIBRARY REXX FILE_SEPARATOR'\n",
                &[("AT=", Ok("file_separator"))],
            ),
            // `::METHOD ... ATTRIBUTE` has no style to choose and prepends
            // like the BOTH one.
            (
                "::class k\n::method m attribute external 'LIBRARY REXX file_separator'\n",
                &[
                    ("M", Err("GETfile_separator")),
                    ("M=", Err("SETfile_separator")),
                ],
            ),
            (
                "::class k\n::method m attribute external 'LIBRARY REXX'\n",
                &[("M", Err("GETM")), ("M=", Err("SETM"))],
            ),
        ];
        for (source, expected) in cases {
            let expected: Vec<(String, Result<&str, String>)> = expected
                .iter()
                .map(|(key, entry)| ((*key).to_owned(), entry.map_err(str::to_owned)))
                .collect();
            assert_eq!(accessors(&last_external(source)), expected, "{source:?}");
        }
    }

    /// The library is tested before the accessors are resolved, so a
    /// directive naming another library carries that library's own name and
    /// never a procedure this registry could not find. The accessor spelling
    /// is still the attribute shape's: measured, oracle, `::attribute at
    /// external "LIBRARY rxregexp NoSuchGet"` is `90.998 Unable to find
    /// external method "GETNoSuchGet".`
    #[test]
    fn a_library_other_than_rexx_names_itself_and_its_own_accessors() {
        let cases: &[(&str, &[&str])] = &[
            (
                "::class k\n::attribute at external 'LIBRARY zzznolib zzz'\n",
                &["GETzzz", "SETzzz"],
            ),
            (
                "::class k\n::attribute at get external 'LIBRARY zzznolib zzz'\n",
                &["zzz"],
            ),
            (
                "::class k\n::attribute at external 'LIBRARY zzznolib'\n",
                &["GETAT", "SETAT"],
            ),
            (
                "::class k\n::method m attribute external 'LIBRARY zzznolib zzz'\n",
                &["GETzzz", "SETzzz"],
            ),
            (
                "::class k\n::method m external 'LIBRARY zzznolib zzz'\n",
                &["zzz"],
            ),
        ];
        for (source, procedures) in cases {
            let MethodExternal::OtherLibrary { library, binds } = last_external(source) else {
                panic!("{source:?} did not name a library other than REXX");
            };
            assert_eq!(library, b"zzznolib".to_vec(), "{source:?}");
            let named: Vec<String> = binds
                .iter()
                .map(|bind| String::from_utf8_lossy(&bind.procedure).into_owned())
                .collect();
            assert_eq!(named, *procedures, "{source:?}");
        }
    }
}
