# Task 8.7b re-review, fix round 1

Reviewed `1fd1d3e01..878d94a13` (code `97ccdac9c`, `848403a01`, `10b0f5081`, `cca0956e0`; records
`878d94a13`) against `task-8.7b-fix-r1.md`.

**Verdict: CHANGES REQUESTED.** Critical 0, Important 3, Minor 3.

| brief item | verdict |
|---|---|
| 1. Important 1, frame names the override | ADDRESSED (new Important 2 below comes from its NIL-scope arm) |
| 2. Important 2, manager set during the main program's install | ADDRESSED |
| 3. Ruling: delayed CALL ON redelivery | NOT ADDRESSED for NOTREADY (Important 1); USER, HALT, ANY, ERROR and FAILURE follow rexxref |
| 4. Minor 1, `push_installing_level` doc | ADDRESSED |
| 5. Minor 3, native-method-frames closing | ADDRESSED |

## What I ran

* Builds: `git archive` of `5053fbd9b` and `cca0956e0` (`rust/`, `interpreter/`) into fresh scratchpad
  directories, `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4` into
  `/tmp/claude-1000/t87b-rr1-target-base` and `-head`, exit 0 both. `.text` (`objcopy -O binary
  --only-section=.text`, `cmp`): base identical to `ooRexx-p9-pin/cg-9d/t8.7b/rexx-head-5053fbd9b`,
  head identical to `rexx-head-10b0f5081` (so the pinned binary carries `cca0956e0`'s code, whose
  last commit changes tests only) and different from `rexx-head-848403a01`. Both target dirs deleted.
