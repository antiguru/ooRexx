# `Class~enhanced` builds its instance without sending NEW

Found by Phase 6.1 scout D (2026-10-07, `clsenh_*`). Queued by Phase 6.1 Task 12 (2026-10-10). Loud here (93.902), the oracle answers. Once subclass `NEW` exists (Phase 9 row), sending NEW is the change.

Probe `b7_enhanced_array.rex`, run from a fresh empty directory:

    d = .stringtable~new; d["GO"] = "return self~items"
    x = .array~enhanced(d, 3)
    say x~go x~class~id

Oracle, rc 0:

    [stdout]
    0 Array
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 163:

    [stdout]
    (empty)
    [stderr]
           *-* Compiled method "INIT" with scope "Object".
           *-* Compiled method "ENHANCED" with scope "Class".
         2 *-* x = .array~enhanced(d, 3)
    Error 93 running b7_enhanced_array.rex line 2:  Incorrect call to method.
    Error 93.902:  Too many arguments in invocation of method; 0 expected.

Probe `b7_enhanced_string.rex`, run from a fresh empty directory:

    d = .stringtable~new; d["GO"] = "expose x; x = 3; self~setMethod('m', 'return 42', 'OBJECT'); return x self~m"
    x = .string~enhanced(d, 'abc')
    say x~go x~class~id

Oracle, rc 0:

    [stdout]
    3 42 String
    [stderr]
    (empty)

This crate (`rexx-run` at `6a87cd616`), rc 163:

    [stdout]
    (empty)
    [stderr]
           *-* Compiled method "INIT" with scope "Object".
           *-* Compiled method "ENHANCED" with scope "Class".
         2 *-* x = .string~enhanced(d, 'abc')
    Error 93 running b7_enhanced_string.rex line 2:  Incorrect call to method.
    Error 93.902:  Too many arguments in invocation of method; 0 expected.

Suspected site: `dispatch/class_protocol.rs` `native_enhanced`, which calls `new_instance` and then INIT with every argument; the oracle (`ClassClass.cpp:1470`) sends NEW to the receiver.
