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
//!
//! # Which `EXTERNAL` form binds here
//!
//! A `::METHOD` or a `::ATTRIBUTE` whose `EXTERNAL` names the library `REXX`.
//! [`method_external`] and [`attribute_external`] are the two places that
//! decide it, they share the library test, and `crate::directive_gap` reads
//! them, so the forms that bind and the forms that are refused cannot come
//! apart.
//!
//! What stays refused, each with the oracle's own answer measured beside it:
//!
//! ```text
//! ::routine r external 'LIBRARY REXX Filespec'     rc 0, and the routine
//!                                                  runs: it resolves
//!                                                  against a routine table
//!                                                  this registry is not
//! ::routine r external 'LIBRARY zzznolib zzzr'     98.903 rc 158
//! ::attribute at external 'LIBRARY zzznolib zzz'   98.903 rc 158
//! ::method m external 'LIBRARY zzznolib zzzr'      98.903 rc 158
//! ```
//!
//! Every `::ROUTINE` spelling is one row of gate table D and its probe names
//! a shared library, so the `LIBRARY REXX` routine form has no row of its own
//! there either; `tests/gate_table_d.rs`'s `owning_phase` carries that.
//!
//! # The library name is compared exactly and the entry point caselessly
//!
//! `PackageManager::loadLibrary` looks the library up with
//! `packages->get(name)` (`package/PackageManager.cpp:233`), whose key is the
//! `REXX` that `loadInternalPackage(GlobalNames::REXX, rexxPackage)` put
//! there (`package/PackageManager.cpp:86`), while
//! `LibraryPackage::locateMethodEntry` walks the exported table with
//! `strCaselessCompare` (`package/LibraryPackage.cpp:324`). Measured, oracle,
//! both halves: `::method m external "library rexx file_separator"` is
//! `98.903 Unable to load library "rexx"` at rc 158, and `::method m external
//! "LIBRARY REXX FILE_SEPARATOR"` is rc 0.
//!
//! **`words()` upcases the first word only**, so `library` reaching
//! `LibraryPackage` as `rexx` is not a parse question -- `rexx-parse`'s
//! `decode_external` already keeps the library name's case.
//!
//! # A missing third word is the method's upcased name
//!
//! `decodeExternalMethod` opens with `procedure = methodName`, the upcased
//! `internalname` (`parser/DirectiveParser.cpp:1406`), and only a third word
//! replaces it. Measured, oracle: `::method m external "LIBRARY REXX"` is
//! `90.998 Unable to find external method "M"` at rc 166, naming the upcased
//! spelling, and `::method file_temporary_path external "LIBRARY REXX"` is
//! rc 0 -- the caseless compare above is what makes the upcased default reach
//! a table spelled in lower case. `StreamClasses.orx:510` is that shape.
//!
//! An accessor's procedure is that name with `GET` or `SET` in front of it
//! instead, and [`attribute_binds`] is where the rule and its measurements
//! are.
//!
//! # Binding is eager and a miss stops the program before its prologue
//!
//! `createNativeMethod` raises `Error_External_name_not_found_method` from
//! inside the directive parse (`parser/DirectiveParser.cpp:1385`), so the
//! whole file is refused rather than the method. Measured, oracle: a file
//! whose first clause is `say "prolog ran"` carrying one `::METHOD EXTERNAL`
//! that names a missing entry point is rc 166 with **empty stdout**, and the
//! same file naming `file_separator` is rc 0 printing `prolog ran`.
//! `crate::Interp::install_directives` is where this crate does it, in the
//! walk that answers every other translation-time refusal.
//!
//! # What an entry this phase does not implement does
//!
//! It resolves, and a send to it is loud (`crate::NOT_IMPLEMENTED_EXIT`),
//! naming the entry point and the phase [`Family::owner`] gives it. Resolving
//! is not implementing: what the eager bind decides is whether the *file*
//! installs, which is what `CoreClasses.orx` and `StreamClasses.orx` need,
//! and every one of their methods still has a body somewhere else.
//!
//! **The corpus differential cannot see one of these refusals become a wrong
//! answer**, because the oracle answers where this crate declines: measured,
//! `.k~ch` on `::method ch class external 'LIBRARY REXX stream_chars'` is the
//! oracle's `48.1 Failure in system service: Stream not initialized.` at rc
//! 208. So the instruments are `tests/native_entries.rs`, which runs one
//! program per family through both engines and pins the refusal, and this
//! module's own tests over the table.

