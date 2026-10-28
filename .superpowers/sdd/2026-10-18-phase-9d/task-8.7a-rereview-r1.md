# Task 8.7a fix round 1: re-review

Reviewed `cc1e48c0c` (code) and the records to `4977083c6`, against the fix brief
`task-8.7a-fix-r1.md` and the findings of `task-8.7a-review.md`. Probes are under
`t87a-rr1-scratch/` in this directory.

**Verdict: APPROVED.** Every finding of the brief is ADDRESSED. New findings: Critical none,
Important none, Minor 1 (a false sentence in the report).

## Verdicts per finding of the brief

| brief item | verdict |
|---|---|
| 1. Important 1, traced message result dead before its `>M>` line | ADDRESSED |
| 2. Minor 1, `operand_number`'s doc on `requested_number` | ADDRESSED |
| 3. Minor 2, crate-only test pinning `equals` with a reference argument | ADDRESSED |
| 4. `oracle-crashes.txt` entry 63 | ADDRESSED |

## What I ran

* **Builds.** `git archive` of `030af45a1` and `cc1e48c0c` (`rust/` and `interpreter/`) into
  `t87a-rr1-scratch/{base,head}-src`, each with an untracked `crates/rexx-exec/examples/stress.rs`
  (the implementer's `probes/stress-runner-main.rs`: `run_program_collect_every_alloc` on one
  file), built with `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx` and
  `-p rexx-exec --example stress` into `/tmp/claude-1000/t87a-rr1-target-base` and `-head`, exit 0
  on all four.
* **Pinned binaries.** The `.text` of my `030af45a1` build is byte-identical to
  `ooRexx-p9-pin/cg-9d/t8.7a/rexx-head-030af45a1` (sha256 `f71f0089...` on both). The `.text` of my
  `cc1e48c0c` build is not byte-identical to `rexx-head-cc1e48c0c` (16 bytes shorter in the pin),
  so I compared them by function: the same symbol set with the same sizes (`nm -S`; the only
  difference is which name identical-code folding keeps for one 0x13-byte `drop_slow`), and
  `objdump -d` of both, sorted per function with RIP-relative displacements and `int3` padding
  normalised, has no remaining difference. `message_term` is `0x10cd` and `trace_sent` `0x121`
  bytes in both. The pin carries `cc1e48c0c`'s code; only the layout differs.
* **Runner.** `t87a-rr1-scratch/bin/run5 FILE`: the oracle under the standard wrapper, base and head
  normal binaries and base and head stress runners under `memcap 2G timeout -k 5 20`, each from a
  fresh empty directory, stdout, stderr and rc compared with the oracle separately. Result lines
  below are `name o=<rc> b=... h=... bs=... hs=...` with `same` or the descriptors that differ.
* **Tests** (archived copies, own target dirs): `collect_stress` at `030af45a1` with the new
  witness copied in, at `cc1e48c0c`; `rexx-exec --lib equals` and a mutant of it at `cc1e48c0c`;
  `rexx-parse --test sourceline_oracle` and a mutant of its data; `cargo fmt --all --check`.
* **Perf.** `bench-programs/callgrind.sh -r 1 -j 4 -B 0.5 -p "define startup interpret"` over my two
  builds, and a reading of the round's `callgrind.txt`, `callgrind-own-delta.tsv` and the raw
  outputs' `cmd:` lines.
* **Memory.** One traced loop over the changed path at two counts, both binaries.

## Item 1: the traced message result

**The review's probes at head.**

    c4f o=0 b=0:same h=0:same bs=120:out,err, hs=0:same
    c4n2 o=0 b=0:same h=0:same bs=120:out,err, hs=0:same
    c4o o=0 b=0:same h=0:same bs=101:out,err, hs=0:same
    c4o_r o=0 b=0:same h=0:same bs=0:same hs=0:same
    c4g o=0 b=0:same h=0:same bs=0:same hs=0:same
    c4k o=0 b=0:same h=0:same bs=0:same hs=0:same

`c4o.bs.err`: `panicked at crates/rexx-exec/src/dispatch.rs:1831:9: a live value`. `c4m` differs
from the oracle on stderr only, identically on all four crate runs: the `>O>   ">" => "X"` line
the report names as the `s25` queue candidate (oracle `"V"`), which predates the round.

**The witness red at `030af45a1`.** `trace_reference_value_rooted.rex` at `cc1e48c0c` run directly:

    trace_reference_value_rooted o=0 b=0:same h=0:same bs=101:out,err, hs=0:same

with `dispatch.rs:1831:9: a live value` on the base stress run. `collect_stress` built from the
`030af45a1` archive with only the witness replaced by the round's
(`t87a-rr1-scratch/collect-stress-base-newwit.txt`): `test result: FAILED. 73 passed; 1 failed`,
`the_l0_subset_passes_again_under_collect_on_every_allocation`, panic `dispatch.rs:1831:9`. At
`cc1e48c0c` (`collect-stress-head.txt`): `test result: ok. 74 passed; 0 failed`. I read the
witness's stdout and the oracle's trace: the three new SAY lines print
`The VariableReference class` and `The VariableReference class The VariableReference class`, and
the oracle's stderr carries `>M>   "R" => "an Array"`, `>M>   "[]" => "an Array"` (inside the
INTERPRET) and `>M>   "R" => "1"`, so the method, SAY, INTERPRET and `v = 1` paths each run.

