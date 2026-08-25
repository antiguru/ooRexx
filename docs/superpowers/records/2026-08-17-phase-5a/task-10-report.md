# Task 10: the send path -- `resolve`'s inputs, and D24's three surviving constraints

Base `5fd2002cf`; branch `plan/rust-rewrite`, worktree `/home/moritz/dev/repos/ooRexx-rust-rewrite`.
Commits: **`64c78d0c6`** the roadmap amendment, **`b55a721d1`** the code, **`207756aae`** the sitting
rows. The report itself stays untracked: `.gitignore:30` excludes `.superpowers/`, and this
directory's earlier reports have no commit either.

Four things landed. `Interp::resolve` and `Interp::send_message` gained the sending side as one
`Caller` value. `Primitive::SmallInt` is a behaviour arm of its own. The receiver is a field of
`CallContext`, and `SELF` is bound out of it. Message selectors are interned by the parse that reads
them, and `ExprKind::Message` holds one rather than its own copy of the bytes. The roadmap's
patchable-slot sentence is **amended**, on the evidence below.

Nothing here changes an answer. That is the point of the task and it is also the weakness of its
differential, which the brief states and this report does not talk around: **the corpus, `ir_dual`
and both gate tables would read exactly the same against a commit that did nothing.** The
instruments that can fail are the three in-crate tests and the sitting, and each is shown below
failing under a mutation that drops its constraint.

## What "the caller's scope" turned out to be

The brief and D27's amendment both say `resolve` gains **the caller's scope** and the caller's
package. Printed, the C++ says receiver:

* `RexxObject::checkPrivate` (`classes/ObjectClass.cpp:609`) reads
  `RexxObject *sender = activation->getReceiver();` at `:616`, allows the send when
  `sender == this` (`:617`-`:620`), and refuses outright when `sender == OREF_NULL` (`:622`-`:626`).
* The only scope it reads is the **resolved method's own** -- `RexxClass *scope = method->getScope();`
  at `:628` -- which `Resolution` already carries, so a parameter named for the caller's scope would
  have had nothing to put in it.
* `RexxObject::checkPackage` (`:659`) refuses when there is no calling activation (`:665`-`:669`)
  and otherwise reads `PackageClass *callerPackage = activation->getPackage();` at `:671`.

So `Caller` carries the caller's **receiver** and the caller's **package**. Ruled by the controller
before this report was written and recorded at the plan (`fc31fb82a`).

## The four pieces, at their sites

### `resolve`'s signature

`pub(crate) fn resolve(&mut self, receiver: ObjRef, name: &[u8], start_scope: Option<ObjRef>,
caller: Caller) -> Option<Resolution>`, and `send_message` takes and forwards the same value.
`Interp::caller()` builds one from the running state: the receiver out of `CallContext`, the package
out of `Interp::running_program()`.

