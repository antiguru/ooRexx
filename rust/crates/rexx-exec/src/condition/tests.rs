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

/// **A SYNTAX condition object's entries survive the allocations that
/// build it.** Under a collection at every allocation, `MESSAGE`,
/// `ERRORTEXT` and `PROGRAM` read back as under none; the oracle prints
/// the same line. Each of those is long enough to live in the arena, where
/// `CODE` is not, which is why reading `CODE` alone could not see a
/// missing root.
#[test]
fn a_syntax_condition_objects_entries_survive_a_collection_at_every_allocation() {
    let text = b"signal on syntax\n\
        y = 1/0\n\
        exit\n\
        syntax:\n\
        c = condition('O')\n\
        say c~code '|' c~message '|' c~errortext '|' (c~program == .context~package~name)\n"
        .to_vec();
    let name = "/tmp/condition_entries.rex";
    let plain = crate::run_program(name, text.clone(), crate::Invocation::none());
    assert_eq!(
        plain.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&plain.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&plain.stdout),
        "42.3 | Arithmetic overflow; divisor must not be zero. | Arithmetic overflow/underflow. \
         | 1\n"
    );
    let swept = crate::run_program_collect_every_alloc(name, text, crate::Invocation::none());
    assert_eq!(swept.exit_code, plain.exit_code);
    assert_eq!(swept.stdout, plain.stdout);
    assert_eq!(swept.stderr, plain.stderr);
    assert!(swept.collections > 0, "the stress mode did not collect");
}

/// **A `CALL ON` handler's condition object is a root for as long as the
/// handler runs**, a routine it calls included, once the trap queue has
/// handed it to the handler's activation. The stdout is the oracle's; the
/// output before the nested call is what gives a collection somewhere to
/// fall between the handover and the read.
#[test]
fn a_call_on_handlers_condition_object_survives_a_collection_at_every_allocation() {
    let text = b"call on error name onerror\n\
        \"sh -c 'exit 3'\"\n\
        exit\n\
        onerror:\n\
        say 'a command condition'\n\
        call report\n\
        return\n\
        report:\n\
        o = condition('O')\n\
        say o~class~id\n\
        return\n"
        .to_vec();
    let name = "/tmp/handler_condition.rex";
    let plain = crate::run_program(name, text.clone(), crate::Invocation::none());
    assert_eq!(
        plain.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&plain.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&plain.stdout),
        "a command condition\nDirectory\n"
    );
    let swept = crate::run_program_collect_every_alloc(name, text, crate::Invocation::none());
    assert_eq!(swept.exit_code, plain.exit_code);
    assert_eq!(swept.stdout, plain.stdout);
    assert_eq!(swept.stderr, plain.stderr);
    assert!(swept.collections > 0, "the stress mode did not collect");
}

/// A fresh loop condition remains readable after its trace line allocates.
#[test]
fn a_traced_loop_condition_keeps_its_fresh_concatenation() {
    let text = b"signal on syntax\ntrace r\ndo while ('badvalue' || 'badvalue')\n  nop\nend\nexit\nsyntax:\ntrace off\nsay condition('O')~code condition('O')~additional[1]\n".to_vec();
    let plain = crate::run_program("/tmp/rooted-loop.rex", text.clone(), crate::Invocation::none());
    assert_eq!(plain.exit_code, 0);
    assert_eq!(plain.stdout, b"34.3 badvaluebadvalue\n");
    let stressed = crate::run_program_collect_every_alloc("/tmp/rooted-loop.rex", text, crate::Invocation::none());
    assert!(stressed.collections > 0);
    assert_eq!(stressed.stdout, plain.stdout);
    assert_eq!(stressed.stderr, plain.stderr);
    assert_eq!(stressed.exit_code, plain.exit_code);
}
