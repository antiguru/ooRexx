# Phase 6.1 final whole-branch review

Range `e6af1198b..b46ee119d`, reviewed at `b46ee119d` (Fable, 2026-10-10). Verdict: **needs fixes**.
Critical 0, Important 4, Minor 2.

Binaries: `git archive b46ee119d` and `git archive e6af1198b` (base61) extracted under
`/tmp/claude-1000/p61/final/`, each built in its own target directory with
`CARGO_INCREMENTAL=0 cargo build -j 4 -p rexx-exec --bin rexx-run` (release; HEAD also debug), each
log with its own `Compiling rexx-exec` line. Every run through
`/tmp/claude-1000/p61/final/probe.sh` (the Task 12 wrapper: oracle under `ulimit -v 1048576`, ours under
`memcap 2G timeout -k 5 60`, each from a fresh `mktemp -d` directory, three descriptors kept apart).
Probe programs are under `/tmp/claude-1000/p61/final/probes/`; the text of each finding's probe is
inline below. Concurrent programs ran 5 times per engine. The target directories were deleted after
the run.

## The defect class

Every Critical and Important the task reviews found has one shape: a mechanism that must cover every
site of a kind missed one site, and the miss was silent until a collection, a schedule or an unusual
input reached it. Rooting missed a TO/BY object (Task 3), `cold.executable` (Task 4), the stream table
(11b) and a COUNTER temp (Task 2); byte charging missed `~copy` (5a), `array_reshape` (5a) and a
MutableBuffer growth that exits through `?` (heapshape round); the drain missed INTERPRET and native
returns (11b); debug pauses missed CALL ON, LEAVE and NOTREADY (Task 6); the disposition test missed
`-> Self` constructors (Task 7); the gate judge missed early-ended runs (Task 10); the R11 consumer
classification missed seven instruction sites (11a). The hunt below forces each mechanism and sweeps
its site class by running, not reading.

| mechanism | forcing | shapes | result |
|---|---|---|---|
| live-byte accounting (`Heap::collect` debug assertion `survivor_bytes == held_bytes`) | debug binary, each shape followed by 300,000 small and 30 x 3 MB allocations; then again under `REXX_SWITCH_MODE=sim:7,gc=0.0005` | 25 in-place mutations of Array, MutableBuffer, Text, List, Queue, Directory, Table bodies, plus a class UNINIT emptying an array during a drain (`probes/b/`) | no assertion fired; 23 rc 0, 2 timed out on the quadratic paths of finding 4 |
| UNINIT drain at return against collection, resurrection, pool and sim | 8,000 finalizable objects with 300 KB pads under `memcap 2G`; drain from an internal call, a method send, SAY, INTERPRET left by SIGNAL, a DO WITH COUNTER loop, a started activity, a REPLY continuation, an UNINIT that starts an activity, an UNINIT that runs a command at a full pool, TRACE R; release and debug; `sim:1`, `sim:2`, `sim:5` with `gc=0.001` | `probes/u/u1`, `u2b`, `u2c`, `u2d`, `u4b`, `u5b`, `u6`, `u7`, `u8`, `u9` | every shape identical to the oracle, 5 of 5 where concurrent; bounded memory; one trace hash per seed |
| package parent lookup against REQUIRES and INTERPRET | Routine~new, Method~new, Package~new with a context; INTERPRET inside each; a nested Routine~new; `::REQUIRES` inside the child; a parent's `::REQUIRES` public and private classes and routines | `probes/p/` (10) | identical, including the two expected 97.1 failures |
| R11 STRING answer, instruction-level consumers | a class whose `STRING` answers `.object~new`; also `.nil` and a Queue | `probes/s/` (67) | every site is in the `.nil` set or the licensed refusal set; see finding 1 for the class the sweep found |
| R11 STRING answer, builtin argument positions | the same class across 112 builtin calls, the object at each position | `probes/s3/` | finding 1 |

The gate record's Task 12 claims checked against the tree: `13bbff35f` and `97cb37712` change one
table and one test (`git show --stat`); the criterion 5 rows match `task-12-evidence/whole-groups-table.tsv`
(Class whole fails TEST_ACTIVATE and TEST_METHODS; RexxContext rest fails TESTRS01; Method whole stops at
TEST_NEW_FOUR_ARGS); `dirread.rex` is named by `callgrind.sh`; the exclusions row "Method~new AND
Routine~new REFUSED ANY THIRD ARGUMENT" is CLOSED (line 5615); roadmap row 9's four `OWNER: Phase 9`
rows are the ones it names (lines 5264, 5522, 6094, 6159) and `Loud::native_method` carries
`Some("Phase 9")` (`lib.rs:612`); the `lib.rs:3052` comment's oracle citations resolve
(`Activity.cpp:249`, `RexxActivation.cpp:705`, `NativeActivation.cpp:1361`, `RexxMemory.cpp:337`).

## Important

### 1. R11: every non-target builtin argument position is a consumer in neither group (new)

