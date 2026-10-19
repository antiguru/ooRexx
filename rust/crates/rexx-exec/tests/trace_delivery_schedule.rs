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

//! A trace line's delivery to a short Rexx destination lets no other
//! activity's lines in, under any schedule of the simulation mode. The oracle
//! gives up the kernel only in an activation past `yieldInstructions` or at a
//! native call (`RexxActivation.cpp:616`, `NativeActivation.cpp:1304`), and
//! this destination's `lineOut` has neither.

use std::process::Command;

/// ooTest `base/keyword/TRACE` `test_forwarded3`, its `ArrayStream`
/// destination included: a REPLY continuation's `>I>` and `<I<` race the
/// replier's own exit lines.
const PROGRAM: &str = "as = .ArrayStream~new\n\
     .TraceOutput~destination(as)\n\
     src = .array~of('t=.test~new', 't~one', 'call syssleep 0.001', 'res=t~three', \
     'call syssleep 0.001', 'return res', '::class test', '::method one', \
     '  forward message \"onetwo\"', '  i=i/0', '::method onetwo', '  reply', \
     '::method three', '  reply 42', '  forward message \"threefour\" continue', \
     '  if result<>42 then i=i/0', '::method threefour', '  return 42', \
     '::options trace labels')\n\
     r = .routine~new('fw3', src)\n\
     res = r~call\n\
     .TraceOutput~destination\n\
     say 'res' res\n\
     do v over as\n\
     \x20 if word(v, 2) <> 'Routine' then say left(v, pos(' in package', v' in package') - 1)\n\
     end\n\
     ::class ArrayStream subclass Array inherit Stream\n\
     ::method init\n  expose line\n  line = 0\n\
     ::method lineOut\n  parse arg line\n  self~append(line)\n  return 0\n";

/// What the oracle printed, 8 runs of 8.
const ORACLE_STDOUT: &str = "res 42\n\
     \x20      >I> Method \"ONE\" with scope \"TEST\"\n\
     \x20      >I> Method \"ONETWO\" with scope \"TEST\"\n\
     \x20      <I< Method \"ONETWO\" with scope \"TEST\"\n\
     \x20      <I< Method \"ONE\" with scope \"TEST\"\n\
     \x20      >I> Method \"ONETWO\" with scope \"TEST\"\n\
     \x20      <I< Method \"ONETWO\" with scope \"TEST\"\n\
     \x20      >I> Method \"THREE\" with scope \"TEST\"\n\
     \x20      <I< Method \"THREE\" with scope \"TEST\"\n\
     \x20      >I> Method \"THREE\" with scope \"TEST\"\n\
     \x20      >I> Method \"THREEFOUR\" with scope \"TEST\"\n\
     \x20      <I< Method \"THREEFOUR\" with scope \"TEST\"\n\
     \x20      <I< Method \"THREE\" with scope \"TEST\"\n";

#[test]
fn a_short_destination_keeps_each_delivery_whole_under_every_schedule() {
    let dir = std::env::temp_dir().join(format!("rexx-trace-delivery-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a directory");
    std::fs::write(dir.join("p.rex"), PROGRAM).expect("the program written");
    let mut differing = Vec::new();
    for seed in 1..=20 {
        for policy in ["uniform:1", "uniform:0.2", "pct:3,k=4"] {
            let mode = format!("sim:{seed},{policy}");
            let output = Command::new(env!("CARGO_BIN_EXE_rexx"))
                .arg(dir.join("p.rex"))
                .env("REXX_SWITCH_MODE", &mode)
                .stdin(std::process::Stdio::null())
                .output()
                .expect("rexx runs");
            if !output.status.success() || output.stdout != ORACLE_STDOUT.as_bytes() {
                differing.push(format!(
                    "{mode}:\n{}",
                    String::from_utf8_lossy(&output.stdout)
                ));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(differing.is_empty(), "{}", differing.join("\n"));
}
