# Task 10 fix report: the close run's concurrency_tests failures

Base 9cd9ebd7c. Commits dbec28ca8 (fix, witness, Method rows) and 18bd99117 (ATTRIBUTE and METHOD
rows). Probes ran from fresh directories under /tmp/claude-1000/p9/t10fix/.

## directives/METHOD TESTDELEGATE: fixed

Cause: `Object~instanceMethod` and `~instanceMethods` on a class object keyed the `Method` object
by `class_of_value(receiver)`, which is `.Class` for every class object. So the first class method
named `CD` that any class built became the answer for every class's `CD`, with its flags. The test
builds several packages each with a class `test` whose `classDelegate` has different options, so
the later blocks read the first block's `isPrivate`/`isProtected`/`isGuarded`.

Oracle vs crate (before the fix), `.package~new` twice with `::method cd class guarded` and then
`::method cd class protected unguarded`, `isProtected isGuarded` of the second: oracle `1 0`, crate
`0 1`. stdout differed, stderr empty on both, rc 0 on both. A two-class program (`.E` and `.C` each
with a class method `M`): `.E~instanceMethod('M') == .C~instanceMethod('M')` oracle `0`, crate `1`.

Fix: `method_objects` is keyed by (class, class side, name). Class-side entries from
`instanceMethod`/`instanceMethods` key by the defining class (the lookup's scope) on the class side;
`defineClassMethod` holds its object under the class side too; instance-side keys are unchanged.
Measured against the oracle on the fixed binary, all equal: a subclass reaches its superclass's
class-method object (`.D~instanceMethod('M') == .C~instanceMethod('M')` is `1`), the class-side and
instance-side objects of one name stay distinct, `instanceMethods` items equal `instanceMethod`'s,
and `.Array~instanceMethod('NEW') == .List~instanceMethod('NEW')` is `0` while `METHOD`'s is `1`.

The same fix passes directives/ATTRIBUTE TESTDELEGATE.

Witness: `rust/corpus/lang/class_side_method_objects.rex` in `phase-9.txt`, sourceline expectation
generated with the `.Package~new` driver. Its oracle and crate stdout, stderr and rc are identical
(rc 0, stderr empty). The pre-fix binary was not run on the witness file itself; its statements
were run on it as the probes above, which differed.

## base/class/Method TEST_NEW_ARRAY_FROM_FILE and TEST_NEW_FILE_COMPILED: not fixed

Cause: both tests run `address system "rexxc"` to compile a source file into an encoded and a
binary compiled image, then load each through `.Method~new(name, array)` and `.Method~newFile`.
The crate side's PATH resolves no `rexxc` by design (suite.rs `check_path`), and the crate does not
load the C++ interpreter's compiled image format. Measured: with the oracle's `rexxc` output in
place, the crate raises 13.1 on the encoded image through both `new` and `newFile` where the oracle
answers `43`/`47`; with no `rexxc` the command is rc 127 and the files are missing.

Not listed in `suite-9d-set.txt`: spec section 3 names both tests as 9b/9c witnesses, and the
file's header excludes them by name. They reach the gates through the existing `DIFFERING`
mechanism: the Method `whole` rows now carry their outcome, which `sim_gate` reads through
`failing_unswitched`.

## Re-derived DIFFERING rows (concurrency_tests.rs)

* base/class/Method whole, normal and every: `error, assertions 92, rc 2, last started
  TEST_SOURCE, failing [TEST_NEW_ARRAY_FROM_FILE TEST_NEW_FILE_COMPILED]`.
* base/directives/ATTRIBUTE whole, normal and every: `error, assertions 303, rc 2, last started
  TEST_SET_RETURN, failing [TESTMISPLACEDCLASSMETHOD]`.
* Removed, each part now agreeing with the oracle: ATTRIBUTE derived; METHOD whole, rest and
  derived.

## Checks (all from rust/, exit status read unpiped)

* `cargo fmt --all --check`: 0.
* `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings`: 0
  (log shows `Checking rexx-exec`), at both commits.
* `REXX_CORPUS_GATE=1 REXX_WHOLE_GROUPS_ONLY=base/class/Method.testGroup,base/directives/METHOD.testGroup,base/directives/ATTRIBUTE.testGroup memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests each_group_of_the_derived_list_in_one_run_in_both_modes`:
  101 at dbec28ca8 (ATTRIBUTE whole rows stale), 0 at 18bd99117 with `listed as differing,
  agreeing: []`.
* `REXX_CORPUS_GATE=1 REXX_SIM_ONLY=<group> memcap 8G cargo test -j 4 --release -p rexx-exec --test concurrency_tests the_seeded_gate -- --nocapture`,
  once per group: Method 0 (210 runs, 0 reds), METHOD 0 (210 runs, 0 reds), ATTRIBUTE 0 (140 runs,
  0 reds).
* `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Method,base/directives/METHOD,base/directives/ATTRIBUTE memcap 8G cargo test -j 4 --release -p rexx-exec --test suite`:
  101; METHOD green, Method red on the two rexxc tests, ATTRIBUTE red on TESTMISPLACEDCLASSMETHOD.
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus`: 0 (29
  passed, 1 ignored).
* `memcap 8G cargo test -j 4 --release -p rexx-parse --test sourceline_oracle`: 0.
* `memcap 8G cargo test -j 4 --release -p rexx-exec --lib`: 0 (1087 passed).
* `REXX_CORPUS_GATE=1 memcap 8G cargo test -j 4 --release -p rexx-exec --test bootstrap_install_oracle --test introspection_scopes --test collection_scopes`: 0.

## Left open

* `suite-9d-set.txt` still lists METHOD and ATTRIBUTE TESTDELEGATE. It is derived from the run at
  2415e1733 by its header's command and was not edited; both now pass.
* Instance-side identity across a subclass still differs: `.D~new~instanceMethod('M') ==
  .C~method('M')` (M defined on C) is `1` on the oracle and `0` here. Unchanged by this fix.
* Found while probing: a label inside a `DO` block reports `47.2 ... found "&1"` here where the
  oracle names the label (`found "S1"`).