R11 says a consumer whose oracle answer is defined answers it, and only a consumer where the oracle
reads `.nil` through the string layout, crashes, or answers 88.909 through REXX-package frames is a
loud refusal. The oracle sends `STRING` for every builtin argument position (confirmed with a `STRING`
answering `3` and `*`: `copies`, `substr`, `left`, `d2x`, `max`, `arg`, `random`, `pos`, `sourceline`,
`space`, `center` all print `(string sent)` and the converted answer on both engines,
`probes/s2/numeric_positions.rex`, `pad_positions.rex`). When the answer is an object, the oracle's
answer at every position that is not the string the builtin operates on is defined, at the program's
own line, with no REXX-package frame. This crate refuses all of them with
`a STRING method answering an object with no string value is not implemented`, rc 120.

Probe (`probes/s3/`): `o = .q~new` then one builtin call, with `::class q` / `::method string` /
`return .object~new`. Oracle answers, 2 of 2 where re-run:

| oracle answer | positions |
|---|---|
| 88.909 "Argument N must have a string value", at the program line, N the string method's own position (`changestr('a','a',o)` says Argument 2) | changestr 1 and 3, countstr 1, pos 1, lastpos 1, overlay 1, insert 1, translate 2, verify 2 and 3, strip 2 and 3, compare 2, abbrev 2, wordpos 1, bitand 2, datatype 2 |
| 40.12 "argument N must be a whole number; found "a Q"" (names the original object) | insert 3, space 2, copies 2, d2x 2, c2d 2, format 2, trunc 2, random 1 to 3, sourceline 1, errortext 1, arg 1, changestr 4, pos 3 and 4, substr 2, left 2, abbrev 3, wordindex 2, wordlength 2, overlay 3 |
| 40.23 "argument N must be a single character; found "The NIL object"" | space 3, substr 4, center 3, translate 4, compare 3 |
| 40.19, 40.28, 40.904, 93.904 (max 2: "Method argument 1 must be a number; found "a Q"") | date 2, time 2, xrange 1, condition 1 |

The licensed group is the target position (the string the builtin reads): SIGSEGV rc 139 on the oracle
for `changestr('a',o,'b')`, `countstr('z',o)`, `strip(o)`, `right(o,2)`, `space(o)`, `words(o)`,
`subword(o,1)`, `delword(o,1)`, `center(o,5)`; Error 5 or garbage for `length(o)` (a different number
per run), `pos('z',o)` (`1190`), `verify(o,'abc')`, `abbrev(o,'z')`, `left(o,2)`, `copies(o,2)`,
`reverse(o)`, `translate(o)`, `value(o,1)`; 93.943 "target must be a number; found "The NIL object""
for `abs`, `sign`, `trunc`, `format`, `max 1`, `min` (a layout read whose outcome is stable);
`datatype(o)` `CHAR`; `trace value o` 24.1 found `?`; `address value o` 29.1. Those refusals stand under
R11. `queue o`, `push o`, `lineout('/dev/null', o)` are 88.909 through a REXX-package frame, also
licensed.

Fix or ruling: either the non-target positions take `required_string_or_nil` and the builtin's own
argument check raises the oracle's error (88.909 numbering is the string method's, not the builtin's),
or Deviation 30's text widens to name builtin argument positions and the dispositions row cites this
table. Not queued.

### 2. Native routine arguments that are not strings are rendered, not refused (new, predates 6.1)

Neither engine sends `STRING` for a native routine's argument, so this is not an R11 site; the oracle
converts with `requiredString(position)` and raises 88.909 through the routine's frame, and this crate
renders the object's default name or reads it as a missing file.

```
o = .object~new
say 'filespec' filespec('name', o)       oracle: Compiled routine "FILESPEC", Error 88.909 Argument 2 must have a string value, rc 168
say 'sysfileexists' sysfileexists(o)     ours: filespec an Object / sysfileexists 0 / directory (empty) / sysfiletree 0, rc 0
say 'directory' directory(o)
say 'sysfiletree' sysfiletree(o, 'f.')
```
(`probes/s2/native_routine_args.rex`; `directory(o)` and `sysfileexists(o)` alone are
`probes/s3/s3_69.rex`, `s3_62.rex`: oracle 88.909 Argument 1.) The same four lines answer the same
on base61 (`target-base/release/rexx-run`), so it predates the phase. With a `STRING` answering
`/etc/passwd` (`probes/s2/silent_bifs.rex`) neither engine prints `(string sent)` for these routines,
where `stream(o)` does on both. The exclusions file's nearest row (line 5547, a user REQUEST method
never sent) is a different site. Not queued.

### 3. `Array~append` is still quadratic when the array has trailing empty slots (new; a 6.1 claim)

Task 11a Step 7 (`e8a19b2e6`) and Task 12 mapping row 33 record `Array~append` as fixed, measured on
1e5 appends to a fresh array. `append_slot` (`dispatch/collection.rs:263-280`) finds the slot with
`rposition(Option::is_some)` over the slots, so each append scans the trailing run of empty slots,
and the test `appends_and_reads_copy_slots_linearly` appends to a dense array only.

