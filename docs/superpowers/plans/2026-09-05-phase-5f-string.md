# Phase 5f — `String`'s 112 loud instance rows

Spec: `docs/superpowers/specs/2026-09-05-phase-5f-string.md`. Read it first; this plan does not
restate its measurements.

**This is 5c's unfinished business, not a new phase**, exactly as the `MutableBuffer` follow-up was.
Moritz answered D86 on 2026-09-05: change no committed table. `class-set.txt:57` keeps saying `5c`
for `String`, `CLOSED_PHASES` stays `["5a","5b","5c","5d"]`, and `5f` is these documents' name
rather than a value anything reads. He answered D85 the same day: **all 112 rows, operator rows
included**, reopening 5e's Task 3 exclusion deliberately.

**`String`'s receiver needs no `RECEIVER_OVERRIDES` entry, measured.** The 5c follow-up's review
left open whether the documented receivers make rows unfalsifiable and named `String` among the
classes to check. Task 0 ran it: sending every documented instance name with no arguments to
`.String~new('abc')` and to `.String~new('')` and diffing, **21 of the 112 rows answer differently**
-- `b2x bitAnd bitOr bitXor c2d c2x decodeBase64 encodeBase64 hashCode length lower makeString
reverse space strip translate upper words x2b x2c x2d`. The documented receiver already has teeth.

**The open question's answer for the empty collections is smaller than it looks**, and Task 0
reports it: a populated receiver would sharpen **Queue 10 of 43, Array 11 of 44, List 7 of 38** rows.
The rest cannot discriminate at any receiver, because a zero-argument send to a method that needs
arguments never reaches the receiver's contents. Whether those thirty-odd rows are worth an override
each is not this phase's call.

BASE for Task 0 is the commit this plan lands in.

---

## Global constraints

## Global constraints

Every task, no exceptions.

* **Commit before the full gate run, not after**, per `rust/CLAUDE.md`'s Gates section: fast checks
  yourself, commit code and report together with the report's gate cells left as `**G1**`–`**G7**`,
  start the suite in the background with the commit sha as the status file's first line and a
  pidfile beside it, report `committed at <sha>, gates running, statuses at <path>` and **stop**.
  The tree belongs to the gate run until its status file says `finished`.
* **Every claim gets a red control**, predicted before it is run and marked confirmed, falsified or
  unobservable. A control that varies something *about* the change proves nothing.
* **Both engines every time**: `REXX_ENGINE=ir` and `REXX_ENGINE=tree-walker` — that exact spelling.
* Three separate descriptors, never `2>&1`, `$?` captured immediately.
* Oracle runs from a fresh empty directory, wrapped as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
* **Never `git checkout -- <path>`** on a file you edited; `cp` to your scratch directory and
  restore from the copy, then `touch` it — `cp -a` preserves mtime and cargo will keep the stale
  binary.
* No `unsafe`; the workspace lint is `deny` and it is Moritz's call per site.
* Never `git add -A`; never amend; `git commit -F <file>` with paths named; confirm `Cargo.lock` is
  not staged unless the task deliberately changed it.
* **A method that exists and does nothing is not implemented** — `rust/CLAUDE.md`, and this phase
  exists because of it. Do not add a name that answers nothing to close a row.
* **The corpus control that can go red is `REXX_CORPUS_GATE=1`.** The plain `--test corpus` binary
  is report mode — `corpus_differential` asserts `!gate || mismatches.is_empty()` — so it exits 0 on
  a divergence and prints `N of M matching` to stderr. Measured by Task 3a: M1 exited 0 at `358 of
  359`. Cite the STRICT run and the matching line, never the plain exit.
* **Mutation runs build with `--profile mutation`** (release plus thin LTO, own `target/mutation/`),
  so the release binary the witnesses run on is never a mutant's.
