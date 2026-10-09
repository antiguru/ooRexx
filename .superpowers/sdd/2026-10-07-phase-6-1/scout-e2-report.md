# Scout E2: items 7, 8, 9, P1, P2, P3

| item | verdict | size | files a fix touches | risk |
|---|---|---|---|---|
| 7 user STRING answering an Array / another object | STILL REPRODUCES for the Array answer (`say`, concat, `length()`, truth). The older line "truth tests ignore a user STRING method" is FIXED by `d27a9d441` (Task 4a). A non-Array, non-string answer is oracle garbage (`length` = 139937618269872, concat = Error 5, `say` = 88.909): no target | S (Array); the non-Array answer needs a ruling (loud refusal or Deviation 28-style licence) | `rexx-exec/src/dispatch/reqstr.rs:165` (route the answer through `required_string_answer`/`classify_string_conversion` instead of `string_value_text`) | R9: `truth_of_string_value` goes through `required_string_value`, so the fix changes truth answers too (i7d: oracle `if true`, crate 34.1); `tests/truth/values` should gain an Array-answer case. A new `Loud` for the non-Array case re-derives `refusal-sites.tsv` and `refusal-dispositions.tsv` |
| 8 wrong-type native rows name Phase 9 | STILL REPRODUCES, and wider than queued: Message `START` and `REPLY` refuse the same way as `SEND` | S-M (three sites, one each in hash, buffer and Message code) | `dispatch/hash.rs:183` `not_this_task` (all hash `ITEMS` rows, Stem via `hash/stem.rs:143`, Relation via `hash/relation.rs:124`), `dispatch/buffer.rs:396,407`, `dispatch/object_protocol.rs:1001,1114,1196` | Refusal tables (Task 7's `refusal-dispositions.tsv`) re-derived. `dispatch/buffer.rs` has uncommitted edits in the shared tree. A MutableBuffer subclass whose INIT does not forward still answers (checked), so `buffer_state`'s None arm is wrong-type only |
| 9 Literals rows need a test case | STILL REPRODUCES, both halves: top-level `self~hex` is 97.1 on both engines; inside a test-case class, `.routine~new(name, code, package)` is refused `method "NEW" of class "Routine" ... (Phase 9)` where the oracle answers `AB A` | interpreter half S (accept the third argument by reusing `new_file_context`); harness half M (`program_for` must wrap rows in a test-case class carrying `q`/`hex`/`bin`/`runDynamicSource`) | `dispatch/construct.rs:172` (`args.len() > 2` refusal), `tests/assertions.rs` (`program_for`, `EXEMPT`) | None with sim/seeded/R10/heapshape. The `EXEMPT` rows' `unblocked_by: "Phase 9"` goes stale if the interpreter half lands |
| P1 COPIES byte-by-byte fill | STILL REPRODUCES. callgrind at HEAD, loop999 N=1e6: 13.68 G total, 6.38 G with libc excluded; the scratch doubling fill gives 4.07 G total, 2.21 G with libc excluded. Wall 1.00-1.12 s falls to 0.45-0.50 s | S | `rexx-exec/src/builtin/string.rs:725-727` (`copies_bytes`) | No bench program calls COPIES, so the gate programs see only layout. callgrind.sh's libc-excluded column hides most of this cost (the 3-byte `memcpy` calls are in libc) |
| P2 Array~append quadratic | STILL REPRODUCES. Crate user time 0.02 / 0.07 / 5.4 s at n = 1e3 / 1e4 / 1e5. Oracle 0.00 / 0.00 / 0.01 s, and 0.16 s at 1e6 | S | `dispatch/collection.rs:167` `last_item` (collects `occupied` into a Vec per append) and `:253` `slots_of(..).len()` (`collection.rs:40` clones the slots). The same clone-to-read shape sits in `array/surface.rs` (ITEMS: 6.3 s for 1e5 calls at n=1e5 vs oracle 0.03 s) | heapshape and the gate programs do not append. R10: `append_slot` calls `array_grow`'s `charge_growth`; keep it |
| P3 SYNTAX unwind superlinear | STILL REPRODUCES within the default cap (Ir, depth 2000/4000/8000: 0.24 / 0.48 / 1.35 G; `exit 0` control 0.13 / 0.14 / 0.16 G). Cause found: building TRACEBACK/STACKFRAMES appends one item per frame to a `List`, and `List~append` is O(n) | S | `dispatch/collection/list.rs:136` (`array_slots_owned(handles).len()` clones every handle per append) and `:126` (clones the free stack) | The same fix makes `List~append` linear for programs (listappend 1e4: 0.95 G to 0.15 G Ir). With the fix, unwind is 0.17 / 0.22 / 0.33 G Ir (linear). Wall clock within the cap is small (0.09 s vs 0.08 s at 8000) |

## Setup

Binary: `git archive f7170918c rust interpreter` into `/tmp/claude-1000/p61/se2/src`, touched, then
`CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/p61/se2/target memcap 8G cargo build --release -j 4 -p rexx-exec --bin rexx-run`
from `src/rust`. `build.log` has `Compiling rexx-exec`. Copied to `/tmp/claude-1000/p61/se2/bin/rexx-run`,
sha256 `5bfa31112b586ca1a1a4a7382b44627603720e14d626e5a391de47bf451664aa`.

Scratch builds were made in the same source copy and target dir, each with a `Compiling rexx-exec` line. Afterwards the
sources were restored and `cmp`'d against `git show f7170918c:...`:
* `bin/rexx-run-doubling`, sha256 `06bd516687609f0def32faa041b7bed3642e5a93ab8331b09ef6a9afca906ac8`.
  The patch is `/tmp/claude-1000/p61/se2/copies-doubling.diff`.
* `bin/rexx-run-coll`, sha256 `75f78ea0bde709ed040acde76f68804a91430f82af93cf2f20718521e6c74d56`.
  It has `list.rs` `array_slots` in place of `array_slots_owned` at `:126,:136`, and `last_item` as an `rposition` over the
  borrowed slots. The patch is `/tmp/claude-1000/p61/se2/collections-scratch.diff`.

Runner `/tmp/claude-1000/p61/se2/tools/both.sh FILE` runs these from fresh `mktemp -d` dirs under `runs/`:
* crate: `memcap 2G timeout -k 5 20 bin/rexx-run FILE`;
* oracle: `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.

The oracle's outputs are written outside its run dir.

Callgrind uses `tools/cg.sh`, which runs `rust/bench-programs/callgrind.sh`'s `one()` invocation and `cgsum.py` on a
single program, because callgrind.sh only accepts its fixed program list. Rows are total / libc / ld / total less both.
Wall clock uses `tools/wall.sh`, which interleaves crate and oracle runs and reports `/usr/bin/time` elapsed and user
seconds. Probes are in `/tmp/claude-1000/p61/se2/probes/`. None is concurrent.

## Item 7: a user STRING answering an Array or another object

Probe `i7a.rex`:

    o = .k~new
    say o
    say 'x' o
    say length(o)
    say o || '!'
    ::class k
    ::method string
      return .array~of('a', 'b', 'c')

* crate rc 0, stderr empty, stdout `an Array` / `x an Array` / `8` / `an Array!`
* oracle rc 0, stderr empty, stdout `a` `b` `c` / `x a` `b` `c` / `5` / `a` `b` `c!`. The items are joined by newlines.

Probe `i7d.rex` (truth):

    o = .k~new
    if o then say 'if true'; else say 'if false'
    ::class k
    ::method string
      return .array~of(1)

* crate rc 222, stdout empty. stderr: `2 *-* if o`, `Error 34 running .../i7d.rex line 2:  Logical value not 0 or 1.`, `Error 34.1: ... found "an Array".`
* oracle rc 0, stdout `if true`, stderr empty.

Probe `i7c.rex` (the older ledger line; STRING answers `1`):

    o = .k~new
    if o then say 'if true'; else say 'if false'
    do while o; say 'while ran'; leave; end
    n = 0
    do until o; n = n + 1; if n > 2 then leave; end
    say 'until n' n
    ::class k
    ::method string
      return 1

Both rc 0 with identical stdout `if true` / `while ran` / `until n 1`, stderr empty. This is FIXED by `d27a9d441`
("Phase 6.1 Task 4a: one truth judgment", `git log -S truth_user_string -- rust/corpus`).

Non-Array answers. Each probe's STRING returns `.directory~new` (`i7b`, `i7e`, `i7f`, `i7g`) or another instance (`i7h`):

| probe | crate | oracle |
|---|---|---|
| `i7b` `say o` | rc 0, `a Directory` | rc 168, stderr `Error 88.909: Argument 1 must have a string value.` (traceback through `Stream~SAY`) |
| `i7e` `say length(o)` | rc 0, `11` | rc 0, `139937618269872` |
| `i7f` `say 'x' o` | rc 0, `x a Directory` | rc 251, `Error 5 ... System resources exhausted.` |
| `i7g` `if o` | rc 222, 34.1 `found "a Directory"` | rc 222, 34.1 `found "The NIL object"` |
| `i7h` STRING answers a `.k2` whose STRING is `'inner'`: `say o`, `say length(o)` | rc 0, `a K2` / `4` | rc 168, 88.909 on `say o` |

The oracle reads a non-string answer as a string object. Its `length` is a pointer-sized number, which is the
Deviation 28 shape. Only the Array answer has a well-defined oracle result.

Root cause: `dispatch/reqstr.rs:165` renders the STRING send's answer with `string_value_text`, and
`value.rs:397` answers `an Array` for an array and the default name otherwise. The Array-joining limb that already
exists (`StringConversion::Bytes`, reached through `required_string_answer`) is not applied to the answer.
`length()` and truth share this path through `required_string_value`.

Not probed: the ledger's rooting-harness gap ("could not detect removing truth push or condition_value push"). It
is a final-review item.

## Item 8: wrong-type native rows that name Phase 9

These are crate-only runs. The oracle is not run because `rust/corpus/oracle-crashes.txt` entry 33 covers this exact
shape: SIGSEGV for `List~ITEMS` and others, with garbage answers under Deviation 28. Template, with `.X~method('M')`
varied:

    say .t~new~go(.directory~method('ITEMS'))
    ::class t
    ::method go
    use arg m
    return self~run(m)

Command: `memcap 2G timeout -k 5 20 bin/rexx-run FILE` from a fresh dir. Every run below has stdout empty.

| method | rc | stderr |
|---|---|---|
| Directory, StringTable, Stem, Set, Bag, Relation, Table `ITEMS` | 120 | `rexx-exec: method "ITEMS" of class "T" is not implemented (Phase 9)` |
| MutableBuffer `LENGTH` | 120 | `rexx-exec: method "LENGTH" of class "MutableBuffer" is not implemented (Phase 9)` |
| Message `SEND` | 120 | `rexx-exec: method "SEND" of class "Message" is not implemented (Phase 9)` |
| Message `START` (not in the queued item) | 120 | `rexx-exec: method "START" of class "Message" is not implemented (Phase 9)` |
| Message `REPLY` (not in the queued item) | 120 | `rexx-exec: method "REPLY" of class "Message" is not implemented (Phase 9)` |
| Message `NOTIFY` | 168 | `Error 88.901: Missing argument; argument notification target is required.` (argument check first) |
| control: List `ITEMS` | 120 | `rexx-exec: a message send to a value that is not a list is not implemented` (the right refusal) |

Adjacent success, `i8_mb_subclass_noinit.rex` (`::class b subclass mutablebuffer` / `::method init`, then `length`, `append('xy')`,
`string`): both rc 0, stdout `0` / `xy`, stderr empty. So a missing buffer state means a wrong-type receiver.

Root cause: `dispatch/hash.rs:183` `not_this_task` maps any receiver with a class id to `Loud::native_method` (owner Phase 9),
including a non-hash receiver. The same mapping happens at `dispatch/buffer.rs:396,407` (`buffer_state`/`buffer_state_mut`
None arm) and at `dispatch/object_protocol.rs:1001,1114,1196`, where Message's `native_entry` misses. The fix is to route
these to `Loud::receiver_class`. `relation.rs` and `stem.rs` go through `not_this_task` and need no separate change.

## Item 9: the `Literals` rows

`i9a.rex` follows `program_for`'s shape:

    numeric digits 9
    numeric form scientific
    tab = "09"x
    say "AB"
    say self~runDynamicSource("return" self~hex("41" || tab || "42"))

Both rc 159, stdout `AB`. Both stderr: `5 *-* say self~runDynamicSource(...)`, `Error 97 ... line 5:  Object method not found.`,
`Error 97.1:  Object "SELF" does not understand message "HEX".` The two are byte-identical apart from the path.

`i9b.rex` is the test-case shape, using the framework's `runDynamicSource` body (`ootest/framework/OOREXXUNIT.CLS:1223`) and
`Literals.testGroup`'s `hex`/`bin`:

    say .t~new~go
    ::class t
    ::method go
      tab = "09"x
      return self~runDynamicSource("return" self~hex("41" || tab || "42")) self~runDynamicSource("return" self~bin("0100" || tab || "0001"))
    ::method hex
      return '"' || arg(1) || '"x'
    ::method bin
      return '"' || arg(1) || '"b'
    ::method runDynamicSource
      use strict arg code, parentPackage = (self~class~package)
      r = .routine~new(parentPackage~name, code, parentPackage)
      return r[]

* crate rc 120, stdout empty, stderr `rexx-exec: method "NEW" of class "Routine" is not implemented (Phase 9)`
* oracle rc 0, stdout `AB A`, stderr empty.

`i9c.rex` is the same without the package argument (`.routine~new('x', code)`). Both rc 0, stdout `AB`, stderr empty.

Root cause: `dispatch/construct.rs:172` refuses `args.len() > 2` for `Routine~new`/`Method~new`. `new_file_context` at
`construct.rs:212` already resolves a Package/Routine/Method/`PROGRAMSCOPE` context for `newFile` and is the reuse candidate.
The harness half is `tests/assertions.rs` `program_for` (`:93`), which emits top-level code with no `self` test case.

## P1: COPIES fills byte by byte

The program comes from the Task 5a report (`task-5a-report.md:78`): `loop999.rex` is
`do i = 1 to 1000000; x = copies('abc', 333) || i; end` / `say 'done' i`. It was rewritten here as `probes/loop999_1000000.rex`,
plus `loop999_100000.rex` at N = 100 000. The ledger's 14.4 G / 4.07 G were `perf stat` instructions for the whole process,
libc included.

Callgrind (`tools/cg.sh`). Columns are total Ir / libc / total less libc and ld. Every run was rc 0 with stdout `done N+1`:

| program | HEAD | doubling fill |
|---|---|---|
| loop999 N=1e5 | 1 467 986 791 / 779 391 270 / 688 249 316 | 507 087 610 / 235 691 318 / 271 050 113 |
| loop999 N=1e6 | 13 680 937 843 / 7 297 369 818 / 6 383 221 965 | 4 071 938 128 / 1 860 370 013 / 2 211 221 950 |
| `copies('-', 80)` x1e6 | 4 114 321 764 / 1 734 431 004 / 2 379 544 569 | 2 103 321 891 / 638 431 013 / 1 464 544 699 |
| `copies('abcdefgh', 2)` x3e5 (worst case for doubling) | 589 455 606 / 112 758 052 / 476 351 349 | 590 355 949 / 112 758 595 / 477 251 189 (+0.19%) |

At HEAD, `__memcpy_avx_unaligned_erms` is 40% of loop999's Ir: one libc call per 3-byte piece. As a result, callgrind.sh's
gated column (libc excluded) understates this item. Its gain is 6.38 G to 2.21 G there and 13.68 G to 4.07 G in total.

Wall clock, loop999 N=1e6, 5 interleaved runs each, sorted elapsed s:
`rexx-run` 1.00 1.02 1.07 1.07 1.12; `rexx-run-doubling` 0.45 0.46 0.46 0.47 0.50.

Root cause: `builtin/string.rs:725-727`, `for _ in 0..count { out.extend_from_slice(string) }`. The scratch fix does one
`extend_from_slice`, then `extend_from_within(..min(len, total-len))` until the length reaches `total`.

## P2: Array~append quadratic

Probe `append_N.rex`:

    a = .array~new
    do j = 1 to N; a~append(j); end
    say a~items a[N]

Wall clock, 5 interleaved runs (crate `memcap 2G timeout -k 5 120`, oracle standard wrapper). Every run was rc 0, stdout
`N N`, stderr empty:

| n | crate elapsed / user s | oracle elapsed / user s |
|---|---|---|
| 1e3 | 0.05-0.06 / 0.02 | 0.00 / 0.00 |
| 1e4 | 0.08-0.13 / 0.07-0.08 | 0.00-0.01 / 0.00 |
| 1e5 | 5.44-5.57 / 5.40-5.53 | 0.01-0.02 / 0.01 |
| 1e6 | not run (about 550 s projected) | 0.17 / 0.16 (1 run) |

Callgrind at HEAD: 1e3 140 749 485 total / 68 842 209 less libc; 1e4 1 784 008 374 / 886 592 225. The 1e5 run was stopped
after more than 10 minutes. In the profile, `collection::occupied` is about 50% (it collects every occupied offset into a Vec
per append, via `last_item`, `collection.rs:167`). `memcpy` is 45%, called from `slots_of` (`collection.rs:40`, which clones
the slots via `array_slots_owned`) at `append_slot`'s `slots_of(..).len()` (`collection.rs:253`).

Scratch `rexx-run-coll` fixes `last_item` only: append 1e4 goes to 941 674 206 total / 77 672 756 less libc, so `occupied` is
gone. libc is still 863 M, all from `slots_of` (10 000 calls). The `:253` clone also needs replacing. The same
clone-to-read pattern is in `array/surface.rs` (`:76`, `:88`, `:141`, `:158`, `:175`, `:404`, `:427`, `:472`, `:502`, `:544`),
`array/sort.rs:281,302` and `collection/queue.rs:175`. A loop of `a~items` over a 1e5-slot array takes crate 6.36 s vs
oracle 0.03 s (`arrayitems_100000.rex`, one run each, both rc 0, same stdout). Plain `a[j]` reads and writes are linear
(`arrayread_100000`: 0.09 s vs 0.03 s), because they take another path.

## P3: SYNTAX unwind superlinear

The queued program, with the depth N varied (`unwind_N.rex`; the handler also prints `condition('C') rc`):

    signal on syntax name h
    say f(0)
    exit
    h: say 'h' condition('C') rc
    ::routine f
      use arg n
      if n = N then exit
      return f(n+1)

The controls are `unwindexit_N` (`exit 0`, no trap) and `unwindnotrap_N` (bare `exit`, no trap).

At HEAD the crate's call cap is `MAX_ACTIVATION_DEPTH = 10_000` (`run/call.rs:148`). One run each, wall elapsed / user s:

| depth | unwind crate | unwind oracle | unwindexit crate / oracle | unwindnotrap crate / oracle |
|---|---|---|---|---|
| 5000 | rc 0 `h SYNTAX 44`, 0.07 | rc 0 `h SYNTAX 44`, 0.02 | rc 0 `0` / rc 0 `0` | rc 212 / rc 212 (Error 44 traceback, both) |
| 10000 | rc 0 `h SYNTAX 11`, 0.11 | rc 0 `h SYNTAX 44`, 0.04 | rc 245 / rc 0 `0` | rc 245 / rc 212 |
| 20000-80000 | rc 0 `h SYNTAX 11`, 0.10-0.12 | rc 0 `h SYNTAX 11`, 0.07 | rc 245 / rc 245 | rc 245 / rc 245 |

The oracle's own limit lies between 10 000 and 20 000. The crate refuses at 10 000, where the oracle still answers. This is a
cap difference, not this item. So at HEAD the item's 20k-1M depths cannot run. The superlinearity shows within the cap in
callgrind (total / libc / less libc), every run with the stdout shown:

| depth | unwind HEAD | unwind `rexx-run-coll` | unwindexit HEAD | unwindnotrap HEAD |
|---|---|---|---|---|
| 2000 | 235 789 971 / 147 773 201 / 87 670 565 | 173 299 805 / 85 539 378 / 87 414 262 | 130 929 627 | 165 550 170 (rc 212) |
| 4000 | 479 585 361 / 362 259 244 / 116 979 912 | 224 162 102 / 107 348 346 / 116 467 591 | 140 731 810 | 208 497 530 (rc 212) |
| 8000 | 1 352 760 087 / 1 176 816 196 / 175 597 686 | 325 570 139 / 150 650 618 / 174 573 501 | 160 333 853 | 294 204 056 (rc 212) |

Only the libc part is quadratic. At depth 8000, 1.02 G of 1.03 G `memcpy` Ir comes from `list_insert_at` (16 000 calls, two
per frame). `condition.rs:584-615` (`condition_frames`) appends each frame and each trace line to the TRACEBACK and
STACKFRAMES `List`s. `List~append` goes through `list_next_handle` (`collection/list.rs:124`), which clones the whole handles
array (`:136`) and the free stack (`:126`) to read a length or the last element. With those two borrowed (`rexx-run-coll`),
unwind becomes linear: 173 / 224 / 326 M. `List~append` x1e4 goes from 952 487 549 to 149 578 541 Ir.

Wall clock within the cap is small, 5 interleaved runs each:

| program | crate elapsed s | oracle elapsed s |
|---|---|---|
| unwind_2000 | 0.06-0.08 | 0.01 |
| unwind_8000 | 0.09-0.10 | 0.03 |
| unwindexit_8000 | 0.05-0.07 | 0.01 |

`rexx-run-coll` unwind_8000 takes 0.07-0.09 s. The cost bites only past the cap, which is the cap-lifted build the item came
from. That build was not rebuilt here.