* Probes: fresh directories under `t87b-rr1-scratch/`, oracle under the standard wrapper, base and
  head under `memcap 2G timeout -k 5 20`, the oracle's `build/lib` on `LD_LIBRARY_PATH`, stdout,
  stderr and rc compared separately (`o` oracle, `b` `5053fbd9b`, `h` `cca0956e0`). Shapes like
  `oracle-crashes.txt` entries 64 and 65 (a direct `RAISE ... RETURN` in the handler, a `SIGNAL ON`
  re-issued over a held condition, ERROR or FAILURE again in their own handlers) and an Array native
  on a non-Array receiver (entry 62's family) were run on base and head only.
* At head, from the archived tree with `build`, `oodocs`, `ootest` linked in:
  `cargo test --release -j 4 -p rexx-exec --test signals`: exit 0, `42 passed`; `cargo test
  --release -j 4 -p rexx-exec --lib -- a_condition_raised a_message_halt_inside`: exit 0, `4 passed`
  (`a_condition_raised_while_its_handler_runs_is_held`,
  `a_message_halt_inside_its_handler_runs_it_again_on_return`, and the neighbours the filter matches).
* Perf and memory: below.

## Rule check: what rexxref says for each derived part

All citations `oodocs/rexxref/en-US/condtra.xml` unless named.

* **Redelivery at RETURN**: supported, `:460`-`:461` ("the condition trap is no longer delayed and
  the subroutine is called again immediately"). `d2` (two conditions held in separate clauses) runs
  the handler again for each, in raise order; `d3` (PING's handler raising PONG, whose handler raises
  PING) holds PING for the outer handler and runs it after both return; `d5` (raises inside
  INTERPRET in main and in the handler) and `t1` (a started method and a method after REPLY) behave
  the same. In each the oracle runs the handler once.
* **Re-issued CALL ON**: supported, `:454`-`:456`. `d6` re-issues it in a routine the handler calls
  (whose trap table is the handler's copy, Note 2): the held condition is called there at once.
* **Re-issued SIGNAL ON**: supported, `:454`-`:456`. The condition object built at that clause
  loses the held ADDITIONAL (Minor 1).
* **CALL OFF / SIGNAL OFF then the default action**: supported, `:457`-`:459` with "Action Taken when
  a Condition Is Not Trapped" (`:391`, HALT raises SYNTAX; `:402`, every other condition is
  ignored). `d7`: the 4.1 at `call off halt` is taken by `SIGNAL ON SYNTAX` (`sx SYNTAX 4  10`, rc 3).
  `d9`, `c1/s2`: `SIGNAL OFF HALT` is 4.1 at that clause; `SIGNAL OFF USER PING` drops the held
  condition and a later raise is untrapped.
* **ERROR and FAILURE ignored**: supported, `:449`-`:450`, `:160`-`:161`, `:184`-`:185`. A FAILURE in
  its own handler not becoming ERROR is supported too: the ERROR entry raises ERROR for a failure
  only where no FAILURE trap is active, and a delayed trap "remains enabled" (`:448`-`:449`).
* **NOTREADY**: contradicted. Its own entry: "This condition is ignored if the NOTREADY condition trap
  is already in the delayed state" (`:278`-`:280`); `iostrms.xml:665`-`:667`: "If the NOTREADY
  condition is in the DELAY state ... processing continues, and the NOTREADY condition remains in the
  DELAY state." This round holds and redelivers it (Important 1).
* **The debug pause drops the conditions its own input read queues**: partly supported. Note 4
  (`:489`-`:494`) sets every trap OFF while interactive input is processed, under which a USER or
  NOTREADY condition is ignored (`:402`), so dropping those matches. Two parts do not follow from it:
  under OFF a HALT takes its default action rather than being dropped, and the note covers every
  trap, where `drop_queued_for_delayed` (`run/condition.rs`) touches delayed traps only. The comment
  at `debug_input_line` (`run/interpret.rs:380`) justifies the drop by the handler being called again
  for ever at the end of the input; the corpus programs it names pause in a `CALL ON NOTREADY`
  handler, which is the redelivery Important 1 removes.

The changed tests assert the rule, not something weaker: the scheduler test expects `h again` after
`h out`, the `signals` tests `h again 4` after `h out` and a second `h in 4` / `h out 0`; a drop
fails each. `delayed_trap_cases/crate`'s case "NOTREADY, and a CALL ON ANY trap" asserts the
NOTREADY redelivery Important 1 contradicts, and the pre-existing
`a_condition_raised_again_while_its_call_trap_is_delayed` (`run/tests/conditions.rs:1073`, not this
round) asserts `h NOTREADY` twice for the same reason.

## Findings

### Important 1: NOTREADY raised again while its CALL ON trap is delayed is held and redelivered

`hold_delayed` (`run/condition.rs:188`) and `take_queued_for_delayed` (`:136`, the test at `:154`)
ignore only ERROR and FAILURE. rexxref's NOTREADY entry and the streams chapter (cited above) say a
NOTREADY raised while its trap is delayed is ignored. Probe `d1/d1.rex` (a `CALL ON NOTREADY`
handler whose own `linein` of a missing file raises NOTREADY again):

```
oracle, 5053fbd9b                      cca0956e0
hn 1 no_such_file_zz.txt DELAY         hn 1 no_such_file_zz.txt DELAY
hn 1 returns no_such_file_zz.txt       hn 1 returns no_such_file_zz.txt
after notready, n = 1                  hn 2 no_such_file_zz2.txt DELAY
                                       hn 2 returns no_such_file_zz2.txt
                                       after notready, n = 2
```

stderr and rc equal. Fix: ignore NOTREADY with ERROR and FAILURE, under its own trap and under ANY;
then re-check whether the debug-pause drop is still needed and what its comment should say, and
change the case file's NOTREADY half (and the older test's `notready` row) to the ignore.

### Important 2: a primitive frame with no primitive found answers scope `.NIL`, and its `executable` refuses

Report concern 4, reached. `build_method_level_frame` (`dispatch/context.rs:686`) builds the frame
with the NIL object as scope where `running_primitive` (`dispatch.rs:426`) finds no primitive of the
level's name. A primitive defined under another name, with its own name hidden, reaches it on an
Array receiver. `n5/n5.rex`:

```
.myarr~define('HAS', .array~method('HASITEM'))
.myarr~define('HASITEM')
a = .myarr~of(.k~new)
say a~has(.k~new)          -- .k's '==' prints each line-less frame
```

oracle `METHOD HAS ... Compiled method "HAS" with scope "MYARR".`, base no frame, head `METHOD
HASITEM ... Compiled method "HASITEM" with scope ".NIL".` (the `ORDER`/`SORT` half likewise, scope
`.NIL`). `n6/n6.rex`, the same reading `f~executable`: oracle and base rc 0 printing `0`; head rc 120,
stderr `rexx-exec: the executable of a stack frame built without one is not implemented`
(`lib.rs:730`). The shape is contrived, but it is a loud refusal this round made reachable from a
program that ran at base. Fix: name the method the dispatch resolved (record the resolution or the
message at the primitive's entry) rather than re-deriving it by name; that also closes the queue
candidate below.

### Important 3: a chain of redeliveries keeps about 10 KB per handler run until it ends

`m1/m1.rex` (a `CALL ON USER` handler that calls a routine raising its condition again until `n =
lim`; one clause in main): head peak RSS 40,440 KB at 2000, 217,968 KB at 20,000, 1,007,592 KB at
100,000 (`/usr/bin/time` on the interpreter under `memcap 2G timeout`). Three such chains of 20,000
in a loop (`m2.rex`) peak at 316,560 KB, so the memory comes back after a chain, not before. The
same number of ordinary deliveries at separate clause boundaries (`m3.rex`) stays at 37 MB at
20,000 and 100,000, and `mem/m4.rex` (one redelivery per loop pass) at 37,416 KB at 20,000 and
38,068 KB at 200,000. The report's concern 4 names the unended case ("a handler that raises its
own condition on every run is now called for ever"): `d8/d8.rex` is OOM-killed at the 2G cap after
3.42 s without printing its first progress line at 200,000 runs, where the loop rexxref implies
would run until interrupted (it is interruptible: SIGINT at 1 s ends it with 4.1, rc 124). The cause
is not located; the redelivery loop in `redeliver_held` is iterative, so the retention is something
each handler run leaves rooted or uncollected until the delivering clause ends. Fix: find what each
run leaves behind and release it per run; witness with `m1`'s two counts.

### Minor 1: a SIGNAL ON re-issued over a held condition drops its ADDITIONAL

`release_delayed` (`run/condition.rs:97`-`:120`) builds a new condition from the held one's name, rc
and description. rexxref delays "any action (including the updating of the condition information)"
(`:450`-`:451`), so the signalled condition is the held one. `c1/s1.rex` (crate only, entry 64's
shape), each raise with `additional (dd'-add')`: head `sg two The NIL object SIGNAL 10 ...` where
`condition('a')` in the first handler run printed `one-add`.

### Minor 2: `defects.json`'s `call-on-delayed-condition-dropped` uses its own vocabulary

`"category": "wrong answer"` and `"severity": "divergence"` occur on no other entry; every entry has
the same keys and the file parses (`python3 -I -c 'import json; ...'`, keys counted per entry, no
duplicate id). The other wrong-answer entries use category `wrong result`.

### Minor 3: `oracle-crashes.txt` entry 64's title is wider than its program

The title "A CALL ON condition raised again from its handler, then a SIGNAL ON for one re-issued in
its handler" also describes `u2.rex`'s shape (raised from a routine the handler calls), which the
oracle runs at rc 0. The program itself is given in full and is clear to avoid; it is not narrowed to
the half that crashes, so a `SIGNAL ON` re-issued over a held condition, alone, has no oracle answer
(the entry says so). Entry 65 is described well enough to avoid.

## Items

**1. Primitive frame past overrides.** `f2/f2.rex` and `rn/rn2.rex`: oracle~base stdout differs,
oracle~head equal on all three. `n1/n1.rex` (a comparator that hides `SORT` on the subclass, defines
a Rexx `SORT`, or defines List's `APPEND` as `SORT`, during the sort, then reads the frames, which is
report concern 2): all three equal. `m6.rex`, a forwarding `SORT` override whose comparator reads
`.context~stackFrames[2]~executable`, 41,652 KB at 5000 sorts and 41,924 KB at 50,000.

**2. Install-time manager.** `sm1/main.rex`: oracle~head equal on all three. Beyond it, all equal to
the oracle at head on all three: `v1/` (main requires `a.cls`, which requires `c.cls` then `d.cls`;
`c.cls`'s prologue sets a manager on `a.cls`'s package: `a.cls`'s later `::REQUIRES` and its
prologue's command unasked, its routine called later asked), `v4/` (main CALLs `x.rex`, whose first
`::REQUIRES` prologue sets a manager on `x.rex`: `x.rex`'s later `::REQUIRES` and main-body command
unasked, its routine asked). `v3/` (main body `.context~package~loadPackage('e.cls')`, whose prologue
sets a manager on main's package, then a later `loadPackage` of a package with its own `::REQUIRES`):
every manager answer equal; stdout differs on base and head alike by the oracle's `frame METHOD
LOADPACKAGE` row (queue candidate). `mem/m5.rex` (a `CALL` of `x.rex` in a loop): base 54,332 and
55,076 KB at 5000 and 15,000, head 54,916 and 55,024 KB.

**3. Delayed redelivery.** `u2.rex`, `u3.rex`: oracle~base equal, head differs as the rule says.
`w2.rex`, `w3.rex` (the implementer's, oracle-safe): head differs as the rule says, `w2` at rc 252
for the `CALL OFF HALT` half. `u4/u4.rex` (a direct `RAISE USER` without RETURN) prints nothing at
rc 0 on all three, as the review found; outside this round. Findings above.

**4. Minor 1.** `install.rs:494`-`:499` names both `None` cases, matching `:506`-`:509`.

**5. Minor 3.** `frames/f1.rex` under the pinned `rexx-base-504ff0743` against the oracle: the only
differing lines are the `run`, `sort`, `eq`, `makearray` and `hash` rows, stderr and rc equal, as the
closing now says.

## Perf

`bench-programs/callgrind.sh -B 0.5 -j 4 -r 3 -o /home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7b/rr1
base=/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7b/rexx-base-504ff0743
head=/tmp/claude-1000/t87b-rr1-target-head/release/rexx` (my `cca0956e0` build), every program, exit
0, no OVER or SPREAD flag; log `cg-9d/t8.7b/callgrind-rr1.log`. Libc subtracted against base, every
program at most +0.0304% (pingmsg); startup +0.0232% (57,319,016 to 57,332,331), sayloop +0.0125%,
interpret +0.0066%; pingguard -0.0073%, pingsem -0.3477%. Spreads at most 0.0036% (startup head).
Whole process at most +0.4709% (interpret). Every libc-subtracted delta is within 0.01
percentage points of the implementer's `fix-r1/callgrind-final.txt`. Every `cmd:` line in
`cg-9d/t8.7b/fix-r1-final/*.cg` names `rexx-base-504ff0743` or `rexx-head-10b0f5081`, and the latter's
`.text` is `cca0956e0`'s.

## Queue candidates

* A primitive reached under another message name names its own registered method: `n2/n2.rex`
  (`.myarr~define('ORDER', .array~method('SORT'))`, and `SORT` defined as Array's `STABLESORT`):
  oracle `METHOD ORDER ... scope "MYARR"` and `METHOD SORT ... scope "MYARR"`, base and head `METHOD
  SORT ... scope "MYARR"` and `METHOD STABLESORT ... scope "Array"`. Predates the round; Important 2's
  fix covers it.
* `Package~loadPackage` lists no primitive frame (`v3/`), base and head alike: the review's
  primitives-without-frames candidate.

## New oracle defects

None.
