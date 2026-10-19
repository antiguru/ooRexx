# Task 6.12 report: security manager

Clusters `security-manager-method-event` and `security-manager-environment-result`. The four tests of
these clusters pass, and the whole suite changes in exactly their rows and the known SysFileTree flake.
The queued item `2026-10-19-security-manager-protected-primitive.md` is closed by the method-event fix.

## Commits

* `a404ad404` Phase 9d Task 6.12: security manager. The code.
* `f4d8daa7f` the three witnesses with their `sourceline_oracle/` files and `phase-9.txt` lines, the
  four tests out of `suite-9d-set.txt`, and the queued item's CLOSED line.
* The records commit that follows holds `docs/superpowers/records/2026-10-18-phase-9d/t6.12/` and
  this report.

## Root causes and fixes

### security-manager-method-event

Two causes.

1. **No primitive was protected.** `rexx-classes/build.rs` read `AddProtectedMethod` and
   `AddClassProtectedMethod` as plain method rows, so `special_methods` held no protected row for any
   primitive and the dispatch seam never asked the manager. The oracle's `defineProtectedMethod`
   (`Setup.cpp:122`) sets the flag, and `RexxObject::messageSend` sends it through
   `processProtectedMethod`. Fix: build.rs emits `Op::AddProtectedInstanceMethod` and
   `Op::AddProtectedClassMethod`; `replay` records their identities in
   `ClassRegistry::protected_native_methods` (the image-save skip of `DefineClassMethod` and
   `InheritInstanceMethods` still applies); `install_object_model` files each as
   `AccessScope { access: Default, protected: true, package: Rexx }`. `Method~isProtected` reads the
   row (`Interp::installed_is_protected`), so `.Directory~method('SETMETHOD')~isProtected` is 1 as on
   the oracle, where it was 0.
2. **`setProtected` changed only the readback** (the queued item). It now also marks the method the
   object stands for (`Interp::make_method_protected`, in `security.rs`, the shape of
   `make_method_private`). A `~copy` carries the row through `copy_method_identity` and `define`
   installs that identity, so `.Array~method('APPEND')~copy~~setProtected` defined into a subclass
   asks the manager, and `.Array`'s own `APPEND` stays unprotected.

The seam now tests `security_armed` before the row lookup, so a process that never installed a
manager pays one flag test per send. Without a manager the protected primitives take `Cleared`, not
`Checked`, which is the same answer `check_protected_method` gave them.

### security-manager-environment-result

`security_entry` read the info directory with `AT` and dropped `.nil`, so a manager setting
`RESULT` to `.nil` read as "left unset". `SecurityManager::checkEnvironmentAccess` answers
`securityArgs->get(RESULT)`, which is the `.nil` stored. Fix: `security_entry` reads with
`hash::store_get` (`DirectoryClass::get`: contents, method table, unknown), and an entry set to `.nil`
is `Some(.nil)`. Measured on the oracle (`t6.12/probes/p2`, `p3`): ENVIRONMENT, LOCAL, METHOD and
CALL `RESULT` `.nil` answer `.nil`, COMMAND `RC` `.nil` makes `rc` `.nil`, and COMMAND `FAILURE` or
`ERROR` set to `.nil` raises the condition. Each now matches. Three readers keep `.nil` as unset:
* `STREAM`: the oracle uses `.nil` as the stream, raises 97.1, and then segfaults at exit (rc 139,
  `t6.12/probes/p3.rex`, its `stream-nil` line). Not copied; see Concerns.
* REQUIRES `SECURITYMANAGER`: the oracle's own `secObject != TheNilObject` (`SecurityManager.cpp:339`).
* REQUIRES `NAME`: the crate's answer is unchanged and already differs from the oracle; see Concerns.

## Witnesses, red then green

In `rust/corpus/phase-9.txt`, each with its `sourceline_oracle/` file from that test's driver. Parent
is `b41fc9257`, built in the same target dir. `t6.12/witnesses/summary.tsv`, run by
`t6.12/witnesses/run.sh`:

| program | parent vs oracle | task vs oracle |
|---|---|---|
| `security_manager_protected_primitive` | differs: stdout stderr rc | identical |
| `security_manager_set_protected` | differs: stdout | identical |
| `security_manager_nil_entry` | differs: stdout stderr rc | identical |

Read stdout, each path printing:
* `security_manager_protected_primitive`: from a Routine, METHOD events with `ARGUMENTS`, `NAME` and
  `OBJECT` for `setMethod`, `unsetMethod`, `superclasses`, `setSecurityManager` (after its
  `LOCAL`/`ENVIRONMENT` pair), `define`, `superclass` and `metaClass`; handled with a result
  (`intercepted`), handled with none (`superclasses` leaves `RESULT` unset, `define handled` leaves
  `FOO` undefined), and passed (`define result: 1`). An unprotected primitive sends no event. A
  Package prolog and a Method each send the `SETMETHOD` event. `isProtected` is 1 for each protected
  primitive asked, `DEFAULTOPTIONS` a class method among them, and 0 for four unprotected ones. The
  parent stops at the Method case with 91.999, rc 165.