use rexx_parse::{AttributeDirective, AttributeStyle, ExternalSpec, MethodDirective};

use super::{Arity, NativeMethod, native_file_path_separator, native_file_separator};
use crate::{Loud, accessor_setter_name};

/// Which part of the interpreter an entry point belongs to, and through that
/// which phase owes it a body.
///
/// The split is the C++ tree's own: every name in the registry is defined by
/// exactly one translation unit, checked by grepping each name's
/// `RexxMethodN` definition, and each variant here is one of those units.
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
    /// The phase that owes this family a body, spelled the way every other
    /// owner string in this crate is.
    ///
    /// **Timers are Phase 6's because nothing reaches them on one activity.**
    /// `Alarm~init` is `reply` and then `self~!startTimer(...)`
    /// (`RexxClasses/CoreClasses.orx:1555`, `:1557`), so the entry point runs
    /// on the activity the `REPLY` split off, and this crate already refuses
    /// the `REPLY` forms that would create one with `(Phase 6)`.
    ///
    /// **Stream, queue and file are Phase 7's**, each by a decision the
    /// roadmap states rather than by the file they sit in: D11 puts `.File`
    /// and the file-system surface there, the Phase 7 notes put the stream
    /// model there, and D7 puts external queues there
    /// (`docs/superpowers/plans/2026-07-27-rust-rewrite.md`).
    pub(crate) fn owner(self) -> &'static str {
        match self {
            Family::Timer => "Phase 6",
            Family::Stream | Family::Queue | Family::File => "Phase 7",
        }
    }

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
///
/// **The bodies themselves are written next door in `dispatch.rs`**, beside
/// the primitives, and this table points at them. A body's parameter list
/// carries the seam's own clearance token, and
/// `tests/dispatch_seam.rs`'s `the_seam_token_is_named_only_by_the_dispatch_module`
/// bounds the functions that can consume one by the single file that names
/// it -- so writing them here would cost that bound a second file to read.
pub(in crate::dispatch) enum ExternalBody {
    /// A body this phase runs, with the parameter count the C++ entry
    /// declares.
    Implemented { arity: Arity, run: NativeMethod },
    /// A body [`Family::owner`]'s phase owes. The bind still succeeds; the
    /// send is loud.
    Deferred,
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

const fn deferred(name: &'static str, family: Family) -> NativeExternal {
    NativeExternal {
        name,
        family,
        body: ExternalBody::Deferred,
    }
}

/// The `REXX` package's exported method table.
///
/// **The names and their order are `interpreter/runtime/NativeMethods.h`'s**,
/// which `InternalPackage.cpp` expands into `rexx_methods[]` and hands to
/// `rexx_package_entry` under the package name `REXX`
/// (`runtime/InternalPackage.cpp:213`, `:241`).
/// `interpreter/platform/unix/SysNativeMethods.h`, which the same expansion
/// includes right after it, adds nothing on this platform: its whole body is
/// the comment "Unix doesn't currently have any of these."
///
/// **This is the whole set the bootstrap needs, and that is measured rather
/// than assumed**: the entry points `CoreClasses.orx`, `StreamClasses.orx`
/// and `platform/unix/PlatformObjects.orx` declare and the names here are the
/// same set, compared caselessly and in both directions, by
/// `the_bootstrap_files_and_the_registry_name_the_same_entry_points`.
static LIBRARY_REXX_METHODS: &[NativeExternal] = &[
    deferred("alarm_startTimer", Family::Timer),
    deferred("alarm_stopTimer", Family::Timer),
    deferred("ticker_createTimer", Family::Timer),
    deferred("ticker_waitTimer", Family::Timer),
    deferred("ticker_stopTimer", Family::Timer),
    deferred("stream_init", Family::Stream),
    deferred("stream_chars", Family::Stream),
    deferred("stream_lines", Family::Stream),
    deferred("stream_position", Family::Stream),
    deferred("stream_state", Family::Stream),
    deferred("stream_description", Family::Stream),
    deferred("stream_query_position", Family::Stream),
    deferred("stream_charout", Family::Stream),
    deferred("stream_charin", Family::Stream),
    deferred("stream_linein", Family::Stream),
    deferred("stream_lineout", Family::Stream),
    deferred("stream_arrayin", Family::Stream),
    deferred("qualify", Family::Stream),
    deferred("query_exists", Family::Stream),
    deferred("query_size", Family::Stream),
    deferred("query_time", Family::Stream),
    deferred("handle_set", Family::Stream),
    deferred("std_set", Family::Stream),
    deferred("stream_flush", Family::Stream),
    deferred("query_handle", Family::Stream),
    deferred("query_streamtype", Family::Stream),
    deferred("stream_close", Family::Stream),
    deferred("stream_uninit", Family::Stream),
    deferred("stream_open", Family::Stream),
    deferred("rexx_create_queue", Family::Queue),
    deferred("rexx_open_queue", Family::Queue),
    deferred("rexx_queue_exists", Family::Queue),
    deferred("rexx_delete_queue", Family::Queue),
    deferred("rexx_query_queue", Family::Queue),
    deferred("rexx_push_queue", Family::Queue),
    deferred("rexx_queue_queue", Family::Queue),
    deferred("rexx_pull_queue", Family::Queue),
    deferred("rexx_linein_queue", Family::Queue),
    deferred("rexx_clear_queue", Family::Queue),
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
    deferred("file_case_sensitive", Family::File),
    deferred("file_list_roots", Family::File),
    deferred("file_qualify", Family::File),
    deferred("file_exists", Family::File),
    deferred("file_delete_file", Family::File),
    deferred("file_delete_directory", Family::File),
    deferred("file_isDirectory", Family::File),
    deferred("file_isFile", Family::File),
    deferred("file_isHidden", Family::File),
    deferred("file_get_last_modified", Family::File),
    deferred("file_set_last_modified", Family::File),
    deferred("file_set_read_only", Family::File),
    deferred("file_length", Family::File),
    deferred("file_list", Family::File),
    deferred("file_make_dir", Family::File),
    deferred("file_can_read", Family::File),
    deferred("file_can_write", Family::File),
    deferred("file_rename", Family::File),
    deferred("this_file_case_sensitive", Family::File),
    deferred("file_get_last_accessed", Family::File),
    deferred("file_set_last_accessed", Family::File),
    deferred("file_set_writable", Family::File),
    deferred("file_temporary_path", Family::File),
    deferred("file_search_path_impl", Family::File),
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
            owner: entry.family.owner(),
            implemented: matches!(entry.body, ExternalBody::Implemented { .. }),
        })
}

