# Phase 9b Task 1 report: the 9a carry-in

Base `f9105f68f`. Status: DONE_WITH_CONCERNS (concerns at the end).

## Step 1: the budget instrument

`rust/bench-programs/callgrind.sh`: the table now leads with the whole-process median per binary and
its delta against the first binary, then the libc-and-ld-subtracted median and delta, then the
whole-process spread. `-B PCT` fails the run (`OVER` on stderr, exit 1) when any binary's
whole-process delta exceeds PCT. SPREAD is judged on whole-process. `summary.tsv`'s row format is
unchanged, so `-T` re-tables an old run. README updated. `rexx-bench` has no test for this table, so
none was added; the witness is the runs below: with `-B 0.5` the HEAD-before binary is flagged
`OVER extcall before +3.0454% > 0.5%` and exits 1, and the full-set runs exit 0.

## Step 2: the pin

Built from `git archive a98238be5` into `/tmp/claude-1000/p9b-pin/src`, tree touched,
`CARGO_TARGET_DIR=/tmp/claude-1000/p9b-pin/target`,
`CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`, exit 0, one
`Compiling rexx-exec` line. Copied to `/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run` (the only file
there):

* sha256 `c4b4e8c4ce7f1f12e4a73fa37cb15eca262717092c2c1aed727febf437523c67`, 58,197,968 bytes,
  rustc 1.99.0 (b940084d7 2026-09-28).

A second build at the old paths (`/tmp/claude-1000/p9/pin/`) gave sha256 `93934309...899b8c`, same
size; neither matches the lost pin's 58,283,928 bytes. Recorded in `phase-9a-gate.md`'s pin section
beside the old hash. Both build directories deleted (`rm -rf /tmp/claude-1000/p9b-pin
/tmp/claude-1000/p9/pin`; `ls` afterwards: no such file). As a check on the rebuilt pin, its extcall
whole-process count is 8,975,966,708, against 8,975,965,522 in the 9a rereview's pin.

## Step 3: N1

`Activity::native_handles` and `native_spares` hold `Box<NativeFrame>`, and the scheduler's abandoned
frames are boxed too, so `push_native_frame`, `take_native_frame`, `pop_native_frame` and
`abandon_native_call` move a pointer. A box is allocated only when `native_spares` is empty. Two
`#[expect(clippy::vec_box)]` with reasons, as `spare_activations` has.

memcpy (`__memcpy_avx_unaligned_erms`, exclusive Ir, `callgrind_annotate --threshold=100` on
`extcall.*.r1.cg`): pin 1,112,287,697; before 1,376,289,727 (callers include five native-frame sites,
`push_native_frame` twice); after 68,289,853, with no native-frame caller left in the caller tree.

extcall, interleaved, 3 rounds, from `rust/`:

```
bench-programs/callgrind.sh -r 3 -j 9 -B 0.5 -p extcall -o .../t1/cg-extcall2 \
    pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run before=.../t1/head-before after=.../t1/head-after
```

exit 1 (`OVER extcall before +3.0454% > 0.5%`; `after` is not flagged):

| program | pin | before | after | before d% | after d% | pin -libc | before -libc | after -libc | before -libc d% | after -libc d% |
|---|---|---|---|---|---|---|---|---|---|---|
| extcall | 8975966708 | 9249317821 | 7773316365 | +3.0454 | -13.3986 | 7536919758 | 7558131839 | 7390131667 | +0.2814 | -1.9476 |

Spread 0.0000 on all three. `head-before` is `f9105f68f` rebuilt after `touch` (Compiling line
present, sha256 `16acf5b4...f86`); `head-after` is the commit's tree (sha256 `9f619d0d...584f`).

Full set, pin vs after, one round, two invocations (`-r 1 -j 15 -B 0.5 -p "<batch>"`), both exit 0:

