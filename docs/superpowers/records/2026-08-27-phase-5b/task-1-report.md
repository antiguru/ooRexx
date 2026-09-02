# Task 1 report: `~new`, `INIT`, and the instance seam

**Base:** `6fc65c487` (Task 0).
**Probes:** a fresh empty directory under the session scratchpad, absolute paths, crate runs bounded
with `timeout -s KILL 20`, three descriptors read separately on `REXX_ENGINE=ir` **and**
`REXX_ENGINE=tree-walker`, never `2>&1`.

## Re-measurement of the brief's "what is already there" claims

* **`creo.rex` and `abscla.rex` are rc 120 today at `method "NEW" of class "Object" is not
  implemented (Phase 5)`.** TRUE, both engines. `creo` oracle rc 0, `type a savings account` /
  `balance 1000.00` / `rate 6.25`, empty stderr. `abscla` oracle rc 158, stdout `declared AB Class`,
  stderr the `98.989` transcript with a single `Compiled method "NEW" with scope "Object".` frame.
* **`Body::Instance(ScopePools)` exists and is what a class object's own variables use.** TRUE:
  `rexx-core/src/body.rs:122`, allocated at `rexx-exec/src/run.rs:3088` from `Interp::pool_owner`.
* **`receiver_kind` refuses it at `dispatch.rs:1123` with `Err("an instance of a user class")`.**
  TRUE, exact line.
* **`("Object", "INIT", Arity::Fixed(0), native_no_op)` at `dispatch.rs:375`.** TRUE, exact line.
* **`~objectName`/`~objectName=`/`~string`/`~class`/`~isA` answer for a class receiver.** TRUE:
  `.K~objectName`, `.K~string`, `.K~class~id`, `.K~isA(.Class)` agree with the oracle today.
* **`~defaultName` answers for no receiver at all, and is built here from zero.** TRUE.
  `/bin/grep -rn DEFAULTNAME crates/rexx-exec/src/` matches only doc comments, and `.K~defaultName`
  is rc 120 on both engines against the oracle's `The K class`.
* **The instance-side oracle figures.** TRUE, all of them: `.Object~new~string` is `an Object`; an
  instance of `K` answers `a K` for `~string`, `~defaultName` and `~objectName`; `~class~id` is `K`;
  `~isA(.K)`/`~isA(.Object)` are `1` and `~isA(.Class)` is `0`; after `o~objectName = 'zed'`,
  `~string` and `~objectName` answer `zed` while `~defaultName` still answers `a K`.
* **`Interp::pool_owner`'s doc comment names the `SELF` rooting hazard.** TRUE in substance, off by
  four lines: the doc block starts at `run.rs:3062`, the sentence quoted is at `:3077`, and
  `fn pool_owner` is at `:3081`.

### Three things the brief and the plan did not say

1. **`lib.rs:5452`'s comment on `::CLASS ... ABSTRACT` was false.** It read "any other class takes the
   keyword by setting a flag whose reader is `~new`". No such flag existed: `/bin/grep -rni abstract`
   over `crates/rexx-classes/` matched only test files, and `install_class` raised for the metaclass
   case and did nothing at all otherwise. `checkAbstract` needed the flag built as well as read.
2. **`Object~NEW` is a class-side row and `NATIVE_METHODS` cannot express one.** Every row in that
   table is resolved through `lookup_instance_method`; `Setup.cpp:514` binds `New` with
   `AddClassMethod`, so the first build of this task panicked with `NATIVE_METHODS names Object~NEW,
   which that class's behaviour does not answer`. A second table, `NATIVE_CLASS_METHODS`, resolves
   through `lookup_class_method`.
3. **`Interp::to_text`, `Interp::try_text` and `Interp::heap_to_number` each carried an
   `unreachable!` that an instance reaches the moment `~new` answers.**

## What was built

