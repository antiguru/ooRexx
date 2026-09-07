# Phase 5i Task 2 -- Pointer, Buffer, WeakReference

BASE `9308ca9bd`, confirmed HEAD, tree clean at start.

## Pre-flight: what the brief got wrong

Items 1 to 4 and 6 were sent to the controller before any code was written; item 5 was found while
implementing item 3 and sent then. None was answered before this commit.

### 1. The prescribed `WeakReference` design regresses a case that is green today

The brief says "`new` has to build a `Body::WeakRef` instead of a plain instance". `Body::WeakRef(ObjRef)`
carries no class, no behaviour, no `ScopePools` and no name, so an instance built that way loses
everything a **subclass** of `WeakReference` needs. That case works on both sides today.

Measured, oracle and both engines byte-identical on stdout for the first three lines
(the crate then stops at `~value`, which is this task's row; the oracle continues):

```rexx
o = .Object~new
w = .W~new(o, 'tag')
say w~class~id     -- W
say w~string       -- a W
say w~label        -- tag
::class W subclass WeakReference
::attribute label
::method init
  expose label
  use arg label
```

Widening `Body::WeakRef` to carry the whole instance payload duplicates `Body::Instance`, and every
reader of those fields is in `eval.rs`, `value.rs`, `run.rs` and `lib.rs`. Adding a field to
`Body::Instance` breaks every construction site of it -- `/bin/grep -rn 'Body::Instance {'` over
`crates/` names `value.rs`, `lib.rs`, `run.rs`, `eval.rs`, `environment.rs`, `stem.rs`,
`dispatch.rs`, `dispatch/hash.rs`, `dispatch/collection.rs`, `body.rs` and three `rexx-core` test
files. Both routes reach well outside this task's file list.

### 2. Dispatch and global constraints contradicted each other on the shared tables

The dispatch said "refresh and commit them"; `global-constraints.md` said the controller owned them
and no task committed them. The controller revoked the latter on being asked (its own
`## REVOKED, 2026-09-07` section names this pre-flight). The tables are committed here.

### 3. `crates/rexx-exec/src/error.rs` is needed and was not on the file list

`dispatch.rs` calls `Raised::syntax(` zero times: every `93.9xx` in this crate goes through a named
constructor in `error.rs` carrying the C++ citation, and 93.967 has none.

### 4. `crates/rexx-exec/tests/collect_stress.rs` is needed and was not on the file list

The brief names `collect_stress` as the harness for the dead-referent witness. The in-list
alternative was `dispatch.rs`'s own `#[cfg(test)]` module, since `run_program_collect_every_alloc`
is `pub` in `rexx-exec/src/lib.rs`; `collect_stress.rs` is where every rooting and collection witness
lives and where the `stress.collections > 0` anti-vacuity idiom is, so the witness went there.

### 5. A fifth shared artifact this task has to move, which nothing named

`rust/corpus/refusal-sites.tsv`. `crates/rexx-exec/tests/refusal_sites.rs` re-derives its first four
columns from `src/` on every run and fails if the committed file disagrees. Adding
`Raised::unsupported_new_method` to `error.rs` adds a row **and** shifts the `definition` line number
of 56 later rows. **There is no version of this change that leaves the file alone**: the fallback of
calling `Raised::syntax(93, 967, ...)` inline from `dispatch.rs` is not an escape either, because
`Raised::syntax` is itself a row, currently `body` / `off-send-surface`, and a call from `dispatch.rs`
flips its surface to `body+send` and demands a verdict. So the named constructor in `error.rs` is the
right shape, and it and the table are both committed here.

### 6. Where Pointer's five instance rows actually live

The brief calls them rows; they are `corpus/method-bodies.txt` rows.
`corpus/introspection-arity.tsv` deliberately carries **no** Pointer or Buffer instance arm at all
(its own header says `class-set.txt` gives them no construction expression), so the only rows of
theirs in that table are the two `new` ones the dispatch names.

## (a) `Pointer~new` and `Buffer~new` raise 93.967

### The C++ site, and who shares it

`reportException(Error_Unsupported_new_method, ((RexxClass *)this)->getId())`, one call per file:

| file | line |
| --- | --- |
| `interpreter/classes/PointerClass.cpp` | `:142` |
| `interpreter/classes/BufferClass.cpp` | `:89` |
| `interpreter/classes/StackFrameClass.cpp` | `:127` |
| `interpreter/classes/ContextClass.cpp` | `:99` |
| `interpreter/classes/VariableReference.cpp` | `:100` |
| `interpreter/classes/RexxInfoClass.cpp` | `:92` |

`PointerClass::newRexx` and `BufferClass::newRexx` are the raise and nothing else -- no argument is
read -- and the substitution is the **receiver** class's `getId()`, not the scope the method is
compiled in.

**The sub-number is 967, checked two ways that do not share a source.** Measured on the oracle,
rc 163; and `rexx-inventory`'s generated catalogue, derived from `interpreter/messages/rexxmsg.xml`
at build time, carries
`Message { major: 93, sub: 967, number: 691, symbol: "Error_Unsupported_new_method", text: "NEW method is not supported for the &1 class." }`.
The survey's `93.968` was the next `<SubMessage>` block.

### `.StackFrame~new`, and why this task does not bind it

Measured on the oracle, rc 163, `Error 93.967: NEW method is not supported for the StackFrame class.`
with `*-* Compiled method "NEW" with scope "StackFrame".` above it -- so it does share the site, and
one line of the same helper would bind it.

**It is deliberately left loud, and Task 6 needs to know why.** All ten `StackFrame` rows in
`corpus/method-bodies.txt` are blocked on `method "NEW" of class "StackFrame"`, so binding `NEW`
would flip all ten to `answers` with nothing behind them -- the existence-shaped evidence
`rust/CLAUDE.md` names. The ten `StackFrame` rows in `corpus/introspection-arity.tsv` are a
**different** blockage and `new` closes none of them: they are `setup-differs` on
`RexxContext~stackFrames`, because `corpus/introspection-receivers.tsv` builds that receiver as
`.context~stackFrames[1]`.

`.RexxContext~new` and `.VariableReference~new` also share the site and also diverge today (oracle
rc 163 / 93.967, crate rc 120). `.RexxInfo~new` does **not**: `.RexxInfo` is an instance, so it is
`97.1 Object "a RexxInfo" does not understand message "NEW"` at rc 159, which this crate already
matches byte for byte.

### Pointer's five instance rows close by cascade, and no body is written for them

`=`, `==`, `\=`, `\==` and `isNull` in `corpus/method-bodies.txt` have `.Pointer~new` as their
receiver expression. With `new` raising identically on both sides, the send is never made at all --
and the refreshed table says so in its own verdict column rather than recording `answers`, which is
better than the brief predicted; see the shared-artifact section.

**Leaving them without a body is honest here rather than hollow**: the oracle has no path to a
`Pointer` either -- the
reference says instances come only from native code (`utilityclasses.xml:6910`, `:429` for `Buffer`)
-- so there is no reachable receiver on either side for a body to serve. `rust/CLAUDE.md` forbids
building a body nothing can reach. Phase 8 owns making a `Pointer` that holds something.
**This sentence belongs in the phase's handover.**

### The anti-constant control

The witness's last send is untrapped and its receiver is `::class P subclass Pointer`. The oracle
prints the trace line `*-* Compiled method "NEW" with scope "Pointer".` and the message
`NEW method is not supported for the P class.` -- two different words in one transcript, the scope
the method is compiled in and the receiver class's own `getId()`. An implementation answering the
literal `"Pointer"` prints the same word twice and goes red.

`condition('O')` and `condition('A')` would have put the substitution on stdout for every row, and
both are Phase-5 loud refusals in this crate, so the untrapped send's stderr is the only route.

## (b) `WeakReference~value`

### The state and the collector already existed and nothing reached them

`Body::WeakRef(ObjRef)` is declared (`rexx-core/src/body.rs:235`) with a trace arm that walks
nothing (`:741`), and `rexx-core/src/heap.rs:155`-`:206` implements the whole weak protocol --
`checkWeakReferences` before `checkUninit`, clearing a dead referent to `Body::WeakRef(ObjRef::NIL)`.
Nothing in either crate constructed one: `native_weak_reference_new` built a plain instance through
`new_instance` and dropped the referent, and its own doc comment said so.

### The design, and why it is not the one the brief prescribes

The reference object stays an ordinary `Body::Instance`. `WeakReference~new` additionally allocates
a **cell** whose body is `Body::WeakRef(referent)` and binds it in the instance's scope pool for the
`WeakReference` scope under `REFERENT`, which is the position `COLLECTION_STORES`' `ITEMS`,
`HASHINDEXES` and the rest are in. `Body::Instance`'s trace arm walks the pools, so the cell lives
exactly as long as the reference; the cell's own body traces nothing, so `heap.rs`'s
`checkWeakReferences` pass rewrites it to `Body::WeakRef(ObjRef::NIL)` as soon as the referent is
unreachable. `value` reads the cell and answers what it holds -- a cleared cell holds `ObjRef::NIL`,
which decodes as `.nil`, so the live and the cleared answer are one read.

**What this buys over the brief's shape.** The instance keeps its class, its behaviour, its
`ScopePools` and its name, so a `WeakReference` subclass keeps working -- see pre-flight item 1.
`heap.rs` and `body.rs` are not touched at all: the protocol that was already written runs unchanged
and becomes live for the first time. `receiver_kind`'s `Body::WeakRef(_) => Err("a weak reference")`
arm stays where it is and stays unreachable from a program, because the cell is never handed out;
the brief's third bullet asked for it to be removed and it is not needed. The doc comment the brief
correctly flagged as false is rewritten.

The scope the referent is bound under is `WeakReference` itself and not the receiver's class, so a
subclass's referent is where `value` looks for it. `ObjectModel` gains a `weak_reference` handle for
that, resolved at bootstrap beside `variable_reference` and `stem`.

### The dead-referent witness, and the three controls

`collect_stress.rs`'s `a_weak_reference_clears_only_when_its_referent_becomes_unreachable`, both
engines, under `run_program_collect_every_alloc`. One program drops one referent, holds another, and
reads both references after the same collections:

```rexx
dropped = .Object~new
wd = .WeakReference~new(dropped)
held = .Object~new
wh = .WeakReference~new(held)
drop dropped
do i = 1 to 20
  zj = 'filler' i
end
say 'dropped' (wd~value == .nil)
say 'held' (wh~value == held) wh~value~class~id
```

Expected `dropped 1\nheld 1 Object\n`, with the harness's own `stress.collections > 0`
anti-vacuity assertion. **Every prediction below was written before the run.**

| control | predicted | measured | verdict |
| --- | --- | --- | --- |
| C1 -- `dispatch.rs` restored to BASE, `error.rs` kept | fails at `exit_code`, 120, the Phase-5 refusal at the first `~value` | `assertion left == right failed: TreeWalker / left: 120 / right: 0` | confirmed |
| C2 -- the degenerate implementation: bind the referent in the pool directly, no cell, so weakness is ignored | fails on the first line only, `dropped 0\nheld 1 Object\n` | exactly that | confirmed |
| C3 -- delete `heap.rs`'s Pass 1 weak-clearing loop | fails the same way, `dropped 0\nheld 1 Object\n` | exactly that | confirmed |

A third line was added to the witness after those controls and its answer predicted before it ran:
`say 'temp' (.WeakReference~new(.Object~new)~value == .nil)` -- predicted `0`, because the referent
is a temp of the clause that is still running, so it is rooted across the cell's own allocation.
Measured `0` on both engines. That is the rooting half of the construction path, which C1 to C3 do
not reach: their referents are held in variables.

### Does the new test add coverage, or only "can fail"?

Measured, not argued. The whole workspace was run under the C2 mutation --
`REXX_CORPUS_GATE=1 memcap 8G cargo test --profile mutation --workspace --no-fail-fast`, exit 101 --
and six test names failed:

* `a_weak_reference_clears_only_when_its_referent_becomes_unreachable`, the new one;
* `dispatch::tests::a_constructor_taking_arguments_answers_an_instance_and_refuses_its_state`, which
  fails identically under the **correct** implementation because it asserted that `~value` refuses,
  so its failure does not distinguish C2 from the fix -- it is updated in this commit;
* `every_lang_program_is_run_or_named_unfiled`, `no_row_started_diverging_or_stopped_answering`,
  `the_table_matches_the_three_sides` and
  `sourceline_matches_the_interpreter_for_every_corpus_program`, all four red because the corpus
  registration, the two tables and the two `sourceline_oracle` fixtures had not yet been done at
  that point. None of them is sensitive to C2: the tables record `agree` for a live-reference probe
  either way, and the other two read file lists rather than behaviour.

So the new row is the only thing in the workspace whose result separates the weak implementation
from the strong one.

**A note on the run itself.** The first attempt was
`memcap 8G cargo test --profile mutation --workspace --no-fail-fast` as a single command, and the
cap killed the *build*: `memcap: OOM-killed at the 8G cap (peak 8.0G)`, exit 137, zero test results.
That is the same trap the dispatch warns about for the release fast check, one command over. The
build has to be done first and uncapped -- `--no-run` under a wider cap -- and only the run belongs
under `memcap 8G`.

**C2 is the one the row exists for, and its second half is the measurement that matters.** Under the
same C2 build, `corpus/lang/weak_reference_value.rex` is byte-identical to the oracle on all three
descriptors and both engines. So the differential witness -- this project's main instrument -- cannot
tell this crate's weak reference from a strong one, and only the collect-stress row can. That is the
degenerate implementation the brief named, and it is the one this crate would otherwise have shipped.

**C3 with an honest qualifier.** It shows the weak pass is reachable from a Rexx program for the
first time. It does not show the pass was previously untested: `rexx-core/tests/uninit.rs` builds a
`Body::WeakRef` by hand and 2 of its 7 tests also go red under C3.

Restored from the scratch copy after each control (never `git checkout --`), and `heap.rs` confirmed
back to a clean `git diff --stat`.

### What each reader reads, and where that state comes from

The phase's "a reader that answers from a constant is a defect" rule, one sentence per reader.

* **`WeakReference~value`** reads the `Body::WeakRef` cell bound at `REFERENT` in the receiver's
  `WeakReference` scope pool -- written by `new` from its own first argument and rewritten to
  `ObjRef::NIL` by `Heap::collect`'s weak pass. No constant is involved, and C2 and C3 above each
  move one of those two sources and turn the row red.
* **`Pointer~new` and `Buffer~new`** substitute `interp.class_id_text(class)`, the receiver class's
  own id, mirroring `((RexxClass *)this)->getId()`. The `.P~new` send in the witness is what makes
  that a measurement rather than a claim: a literal `"Pointer"` would print the same word as the
  scope line and go red.

The second-send rule is met by `weak_reference_value.rex`, which asks the answered object
`~class~id`, `~hasMethod('CLASS')`, `~isA(.Object)` and `== o` rather than only rendering it.

## Shared-artifact rows: before and after

Refresh commands, run from `rust/`:

```
REXX_METHOD_BODIES_REFRESH=1 cargo test --release -p rexx-exec --test method_bodies
REXX_INTROSPECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec --test introspection_arity
cargo test --release -p rexx-exec --test refusal_sites
cargo test --release -p rexx-exec --test coverage
```

`corpus/phase-5c.txt`, `EXPECTED_SUBSET_5C` and `corpus/refusal-sites.tsv` have no refresh
mechanism; they are edited and the tests above hold them equal to the tree.

### `corpus/method-bodies.txt`

| class | method | arm | before | after |
| --- | --- | --- | --- | --- |
| Buffer | new | class | `loud method "NEW" of class "Buffer"` | `answers rc 163` |
| Pointer | new | class | `loud method "NEW" of class "Pointer"` | `answers rc 163` |
| Pointer | = | instance | `loud method "NEW" of class "Pointer"` | `unanswered a bare ~new raises; the method was never sent` |
| Pointer | == | instance | `loud method "NEW" of class "Pointer"` | `unanswered a bare ~new raises; the method was never sent` |
| Pointer | \\= | instance | `loud method "NEW" of class "Pointer"` | `unanswered a bare ~new raises; the method was never sent` |
| Pointer | \\== | instance | `loud method "NEW" of class "Pointer"` | `unanswered a bare ~new raises; the method was never sent` |
| Pointer | isNull | instance | `loud method "NEW" of class "Pointer"` | `unanswered a bare ~new raises; the method was never sent` |
| WeakReference | value | instance | `loud method "VALUE" of class "WeakReference"` | `answers rc 0` |
| WeakReference | new | class | `answers rc 163` | `answers rc 163` (unmoved) |

**The instrument gave Pointer's five instance rows a better verdict than the brief predicted.** The
brief expected them to record agreement about the construction; the refresh classifies them
`unanswered` with the reason `a bare ~new raises; the method was never sent`. So `corpus/method-bodies.txt`
already says in its own column what pre-flight item 5 above says in prose, and no reader of the
table can mistake those five for methods that work. `StackFrame`'s ten rows stay `loud` on
`method "NEW" of class "StackFrame"`, unchanged.

### `corpus/introspection-arity.tsv`

| class | method | arm | before | after |
| --- | --- | --- | --- | --- |
| Buffer | new | class | `send-differs` -- oracle rc0 SYNTAX 93.967; crate rc120 not implemented | `agree rc0` |
| Pointer | new | class | `send-differs` -- oracle rc0 SYNTAX 93.967; crate rc120 not implemented | `agree rc0` |
| WeakReference | value | instance | `send-differs` -- oracle rc0 VALUE an Object; crate rc120 not implemented | `agree rc0` |
| WeakReference | new | class | `agree rc0` | `agree rc0` (unmoved) |

The two `REFUSED:` rows moved to `agree`, which is what the dispatch asked for, and
`every_refused_row_is_really_refused` passed on the refreshed table.

### `corpus/refusal-sites.tsv`

One row added and 56 later `definition` line numbers shifted by the 17 lines `error.rs` gained:

```
Raised	unsupported_new_method	send	crates/rexx-exec/src/error.rs:1722	agrees	yes	93.967	.Pointer~new
```

### `corpus/phase-5c.txt` and `EXPECTED_SUBSET_5C`

`lang/pointer_buffer_new_refused.rex` and `lang/weak_reference_value.rex` added to both.

## What changed

| file | change |
| --- | --- |
| `crates/rexx-exec/src/error.rs` | `Raised::unsupported_new_method(id)` -- 93.967 |
| `crates/rexx-exec/src/dispatch.rs` | `native_unsupported_new` bound to `Pointer~NEW` and `Buffer~NEW`; `ObjectModel::weak_reference`; `WEAK_REFERENT`, `weak_referent_cell`, `native_weak_reference_value` bound to `WeakReference~VALUE`; `native_weak_reference_new` rewritten and its false doc corrected; two of its own tests updated |
| `crates/rexx-exec/tests/collect_stress.rs` | the dead-referent witness |
| `corpus/lang/pointer_buffer_new_refused.rex`, `corpus/lang/weak_reference_value.rex` | the two differential witnesses, with their `sourceline_oracle` fixtures |
| `corpus/method-bodies.txt`, `corpus/introspection-arity.tsv`, `corpus/refusal-sites.tsv`, `corpus/phase-5c.txt`, `crates/rexx-exec/tests/coverage.rs` | the shared artifacts |

`crates/rexx-core/src/body.rs` and `crates/rexx-core/src/heap.rs` are **not** touched -- see the
design section for why that is the point rather than an omission.

## Fast checks

| check | command | result |
| --- | --- | --- |
| fmt | `cargo fmt --all --check` from `rust/` | exit 0 |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| release suite | `cargo test --release --workspace --no-fail-fast` | exit 0, 2279 passed, 0 failed across 116 binaries |

## Gates

Run in the pinned gate worktree `/home/moritz/dev/repos/ooRexx-5i-gates`, detached at this task's
commit, so the gated tree is the committed tree by construction. Read from
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/t2/gates/status.txt`,
whose first line is `3d2c7dd75a6ec20a49509bfc5d44072d2256f525` and whose last is
`finished 2026-09-08T01:50:46+02:00`.

| gate | command | result |
| --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| G3 | `cargo test --release --workspace --no-fail-fast` | exit 0, 116 `test result: ok`, no `test result: FAILED`, 2279 passed |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0, 116 `test result: ok`, no `test result: FAILED`, 2280 passed |
| G5 | `cargo test --release -p rexx-exec --test collection_arity` | exit 0, 23 passed |
| G6 | `cargo test --release -p rexx-exec --test introspection_arity` | exit 0, 25 passed |
| G7 | `cargo test --release -p rexx-exec --test introspection_scopes` | exit 0, 22 passed |

All seven zero. **The STRICT corpus differential inside G4 reports `445 of 445 matching`** under
`mode: STRICT (the gate) -- REXX_CORPUS_GATE is set`, which is the run that includes
`lang/pointer_buffer_new_refused.rex` and `lang/weak_reference_value.rex`; the plain `--test corpus`
binary is report mode and exits 0 on a divergence, so that is the line to cite. Task 1's gate at
`9308ca9bd` reported 443 of 443, so the two new witnesses are the whole of the increase. G5's 23 and
G7's 22 are unmoved from Task 1's; G6's 25 is unmoved too, and its three moved rows are inside that
count rather than beside it.

G3 carries one test fewer than G4, and the difference was diffed rather than assumed: the extra one
is `bytes::tests::the_bytes_past_len_are_never_part_of_the_value`, which carries
`#[cfg(debug_assertions)]`.

## Concerns

1. **`.StackFrame~new`, `.RexxContext~new` and `.VariableReference~new` still diverge** -- oracle rc
   163 / 93.967 against this crate's rc 120 Phase-5 refusal. Each is one line of the helper this
   task added. `StackFrame` is left alone deliberately (Task 6's, and binding it would flip ten
   `method-bodies.txt` rows to a hollow verdict); the other two are nobody's row and were left
   pending an answer from the controller.
2. **`Pointer` and `Buffer` remain classes with no reachable instance on either side**, which is why
   this task adds no instance body for them. That belongs in the phase's handover.
3. **The weak referent is instance state kept in a scope pool**, which is `COLLECTION_STORES`'
   established idiom in this crate but does mean a method defined into the `WeakReference` scope by
   `Class~define` could `expose referent` and see the cell -- the same exposure `Array`'s `ITEMS`
   has. It is not reachable by any ordinary program.
4. **Four questions to the controller went unanswered** while this ran: the file-list additions
   (`error.rs`, `collect_stress.rs`, `refusal-sites.tsv`) and the `RexxContext`/`VariableReference`
   question. The first three were forced by the change with no alternative and were taken; the
   fourth was left undone.
