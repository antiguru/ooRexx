# Phase 9a Task 8 report: NEW on subclasses of String, Stem, Method, Routine and Message

Base `023565ff1`. Status: DONE_WITH_CONCERNS. `Class~enhanced` running `NEW` is not done: it moved to
Task 8b (plan commit `5b97e7922`); the options are below.

Commits: `5044c382b` (stage 1, String and Stem), `988bf1e08` (stage 2, Method, Routine, Message),
`1c0e1d769` (stage 3, the queued items), `e2f5408cb` (instruction-count follow-up), and the commit
carrying this report.

Probes ran from fresh empty directories under `/tmp/claude-1000/p9/t8/` (runner `run.sh`: a copy
of the file in a new directory, the oracle under the standard wrapper with `timeout -k 5 20`, the
crate's release `rexx-run` under `memcap 2G` and `timeout 20`; stdout, stderr and status compared
separately). `rust/corpus/oracle-crashes.txt` was read first; no probe is near an entry. Every
witness's stdout was read on the oracle side (the lines quoted below) and compared byte for byte.

## String

**Representation.** A subclass instance is a `Body::Instance` of the subclass with
`NativeState::Text(Box<[u8]>)` (`rexx-core/src/body.rs`), the shape scout D section 3 gives:
MutableBuffer's. `to_text`, `text_len`, `try_text` and `to_number` read the bytes in their instance
arms; the operator natives, `numeric_receiver`, `d2x`/`d2c`, `MAKESTRING` and `Object~STRING` read
the receiver through `base_string` (a new rooted `String` holding the bytes); `hash_value` answers
the bytes' hash. INIT gets the arguments after the first. The bytes are counted toward the next
collection without being held (`Interp::charge_unheld`), so `Body::held_bytes` is the base's.

**Oracle probes** (`probes/s_basic`, `s_init`, `s_ident`, `s_ops`, `s_ctx`, `s_err`, `o_req`): all
byte-identical after. Read on the oracle: `K abc 3`, `a K`; `x~makestring`, `~string`,
`~request('STRING')` answer `String`; a Table keyed by the instance answers `'abc'` both ways;
`x~hashcode~c2x = 'abc'~hashcode~c2x` is `1`; a subclass's own `MAKESTRING` is what `'a' || o`,
`say o` and `length(o)` read (`aMS`, `MS`, `2`) while `o~length` and `o || 'q'` read the bytes
(`3`, `abcq`); `.k~new` is 93.903, `.k~new('abc')~abs` 93.943, `.k~new('abc') + 1` and
`1 + .k~new('abc')` 41.1, an abstract subclass 98.989.

**Identity audit** (`return this` in `StringClass.cpp`, `StringClassSub.cpp`, `StringClassMisc.cpp`,
`StringClassWord.cpp`; found with an awk over `return this;` naming the enclosing function). Each
site, the condition the C++ tests, and the witness lines (oracle, `t` column = class):

| C++ | condition | instance | neighbour that builds |
|---|---|---|---|
| `concatRexx` (`""`, `||`, `APPEND`) | other's length 0 | `concat empty`, `abuttal empty`, `append empty` K | `concat`, `append` String; `blank empty` String |
| `center` | width == length | `center 3`, `centre 3` K | `center 4` String |
| `right` | size != 0 and size == length | `right 3` K | `right 5`, `right 0 empty` String |
| `copies` | count 1, length > 0 | `copies 1` K | `copies 2`, `copies 1 empty` String |
| `reverse` | length <= 1 | `reverse 1` K | `reverse 3`, `reverse aba` String |
| `delstr` | position > length | `delstr 9`, `delstr empty` K | `delstr 2 0` String |
| `delWord` | count 0, or fewer words than the position (non-empty) | `delword 5`, `delword 1 0` K | `delword 1`, `delword empty` String |
| `strip` | nothing stripped, length > 0 | `strip`, `strip t` K | `strip blanks`, `strip empty` String |
| `changeStr`, `caselessChangeStr` | count 0 or no match | `changestr none`, `changestr empty`, `changestr 0`, `caseless none`, `caseless 0` K | `changestr same`, `caseless hit` String |
| `upperRexx`, `lowerRexx` | start past the end, or range 0 | `upper 5`, `upper 1 0`, `upper empty`, `lower 5`, `lower 2 0` K | `upper`, `upper ABC`, `lower abc` String |
| `translate` | no tables and no pad: `upperRexx`; else start past the end or range 0 | `translate 9`, `translate tables 9`, `translate tables 1 0` K | `translate`, `translate tables` String |
| `stringValue`, `makeString`, `primitiveMakeString` | base class only | -- | `makestring`, `string`, `request` String |
| `concat`, `upper()`, `lower()` (no arguments), `stringTrace`, `evaluate`, `getValue`, `getRealValue` | internal; no Rexx send reaches them for an instance (`upper()` and `lower()` get a fresh base string from `stringArgument`) | -- | -- |

