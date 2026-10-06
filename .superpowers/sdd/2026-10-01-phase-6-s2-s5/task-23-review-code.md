# Task 23 review, code half

Diff `811610f63..3d0ab296b`, `rust/` only. Probes ran in a `git archive` copy of `3d0ab296b`
(`rust/` and `interpreter/`) under `/tmp/claude-1000/p6-t23-rev-code/`, target dir there, deleted
after.

**Spec verdict: PASS WITH ISSUES.** A1-A5 are in place and correct, P72's seal holds, `sharing` is
off by default and adds no `.text` byte when off, the ping-pong programs measure what they claim, and
the refusal-sites re-derivation changed only the definition column. The issues: the SAFETY argument
for the `Send` grant names one dereference path of two, and the sharing counter counts a heap
registry walk as a touch, which the brief excludes.

**Quality verdict: PASS WITH ISSUES** (findings 1 and 2 should be fixed before close; the rest are
Minor).

## Findings

### 1. Important: the SAFETY comment on the `Send` grant names one dereference path of two (`rexx-exec/src/island.rs:46-51`)

The comment says the root pointer `NonNull<Interp>` "is dereferenced only by `Lent::interp`, while
its lender waits". That is false. `Island::host` (`island.rs:86`) hands the same pointer to the API
through `Requester::host` and `Recalling::host` (`dispatch/library.rs:1247`, `:1271`, both
`self.baton.lent().map(Island::host)`). `HostRef::through` stores it (`rexx-api/src/ffi.rs:152`),
and `HostRef`'s `Deref`/`DerefMut` dereference it on the pool or foreign thread (`ffi.rs:200`,
`:212`). This is every callback a native call makes. The pointer is still dereferenced only by the
thread it is lent to, while the lender waits, so the grant is sound. The stated reason is not.

The second payload's sentence has a gap of the same kind. The `(Box<OffBaton>, ThreadContext)` is
"put back on the activity's record ... under the recall's lend". But when the record's frame no
longer matches (the call was abandoned while it ran), `record.back(call, ..)` never runs. `call`
then drops at the end of `PooledCall::run` (`dispatch/library.rs:1111`), and `ended`, the `Lent`, is
declared after it, so `ended` drops first: the box drops off the baton. That is sound only because
`OffBaton` holds no `Rc` or `Cell` (`Held` is an `Arc` plus an atomic count, and `CStringPool` holds
boxes and `ObjRef` words), and the comment does not say so.

Failure scenario: P72 asked for a SAFETY comment that gives each payload's argument so that a third
payload is weighed against it. Someone checking a change to `Island::host`, `Baton::lent` or
`HostRef` against this comment finds no mention of the path they are changing. If `OffBaton` gains an
`Rc`, as `ThreadContext` already has one, nothing in the comment flags the abandoned-call drop.

Fix: name both dereference sites (`Lent::interp`, and `HostRef` through `Island::host`, each on the
lendee while the lender waits), and give the box's argument as "no `Rc` or `Cell`; only the
`ThreadContext` needs the baton, and it is dropped under the recall's lend".

### 2. Important: `Heap::clear_uninit_all`'s registry walk counts as a touch (`rexx-core/src/heap.rs:332`, filter at `:346`)

The report says "The private `Heap::resolve` is not tagged, so the collector's own walks do not count
as touches". `collect` is resolve-only. But `clear_uninit_all` rebuilds the whole UNINIT registry with
`.filter(|&r| self.get(r).is_some_and(Object::has_uninit))`, and `get` is tagged. Every live object
in the registry is therefore marked as resolved by whichever activity calls `clear_uninit_all`:
`check_uninit` (`dispatch.rs:2123`), `run_ready_uninits` (`dispatch.rs:3432`), or the standard-stream
constructor (`dispatch/stream.rs:90`).