* **The 5c follow-up's gate ruling is inherited, not re-derived.** "Every row moves `loud` →
  `answers`" is retired: its plan review built the stub that satisfies it. The gate is the corpus
  programs. `method-bodies.txt` is a drift check — no `String` row may move to `diverge` — and the
  `loud` → `answers` direction is reported, never required.
* **A method and its like-named builtin agree on the answer and on nothing else.** Measured on
  `left`: 93.903/rc 163 against 40.3/rc 216 for a missing argument, 93.923 against 40.12 for a bad
  one, and the argument *numbering* shifts because the receiver is the builtin's first argument.
  Reuse the value computation; never the builtin's argument handling.
* **Every family's witness sends a short argument list as well as a good one.** The shared
  missing-argument raiser makes those rows agree in `method-bodies.txt` for free, so nothing but a
  corpus program witnesses a wrong error number.
* **Every family owes four bookkeeping steps, and Task 1a paid all four late.** Its fast checks were
  `fmt`, `clippy`, `--test corpus` and `--test coverage`, and G3 came back rc 101 on three suites
  none of them runs. Each family must, in the same commit as its code:
  1. file both witnesses in `corpus/phase-5c.txt` **and** `EXPECTED_SUBSET_5C`, which that
     assertion requires together;
  2. generate a `crates/rexx-parse/tests/sourceline_oracle/<name>.txt` per new corpus program, with
     the driver in that module's own comment;
  3. refresh `corpus/method-bodies.txt` under `REXX_METHOD_BODIES_REFRESH=1` for the rows it moved;
  4. add its file to `dispatch_seam.rs`'s `CLEARANCE_CONSUMERS` if it is a new one -- it will not be,
     while every family lands in `dispatch/string.rs`.

  **And the fast checks are `cargo test --release --workspace --no-fail-fast`, not a chosen subset.**
  It is 12 minutes and it is the whole of what G3 runs.

---

## Task 0 — the module, the wiring, and the receiver check

**Lands alone, ahead of every other task, and adds no method.**

`dispatch.rs` is 11,882 lines and this phase adds 112 bodies to it. Create
`crates/rexx-exec/src/dispatch/string.rs` beside the existing `dispatch/native.rs`, holding its own
`NATIVE_METHODS` slice, and chain it into the registration loop at `dispatch.rs:1188`, which already
reads `NATIVE_METHODS.iter().chain(extra)`. **Land it with the slice empty**: no behaviour changes,
and every later task adds rows to one file that is nobody else's.

**Red control, and an empty slice cannot be its own witness.** Two steps, because they prove
different things and the cheap one alone does not prove enough:

* a row naming a method the class does not answer must **panic** at `ObjectModel::build` with
  `NATIVE_METHODS names String~ZZZPROBE, which that class's behaviour does not answer` — that is the
  loop reading the slice, and it is the reason the row cannot be a made-up name;
* a row binding a real loud name to the wrong body — `("String", "LOWER", Arity::Fixed(2),
  native_reverse)` — must make `'abc'~lower` answer `cba` on both engines. That is the natives map
  populated from the new slice, and the send reaching it.

Remove both and confirm `'abc'~lower` is loud again. A chain that silently registers nothing looks
exactly like a chain that works, until Task 1 blames its own code for it.

**Also measured here, and reported whether or not it is acted on**: run every `covered` class's
documented receiver against its own zero-argument probe and say how many rows are unfalsifiable
because the receiver is empty. The 5c follow-up review left this open as its question 2 and named
`String`, `Queue`, `Array`, `Stem`, `List`, `CircularQueue`. One loop.

---

## Task 1 — the 41 rows `MutableBuffer` already implements

The cheapest block and the largest. `native_mutable_buffer_*` already wrote both halves against the
93.9xx surface this phase needs: the argument layer and the call into `builtin::string`/`word`'s byte
cores.

**41, not 40, and `[]` is the row that hid.** The intersection is computed from
`corpus/method-bodies.txt` — `MutableBuffer`'s `answers` names against `String`'s `loud` names — and a
first pass took it from the *function* names in `dispatch.rs` instead, where the method `[]` is spelt
`native_mutable_buffer_brackets` and so fell out of the intersection and into Task 2's operator
bucket. Task 2 is 33.