The natives return the receiver where the condition holds; for a base string that is the same
value, so no branch tests for a subclass.

**Witness:** `rust/corpus/lang/string_subclass_new.rex` (138 lines; oracle rc 0). A long-value line
keeps the instance's bytes off the inline representation: with `base_string`'s `push_temp`
deleted, `collect_stress`'s `the_l0_subset_passes_again_under_collect_on_every_allocation` panics
`a live value` (mutation run, exit 101; restored from a copy, `git diff --stat` back to the
committed counts).

## Stem

**Representation.** A subclass instance is a `Body::Instance` whose pool holds a `Body::Stem` under
the `Stem` scope (`STEM_STORE`), Array's shape. Every `Stem` native takes the store through
`stem_of`; `EMPTY` answers the instance and `REQUEST`'s `ARRAY` arm tests the instance. The string
value (`redirect_of`'s instance arm, `classify_string_conversion`'s instance arm, `Object~STRING`) is
the stem's; `COPY` duplicates the store.

**Oracle probes** (`st_basic`, `st_init`, `st_more`, `st_err`, `st_remove`): byte-identical after.
Read: `say x` is `S.`; `s. = x` and `use arg q.` take `x` as the default (`S. Stem 1`, `r S. Stem`);
`.k~new` renders `[]` and `x2[7]` is `7`; `x~request('STEM')~class~id` is `Object` (`.nil`);
`x~empty~class~id` is `K`; `x~copy` has its own tails (`a changed`); `x~put('dflt')` makes `say x`,
`x[99]` and `x~string` `dflt`; `q['A']` is `Q.A` beside `q['a']` `1`.

**Found and fixed on the way:** `Stem~remove()` with no tail (`StemClass::remove`,
`classes/StemClass.cpp:346`) answers the value and makes the stem's name its value. Ours answered
the tail `""` and changed nothing. Oracle, read: `b. = 5; b.1 = 'one'; say b.~remove() b.2 b.1` is
`5 B. one 1`, a second `remove()` is `B.`, and `.stem~new('X.')~remove()` leaves `n[1]` reading
`X.1` (a stem with no value keeps none). In the stem witness.

**Witness:** `rust/corpus/lang/stem_subclass_new.rex` (61 lines; oracle rc 0).
`dispatch/tests.rs`' `a_stem_receiver_answers_stem_and_renders_its_own_value` pinned the old
refusal; its last case now asserts `K K.`.

## Method and Routine

**Representation.** `NEW` and `NEWFILE` compile as before; for a subclass, `native_as_instance`
(`dispatch/construct.rs`) turns the built object into a `Body::Instance` of the subclass in place,
so its identity, and every side table keyed by it (`executable_sources`, flag writes, compiled
bodies), stays valid; the annotation table moves to the pool under the base class's scope. This
differs from scout D's "class parameter on the two compile functions" in mechanism only (14 call
sites and `install_executable`'s directive branch would have needed it); the object reached is the
same. `~scope` answers `.nil` and `~annotations` the pool's table; `COPY` carries the records and a
table of its own; `~addRoutine` admits a subclass instance (`classArgument`,
`PackageClass.cpp:1893`). INIT gets the arguments after the context, so a base `NEW` with a fourth
argument is now the oracle's 93.902 from `Object~init` (`TEST_NEW_FOUR_ARGS` in both groups now
pass). `loadExternalMethod`/`loadExternalRoutine` sent to a subclass answer the base class's object
(`MethodClass::loadExternalMethod` does not call `completeNewObject`; oracle `Method`).

