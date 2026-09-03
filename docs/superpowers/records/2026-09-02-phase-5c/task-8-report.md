# Task 8 -- the two dark benchmark axes, and `identityHash`

**BASE** `a7b72abea`. **Landed at `15fc08c72`.** Every figure below was measured on 2026-09-03. No gate row count moved:
table C's 5c count is **110** and table D's is **2**, both read from the harness's own report on
the committed tree.

---

## 1. What was actually blocking each axis, re-measured at BASE

The brief said to re-measure rather than assume, and one of the two blockers named in the plan was
already gone.

```
alloc.rex      rc 120  method "OF" of class "Array" is not implemented (Phase 5)
heapshape.rex  rc 120  a message send to a value that is not a hash collection is not implemented
```

Both engines, identical. `.String~new` had landed with Task 2, so `alloc.rex`'s only remaining
blocker was `Array~of`. `heapshape.rex` was **not** blocked on `.Directory~new` any more either --
that constructs -- it was blocked one step later, on the *write*: `d = .Directory~new` then
`d['X'] = 'v'`. `.Directory~new` answered a `Body::Instance`, and `Interp::hash_entry_write` needs a
`Body::Native`.

## 2. `Array~of`

One row in `NATIVE_CLASS_METHODS` (`AddClassMethod("Of", ArrayClass::ofRexx, A_COUNT)`,
`memory/Setup.cpp:709`) and one function. The body is the arguments as slots; `INIT` is sent with no
arguments, which is what `completeNewObject(newArray)` does.

Two edges decide whether it is a real `~of` or a `~new` that took the argument count, and both are
the oracle's, measured on three descriptors:

| program | oracle | crate, both engines |
|---|---|---|
| `.array~of(1,2,3)~size ~items ~dimension` | `3 3 1` | same |
| `.array~of()~size ~items ~dimension` | `0 0 1` | same |
| `.array~of(1,,3)~size ~items` | `3 2` | same |
| `.array~of(1,2,)~size ~items` | `2 2` | same |
| `.array~of(4,5)[2]`, `.array~of('x')~class~id` | `5 Array` | same |
| `.array~of(7,8)` then `a[3] = 9`, `~size ~toString('l',' ')` | `3 7 8 9` | same |

An interior omission is a slot with no item; a **trailing** one is not an argument at all. An empty
argument list fixes the shape (`~dimension` `1`) where `.array~new()` leaves it open (`0`) --
`ArrayClass::ArrayClass(objs, count)` (`classes/ArrayClass.cpp:314`) gives a zero-length array a
one-entry dimension array and `newRexx` only does so for an explicit `0`.

`~of` on a subclass refuses, which is the position `native_array_new` was already in: measured,
`.array~subclass('K')~of(1,2)~size` is `2` on the oracle and rc 120 here,
`method "OF" of class "K" is not implemented (Phase 5)`. `array_of_on_a_subclass_is_loud` holds it.

`alloc.rex` is then rc 0 on both engines with stdout `21000000` and empty stderr -- byte-identical
to the oracle on all three descriptors.

## 3. `Directory~new`

`.Directory~new` now answers the same hash body `.StringTable~new` does. That is one table row
repointed and one wrapper; `native_string_table_new` is renamed `native_hash_collection_new` because
`DirectoryClass::newRexx` (`memory/Setup.cpp:928`) is the same shape as `StringTable::newRexx`.

Measured, oracle and both engines byte-identical on three descriptors:

```
d = .Directory~new
say d['X'] d~at('X') d~zork      ->  The NIL object The NIL object The NIL object
d~put('v','X')  ; say ...        ->  v v The NIL object
d['Y'] = 'w'    ; say ...        ->  w w The NIL object
d~zork = 'z'    ; say ...        ->  z z z
d2 = .Directory~new ; say ...    ->  The NIL object The NIL object
```

The last line is the control -- a second directory does not see the first one's entries -- and the
`d~zork` column is the entry-method fold: `~zork` reads and writes under the upper-cased name where
`~at` and `~put` do not.