* **`Body::Instance { class, name: Option<Box<[u8]>>, pools }`** (`rexx-core/src/body.rs`). `trace`
  pushes the class the way the `Native` arm pushes its own. `name` is `None` until `~objectName=`
  sets one, which is not the same as holding the default rendering -- see the next section.
* **`Primitive::Instance(class)` and `Behaviour::Instance(class)`**, which is `receiver_kind`'s old
  `Err` turned into an answer.
* **`native_new`**, `completeNewObject`'s four steps in order: `checkAbstract`, the behaviour, the
  `UNINIT` registration, the `INIT` send with `~new`'s own arguments.
* **`NATIVE_CLASS_METHODS`** and its resolution through `lookup_class_method`.
* **`ClassGraph::make_abstract`/`is_abstract`**, set by `::CLASS ... ABSTRACT` after the metaclass
  refusal, and `Raised::abstract_class` for 98.989.
* **`Class~defaultName` and `Object~defaultName`**, both from zero.
* **An instance's `~objectName`, `~objectName=`, `~string`, `~class` and `~isA`**, and the instance
  arms of `to_text`, `try_text`, `heap_to_number`, `text_len_inner` and `operator_operand_gap`.
* **`dispatch.rex` becomes a `Role::Loop` bench axis.** It exits 0 now, and
  `every_blocked_axis_still_fails_on_this_crate` is what said so -- a red test, not a decision anyone
  remembered to make. The plan's Task 10 predicts exactly this and owns the `dispatch` versus
  `dispatchclass` stand-in decision, which is left there.

## The silent wrong answer probing found

**An object's name is a send, not stored text.** `RexxObject::objectName`
(`classes/ObjectClass.cpp:1695`) reads a name something set out of the object's `Object`-scope pool
and otherwise, for anything that is not a base class, **sends `DEFAULTNAME`**;
`RexxObject::stringValue` (`:1157`) is a `sendMessage(OBJECTNAME)`; and the required-string
protocol's own fallback is a `sendMessage(STRING)`. So a user class overriding any of the three
decides what a program sees, and the three reach different sets of contexts. Measured, oracle rc 0,
one program per override:

```
::METHOD defaultName -> overridden       ~objectName, ~string and `say o` follow it;
                                         `o~objectName = 'zed'` then wins over it
::METHOD string      -> from-string      ~string, `say o`, `'x' o` and length(o) follow it;
                                         ~objectName and ~defaultName do not
::METHOD makeString  -> from-makestring  `say o`, `'x' o` and ~request('STRING') follow it;
                                         ~string and ~objectName do not
```

and `defaultName` is genuinely sent each time rather than cached: a `defaultName` counting its own
calls prints `call 1`.

**A first implementation stored the rendering on the object at construction and was silently wrong,
at rc 0, for the first two.** It is replaced. `native_string` sends `OBJECTNAME` for an instance,
`native_object_name` reads the set name and otherwise sends `DEFAULTNAME`, and
`required_string_dispatch`'s fallback sends `STRING` for an instance where every other value kind
keeps the shortcut. `corpus/lang/instance_naming_overrides.rex` is the witness; all three shapes
agree byte for byte on both engines.

**`Interp::reqstr_armed` is armed at construction rather than by a name.** Its own doc named the hole
this task opens -- "a user class that inherited a native `MAKESTRING` would install no `MAKESTRING`
name of its own ... `~new` is refused in this phase, so no such receiver exists yet" -- and the hole
is wider than that once the protocol's fallback can send `STRING`: `~define`, `~defineMethods` and an
inherited mixin are all routes by which an instance's class can come to answer `STRING`,
`OBJECTNAME` or `DEFAULTNAME` differently, and `arm_reqstr_for` sees only the directive route.
`native_new` sets the latch, which costs the protocol walk to a program that builds an instance and
cannot answer wrongly.

## The rooting hazard: what was measured, and what was removed