| program | pin | after | d% | pin -libc | after -libc | -libc d% |
|---|---|---|---|---|---|---|
| alloc | 22825678004 | 22858608561 | +0.1443 | 20362612047 | 20388007210 | +0.1247 |
| alloc4c | 3673339058 | 3665696521 | -0.2081 | 3106074440 | 3098440032 | -0.2458 |
| arith | 12084068328 | 12093904717 | +0.0814 | 11478090242 | 11486760903 | +0.0755 |
| assign | 18443711984 | 18444071179 | +0.0019 | 18381875604 | 18382087938 | +0.0012 |
| compound | 9247948373 | 9252621542 | +0.0505 | 8836768877 | 8836980906 | +0.0024 |
| decloop | 2503233107 | 2506500903 | +0.1305 | 2404646660 | 2407730451 | +0.1282 |
| decrender | 4631810561 | 4623811653 | -0.1727 | 4171162211 | 4162245663 | -0.2138 |
| dirread | 3130118068 | 3142410616 | +0.3927 | 2663580961 | 2675465087 | +0.4462 |
| dispatch | 22105710248 | 22136061007 | +0.1373 | 20673777230 | 20663990667 | -0.0473 |
| dispatchclass | 16952025925 | 16288379026 | -3.9149 | 15654209631 | 15242423783 | -2.6305 |
| emptyloop | 7797087649 | 7797444651 | +0.0046 | 7735375461 | 7735587725 | +0.0027 |
| extcall | 8975966325 | 7773316365 | -13.3986 | 7536919758 | 7390131667 | -1.9476 |
| fibcall | 9109404653 | 8816064451 | -3.2202 | 8469931688 | 8296485078 | -2.0478 |
| fibfunc | 8938581998 | 8646467383 | -3.2680 | 8300416595 | 8126969985 | -2.0896 |
| hashnew | 2355780743 | 2362991924 | +0.3061 | 1588972591 | 1595703588 | +0.4236 |
| heapshape | 2432938994 | 2427482362 | -0.2243 | 2306076206 | 2300494163 | -0.2421 |
| nop | 9243390364 | 9243738598 | +0.0038 | 9181614211 | 9181825920 | +0.0023 |
| parse | 1654598628 | 1655716685 | +0.0676 | 1552802916 | 1553728594 | +0.0596 |
| pkglocal | 4185405439 | 4071779398 | -2.7148 | 3917669232 | 3767896530 | -3.8230 |
| sayloop | 179667649 | 180002888 | +0.1866 | 107724363 | 107936081 | +0.1965 |
| sendloop | 14584920991 | 14575275672 | -0.0661 | 13818085980 | 13808299421 | -0.0708 |
| startup | 119095416 | 119430319 | +0.2812 | 57373104 | 57584822 | +0.3690 |
| strings | 18808435209 | 18793356727 | -0.0802 | 17536541845 | 17521278847 | -0.0870 |
| textnum | 1224168037 | 1224509727 | +0.0279 | 1150021499 | 1150233217 | +0.0184 |
| varlookup | 13194527955 | 13194882317 | +0.0027 | 13132805929 | 13133018184 | +0.0016 |
| rexxcps | 19278651778 | 19280231948 | +0.0082 | 17792033303 | 17792136296 | +0.0006 |
| pingguard | 1314662440 | 1315775033 | +0.0846 | 1148211424 | 1148412295 | +0.0175 |
| pingmsg | 7581728655 | 7489615984 | -1.2149 | 1667809819 | 1617263106 | -3.0307 |
| pingsem | 1299613521 | 1300733623 | +0.0862 | 1171642180 | 1171847393 | +0.0175 |

Worst three whole-process: dirread +0.3927%, hashnew +0.3061%, startup +0.2812%. Each run's
`summary.tsv`, `table.txt` and `binaries.txt` are in
`docs/superpowers/records/2026-10-13-phase-9b/task-1-callgrind/` (`cg-extcall2`, `cg-full-a`,
`cg-full-b`).

## Step 4

**m1.** `package_local_get` carries `directory_get`'s view assertion again. Witness
`corpus/lang/package_local_miss_repeat.rex` (in `phase-9.txt`, sourceline file generated by the
driver): a miss read twice after a write under another name, a miss read in a loop that writes
nothing, then a fill and a remove each followed by two reads. Oracle and crate byte-identical on
stdout, stderr and rc 0; stdout read: `.PLMRNAME .PLMRNAME`, `.PLMRNONE` three times,
`filled filled`, `.PLMRNAME .PLMRNAME`.
Reach, by mutation on a debug `rexx-run` over every program in the `phase-*.txt` union (1024), each
run from a fresh directory with its `.d` copied:
* memo assertion changed to expect `Found(_)` (fires on every memo hit): fires on
  `package_local_miss_repeat`, and also on `package_local_find_class` and `package_store_tables`;
* view assertion changed to `debug_assert_ne!`: fires on `package_local_class_search_raise`,
  `package_local_find_class`, `package_local_miss_repeat`, `package_local_symbol_writes`,
  `package_namespace`, `package_store_tables`.
Both mutants reversed by exact edit; `git diff --stat` matched the pre-mutant figures.

