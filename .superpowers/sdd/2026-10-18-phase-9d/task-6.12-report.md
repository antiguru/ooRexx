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

## Fix round 1

Review: `task-6.12-review.md`; ruling at the end of `progress.md`. Code commits `174c357be`,
`3f3dea5f8` and `d995b8f17`; the records commit that follows holds `t6.12/r1/` and this section.
Every change follows rexxref's rule for its checkpoint, and no Deviation arises.

### Critical 1: a started message

`effective_security_manager` read `self.activation()`, which panics on an activity running no
activation, which is what a started message's activity is. It now answers `None` there, as the
oracle sends no event (`start -> Object events:` empty). The lookup past `security_armed` is an
out-of-line `armed_security_manager`, which keeps it out of the send path (Perf below).

### Critical 2: a protected `PUT`

`security_arguments` filled the info directory by sending `PUT`, so a protected `PUT` asked the
manager again for each checkpoint. It now calls `hash::directory_put`, `DirectoryClass::put`
itself, as `SecurityManager.cpp`'s `securityArgs->put` does.

### Important 3: a shared primitive's flags

`setProtected` and `setPrivate` marked the `MethodId` that `InheritInstanceMethods` shares between
the hash collections, and `Array` shares with `Queue`. `Interp::unshared_identity`
(`environment/identities.rs`) finds the dictionary entry the method object stands for, at its
scope. Where another class's own dictionary holds the same identity
(`ClassGraph::held_elsewhere`), it mints a copy, installs it in the asked class's entry, and repoints
that class's method objects. The class asked about and its subclasses change, and nothing else does.
Both setters use it.

### Important 4: a Directory entry method and `Object~run`

`MethodClass::run` is not a send, so the seam must not ask about it. `invoke_executable`, which a
Directory entry read and `Object~run` share, takes a cold `invoke_armed_executable` once a manager
is installed. That function clears the method's protection row around `begin_invoke` and puts it
back after. A Rexx body has only been entered by then, and a native body runs inside it.
`method_is_protected` keeps its one caller, the seam (`tests/dispatch_seam.rs`).

### Important 5 and Minor 6: STREAM and REQUIRES replacements

The `.nil` filters in `check_stream_access` and on the REQUIRES `NAME` entry are gone. A STREAM
replacement that is not a Stream, `.nil` included, is the target of the builtin's message, so the
builtin raises 97.1 and never writes the file. A REQUIRES file that cannot be read is 3.1 naming the
file that was to be loaded, the manager's replacement included (`LanguageParser::createPackage`).
This replaces 43.901 naming the original. An unreadable original file is 3.1 on the oracle as well
(`t6.12/r1/probes/requires_unreadable.*`, a mode-000 file).

### Minor 7: a manager-raised command condition

`command_from_manager` now always supplies the RC object: the manager's entry, or 0 where it set
none. `raise_command_condition` takes that object. The condition carries it as `RC`
(`pending_rc`) with no `RESULT`, which is how `checkCommand`'s `createConditionObject` builds it.
The `RC` variable keeps the object.

### oracle-crashes.txt

Entry 54 is a STREAM replacement that does not support the Stream methods: 97.1, then SIGSEGV at
exit. Entry 55 is a REQUIRES `NAME` replacement that is not a string: SIGSEGV, 8 of 8. The oracle
cannot answer either case, so `tests/security_replacements.rs` holds both, run on this crate alone.
A non-string `NAME` is read as its string value, which gives 3.1 naming `an Object`.

### Witnesses

New in `phase-9.txt`, each with its `sourceline_oracle/` file: `security_manager_started_message`,
`security_manager_protected_put`, `security_manager_donated_flags`,
`security_manager_directory_entry`, `security_manager_requires_replacement` and
`security_manager_command_rc_condition`. `t6.12/r1/witnesses/summary.tsv`, red is `7fba9068c`'s
binary and green is `d995b8f17`'s:

| program | red vs oracle | green vs oracle |
|---|---|---|
| `security_manager_started_message` | differs: stdout stderr rc (panic, rc 101) | identical |
| `security_manager_protected_put` | differs: stdout stderr rc (stack overflow, rc 134) | identical |
| `security_manager_donated_flags` | differs: stdout stderr rc | identical |
| `security_manager_directory_entry` | differs: stdout | identical |
| `security_manager_requires_replacement` | differs: stdout | identical |
| `security_manager_command_rc_condition` | differs: stdout | identical |
| the three round-0 witnesses | identical | identical |