/// The entry point the `REXX` package exports under `name`, or `None` for a
/// name it does not export.
///
/// A linear walk with a caseless compare, which is `locateMethodEntry`'s own
/// shape (`package/LibraryPackage.cpp:313`, whose compare is `:324`). It runs once per
/// `::METHOD ... EXTERNAL` directive at install and never on a send.
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
    /// `'LIBRARY <name>'` for any name but `REXX`: a shared library to load.
    OtherLibrary,
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
///
/// `upper` is the directive's own name upcased, which is `internalname` in
/// both parsers. The procedure is the specification's third word or `upper`
/// where there is none, and an accessor's own procedure is that with `GET` or
/// `SET` in **front** of it -- `concatToCstring` appends its receiver to its
/// argument (`classes/StringClass.cpp:1405`-`:1416`).
///
/// [`AttributeStyle::Both`] prepends unconditionally
/// (`parser/DirectiveParser.cpp:1678`-`:1679`, and `:867`-`:868` for the
/// `::METHOD` spelling, which reaches the same code). A `GET` or `SET` style
/// prepends only where the procedure is the default one, which the C++ asks
/// as `internalname == procedure` (`parser/DirectiveParser.cpp:1737`, `:1802`)
/// over strings both interned by `commonString` -- so the test is on the
/// spelling, and an explicit third word spelled exactly like the upcased name
/// takes the prefix too. Measured, oracle, one program per row:
///
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
    let procedure: Vec<u8> = match &spec.entry {
        Some(entry) => entry.to_vec(),
        None => upper.to_vec(),
    };
    let prefixed = |prefix: &str| [prefix.as_bytes(), &procedure].concat();
    let defaulted = *procedure == *upper;
    let accessors = match style {
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
    };
    accessors
        .into_iter()
        .map(|(key, procedure)| AttributeBind {
            key,
            entry: entry_point(&procedure).ok_or(procedure),
        })
        .collect()
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
        return Some(MethodExternal::OtherLibrary);
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
    if *spec.library != *b"REXX" {
        return Some(MethodExternal::OtherLibrary);
    }
    Some(MethodExternal::Attribute(attribute_binds(
        &attribute.name.to_ascii_uppercase(),
        attribute.style,
        spec,
    )))
}