**Nothing reads either half yet**, because no access scope is implemented in this crate --
`rexx-classes/src/registry.rs:414`-`:416` records that `Setup.cpp`'s
`AddMethod`/`AddProtectedMethod`/`AddPrivateMethod`/`AddUnguardedMethod` distinctions are not
modelled. `Interp::caller` is the only production route to a `Caller` (`dispatch.rs:476`; the other
construction is the test module's `no_caller`, `:1656`).

**What keeps the two accessors alive is an `allow`, stated as such** -- fix round 1 replaced a
`debug_assert` here that no in-tree path could falsify. See that round's section below.

### The `SmallInt` behaviour arm

`receiver_kind` answers `Primitive::SmallInt` for `Decoded::SmallInt`, where one arm previously
covered the tag and the handle's inline text together, and `receiver_behaviour` maps both kinds to
`String`'s instance behaviour. The answer is unchanged by construction: `RexxInteger`'s own id is
`String` (`classes/IntegerClass.cpp:2066`, `CLASS_CREATE_SPECIAL(Integer, "String",
RexxIntegerClass)`), measured live -- `12345~class~id` is `String` and `12345~length` is 5, oracle
rc 0 and both engines byte-identical.

**It buys nothing today and is not a cheaper route either**, which the first draft of its own doc
comment claimed. See fix round 1's section: the arm it was split out of decided from the tag as well,
so no arena read or class-range test was being paid before the split; every consumer folds the new
variant straight back; and a small integer is indistinguishable from the same digits as a string
across everything this crate can ask. What it is, is the named place D24 asks for.

### The receiver in the calling convention

`CallContext` gains `receiver: Option<ObjRef>`. `enter_method_body` replaces the convention before it
pushes the activation, then reads the receiver back out of it for both the `MethodIdentity` and the
`SELF` slot; `invoke_call` fills `None`, which is `RexxActivation::getReceiver`'s `OREF_NULL` for a
frame no send entered (`execution/RexxActivation.cpp:2342`-`:2349`).

The read-back is the point rather than ceremony: a method's own `SELF` and the caller a send inside
that method resolves as now come from one field, which is what the oracle's single `getReceiver`
makes them. It also matches the oracle's frame semantics without a special case -- an `INTERPRET`
fragment runs in the activation that interpreted it and so reads that activation's convention, where
the C++ forwards to the parent explicitly (`:2344`-`:2347`).

### Selectors interned at compile time

`rexx-parse` gains `Selector` and `SelectorTable`. `Selector` is a shared `Arc<[u8]>` that derefs to
the bytes; `Selector::same` is allocation identity. `ParseCtx` carries the pool in a `RefCell`
because every `parse_*` function takes `&ParseCtx` and a message name is minted while one of them
runs. `Parser::message` interns the upcased name and `Parser::collection_message` interns `[]`, so
`ExprKind::Message.name` is a `Selector`.

This is the oracle's own shape, at the same layer: `parseMessage` does
`messagename = commonString(messagename->upper())` (`parser/LanguageParser.cpp:3391`), `commonString`
hands back the pool's copy when the spelling is already there (`:2269`-`:2280`), and the pool is a
field of the parser (`parser/LanguageParser.hpp:481`) -- so its scope is one parse, which is this
pool's scope too.

**What landed and what did not, stated rather than dressed up.** The pool and the identity land, and
that is D24's constraint as the spec words it -- selectors interned at compile time. **The method
dictionary stays keyed by `String`** (`rexx-classes`'s `MethodDict::entries`), so `resolve` still
does `String::from_utf8_lossy(name)` per send and no lookup compares selectors yet. Keying the
dictionary on a selector is what the not-yet-landed half would buy: it would remove that per-send
conversion and turn a name comparison into an integer or pointer comparison, and it needs a numeric
id rather than a shared allocation, a pool that outlives one parse (a dynamically computed name --
`~hasMethod('M')`, and the `UNKNOWN` forwarding Task 12 adds -- has no compile-time selector), and a
decision about non-UTF-8 names, whose lossy spelling is not their raw one. Nobody has costed that,
and this task does not pretend to have.

`Deref` is why the blast radius is one crate, and the compiler is what says so rather than a grep:
the field's type changed and **no file outside `rexx-parse` needed an edit for it** -- `cargo build
--workspace` was clean at that point, and the `rexx-exec` changes in this task belong to the other
three constraints. That is also the honest measure of what the interning currently buys the send
path: the AST holds one allocation per spelling per parse instead of one per occurrence, and the
dispatch path does byte for byte the work it did.

## The roadmap's patchable-slot sentence: amended

`docs/superpowers/plans/2026-07-27-rust-rewrite.md:492` read, printed before and after the edit:

```
492: Its reasons are prior and architectural: it founds OO dispatch for Phase 5 by making a call
     site a patchable slot, it is the shape a Cranelift or WebAssembly backend consumes, and it
     makes trace an emission decision rather than a mode flag.
```

Amended, because the claim is not about this tree:

* `Op::Message { index: u32 }` (`rust/crates/rexx-exec/src/ir.rs:1060`) has no `site` field, and the
  op's own doc says why (`:1037`-`:1040`): "No `site`, and that is D28 rather than an omission."
* `Op::Call`'s `site` is the classic-call cache, `struct CallSite(Cell<Option<Resolved>>)`
  (`ir.rs:1509`).
* `dispatch.rs:23`-`:27` states the consequence: "`crate::ir::CallSite` is a classic-call cache and
  no send goes near it."
* D28 is carried unchanged (`docs/superpowers/specs/2026-08-17-phase-5-object-model.md:1123`).

So there is no slot a send could be patched into, and the sentence's first reason was false about OO
dispatch specifically. The edit keeps the sentence's other two reasons, narrows the first to a
**classic** call site, and adds what the IR does give OO dispatch: the shared `resolve`/`invoke` pair
and the clause region a send op sits inside. The spec's own citation of `:492` still lands on the
sentence it is about. It is its own commit.

## The five gate commands

Run from `rust/`, each status read unpiped from its own `$?`.

| # | command | exit |
|---|---|---|
| 1 | `cargo fmt --all --check` | 0 |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| 3 | `cargo test --release --workspace` | 0 |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 0 |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 |

Gate 3 and gate 4 each report **1818 passed, 0 failed** summed over their test binaries, and gate 5
**1819 passed, 0 failed**. The extra one is `rexx-core/src/bytes.rs:320`'s
`the_bytes_past_len_are_never_part_of_the_value`, which carries `#[cfg(debug_assertions)]` on the
test itself -- checked in the logs, it appears in gate 5's and not in gate 3's, and it is the only
`#[cfg(debug_assertions)] #[test]` in the workspace. It is not a `debug_assert` firing, which is what
this sentence claimed first. Gate 4's corpus report reads **143 of 143 matching**
under `mode: STRICT (the gate)`. `memcap` was present (`/home/moritz/.local/bin/memcap`), checked
rather than assumed.

All five ran against the tree that became `b55a721d1` -- no file under `rust/` was edited after gate
3 started -- and each status above is that command's own `$?` with no pipe in front of it.

## Both tables' 5a row counts

Each table prints its own per-phase summary; these are those lines, not a count of my own.

| table | run | 5a rows before | 5a rows after |
|---|---|---|---|
| C | `cargo test --release -p rexx-exec --test gate_table_c` | 135 rows, 117 not yet `agree` | 135 rows, 117 not yet `agree` |
| D | `cargo test --release -p rexx-exec --test gate_table_d` | 36 rows, 7 not yet `agree` | 36 rows, 7 not yet `agree` |

The "before" runs are at base `5fd2002cf`: table C under `REXX_CORPUS_GATE=1 REXX_PHASE_GATE=5a`
(which exits 101 at HEAD with or without this change -- 117 rows of an open phase do not agree, as
Task 9's report records), table D under `REXX_CORPUS_GATE=1`. The "after" figures are both from gate
4's own run of those two binaries. **The phase column is `owning_phase`, a committed assignment this
change does not touch, and the row sets come from `corpus/docs/*.txt`, which it does not touch
either** -- so the counts could only have moved if the diff had touched one of those, and it does
not.

## The three in-crate tests, each under its mutation

Each mutation was applied by a script, run as `cargo test --release --workspace --no-fail-fast` --
`--no-fail-fast` because without it the run stops at the first catcher and "nothing else caught it"
is unmeasured -- and then **restored from a copy taken before the edit, never with `git checkout`**,
with the restore verified by `diff -q` against that copy. Every run reports a non-zero test count, so
none of these is the "matched nothing, exited 0" shape.

**The line numbers below are the tree's, not the mutant's**, and that distinction cost a correction
here: a mutation that deletes lines shifts every panic location after it, so `libtest` reported
`selector.rs:118` and `dispatch.rs:1912` where the assertions sit at `:121` and `:1915` in the
committed file. Each was re-read at HEAD before being written down.

All three constraints are **structural**: each failure below is red in a run with no gate environment
variable set.

| # | constraint | mutation | suite result | reddened |
|---|---|---|---|---|
| 1 | selectors interned at compile time | `SelectorTable::intern` drops the pool lookup and returns a fresh `Arc` per call | exit 101, 1815 passed / 3 failed | 3 tests |
| 2 | a `SmallInt` behaviour arm | `receiver_kind`'s `Decoded::SmallInt` arm answers `Primitive::String` | exit 101, 1817 passed / 1 failed | 1 test |
| 3 | a receiver in the calling convention | `enter_method_body` binds from `resolution.scope` instead of reading the convention back | exit 101, 1817 passed / 1 failed | 1 test |

### 1. Interning

`one_method_name_is_one_selector_across_a_whole_program` fails on its first assertion, the message
**"two symbol-spelled sends of one name hold two selectors"** (`expr/tests.rs:909`).
`the_bracket_form_and_the_written_name_are_one_selector` fails on
`assert!(crate::Selector::same(&selectors[0], &selectors[1]))` (`:931`), and
`one_spelling_interns_to_one_allocation_and_two_spellings_do_not` on **"the same spelling interned
twice is two allocations"** (`selector.rs:121`).

**Nothing else in the workspace caught it** -- 3 failed, all three mine -- so these tests add coverage
the suite did not have rather than merely being able to fail. Which is what a byte-identical
mutation should do: every `==` comparison in the tree still answers the same, and the corpus, both
gate tables and `ir_dual` stay green.

### 2. The `SmallInt` arm

`a_small_integer_receiver_takes_the_small_int_arm` fails at `dispatch.rs:1865` on
`assert_eq!(interp.receiver_kind(integer)..., Primitive::SmallInt)` with the message **"a tagged
integer receiver went down the general path"**, printing `left: String  right: SmallInt`.

**One failure in the workspace, and it is mine.** The mutation is behaviour-preserving by
construction -- both kinds map to `String`'s instance behaviour -- so nothing that compares output
can see it. The test's later assertions, that both kinds answer one behaviour and that
`12345~length` is 5 on both engines, are **not reached** under this mutation, because the kind
assertion panics first; they are the half that would redden if an arm pointed the tag at some other
class, which is a different mistake from folding the arm away.

### 3. The receiver in the calling convention

`a_method_send_binds_self_from_the_receiver_in_the_calling_convention` fails at `dispatch.rs:1915`
on the `assert_eq!` over `both_engines(...)` with the message **"SELF is not the receiver of the
send"**, printing

```
  left: (0, "The K class\n", "")
 right: (0, "The J class\n", "")
```

-- the scope where the receiver belongs, on both engines.

**One failure in the workspace, and it is mine, which is the result worth reading.** The existing
`self_and_super_are_bound_before_the_bodys_first_instruction` stays green under this mutation,
because its method is defined on the class the send is addressed to and so cannot tell a receiver
from a scope; and the corpus, both gate tables and `ir_dual` stay green too. So a `SELF` re-derived
from the resolution is a wrong answer that nothing else in this tree sees, and the inherited-method
program is what makes it visible.

## The sitting

Run at `b55a721d1`, after rebuilding the release binary at that commit:

```
./target/release/rexx-arms --build pinned=bench-baselines/pinned/rexx-run-15a1ffa98 \
                           --build head=target/release/rexx-run \
    --axis alloc4c --axis arith --axis compound --axis emptyloop --axis strings --axis varlookup \
    --rounds 5 --task 10 --commit b55a721d1 --baseline bench-baselines/phase-5a-arms.tsv
```

exit 0. The axes are the six named on those `--axis` flags, copied from `global-constraints.md`'s
command line -- `rexx-bench-band.rs:182`'s own default axis list is a different tool's default and is
not what ran.

**The pin, checked rather than assumed.** `bench-baselines/pinned/rexx-run-15a1ffa98` has sha256
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, which is what `PINNED.md`
records. `15a1ffa98` is an ancestor of HEAD (`git merge-base --is-ancestor`, exit 0), and every
commit `git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml rust/Cargo.lock Cargo.toml`
lists is named in this plan's ledger, checked by matching each abbreviated hash against
`progress.md` rather than by reading the list.

**The head binary this table measured** has sha256
`2fed984120514fc74e4ec591dc823d66bb7ed7b677f45ac66f5c4a230ac617ce`. Stated because the mutation runs
left a mutant under that same path -- sha256 `9ec0c7ebb87faaeca905b338da87c43e6d31cdc8d40df8d103f13ee9ca96ddf4`
-- and it was rebuilt before this sitting rather than reused.

### `instructions:u`, ratio `pinned>head`, `value_median` column, task `10` rows of this run

| axis | `tw` small | `ir` small | `tw` large | `ir` large |
|---|---|---|---|---|
| `alloc4c` | 1.000795 | 1.001202 | 1.000774 | 1.001157 |
| `arith` | 1.001006 | 1.001284 | 1.001036 | 1.001294 |
| `compound` | 1.000000 | 1.000000 | 1.000000 | 1.000000 |
| `emptyloop` | 1.000000 | 1.000000 | 1.000000 | 1.000000 |
| `strings` | 1.005749 | 1.009685 | 1.005749 | 1.009684 |
| `varlookup` | 1.000000 | 1.000000 | 1.000000 | 1.000000 |

**Nothing reaches 1%**, so no axis is a finding and no interleaved control is owed.

**Every cell against `9-fixround-1`'s, compared by running the comparison rather than by reading the
two tables.** Of the cells this run and that one share, these differ, each by one in the last digit:

| cell | `9-fixround-1` | this run |
|---|---|---|
| `alloc4c`/`ir`/small | 1.001203 | 1.001202 |
| `strings`/`ir`/large | 1.009685 | 1.009684 |
| `strings`/`tw`/small | 1.005750 | 1.005749 |

Every other cell is identical to six decimals. **Worth saying how this paragraph was written**: read
off by eye it named the wrong set -- two cells, one of them `alloc4c`/`ir`/large, which is 1.001157
in both runs and whose 1.001156 belongs to a third run entirely. The script that keys both row sets
by (axis, arm, size) and compares the `value_median` column found three, and that is what stands
here.

### The budget: `strings`/`ir` per pass, `instructions:u`, `value_median` column, task `10` rows of this run

The ratio alone is not the acceptance, so here is the instruction count it stands for.

| | instructions per pass |
|---|---|
| `pinned` | 5369.518229 |
| `head` | 5421.517944 |
| head - pinned | **51.999715** |
| 1% of `pinned`, which is the guard | 53.695182 |
| headroom remaining | **1.695467** |

**This task's own contribution to that figure is -0.000180 instructions per pass** -- the branch stood
at 51.999895 at `5fd2002cf` (`9-fixround-1`'s rows in the same file, same column, same arithmetic),
and it now stands at 51.999715.

**Both numbers, because "smaller than the pin's own spread" was the wrong comparison and I made it.**
The pinned build's per-pass figure moved by **0.000092** between the two runs (5369.518137 to
5369.518229) while measuring the same bytes; my delta against the previous task is **0.000180**, so it
is about twice that movement, not smaller than it. What holds either way is the scale: both are four
orders of magnitude below the 1.70 instructions per pass the budget has left, so the honest statement
is **nothing this axis can resolve was added**, and not that 0.00018 was given back.

**Why that is the expected answer, anchored so a reviewer's own check agrees.** None of the six axis
programs sends a message or makes a `CALL`, so `resolve`, `receiver_kind` and the `CallContext` that
grew are on no executed path in any of them. **A bare `grep` says the opposite and is wrong**:
`/bin/grep -c '~' bench-programs/alloc4c.rex` answers `4`, and **that 4 is a count of lines, not of
tildes** -- `grep -c` counts matching lines, and the file holds **8** occurrences on those 4 lines,
every one inside its prose comment, which exists to say that `.array~of`, `.string~new`, `~size` and
`~length` are *not* used because they are unimplemented. Either number is fine to quote; saying
which is not optional, and the first draft of this paragraph said "all four `~`". The anchored form strips `/*...*/` first and then counts the
only two spellings the parser turns into a message -- `~`/`~~` and `[`, which reach the two
`ExprKind::Message` constructors at `expr.rs:866` and `:932` and nothing else does -- and it answers
**0 and 0 for all six**. Two controls, because a checker that always answers zero would pass this
too: the same stripper answers `1` for a probe whose send is real code, and `0` for `/* a~b */` with
a `say` after it.

`compound`, `emptyloop` and `varlookup` at exactly 1.000000 are the same statement from the other
side: those three retire the pinned build's instruction count to the last instruction.

## What I could not close

**Read this section against fix round 1, which closed part of it.** Three of the items below were
written before that round and are corrected in place; each says what closed it. The lesson is the
brief's and it is worth keeping: **a section headed "what I could not close" is the one a reader
trusts to be current, and it is the section least likely to be re-read on the day one of its items
gets closed.** In round 1 I corrected four false statements in place elsewhere in this file and left a
stale item standing here, one paragraph from another whose closure I had announced in a later section
without amending the item itself.

**No axis exercises the send path, so the sitting cannot see a regression in `resolve` itself.**
`bench-programs/dispatch.rex` is the send-heavy program and it is not one of the axes this guard
names; it also cannot run here, measured at the commit -- `c = .counter~new` gives rc 120 and
`rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)`, instance construction being
5b's.
So the sitting's answer for this task is "nothing I added is on any executed path", which is true and
is a weaker statement than "the send path did not get slower".

**Closed by fix round 1's item 3**, which built the missing instrument: three interleaved probes over
a native send, a method send and an internal `CALL`, against a build at the previous task's head. The
sentence that stood here -- that the instrument saying "the send path did not get slower" does not
exist in this plan -- was wrong twice over, because it was buildable in an afternoon and because the
answer it gives is not the one this paragraph assumed: the reshaped paths cost about 3, about 12 and
18 instructions per operation. What remains true is the narrow claim: **the six axes cannot see any of
that**, so the budget the guard enforces is not the instrument that bounds this task's cost.

**The selector is not a dispatch key.** Stated in full above; the summary is that `MethodDict` is
keyed by `String`, `resolve` still converts the name per send, and turning the selector into the key
needs a numeric id, a pool wider than one parse, and a decision about non-UTF-8 names. Nobody has
costed it.

**`Caller`'s two halves still have no reader, and what stands behind them is an `allow`, not an
assertion.** Task 13 is what reads them, per the brief. This item said the `debug_assert` in `resolve`
was what stood behind them; **fix round 1's item 2 deleted that assert**, precisely because no
in-tree path could falsify it, and replaced it with
`#[allow(dead_code, reason = "read by Task 13's PRIVATE check")]` on each accessor -- a marker that
says what it is instead of a check that cannot fail. The residue is real and unchanged: until Task 13
lands, nothing reads either field.

**The trap for whoever writes the `PACKAGE` check: closed, not documented.** It stood here as an
`Option<ProgramId>` on both sides whose `None` meant "no activation is running" in `Caller::package`
and "the `REXX` package" in `Interp::package_objects` -- one type, two opposite meanings, and a
`PACKAGE` check comparing them directly would have called a primitive class's package "no package".
**Fix round 1's item 2 made both sides variants** (`plan::Package { Rexx, Program }` and
`CallerPackage { NoActivation, Package }`), on the controller's ruling that a hazard a comment carries
is a hazard that ships. Nothing here is left for Task 13 to avoid.

**The message-assignment form still derives its setter name per execution.** `exec_message` builds
`NAME=` with a fresh `Vec<u8>` on every execution of an assignment-form send (`run.rs`, the
`Some(value)` arm). The parser knows that form at parse time and could intern the setter selector
with the getter, which would remove that allocation; it is not in this task's scope and no measurement
here prices it.

**One fidelity point that reads like a gap and is not.** A selector interned by a program's parse and
one interned by an `INTERPRET` fragment's parse are different allocations with equal bytes. That is
the oracle's own behaviour, not an approximation of it: `strings` is a field of `LanguageParser`
(`parser/LanguageParser.hpp:481`) and `INTERPRET` builds a fresh parser, so its pool is fresh too.

---

# Fix round 1

Three Important, eight minors. Base for this round: `207756aae`, on top of the plan amendments
`fc31fb82a` and `936b5687f`. Commits: **`1d7bc4b95`** the code and the two document corrections,
**`0d8203280`** the sitting rows. The four false statements the
round found in the sections above are corrected **in place** rather than only here, so nothing in
this file states them any more; each correction says what it said first.

## Item 1: `Entered::Label` carried the wrong receiver, and the comment said it was right

The comment was the worse half and it is gone. `RexxActivation::internalCall` passes **its own
receiver** -- `return newActivation->run(receiver, name, ...)` at
`execution/RexxActivation.cpp:3313`, which `RexxActivation::run` assigns at `:474` -- and only
`RexxActivation::internalCallTrap` passes `OREF_NULL`, at `:3343`. So an internal `CALL` inherits and
a trap does not, where the shipped comment asserted that neither did.

**Measured myself, from a fresh empty directory, four programs.** A `::method priv class private` is
the instrument, because a private send is allowed exactly when the sending activation's receiver is
the object being sent to:

| probe | oracle |
|---|---|
| `CALL inner` inside a class method, `inner` sending `self~priv` | **rc 0**, stdout `private-reached` |
| the same send from that method's `CALL ON ERROR` handler | **rc 159**, `97.2 Object "The K class" cannot accept private message "PRIV" from this context.` |
| the same send from a `::ROUTINE` the method called with `self` as its argument | **rc 159**, the same 97.2 |
| `.K~priv` at the top level -- the control | **rc 159**, the same 97.2 |

The third row is mine to have added: the brief named three probes and a `::ROUTINE` is a third arm of
the decision, so it needed its own measurement rather than an assumption that it behaves like a trap.

**Split at the type level, not in prose.** `CallEntry { Written, Trap }` (`run.rs:406`) is threaded
through `invoke_call`, `invoke_call_over` and `resolve_and_run_call`; the condition-delivery site is
the one `CallEntry::Trap`. The decision is a pure function, `entered_receiver(entered, entry, caller)`
(`run.rs:437`), whose doc carries all four transcripts above.

**The test**, `a_trap_handler_and_an_internal_call_do_not_carry_the_same_receiver`
(`run/tests.rs:8491`): the label-and-`Written` row must answer the caller's receiver, the
label-and-`Trap` row and both `Routine` rows must answer `None`, and a caller with no receiver must
answer `None` on every route. A build that gave a trap the same receiver as an internal `CALL`
reddens the second assertion, `"a CALL ON handler was given the receiver internalCallTrap
withholds"`.

**Why it is a unit test over the decision and not a differential row**, said in the test's own doc:
no access scope is implemented here, so no program this crate can run tells the three routes apart in
either direction. The differential cannot see this field at all.

## Item 2: two `None`s with different meanings, and a guard that could not fail

**Both sides are variants now.** `plan::Package { Rexx, Program(ProgramId) }` is what
`Interp::package_objects` and `package_root_key` key on, so the interpreter's own package is a variant
rather than an absent program id; `CallerPackage { NoActivation, Package(Package) }` is the caller
side. Neither absence is an `Option<ProgramId>` any more, so the two cannot be compared by accident
-- which was the trap this report's own "what I could not close" section had recorded for Task 13 and
which is now closed instead of documented.

**The `debug_assert` is deleted.** It asserted that a caller with a receiver has a package, which
`Interp::caller` fills together and which no in-tree path could falsify: an assertion standing where
a reader expects a check, finding nothing. What keeps the two accessors out of `dead_code` is now
stated as what it is:

```
#[allow(dead_code, reason = "read by Task 13's PRIVATE check")]
```

**With the control run, because an `allow` that suppresses nothing is decoration.** Stripping both
attributes and running `cargo clippy -p rexx-exec` gives exactly one warning, `methods 'receiver' and
'package' are never used`, pointing at `Caller`'s `impl` block -- so the lint fires on the accessors,
not on the fields, and the attribute names the right thing. (The line number that warning printed is
the stripped file's, two lines short of the committed one, which is the same shift that made round
1's panic-site citations wrong; the accessors are at `dispatch.rs:437` and `:444` as committed.) `expect` is the wrong tool here for the reason
`rexx-parse/src/lib.rs`'s own note records: the lint fires in the library compilation and not in the
library-as-test one, so the expectation would be unfulfilled in the second and that is a warning of
its own.

`resolve` now says plainly that nothing reads `caller` yet and binds it with `let _ = caller;`.

## Item 3: the send-path figure, taken

**My own interleaved run**, three builds alternated inside each round -- the phase pin, a `rexx-run`
I built at `5fd2002cf` for attribution (sha256 `4a56081eed521eca03a22870a01670847bf1cf3aa1ed780d73a76287720794af`),
and this head -- five rounds, `instructions:u` through `perf stat`, three probes. Medians, with the
`head/prev` spread across the five rounds:

| probe | engine | head/pin | head/prev | min | max | per operation vs `prev` |
|---|---|---|---|---|---|---|
| `s~length` loop, 2,000,000 sends | ir | 1.00171 | 1.00129 | 1.00123 | 1.00129 | **+3.03** |
| | tw | 1.00158 | 1.00120 | 1.00116 | 1.00553 | **+3.03** |
| `.K~m` loop, 500,000 sends | ir | 1.00110 | 1.00187 | 1.00182 | 1.00191 | **+11.95** |
| | tw | 1.00107 | 1.00182 | 1.00179 | 1.00643 | **+11.85** |
| internal `CALL` loop, 1,000,000 calls | ir | 1.00708 | 1.00708 | 1.00708 | 1.00708 | **+18.00** |
| | tw | 1.00572 | 1.00572 | 1.00572 | 1.00572 | **+18.00** |

**What they cover**: the paths this task reshaped -- a native send, a Rexx-bodied method send, and an
internal `CALL` -- did not slow by anything close to 1%, on both engines, with the pin and both heads
measured inside one sitting rather than in separate ones.

**What they do not cover.** They are not axis rows and they do not touch the 1.695467-per-pass
budget. They say nothing about `PRIVATE`/`PACKAGE`, which are unimplemented, nor about the trap route,
which no program can reach observably. And they are three programs I wrote, so they cover the shapes I
thought of.

**The `ir` arm does not settle, and the tree-walker arm is the one to reason from.** Five rounds:
`send_call` is exact on both arms (min = median = max), but the two send probes show single-round
excursions -- `send_native`/tw round 3 and `send_method`/tw round 5 each about 0.4% above their own
median, and an earlier three-round run had `send_method`/ir at 0.99800, 1.00575, 1.00452 where tw held
1.00112, 1.00111, 1.00106. So these probes are not reproducible to the eight figures the axis rows
manage, medians over five rounds are what the table reports, and a single round of either arm is not
evidence.

### The +18 per internal `CALL`, attributed with a control rather than a story

`head/pin` equals `head/prev` on that probe, so Tasks 1-9 added nothing to the call path and all 18
instructions are this task's.

**A build whose `entered_receiver` body is replaced by `None`** (sha256
`23cbc48dd69d4f669defdbe08ae2fe8774e0fb0f28e1f55a101c7a4cb7a3d0f2`) measures **2,539,641,266** where
Task 9's head measures **2,540,641,934** and this head measures **2,558,640,735** -- one instruction
per call *below* the previous head. So the cost is neither the parameter threading nor `CallContext`
growing by an `Option<ObjRef>`; both are present in that build. `#[inline(always)]` on the function
changes nothing (2,558,641,518 against 2,558,641,244), so it is not an inlining decision.

**I am not naming a mechanism past that.** What the instrument supports is: the cost appears when the
decision is computed rather than folded to a constant. Note the probe's `CALL` is written at the top
level, where the caller has no receiver, so both builds compute the same answer -- the difference is
the computing, not the value.

**The route I rejected, with its reason.** `Interp::caller` could walk the activation stack to the
nearest method identity instead of reading a field, which would pay on the send path (rare) rather
than the call path (hot). That is precisely not "a receiver in the calling convention", which is the
D24 constraint this task exists to land. Kept, at a stated price.

## The `.text` predicate, and the sitting

**`.text` changed, so a sitting is owed and was run.** `size -A` gives **1,221,865** bytes at
`5fd2002cf` against **1,226,713** at this round's head, and the binaries differ
(`4a56081eed521eca03a22870a01670847bf1cf3aa1ed780d73a76287720794af` against
`925c4a697bb475debfd1c5799a08634d7b330fe8a80f2fc67d95ac7ba0a878cd`). Round-1 sitting below.

## The five gate commands, this round

| # | command | exit | result |
|---|---|---|---|
| 1 | `cargo fmt --all --check` | 0 | |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | |
| 3 | `cargo test --release --workspace` | 0 | 1819 passed, 0 failed |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace` | 0 | 1819 passed, 0 failed; corpus **143 of 143 matching** |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 | 1820 passed, 0 failed |

Each status is that command's own `$?`, read with no pipe in front of it. One test more than the
round before it, which is `a_trap_handler_and_an_internal_call_do_not_carry_the_same_receiver`. Both
tables' 5a row counts are unchanged again: C **135 rows, 117 not yet `agree`**, D **36 rows, 7 not yet
`agree`**.

## The minors

* **The false spread claim** and **the tilde count** and **the `expr.rs` constructor lines** and
  **gate 5's extra test**: all four corrected in place above, each saying what it said first.
* **The `SmallInt` rationale claimed more than the arm does.** Rewritten: it states that the arm
  changes nothing today, that it is not a cheaper route because the arm it was split out of also
  decided from the tag, and that every consumer folds it back. The indistinguishability is now
  measured by me rather than taken: `~class~id`, `~length`, `~isA`, `~hasMethod` and `~reverse` are
  byte-identical between `12345` and `'12345'` on the oracle and on both engines, rc 0, stderr empty.
  Class-object `==` could not be included -- it is a Phase 5 gap here
  (`the operator == applied to a class object is not implemented`), which is itself worth knowing.
* **`dispatch.rs`'s `.expect` was unreachable and the parameter stayed in scope.** The binding now
  **shadows** `receiver`, so past that line the name is the convention's field and the argument is
  unreachable -- the property is enforced by scope instead of asserted in a comment.
* **Roadmap `:494`'s "nothing reads it" was false.** Corrected to name the reader: `version_at` is
  defined once (`rexx-classes/src/class_graph.rs:899`) and called only from
  `behaviour_wiring.rs:807`-`:825`, and the field behind it is written at `:493` and `:790` and read
  only inside that accessor -- so "no interpreter path reads it" is what the line says now.
* **The spec's `:1179` row** quoted the pre-amendment sentence. It now records the amendment, what the
  line says instead, and that the old spec's `:482` citation is the stale one.

## Fix round 1's sitting

Same command as round 1's, `--task 10-fixround-1 --commit 1d7bc4b95`, exit 0. Pin verified again by
sha256 against `PINNED.md`; head binary
`925c4a697bb475debfd1c5799a08634d7b330fe8a80f2fc67d95ac7ba0a878cd`.

**Nothing reaches 1%.** Of the 24 `instructions:u` ratio cells, two differ from task `10`'s by one in
the last digit -- `alloc4c`/`ir`/large 1.001157 to 1.001156 and `strings`/`tw`/small 1.005749 to
1.005750 -- and the other 22 are identical. Compared by running the comparison, not by reading the two
tables, after that method caught the wrong answer in round 1.

The budget, `strings`/`ir` per pass, `value_median`:

| | round 1 (`10`) | this round (`10-fixround-1`) |
|---|---|---|
| `pinned` | 5369.518229 | 5369.517840 |
| `head` | 5421.517944 | 5421.517533 |
| head - pinned | 51.999715 | **51.999693** |
| 1% of `pinned` | 53.695182 | 53.695178 |
| headroom remaining | 1.695467 | **1.695485** |

**Which is the expected answer and for the stated reason**: no axis program sends a message or makes a
`CALL`, so `entered_receiver`, the wider `CallContext` and the `Package` rekeying are on no executed
path in any of the six. The paths they *are* on are measured in item 3 above, where they cost about 3
instructions per native send, about 12 per method send and 18 per internal `CALL` -- figures the axis
sitting cannot produce, which is why item 3 existed.

---

# Fix round 2 -- three documentation fixes, no behaviour

Base `0d8203280`, on the plan amendment `936b5687f`. Commit: **`5957d6726`**. Round 1 came back substantially solid: the
four-arm receiver decision confirmed against the oracle by the reviewer's own probes, the test red
under a sandboxed mutation of the `Trap` arm, the `allow` control real, all five C++ citations
correct, and **the +18 per `CALL` independently reproduced** at 17.999847 against my 18.00.

## 1. A false sentence in new code

`plan.rs`'s new `Package` doc said `Interp::package_objects` **and `Interp::class_packages`** key on
`Package`. Printed, `class_packages` is `HashMap<ObjRef, ProgramId>` (`lib.rs:2347`) -- keyed on a
class handle, not on a package at all -- and my diff never touched it. The sentence is now two, and
the second answers the "should it be rekeyed" question with a reason rather than deferring it to
somebody (`plan.rs:62`):

> **`Interp::class_packages` does not key on this and wants no variant.** It maps a class handle to
> the `ProgramId` whose directives installed that class, so a class the bootstrap registered is
> simply absent from it, and `Interp::package_object_for` is the one reader that turns the absence
> into `Package::Rexx`. Putting a `Package` in there would give one state two spellings -- absent,
> and present as `Rexx` -- which is the shape this enum exists to remove.

**"The one reader" is checked, not assumed**: `grep -rn class_packages crates/ --include=*.rs` gives
one write (`environment.rs:617`), one read (`:629`, inside `package_object_for`), the declaration and
the initialiser, and nothing else.

**How it got in.** I wrote the doc while rekeying `package_objects` and named the other package map
beside it from memory instead of opening it. It is the shape this project already has a record for --
a list copied from an authoritative-feeling source without opening each member -- and the fix that
holds is the one applied here: name what the code holds, or say nothing about it.

## 2. A stale claim in the report -- and re-reading the section found three, not one

The brief named one: "What I could not close" still said `resolve`'s `debug_assert` stood behind
`Caller`'s two fields, which **round 1's item 2 deleted**. Re-reading the whole section rather than
the named paragraph found two more:

| item as it stood | what was wrong |
|---|---|
| `Caller`'s halves have no reader **but an assertion**, compiled out of every release gate | the assert was deleted in round 1; an `allow` on each accessor is what stands there |
| **a trap for whoever writes the `PACKAGE` check** -- two `Option<ProgramId>` with opposite `None`s | closed in round 1 by making both sides variants; the types it describes no longer exist |
| the instrument that would say "the send path did not get slower" **does not exist in this plan** | round 1's item 3 built it, and its answer is not the one the paragraph assumed: about 3, about 12 and 18 instructions per operation |

All three are corrected in place, each stating what it said first and what closed it. The third is the
worst of them: it was not merely stale, it asserted the non-existence of something that took an
afternoon to build, and it would have discouraged the next reader from building it.

**The lesson, which is the brief's and worth keeping.** A section headed "what I could not close" is
the one place a reader trusts to be current, and it is the section least likely to be re-read on the
day one of its items gets closed. In round 1 I corrected four false statements *in place* in this same
file and walked past a stale item sitting one paragraph from a closure I had announced in a later
section without amending the item itself. The rule that would have caught it: **when a round closes
something, re-read the section that recorded it as open, not just the section you are writing.**

## 3. The `task` column's two spellings

`rust/bench-baselines/README.md` documented neither spelling. It now has a section that does, and the
controller's correction to their own framing is what it states rather than the framing: **a bare
number does not mean a single sitting.**

Re-derived by running the command the README names, not by reading the file:

| label | commits |
|---|---|
| `6` | 4 |
| `7` | 2 |
| `8` | 2 |
| `9` | 2 |
| `9-fixround-1` | 1 |
| `10` | 1 |
| `10-fixround-1` | 1 |

So the asymmetry is narrow: **only the reviewer-driven fix round took a `-fixround-N` suffix, and
only for Tasks 9 and 10; Tasks 6, 7 and 8 recorded theirs under the bare number.** The README says
the `commit` column is the reliable discriminator either way, states this as what the file contains
rather than as a rule a later plan inherits, and says why the rows are not rewritten: they are
measurements, and editing them to a spelling decided afterwards would be editing data. **No committed
row was touched** -- this round's diff is `README.md` and one doc comment.

## The five gate commands, as this shell's own children

The re-review could not confirm round 1's gate 5 because it ran as a background job. Each of these ran
in the foreground with `echo` reading `$?` on the same line, so every status below is that command's
own:

```
GATE1 cargo fmt --all --check -> exit 0
GATE2 cargo clippy --workspace --all-targets -- -D warnings -> exit 0
GATE3 cargo test --release --workspace -> exit 0                                   1819 passed, 0 failed
GATE4 REXX_CORPUS_GATE=1 cargo test --release --workspace -> exit 0                1819 passed, 0 failed
GATE5 REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast -> exit 0  1820 passed, 0 failed
```

Gate 4's corpus report reads **143 of 143 matching**, and both tables' 5a counts are unchanged for the
third round running: C **135 rows, 117 not yet `agree`**, D **36 rows, 7 not yet `agree`**.

## The `.text` predicate: no sitting, and the claim is measured on `.text` itself

**I land on "`.text` does not change", and the evidence is the section's own hash rather than its
size.** Same size would not have settled it:

| | pre-round | this round |
|---|---|---|
| `.text` sha256 | `0bb65c711a993e1090c3fcc5bb1b5d0a26cc337afb068d7e7cb3ed3ee78cb1c3` | **identical** |
| `.text` bytes | 1226713 | 1226713 |
| `.rodata` sha256 | `5e6e5b7ba6e19d8a…` | **identical** |
| whole file sha256 | `925c4a697bb475debfd1c5799a08634d7b330fe8a80f2fc67d95ac7ba0a878cd` | `d4dd747d6b6928b6bf6347465ebacd9bb85af2e4b125b63ae47c5b946b6c1f0b` |

**The whole-file sums differ, and that is not a contradiction -- it is why the predicate names
`.text`.** One section moved, `.data.rel.ro`, and rather than guess at it I ran a control: the same
comment text placed at the **end** of `plan.rs`, where it shifts no item's line number, gives
`.data.rel.ro` `d66c0eec7208703c` -- **byte-identical to the pre-round build** -- where the same text
as a doc comment above `enum Package` gives `cd9ae278860ad61d`. So what changed is line-number data
for code below the insertion, and nothing else; `.text` is untouched either way. No sitting is owed
and none was run.