* `security_manager_set_protected`: a copy is protected only after `setProtected`, its original
  stays 0, and the defined copy sends METHOD with `OBJECT=MYARR`, passed and handled; `.Array`'s
  `append` sends none. `.K~method('FOO')~setProtected` makes `foo` send METHOD, passed, handled with
  a result and with none; `bar` sends none; a `::method baz protected` already did on the parent.
* `security_manager_nil_entry`: each `.nil` line above answers `The NIL object` (FAILURE and ERROR
  `failure 0` and `error 0`, the failure's command trace on stderr), and each unset line answers the
  ordinary value. The parent answers `a RexxInfo` and `STDOUT` for the first two `.nil` lines and dies
  at `method nil` with 91.999; its CALL line was 44.1 (`t6.12/probes/p3.rex` on the parent).

## The tests

`suite-9d-set.txt` loses `base/security.manager/SecurityManager` `TEST_ENVIRONMENT_HANDLE`,
`TEST_METHOD_HANDLE_NO_RESULT`, `TEST_METHOD_HANDLE_RESULT` and `TEST_METHOD_PASS`, each now passed.
No exclusion row is touched. Queued: `2026-10-19-security-manager-protected-primitive.md` CLOSED by
`security-manager-method-event` (the `setProtected` half; the primitive half covers the built-ins
`Setup.cpp` marks).

## Changed rows

Against `t6.11/r2/suite/tests.tsv`, by the command at the top of `t6.12/suite/changed-rows.txt`: the
four tests above, `failed` to `passed` with no row, and SysFileTree `TEST_TREE_DIRECTORY`, the known
flake, here on the crate side. Suite: 402 groups, 5 red.

## Checks

From `rust/`, `CARGO_TARGET_DIR=<scratchpad>/t612-target`, `CARGO_INCREMENTAL=0`, each exit status
read unpiped. Code at `a404ad404`; the witness run and corpus at `f4d8daa7f`.

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-classes --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1155 passed.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --test refusal_sites --test owners --test
  closed_phases --test phase_ten_reasons --test representation_refusals --test
  refusal_dispositions --test coverage --test rexxc --test licensed_divergences`: exit 0. No `Loud`
  site added or removed.
* `memcap 16G cargo test -j 4 --release -p rexx-parse -p rexx-classes`: exit 0 (includes
  `sourceline_oracle`).
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1126 of 1126 matching.
* `REXX_SUITE_GATE=1 REXX_SUITE_TABLE=.../t6.12/suite/table.tsv memcap 8G cargo test -j 4 --release
  -p rexx-exec --test suite l3_core_suite`: exit 101 as expected, 402 groups, 5 red, 279 s.

## Perf

`t6.12/callgrind.txt`: pin, parent (`b41fc9257`) and head (`a404ad404`) in one run, exit 0. Own delta
is head d% minus parent d% in the same run, raw and with libc and ld-linux subtracted:

| program | head -libc vs pin | own -libc | own raw |
|---|---|---|---|
| startup | -0.1190 | +0.0041 | +0.0077 |
| interpret | -4.6297 | +0.0001 | -0.0077 |
| parse | -0.1416 | +0.0002 | +0.0001 |
| churn | +0.1069 | -0.0100 | -0.0057 |
| sayloop | -0.0630 | +0.0021 | -0.0014 |
| rexxcps | +0.5193 | +0.0001 | -0.0006 |
| sendloop | -0.2176 | 0.0000 | -0.2742 |
| dispatch | -0.1454 | 0.0000 | -0.1809 |

Every own delta is under +0.05% on both measures. sendloop and dispatch move only in glibc, the shape
Task 6.11 recorded. The parent's raw sendloop and dispatch (+0.0693, -0.1352) are Task 6.11's first
head figures, and the head's (-0.2049, -0.3161) are within 0.0001 of 6.10 r3's (-0.2050, -0.3162),
so this is the known heap-layout swing. The
whole-process figures against the pin stay under +0.5%. No error construction was added.

## Concerns

1. **A `.nil` STREAM entry crashes the oracle.** A manager that handles `STREAM` and sets the entry to
   `.nil` gets 97.1 at the stream builtin and then rc 139 at exit (`t6.12/probes/p3.rex`). This crate
   reads `.nil` as no replacement and opens the ordinary stream. Not in
   `rust/corpus/oracle-crashes.txt`, which is outside this task's files; proposed for it.
2. **A `.nil` REQUIRES `NAME` differs, unchanged by this task.** `t6.12/probes/p4.rex` mode 1: the
   oracle sends both REQUIRES events and raises 3.1; this crate sends one and raises 43.901.
   Proposed as a queued item.
3. The fix touches `rexx-classes` (`build.rs`, `native_classes.rs`, `registry.rs`), `dispatch.rs`,
   `dispatch/executable.rs`, `install.rs` and `run.rs` beside `security.rs`: the cause of both
   clusters sits in those readers.
4. The parent binary is this task's own build of `b41fc9257`, not Task 6.11's `t6.11-r2/rexx-head`.
   The code is the same (`git diff 7efdf2dc5 b41fc9257 -- rust/crates` is empty), but the sha256
   differs.
