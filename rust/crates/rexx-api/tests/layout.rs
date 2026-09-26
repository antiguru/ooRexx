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
use std::collections::BTreeSet;
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
    declarations_of(header, name)
        .into_iter()
        .map(|(member, _)| member)
        .collect()
}

/// [`members_of`], each with the return type a function member declares and
/// `None` for a data member.
fn declarations_of(header: &str, name: &str) -> Vec<(String, Option<String>)> {
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
                found.push((entry, Some(declaration[..at].trim().to_string())));
            }
            None => found.push((trailing_name(declaration), None)),
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

/// **A refusing member aborts exactly where returning is unsafe**: it answers
/// a data pointer the extension dereferences (`POINTER`, `CSTRING` or a
/// `RexxInstance *`), or it is one of the `Throw` members, which the oracle
/// leaves by a C++ throw (`interpreter/api/CallContextStubs.cpp:207-213`).
/// Every other member records itself and returns. Derived from the header's
/// return types, so a member the header adds is classified here.
#[test]
fn a_refusing_member_aborts_exactly_where_no_return_is_safe() {
    let header = header();
    for (name, fields, aborts) in [
        (
            "RexxInstanceInterface",
            RexxInstanceInterface::FIELDS,
            RexxInstanceInterface::ABORTS,
        ),
        (
            "RexxThreadInterface",
            RexxThreadInterface::FIELDS,
            RexxThreadInterface::ABORTS,
        ),
        (
            "MethodContextInterface",
            MethodContextInterface::FIELDS,
            MethodContextInterface::ABORTS,
        ),
        (
            "CallContextInterface",
            CallContextInterface::FIELDS,
            CallContextInterface::ABORTS,
        ),
    ] {
        let expected: Vec<bool> = declarations_of(&header, name)
            .into_iter()
            .map(|(member, returns)| match returns {
                None => false,
                Some(returns) => {
                    member.starts_with("Throw")
                        || returns == "POINTER"
                        || returns == "CSTRING"
                        || returns.contains('*')
                }
            })
            .collect();
        let ours: Vec<(&str, bool)> = fields.iter().copied().zip(aborts.iter().copied()).collect();
        let theirs: Vec<(&str, bool)> = fields.iter().copied().zip(expected).collect();
        assert_eq!(ours, theirs, "{name}");
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
        rexx_api::layout::METHOD_CONTEXT_INTERFACE.interfaceVersion,
        rexx_api::layout::METHOD_INTERFACE_VERSION
    );
    assert_eq!(
        rexx_api::ffi::CALL_CONTEXT.interfaceVersion,
        rexx_api::layout::CALL_INTERFACE_VERSION
    );
    assert_eq!(
        rexx_api::ffi::INSTANCE.interfaceVersion,
        rexx_api::layout::INSTANCE_INTERFACE_VERSION
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

/// The tables outside the L2 slice are declared and nothing builds one, so
/// the site that would hand one out has to say so rather than answer.
#[test]
fn a_table_outside_the_slice_refuses_where_it_would_be_handed_out() {
    for (name, hand_out) in [
        (
            "ExitContextInterface",
            (|| {
                rexx_api::layout::exit_context_interface();
            }) as fn(),
        ),
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

/// The Rust type the header's C type `c` is, as `std::any::type_name` spells
/// it once module paths are dropped.
fn rust_type(c: &str) -> String {
    let mut stars = c.matches('*').count();
    let words: Vec<&str> = c
        .split(|ch: char| ch == '*' || ch.is_whitespace())
        .filter(|word| !word.is_empty())
        .collect();
    let mut rust = match words.as_slice() {
        ["const", "char"] => {
            stars -= 1;
            "*const i8".to_string()
        }
        [name] => match *name {
            "void" => String::new(),
            "CSTRING" => "*const i8".to_string(),
            "POINTER" | "REXXPFN" => "*mut c_void".to_string(),
            "size_t" | "stringsize_t" | "logical_t" | "uintptr_t" => "usize".to_string(),
            "wholenumber_t" | "intptr_t" => "isize".to_string(),
            "int" | "int32_t" => "i32".to_string(),
            "uint32_t" => "u32".to_string(),
            "int64_t" => "i64".to_string(),
            "uint64_t" => "u64".to_string(),
            "double" => "f64".to_string(),
            "float" => "f32".to_string(),
            "RexxCondition" | "ValueDescriptor" | "RexxPackageEntry" => (*name).to_string(),
            "RexxInstance" => "RexxInstance_".to_string(),
            name if name.starts_with("Rexx") && name.ends_with("Context") => format!("{name}_"),
            name if name.starts_with("Rexx")
                && (name.ends_with("Object") || name.ends_with("ObjectPtr")) =>
            {
                format!("*mut {name}_")
            }
            other => panic!("{c}: {other} is not a type this test knows"),
        },
        _ => panic!("{c}: not a type this test knows"),
    };
    for _ in 0..stars {
        rust = format!("*mut {rust}");
    }
    rust
}

/// `c` without the parameter name the header gives some arguments, which is
/// a trailing word after a complete type.
fn without_parameter_name(c: &str) -> &str {
    let c = c.trim();
    match c.rsplit_once(' ') {
        Some((head, name))
            if !name.contains('*') && head != "const" && !head.trim_end().ends_with("const") =>
        {
            head.trim_end()
        }
        _ => c,
    }
}

/// Each member's type as the header declares it, in the spelling
/// [`rust_type`] gives.
fn header_types(header: &str, name: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = header.lines().collect();
    let close = format!("}} {name};");
    let end = lines
        .iter()
        .position(|line| line.trim() == close)
        .expect("the struct closes");
    let start = lines[..end]
        .iter()
        .rposition(|line| line.trim() == "typedef struct")
        .expect("the struct opens");
    let mut found = Vec::new();
    for line in &lines[start + 1..end] {
        let code = code_of(line).trim();
        if code.is_empty() || code == "{" {
            continue;
        }
        let declaration = code.strip_suffix(';').expect("a member declaration");
        if let Some(at) = declaration.find("(RexxEntry") {
            let returns = rust_type(&declaration[..at]);
            let rest = &declaration[at + "(RexxEntry".len()..];
            let close = rest.find(')').expect("the pointer declarator closes");
            let member = rest[..close]
                .trim()
                .trim_start_matches('*')
                .trim()
                .to_string();
            let arguments = rest[close + 1..]
                .trim()
                .strip_prefix('(')
                .and_then(|a| a.strip_suffix(')'))
                .expect("a parameter list");
            let arguments: Vec<String> = arguments
                .split(',')
                .map(|argument| rust_type(without_parameter_name(argument)))
                .collect();
            let tail = if returns.is_empty() {
                String::new()
            } else {
                format!(" -> {returns}")
            };
            found.push((
                member,
                format!("unsafe extern \"C\" fn({}){tail}", arguments.join(", ")),
            ));
        } else {
            let member = trailing_name(declaration);
            let ty = declaration[..declaration.len() - member.len()].trim();
            found.push((member, rust_type(ty)));
        }
    }
    found
}

/// `type_name`'s spelling with every module path dropped.
fn unqualified(type_name: &str) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let mut chars = type_name.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            word.push(ch);
        } else if ch == ':' && chars.peek() == Some(&':') {
            chars.next();
            word.clear();
        } else {
            out.push_str(&word);
            word.clear();
            out.push(ch);
        }
    }
    out.push_str(&word);
    out
}

/// **Every member's type is the header's**, arguments and result, not only
/// its name: a slot declared with the wrong width or one argument short
/// would link and be called with the header's arguments.
#[test]
fn every_member_has_the_type_the_header_declares() {
    let header = header();
    for (name, fields, types) in [
        (
            "RexxInstanceInterface",
            RexxInstanceInterface::FIELDS,
            RexxInstanceInterface::member_types(),
        ),
        (
            "RexxThreadInterface",
            RexxThreadInterface::FIELDS,
            RexxThreadInterface::member_types(),
        ),
        (
            "MethodContextInterface",
            MethodContextInterface::FIELDS,
            MethodContextInterface::member_types(),
        ),
        (
            "CallContextInterface",
            CallContextInterface::FIELDS,
            CallContextInterface::member_types(),
        ),
        (
            "ExitContextInterface",
            ExitContextInterface::FIELDS,
            ExitContextInterface::member_types(),
        ),
        (
            "IORedirectorInterface",
            IORedirectorInterface::FIELDS,
            IORedirectorInterface::member_types(),
        ),
    ] {
        let ours: Vec<(String, String)> = fields
            .iter()
            .zip(types)
            .map(|(field, ty)| ((*field).to_string(), unqualified(ty)))
            .collect();
        assert_eq!(ours, header_types(&header, name), "{name}");
    }
}

/// The negative control for the comparison above: the spellings it compares
/// are the ones a wrong member would change.
#[test]
fn the_type_comparison_sees_a_width_and_an_argument() {
    assert_eq!(
        rust_type("RexxThreadContext **"),
        "*mut *mut RexxThreadContext_"
    );
    assert_eq!(rust_type(without_parameter_name("size_t count")), "usize");
    assert_eq!(without_parameter_name("CSTRING *"), "CSTRING *");
    assert_ne!(rust_type("int32_t"), rust_type("int64_t"));
    assert_eq!(
        unqualified(std::any::type_name::<
            unsafe extern "C" fn(*mut RexxThreadContext_, i32) -> rexx_api::layout::RexxObjectPtr,
        >()),
        "unsafe extern \"C\" fn(*mut RexxThreadContext_, i32) -> *mut RexxObjectPtr_"
    );
    let thread = header_types(&header(), "RexxThreadInterface");
    let (_, append) = thread
        .iter()
        .find(|(member, _)| member == "ArrayAppendString")
        .expect("the header declares ArrayAppendString");
    assert_eq!(
        append,
        "unsafe extern \"C\" fn(*mut RexxThreadContext_, *mut RexxArrayObject_, *const i8, usize) -> usize"
    );
}

/// For each table something hands an extension, the members whose address is
/// still the refusing stub's, as `Table.Member`.
fn refusing_members() -> BTreeSet<String> {
    let header = header();
    let mut refusing = BTreeSet::new();
    for (name, populated, stubs) in [
        (
            "RexxInstanceInterface",
            rexx_api::ffi::INSTANCE.addresses(),
            RexxInstanceInterface::REFUSING.addresses(),
        ),
        (
            "RexxThreadInterface",
            rexx_api::ffi::THREAD.addresses(),
            RexxThreadInterface::REFUSING.addresses(),
        ),
        (
            "MethodContextInterface",
            rexx_api::ffi::METHOD_CONTEXT.addresses(),
            MethodContextInterface::REFUSING.addresses(),
        ),
        (
            "CallContextInterface",
            rexx_api::ffi::CALL_CONTEXT.addresses(),
            CallContextInterface::REFUSING.addresses(),
        ),
    ] {
        let functions: Vec<String> = declarations_of(&header, name)
            .into_iter()
            .filter(|(_, returns)| returns.is_some())
            .map(|(member, _)| member)
            .collect();
        let named: Vec<String> = populated.iter().map(|(m, _)| (*m).to_string()).collect();
        assert_eq!(named, functions, "{name}'s function members");
        for ((member, ours), (_, stub)) in populated.iter().zip(&stubs) {
            if ours == stub {
                refusing.insert(format!("{name}.{member}"));
            }
        }
    }
    refusing
}

/// **No stub remains that a populated table should have replaced**: the
/// members still refusing are exactly the ones `REFUSING_MEMBERS` names with
/// their owner, derived from the header's members rather than from a list.
#[test]
fn a_populated_table_refuses_exactly_the_members_it_names() {
    let listed: BTreeSet<String> = rexx_api::layout::REFUSING_MEMBERS
        .iter()
        .map(|(member, _)| (*member).to_string())
        .collect();
    assert_eq!(
        listed.len(),
        rexx_api::layout::REFUSING_MEMBERS.len(),
        "a member is listed twice"
    );
    assert_eq!(refusing_members(), listed);
}

/// The negative control for the comparison above: a refusing table compared
/// with itself refuses everything, and a filled member differs from its stub.
#[test]
fn the_refusal_comparison_tells_a_filled_member_from_a_stub() {
    let stubs = RexxThreadInterface::REFUSING.addresses();
    assert!(
        stubs
            .iter()
            .zip(RexxThreadInterface::REFUSING.addresses())
            .all(|(ours, theirs)| ours.1 == theirs.1)
    );
    let filled = rexx_api::ffi::THREAD.addresses();
    let (_, stub) = stubs
        .iter()
        .find(|(member, _)| *member == "WholeNumberToObject")
        .expect("the thread table has WholeNumberToObject");
    let (_, ours) = filled
        .iter()
        .find(|(member, _)| *member == "WholeNumberToObject")
        .expect("the thread table has WholeNumberToObject");
    assert_ne!(stub, ours);
}

/// The C++ wrapper struct an interface table's members are called through,
/// and that table.
const WRAPPERS: [(&str, &str); 4] = [
    ("RexxInstance_", "RexxInstanceInterface"),
    ("RexxThreadContext_", "RexxThreadInterface"),
    ("RexxMethodContext_", "MethodContextInterface"),
    ("RexxCallContext_", "CallContextInterface"),
];

/// Each inline method `wrapper` defines in the header, with the method's
/// body.
fn inline_methods(header: &str, wrapper: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = header.lines().collect();
    let open = format!("struct {wrapper}");
    let start = lines
        .iter()
        .position(|line| line.trim() == open)
        .unwrap_or_else(|| panic!("{wrapper} is not defined in the header"));
    let end = start
        + lines[start..]
            .iter()
            .position(|line| line.trim() == "};")
            .expect("the struct closes");
    let mut found = Vec::new();
    let mut at = start + 1;
    while at < end {
        let line = code_of(lines[at]).trim();
        if line.ends_with(')') && lines.get(at + 1).is_some_and(|next| next.trim() == "{") {
            let head = &line[..line.find('(').expect("a parameter list")];
            let name = trailing_name(head.trim_end());
            let close = at
                + 2
                + lines[at + 2..end]
                    .iter()
                    .position(|next| next.trim() == "}")
                    .expect("the method closes");
            found.push((name, lines[at + 2..close].join("\n")));
            at = close + 1;
        } else {
            at += 1;
        }
    }
    found
}

/// Every table member the inline method `name` reaches, through any wrapper
/// that defines it, following one wrapper method calling another.
fn members_reached(header: &str, name: &str) -> BTreeSet<String> {
    let mut reached = BTreeSet::new();
    for (wrapper, _) in WRAPPERS {
        reached.extend(members_reached_in(header, wrapper, name));
    }
    reached
}

fn members_reached_in(header: &str, wrapper: &str, name: &str) -> BTreeSet<String> {
    let table = |struct_name: &str| {
        WRAPPERS
            .iter()
            .find(|(w, _)| *w == struct_name)
            .map(|(_, t)| *t)
            .expect("a known wrapper")
    };
    let mut reached = BTreeSet::new();
    for (method, body) in inline_methods(header, wrapper) {
        if method != name {
            continue;
        }
        for (prefix, owner) in [
            ("threadContext->functions->", "RexxThreadContext_"),
            ("instance->functions->", "RexxInstance_"),
            ("functions->", wrapper),
        ] {
            for (at, _) in body.match_indices(prefix) {
                if prefix == "functions->"
                    && (body[..at].ends_with("threadContext->")
                        || body[..at].ends_with("instance->"))
                {
                    continue;
                }
                let member = identifier_at(&body[at + prefix.len()..]);
                reached.insert(format!("{}.{member}", table(owner)));
            }
        }
        for (receiver, owner) in [
            ("threadContext->", "RexxThreadContext_"),
            ("instance->", "RexxInstance_"),
        ] {
            for (at, _) in body.match_indices(receiver) {
                let rest = &body[at + receiver.len()..];
                let called = identifier_at(rest);
                if rest[called.len()..].trim_start().starts_with('(') {
                    reached.extend(members_reached_in(header, owner, &called));
                }
            }
        }
    }
    reached
}

/// The identifier `text` starts with.
fn identifier_at(text: &str) -> String {
    text.chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

/// The table members the oracle's `METHOD`, `CONVERSION` and `FUNCTION`
/// extensions call, through the header's inline wrappers.
fn members_the_test_extensions_call() -> BTreeSet<String> {
    let header = header();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../testbinaries");
    let mut called = BTreeSet::new();
    for file in ["orxmethod.cpp", "orxfunction.cpp"] {
        let path = root.join(file);
        let source =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for (at, _) in source.match_indices("->") {
            let name = identifier_at(&source[at + 2..]);
            if source[at + 2 + name.len()..].trim_start().starts_with('(') {
                called.insert(name);
            }
        }
    }
    called
        .iter()
        .flat_map(|name| members_reached(&header, name))
        .collect()
}

/// The derivation above reads wrappers that call other wrappers: `True()` in
/// a method context reaches the thread table's data member, and
/// `InterpreterVersion()` reaches the instance table through two wrappers.
#[test]
fn the_wrapper_scan_follows_one_wrapper_into_another() {
    let header = header();
    assert_eq!(
        members_reached_in(&header, "RexxMethodContext_", "InterpreterVersion"),
        BTreeSet::from(["RexxInstanceInterface.InterpreterVersion".to_string()])
    );
    assert_eq!(
        members_reached_in(&header, "RexxMethodContext_", "GetArguments"),
        BTreeSet::from(["MethodContextInterface.GetArguments".to_string()])
    );
    assert_eq!(
        members_reached_in(&header, "RexxCallContext_", "String"),
        BTreeSet::from([
            "RexxThreadInterface.NewString".to_string(),
            "RexxThreadInterface.NewStringFromAsciiz".to_string()
        ])
    );
    let reached = members_the_test_extensions_call();
    assert!(reached.contains("RexxInstanceInterface.AttachThread"));
    assert!(reached.contains("RexxThreadInterface.RexxTrue"));
    assert!(!reached.contains("RexxThreadInterface.HaltThread"));
}

/// **What the test extensions call is filled first**: every member they reach
/// that still refuses is one this list names, each with the reason it is not
/// filled here.
#[test]
fn the_test_extensions_reach_only_members_that_answer() {
    const STILL_REFUSING: &[&str] = &[
        "CallContextInterface.DropContextVariable",
        "CallContextInterface.FindContextClass",
        "CallContextInterface.GetAllContextVariables",
        "CallContextInterface.GetArgument",
        "CallContextInterface.GetArguments",
        "CallContextInterface.GetContextVariable",
        "CallContextInterface.GetRoutine",
        "CallContextInterface.GetRoutineName",
        "CallContextInterface.ResolveStemVariable",
        "CallContextInterface.SetContextVariable",
        "CallContextInterface.ThrowCondition",
        "CallContextInterface.ThrowException",
        "CallContextInterface.ThrowException0",
        "CallContextInterface.ThrowException1",
        "CallContextInterface.ThrowException2",
        "MethodContextInterface.AllocateObjectMemory",
        "MethodContextInterface.FindContextClass",
        "MethodContextInterface.ForwardMessage",
        "MethodContextInterface.FreeObjectMemory",
        "MethodContextInterface.GetArgument",
        "MethodContextInterface.GetArguments",
        "MethodContextInterface.GetMessageName",
        "MethodContextInterface.GetMethod",
        "MethodContextInterface.GetObjectVariable",
        "MethodContextInterface.GetObjectVariableReference",
        "MethodContextInterface.GetScope",
        "MethodContextInterface.GetSelf",
        "MethodContextInterface.GetSuper",
        "MethodContextInterface.ReallocateObjectMemory",
        "MethodContextInterface.SetGuardOff",
        "MethodContextInterface.SetGuardOffWhenUpdated",
        "MethodContextInterface.SetGuardOn",
        "MethodContextInterface.SetGuardOnWhenUpdated",
        "MethodContextInterface.ThrowCondition",
        "MethodContextInterface.ThrowException",
        "MethodContextInterface.ThrowException0",
        "MethodContextInterface.ThrowException1",
        "MethodContextInterface.ThrowException2",
        "RexxInstanceInterface.AddCommandEnvironment",
        "RexxInstanceInterface.AttachThread",
        "RexxThreadInterface.CallProgram",
        "RexxThreadInterface.CallRoutine",
        "RexxThreadInterface.DetachThread",
        "RexxThreadInterface.FindClass",
        "RexxThreadInterface.FindPackageClass",
        "RexxThreadInterface.GetInterpreterInstance",
        "RexxThreadInterface.GetMethodPackage",
        "RexxThreadInterface.GetPackageClasses",
        "RexxThreadInterface.GetPackageMethods",
        "RexxThreadInterface.GetPackagePublicClasses",
        "RexxThreadInterface.GetPackagePublicRoutines",
        "RexxThreadInterface.GetPackageRoutines",
        "RexxThreadInterface.GetRoutinePackage",
        "RexxThreadInterface.HasMethod",
        "RexxThreadInterface.IsInstanceOf",
        "RexxThreadInterface.IsMethod",
        "RexxThreadInterface.IsRoutine",
        "RexxThreadInterface.IsVariableReference",
        "RexxThreadInterface.LoadPackage",
        "RexxThreadInterface.LoadPackageFromData",
        "RexxThreadInterface.NewMethod",
        "RexxThreadInterface.NewRoutine",
        "RexxThreadInterface.SendMessage",
        "RexxThreadInterface.SendMessage0",
        "RexxThreadInterface.SendMessage1",
        "RexxThreadInterface.SendMessage2",
        "RexxThreadInterface.SendMessageScoped",
        "RexxThreadInterface.SetVariableReferenceValue",
        "RexxThreadInterface.VariableReferenceName",
        "RexxThreadInterface.VariableReferenceValue",
    ];
    let refusing = refusing_members();
    let reached: BTreeSet<String> = members_the_test_extensions_call()
        .into_iter()
        .filter(|member| refusing.contains(member))
        .collect();
    let expected: BTreeSet<String> = STILL_REFUSING.iter().map(|m| (*m).to_string()).collect();
    assert_eq!(reached, expected);
}