**Oracle probes** (`ex_basic`, `ex_init`, `ex_err`, `ex_file`, `ex_more`): identical after, except
`ex_err`'s compile-error case, where ours prints `"&1"` for the token (concern 3). Read:
`KM 1 return 42 1 0 Package`, `The NIL object StringTable`, copy `KM return 42 1 0`, `KR 6 10`,
`addRoutine` then `rr(3)` `4`, `call (r)` 43.1 `"a KR"`, `define`, `setMethod` and `run` with a KM
93.974, `init 2 x y` for KM with two leftovers, `rinit 0`, `KRI 42` from `newFile`.

**Identity audit:** none; the C++ `newRexx` answers the built object, and the side tables keep it.

**Witness:** `rust/corpus/lang/executable_subclass_new.rex` (86 lines; oracle rc 0; it writes and
deletes `executable_subclass_new.tmp` in the run directory).

## Message

**Representation.** A `Body::Instance` of the subclass whose pool holds the `MESSAGE_*` entries under
the `Message` scope: `native_entry`'s instance arm, `set_native_entry`'s cold
`set_message_subclass_entry` and `remove_native_entry` test `message_entry_scope`; `is_message`
admits a subclass instance, so `COPY` resets it as the base's.

**Oracle probes** (`msg`): identical, eight runs of eight. Read: `init 0` before each, `KMSG 2 Array
ITEMS 0`, `3 1 1 0` after `start`, `1 97.1` and the started thread's 97.1 on stderr, an INIT that
`expose`s (`HELLO set`).

**Witness:** `rust/corpus/lang/message_subclass_new.rex` (29 lines; oracle rc 0).

## Queued items

* `2026-10-10-subclass-package`: `package_new` makes the package object an instance of the subclass
  with `native_as_instance`, as for Method. The queued probe answers `1` `3` on both. Witness
  `lang/package_subclass_new.rex` (routines answer it as their package, INIT gets the leftovers,
  `addPackage` admits it, an abstract subclass 98.989).
* `2026-10-10-subclass-rexxinfo`: `directive_class` answers 99.949 (`Raised::not_a_valid_class`)
  where `.environment`'s entry is an object that is not a class and no class table holds the name
  (`ClassResolver::lookup`, `ExpressionClassResolver.cpp:159`). Probed `subclass` `rexxinfo`, `nil`,
  `true`, `endofline`, `environment`, `metaclass rexxinfo`, `inherit nil`: all rc 157 on both;
  `subclass nosuchthing` stays 98.909 rc 158. Witness `lang/class_target_not_a_class.rex`, added to
  `collect_stress`'s `NO_ALLOCATION_PROGRAMS`.