/// The entry point one dictionary key is bound to, or `None` for a key this
/// `EXTERNAL` does not bind, for a procedure the `REXX` package does not
/// export, and for a directive carrying no `EXTERNAL`.
///
/// A [`MethodExternal::LibraryRexx`] answers whatever key it is asked about,
/// because the directive it comes from claims exactly one: `DELEGATE` is what
/// gives a plain `::METHOD` a second key, and measured, oracle, `::method m
/// delegate p external 'LIBRARY REXX file_separator'` is `25.902 Unknown
/// keyword on ::METHOD directive; found "EXTERNAL".` at rc 231.
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
        MethodExternal::OtherLibrary => None,
    }
}

/// The first procedure an `EXTERNAL` names that the `REXX` package does not
/// export -- the one the oracle reports 90.998 for -- or `None` when every
/// accessor resolved.
///
/// First in [`attribute_binds`]' order, which is the order the C++ calls
/// `createNativeMethod` in (`parser/DirectiveParser.cpp:1682`, `:1689`):
/// measured, oracle, `::attribute at external 'LIBRARY REXX zzz_no_entry'`
/// names `GETzzz_no_entry` and never `SETzzz_no_entry`.
pub(crate) fn unresolved_entry(external: Option<&MethodExternal>) -> Option<&[u8]> {
    match external? {
        MethodExternal::LibraryRexx(entry) => entry.as_ref().err().map(Vec::as_slice),
        MethodExternal::Attribute(binds) => binds
            .iter()
            .find_map(|bind| bind.entry.as_ref().err().map(Vec::as_slice)),
        MethodExternal::OtherLibrary => None,
    }
}

