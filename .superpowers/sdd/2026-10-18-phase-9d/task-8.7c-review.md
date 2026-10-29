# Phase 9d Task 8.7c review

Reviewer t87c-review. Code BASE `97eee8a9a`, code HEAD `e620097f0`; records to `5be253e4f`.

**Spec compliance: COMPLIANT.** **Quality: CHANGES REQUESTED** (Critical 0, Important 2, Minor 3).

## What I ran

* Built `97eee8a9a` and `e620097f0` from `git archive` trees (`rust`, `interpreter`, `extensions`) with
  `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx` into
  `/tmp/claude-1000/t87c-review-target-base` and `-head`, exit 0 both. Each binary's `.text`
  (`objcopy -O binary -j .text`) is byte-identical to the pinned `cg-9d/t8.7c/rexx-base-97eee8a9a`
  and `rexx-head-e620097f0`. Every head `.vg` in `cg-9d/t8.7c/final2` names
  `rexx-head-e620097f0` and every base `.vg` `rexx-base-97eee8a9a` (`grep Command:`, 99 each).
* Probes three ways (oracle under the standard wrapper, base, head under `memcap 2G timeout -k 5 20`)
  from fresh directories under `t87c-review-scratch/` with stdout, stderr and rc compared separately
  (`t87c-review-scratch/bin/r3.sh`). Crash-shaped NOTREADY and debug-input programs (entries 66 and
  67's shapes, `dp2`/`dp3`) on the crate only.
* `cargo test --release -p rexx-exec --lib` at HEAD from the archive tree: 1139 passed, 68 failed,
  every failure environmental (the archive has no `../../../build/lib`, `library/tests.rs:30`,
  `routines.rs:724`-`:824`, `scheduler/tests/pool.rs:239`, `native.rs:45`; `rexxutil/files/tests.rs:66`
  needs `target/tmp` on another file system than `/tmp`). Every `run::tests::conditions` case,
  the two new `delayed_trap_cases` among them, passes.
* Callgrind controls (below). I did not rerun the suite or the workspace test; the suite record files
  `tests.tsv`, `table.tsv`, `verdicts.tsv`, `oracle-differences.tsv` are byte-identical to
  `t8.7b/fix-r3/suite/` (`cmp`).

## Focus 1: witnesses, closings, rows

* The six new corpus programs, run from scratch copies: every one differs from the oracle at base
  (stdout or stderr) and agrees at head on stdout, stderr and rc. I read the oracle stdout of
  `internal_rendering_default_name` and `collection_primitive_compare_frames`: every claimed line
  prints (`frames 4 METHOD OBJECTNAME`, `DNC`, `DNI`, `ON` under `METHOD STRING`, each primitive's
  `METHOD` frame).
* The implementer's probes re-run: `dn4`-`dn6`, `lineout4/l2`, `primframe/f3`, `traceobj/*`,
  `varref/v1`-`v4`, `stemarr/b`, `stdinq/q2`, `q6` (stdin `in.txt`) all o~b differ, o~h equal. The
  residual differences are the ones the report names: `dn7` (queue candidate 2, two oracle lines
  more), `f4` (queue candidate 1: extra `hashCode METHOD REMOVE`/`REMOVEITEM`, `==` where the oracle
  hashes for Bag `HASITEM` and Relation `ALLAT`/`HASITEM`/`ITEMS`/`REMOVEITEM`), `stemarr/a1`
  (`say t.` joins: candidate 1), `q5` (`out size  ` against `out size 34 1`, and the refused write and
  zero SEEKs; the `linein  1 19` against `linein  0 0` line follows from those SEEKs).
* `notready` n4/n7, crate only: base runs the handler twice, head once (`after a, m = 1`).
* Both "probe agrees" gate closings hold: `scopen.rex` is equal on all three descriptors at base and
  head; `samples` `TEST_MAKESTRING`, `scclient` `TEST_04_SCCLIENT.REX` and `sfclient`
  `TEST_04_SFCLIENT.REX` are `passed passed` in `t8.7c/suite/tests.tsv`.
* Closing lines: each closed queued file starts `**CLOSED 2026-10-29 by Phase 9d Task 8.7c
  (e620097f0).**`; the open ones (stem-default-arithmetic, stream-query-residuals, append,
  hash-identity, debug-input) are unchanged. Inventory rows for the closed items read
  `closed:e620097f0` in columns 4 and 6, the open ones `9d`; `t8.7c/closed.tsv` has the two gate rows
  as `closed:probe agrees`, the form 8.7b used.

## Focus 2: the Deviation candidates

Each candidate's code state at head is the base's for the positions it names (base and head outputs
equal on `s1`, `s2` except the `t.~string` line the closed item changed, `hashid`, `append/a1`,
`lineswo/w1`, `q4`, `dp2`, `dp3`). No half-done change.

1. **Stem in a required-string position.** Re-derived: provide.xml:758 (`request("STRING")`) and
   collclasses.xml Stem `request` ("All conversion requests except Array are forwarded to the stem's
   current stem default value"), so `s. = 7; say s. ''` is `7` and `say t.` with an array default is
   the array's string value, its `makeString` join. Confirmed by running `s2`: oracle `NOSTRING |7|
   Stem` in every listed position, head `7`; `1 + v.` oracle `|NSTR| NSTR`, head `|a NSTR| Stem`; and
   the crate's own non-uniformity, `say v. ''` naming the default. Also in this candidate's reach:
   `parse value t. with c` assigns `1` at head (and base) and `an Array` on the oracle.
2. **Hash lookup of the identical index.** Re-derived: "Index equality is determined by using the `==`
   method" (collclasses.xml); no text makes identity a precondition. Confirmed: `== called` twice at
   head and base, never on the oracle.
3. **Positioned writes on APPEND.** funct.xml:4307-4312 confirmed. `a1`: both answer `seek 1`; oracle
   `pos3 4`, head `pos3 17`; `w4` oracle 0, head 1 (Deviation 55). Confirmed.
4. **Debug input with traps OFF.** condtra.xml:489-494 (Note 4) confirmed. Crate only: `dp2`/`dp3`
   deliver `h 1 from input` and `h 2 from input` at base and head alike. The derived answer is no
   handler run (USER's default action). Not run on the oracle, as the report says.
5. **SEEK of an unopened transient stream.** iostrms.xml:194-197 ("SEEK and POSITION ... are not
   applicable to transient streams") and 93.958 (errnums.xml:3525). `q4`: oracle `seek 1 READY` then
   93.958 after `open('read')`; base and head 93.958 at the first SEEK. Confirmed.
6. **STRING-overriding renderings.** instrc.xml:3697-3699 ("obtained by sending them a STRING
   message") confirmed. `dn7`: `trace i; x = q` with `q~string` = `SN` renders `a SN` on all three.
   Confirmed. Note the closed DEFAULTNAME item implements the oracle's `OBJECTNAME` send; it gives
   the rule's answer wherever STRING is not overridden, so the item and this candidate do not
   conflict, but a ruling for the rule changes the same function.

The report's two new oracle defects: both confirmed by running (`s2`: `NOSTRING |7| Stem`; `q4`:
`seek 1 READY`).

## Findings

### Important

**I1. A failing `OBJECTNAME`/`DEFAULTNAME` send inside a rendering is dropped (concern 1,
confirmed, and reachable).** `value.rs:474`-`:475` (`Ok(None) | Err(_) => None`) turns any failure
of the send into the default name. The report's "No probe reaches it" is false: every program below
reaches it, and head now runs the program's code there and discards its error (base never sent).

    e1: ::method defaultname; return 1 + 'a'   with  signal on syntax; say .array~of(.bad~new)~makestring
        oracle rc 0: trapped 41.1 Nonnumeric value ("a") used in arithmetic operation.
        head   rc 0: a BAD / not reached?
    e3: same class raising 98.900, rendered by  trace i; x = o
        oracle rc 158: Error 98.900: boom. (traceback under Compiled method "OBJECTNAME")
        head   rc 0: >V> O => "a BAD" ... after trace
    e4: DEFAULTNAME saying 'dn ran' then raising 98.900, rendered by o~nosuch's 97.1 insert
        oracle: dn ran / trapped 98.900 boom.
        head:   dn ran / trapped 97.1 Object "a BAD" does not understand message "NOSUCH".
    r4: ::method defaultname; return .array~of(self)~makestring   with  say .array~of(o)~makestring
        oracle rc 245: Error 11 (control stack), under OBJECTNAME and MAKESTRING frames
        head   rc 0: a REC / after

No rexxref text lets a raised condition vanish; a method's SYNTAX propagates to its sender. The
rendering needs to carry the `Failure` to the trace line, message and join that asked for it.

**I2. An internal routine's own `CALL ON NOTREADY` is ignored while its caller holds a queued
NOTREADY.** `run/condition.rs:1101`-`:1120` counts a condition queued at any caller in the
internal-call chain without asking whether the trap in force is still the inherited one. rexxref
condtra.xml Notes: the trap state is saved on entry, and "CALL ON ... can be used in a subroutine
without affecting the conditions set up by the caller", so a new `CALL ON` in the routine is ON, not
delayed. Crate only (two NOTREADYs, entry 66's family), `t87c-review-scratch/nr/n1.rex`:

    call on notready name hn
    x = linein('no_such_file_zz.txt') || rn()
    rn: call on notready name hn2; y = linein('no_such_file_zz2.txt'); say 'rn after its linein'; return y
    base: hn2 NOTREADY no_such_file_zz2.txt / rn after its linein / hn 1 NOTREADY no_such_file_zz.txt
    head: rn after its linein / hn 1 NOTREADY no_such_file_zz.txt

Head regresses the base's derived answer. `SIGNAL ON NOTREADY` in the routine (`n2.rex`) still
signals on both. The chain should stop at the first activation that set its own NOTREADY (or ANY)
trap.

### Minor

**M1. A `DEFAULTNAME` or `OBJECTNAME` answering a non-string renders by its `makeString`.**
`value.rs:474` uses `to_text(answered)`, where `native_object_name` and `native_string` render the
answer by `default_string_value_text`. `t87c-review-scratch/c1/e6.rex`, a `DEFAULTNAME` returning
`.array~of('p','q')`: oracle `>V> O => "an Array"` and `Object "an Array" does not understand`, head
`"p` newline `q"` in both, while head's own `.dn~new~objectName` and `~string` answer `an Array`.

**M2. The `leveled_store_at` indirection has no measured effect, and its comment is false at
`e620097f0`.** `hash.rs:1960`-`:1963` says a direct third caller costs `dirread` +0.19%. With only
that `black_box` removed (`nobbhash`, below) every program is within 0.0001% of head libc
subtracted (`dirread` -0.0223% both). `store_at` gained `#[inline(always)]` in the same diff, which
plausibly made the indirection redundant. Delete the indirection and the sentence.

**M3. An inserted function took another's doc block.** `dispatch/stream.rs:222`-`:226`: the doc of
`unopened_standard` ("Whether `receiver` is a standard or `HANDLE:n` stream nothing has opened ...")
now heads `standard_input_file`'s own doc, and `unopened_standard` has none.

## Focus 3: other hunts that came back clean

* Compare frames through a `String` subclass index (`Array~index`, `List~hasItem`, `Table~hasIndex`,
  `t[k]`) and an `enhanced` object (`Array~hasItem`, `~index`, `List~removeItem`): head equals the
  oracle on every line (`pf/s1.rex`, `pf/s2.rex`); base shows `PROGRAM` frames.
* `>O>` of references to a stem, through `USE ARG >z.`, and of a reference to a reference: head
  equals the oracle (`tr/t1.rex`); base named `Z.` for `w = >z.`.
* Stream methods with trailing omitted, omitted-then-third, and the builtins' 40.4: equal on all
  three (`st/x1.rex`).
* An `EXIT 7` inside `DEFAULTNAME` renders `7` on oracle and head (`c1/e2.rex`).
* `trace i` inside a `DEFAULTNAME` rendering the object itself (`c1r/r2.rex`) never ends on either
  side (oracle rc 124, head killed at the 20 s timeout); not a divergence.

## Focus 4: perf (concern 2)

Controls built from the head archive, each in its own target dir: `nobb` (both `black_box` calls
replaced by direct calls), `headpad` (head plus an 11 kB never-called `#[used]` function in
`value.rs`), `nobbpad` (`nobb` plus the same pad), `nobbhash` and `nobbvalue` (one site each).
`callgrind.sh -r 1` (`cg-9d/t8.7c-review/ctl`, `ctl2`; one round, the counts being deterministic
in the report's three-round table), exit 0. Libc-subtracted
delta against base, %:

| program | head | nobb | headpad | nobbpad | nobbhash | nobbvalue |
|---|---|---|---|---|---|---|
| dirread | -0.0223 | -0.0299 | -0.0223 | -0.0299 | -0.0223 | -0.0299 |
| parse | +0.0007 | +0.1043 | +0.0008 | +0.1043 | +0.0007 | +0.1043 |
| decloop | +0.0005 | +0.0837 | +0.0005 | +0.0837 | +0.0005 | +0.0837 |
| fibcall | +0.0001 | -0.0207 | +0.0001 | -0.0207 | +0.0001 | -0.0207 |
| startup | +0.0186 | +0.0188 | +0.0189 | +0.0186 | | |
| hashnew | +0.0198 | +0.0198 | +0.0198 | +0.0198 | +0.0198 | +0.0198 |
| heapshape | -0.0436 | -0.0879 | -0.0436 | -0.0879 | -0.0436 | -0.0878 |
| interpret | | | | | +0.0007 | +0.0007 |

Ruling on concern 2: **not layout luck.** The pad moves no program on either arm, so the
`string_value_text` indirection changes executed instructions through code generation, not
placement; without it `parse` (+0.1043%) and `decloop` (+0.0837%) are over the +0.05% budget,
which confirms `value.rs:401`-`:404`. It is still a workaround for an optimiser decision, and a later
edit near these callers may move it again. Its direct-call cost is the only one: `dirread` is
better without either indirection, and the `hash.rs` one does nothing (M2).

The report's table (`t8.7c/callgrind.txt`): every program at most +0.0404% libc subtracted (pingmsg),
no OVER or SPREAD, `status 0`. My base and head runs differ from its raw counts by a few hundred
instructions and reproduce its libc-subtracted deltas on `parse`, `decloop`, `fibcall`, `heapshape`,
`hashnew` and `dirread` to the fourth decimal.

## Focus 5: memory

`mem/m1.rex`, a loop doing an Array join of `DEFAULTNAME` objects, a `Table` `[]` with a Rexx-hashed
index and a 97.1 insert of a `DEFAULTNAME` object: head 38512 KB at 20000 and 38132 KB at 200000
(base 37824 and 37752). Flat.

## Queue candidates

None beyond the report's three, which I confirm by running (`f4`, `dn7`, `q5`).

## New oracle defects

1. An Array join of an item whose `OBJECTNAME` answers a non-string writes raw bytes: `::class on;
   ::method objectname; return .array~of('p','q')` with `say .array~of(.on~new)~makestring` prints
   `\0\0` newline (`od -c`), `t87c-review-scratch/c1/e6.rex`, rc 0.
