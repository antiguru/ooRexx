# A REPLY continuation that runs to its end is not announced with `>I>`/`<I<`

Found by Phase 6.1 Task 2 (whole-group row TRACE TEST_TRACE_LABEL_WITH_FORWARD; the Task 2 review's `p7.rex`, the test's `test_forwarded3.rex` resource with no loop). Ruled not a 6.1 item and queued at Task 12 (ledger, Task 2). Queued by Phase 6.1 Task 12 (2026-10-10). Concurrent: 5 runs per engine. Same stdout and rc; the oracle traces 12 `>I>`/`<I<` lines in every run, this crate 10.

Probe `p7.rex`, run from a fresh empty directory:

    t=.test~new
    t~one
    call syssleep 0.001
    res=t~three
    call syssleep 0.001
    say res
    ::class test
    ::method one
      forward message "onetwo"
      i=i/0
    ::method onetwo
      reply
    ::method three
      reply 42
      forward message "threefour" continue
      if result<>42 then i=i/0
    ::method threefour
      return 42
    ::options trace labels

Oracle, rc 0, 5 runs, 1 distinct outcome:

    [stdout]
    42
    [stderr]
           >I> Method "ONE" with scope "TEST" in package "p7/p7.rex".
           >I> Method "ONETWO" with scope "TEST" in package "p7/p7.rex".
           <I< Method "ONETWO" with scope "TEST" in package "p7/p7.rex".
           <I< Method "ONE" with scope "TEST" in package "p7/p7.rex".
           >I> Method "ONETWO" with scope "TEST" in package "p7/p7.rex".
           <I< Method "ONETWO" with scope "TEST" in package "p7/p7.rex".
           >I> Method "THREE" with scope "TEST" in package "p7/p7.rex".
           <I< Method "THREE" with scope "TEST" in package "p7/p7.rex".
           >I> Method "THREE" with scope "TEST" in package "p7/p7.rex".
           >I> Method "THREEFOUR" with scope "TEST" in package "p7/p7.rex".
           <I< Method "THREEFOUR" with scope "TEST" in package "p7/p7.rex".
           <I< Method "THREE" with scope "TEST" in package "p7/p7.rex".

This crate (`rexx-run` at `6a87cd616`), rc 0, 5 runs, 1 distinct outcome:

    [stdout]
    42
    [stderr]
           >I> Method "ONE" with scope "TEST" in package "p7/p7.rex".
           >I> Method "ONETWO" with scope "TEST" in package "p7/p7.rex".
           <I< Method "ONETWO" with scope "TEST" in package "p7/p7.rex".
           <I< Method "ONE" with scope "TEST" in package "p7/p7.rex".
           >I> Method "THREE" with scope "TEST" in package "p7/p7.rex".
           <I< Method "THREE" with scope "TEST" in package "p7/p7.rex".
           >I> Method "THREE" with scope "TEST" in package "p7/p7.rex".
           >I> Method "THREEFOUR" with scope "TEST" in package "p7/p7.rex".
           <I< Method "THREEFOUR" with scope "TEST" in package "p7/p7.rex".
           <I< Method "THREE" with scope "TEST" in package "p7/p7.rex".

The missing pair is the second `>I>`/`<I<` for `ONETWO`: after its bare `reply`, the oracle announces the continuation on the new activity and its end; here the continuation runs unannounced. `THREE`'s continuation is announced on both.

Suspected site: the trace entry and exit for a REPLY continuation with no clause left to run (`scheduler.rs` continuation start, `trace_invocation_entry`).
