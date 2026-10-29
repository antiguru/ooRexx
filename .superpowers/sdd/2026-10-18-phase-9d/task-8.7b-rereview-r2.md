# Task 8.7b re-review, fix round 2

Reviewed `4880b6bb9..3d23302bc` (code `d775871f7`, `d1f8b387f`; records `3d23302bc`) against
`task-8.7b-fix-r2.md`.

**Verdict: CHANGES REQUESTED.** Critical 0, Important 1, Minor 2. One new oracle crash.

| brief item | verdict |
|---|---|
| 1. NOTREADY ignored while delayed | ADDRESSED for NOTREADY; the debug-pause re-derivation NOT ADDRESSED (Important 1) |
| 2. Hidden primitive frame | ADDRESSED |
| 3. Redelivery chain memory | ADDRESSED as ruled: the controller's claim holds by running (below) |
| 4. SIGNAL ON release keeps ADDITIONAL | ADDRESSED (ADDITIONAL and RESULT); POSITION is Minor 1 |
| 5. Entry 64 title | ADDRESSED |
| 6. Deviation pins | ADDRESSED |

## What I ran

* Builds: `git archive` of `cca0956e0` and `d1f8b387f` (`rust/`, `interpreter/`) into fresh scratchpad
  directories, `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx` into
  `/tmp/claude-1000/t87b-rr2-target-base` and `-head`, exit 0 each. `.text` (`objcopy -O binary
  --only-section=.text`, `cmp`): head identical to `ooRexx-p9-pin/cg-9d/t8.7b/rexx-head-d1f8b387f`, base
  identical to `rexx-head-10b0f5081`. Both target dirs deleted at the end.
* Probes: fresh directories under `t87b-rr2-scratch/`, sides `o` oracle (standard wrapper), `b`
  `rexx-head-5053fbd9b` (before fix round 1), `c` `cca0956e0`, `h` `d1f8b387f`, crate under `memcap 2G
  timeout -k 5 20`, stdin `/dev/null` unless named, stdout, stderr and rc compared separately. Shapes
  of entries 64, 65, 66 (and `p8`, `dp1`, `dp2`, see below) ran on the crate only.
* Tests at head, from the archived tree with `build`, `oodocs`, `ootest` linked in:
  `memcap 8G cargo test --release -j 4 --no-fail-fast -p rexx-exec --lib --test signals --test
  collect_stress`: exit 101; lib `1206 passed; 1 failed`, the failure
  `a_move_across_file_systems_copies_and_unlinks` ("needs ... on different file systems": my archive
  sits on `/tmp`, environmental); `signals` 42 passed; `collect_stress` 74 passed.
* The round's new tests against `cca0956e0`'s code (test hunks of `cca0956e0..d1f8b387f` patched into
  the base archive, `cargo test --release -p rexx-exec --lib -- <names>`): exit 101, `2 passed; 4
  failed`. Red: `a_hidden_primitive_lists_no_frame` (`left: (120, "", "rexx-exec: the executable of a
  stack frame built without one is not implemented\n")`),
  `a_condition_raised_again_while_its_call_trap_is_delayed` (`notready`),
  `conditions_waiting_for_their_handler_survive_a_collection` (`left: "h 1 first first-add\nh 2 second
  second-add\n"`), `a_condition_raised_while_its_handler_runs_is_held` (the case file's `SIGNAL ON`
  case at `crate:100`). Green: the Deviation pins, as the report says.

## Items