**Two contracts, not one, and only 30 rows share a body.** Measured on the oracle:

```text
'abcdef'~insert('XY', 2)                    abXYcdef, and the receiver is still abcdef
.MutableBuffer~new('abcdef')~insert('XY',2) returns the receiver; the buffer is abXYcdef after
```

The same holds for `append changeStr delStr replaceAt`, checked one by one. So:

* **30 reader rows** — `[] pos lastPos verify word words countStr contains containsWord startsWith
  endsWith match matchChar subChar substr subWord subWords wordIndex wordLength wordPos`, and the
  eleven `caseless*` — differ from their `MutableBuffer` twin **only in the byte source**:
  `buffer_state(interp, receiver, b"SUBSTR")?.bytes` there, `interp.to_text(receiver)` here.
* **11 mutator rows** — `append caselessChangeStr changeStr delStr delWord insert lower overlay
  replaceAt space translate`, the ones whose `MutableBuffer` native reaches `buffer_state_mut` —
  share the core and the argument layer and **not** the return contract. `String` builds a new value;
  the buffer writes itself and answers the receiver.

**Extract the shared half rather than copying it.** For the 30 that is a whole body parameterised by
its byte source. For the 11 it is the argument layer and the core call, with two endings. Copying
gives 41 chances for the two receivers to drift apart on an error message, and the corpus programs
would have to catch every one.

**The borrow decides the shape and is worth stating before anyone fights it.** `to_text` is
`&mut Interp` where `buffer_state` is `&Interp`, and `take_result_buffer` is `&self` with interior
mutability. So a shared body takes the result buffer *first*, then the byte source, and drops the
byte borrow before `text_built`. Resolving the bytes into an owned `Vec` instead would put an
allocation on every send, which is what entry 75 has just finished taking out of this path.

**Several commits, each its own family, each its own gate run, each filing its own witness** — the
5c follow-up's Task 3 shape, for its reason: one program exercising eight names says nothing about
the other thirty-two.

**Risk this task owns.** These bodies have been tested once, under one receiver. A divergence in the
extracted half now reddens `MutableBuffer`'s witnesses too, which is the instrument working; a
divergence introduced *by* the extraction reads as a `String` defect. Run
`corpus/lang/mutablebuffer_*.rex` before and after the extraction commit and say they did not move.

---

## Task 2 — the 33 operator rows

D85 put them in scope; D83 is the open shape question and it is a question about *stderr*.

`eval::apply_binary(op, left, right)` (`eval.rs:1872`) already computes every one, and
`operator_argument(args)` (`dispatch.rs:4666`) is already the argument layer `Object`'s operator
natives use. **Two shapes** — one native per name, or one native carrying its `Operator` in the table
row — **and the traceback decides between them, not taste.**

Measured on the oracle, an operator sent as a message prints one line the expression form does not:

```text
$ say 'abc' + 1                    $ say 'abc'~"+"(1)
                                          *-* Compiled method "+" with scope "String".
   1 *-* say 'abc' + 1                1 *-* say 'abc'~"+"(1)
Error 41.1 ... rc 215              identical, rc 215
```

Measured on this crate, the frame already appears for a raising native send, byte-identical to the
oracle on both engines (`b~substr` on a `MutableBuffer`). **So the machinery exists and the question
is only whether a route through `apply_binary` stays inside it.** Run that before choosing.

**The witness must make each of the 33 raise as well as succeed**, because the extra line appears
only on a failing operand and a program of successful sends cannot see it.

`+ - * ** / // % = == \= \== < <= << <<= <> > >= >< >> >>= \< \<< \> \>> \ & && | || ?`, abuttal and
blank. `?` is not arithmetic and gets its own reading. **`[]` is not here** — it is Task 1's, because
`MutableBuffer` already implements it.