`a_new_directory_reads_nil_for_every_index_and_refuses_every_write` asserted the old refusal and is
replaced by `a_new_directory_reads_nil_until_an_entry_is_put_there`, which asserts the whole
sequence above.

### The subclass split, which is where I stopped

Repointing the row naively gives a `Directory` **subclass** a `Body::Native` too, and that regresses
two things a plain instance has. Measured on the oracle against a build that did exactly that:

| program | oracle | native body | committed |
|---|---|---|---|
| `::class K subclass Directory`, `::method init` with `expose n` | `K 1 5` | rc 120 `EXPOSE on an object with no variable pool` | `K 1 5` |
| `::class K subclass Directory`, `::method uninit` printing | `after` then `uninit ran` | `after` only, rc 0 | both lines |
| `.Directory~subclass('K')~new` then `o['A'] = 1` | `1` | `1` | rc 120, not a hash collection |

The third row is what the split costs and the second is why it is worth paying: a dropped `UNINIT`
is rc 0 with a line missing, the silent class. `native_directory_new` therefore sends `.Directory`
itself to the hash constructor and a subclass to `native_new`, and
`a_directory_subclass_keeps_the_instance` holds the pair so that unifying them turns one row red and
the other green rather than one alone.

`receiver_kind`'s `Directory` arm stays on class identity rather than moving to descent, because
with this split nothing builds a `Body::Native` of a subclass. `StringTable`'s subclass path is
untouched -- it was already a native body and already matches the oracle
(`.StringTable~subclass('K')~new` then `o['A'] = 1` answers `K 1 1` on both).

**Handover:** a `Directory` subclass cannot hold entries here. Closing it needs a variable pool and
an `UNINIT` registration on `Body::Native`, or a `Directory` model that is neither. It has no owner.

## 4. The finding: the heap-shape axis was measuring a graph a thousand times too small