**1. NOTREADY.** `d1` (rr1's): `o`, `b`, `h` equal on all three descriptors, `c` runs the handler
twice. Siblings, all `o`~`h` equal on all three descriptors:

* `p1`: the handler CALLs a routine whose `linein` fails (Note 2, the routine inherits DELAY). `c`
  calls the handler without end and is OOM-killed at the 2G cap (rc 137).
* `p2`: in the handler, `charin` of a missing file, `lineout`/`charout` to `/`, `linein('/')`,
  `chars`, `stream(..., 'c', 'open read')` and `linein` of a missing file, one per clause. `c` loops
  until the timeout (rc 124).
* `p3`: NOTREADY raised in a `CALL ON USER` handler, its own trap not delayed: delivered there
  (`hn 1 no_such_file_zz.txt` before `hu returns`).
* `p5`: `CALL ON ANY` entered for USER, NOTREADY raised in it: ignored. `c` OOM-killed.
* `p6`: `SIGNAL ON NOTREADY` in the handler, then a failing `linein`: `sg no_such_file_zz2.txt SIGNAL
  10`.
* `p7`: `CALL ON NOTREADY name hn2` re-issued in the handler, then a failing `linein`: `hn2` runs.
* Corpus debug programs (`debug_*call_on*`, `debug_input_eof*`, `debug_handler_exit_at_iterate`,
  `debug_pause_sigl_notready`, `debug_typed_condition_ignored`, each with its `.stdin`): every
  program ends, `o`, `b`, `c`, `h` equal on all three descriptors.

The debug-pause drop's removal: Important 1.

**2. Hidden primitive.** `n5`, `n6` (rr1's): `h` equal to `b` on all three; `c` names scope `.NIL`
(`n5`) and refuses at rc 120 (`n6`). `n2`: the queue candidate, unchanged. The untraced path:
`retain_found_primitives` is `#[cold] #[inline(never)]` behind `!method_levels.is_empty()`
(`dispatch/context.rs:656`); pingmsg and startup below. A mutant dropping every primitive level
(`retain_found_primitives` answering false for every `Method` level, gate forced on) leaves the lib
suite as at head (`1206 passed; 1 failed`, the environmental test), so the found-primitive half rests
on the corpus differential: `corpus/lang/primitive_frame_past_override.rex` lists the `SORT`, `[]=`
and `REQUEST` primitive rows, `o`~`h` equal on all three.

**3. Memory.** Peak RSS, `memcap 2G timeout -k 5 60 /usr/bin/time -f %M BIN PROG N` (so `time`
measures the interpreter itself); oracle under the standard wrapper with `/usr/bin/time` inside it.

| program | N | `c` | `h` | oracle |
|---|---|---|---|---|
| `m1.rex` (redelivery chain) | 2000 | 39,412 KB | 39,012 KB | |
| `m1.rex` | 20000 | 217,464 KB | 217,892 KB | |
| `ctl3.rex` (same graph, ordinary deliveries) | 2000 | 40,872 KB | 39,900 KB | 21,200 KB |
| `ctl3.rex` | 20000 | 226,372 KB | 225,588 KB | 99,664 KB |
| `m3.rex` (ordinary delivery loop) | 20000 | 37,516 KB | 37,616 KB | |
| `m3.rex` | 100000 | 37,936 KB | 36,856 KB | |
| `mb.rex` (`m1`, a `PROCEDURE` handler holding a 100 KB local) | 200 / 2000 | 53,456 / 74,368 KB | 54,236 / 74,236 KB | |
| `ctlb.rex` (`ctl3`, the routine holding a 100 KB local) | 200 / 2000 | 53,732 / 72,332 KB | 53,156 / 73,472 KB | 24,124 KB at 2000 |
| `mbo.rex` (`m3` with the 100 KB local) | 2000 | 53,128 KB | 53,768 KB | |

`walk.rex` 2000: `walked 2000 back to first`, `c` and `h`. The ruling holds: the chain grows exactly
as the ordinary-delivery control building the same graph does, and the ordinary loop stays flat. The
100 KB local is the discriminating control: were each redelivery keeping its handler's activation
(variables, not only the `StackFrame` snapshot), `mb` at 2000 would carry about 200 MB more than `m1`;
it carries 35 MB more, as much as `ctlb` over `ctl3`, where no trap is delayed. Held conditions rooted:
the new test is red at `cca0956e0` (above) and green at head.

**4. Minor 1.** The case file's `SIGNAL ON` case asserts `sg two SIGNAL 10 two-add two-res`; red at
`cca0956e0`. POSITION: Minor 1 below.

**5. Entry 64** is titled for its program's halves. **Entry 66**: its program is
`fix-r2/nr/nr.rex` byte for byte, and its effect line matches `fix-r2/nr/out/nr.rex.o.*` (stdout ends
`hn 1 NOTREADY no_such_file_zz4.txt`, stderr empty, rc 139). The recorded stdout shows the first half
(`ha 1 returns`, `after any, m = 1`) completing, so the crash is the second half; the entry says "not
narrowed", which is true.

**6. Deviation pins.** Each test program run as a file on all sides: `a.rex` oracle `0 35.1`, crate
`1 35.1`; `b1.rex` oracle `[    14 *-*   return 1 / 0]` and `[     5 *-* x = f()]`, crate as the test
expects; `b2.rex` oracle `t2      5 *-*   do f over .context~stackframes;`, crate as the test expects.
Each test's expected text equals the crate's output and differs from the oracle's, so an
implementation giving the oracle's answer fails it. They pass at `cca0956e0`, being tests only.

## Findings

### Important 1: a condition the debug pause's input read raises inside its own handler now redelivers without end

`debug_input_line` (`run/interpret.rs:376`) is `5053fbd9b`'s again. With the drop gone, a condition
the `.local~DEBUGINPUT` read raises while its `CALL ON` trap is delayed is held (only ERROR, FAILURE
and NOTREADY are ignored, `ignored_while_delayed`, `run/condition.rs:1803`), so the handler runs
again at RETURN, pauses again, reads again and raises again. `dp2/dp2.rex` (crate only):

```
.local~debuginput = .src~new
call on user ping name h
n = 0
trace ?r
nop
say 'after' n
exit
h:
  n = n + 1
  say 'h' n condition('d')
  return
::class src
::method linein
  say 'linein'
  raise user ping description 'from input' return ''
```

`b`, `c`: rc 0, stdout ends `after 1`, `h 2 from input`, `linein`. `h`: rc 137, `memcap: OOM-killed
at the 2G cap (peak 2.0G)`, after 52,710 `h ... from input` lines. `dp1/dp1.rex` (the pause in a
handler entered for `first`, bounded by `n > 3`): `b`, `c` `h 1 first`, `linein`, `after 1`; `h`
`h 2 from input` to `h 4 from input`, `after 4`.

rexxref supports dropping these: Note 4 (`condtra.xml:489`-`:494`) sets every trap OFF while
interactive input is processed, and an untrapped USER or NOTREADY is ignored (`:402`-`:403`). That is the
part of the drop rr1 found supported. The report's re-derivation reads the drop as existing only
for the NOTREADY loop, and removes the supported part with it. Fix: ignore the conditions the input
read raises as Note 4 says (under OFF a HALT takes its default action, rr1's caveat), with `dp2` as
the witness. Note 4 reaches further than `b` and `c` go: `dp2`'s first read, in main with no trap
delayed, delivers `h 1` on every side, where Note 4 says it is ignored too. That part predates the
task; it belongs to the same fix or to the queue. The oracle was not run on `dp1` or `dp2`: `b` reads
DEBUGINPUT more than once at one pause, which could queue one condition twice at one boundary.

### Minor 1: a SIGNAL ON released over a held condition reports the SIGNAL ON clause as POSITION

`release_delayed` (`run/condition.rs:127`) sets `raised.position` from the release clause.
rexxref: POSITION is "the line number in source code at which a condition was raised" (`:769`), and
a delayed condition's action "including the updating of the condition information" waits for the
release (`:450`-`:451`), so the information signalled is the held condition's. SIGL is right as it is
("the last clause processed ... before the CALL or SIGNAL took place", `:677`-`:678`). The CALL ON and
RETURN releases already report the held POSITION. `p8/p8.rex` (crate only; each condition raised by
`raise user ... return` in a one-line routine at lines 10, 11, 12; my annotations after each line):

```
hs two 10 2                  RETURN redelivery: the raise's line
hc two 11 21                 CALL ON re-issued: the raise's line, SIGL the CALL ON
sg two 27 27     27 *-*   signal on user s2 name sg
```

The last line's POSITION and first TRACEBACK line name the `SIGNAL ON` clause (27); the held
condition was raised at line 12. This is the report's concern 3, derived. Fix: take POSITION (and
the traceback and stack frames) from the held condition object, as ADDITIONAL and RESULT now are.

### Minor 2: NOTREADY is ignored under a CALL ON ANY trap delayed by a running handler, not by another condition queued at the clause

`raise_notready` (`run/condition.rs:259`) treats the trap as delayed when `trap.delayed` or a
NOTREADY is already queued at the clause (`call_trap_delayed_at(trapping, b"NOTREADY")`). Under `CALL
ON ANY`, a USER queued at the same clause puts the ANY trap into the delayed state too ("When the
condition is raised, and before the CALL is made, the condition trap is put into a delayed state",
`:444`-`:445`), and the crate already reads a handler running for USER as that state (`p5`: NOTREADY
ignored in an ANY handler entered for USER). `p4/p4.rex`:

```
call on any name ha
m = 0
x = ru('one') || linein('no_such_file_zz.txt')
say 'after a, m =' m
...
ru: raise user u description (arg(1)) return 'r'
```

`b`, `c`, `h`: `ha 1 USER U one`, `ha 2 NOTREADY no_such_file_zz.txt`, `after a, m = 2`. By the rule
the crate applies in `p5`, the NOTREADY is ignored and `m = 1`. No oracle answer: the oracle segfaults
on this program (below).

## Perf

The implementer's `fix-r2/callgrind-final.txt`: every program, no OVER or SPREAD flag; libc
subtracted at most +0.0353% (pingmsg); every `cmd:` in `cg-9d/t8.7b/fix-r2-final/*.cg` names
`rexx-base-504ff0743` or `rexx-head-d1f8b387f` (99 each, by `grep -h '^cmd:' | awk '{print $2}' |
sort | uniq -c`), and the latter's `.text` is my `d1f8b387f` build's.

Re-measured: `bench-programs/callgrind.sh -B 0.5 -j 4 -r 3 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7b/rr2 -p "pingmsg startup"
base=.../rexx-base-504ff0743 head=.../rexx-head-d1f8b387f`, exit 0, log `cg-9d/t8.7b/callgrind-rr2.log`:

| program | head d% | -libc d% | spread base / head |
|---|---|---|---|
| pingmsg | +0.0055 | +0.0357 | 0.0004 / 0.0001 |
| startup | +0.0180 | +0.0236 | 0.0003 / 0.0004 |

Within +0.05% and within 0.001 points of the implementer's figures. SPREAD (pingguard, pingsem, from
the implementer's table): pingguard head spread 0.0003%, -libc -0.0076%; pingsem 0.0005%, -0.3481%.

## New oracle defects

**`p4/p4.rex` segfaults the oracle**: rc 139, stdout `ha 1 NOTREADY no_such_file_zz.txt` only,
stderr empty, run once, 2026-10-29. Its first clause raises USER (`raise ... return 'r'` in a function
call) and NOTREADY, different names, both trapped by one `CALL ON ANY`. `rust/CLAUDE.md`'s probe rule
says two differently named conditions in one clause are safe; under `CALL ON ANY` they are not. Not
narrowed, not re-run. A candidate entry 67 and a probe-rule sentence, the controller's. This crate
prints the six lines above at rc 0.