What each witness's stdout shows:
* `started_message` shows `superclass`, `setMethod` and a `Message~start` of `superclass` through
  `~start` after a manager was installed elsewhere. It shows a protected Rexx method run by
  `~start`, and a start inside a managed routine, which sends no event. The crate's stdout is
  identical under `sim:1`, `sim:2`, `sim:7,fifo`, `sim:3,uniform:0.2`, `sim:5,pct:3,k=40`,
  `sim:11,uniform:0.2` and `sim:13,pct:3,k=40`. The oracle gave the same stdout on 8 of 8 runs
  (`t6.12/r1/witnesses/sim.txt`).
* `protected_put` makes Directory's `PUT` protected. A routine then sends `LOCAL`, a `PUT` and
  `SUPERCLASS`, and the manager sees each event once, with its entries.
* `donated_flags`: after `.Bag~method('PUT')~setProtected`, Bag alone is 1 of the seven classes,
  and Queue's `APPEND` stays 0 after Array's is set. A managed routine's sends ask about Bag, MyBag
  and Array, not Table or Queue. After `setPrivate` on Bag's `PUT`, a Table `put` succeeds and a
  Bag `put` is 97.2.
* `directory_entry`: a protected method read as a Directory entry runs with no `METHOD` event,
  through the message, `at`, `[]`, `entry`, `UNKNOWN` and `[]` of an unknown name. Run with
  `Object~run`, it also sends none. Installed by `setMethod` on an object, it is intercepted. The
  last two lines also pass on red.
* `requires_replacement`: `.nil` and a missing name each give 3.1 naming the replacement, after
  two events. Removing the entry gives 43.901 naming the original after one event. An unchanged
  entry loads the file.
* `command_rc_condition`: RC `.nil`, `abc` and 7 with FAILURE and with ERROR, under `SIGNAL ON` and
  `CALL ON`. In each, `rc == .nil`, `rc`, the condition's `rc` and the absence of `RESULT` match the
  oracle, as does `rc` after the call handler returns.

`tests/security_replacements.rs` covers the two crash shapes. On red, the STREAM probe writes the
file and answers `wrote` (`t6.12/r1/probes/stream_replacement.txt`, which also holds the oracle's
97.1 and rc 139).