`heapshape.rex` built its million slot values with `"e" || j`. A string of up to
`rexx_core::INLINE_TEXT` (7) bytes lives **in the handle** and allocates no heap object at all, so
this crate built a ~1,001-object graph where the oracle built ~1,001,001. The program's own header
already warned about the *interning* form of this trap ("a bare literal would be one interned object
shared by all 1M slots"); the handle is the same trap by a second route, and the header's warning
does not reach it.

It is directly measurable rather than inferred. `("e"||j)~identityHash == ("e"||j)~identityHash` is
`1` here and `0` on the oracle; at eight bytes both answer `0`. And on a program that builds
`heapshape.rex`'s graph with `"e" || j` and reads it back but does not collect, `/usr/bin/time -v`
reports a maximum resident set of **32,780 kB here against 219,816 kB** for the oracle, with both
sides printing the same readbacks.

What it did to the figure, 9 interleaved pairs per variant, oracle first in each pair, same binary:

| slot strings | oracle `gc_pause_seconds` | this crate | ratio |
|---|---:|---:|---:|
| `"e" \|\| j` (<= 7 bytes) | 0.016945 | 0.002113 | 0.12x |
| `"element-" \|\| j` (>= 9 bytes) | 0.017411 | 0.014672 | 0.84x |

**The oracle's own figure barely moves**, which is what identifies whose graph changed. Reported
without this, the axis would have said the collector is eight times faster than the oracle's; it is
about 19% faster on the same graph.

`heapshape.rex`'s slot strings and directory keys are therefore widened past seven bytes, which is
the program's own stated contract ("a ~1M-object graph") rather than a new one. `d1-decision.md`'s
oracle figures survive the change: its median was 17.966 ms and the widened program measures
0.016761-0.018629 s on the oracle. `phase-4d-gate.md`'s criterion 6 asks for "both arms building the
same object count", and this is the first run in which the **interpreter** arm meets it -- the arm
that criterion was checked against is `crates/rexx-core/benches/heap.rs`, which allocates
`Body::Text` directly and was never affected.

`alloc.rex` is **left alone**, deliberately. It has the same asymmetry -- measured,
`.string~new("item")~identityHash == .string~new("item")~identityHash` is `0` on the oracle and `1`
here, so the oracle allocates a `RexxString` per iteration this crate does not, plus a `RexxInteger`
per arithmetic result past its small-integer cache: measured,
`((i+1)~identityHash == (i+1)~identityHash)` is `0` on the oracle at `i = 2999999` and `1` at
`i = 7`, and `1` here at both, so the oracle's cache rather than a shared representation is what
makes the small case agree. But `alloc.rex`'s own contract is "enough short-lived objects
per iteration to force collections", and 3,000,000 fresh arrays meet it on both sides.
`heapshape.rex`'s contract was not met, and a full collection over 1,001 objects is insensitive to
anything this phase could have broken.

## 5. The suite: two roles moved, and the role test now asserts both directions

`rexx-bench-suite` declared both axes `Role::Blocked`, and
`every_blocked_axis_still_fails_on_this_crate` checks that claim by running them -- so the roles had
to move in the same change or gates 3-5 go red. Confirmed as a negative control: putting `heapshape`
back to `Role::Blocked` fails with *"heapshape is declared Role::Blocked but exited 0"*, and
restoring it passes.

* `alloc` -> `Role::Loop`. It has a loop bound and the harness times it like any other axis.
* `heapshape` -> **`Role::SelfTimed`**, new. It is run interleaved on both sides exactly like a loop
  axis, and reported from the `label= value` figures the program prints, because timing the process
  measures the sum of the parts the program separates -- which is the reason it is in
  `NOT_BENCHMARKED` and why `Role::Loop` would be the wrong answer.
* The role test is renamed `every_declared_runnability_still_holds` and asserts a blocked axis fails
  **and** a self-timed axis runs. The old one asserted `!blocked.is_empty()`; with both axes moved,
  `Role::Blocked` is empty and that test would have panicked on its own non-vacuity guard. Checking
  both directions is what keeps it saying something now that nothing is blocked.
* `write_blocked` prints "None: every axis in the list runs on both sides" rather than an empty
  table.

Verified end to end with `./target/release/rexx-bench-suite --self-check`, exit 0, binary sha256
`8899d29141f1f16644f274e083d3411c719ddb90149a829f25413cb9792e345c`:

```
### Axes that report their own figures

`heapshape`, 1 interleaved pair(s). Every number below is the median of what the program itself
printed; this harness timed nothing here, because the process wall time is the sum of parts the
program separates.

| figure | oracle | this crate | this crate / oracle | runs (oracle, crate) |
|---|---:|---:|---:|---:|
| `build_seconds` | 0.208038 | 0.356586 | 1.714x | 1, 1 |
| `gc_forced` | 1.000000 | 1.000000 | 1.000x | 1, 1 |
| `gc_pause_seconds` | 0.017771 | 0.014677 | 0.826x | 1, 1 |

### Axes this crate cannot run

None: every axis in the list runs on both sides.
```

The same run puts `alloc` in the axis table at `1.1904 s` against `3.6468 s`, **3.06x SLOWER**, with
its output checked identical on both sides (`21000000`) and `instructions:u` 13,280,211,248 against
42,824,240,996 -- **3.22x**. One pair, so it is a wiring check and a second reading rather than the
figure; section 6 is the figure.

## 6. The figures at head

`rexx-run` sha256 **`983ed01bf5bbb25d77981f05ab81d9b9462d5f7ce63323453fbff207b2ce1495`**, read at
the time of the run; oracle `bin/rexx` sha256 `bb5bb8ccbb96c376e329b91aafdad891f975ba06c941dbceba82c3848fa13019`,
`lib/librexx.so.4` sha256 `42136c4038004fe2d5104181e06873301f032ced97c54a0fe84042e006d9b6fb`.
9 interleaved pairs, oracle first in each pair, one discarded warm pair, `ir` arm, machine otherwise
idle (load average 2.2 at the start).

| axis | figure | oracle median [min-max] | this crate median [min-max] | crate / oracle |
|---|---|---:|---:|---:|
| `alloc` | wall seconds | 1.14 [1.11-1.16] | 3.64 [3.61-3.66] | **3.19x** |
| `heapshape` | `build_seconds` | 0.20786 [0.20675-0.21665] | 0.355455 [0.351882-0.35917] | **1.71x** |
| `heapshape` | `gc_forced` | 1 | 1 | 1.00x |
| `heapshape` | `gc_pause_seconds` | 0.017411 [0.016761-0.018629] | 0.014672 [0.014208-0.014866] | **0.84x** |

Both engines, 5 interleaved pairs, `ir` first in each pair: `alloc` is 3.63 s on `ir` against 3.90 s
on `tree-walker` (1.07x), and `heapshape`'s two engines are indistinguishable -- the work is
allocation and collection, not clause dispatch.

**There is no pin-relative figure**, as the plan says: `rexx-run-f558ea501` refuses both programs and
`bench-baselines/phase-5b-arms.tsv` carries neither axis. Nothing was compared against the pin.

## 7. `identityHash`: a licensed divergence, committed with the measurement

The oracle's answer is `((uintptr_t)this) ^ UINTPTR_MAX` (`classes/ObjectClass.hpp:340`), handed to
`new_integer` and rendered as a signed decimal -- an address, bitwise-inverted.

Measured, ten runs of `say .Object~new~identityHash`:

```
-139691328307761  -139867903783473  -139807698133553  -140273322500657  -140519509881393
-139704636834353  -140025509478961  -140280459719217  -139848050573873  -140126764171825
```

Ten runs, ten values. **The oracle does not reproduce its own answer across runs**, so there is no
value to match. Its rendering is a `-` and fifteen digits: 16 characters in 20 of 20 runs. Matching
*that* would mean pinning this machine's mmap address range (`0x7f...`) into the crate, which is a
machine property and not a language one -- a guess that looks right, which the brief names as the
wrong outcome.

**Conclusion: licensed divergence, not a match.** It is deviation 4 (D41) read at this message, and
`native_identity_hash`'s doc now carries the C++ expression, the ten-run spread and the reason the
width is not matched either.

What is asserted instead is everything the licence leaves standing, and every row is the oracle's,
measured on three descriptors, byte-identical on both engines:

```
numeric digits 20                                                 -> (needed: the answers exceed 9 digits)
datatype(<stem>~identityHash,'W') datatype('abc'~identityHash,'W')      1 1
datatype(.Object~new / .Array~new / .StringTable~new / .Directory~new)  1 1 1 1
a = .Object~new ; b = .Object~new
(a~identityHash == a~identityHash) (a~identityHash == b~identityHash)   1 0
```

That covers the receiver kinds the three tasks that hit this reported it on --  Task 2's
constructors, Task 5's sweep, Task 7's stem -- and a build answering a per-receiver constant would
pass the rendering and fail the last row.

**The identity half of deviation 4 now has a witness of its own**, found while measuring the above
and asserted by `two_equal_inline_strings_share_one_handle`:

```
j = 5
say (("eeeeee"||j)~identityHash == ("eeeeee"||j)~identityHash)   crate 1   oracle 0
say (("eeeeeee"||j)~identityHash == ("eeeeeee"||j)~identityHash) crate 0   oracle 0
```

Seven bytes is one handle here and two objects there; eight bytes is two objects on both. The second
row is the boundary control, which is what pins the divergence to the inline case rather than to
`~identityHash` at large. This is the same mechanism as section 4's, reached from the other end.

## 8. Gates

Five gates from `rust/`, statuses read unpiped from files written by the runner, on the tree as
committed. Working-tree fingerprint (`git status --porcelain` plus `git diff HEAD -- rust/`, sha256)
recorded before the first gate and after the last, and identical -- so no other agent's edit landed
inside the window.

| gate | command | status |
|---|---|---|
| 1 | `cargo fmt --all --check` | 0 |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| 3 | `cargo test --release --workspace --no-fail-fast` | 0 -- 2025 passed, 0 failed, 109 binaries |
| 4 | `REXX_CORPUS_GATE=1 cargo test --release --workspace --no-fail-fast` | 0 -- 2025 passed, 0 failed, 109 binaries |
| 5 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | 0 -- 2026 passed, 0 failed, 109 binaries |

Gate 5 runs one test more than 3 and 4 over the same 109 binaries, and it is the expected one: diffing the two test-name lists, the extra is `bytes::tests::the_bytes_past_len_are_never_part_of_the_value`, which is `cfg(debug_assertions)`.

`corpus/refusal-sites.tsv` needed no re-derivation and that was checked rather than assumed: it
cites line numbers in `error.rs`, `lib.rs`, `run.rs`, `trace.rs` and `dispatch/native.rs` and none in
`dispatch.rs`, this change adds no `Loud`/`Raised` constructor (`unbuilt_class_method` returns
`Failure`), and `crates/rexx-exec/tests/refusal_sites.rs` re-derives its first four columns on every
run and passed in all three gates.

Corpus differential under the gate: **331 of 331 matching**, STRICT, unchanged from BASE.
Table C: `5c: 1347 rows, 110 not yet agree`. Table D: `5c: 38 rows, 2 not yet agree`.
`directive_options_trace_reply.rex` appears in no failure line in any of the three runs -- the
licensed flake did not fire.

## 9. What the commit holds, and what it deliberately leaves alone

`15fc08c72` -- `rust/bench-programs/heapshape.rex`, `rust/bench-programs/README.md`,
`rust/crates/rexx-exec/src/dispatch.rs`, `rust/crates/rexx-bench/src/bin/rexx-bench-suite.rs`, and
the Task 8 block in `docs/superpowers/plans/2026-09-02-phase-5c.md`. `Cargo.lock` is not staged.

Another agent was editing `docs/superpowers/plans/2026-09-02-phase-5c.md`,
`docs/superpowers/plans/phase-4-exclusions.txt` and
`rust/crates/rexx-exec/tests/directive_options.rs` in this worktree at the same time (Deviation 7,
the `directive_options` flake). Their hunk of the shared plan file was kept out of this commit by
staging a reconstructed blob through `git hash-object` + `git update-index --cacheinfo`, which never
touches the working copy; verified afterwards that their text is still in the worktree, still
unstaged, and absent from the commit. Their two other files are untouched and unstaged.

The `rust/` working-tree fingerprint was identical before the first gate and after the last, so
nothing of theirs landed inside the gate window, and every `rust/` path I gated is committed byte
for byte.

## 10. What I did not do

* **`heapshape.rex` cannot be byte-identical to the oracle and never will be.** Two of its three
  output lines are `TIME()` figures. The axis agrees on exit status, on stderr and on `gc_forced= 1`;
  the other two lines *are* the measurement. The brief's "output matches the oracle on three
  descriptors" is met for `alloc.rex` in full and for `heapshape.rex` on everything that is not a
  clock.
* **No corpus program was added.** `alloc.rex` is byte-identical to the oracle and would make a
  clean differential row, but adding one moves the corpus headline and needs a
  `sourceline_oracle/*.txt` generated through the `.Package~new` exception. Not asked for, not done.
* **`Array~of` on a subclass refuses**, where the oracle answers. Same gap as `Array~new`, no new
  owner.
* **A `Directory` subclass cannot hold entries**, section 3. Handover, no owner.
* **`Directory`'s remaining method bodies are untouched** -- `~items`, `~hasIndex`, `~remove`,
  `~setMethod`, `~unsetMethod` and the rest are still rc 120. `heapshape.rex` needs only `[]` and
  `[]=`, and method bodies are out of this phase.
* **`alloc4c.rex` was not examined for the inline-handle trap.** It is a pinned axis with a measured
  history and changing it would invalidate `PINNED.md`; whether its concatenated string crosses seven
  bytes is unchecked.
* **No full `rexx-bench-suite` run.** The `--self-check` run proves the plumbing; the committed
  figures come from a purpose-built interleaved script rather than from a full 9-pair suite report,
  which would rewrite `perf-baseline.md`'s territory and was not asked for.
* **`reqstr_armed` and the abstract check are not applied to the hash constructor.** `native_new`
  does both and `native_hash_collection_new` does neither; that was true for `StringTable` before
  this change and I did not measure what either is worth, so I left it rather than change behaviour
  I had no oracle row for.
