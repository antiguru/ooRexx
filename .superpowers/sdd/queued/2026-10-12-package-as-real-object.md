**CLOSED: Phase 9b1.** phase-9b1-gate.md: L5 to L7 fixed, TESTRECLAIM01 passed; side-maps.tsv keeps package state on the package object's body.

# Package becomes a real object; sweep for other built-ins that should be

Moritz 2026-10-12: "We'll need to turn Package into a real class/object, with a handle to the
built-in data. We can defer building this, and when we do we should combine it with a sweep over
what else should be real classes rather than built-ins. We can add more classes if it helps us, as
long as they're somewhat namespaced."

Today a Package object is an identity: `object_packages` maps its ObjRef to `Package::{Rexx,
Program(ProgramId)}`, and its state lives in per-ProgramId side maps on `Interp` (options, security
managers, routines and public/merged routines, classes and public/merged classes, namespaces,
imports, parents, required paths, routine objects, compiled method names; `rust/crates/rexx-exec/src/lib.rs`).

Symptoms found in Phase 9a Task 8b: `Package~copy` must find and clone every map (an exhaustive
destructuring guards it); the REXX package cannot be copied (no ProgramId); copies are never
collected (~5.8 kB each, queued in 2026-10-12-gc-trigger-blind-to-class-weight.md); each new
per-package feature adds a map the collector must trace.

Target shape: a heap object holding the mutable per-package state as fields plus a handle to the
immutable compiled program (`Rc<Program>`); copy is a field copy sharing the program; the collector
traces and frees it; activations hold the package ObjRef.

Scope when built: combine with a sweep over every other built-in kept as side-table state keyed by
an id rather than as an object (candidates to find, not a list to trust). New internal classes are
allowed if they help, as long as they are namespaced. Start with a scout mapping each side map, its
readers, and which sit on the CALL / send hot path (callgrind before and after).

Owner: deferred (not Phase 9a).

Symptom found in Phase 9a Task 9: `base/class/Package`'s `TESTRECLAIM01` wraps a package in a
`WeakReference` and loops until it is `.nil`; every package object here is a global root
(`Interp::package_object`), so the loop never ends, and the oracle's requires cache holds its
packages weakly (`PackageManager::addToRequiresCache`, `interpreter/package/PackageManager.cpp:921`).
Even `.package~new('src', source)`, dropped, stays: the crate's weak reference is not cleared in
10 passes of 100 `copies('x', 1000000)`, the oracle's after 1.

Phase 9a (Moritz 2026-10-12): ooTest base/class/Package TESTRECLAIM01 waits for a dropped package to be collected and stays renamed out on timeout until this lands; its witness is that test.