### Checks, on `d995b8f17`

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-classes --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1155 passed.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --test dispatch_seam --test environment_seam
  --test security_replacements --test refusal_sites --test owners --test closed_phases --test
  phase_ten_reasons --test representation_refusals --test refusal_dispositions --test coverage
  --test rexxc --test licensed_divergences --test package_requires`: exit 0. On `3f3dea5f8` it was
  exit 101: `dispatch_seam`'s one-caller rule for `method_is_protected`, which `d995b8f17` restores.
* `memcap 16G cargo test -j 4 --release -p rexx-parse -p rexx-classes`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1132 of 1132 matching.
* The whole suite with the table at `t6.12/r1/suite/table.tsv`: exit 101 as expected, 402 groups,
  4 red, 279 s. Against `t6.12/suite/tests.tsv`, the only changed row is SysFileTree
  `TEST_TREE_DIRECTORY`, the known flake, which passes this time (`t6.12/r1/suite/changed-rows.txt`).

### Perf

`t6.12/r1/callgrind.txt`: pin, round 0 (`7fba9068c`) and `d995b8f17` in one run, exit 0. The
round's own delta, head -libc d% minus parent -libc d%:

| program | parent -libc | head -libc | own | own raw |
|---|---|---|---|---|
| startup | -0.1188 | -0.1201 | -0.0013 | +0.0003 |
| interpret | -4.6297 | -4.6297 | 0.0000 | +0.0065 |
| parse | -0.1416 | -0.1416 | 0.0000 | -0.0001 |
| churn | +0.1069 | +0.1068 | -0.0001 | -0.0001 |
| sayloop | -0.0630 | -0.0637 | -0.0007 | -0.0007 |
| rexxcps | +0.5192 | +0.5192 | 0.0000 | -0.0001 |
| sendloop | -0.2176 | -0.2176 | 0.0000 | 0.0000 |
| dispatch | -0.1454 | -0.1454 | 0.0000 | 0.0000 |

The first shape (`174c357be`) cost 3 Ir per send, all of it counted in `begin_invoke`: sendloop
+0.1086 and dispatch +0.0725 own. Moving the armed manager lookup out of line removed it.
`t6.12/r1/perf-shapes.txt` lists the shapes tried.

### Concerns

1. `invoke_armed_executable` clears the protection row while `begin_invoke` runs, and a native
   body runs inside `begin_invoke`. A send of the same method made from inside that native body
   would not ask the manager. Not measured.
2. A manager-raised condition carries no `RESULT`, as the oracle's does not. An ordinary command's
   condition carries one (`result_is_rc`). rexxref names `RC` for both and does not mention
   `RESULT`, so I followed the oracle.

## Fix round 2

Re-review: `task-6.12-rereview.md`. Code and witness commit `45bd0c3ba`; the records commit that
follows holds `t6.12/r2/` and this section.

### Important 1: instances made before the setter

`unshared_identity` installed the copy with `define`, which gives the class a new behaviour and
leaves every existing instance on the old one, still dispatching the shared identity. It now calls
`ClassGraph::repoint_instance_method`, which repoints the class's own entry and the same entry
(that name, that scope, that identity) in every behaviour already built, retired ones included,
and bumps each changed behaviour's version. Instances made before the setter, those already sent a
message, and a subclass's instances now dispatch the marked copy. Other classes' entries for the
shared identity have other scopes and stay untouched.

Witness `lang/security_manager_existing_instances.rex`, in `phase-9.txt` with its
`sourceline_oracle/` file. Bags are made before `setProtected` on `PUT` and `ITEMS`, two of them
already sent a message, alongside a `MyBag`, an Array (with `APPEND` set) and a Table. A managed
routine's `put`, `items` and `append` to each send `METHOD` for every Bag, the `MyBag` and the
Array, and none for the Table. Bags made before `setPrivate` on `REMOVE` give 97.2, and a Table's
`remove` runs. On round 1's binary the three Bag lines send no event and the two Bags made before
remove without error (`t6.12/r2/witnesses/summary.tsv`, red is `d995b8f17`).

### Important 2: a per-call decision

`lift_protection` and `restore_protection` are gone, so no method's flags change during a run.
`invoke_armed_executable` sets `Interp::unasked_run` around its `begin_invoke`. The seam reads the
mark after `security_armed` with `std::mem::take`, so it consumes the mark on that call's own pass
and asks no manager for it. `method_is_protected` is still called only inside the seam
(`tests/dispatch_seam.rs` passes). The unarmed send path tests `security_armed` alone, as before.
`lang/security_manager_directory_entry.rex` gains a protected method that, run with `Object~run`,
sends itself: identical to the oracle.

### Minor 3: entry 55

The heading names the measured shape, an untrapped `loadPackage` with `NAME` replaced by
`.object~new`. The entry adds that a trapped run after a `.nil` replacement can answer 3.1 naming
`an Object`.

### Checks, on `45bd0c3ba`

* `cargo fmt --all --check`: exit 0.
* `memcap 8G cargo clippy -j 4 -p rexx-exec -p rexx-classes --all-targets -- -D warnings`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-exec --lib`: exit 0, 1155 passed.
* The table and seam set of round 1 (`dispatch_seam`, `environment_seam`, `security_replacements`,
  the refusal tables, `licensed_divergences`, `package_requires`): exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-parse -p rexx-classes`: exit 0.
* `memcap 16G cargo test -j 4 --release -p rexx-core --test unsafe_sites`: exit 0.
* `REXX_CORPUS_GATE=1 memcap 16G cargo test -j 4 --release -p rexx-exec --test corpus`: exit 0,
  1133 of 1133 matching.
* The whole suite with the table at `t6.12/r2/suite/table.tsv`: exit 101 as expected, 402 groups,
  4 red, 288 s. Against `t6.12/r1/suite/tests.tsv` no row changed (`t6.12/r2/suite/changed-rows.txt`
  holds only its command).
* Every witness of this task: identical to the oracle (`t6.12/r2/witnesses/summary.tsv`).

### Perf

`t6.12/r2/callgrind.txt`: pin, round 1 (`d995b8f17`) and `45bd0c3ba` in one run, exit 0. With
libc and ld-linux subtracted, head minus parent is 0.0000% on all eight programs, sendloop and
dispatch included (at most 216 Ir, rexxcps, of 17.9 G). Raw, it is at most +0.0002% (interpret).