```
a = .array~new; do i = 1 to 100000; a~append(i); end          plain append 0
b = .array~new; b[200000] = 1; b~empty; 1e5 appends           after sparse+empty: ours 5.62 s, base61 18.91 s, oracle 0.011 s
c = .array~new(200000); c~empty; 1e5 appends                   after new(200000)+empty: ours 11.00 s, base61 38.05 s, oracle 0.024 s
d = .array~new; d[200000] = 1; 1e5 appends                     after sparse: ours 11.01 s, base61 timed out, oracle 0.039 s
e = .array~new; 1e5 appends; e~empty; 1e5 appends              after append+empty: ours 12.87 s, oracle 0.072 s
```
(`probes/b/t22a.rex`, `time('e')` after each block, one run per binary; base61 hit the 60 s
timeout before block d.) The phase made it three times faster and the record calls it fixed. The
oracle keeps `lastItem` as a field (`ArrayClass::appendRexx`). Not queued.

### 4. `List~remove` copies the list per call (new, predates 6.1, not a 6.1 item)

```
l = .list~new; do i = 1 to 200000; l~append(i); end
do i = 1 to 20000; l~remove(l~first); end     ours 27.89 s, oracle 0.0047 s
do i = 1 to 20000; l~remove(l~last); end      ours: 60 s timeout, oracle 0.0075 s
```
(`probes/b/t21a.rex`; `l~first` alone 20,000 times is 0 s on both.) base61 times out on the first
loop. `list_take` (`dispatch/collection/list.rs:278`) removes from the items and handles arrays by
position. `Queue~pull` was not reached on ours; the oracle itself takes 2.18 s for 20,000 pulls.
Scout E2's P2 named `Array~append`, `items` and `List~append`; `remove` was not in Task 11a's scope.
No queued item names it (`grep -l -i 'list~remove\|remove' .superpowers/sdd/queued/*.md` finds none
on List). Queue candidate; it does not block the close.

## Minor

### 5. `size_of::<Activation>()` records disagree with the tree (new)

Spec section 2: "`size_of::<Activation>() == 512` holds (`activation.rs:497`)" and R4 "Kept"; ledger
l.34 and the Task 12 report's "For Moritz" (l.34): "pinned at 480, not 512". `activation.rs:502` at
HEAD: `const _: () = assert!(size_of::<Activation>() == 472);`, since `d27a9d441` (Task 4a, whose
report says "size assertion 480 -> 472"). The spec sentence and the Task 12 line are false at HEAD;
the Task 4a report is right.

### 6. BEEP's wrong-type argument message differs (new, predates 6.1)

`say beep(.object~new)` (`probes/s2/beep_msg.rex`): oracle `Error 88.907: Argument 1 must be in the
range -999999999999999999 to 999999999999999999; found "an Object"`; ours and base61 `Argument
frequency must be in the range 37 to 32767; found "an Object"`, both rc 168 through `Compiled routine
"BEEP"`. The oracle checks the whole-number range before the frequency range.

## Checked and clean

* Spec R11's amended list, instruction level: `say`, concatenation, `length` refuse (oracle 88.909
  through REXX frames, Error 5, or garbage); truth, `&`, `\`, WHILE, UNTIL, WHEN, SELECT CASE, `=`,
  `==`, `\=`, `<`, NUMERIC DIGITS/FUZZ/FORM, DO count/TO/BY/FOR/from, INTERPRET, SIGNAL VALUE, a host
  command, a stem tail, a semaphore timeout, `hasMethod`, `pos`, a Directory index, `.environment[o]`,
  `hasItem`, `caselessEquals`, DO OVER, `exit o`, `return o`, `arg(1)`, `defaultName` are identical
  to the oracle (`probes/s/`, 67 programs, one run each, `s-results.txt`). `parse var o a` (oracle
  `.nil`) and `options o` are the parked and ruled refusals (ledger l.245, fix round 2).
* 11b's drain: the memory stays bounded under `memcap 2G` for 8,000 x 300 KB finalizable objects on a
  started activity (oracle and ours `ran`, `finalised 1`, 5 of 5); an UNINIT that starts an activity and
  waits on it, and one that runs a command while two pool threads sleep, finalise on both; TRACE R
  prints the same lines on both (neither traces the UNINIT body). With no activation return inside the
  loop neither engine drains (oracle Error 5, ours OOM at the cap), which is the oracle's rule.
* 5a and the heapshape round: no `held_bytes` mismatch under 25 mutation shapes with and without
  seeded collections on the debug binary, resurrection included.
* 11a Step 6b: class and routine lookup through INTERPRET inside Routine~new, Method~new and
  Package~new bodies, nested children, and `::REQUIRES` on either side agree with the oracle.
* Task 2 temps and 11b: a drain from a `call` inside `do counter c with index k item v over d` passes
  the debug temps watermark and matches (`sum 45150 count 300 finalised 1`).

## Not covered

No perf or callgrind runs, no full gate runs and no subagents, per the brief. The R11 sweep used one
`STRING` shape (`.object~new`) across builtins; `.nil` and a Queue were swept at instruction level only.
