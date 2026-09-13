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

//! The `#[repr(C)]` surface measured against the frozen header.
//!
//! Member names and interface versions are read out of `api/oorexxapi.h` here
//! rather than written down, because an omitted member is what a written list
//! cannot see. The sizes and offsets are what `g++` reports for the same
//! header on this platform.

use rexx_api::layout::{
    CallContextInterface, ExitContextInterface, IORedirectorInterface, MethodContextInterface,
    Owned, RexxCallContext_, RexxCondition, RexxExitContext_, RexxIORedirectorContext_,
    RexxInstance_, RexxInstanceInterface, RexxMethodContext_, RexxMethodEntry, RexxPackageEntry,
    RexxRoutineEntry, RexxThreadContext_, RexxThreadInterface, ValueDescriptor,
};
use std::mem::{align_of, offset_of, size_of};
use std::path::PathBuf;

/// The frozen header (D5), which `api/` holds read-only.
fn header() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../api/oorexxapi.h");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// `line` with any `//` comment cut off.
fn code_of(line: &str) -> &str {
    match line.find("//") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// The trailing identifier of `text`, which for a member declaration is its
/// name.
fn trailing_name(text: &str) -> String {
    text.chars()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}

/// The member names `struct` declares, in declaration order.
///
/// A member is either `RET (RexxEntry *Name)(...)` or a plain declaration.
fn members_of(header: &str, name: &str) -> Vec<String> {
    let lines: Vec<&str> = header.lines().collect();
    let close = format!("}} {name};");
    let end = lines
        .iter()
        .position(|line| line.trim() == close)
        .unwrap_or_else(|| panic!("{name} has no closing brace in the header"));
    let start = lines[..end]
        .iter()
        .rposition(|line| line.trim() == "typedef struct")
        .unwrap_or_else(|| panic!("{name} has no opening typedef in the header"));

    let mut found = Vec::new();
    for line in &lines[start + 1..end] {
        let code = code_of(line).trim();
        if code.is_empty() || code == "{" {
            continue;
        }
        let declaration = code
            .strip_suffix(';')
            .unwrap_or_else(|| panic!("{name}: {code} is not a member declaration"));
        match declaration.find("(RexxEntry") {
            Some(at) => {
                let rest = declaration[at + "(RexxEntry".len()..].trim_start();
                let rest = rest.strip_prefix('*').expect("a function pointer member");
                let entry: String = rest
                    .trim_start()
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                found.push(entry);
            }
            None => found.push(trailing_name(declaration)),
        }
    }
    found
}

/// The value of `#define <name> <integer>`.
fn defined(header: &str, name: &str) -> isize {
    let needle = format!("#define {name} ");
    let line = header
        .lines()
        .find(|line| line.starts_with(&needle))
        .unwrap_or_else(|| panic!("{name} is not defined in the header"));
    line[needle.len()..]
        .trim()
        .parse()
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn every_interface_declares_the_members_the_header_does() {
    let header = header();
    for (name, fields) in [
        ("RexxInstanceInterface", RexxInstanceInterface::FIELDS),
        ("RexxThreadInterface", RexxThreadInterface::FIELDS),
        ("MethodContextInterface", MethodContextInterface::FIELDS),
        ("CallContextInterface", CallContextInterface::FIELDS),
        ("ExitContextInterface", ExitContextInterface::FIELDS),
        ("IORedirectorInterface", IORedirectorInterface::FIELDS),
    ] {
        assert_eq!(
            members_of(&header, name),
            fields,
            "{name} does not declare what the frozen header declares"
        );
    }
}

/// The negative control for the scan above: it reads one struct's body and
/// not the whole header, so an empty or over-wide match would show here.
#[test]
fn the_scan_reads_one_struct_at_a_time() {
    let header = header();
    let thread = members_of(&header, "RexxThreadInterface");
    let method = members_of(&header, "MethodContextInterface");
    assert!(thread.contains(&"SendMessage".to_string()));
    assert!(method.contains(&"GetMessageName".to_string()));
    assert!(
        !thread.contains(&"GetMessageName".to_string()),
        "the scan is reading past the struct it was asked for"
    );
    assert!(
        !method.contains(&"SendMessage".to_string()),
        "the scan is reading past the struct it was asked for"
    );
}

#[test]
fn every_interface_version_is_the_frozen_headers() {
    let header = header();
    for (name, ours) in [
        (
            "INSTANCE_INTERFACE_VERSION",
            rexx_api::layout::INSTANCE_INTERFACE_VERSION,
        ),
        (
            "THREAD_INTERFACE_VERSION",
            rexx_api::layout::THREAD_INTERFACE_VERSION,
        ),
        (
            "METHOD_INTERFACE_VERSION",
            rexx_api::layout::METHOD_INTERFACE_VERSION,
        ),
        (
            "CALL_INTERFACE_VERSION",
            rexx_api::layout::CALL_INTERFACE_VERSION,
        ),
        (
            "EXIT_INTERFACE_VERSION",
            rexx_api::layout::EXIT_INTERFACE_VERSION,
        ),
        (
            "REDIRECT_INTERFACE_VERSION",
            rexx_api::layout::REDIRECT_INTERFACE_VERSION,
        ),
    ] {
        assert_eq!(defined(&header, name), ours, "{name}");
    }
}

#[test]
fn every_interface_is_one_word_per_member() {
    let word = size_of::<usize>();
    for (name, fields, size) in [
        (
            "RexxInstanceInterface",
            RexxInstanceInterface::FIELDS,
            size_of::<RexxInstanceInterface>(),
        ),
        (
            "RexxThreadInterface",
            RexxThreadInterface::FIELDS,
            size_of::<RexxThreadInterface>(),
        ),
        (
            "MethodContextInterface",
            MethodContextInterface::FIELDS,
            size_of::<MethodContextInterface>(),
        ),
        (
            "CallContextInterface",
            CallContextInterface::FIELDS,
            size_of::<CallContextInterface>(),
        ),
        (
            "ExitContextInterface",
            ExitContextInterface::FIELDS,
            size_of::<ExitContextInterface>(),
        ),
        (
            "IORedirectorInterface",
            IORedirectorInterface::FIELDS,
            size_of::<IORedirectorInterface>(),
        ),
    ] {
        assert_eq!(size, fields.len() * word, "{name}");
    }
}

#[test]
fn the_populated_tables_carry_the_interface_version() {
    assert_eq!(
        RexxThreadInterface::REFUSING.interfaceVersion,
        rexx_api::layout::THREAD_INTERFACE_VERSION
    );
    assert_eq!(
        MethodContextInterface::REFUSING.interfaceVersion,
        rexx_api::layout::METHOD_INTERFACE_VERSION
    );
}

#[test]
fn an_instance_has_a_table_and_application_data() {
    assert_eq!(size_of::<RexxInstance_>(), 16);
    assert_eq!(align_of::<RexxInstance_>(), 8);
    assert_eq!(offset_of!(RexxInstance_, functions), 0);
    assert_eq!(offset_of!(RexxInstance_, applicationData), 8);
}

#[test]
fn a_thread_context_has_an_instance_and_a_table() {
    assert_eq!(size_of::<RexxThreadContext_>(), 16);
    assert_eq!(align_of::<RexxThreadContext_>(), 8);
    assert_eq!(offset_of!(RexxThreadContext_, instance), 0);
    assert_eq!(offset_of!(RexxThreadContext_, functions), 8);
}

#[test]
fn a_method_context_has_a_thread_a_table_and_arguments() {
    assert_eq!(size_of::<RexxMethodContext_>(), 24);
    assert_eq!(align_of::<RexxMethodContext_>(), 8);
    assert_eq!(offset_of!(RexxMethodContext_, threadContext), 0);
    assert_eq!(offset_of!(RexxMethodContext_, functions), 8);
    assert_eq!(offset_of!(RexxMethodContext_, arguments), 16);
}

#[test]
fn a_call_context_is_shaped_as_a_method_context() {
    assert_eq!(size_of::<RexxCallContext_>(), 24);
    assert_eq!(offset_of!(RexxCallContext_, threadContext), 0);
    assert_eq!(offset_of!(RexxCallContext_, functions), 8);
    assert_eq!(offset_of!(RexxCallContext_, arguments), 16);
}

#[test]
fn an_exit_context_is_shaped_as_a_method_context() {
    assert_eq!(size_of::<RexxExitContext_>(), 24);
    assert_eq!(offset_of!(RexxExitContext_, threadContext), 0);
    assert_eq!(offset_of!(RexxExitContext_, functions), 8);
    assert_eq!(offset_of!(RexxExitContext_, arguments), 16);
}

#[test]
fn a_redirector_context_is_a_table_alone() {
    assert_eq!(size_of::<RexxIORedirectorContext_>(), 8);
    assert_eq!(offset_of!(RexxIORedirectorContext_, functions), 0);
}

#[test]
fn a_value_descriptor_is_a_word_and_two_shorts() {
    assert_eq!(size_of::<ValueDescriptor>(), 16);
    assert_eq!(align_of::<ValueDescriptor>(), 8);
    assert_eq!(offset_of!(ValueDescriptor, value), 0);
    assert_eq!(offset_of!(ValueDescriptor, r#type), 8);
    assert_eq!(offset_of!(ValueDescriptor, flags), 10);
}

#[test]
fn a_condition_is_what_decode_condition_info_fills() {
    assert_eq!(size_of::<RexxCondition>(), 72);
    assert_eq!(offset_of!(RexxCondition, code), 0);
    assert_eq!(offset_of!(RexxCondition, rc), 8);
    assert_eq!(offset_of!(RexxCondition, position), 16);
    assert_eq!(offset_of!(RexxCondition, conditionName), 24);
    assert_eq!(offset_of!(RexxCondition, message), 32);
    assert_eq!(offset_of!(RexxCondition, errortext), 40);
    assert_eq!(offset_of!(RexxCondition, program), 48);
    assert_eq!(offset_of!(RexxCondition, description), 56);
    assert_eq!(offset_of!(RexxCondition, additional), 64);
}

#[test]
fn a_package_entry_is_what_an_extension_publishes() {
    assert_eq!(size_of::<RexxPackageEntry>(), 64);
    assert_eq!(align_of::<RexxPackageEntry>(), 8);
    assert_eq!(offset_of!(RexxPackageEntry, size), 0);
    assert_eq!(offset_of!(RexxPackageEntry, api_version), 4);
    assert_eq!(offset_of!(RexxPackageEntry, required_version), 8);
    assert_eq!(offset_of!(RexxPackageEntry, package_name), 16);
    assert_eq!(offset_of!(RexxPackageEntry, package_version), 24);
    assert_eq!(offset_of!(RexxPackageEntry, loader), 32);
    assert_eq!(offset_of!(RexxPackageEntry, unloader), 40);
    assert_eq!(offset_of!(RexxPackageEntry, routines), 48);
    assert_eq!(offset_of!(RexxPackageEntry, methods), 56);
}

#[test]
fn a_table_row_is_what_the_entry_macros_expand_to() {
    assert_eq!(size_of::<RexxMethodEntry>(), 32);
    assert_eq!(offset_of!(RexxMethodEntry, style), 0);
    assert_eq!(offset_of!(RexxMethodEntry, reserved1), 4);
    assert_eq!(offset_of!(RexxMethodEntry, name), 8);
    assert_eq!(offset_of!(RexxMethodEntry, entry_point), 16);
    assert_eq!(offset_of!(RexxMethodEntry, reserved2), 24);
    assert_eq!(offset_of!(RexxMethodEntry, reserved3), 28);

    assert_eq!(size_of::<RexxRoutineEntry>(), 32);
    assert_eq!(offset_of!(RexxRoutineEntry, style), 0);
    assert_eq!(offset_of!(RexxRoutineEntry, reserved1), 4);
    assert_eq!(offset_of!(RexxRoutineEntry, name), 8);
    assert_eq!(offset_of!(RexxRoutineEntry, entry_point), 16);
    assert_eq!(offset_of!(RexxRoutineEntry, reserved2), 24);
    assert_eq!(offset_of!(RexxRoutineEntry, reserved3), 28);
}

#[test]
fn a_wrapper_puts_the_public_context_first() {
    assert_eq!(offset_of!(Owned<RexxInstance_, u8>, context), 0);
    assert_eq!(
        offset_of!(Owned<RexxInstance_, u8>, owner),
        size_of::<RexxInstance_>()
    );
    assert_eq!(offset_of!(Owned<RexxThreadContext_, u8>, context), 0);
    assert_eq!(
        offset_of!(Owned<RexxThreadContext_, u8>, owner),
        size_of::<RexxThreadContext_>()
    );
    assert_eq!(offset_of!(Owned<RexxMethodContext_, u8>, context), 0);
    assert_eq!(
        offset_of!(Owned<RexxMethodContext_, u8>, owner),
        size_of::<RexxMethodContext_>()
    );
    assert_eq!(offset_of!(Owned<RexxCallContext_, u8>, context), 0);
    assert_eq!(
        offset_of!(Owned<RexxCallContext_, u8>, owner),
        size_of::<RexxCallContext_>()
    );
    assert_eq!(offset_of!(Owned<RexxExitContext_, u8>, context), 0);
    assert_eq!(
        offset_of!(Owned<RexxExitContext_, u8>, owner),
        size_of::<RexxExitContext_>()
    );
}

/// The variable this test sets on the child it spawns, so that the child
/// reaches the stub and this process does not.
const CALL_A_STUB: &str = "REXX_API_CALL_AN_UNBUILT_ENTRY";

/// A stub is reached the way an extension would reach it, through the table.
///
/// The call is in a child because an `extern "C"` frame aborts on panic, so
/// there is no returning from it.
#[test]
fn an_unbuilt_entry_refuses_loudly() {
    if std::env::var_os(CALL_A_STUB).is_some() {
        (RexxThreadInterface::REFUSING.HaltThread)(std::ptr::null_mut());
        unreachable!("the stub returned");
    }

    let binary = std::env::current_exe().expect("this test binary's own path");
    let output = std::process::Command::new(binary)
        .args(["an_unbuilt_entry_refuses_loudly", "--exact", "--nocapture"])
        .env(CALL_A_STUB, "1")
        .output()
        .expect("the child runs");

    assert!(
        !output.status.success(),
        "the child returned from an entry nothing has built: {:?}",
        output.status
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("RexxThreadInterface.HaltThread is not implemented (Phase 8)"),
        "the refusal did not name the entry and the phase that owes it:\n{stderr}"
    );
}

/// The tables outside the L2 slice are declared and nothing builds one, so
/// the site that would hand one out has to say so rather than answer.
#[test]
fn a_table_outside_the_slice_refuses_where_it_would_be_handed_out() {
    for (name, hand_out) in [
        (
            "RexxInstanceInterface",
            (|| {
                rexx_api::layout::instance_interface();
            }) as fn(),
        ),
        ("CallContextInterface", || {
            rexx_api::layout::call_context_interface();
        }),
        ("ExitContextInterface", || {
            rexx_api::layout::exit_context_interface();
        }),
        ("IORedirectorInterface", || {
            rexx_api::layout::io_redirector_interface();
        }),
    ] {
        let raised = std::panic::catch_unwind(hand_out).expect_err(&format!("{name} answered"));
        let message = raised
            .downcast_ref::<String>()
            .map_or_else(String::new, Clone::clone);
        assert_eq!(message, format!("{name} is not implemented (Phase 8)"));
    }
}