`Interp::pool_owner`'s doc comment said a running send's receiver is rooted only by the `SELF` slot.
**Measured, that is false for an ordinary send.** `Interp::message_term` takes
`self.roots.push_temp(receiver)` before it resolves anything, and both engines enter a method body
through that one function, so the receiver is rooted by the sending clause's temporaries frame for
the whole send. `native_new` takes a second `push_temp` over the `INIT` send it makes itself, which
is the one window `message_term` does not cover, because there the receiver is the new object and
`message_term`'s temporary holds the class.

**A third root was written and then removed.** `Interp::collect_now` was extended to hand the sweep
each activation's `method_identity.receiver` beside its context object. Measured: with the witness
program, removing that extension left the targeted case and the whole of `collect_stress.rs` green,
and so did removing `message_term`'s `push_temp` on its own. It is deleted rather than kept, because
nothing witnesses it.

**Two false starts on the witness itself, both worth recording.**

* The brief's program stores the instance in `o`, so the caller's *variable slot* roots it and no
  rooting mutation can redden the row. The committed program therefore clobbers `SELF` inside `INIT`
  as well, where `native_new`'s `push_temp` is the only root there is.
* The brief's exposed value is `'kept'`, five bytes, which rides inside the handle and is never a
  heap object -- so with it the row stayed green under **every** mutation tried, including
  `Body::trace` losing the pools entirely. The committed program exposes a 65-byte value.

Both are the shape `collect_stress.rs`'s own `a_parked_reply` doc already warns about, found again.

## Controls, each recorded as run

Every control below was re-run against the code as committed, after the naming rework, and each
gate-table reading is from

```
REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec \
    --test gate_table_c --no-fail-fast
```

**The unmutated reading**, from that command with `--test gate_table_d` beside it: exit 101,
`5b: 6 rows, 4 not yet agree` for table C and `5b: 2 rows, 0 not yet agree` for table D, with
`abscla` and `creo` both `agree`. The four still red are `objcla`, `usesem`, `obdes` and
`methodsbyclass`, which are Tasks 2, 3, 5 and 8.

**Control 1 -- drop `checkAbstract` from the `~new` path.** Exit 101, `5b: 6 rows, 5 not yet agree`
against the unmutated run's 4:

```
  diverge-both   loud=no  5b   abscla Abstract Classes
      oracle rc=158  out="declared AB Class\n" err="       *-* Compiled method \"NEW\" with scope \"Object\".\n..."
      crate  rc=0    out="declared AB Class\ninstance an AB\n" err=""
  agree          loud=no  5b   creo Initialization
```

The abstract class constructs; `creo` is untouched. Restored from a scratchpad copy.

**Control 2 -- do not send `INIT` after the object is built.** Exit 101, `5b: 6 rows, 5 not yet
agree`:

```
  diverge-stdout loud=no  5b   creo Initialization
      oracle rc=0    out="type a savings account\nbalance 1000.00\nrate 6.25\n" err=""
      crate  rc=0    out="type a savings account\nbalance BALANCE\nrate INTEREST_RATE\n" err=""
  agree          loud=no  5b   abscla Abstract Classes
```

`creo` reddens **silently** -- rc 0, empty stderr, two uninitialised renderings -- and `abscla` is
untouched. Restored from a scratchpad copy.

**Control 3 -- `Body::trace`'s `Instance` arm stops tracing the pools.**
`cargo test --release -p rexx-exec --test collect_stress`: 5 passed, 3 failed. The new
`a_method_that_assigns_over_self_keeps_its_exposed_variables` fails on `a live value`, and so do
`a_parked_reply_keeps_its_variables_across_a_collection` and
`the_l0_subset_passes_again_under_collect_on_every_allocation`.

**Control 4 -- `native_new`'s `push_temp` removed.** Same command: 6 passed, 2 failed. The new row
and the subset row fail on `an exposed variable's owner is a rooted Body::Instance`. This is the
mutation the `INIT`-side `SELF` clobber in the witness program exists for; without it, control 4 is
green.