**Hunt beyond `s01`-`s26`.** First a routing check: with an `eprintln!` at the top of
`message_term` in a scratch copy (restored and confirmed by `cmp` against `cc1e48c0c`), a traced
`w = o~fresh(1)` (`p3/e1.rex`) and `w = o~fresh` (`p2/q05.rex`) never reach `message_term`, while
`w = o~r3(>v)` (`p3/e2.rex`) does (`[message_term R3]`). A traced send whose shape the IR compiles
natively goes through `deliver_sent`, which already rooted, so the probes below each pass a `>v`
argument (or a scope override) to reach the tree path. Methods return `>loc` for a fresh local
`loc = .array~new`.

    r04 o=0 b=0:same h=0:same bs=101:out,err, hs=0:same     forward to a method returning >loc
    r06 o=0 b=0:same h=0:same bs=101:out,err, hs=0:same     o~fresh(>v) == o~fresh(>v)
    r07 o=0 b=0:same h=0:same bs=101:out,err, hs=0:same     o~r(o~fresh(>v)), a result as argument
    r08 o=0 b=0:same h=0:same bs=101:out,err, hs=0:same     o~~fresh(>v)~fresh(>v)
    r09 o=0 b=0:same h=0:same bs=101:out,err, hs=0:same     o[>v], and o[>v]~class in SAY
    r10 o=0 b=0:same h=0:same bs=120:out,err, hs=0:same     SELECT WHEN
    r11 o=0 b=0:same h=0:same bs=120:out,err, hs=0:same     DO count, IF, DO WHILE, DO UNTIL
    r15 o=0 b=0:same h=0:same bs=101:out,err, hs=0:same     .array~of(o~fresh(>v), o~fresh(>v))
    r16 o=0 b=0:same h=0:same bs=101:out,err, hs=0:same     sendWith, send
    r19 o=0 b=0:same h=0:same bs=120:out,err, hs=0:same     two results in one concatenation
    r20 o=0 b=0:same h=0:same bs=120:out,err, hs=0:same     o~fs(>v)~fresh(>v), a chain
    r21 o=0 b=0:same h=0:same bs=120:out,err, hs=0:same     INTERPRET of a setter and a send
    r22 o=0 b=0:same h=0:same bs=120:out,err, hs=0:same     PARSE VALUE
    r23 o=0 b=0:same h=0:same bs=120:out,err, hs=0:same     o~fresh(>v)~~class, o~~fresh(>v)
    r24 o=159 b=159:same h=159:same bs=120:err, hs=159:same message instruction chain, RESULT
    r25 o=0 b=0:same h=0:same bs=101:out,err, hs=0:same     internal function returning a send
    r27 o=0 b=0:same h=0:same bs=120:out,err, hs=0:same     ||, and a parenthesised send
    sc  o=0 b=0:same bs=101:out,err, hs=0:same              self~fresh:super(>z), self~fresh:.k(>z)
    t05 o=0 b=0:same h=0:same bs=101:out,err, hs=0:same     a Rexx .traceOutput destination that
                                                            allocates per line, over sends,
                                                            setters, []=, == and RESULT