Probe (a scratch test in `concurrency_tests.rs`'s `sharing` module, `--features sharing`): main makes
100 objects of a class with `::method uninit`, starts a second activity whose method only does
`return 'ok'`, takes `~result`, then `say a~items`. The second activity never names those objects.

| variant | shared |
|---|---|
| as written at `3d0ab296b` | 102 |
| same, `:346` changed to a resolve-only filter (`self.resolve(r)` and a direct slot match) | 2 |
| no `~start` (the method called synchronously) | 0 |
| `~start` made before the 100 objects exist | 2 |

Over the corpus (`--release`), the recorded figure reproduces exactly (1088) at `3d0ab296b`. With the
resolve-only filter it is 1085. The three-object difference is in
`lang/reply_split_after_a_trap_at_the_reply.rex` (14 to 13),
`reply_split_after_a_trap_does_not_wait.rex` (1 to 0) and
`reply_split_after_a_trap_takes_the_failure.rex` (10 to 9).

Failure scenario: any program where one activity holds UNINIT-bearing objects (every `.Stream`, every
class with an `uninit` method) while another activity creates a stream, changes an `UNINIT` method,
or runs readied UNINITs. All of those objects are counted as shared without either activity
resolving them. The corpus effect is small, but the derived list was not re-run and ooTest makes
streams freely. Neither witness test would catch this. `one_activity_shares_nothing` has one
activity, so any walk tags with the same activity. `an_object_read_by_a_started_activity_is_shared`
asserts only `> 0`.

Fix: make the filter resolve-only (the shape the probe used), and add a witness: the 100-object
program above asserting `shared` below the number of UNINIT objects, which fails today.

### 3. Minor: the figure depends on the build profile

Same tree, same command without `--release` (the test profile has `debug_assertions` on): corpus
shared 1239, the same in two runs. With `--release` it is 1088, matching the record. Some
debug-only path resolves objects through the tagged accessors. With the registry walk removed, the
debug figure is 1236, so finding 2 is not the cause. The record's command does say `--release`, so
the committed figure reproduces. But the instrument counts reads the program does not make, and a
reader rerunning it with plain `cargo test` gets a figure about 14% higher.

Failure scenario: someone re-measures criterion 6 after a later change with the usual `cargo test
--features sharing` and reads the jump as a sharing regression. Fix: state in the record that the
figure is release-only, or find and exclude the debug-only reads.

### 4. Minor: `shared` is not split at the bootstrap boundary (`rexx-exec/src/lib.rs:318`)

`SharingReport` splits `objects` with `before_program` but gives `shared` as a single number. In the
corpus record, many rows have a program that made fewer objects than it shared, for example
`lang/method_reply.rex`: objects 281, made before the program 272, shared 9. So `shared` is mostly
or entirely bootstrap objects (classes, methods) that a started activity resolves. Neither
`shared / objects` nor `shared / (objects - before_program)` is the program's sharing fraction, and
the record cannot tell which one it is.

Failure scenario: criterion 6's figure is read as "a small fraction of program objects", but the
shared count's composition is unknown. Fix: count shared objects made before the program separately,
for example by recording the slot's `made` serial against `objects_before_program`.

### 5. Minor: `PROGRAMS` names two different lists in one usage block (`rust/bench-programs/wallclock.sh:4,7,11`)

The `-x PROGRAMS` placeholder means "programs also run on the oracle". The new environment variable
`PROGRAMS` means "the programs to run". The record's command passes the same list to both, which
works, but the usage text reads as if `-x` set the variable. Failure scenario: a user runs
`-x "pingpong/pingmsg"` without the environment variable, expecting only that program, and gets the
whole callgrind list with one oracle arm. Fix: rename the placeholder (`-x ORACLE_PROGRAMS`).

## Checked and holding

- A1 (`frame.rs:90`, `:99`, control `:108`): `cargo test -p rexx-core --doc RegFrame` runs 2 compile
  fail tests and 1 passing test, all ok. rustdoc checks the error codes, so each doctest fails for the
  stated reason.
- A2 (`body.rs`), A4 (`scheduler.rs`), A3 (`island.rs`): correct as written. A3's control (E0283 on a
  `Send` type) is shown in the report.
- A5: `IslandPayload` is crate-private and implemented for exactly the two instantiations, plus
  `#[cfg(test)] i32`. That impl is needed: `an_island_value_is_taken_only_on_the_baton` moves an
  `Islanded<i32>` into `thread::scope`. `Island::of` builds an `Islanded` without `new`'s assertion,
  but it did so before this change, and the payload is listed.
- Sharing semantics: tags switch in `switch_to` and `swap_running`. `serve_recall` swaps the calling
  activity in before it lends (`scheduler.rs:1342`), so a pool thread's heap reads are tagged with the
  call's own activity. The main activity's tag (0) matches the heap's starting tag, and main keeps its
  tag after a `~start`/`~result` round trip (probe: 100 objects re-read by main after the round trip,
  shared 2). `made` covers both slot-creating paths of `alloc_with_uncollected`. `collect` uses only
  `resolve`.
- Zero cost when off: the `.text` record's method is sound (a determinism control, plus the A-only
  tree matching head). After `0ac73b804` the only Rust change is a comment in `island.rs`.
- Ping-pong: a scratch variant of `pingsem` and `pingguard` that counts turn violations (with a delay
  inside the player's turn) prints 0 over 2000 handoffs each, so the waits really block and hand off.
  `pingmsg` is a `~start` plus `~result` per iteration, as the README says. Both sides ran (record
  `pingpong/binaries.txt`, `table.txt`).
- `wallclock.sh`: `prog()`, the output file names (`/` spelled `_`), the `-x` match and the stdout
  comparison all work with `pingpong/` names.
- `refusal-sites.tsv`: with the definition column cut, base and head are byte-identical
  (`cut -f1-3,5-` then `cmp`). Every changed row is a `lib.rs` site, and every `lib.rs` row's line
  number at head is the line of its `fn NAME`.
- Rules: the only new `unsafe` is the grant in `island.rs`. There are no new dependencies (features
  only), no new mutable statics (the counter lives in `Heap`), and no em-dashes in added prose.