**Six of the 33 crash the oracle, and this task decides what the table does about it.** Measured by
Task 0 and recorded as `corpus/oracle-crashes.txt`'s newest entry: `say "abc"~"<<"` is a silent
SIGSEGV at rc 139, and so are `<<=`, `>>`, `>>=`, `\<<` and `\>>`, because
`RexxString::primitiveStrictComp` dereferences a missing argument with no check
(`classes/StringClass.cpp:920`-`:923`) where the other twelve comparison operators go through
`RexxString::comp`'s `requiredArgument` (`:762`). The rows are `loud` today and reach that verdict in
`method_bodies.rs`'s crate-only pass, so nothing runs the oracle on them; **the commit that binds any
of the six sends the refresh to the oracle and turns the row into a `Structural` failure** -- "the
oracle did not finish" -- which is red and correct and still red. Decide how the table carries a row
whose oracle answer cannot be obtained before writing the bodies, not after.

---

## Task 3 — the 25 rows with a builtin behind them

`abbrev abs b2x bitAnd bitOr bitXor c2d c2x center centre compare copies d2c d2x dataType format left
max min right strip trunc x2b x2c x2d`.

**The builtin computes the value and none of the error surface**, per the global constraint. For the
names Task 3a of the 5c follow-up already lifted into `&[u8]` cores, this is an argument layer over an
existing function. For the rest — the numeric and base-conversion family especially — the computation
still sits inside the builtin entry with 40.x handling baked in, and lifting it out is part of the
work. **Say which of the 25 needed a lift and which did not**; that number is what tells the next
phase whether the collections are the same shape.

The builtin's own tests are the witness for a lift; the corpus program is the witness for the method.

---

## Task 4 — the 13 rows with nothing behind them

`caselessAbbrev caselessCompare caselessCompareTo caselessEquals ceiling compareTo decodeBase64
encodeBase64 equals floor hashCode modulo round`.

**`hashCode` is `Object`'s, not `String`'s.** Its evidence column reads `method "HASHCODE" of class
"Object"`, so it is inherited, and implementing it moves rows on every class that inherits it.
Land it in its own commit, name the classes whose rows move, and read them before and after.

`ceiling floor round modulo` are the numeric group and want `NUMERIC DIGITS` read from the current
setting rather than assumed; `decodeBase64`/`encodeBase64` want a non-ASCII case in the witness.

---

## Task 5 — file the witnesses, and close

**No `CLOSED_PHASES` change, no new subset file, no `class-set.txt` edit** — D86. The witnesses are
filed by the tasks that wrote them; this task confirms `corpus/unfiled.txt` carries none of them,
refreshes `corpus/method-bodies.txt` under `REXX_METHOD_BODIES_REFRESH=1`, and reports the before and
after for every class that moved — not only `String`, because Task 1's extraction and Task 4's
`hashCode` both reach further.

**Enumerate from the tree, not from this plan.**

---

## The gate

**The corpus programs pass, and no `String` row moves to `diverge`.** Each family's witness agrees
with the oracle byte for byte on all three descriptors, on both engines, sending a short argument
list as well as a good one.

**Shown to fail**: replace one landed body with a stub that raises 93.903 and nothing else. Its
corpus program must go red. If it stays green that row is unwitnessed however `method-bodies.txt`
reads, because the table will say `answers` either way.

---

## Open for a second review, if one is taken

1. **Is Task 1's extraction one commit or one per family?** The plan says several commits; whether
   the *extraction* is separable from the first family's binding is the implementer's measurement.
2. **Does `Object~hashCode` belong in this phase at all?** It is one of the 13 by the table's
   arithmetic and it is not `String`'s by ownership.
3. **`String~verify` and the past-the-end guard.** `MutableBuffer~verify` answers `counted` on every
   path and is the only caller reaching `verify_bytes`'s start-past-the-end guard; whether the
   builtin follows is open (`task-3a-report.md` §6.1), and `String~verify` is now a second caller.