(`sc`'s `h` column was taken with the instrumented scratch binary and is not a result; its `hs`
uses the clean head's stress runner.) Green at base and head under stress, and identical to the
oracle: `trace r` and `trace a` over the same sends (`r13`, `r14`), the untraced form (`r26`), the
message-assignment setter `>M>   "X=" => "an Array"` with and without a reference argument, `[]=`
and `INTERPRET` of a setter (`p5/t01`-`t04`; `run.rs:2981` renders the setter's answer, and no
probe of mine reddens it at either commit), and `p2/q01`-`q22`, the same shapes without a
reference argument. No send path I found still renders unrooted at head.

**Traced output unchanged.** `message_term` now calls `trace_sent` under
`self.trace_mode().intermediates`, and `trace_sent` roots the value and calls
`intermediate_text`, which tests the same flag again before rendering, so the line is the same
one written at the same point. By running: `p6/u_i`, `u_r`, `u_a`, `u_c`, `u_n` (the same program
under TRACE I, R, A, C and N: plain sends, a send with arguments built from sends, the message
instruction and RESULT, `o~~n`, `'abc'~~length~~reverse`, `.array~new~~append(1)~~append(o)`, a
setter, `o~~s(7)`, `o[]`, INTERPRET, IF, DO, an object with `STRING`, ending in 91.999), and
`p6/u_dest` (a Rexx `.traceOutput` destination collecting every line): every one `same` against
the oracle on all four crate runs, rc 165 for the five TRACE programs and 0 for `u_dest`, and
base and head byte-identical on stdout, stderr and rc. Every `r*`, `q*` and `t*` probe above has
`b` and `h` both `same`.

**Untraced path.** The changed block is entered only when `trace_mode().intermediates`; before, the
same flag was tested inside `intermediate_text`, which returned `None`. My callgrind spot check
over my own builds (`030af45a1` against `cc1e48c0c`, libc subtracted): `define +0.0006`,
`startup -0.0000`, `interpret -0.0000`.

**Memory.** `p7/m<n>.rex`, `w = o~r(>v)` and `o~r(>v)` under TRACE I, stderr discarded:
`base n=20000 8.29 s 41732 KB`, `head n=20000 8.32 s 42184 KB`, `base n=200000 82.94 s 41596 KB`,
`head n=200000 83.05 s 41772 KB`. The temporary root `trace_sent` pushes does not accumulate.

## Item 2: the doc block

`git show cc1e48c0c:rust/crates/rexx-exec/src/value.rs | sed -n 770,812p`: `requested_number`
carries only its own doc ("`RexxInternalObject::numberValue` for an argument ..."), and
"[`Interp::to_number`] for the receiver of an operator ..." sits on `operand_number`.

## Item 3: the `equals` test

`tests::a_references_equals_argument_is_its_referenced_values_string` at `cc1e48c0c`: `ok`. Its
program run on the oracle and the crate (`eq/e.rex`): oracle `0 1 0` / `NOSTRING`, base and head
`1 0 1` / `1`, so it pins this crate's answer where the oracle differs. It can fail: a mutant in the
scratch copy making `request('STRING')` of a reference fall to `make_string_or_none`
(`dispatch/reqstr.rs:437`, `Body::VarRef(_) => return self.make_string_or_none(value)`) gives

    test tests::a_references_equals_argument_is_its_referenced_values_string ... FAILED
      left: "0 1 0\nNOSTRING\n"
     right: "1 0 1\n1\n"

which is the oracle's answer; the file was restored and confirmed by `diff` against
`cc1e48c0c`. `phase-4-exclusions.txt` is not touched by `cc1e48c0c`.

## Item 4: crash entry 63

`c63/c63.rex` (`q = >y`, `trace i`, `q~value = 3`, `say y`), oracle under the standard wrapper,
each from a fresh empty directory: `run 1 rc=139 out=0`, `run 2 rc=139 out=0`, `run 3 rc=139
out=0`, stderr ending `>L>   "3"` / `>A>   "3"`. Without TRACE (`c63/n.rex`) the oracle prints `3`
at rc 0. The pinned head prints `3` at rc 0, with `>V>   Y => "3"` and `>>>   "3"` for `say y`.
The entry has entry 62's shape (heading, indented program, an `Effect:` paragraph naming the date,
wrapper, run count and this crate's answer, `No upstream ticket.`), and each statement in it
matches these runs.

## Other checks

* `rexx-parse --test sourceline_oracle` at `cc1e48c0c`: `ok. 1 passed`. With the witness's header
  changed to `count 37` in the scratch copy: `FAILED`, `trace_reference_value_rooted: header count
  disagrees with the captured lines`; restored and confirmed by `cmp`. The test reads every
  `corpus/lang/` program, this one included.
* `cargo fmt --all --check` on the `cc1e48c0c` archive: exit 0.
* The witness is in `rust/corpus/phase-9.txt` (line 417).

## Callgrind figures

`docs/superpowers/records/2026-10-18-phase-9d/t8.7a/r1/callgrind.txt`: the largest
`head -libc d%` against `rexx-base-81f4bf250` is `startup +0.0016`. `callgrind-own-delta.tsv`:
the largest own delta is `define +0.0022` libc subtracted; its two inputs agree with the tables
they cite (`define` head `-libc` `880121350` in `r1/callgrind.txt`, `880101739` in
`t8.7a/callgrind-final.txt`). pingguard `-0.0000`, pingsem `-0.0001`, pingmsg `+0.0002`. Every
program is within +0.05% libc subtracted. The `cmd:` lines of the raw outputs in
`ooRexx-p9-pin/cg-9d/t8.7a/r1` (`grep -h "^cmd:" *.cg | awk '{print $2}' | sort | uniq -c`) name
only `rexx-base-81f4bf250` and `rexx-head-cc1e48c0c`, in equal numbers.

## Findings

### Critical

None.

### Important

None.

### Minor

1. **A false sentence in the report.** The "Fix round 1" section of `task-8.7a-report.md` calls
   `Interp::message_term` "the tree path every traced send takes". Only a send whose shape the IR
   does not compile natively (`native_message_instruction` and `native_shape` in
   `ir/compile.rs`; a reference argument and a scope override are two such shapes) reaches it; every other traced send goes through `deliver_sent` and `trace_sent` in the
   IR. Probe: with an `eprintln!("[message_term {}]", ...)` at the top of `message_term` in a
   scratch copy, traced `w = o~fresh(1)` (`p3/e1.rex`) prints no `[message_term FRESH]` line, and
   `w = o~r3(>v)` (`p3/e2.rex`) prints `[message_term R3]`; consistently, `e1` is green under
   stress at `030af45a1` and `e2` is red there (`bs=101`). The code comment the round wrote ("The
   tree path's `>M>` site") is right; only the report's sentence is wrong. A false sentence is
   deleted (global constraints, Prose).