**What control 3 does not add**, per this project's own rule that "can fail" is not "adds coverage":
two existing rows catch it too. The new row's own contribution is control 4, which nothing else
catches except the subset run over the same committed program.

## Probing past the rows

`~new` on `Object` is a class-side row inherited by every class whose own class behaviour declares
none, so landing it changes the answer for far more than two probes. Every class name
`environment.rs`'s `ORACLE_ENVIRONMENT` holds was run as `say 'r' .NAME~new~string`, three
descriptors, oracle against `REXX_ENGINE=ir`.

**Newly agreeing at rc 0** (they refused `method "NEW" of class "Object"` before):
`AlarmNotification`, `ArgUtil`, `CaselessComparator`, `CaselessDescendingComparator`, `Collection`,
`Comparable`, `Comparator`, `DescendingComparator`, `InputOutputStream`, `InputStream`,
`MapCollection`, `MessageNotification`, `NumericComparator`, `Object`, `Orderable`,
`OrderedCollection`, `OutputStream`, `RexxQueue`, `SetCollection` and `Validate`, plus `.environment`
and `.local`, whose `~new` goes through `Directory`'s `UNKNOWN` and answers `The NIL object` on both
sides. `.DateTime~new` answers a timestamp and so can never be a differential row.

**Still refusing, unchanged**: every class carrying its own class-side `NEW` in `Setup.cpp` resolves
that row and stays `method "NEW" of class "<its own>" is not implemented (Phase 5)`. Those are 5c's.

**Eight classes moved from a loud refusal to a wrong error number, and the cause is a 5a defect this
task did not introduce.** `USE STRICT ARG` inside a `::METHOD` body reports the *routine* error.
Measured at HEAD with no instance anywhere -- `::METHOD m CLASS` carrying `use strict arg v`, sent as
`.K~m` -- the oracle is rc 163, `Error 93 running ...:  Incorrect call to method.` /
`Error 93.901:  Not enough arguments for method; 1 expected.`, and this crate is rc 216,
`Error 40 running ...:  Incorrect call to routine.` /
`Error 40.3:  Not enough arguments in invocation of M; minimum expected is 1.`, with the traceback
frames above the error agreeing. `~new` now reaches a prologue class's Rexx `::METHOD init` for
`Alarm`, `CaselessColumnComparator`, `ColumnComparator`, `File`, `InvertingComparator`, `Stream`,
`Ticker` and `TimeSpan`, and each reports that wrong pair. **Written into the plan's Task 9**, which
owns the audit of 5a's limits. No gate reddens: those rows are 5c's method rows, reported and not
gated, and no corpus program constructs one. The too-many-arguments direction was not measured.

**Operators.** `Interp::operator_operand_gap` refuses an instance the way it already refuses a class
object. That is a divergence: measured, oracle rc 159, `o + 1` is
`97.1 Object "a K" does not understand message "+".` even after `o~objectName = '123'` -- the
rendering being numeric does not make it a conversion -- and this crate refuses loudly with
`the operator '+' applied to an instance of a user class is not implemented (Phase 5)`. It is the
same divergence the class-object arm has carried since 5a (`.K + 1` is 97.1 on the oracle and loud
here), and operators as message sends are not this task's. Nothing in the corpus can carry it, so
the instrument is the loud message and this paragraph.

## The row this task made silent, which is Task 2's

`objcla` was `diverge-both`, loud, rc 120. It is now **`diverge-stdout`, rc 0 on both sides, empty
stderr on both sides**, `after 1` against the oracle's `after 0` -- the exact silent wrong answer D58
predicts for an implementation that walks the class graph live on every send. `Body::Instance`
carries its class and `receiver_behaviour` resolves against that class's instance behaviour as it
stands, which is a live walk.