/// The refusal a send to an entry point this phase does not implement gets.
pub(crate) fn deferred_send(entry: &NativeExternal) -> Loud {
    Loud {
        message: crate::owned_message(
            &format!("the LIBRARY REXX entry point \"{}\"", entry.name),
            Some(entry.family.owner()),
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
    ///
    /// Hardcoded for the reason `tests/gate_tables/orx.rs`'s `orx_root` is:
    /// this crate is checked against *that* tree, and a variable pointing
    /// somewhere else would let a different checkout answer for it with
    /// nothing to notice. **`platform/unix/`, and only unix**: the windows
    /// file is a separate working copy of the same name, unread here exactly
    /// as it is unembedded by the bootstrap.
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
    ///
    /// **Parsed, so that what is asked about a directive is what the
    /// interpreter will ask.** A regular expression over the text would agree
    /// with the parser on every line these files happen to hold today and
    /// would stop agreeing the moment one of them is written differently --
    /// `StreamClasses.orx:510` is already a lower-case `library` with no
    /// third word.
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
    ///
    /// **This is the "no unresolved external" half of the bootstrap, checkable
    /// before the bootstrap runs.** Installing the files needs everything else
    /// Phase 5a builds; resolving their entry points needs only this table, so
    /// it is asked here rather than left for whatever drives them.
    ///
    /// **The `reached` guard is global, and it cannot be made per file.**
    /// `PlatformObjects.orx` declares no `EXTERNAL` at all, so it contributes
    /// nothing to the walk and a per-file bound on it would be a bound of
    /// zero. Nor can this test check that [`bootstrap_files`] still names the
    /// files it should: that list is the only statement of which files those
    /// are, so comparing the walk against it compares a list with itself.
    /// What does catch a file dropping out is
    /// [`the_bootstrap_files_and_the_registry_name_the_same_entry_points`],
    /// through the names the file contributes -- which is why the file that
    /// contributes none is the one nothing here can speak for.
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
            MethodExternal::OtherLibrary => panic!(
                "{} declares a {keyword} whose EXTERNAL names a library other than \
                 REXX, which this phase refuses, so the file cannot install",
                path.display()
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
    ///
    /// **Both directions.** A name the files need and the registry lacks is a
    /// file that cannot install, which the test above already reddens on; a
    /// name the registry holds and no file uses is a row nothing exercises,
    /// and reddening on it is how a hand-written table's drift from
    /// `NativeMethods.h` gets noticed at all -- the C++ tree gaining an entry
    /// point moves this, and nothing else in this crate would.
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
    ///
    /// `tests/native_entries.rs` is what runs those programs; this is the
    /// half of the pair that says none of them was made unwritable by an
    /// entry point moving into [`ExternalBody::Implemented`].
    #[test]
    fn every_family_still_defers_something() {
        for family in LIBRARY_REXX_METHODS.iter().map(|entry| entry.family) {
            assert!(
                LIBRARY_REXX_METHODS.iter().any(|entry| {
                    entry.family == family && matches!(entry.body, ExternalBody::Deferred)
                }),
                "the {} family implements every entry point it holds, so no program \
                 can pin its refusal",
                family.label()
            );
        }
    }

    /// The lookup is caseless and the spelling the table carries is not what
    /// makes a name resolve.
    ///
    /// The upcased spelling is the one an `EXTERNAL` with no third word
    /// produces, so this is the path `StreamClasses.orx:510` takes.
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
    ///
    /// Each row's *reported* procedure is a measurement on the oracle, and
    /// [`attribute_binds`] carries the transcripts. Asked of the directive
    /// rather than of a program, because a row whose entry point resolves runs
    /// to rc 0 and prints nothing that names the procedure it resolved -- so
    /// the corpus programs beside this cannot tell `GET`-prefixed from
    /// unprefixed on exactly the rows where the prefix rule is subtlest.
    ///
    /// **The setter half of a pair whose getter already failed is not
    /// observable on the oracle at all**, since the file stops at the getter,
    /// and those cells come from `attributeDirective`'s own two
    /// `createNativeMethod` calls (`parser/DirectiveParser.cpp:1682`,
    /// `:1689`) rather than from a run.
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

    /// The library is tested before the accessors are built, so a directive
    /// naming another library is that refusal and never a procedure this
    /// registry could not find.
    ///
    /// Measured, oracle: `::method m attribute external 'LIBRARY nosuchlib x'`
    /// is `98.903 Unable to load library "nosuchlib"` at rc 158, and the
    /// `::attribute` spelling of it is the same.
    #[test]
    fn a_library_other_than_rexx_is_refused_before_any_accessor_is_built() {
        for source in [
            "::class k\n::attribute at external 'LIBRARY zzznolib zzz'\n",
            "::class k\n::attribute at get external 'LIBRARY zzznolib zzz'\n",
            "::class k\n::attribute at external 'LIBRARY zzznolib'\n",
            "::class k\n::method m attribute external 'LIBRARY zzznolib zzz'\n",
            "::class k\n::method m external 'LIBRARY zzznolib zzz'\n",
        ] {
            assert!(
                matches!(last_external(source), MethodExternal::OtherLibrary),
                "{source:?}"
            );
        }
    }
}
