# Task 6.12 review: security manager

Reviewed `b41fc9257..7fba9068c`. The code is all in `a404ad404`. The other commits in the range are
witnesses, records and controller documents. I built HEAD into the scratchpad (`rev612-target`,
`cargo build --release --bin rexx`, exit 0, sha256 `0a3b9784...`). The parent binary is
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.12-parent/rexx-head`, the implementer's build of
`b41fc9257`. Every probe ran from a fresh directory: the oracle under the standard wrapper, the crate
under `memcap 2G timeout 20`. I compared stdout, stderr and rc separately. The probe sources were in
`scratchpad/rev612-probes.lQEo/`, which is not durable, so the deciding lines are quoted below.

## Spec compliance

The two clusters' tests pass, and the three witnesses match the oracle on stdout, stderr and rc
(rerun here). The `.nil`-as-set rule for ENVIRONMENT, LOCAL, METHOD, CALL and COMMAND follows the
documentation as well as the oracle. But the change opens two process-killing regressions on programs
the parent ran cleanly (findings 1 and 2), and Ruled 13 is not met for STREAM (finding 5). For REQUIRES
NAME the task's own rule was not applied (finding 6).

## Strengths

* **Every built-in class has the oracle's `isProtected` set.** For every class in `.environment`, I
  listed each method whose `isProtected` is 1, over `instanceMethods(c)`, `instanceMethods` and
  `methods(.nil)`. Oracle and HEAD give the same 585 lines, and the parent gives 0. The generated table
  agrees: `setup_classes.rs` from the parent's build and from HEAD's differ only in the
  `AddProtected*` rows. With `AddProtectedInstanceMethod` mapped back to `AddInstanceMethod` and
  `AddProtectedClassMethod` to `AddClassMethod`, only the enum declaration differs. That leaves
  19 changed rows, the same as the 19 `AddProtectedMethod`/`AddClassProtectedMethod` lines in
  `Setup.cpp` (455-478, 937-938, 1107-1108, 1134, 1157, 1161, 1198). No row moved or was renamed.
* **The METHOD event matches the oracle, with the same arguments, for each protected primitive and
  through each send path.** I sent BaseClass, MetaClass, Subclasses, Superclasses, Superclass,
  Define (with one and two arguments), DefineMethods, Delete, Uninherit, Directory SetMethod (also
  `setMethod('X', , )`) and UnsetMethod, Method SetProtected and SetSecurityManager, Routine and
  Package SetSecurityManager, Package Options and `.Package~defaultOptions`. Each was sent plainly and
  also through `send`, `sendWith`, `.Message~send`, `~~`, INTERPRET and `superclass:.Class`. The manager
  either passed, handled with a result, or handled with none. stdout, stderr and rc are identical.
  `OBJECT`, `NAME`, and the size, items and holes of `ARGUMENTS` match:
  `METHOD ARGUMENTS=[1/1:1=X,] NAME=SETMETHOD OBJECT=Directory/a Directory`. A metaclass that
  overrides `superclass` sends no event, and a Directory subclass sends `SETMETHOD` with
  `OBJECT=MYDIR`, as on the oracle. A manager that raises 98.948, answers `banana` or returns nothing
  matches too.
* **`setProtected` matches the oracle for copies, `define`, `setMethod` on an object,
  `Method~new` and the no-manager case.** Probes: a fresh `Method~new`, then `setProtected`, then
  `define`, asks the manager. A copy of a protected method is protected. `Object~setMethod` with a
  protected method asks (`METHOD Q`). `.Method~new` with a protected method as context or source is
  0. `define` of an existing protected method asks (`METHOD ATTR`). `~copy~setProtected` leaves the
  source at 0. Uncopied, `.CircularQueue~method('PUSH')~setProtected` affects CircularQueue alone.
  With no manager in the running package, the method just runs (`after arming, no manager here: m0
  ran baz ran`). The witnesses' `setProtected` claims hold.
* **The armed flag behaves.** After `r~setSecurityManager` with no argument, a second call sends no
  new event. A Routine, a Package prolog and a Method each install their manager (witness). A
  `reply`ed method running on its own thread asks its package's manager
  (`reply -> replied Object events: METHOD SUPERCLASS`, same as the oracle).
* **Perf reproduces.** `callgrind.sh -r 1 -B 0.5 -p "sendloop dispatch rexxcps startup"
  pin=... parent=... head=...`, exit 0. Head minus parent, libc and ld-linux subtracted: sendloop
  +2360 Ir in 13.79 G, dispatch +2360 in 20.64 G, rexxcps +4956, startup +2515 in 57.3 M (+0.0044%).
  Raw sendloop and dispatch drop (-0.2049, -0.3162 against the pin), which is the known glibc swing.
* **Tests:** see the Checks section at the end.

## Issues

### Critical

1. **A protected primitive sent from a started message panics the process once any manager has been
   installed (rc 101).** The parent runs this program.
   ```
   r = .Routine~new("r", "return 1")
   r~setSecurityManager(.object~new)
   m = .Array~start('SUPERCLASS')
   say "superclass via start:" m~result~id
   ```
   Oracle rc 0, `superclass via start: Object`. Parent rc 0, same output. HEAD rc 101:
   `panicked at crates/rexx-exec/src/activation.rs:1254:42: a live activation`, with the earlier
   output lost. `.Directory~new~start('SETMETHOD', 'X', 'return 1')` and
   `.Message~new(.Array, 'SUPERCLASS')~start` panic the same way. The cause predates the task:
   `check_protected_method` calls `effective_security_manager`, which calls `self.activation()`, and a
   started message's activity has no running activation. The parent panics the same way for a Rexx
   `::method foo protected` sent by `~start` (`startpanic2`, parent rc 101). But the task has made
   every `Setup.cpp` protected primitive (`superclass`, `define`, `setMethod`, ...) reach that path,
   so the trigger now covers ordinary class introspection. A server that sandboxes one agent routine and
   later starts a message elsewhere dies. Fix: `effective_security_manager` answers `None` when no
   activation is running. The oracle sends no event there (`start -> Object events:` is empty).
   Add a witness line for `~start` of a protected primitive and of a protected Rexx method.

2. **A protected `PUT` on Directory makes every checkpoint recurse until the stack overflows (rc
   134).** The parent runs both programs.
   ```
   .Bag~method('PUT')~setProtected          -- or .Directory~method('PUT')~setProtected
   mgr = .Mgr~new                           -- ::method unknown: say "event" ...; return 0
   r = .Routine~new("", "say 'in routine'; return .Array~superclass~id")
   r~setSecurityManager(mgr)
   r~call
   ```
   Oracle rc 0: `in routine` / `event METHOD SUPERCLASS` / `result Object`. Parent rc 0. HEAD rc 134,
   `thread 'rexx-interp' has overflowed its stack`, and stdout is empty, so even `in routine` is lost.
   With `.Directory~method('PUT')~setProtected`, a `.stdout` lookup (a LOCAL event) overflows the
   same way. There are two causes, and each needs its own fix:
   * `security_arguments` fills the info directory by *sending* `PUT`, so the send passes the
     dispatch seam. A protected `PUT` then asks the manager, which builds another info directory, and
     so on without end. `SecurityManager.cpp` calls `securityArgs->put` directly, and the reads already
     do the same (`security_entry` now calls `hash::store_get`). The writes need the same direct path.
   * The `Bag` line reaches Directory because of finding 3.

### Important

3. **`setProtected` on a donated primitive protects it on every class that shares the row.** After
   `.Bag~method('PUT')~setProtected`, the oracle prints `Directory 0`, `Table 0`,
   `IdentityTable 0`, `Relation 0`, `Set 0`, `StringTable 0` and `Bag 1`. HEAD prints 1 for all seven,
   and the parent prints 0 for all but Bag. In the same way, `.Array~method('APPEND')~setProtected`
   makes `.Queue~method('APPEND')~isProtected` 1 on HEAD and 0 on the oracle. In the oracle,
   `setProtected` marks the one method object. Here, `make_method_protected` marks the shared
   `MethodId` (the donated-identity shape: one native row stands for several classes). Fix: give the
   receiving class's own copy of the method its own identity before marking it, or key the mark on
   that copy, so that only the class asked about changes. `make_method_private` has the same defect,
   and the parent has it too: after `.Bag~method('PUT')~setPrivate`, a `.Table` put gives 97.2 on the
   crate and succeeds on the oracle. That part is a queue candidate. The setProtected half is this
   task's.

4. **A protected method stored with `Directory~setMethod` asks the manager when the entry is read.
   Neither the oracle nor the documentation does this.**
   `m = .Method~new('foo', 'return "m ran"'); m~setProtected; d~setMethod('DIRM', m); d~setMethod('UNKNOWN', m)`.
   Then a routine with a manager that intercepts METHOD reads the entries:

   | read | oracle | HEAD |
   |---|---|---|
   | `.d~dirm`, `.d~at('DIRM')`, `.d['DIRM']`, `.d~entry('DIRM')` | `m ran`, events `LOCAL D` | `intercepted`, events `LOCAL D \| METHOD DIRM` |
   | `.d~nosuch`, `.d['NOSUCH']` | `m ran` | `intercepted`, `METHOD UNKNOWN` |

   The parent matches the oracle. rexxref's Directory `setMethod` says the object "returns the result
   of running method when you access this entry" with `[]`, `at`, `entry` or `remove`. That is an
   entry read, not "a message for a protected method", which is the METHOD checkpoint's definition.
   The oracle and the documented rule agree, and the crate departs from both. The cause: a directory's
   method entry runs through the dispatch seam with the `MethodId` that the task now marks protected.
   Fix: run a directory's entry method without the protection check, as `DirectoryClass::get` does.

5. **STREAM `.nil` is still read as "no replacement", which contradicts the documentation and the
   task's own rule. The agent then writes the very file the manager replaced.** rexxref, STREAM
   checkpoint: "the information directory entry STREAM must be set to an object to be used as the
   stream target. This should be a stream object or another object that supports the Stream class
   methods." An entry set to `.nil` is set (the rule this task adopts everywhere else), so `.nil` is
   the target, and the first stream message to it is 97.1. Probe: the routine runs
   `call lineout 'secret.txt', 'written'`, and the manager returns 1 and sets `info~stream`:
   * `.object~new`: oracle `syntax 97.1 Object "an Object" does not understand message "LINEOUT".` /
     `exists: 0`, then a segfault at exit (rc 139). HEAD and parent give the same lines with rc 0.
   * `.nil`: oracle `syntax 97.1 Object "The NIL object" does not understand message "LINEOUT".` /
     `exists: 0`, rc 139. HEAD and parent: `nil -> wrote` / `exists: 1`.

   So the oracle's crash at exit is not specific to `.nil`: it follows *any* replacement that is not a
   Stream. This crate already declines to copy that crash for `.object~new` without treating the entry
   as unset. The justification in `run.rs`'s comment and in the report's Concern 1 is therefore false.
   It reads ".nil crashes the oracle, so .nil is read as no replacement", but the crash is not caused
   by `.nil`, and reading the entry as unset is not how the crash is avoided. Following the documented
   rule gives the oracle's own observable up to the crash, so no Deviation is involved. The crash
   itself is an oracle bug that is not copied, as it already is not for every other object. Fix: drop
   the `.nil` filter in `check_stream_access`, delete the comment, and add a witness line. The
   `oracle-crashes.txt` proposal should name the general trigger: a STREAM replacement that does not
   support the Stream methods. The parent behaves the same way, so this is not a regression. But this
   task decided the behaviour, documented it, and Ruled 13 governs that decision.

### Minor

6. **REQUIRES NAME `.nil` follows the documented rule on the oracle, not here.** rexxref: "the entry
   NAME in the information directory is used as the actual file to load". A `.nil` entry is set, its
   string is `The NIL object`, and there is no such file. On the oracle that is exactly what happens,
   and it treats `.nil` like any other unloadable replacement:
   ```
   nil     result: syntax 3.1 Failure during initialization: File "The NIL object" is unreadable.
   object  result: syntax 3.1 Failure during initialization: File "an Object" is unreadable.
   missing result: syntax 3.1 Failure during initialization: File "nosuchfile.cls" is unreadable.
   unset   result: syntax 43.901 Could not find file "p4req.cls" for ::REQUIRES.   (one event)
   ```
   (`nil`, `object` and `missing` each send two REQUIRES events.) The `RexxString *` cast at
   `SecurityManager.cpp:343` is unchecked, but the observable is the documented one, so Concern 2 is not
   an oracle departure. HEAD gives `nil` the `unset` answer (one event, 43.901) because of the new
   explicit `.filter(|name| *name != ObjRef::NIL)` in `install.rs`. HEAD gives `object` and `missing`
   `43.901 Could not find file "p4req.cls"`, which names the *original* file, and the parent does the
   same. So the wider divergence is that an unloadable replacement name is reported as the original
   name with the wrong error. It predates the task and is a queue candidate. The `.nil` filter should
   go with finding 5, so that `.nil` takes whichever path a string replacement takes.
   `SECURITYMANAGER .nil` read as none is the oracle's explicit `secObject != TheNilObject`, and the
   documentation says nothing either way. No change is needed there.

7. **A manager-raised FAILURE or ERROR ignores the RC entry.** The manager sets `RC` to `.nil` (or
   `'abc'`) and sets `FAILURE`. The oracle's handler sees `rc == .nil` as 1 and
   `condition('o')~rc` as `The NIL object` (or `abc`). HEAD and the parent give `failure 0 0`. The
   documentation says "RC: The command return code". The report's "COMMAND `RC` `.nil` makes `rc`
   `.nil`" holds only when no condition is raised, and `command_from_manager` keeps the object only
   on that path. This predates the task. Queue it, or fold it into the round, since the round touches
   this reader.

### Queue candidates (outside the clusters, present on the parent)

* `setPrivate` on a donated primitive (finding 3's parent half).
* The REQUIRES replacement-name error and the file name in its message (finding 6).
* The RC entry with FAILURE/ERROR (finding 7), unless it is folded into the round.

## Assessment

**Task quality:** Needs a fix round.

**Reasoning:** The protected-primitive table is exact, and on a plain send the METHOD event matches
the oracle in every shape I built. The `.nil` rule is right for the checkpoints it was applied to. But
marking primitives protected sends two new kinds of program down paths that kill the process. One is
a started message: the panic predates the task, and the task widens it to every protected primitive.
The other is a protected Directory `PUT`, which recurses until the stack overflows. `setProtected`
also reaches classes it was not sent to, and a Directory entry method. STREAM keeps a `.nil` special
case that the documentation does not support, and its stated justification is false.

**Verdict: CHANGES REQUESTED** (findings 1-5. Finding 6's `.nil` filter goes with 5. Finding 7 and the
pre-existing halves can be queued.)

## Checks

All runs from `rust/`, with `CARGO_INCREMENTAL=0`, the scratchpad target dir and `memcap 8G -j 4 --release`.

* `cargo test -p rexx-exec --lib`: the first run had 1154 passed and 1 failed. The failure was
  `scheduler::tests::native::quick_native_calls_at_the_smallest_bound_run_alike` (`left: (120, "",
  "...a native call in the simulation mode that runs longer than its bound is not implemented")`).
  Probes were running on the machine at the same time. Three isolated reruns of that test passed, and
  a full rerun had 1155 passed, exit 0. This is a timing-bound test under load, and the task does not
  touch the scheduler.
* `-p rexx-parse --test sourceline_oracle`: exit 0.
* `REXX_CORPUS_GATE=1 -p rexx-exec --test corpus`: exit 0, `1126 of 1126 matching`.
* Witnesses rerun by hand: all three identical to the oracle on HEAD. The parent's rc is 165 for
  `protected_primitive` and `nil_entry`.