**That is the shape Task 2 inherits and has to change**, and it is written into the plan's Task 2.
The pieces it needs exist: `rexx_classes::ClassGraph` already models the copy on `~define`,
`~defineMethods` and `~delete` and the in-place rebuild on `~inherit`/`~uninherit`, and exposes
`instance_behaviour_handle` with a `lookup_at`/`has_method_at`/`resolve_super_scope_at` family over a
`BehaviourHandle`. What blocks putting that handle in the body is the crate layering: `rexx-core`'s
`Body` cannot name a `rexx-classes` type, so the handle would have to move down a crate. That is
bigger than this task's seam and it is Task 2's subject, so it was not taken here.

## What this task leaves open

* **`Interp::to_text` does not send.** The oracle's `stringValue()` is an `OBJECTNAME` send in every
  context; the three paths a program reaches -- `Object~string`, `Object~objectName` and the
  required-string protocol -- send, and `to_text` derives the article-and-id rendering for an unnamed
  instance instead. What is left on the wrong side of that line is every rendering reached through an
  infallible function: `TRACE`'s value lines, measured and written up below, and this crate's own
  error-message substitutions, for which no probe was constructed.
* **`~define('STRING', ...)` and `~defineMethods` do not arm `Interp::reqstr_armed` by name.** They do
  not need to, because `native_new` arms it outright; the hole they leave for `MAKESTRING` predates
  this task and is unchanged.
* **The `USE STRICT ARG` error family**, above, in the plan's Task 9.

## The corpus witnesses this task commits

`corpus/phase-5b.txt` gains six entries and `EXPECTED_SUBSET_5B` the same six in the same order.

| entry | what it pins |
|---|---|
| `gate-tables/concepts/creo.rex` | the row `~new` and `INIT` make agree |
| `gate-tables/concepts/abscla.rex` | the row `checkAbstract` makes agree |
| `lang/instance_naming.rex` | `~string`, `~defaultName`, `~objectName`, `~objectName=`, `~class` and `~isA` on an instance, plus `Class~defaultName`, plus the article rule both ways round |
| `lang/instance_naming_overrides.rex` | the same names against a class overriding `defaultName`, `string` or `makeString`, which is what makes them sends |
| `lang/instance_naming_raises.rex` | the stderr arm: a raising `defaultName` reports its own clause under `Compiled method "OBJECTNAME" with scope "Object".` and `Compiled method "STRING" with scope "Object".`, two frames no stored rendering could produce. Oracle rc 214, byte for byte on both engines |
| `lang/instance_self_reassigned.rex` | the `SELF` clobber, twice, with a heap-wide exposed value |

A raising `::METHOD string` takes a **different** shape and is measured but not committed: oracle
rc 214, the method's own clause then the sending clause and nothing between them, so the
required-string protocol's `STRING` send contributes no `REQUEST` frame where its conversion limbs
do. That is why `required_string_dispatch`'s new arm propagates the failure instead of calling
`blame_request`, and both shapes agree on all three descriptors on both engines.

## A second silent wrong answer, measured, left open and written into the plan

`TRACE`'s value lines render through `stringValue()`, which is an `OBJECTNAME` send, so an override
moves them too. Measured, `trace i` over `o = .K~new` with `::METHOD defaultName` returning
`overridden`, rc 0 on both sides and the same (empty) stdout:

```
oracle  >M>   "NEW" => "overridden"     >>>   "overridden"     >=>   O <= "overridden"
crate   >M>   "NEW" => "a K"            >>>   "a K"            >=>   O <= "a K"
```

**It is left open and written into the plan's Task 9.** The three message paths send;
`Interp::string_value_text` is infallible and `Interp::intermediate_text`/`Interp::result_text`
answer `Option<Vec<u8>>`, so closing this means making the trace rendering path fallible from the
emit sites down -- a change to the trace subsystem rather than to the instance seam, and Task 9 is
the audit that owns "every limit 5a pinned with a class as the receiver". The comment on
`Interp::to_text`'s `Redirect::InstanceDefault` arm names it at the site.

## The gates

Each command run from `rust/`, each status read unpiped, at the tree as committed.

