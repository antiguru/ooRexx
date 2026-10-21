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

use super::*;
use crate::Activation;
use crate::plan::{BodyKey, ProgramId};
use rexx_parse::{Program, parse_program};

/// Pushes a fresh top-level activation for `program`, the same setup
/// `Interp::run` does, so a test can drive `step` through a live
/// activation without the full instruction loop. Copied rather than
/// shared, matching every other test module in this crate (`eval.rs`,
/// `plan.rs`, `stem.rs` each keep their own).
fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
    let program = Rc::new(program);
    let program_id = interp.programs.insert(Rc::clone(&program));
    let plan = interp.plan_for(
        BodyKey {
            program: program_id,
            directive: None,
        },
        &program.main,
        &program.symbols,
        &program.source,
    );
    let frame = interp.roots.push_slots(plan.len());
    let id = interp.next_activation_id();
    interp.push_activation(Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    program
}

/// Parses `source`, activates it, and runs its whole body -- through
/// `Interp::run_activation` itself, not through a miniature of it.
fn run_source(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = activate(interp, program);
    run_activated(interp, &program)
}

/// `run_source`, with the program's directives installed first.
fn run_source_with_directives(
    interp: &mut Interp,
    source: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = Rc::new(program);
    let program_id = interp.programs.insert(Rc::clone(&program));
    interp.install_directives(program_id, &program)?;
    let plan = interp.plan_for(
        BodyKey {
            program: program_id,
            directive: None,
        },
        &program.main,
        &program.symbols,
        &program.source,
    );
    let frame = interp.roots.push_slots(plan.len());
    let id = interp.next_activation_id();
    interp.push_activation(Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    run_activated(interp, &program)
}

/// [`run_source_with_directives`], keeping what the program printed.
fn say_output_with_directives(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    run_source_with_directives(interp, source).expect("test program runs");
    std::mem::take(&mut interp.out)
}

/// A method whose own entry is taken back while it runs, and whose `Method`
/// objects a forced collection then sweeps, gets a `.context~executable`
/// rebuilt under the `.nil` scope it ran in. The rebuild keyed that object's
/// annotation table by `.nil` as if it were a class, and panicked.
#[test]
fn an_executable_rebuilt_under_the_nil_scope_answers() {
    let mut interp = Interp::new();
    let out = say_output_with_directives(
        &mut interp,
        b"say .t~new~go\n::class t\n::method go\n  \
          self~setMethod('x', 'self~unsetMethod(\"X\"); call gc \"force\"; \
          e = .context~executable; return e~class~id e~scope e~source~items')\n  \
          return self~x\n",
    );
    assert_eq!(String::from_utf8_lossy(&out), "Method The NIL object 1\n");
    assert_eq!(
        interp.executables_rebuilt, 1,
        "the rebuild path did not run"
    );
}

/// `run_source`'s second half, split out so `run_source_traced` can put a
/// `TRACE` setting on the activation between the push and the run.
fn run_activated(interp: &mut Interp, _program: &Program) -> Result<Option<ObjRef>, Failure> {
    interp.run_activation().map(Ended::value)
}

fn say_output(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    run_source(interp, source).expect("test program runs");
    std::mem::take(&mut interp.out)
}

/// `run_source`, with `TRACE R` already in force for the activation it
/// pushes.
fn run_source_traced(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let program = activate(interp, program);
    interp
        .set_trace_mode(crate::trace::mode_from_setting(b"r").expect("R is a valid TRACE setting"));
    run_activated(interp, &program)
}

fn say_output_traced(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
    run_source_traced(interp, source).expect("test program runs");
    std::mem::take(&mut interp.out)
}

/// A native frame's name for rexx-api resolves while that frame is live and
/// is refused once another frame holds its row, as a context kept past its
/// call names it.
#[test]
fn a_stale_native_frame_token_is_refused() {
    let mut interp = Interp::new();
    activate(&mut interp, parse_program(b"nop".to_vec()).expect("parses"));
    let caller = interp.activation().id;
    activate(&mut interp, parse_program(b"nop".to_vec()).expect("parses"));
    let frame = |id| {
        Box::new(crate::NativeFrame {
            owner: ObjRef::NIL,
            scope: ObjRef::NIL,
            method: false,
            receiver: ObjRef::NIL,
            name: Vec::new(),
            arguments: Vec::new(),
            argument_list: None,
            locals: rexx_api::handles::Table::new(),
            raised: None,
            additional: None,
            result: None,
            condition: None,
            code: None,
            executable: None,
            kept: rustc_hash::FxHashSet::default(),
            lent: Vec::new(),
            caller: Some(caller),
            id,
            packaged: false,
            reserved: false,
            temps_floor: 0,
        })
    };
    interp.activity.native_handles.push(frame(6));
    assert_eq!(interp.suspended_caller(6), Some(0));
    interp.activity.native_handles.pop();
    interp.activity.native_handles.push(frame(7));
    assert_eq!(interp.suspended_caller(7), Some(0));
    assert_eq!(interp.suspended_caller(6), None, "the old frame's token");
    assert_eq!(
        interp.suspended_caller(1 << 32 | 7),
        None,
        "a row no frame holds"
    );
}

mod address;
mod assignment;
mod branch;
mod conditions;
mod directives;
mod indent;
mod iterate_leave;
mod loops;
mod message;
mod numeric;
mod result;
mod routines;
mod scope;
mod signal;

/// Programs freed under collect-on-every-allocation give their slots to later
/// programs under a new generation: no id issued before a free resolves
/// again, and no row naming a freed id is left.
#[test]
fn a_freed_programs_id_never_resolves_again() {
    let mut interp = Interp::new();
    interp.enable_stress_collect();
    let out = say_output_with_directives(
        &mut interp,
        b"do i = 1 to 4\n  m = .Method~new('m', 'return' i)\n  m~setPrivate\n  \
          r = .Routine~new('r', 'return' i)\n  \
          p = .Package~new('p', ('::routine q public', 'return' i))\n  \
          say .runner~new~go(m) r~call p~findRoutine('Q')~call\nend\n\
          ::class runner\n::method go\n  use arg m\n  return self~run(m)\n",
    );
    assert_eq!(
        String::from_utf8_lossy(&out),
        "1 1 1\n2 2 2\n3 3 3\n4 4 4\n"
    );
    let issued = interp.programs.issued.clone();
    for (index, id) in issued.iter().enumerate() {
        assert!(!issued[..index].contains(id), "{id} was issued twice");
    }
    let freed = interp.programs.freed.clone();
    assert!(
        freed.iter().any(|stale| issued
            .iter()
            .any(|id| id != stale && id.slot_index() == stale.slot_index())),
        "no freed slot was reused, so no stale id was put to the test: {issued:?}"
    );
    assert!(
        !interp
            .annotations
            .keys()
            .any(|site| matches!(site, crate::environment::Annotated::Compiled(_))),
        "a compiled method's annotation table is held by a row of its own"
    );
    for method in interp.executable_objects.keys() {
        assert!(
            interp.method_bodies.contains_key(method)
                || interp.generated_methods.contains_key(method)
                || interp.native_externals.contains_key(method)
                || interp.library_externals.contains_key(method),
            "executable_objects names {method:?}, whose body was pruned"
        );
    }
    for id in &freed {
        let suffix = format!(" of program {id}");
        assert!(
            !interp
                .roots
                .global_names()
                .any(|name| name.ends_with(&suffix)),
            "a global root names {id}"
        );
        assert!(issued.contains(id), "{id} was freed and never issued");
        assert!(
            interp.programs.get(*id).is_none(),
            "{id} resolves after its free"
        );
        assert_eq!(interp.programs.package(*id), None, "{id}'s package");
        assert!(
            !interp.plans.keys().any(|key| key.program == *id),
            "{id} in plans"
        );
        assert!(
            !interp.chunks.keys().any(|(key, _)| key.program == *id),
            "{id} in chunks"
        );
        assert!(
            !interp.compiled_method_names.contains_key(id),
            "{id}'s name"
        );
        assert!(!interp.required_paths.contains_key(id), "{id}'s path");
        assert!(!interp.untranslated.contains(id), "{id} untranslated");
        assert!(
            !interp
                .method_bodies
                .values()
                .any(|body| body.program == *id),
            "{id} in method_bodies"
        );
        assert!(
            !interp.special_methods.rows().any(
                |scope| matches!(scope.package, crate::plan::Package::Program(program) if program == *id)
            ),
            "{id} in special_methods"
        );
        assert!(
            !interp
                .routine_objects
                .keys()
                .any(|installed| installed.program == *id),
            "{id} in routine_objects"
        );
        assert!(
            !interp
                .executable_sources
                .values()
                .any(|record| record.names_program(|program| program == id)),
            "{id} in executable_sources"
        );
        assert!(
            !interp.annotations.keys().any(|site| matches!(
                site,
                crate::environment::Annotated::Package(crate::plan::Package::Program(program))
                    | crate::environment::Annotated::Routine(program, _)
                    | crate::environment::Annotated::Unattached(program, _)
                    if program == id
            )),
            "{id} in annotations"
        );
    }
}

/// A collection drops every row keyed by a `Method`, `Routine` or `Message`
/// object it swept, and every annotation row of a class it swept, while the
/// program they came from stays loaded.
#[test]
fn a_collection_drops_the_rows_of_swept_executables_and_messages() {
    let mut interp = Interp::new();
    let out = say_output_with_directives(
        &mut interp,
        b".local['KEEP'] = .context~package\no = .array~of(1, 2)\ndo i = 1 to 3\n  \
          m = .Message~new(o, 'ITEMS'); m~send\n  s = .Message~new(o, 'ITEMS'); s~start\n  \
          c = .k~method('FOO')~copy; c~setPrivate\n  \
          t = .object~subclass('t'); a = t~annotations\nend\n\
          say m~completed c~isPrivate a~items\n\
          ::class k\n::method foo\n  return 1\n",
    );
    assert_eq!(String::from_utf8_lossy(&out), "1 1 0\n");
    let late = interp.run_started_to_end();
    assert!(late.is_empty(), "a started message failed");
    assert!(
        !interp.message_outcomes.is_empty(),
        "no message outcome was recorded"
    );
    assert!(
        !interp.started_messages.is_empty(),
        "no started message was recorded"
    );
    assert!(
        !interp.method_flag_writes.is_empty(),
        "no flag write was recorded"
    );
    assert!(
        interp
            .annotations
            .keys()
            .any(|site| matches!(site, crate::environment::Annotated::Class(_))),
        "no class annotation table was recorded"
    );
    interp.collect_now();
    let heap = &interp.heap;
    let live = |object: &ObjRef| heap.peek(*object).is_some();
    assert!(
        interp.executable_sources.keys().all(live),
        "executable_sources keeps a swept object"
    );
    assert!(
        interp.method_flag_writes.keys().all(live),
        "method_flag_writes keeps a swept object"
    );
    assert!(
        interp.executable_objects.values().all(live),
        "executable_objects names a swept object"
    );
    assert!(
        interp.message_outcomes.keys().all(live),
        "message_outcomes keeps a swept message"
    );
    assert!(
        interp.started_messages.iter().all(live),
        "started_messages keeps a swept message"
    );
    assert!(
        interp.unnotified_messages.iter().all(live),
        "unnotified_messages keeps a swept message"
    );
    assert!(
        interp
            .activities
            .message_rows()
            .all(|message| live(&message)),
        "the scheduler keeps a swept message"
    );
    assert!(
        interp.annotations.keys().all(|site| match site {
            crate::environment::Annotated::Class(class)
            | crate::environment::Annotated::Member(class, _, _) => live(class),
            _ => true,
        }),
        "annotations keeps a swept class's row"
    );
}

/// A collection drops every row of the method identity a swept `Method` copy
/// owned, and keeps the rows of the identity it was copied from.
#[test]
fn a_collection_drops_the_identity_rows_of_swept_method_copies() {
    let mut interp = Interp::new();
    let out = say_output_with_directives(
        &mut interp,
        b".local['KEEP'] = .context~package\ndo i = 1 to 3\n  \
          c = .k~method('FOO')~copy; c~setPrivate\n  \
          n = .object~method('STRING')~copy; n~setProtected\n  \
          g = .k~method('BAR')~copy\nend\n\
          say c~isPrivate n~isProtected .k~new~foo\n\
          ::class k\n::method foo\n  return 1\n::attribute bar\n",
    );
    assert_eq!(String::from_utf8_lossy(&out), "1 1 1\n");
    let owners: Vec<_> = interp
        .executable_sources
        .iter()
        .filter(|(_, record)| record.owns_installed)
        .filter_map(|(object, record)| Some((*object, record.installed?)))
        .collect();
    assert_eq!(owners.len(), 9, "the copies' identities were not recorded");
    let owned: Vec<_> = owners.iter().map(|(_, method)| *method).collect();
    let rows = |interp: &mut Interp, method: rexx_classes::MethodId| {
        [
            interp.method_bodies.contains_key(&method),
            interp.generated_methods.contains_key(&method),
            interp.special_methods.get(method).is_some(),
            interp.has_native_entry(method),
            interp.executable_objects.contains_key(&method),
        ]
    };
    for method in &owned {
        assert!(
            rows(&mut interp, *method).iter().any(|row| *row),
            "copy identity {method:?} has no row before the collection"
        );
    }
    let shared: Vec<_> = interp
        .method_bodies
        .keys()
        .chain(interp.generated_methods.keys())
        .filter(|method| !owned.contains(method))
        .copied()
        .collect();
    assert!(!shared.is_empty(), "no directive identity was recorded");
    interp.collect_now();
    let mut swept = 0;
    for (object, method) in &owners {
        if interp.heap.peek(*object).is_some() {
            assert!(
                rows(&mut interp, *method).iter().any(|row| *row),
                "copy identity {method:?} of a live object lost its rows"
            );
            continue;
        }
        swept += 1;
        assert_eq!(
            rows(&mut interp, *method),
            [false; 5],
            "copy identity {method:?} keeps a row after its object was swept"
        );
    }
    assert!(swept >= 6, "only {swept} copies were swept");
    for method in &shared {
        assert!(
            interp.method_bodies.contains_key(method)
                || interp.generated_methods.contains_key(method),
            "directive identity {method:?} lost its row"
        );
    }
}

/// How many objects the heap holds immortal after `source` ran and a
/// collection followed.
fn immortal_after(source: &[u8]) -> usize {
    let mut interp = Interp::new();
    say_output_with_directives(&mut interp, source);
    interp.collect_now();
    interp.heap.immortal_count()
}

/// The literals an `INTERPRET` fragment's chunk interned are released with
/// the chunk: three passes intern no more than one does.
#[test]
fn an_interpreted_literal_is_released_with_its_chunk() {
    let passes = |n: u32| {
        immortal_after(
            format!("do i = 1 to {n}\n  interpret 'x = \"' || copies('a', 50) || i || '\"'\nend\n")
                .as_bytes(),
        )
    };
    assert_eq!(passes(3), passes(1));
}

/// The literals a freed program's chunks interned are released with the
/// program, and a literal `::CONSTANT` is held by its package alone.
#[test]
fn a_freed_programs_literals_are_released_with_it() {
    let passes = |n: u32| {
        immortal_after(
            format!(
                "do i = 1 to {n}\n  m = .Method~new('m', 'return \"' || copies('b', 50) || i '\"')\n  \
                 x = .k~new~go(m)\n  \
                 p = .Package~new('p', ('::class c', '::constant v \"' || copies('c', 50) || i || '\"'))\n  \
                 y = p~classes['C']~new~v\nend\nsay length(x) length(y)\n\
                 ::class k\n::method go\n  use arg m\n  return self~run(m)\n"
            )
            .as_bytes(),
        )
    };
    assert_eq!(passes(3), passes(1));
}

/// A collection drops the notification and waiter-identity rows of the
/// messages it swept: a party raising in `messageComplete` leaves a message
/// unnotified, and a waiter identity is minted for one by hand.
#[test]
fn a_collection_drops_the_unnotified_and_waiter_rows_of_swept_messages() {
    let mut interp = Interp::new();
    interp.bootstrap_library().expect("library");
    let out = say_output_with_directives(
        &mut interp,
        b".local['KEEP'] = .context~package\no = .array~of(1, 2)\ndo i = 1 to 3\n  \
          call trybad o\nend\nsay 'ok'\nexit\n\
          trybad: procedure\n  use arg o\n  m = .Message~new(o, 'ITEMS'); m~notify(.bad~new)\n  \
          signal on syntax name s1\n  m~send\n  return\n  s1: return\n\
          ::class bad inherit MessageNotification\n::method messageComplete\n  \
          raise syntax 93.900 array('bad')\n",
    );
    assert_eq!(String::from_utf8_lossy(&out), "ok\n");
    assert!(
        !interp.unnotified_messages.is_empty(),
        "no unnotified message was recorded"
    );
    let message = *interp
        .message_outcomes
        .keys()
        .next()
        .expect("a message outcome");
    interp.activities.message_id(message);
    interp.collect_now();
    let heap = &interp.heap;
    let live = |object: &ObjRef| heap.peek(*object).is_some();
    assert!(heap.peek(message).is_none(), "the probe message survived");
    assert!(
        interp.unnotified_messages.iter().all(live),
        "unnotified_messages keeps a swept message"
    );
    assert!(
        interp
            .activities
            .message_rows()
            .all(|message| live(&message)),
        "message_ids keeps a swept message"
    );
}

/// Identities minted and dropped in a loop leave the method-row tables the
/// size one live copy needs: a private copy of a Rexx method and of a native
/// one each keep a row in the minted half, and the indexed half does not grow.
#[test]
fn minted_method_identities_do_not_grow_the_method_rows() {
    let mut interp = Interp::new();
    interp.bootstrap_library().expect("library");
    let extents =
        |interp: &mut Interp| (interp.special_methods.extent(), interp.native_rows_extent());
    let (specials, natives) = extents(&mut interp);
    let out = say_output_with_directives(
        &mut interp,
        b".local['KEEP'] = .context~package\ndo i = 1 to 200\n  \
          m = .k~method('FOO')~copy; m~setPrivate\n  \
          n = .object~method('STRING')~copy; n~setPrivate\nend\n\
          .local['KEEP2'] = (m, n)\nsay n~isPrivate\n::class k\n::method foo\n  return 1\n",
    );
    assert_eq!(String::from_utf8_lossy(&out), "1\n");
    interp.collect_now();
    assert_eq!(
        extents(&mut interp),
        ((specials.0, specials.1 + 2), (natives.0, natives.1 + 1)),
        "the method-row tables grew with the identities minted"
    );
}

/// A collection frees the behaviours `define` retired once no instance
/// holds them, and keeps the one a live instance holds.
#[test]
fn a_collection_frees_retired_behaviours_no_instance_holds() {
    let mut interp = Interp::new();
    interp.bootstrap_library().expect("library");
    let out = say_output_with_directives(
        &mut interp,
        b".local['KEEP'] = .context~package\n.local['OLD'] = .k~new\n\
          do i = 1 to 20\n  o = .k~new\n  .k~define('FOO', 'return' i)\nend\n\
          say .local['OLD']~foo o~foo .k~new~foo\ndrop o\n::class k\n::method foo\n  return 0\n",
    );
    assert_eq!(String::from_utf8_lossy(&out), "0 19 20\n");
    interp.collect_now();
    assert!(
        interp.has_retired_behaviours(),
        "the behaviour a held instance dispatches against was freed"
    );
    let out = say_output_with_directives(&mut interp, b".local['OLD'] = .nil\nsay 'gone'\n");
    assert_eq!(String::from_utf8_lossy(&out), "gone\n");
    interp.collect_now();
    assert!(
        !interp.has_retired_behaviours(),
        "a retired behaviour no instance holds survived a collection"
    );
}

/// A class lets go of the `Method` objects only freed behaviours named, so
/// `define` in a loop leaves the copies it made to the collector.
#[test]
fn a_class_lets_go_of_methods_only_freed_behaviours_named() {
    let mut interp = Interp::new();
    interp.bootstrap_library().expect("library");
    let baseline = interp.executable_sources.len();
    let out = say_output_with_directives(
        &mut interp,
        b".local['KEEP'] = .context~package\nm = .k~method('FOO')\n\
          do i = 1 to 200\n  .k~define('BAR', m)\nend\nsay .k~new~bar\n\
          ::class k\n::method foo\n  return 1\n",
    );
    assert_eq!(String::from_utf8_lossy(&out), "1\n");
    interp.collect_now();
    interp.collect_now();
    assert!(
        interp.executable_sources.len() < baseline + 20,
        "the class kept the copies define made: {baseline} -> {}",
        interp.executable_sources.len()
    );
}

/// A program's package is charged toward the next collection when it is
/// built.
#[test]
fn building_a_programs_package_charges_it() {
    let mut interp = Interp::new();
    interp.bootstrap_library().expect("library");
    let program = parse_program(b"nop\n".to_vec()).expect("parses");
    let id = interp.load_program(Rc::new(program));
    let before = interp.heap.charge_body_bytes(0);
    interp
        .package_object(crate::plan::Package::Program(id))
        .expect("a package");
    let after = interp.heap.charge_body_bytes(0);
    assert!(
        after >= before + crate::PACKAGE_BYTES,
        "the package was not charged: {before} -> {after}"
    );
}

/// A collection drops the method-identity rows of the `loadExternalMethod`
/// answers it swept, for a `LIBRARY REXX` entry point and a shared library's
/// procedure, flag writes included.
#[test]
fn a_collection_drops_the_rows_of_swept_loaded_methods() {
    let mut interp = Interp::new();
    // Where `rxregexp` is: the oracle's own libraries, as `tests/support/oracle.rs` names them.
    interp.library_search = vec![std::path::PathBuf::from(
        "/home/moritz/dev/repos/ooRexx/build/lib",
    )];
    interp.bootstrap_library().expect("library");
    let rows = |interp: &Interp| {
        (
            interp.defined_library_codes.len(),
            interp.library_externals.len(),
            interp.native_externals.len(),
            interp.special_methods.rows().count(),
        )
    };
    let baseline = rows(&interp);
    let out = say_output_with_directives(
        &mut interp,
        b".local['KEEP'] = .context~package\ndo i = 1 to 3\n  \
          m = .Method~loadExternalMethod('m', 'LIBRARY REXX stream_uninit'); m~setPrivate\n  \
          n = .Method~loadExternalMethod('n', 'LIBRARY rxregexp RegExp_Match'); n~setPrivate\n  \
          c = n~copy\nend\nsay m~class~id n~class~id c~isPrivate\ndrop m n c\n",
    );
    assert_eq!(String::from_utf8_lossy(&out), "Method Method 1\n");
    let (codes, libraries, natives, specials) = rows(&interp);
    assert!(
        codes > baseline.0
            && libraries > baseline.1
            && natives > baseline.2
            && specials > baseline.3,
        "a loaded method left no row: {baseline:?} -> {:?}",
        rows(&interp)
    );
    interp.collect_now();
    assert_eq!(rows(&interp), baseline, "a swept loaded method kept a row");
}

/// A `::CLASS` is charged once its own `::METHOD`s are installed: the same
/// source with the methods attached to the class is charged more than with
/// them ahead of it, attached to none.
#[test]
fn a_class_directive_is_charged_with_its_methods() {
    let methods: String = (1..=20).map(|n| format!("::method m{n}\n")).collect();
    let charged = |source: String| {
        let mut interp = Interp::new();
        interp.bootstrap_library().expect("library");
        let collections = interp.heap.collections_performed();
        let before = interp.heap.charge_body_bytes(0);
        say_output_with_directives(&mut interp, source.as_bytes());
        assert_eq!(
            interp.heap.collections_performed(),
            collections,
            "a collection reset the charge"
        );
        interp.heap.charge_body_bytes(0) - before
    };
    let attached = charged(format!("nop\n::class c\n{methods}"));
    let unattached = charged(format!("nop\n{methods}::class c\n"));
    assert!(
        attached >= unattached + 20 * (8 << 10),
        "the class's methods were not charged: {attached} against {unattached}"
    );
}