**m2.** `class_in_program` answers `Result<Option<ObjRef>, Failure>` and propagates `Err` and `Owed`
from the package local and from the `.local`/`.environment` lookup, as `package_find_class` does
(`Loud::environment_entry` for `Owed`). Its one caller, `Surface::find_class`, holds the failure as
the native call's condition (`held`).
* Witness `corpus/lang/package_local_class_search_raise.rex` (+ `.d/lib.cls`, `.env`): a native
  method (`orxmethod TestFindClass`) whose package local's `UNKNOWN` raises 93.900. Oracle:
  `unknown called NOSUCH`, `syntax 93.900 from unknown NOSUCH.`, position 6, rc 0. Crate before the
  fix: `syntax 91.999 Message "FINDC" did not return a result.`, position 1. After: byte-identical
  to the oracle on all three descriptors. (`TestFindContextClass` cannot tell the two apart: it raises
  88.900 on any NULL, and both sides already agreed on it.)
* `Owed` cannot be reached from Rexx (the placeholder is the crate's). Crate test
  `environment::tests::a_class_search_from_a_program_refuses_an_owed_entry`: the owed placeholder in
  a program's local and `.local`'s `STDQUE` refuse loudly, a class beside it answers, a missing name
  answers none. Red with the package-local `Owed` arm folded into `Absent` (panic at
  `environment/tests.rs:286`), restored from a copy.

**m3.** `deviation-probes/README.md`: `objname-table.rex`'s fourth line is an internal routine; the
`::ROUTINE` row is `objname4.rex`'s `::routine other` (lines 10, 25-26), oracle `long`, its eighth
output line (read from `objname4.oracle.out`).

**m4.** `a_metaclass_new_answering_no_new_class_is_loud` adds the "a subclass already took" shape:
the held class answered to a second `~subclass`. Crate rc 120, stdout `XX`, the refusal on stderr;
oracle `XX` then `Array`, rc 0 (run here, `metatwice.rex` in the scratchpad).

## Checks (from `rust/`, all `CARGO_INCREMENTAL=0 ... memcap 8G ... -j 4`, statuses unpiped)

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -p rexx-exec --all-targets -- -D warnings` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --no-fail-fast --test corpus --test release_local_reference --test phase_ten_reasons --test owners --test refusal_sites --test refusal_dispositions --test loud --test collect_stress` | 0 (corpus 29 passed 1 ignored; collect_stress 37; release_local_reference 3; others all passed) |
| `cargo test --release -p rexx-exec --lib` (in a run whose concurrency_tests leg hit `timeout 590`, rc 124) | lib 1088 passed; closed_phases 8 passed |
| `REXX_CORPUS_GATE=1 REXX_WHOLE_GROUPS_ONLY=base/rexxutil/SysSleep.testGroup,base/directives/METHOD.testGroup,base/directives/ATTRIBUTE.testGroup,base/keyword/CALL.testGroup,base/class/Method.testGroup cargo test --release -p rexx-exec --test concurrency_tests each_group_of_the_derived_list_in_one_run_in_both_modes` | 0 |
| `REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` (debug) | 0 (29 passed 1 ignored) |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `cargo test --release -p rexx-core --test unsafe_sites` | 0 |
| `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=API/oo/METHOD,API/oo/FUNCTION,base/class/Package,base/rexxutil/SysSleep cargo test --release -p rexx-exec --test suite` | 101: FUNCTION and SysSleep green; METHOD 319/320 (`TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA`, the 9c rexxc test) and Package 77/80 (`TEST_NEW01 TEST_PROLOG`, `TESTRECLAIM01` timeout), the same sets the 9a final fix report records |

## Concerns

1. **m2 turns one oracle-matching answer into a refusal.** The `.local` step now propagates `Owed`,
   so a native method's `FindClass('STDQUE')` from a program refuses (rc 120) where it answered NULL
   before; on the oracle `STDQUE` is a queue, not a class, so NULL was right. This follows the brief
   ("as `package_find_class` does"); the controller may prefer the `.local` step to answer `None` for
   `Owed`. Not run against the oracle as a program.
2. **m1's premise was partly wrong.** The memo's hit assertion was already reached by
   `package_local_find_class` and `package_store_tables` (a second `.context` read hits it). The new
   witness still pins the miss-after-write shape the item names.
3. `Surface::find_class`'s other branch (`class_named`, `dot_variable(...).ok()`) still drops a
   raised condition for a routine or a method whose scope is not a program class. Not in m2's scope.
4. The pin is not bit-reproducible; Phase 9 measures against the rebuilt hash, not the old one.
5. The whole `concurrency_tests` binary was not run (over the 10-minute bound); the native-call
   groups slice above was.