| command | status |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --release --workspace` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 0 |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 |
| `REXX_PHASE_GATE=5b REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test gate_table_c --test gate_table_d --no-fail-fast` | 101 |

The phase gate's 101 is the four 5b rows Tasks 2, 3, 5 and 8 own; `abscla` and `creo` read `agree`
in that run's own report. The corpus figure is **270 of 270 matching**, printed by both
`REXX_CORPUS_GATE=1` runs -- the plan's base records 264 of 264, Task 0 added none, and this task
adds six.

`REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` had to be launched under
`setsid` to finish: run as an ordinary background command it was reaped twice at about the
twenty-minute mark, once leaving no `G5` line at all. The figure above is from the run that
completed.

## The performance sitting: not taken, and why

This task lands code in `src/`, so the guard applies. **It could not be satisfied: the machine
exposes one usable hardware performance counter, and the guard's instrument needs two at once.**

**Which command the guard is.** The 5b plan states no per-task sitting command; its only concrete
statement is Task 10's, which names eight axes against `bench-baselines/pinned/rexx-run-15a1ffa98`,
interleaved. `docs/superpowers/records/2026-08-17-phase-5a/global-constraints.md` gives the command
with those same eight axes and the same pin, and adds `--baseline
bench-baselines/phase-5a-arms.tsv`. The two agree on pin and axes, and the 5b plan is silent on the
baseline file; `bench-baselines/PINNED.md`'s rule is one baseline file per pin and the pin is
unchanged, so `phase-5a-arms.tsv` is the file. That is the command that was run.

**The pin is not stale.** `sha256sum bench-baselines/pinned/rexx-run-15a1ffa98` is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching `PINNED.md`.
`git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml` lists
the 5a plan's own body, the merge and documentation commits the 5b plan's Base names, and 5b Task 0
-- no commit outside those ledgers.

**What happened, twice, from a clean launch:**

```
rexx-arms: alloc4c: warming 8 cells
rexx-arms: `perf stat` produced no cycles:u and instructions:u pair for pinned tw small
```

**Framed with the probe before and after**, which is what separates a transient shortage from a
machine property. `perf stat` itself works and the run completes; what fails is the pair:

```
$ perf stat -x, -e instructions:u -- ./target/release/rexx-run bench-programs/emptyloop.rex
9276582154,,instructions:u,521945739,100.00,,

$ perf stat -x, -e cycles:u,instructions:u -- ./target/release/rexx-run bench-programs/emptyloop.rex
1557676545,,cycles:u,273191466,50.00,,
9362203490,,instructions:u,263653929,49.00,,
```

One event alone is scheduled for 100.00% of the run; the pair multiplexes to about half each, three
further repetitions reading 50.00/50.00, 49.00/50.00 and 50.00/49.00. `rexx_bench::parse_counters`
refuses anything below 99.995% **on purpose**, and its own doc says why: below 100 `perf` reports the
count scaled up to what it would have been, and on a quantity otherwise deterministic to eight
significant figures an estimate is indistinguishable from a real movement of a few per cent. So the
harness is behaving correctly and the reading is unavailable, not wrong.

`/proc/sys/kernel/nmi_watchdog` is `1`, `/sys/bus/event_source/devices/` is empty and
`perf_event_paranoid` is `-1`; nothing else on the machine was using the CPU (`ps` showed no
benchmark, no `cargo`, load from this session alone). `rexx-arms` has no option that drops the
counters, by design -- it is the instruction-count instrument.

**Nothing was written to `bench-baselines/phase-5a-arms.tsv`**, which is unchanged at the row count
it had before this task, and no figure is claimed. **The guard is unmet for this commit and the
controller should decide whether to re-take the sitting when the machine exposes both counters
again.** `dispatch.rex` also goes live with this task and owes a first baseline row rather than a
comparison, which the plan's Task 10 already says; that row is unmeasured for the same reason.