* `2026-09-30-hash-subclass-init-capacity` and `2026-09-30-stringtable-subclass-new-arg`: one cause.
  `native_hash_new` checks no capacity and installs no store; `native_capacity_init` (each hash
  class's `INIT`) installs one sized for its argument unless one exists
  (`HashCollection::initialize`); a subclass whose INIT does not forward gets the default store on
  first use. Read: `kt 3 4 5 6 10 7 11 8 12 9 1 2` (default order) beside `table100 1 2 3 4 10 5 11
  6 12 7 8 9`; `.ks~new('abcdefghij', 'q')` runs INIT; base `.stringtable~new('x')` now carries the
  oracle's `Compiled method "INIT" with scope "StringTable".` line. Witness
  `lang/hash_subclass_init_capacity.rex`.

## `Class~enhanced` (moved to Task 8b)

The oracle (`ClassClass.cpp:1440`-`:1480`) makes a dummy subclass "Enhanced Subclass" of the
receiver, merges the methods into its instance behaviour with scope `.nil`, sends `NEW` to the
dummy (so `NEW`'s INIT reaches an enhancing INIT, and a primitive like Array builds a subclass
instance), then sets the owning class back to the receiver and the enhanced flag. Ours installs own
methods after `new_instance` and then sends INIT. Two blockers for "send NEW instead":
`.array~new(3)` builds a `Body::Array`, which carries no own methods, and `NEW` sends INIT before
own methods could be installed.

* A. Per-call dummy subclass with no methods; a pending-enhancement hook in `new_instance`, keyed by
  the dummy class, installs the own methods before `NEW`'s INIT; afterwards the instance's `class`
  is set back to the receiver and it is marked enhanced. Keeps today's own-method representation.
  Costs: one check on every `new_instance`; one class object per call, never collected (D59).
* B. The oracle's shape: per-call dummy whose instance behaviour holds the methods under `.nil`
  scope; `NEW` to the dummy; reset the class; mark enhanced. No hook. Costs: the same class per
  call; enhanced objects change from own methods to behaviour methods, so every enhanced witness
  needs re-checking; whether the registry can define instance methods under `.nil` was not checked.
* C. A with the dummy cached per receiver class and the pending entry keyed by (activity, dummy),
  stacked for nesting: no class retention, more machinery.

## Refusal tables

`refusal-sites.tsv` re-derived in `1c0e1d769` (`REXX_REFUSAL_SITES_REFRESH=1`). Modulo line numbers
(`sed 's/\.rs:[0-9]*/.rs:N/'` on both sides, then `diff`) the only change is one added row:
`Raised not_a_valid_class body crates/rexx-exec/src/error.rs:N off-send-surface - - -`. No `Loud`
constructor was added or deleted; `unbuilt_new` remains for `native_package_new`.
`refusal_dispositions`, `closed_phases` and `owners` green.

## Group verdicts

Before: Task 3's baseline (`docs/superpowers/records/2026-10-11-phase-9a/suite-baseline/`). After:
the final tree, `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=<below> ... --test suite` (exit 101: the gate
test is red while any group is, as at the baseline).

| group | before | after |
|---|---|---|
| base/class/String/String | green | green |
| base/class/Stem | green | green |
| base/class/Message | red (3 blocked, `Object~MAKEARRAY`) | green |
| base/class/Method | red: 6 | red: TEST_NEW_ARRAY_FROM_FILE, TEST_NEW_FILE_COMPILED |
| base/class/Routine | red: 4 | red: TEST_NEW_ARRAY_FROM_FILE, TEST_NEW_FILE_COMPILED |
| base/class/Class | red: TEST_ACTIVATE, TEST_METHODS | same |
| base/directives/CONSTANT | red: TEST_EXPRESSION_NOVALUE_ERROR | same |
| samples/samples | blocked at load | red: TEST_MAKESTRING (`SysTempFileName`); TEST_FACTOR passes |
| base/class/Package | red: 5 | red: TESTRECLAIM01, TEST_NEW01, TEST_PACKAGE_LOCAL_GET_EMPTY, TEST_PROLOG |
| base/keyword/VarRef | red: 7 | red: the same less TEST_UNKNOWN_OK |
| Bag, IdentityTable, collections/directory | red | red, the same tests |
| Directory, Relation, Set, Table, collections/stringtable | green | green |

No test newly fails; the improvements between baseline and now include Tasks 4 to 7's.

## Instruction counts

`bench-programs/callgrind.sh -r 1` over all 27 programs, base `023565ff1`'s binary against the final
one (outputs `/tmp/claude-1000/p9/t8/cgG`, `cgH`; spread 0.0000 everywhere). Largest increase alloc
+0.0589%, then pingmsg +0.0279%, dirread +0.0147%; strings -0.1711%, rexxcps -0.0850%, alloc4c
-0.3218%, decrender -0.2878%, parse -0.0777%; every other program within 0.012%. Stage 1 as
committed was +0.17% on strings: the stem arm in `redirect_of` made every heap string pay five
register saves (read from `objdump`). `e2f5408cb` and `1c0e1d769` moved each new arm out of line
after a per-function `cgdiff.py` named it (`redirect_of`, `alloc_charged`'s inlining into
`array_of_class`, `classify_string_conversion`, `set_native_entry`).

## Checks (final tree, each exit status read unpiped)

* `cargo fmt --all --check`: 0.
* `cargo clippy -j 4 -p rexx-core -p rexx-exec --all-targets -- -D warnings`: 0.
* `REXX_CORPUS_GATE=1 timeout 595 cargo test --release -p rexx-exec --lib --test corpus --test
  collect_stress --test refusal_sites --test refusal_dispositions --test closed_phases --test owners
  --test method_bodies`: 124, the `timeout` cutting `method_bodies` under a load average of 24
  (`collect_stress` took 297 s where it had taken 150). Every binary that finished passed: lib 1085,
  `closed_phases` 8, `collect_stress` 37, `corpus` 27 and 1 ignored.
* `REXX_CORPUS_GATE=1 timeout 595 cargo test --release -p rexx-exec --test method_bodies --test
  refusal_sites --test refusal_dispositions --test owners --test ir_recorded --test trace_oracle
  --test coverage --test gate_table_c --test gate_table_d`: 0.
* `cargo test --release -p rexx-parse --test sourceline_oracle`: 0 (expectations for the seven
  witnesses generated with the module comment's driver over copies in `/tmp/claude-1000/p9/t8/srcg`).
* `cargo test --release -p rexx-core`: 0.
* Suite groups as above: exit 101, verdicts as tabled.

All under `CARGO_TARGET_DIR=/tmp/claude-1000/p9/t8-target CARGO_INCREMENTAL=0 memcap 8G`, `-j 4`.
The per-task check ran past 5 minutes: the gated corpus and `collect_stress` together take 4 to 6.

## Concerns

1. `'12'~translate('', '')` blanks the string where the oracle answers it unchanged (queued
   `2026-10-10-string-translate-empty-tables`): `translate`'s `tablei != NULLSTRING` is a pointer
   test, so a literal `''` is an empty table, not an omitted one. Kept out of the String witness.
2. `.Method~new('m', 'x', , 'y')` (context omitted, a fourth argument given) leaves the C++ with a
   NULL source context (`processNewExecutableArgs` returns before the program-scope default); ours
   uses the caller's package. Not probed.
3. A compile error at the end of a clause prints `"&1"` for its token (`return (`, `-y +y \y`):
   queued `2026-10-10-translation-error-inserts`. Kept out of the witnesses.
4. `dispatch.rs`'s `NATIVE_CLASS_METHODS` still lists `("Package", "NEW", native_package_new)`
   beside `package.rs`'s row that answers; the first looks shadowed. Not changed.
5. `Method~annotations~items` refuses (`StringTable` `ITEMS`, Phase 9) on the base as on a subclass;
   the witness compares the table's identity instead.

## Fix round 1

Review: `task-8-review.md`. Code commit `f3741cfad`; this section and the queued-file closings are
committed after it.

* **I1.** `copy_object` charges a `String` subclass instance's text toward the next collection with
  `charge_unheld`, as `String~new` does. `Body::held_bytes` stays the base's: reporting the text
  there stopped `alloc_charged` inlining into `array_of_class` (+0.18% on alloc, measured in the
  first round). The other subclasses' copies carry no unreported bytes. A copied Stem store is a
  `Body::Stem`, which `held_bytes` reports as 0 for a base stem's copy too. Method/Routine copies
  allocate an annotation table, as `copy_method_object` does for a base one. A Message copy is the
  base's copy. Witness: `rust/crates/rexx-exec/tests/instance_copy_memory.rs`, the reviewer's
  3000 x 1,000,000-byte copy loop under `ulimit -v 1048576`, the cap and harness shape of
  `library_routine_memory.rs`. The corpus runs the crate in-process and uncapped, and
  `collect_stress` collects on every allocation whatever is charged, so neither can see this. Green
  (exit 0). With `charge_unheld(text)` replaced by `let _ = text;` it fails, `(None, "", "memory
  allocation of 1000000 bytes failed ...")`, exit 101; restored from a copy. Probe: oracle `done
  1000000 K` rc 0; the previous head OOM-killed at `memcap 2G`, rc 137; now identical to the oracle.
* **I2.** `STORE_HOLDERS` (less `Properties`, a `Directory` subclass: oracle
  `.properties~superclass~id` is `Directory`) is resolved once in `ObjectModel::build` into
  `store_holders`. `holds_store_class` tests membership, then `is_a`. `initialize_store` reads the
  instance's pool for the store directly instead of `class_of_value` and `read_store`. Bench program
  `rust/bench-programs/hashnew.rex` is added to the set: registered in `callgrind.sh`'s `PROGRAMS`,
  `rexx-bench`'s `PROGRAMS` and `rexx-bench-suite`'s `AXES`, with a README row. Measured, interleaved:
  `bench-programs/callgrind.sh -r 3 -j 6 -p hashnew base=<023565ff1 build> head=<e2f5408cb build>
  fix=<f3741cfad build>`: 1,589,251,545 / 1,915,694,841 (+20.5407%) / 1,595,794,650 (+0.4117%),
  spread 0.0000. The full set (`-r 1`, two batches, 27 programs) on `f3741cfad` against the base
  matches the figures above: largest alloc +0.0589%, pingmsg +0.0279%.
* **Minor 1.** The four queued files carry a closing line naming the witness and `1c0e1d769`.
* **Minor 2.** The audit table's internal row names `upper()` and `lower()`.
* **Minor 3.** The `changestr` pre-scan is left as is (no change requested).
* **Minor 4.** Which `Package` `NEW` row won: `ObjectModel::build` writes `NATIVE_CLASS_METHODS` and
  then `package::RESUMABLE_CLASS_METHODS` into the same method slot through `put_native`, and the
  later write wins. So the `dispatch.rs` row was dead, which `.Package~new` answering on both sides
  confirms. Deleted with `native_package_new`, `unbuilt_new` and `unbuilt_class_method`, which only
  it used. `REXX_REFUSAL_SITES_REFRESH=1` re-derived `refusal-sites.tsv` byte-identical.
* **Minor 5.** The ` -- ` dashes in the doc lines this task added (`native_capacity_init`,
  `native_executable_new`, `native_new_file`, `initialize_store`) are now sentences.
  `executable_name_argument`'s doc no longer names `Package`.

Checks, each exit status read unpiped (load average 28 to 36 throughout):

* `cargo fmt --all --check`: 0.
* `cargo clippy -p rexx-core -p rexx-exec --all-targets -- -D warnings`: 0.
* `cargo clippy -p rexx-bench --all-targets -- -D warnings`: 0.
* `REXX_CORPUS_GATE=1 timeout 595 cargo test --release -p rexx-exec --lib --test corpus --test
  collect_stress ...`: 124. The timeout cut `corpus` after lib 1085, `closed_phases` 8 and
  `collect_stress` 37 had passed (`collect_stress` took 307 s).
* `REXX_CORPUS_GATE=1 ... --test corpus --test refusal_sites --test refusal_dispositions --test
  owners --test instance_copy_memory`: 0.
* `REXX_CORPUS_GATE=1 ... --test method_bodies --test ir_recorded --test trace_oracle --test
  coverage --test gate_table_c --test gate_table_d`: 0.
* `cargo test --release -p rexx-bench`: 101. `every_declared_runnability_still_holds` looks for
  `rust/target/{release,debug}/rexx-run` and does not follow `CARGO_TARGET_DIR`. Every other test
  passed, `the_axis_list_covers_every_bench_program` among them.
* Suite groups String/String, Stem, Package, Bag, Directory, IdentityTable, Relation, Set, Table,
  collections/directory, collections/stringtable, Properties, Method, Routine, Message: exit 101.
  The not-passed set is the one tabled above, and no test newly fails.
