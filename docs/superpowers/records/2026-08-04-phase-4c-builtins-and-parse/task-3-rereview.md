# Task 3 re-review: fix round 1 (`63a9ea9f..c0889d74`)

Scoped re-review of the fix diff and its neighbourhood only. Commits after
`c0889d74` (`f1db85b6`, and `6f4d0745` which landed during this review) are
documentation-only and were confirmed as such (`git diff --stat`: three `.md`
/ `.txt` files, no source).

**Verdict: CHANGES-REQUESTED.** Both Criticals are genuinely addressed on the
paths they were reported on, and every number in the report reproduces. But
the round introduced one new value divergence in its own new code, left the
same byte defect live on a second output channel while reporting it fixed,
and recorded two justifications that do not survive a single experiment --
one of which a later task is meant to act on.

---

## 1. Re-run verification (each status read unpiped; real tree never mutated)

| Command | Exit | Result |
|---|---|---|
| `cargo test --offline --workspace --no-fail-fast` | **0** | 1058 passed, 0 failed |
| `cargo fmt --all --check` | **0** | clean |
| `cargo clippy --offline --workspace --all-targets -- -D warnings` | **0** | clean |
| `REXX_CORPUS_GATE=1 cargo test --offline -p rexx-exec --test corpus` | **0** | `mode: STRICT (the gate)`, `42 of 42 matching` |

Differential re-runs I built or re-ran myself, oracle wrapped exactly as
mandated, each probe from a fresh `mkdir`ed subdirectory:

| Corpus | Programs | Mismatches |
|---|---|---|
| 256-byte pad sweep (mine, `left('ab',5,'NN41'x)` for every `NN`) | 256 | **0** |
| C, byte-widened (`scratchpad/wchunks`, re-run) | 18,576 | **0** |
| D, null-string singleton (`scratchpad/probes/singles.txt`, re-run) | 234 | **20**, all the recorded TRANSLATE gap |
| 4a/4b raised-condition spot-check (mine, no builtins) | 30 | 3, all pre-existing `loud` paths |
| Trace-channel byte probes (mine) | 6 | **3** -- see F-N2 |
| `VERIFY` option-byte probes (mine) | 16 | **3** -- see F-N1 |

`git status --porcelain` is empty. **Nothing tracked was modified at any
point.** Every mutation experiment ran in `git archive`-extracted copies
under my scratchpad (`scratchpad/cur`, `scratchpad/parent`), never in the
working tree.

---

## 2. Were the original findings addressed?

### Critical 2 (byte-wrong substitutions) -- addressed on the report path,
### **not** on the trace path

**The rule, verified over the whole range rather than sampled.** I drove all
256 byte values through a 40.23 message (`say left('ab',5,'NN41'x)`, whose
two-byte pad raises for every `NN`) and extracted what the oracle printed
between `found "` and `A".`:

```text
rendered as ? :  00-08  0b-0c  0e-1f
rendered raw  :  09-0a  0d     20-ff        (0x7f and all of 0x80-0xff raw)
```

That is exactly the rule the report recorded and exactly what `displayable`
implements. The same 256 programs run differentially are **0 mismatches**
on stdout, stderr and exit status.

The fix is in `Raised::report`, applied once over the whole report, which is
the right place: `Activity::display` (`concurrency/Activity.cpp:1414`) sends
each line through `displayUsingTraceOutput` -> `processTraceInfo` ->
`RexxString::stringTrace`, and the rule is per byte and leaves `\n` alone, so
sanitising the concatenation is the same bytes. Confirmed on multi-line
reports: a traceback through two nested routines with control and high bytes
in each echoed line is byte-identical (`rr1/rep2/r2`), as is a `1/0` whose
clause carries a control byte inside a comment (`r3`).

**The clause-echo claim is half true.** The *error report's* echo is fixed --
`say copies('a<0x01><0xff>b','x')` is byte-identical now and was not before.
But `processTraceInfo` is also the sink for every **live TRACE line**, and
that path was not touched. See F-N2.

### Important 3 (TRANSLATE gap row) -- row added, producer list complete, **recorded reason wrong**

The `KNOWN GAP: TRANSLATE` row exists in `phase-4-exclusions.txt` with the
C++ citation, both measured lines and the full producer list.

**The nine producers are complete, checked by scanning the oracle rather
than by taking the list.** Every `return GlobalNames::NULLSTRING` in the
files implementing the 23 names accounts for exactly: `center`/`centre`
(`StringClassSub.cpp:77`), `delstr` (:133), `left` (:258), `right` (:495),
`strip` (:584), `copies` (`StringClassMisc.cpp:298`), `substr`
(`support/StringUtil.cpp:86`,`:140`) and `space` (`StringClassWord.cpp:121`).
`insert`, `overlay`, `reverse`, `changeStr`, `upper` and `lower` have no such
return, matching the report's measured non-producers.

**The scope claim also holds.** `!= GlobalNames::NULLSTRING` appears exactly
once under `interpreter/classes/` (`StringClassMisc.cpp:730`, `translate`).
The only other identity comparisons against the singleton in the whole
interpreter tree are `PackageClass.cpp:575` (a source line) and
`Activity.cpp:1448` (a program name), neither on a string-BIF path.

The *reason* recorded for keeping the status row `implemented` does not
survive one experiment. See F-N4.

### Important 4 (sweep alphabet) -- addressed

Corpus C exists, and its reported alphabet matches the corpus: 18,576 lines,
all 23 names present, and every literal the table names is really there
(`'ff'x` 7,981 lines, `'fffefd'x` 1,161, `'80'x` 3,081, `'7f'x` 1,281,
`'00'x` 4,251, `'01'x` 4,131, `'09'x` 1,161, `'0a'x` 2,766, `'61ff62'x`
1,161, `'6100620a63'x` 1,161, `'001f7f80ff'x` 1,161, `'00'x||'L'` 64,
`'L'||'00'x` 64). Re-run: **18,576 programs, 0 mismatches.** The plan file
also gained a standing rule requiring a byte alphabet in every probe set,
which is the right shape of fix.

Two overstatements, both minor: C is described as *adding* to A's alphabet,
but `'banana'`, `'aXbXc'` and `'a b  c '` appear zero times in C, so C is a
separate narrower corpus rather than A widened; and D's row claims "every
optional-string position of all 23 names" (see F-N5).

### Critical 1 (large result) -- residue accepted; recorded diagnosis is **two-thirds sound**

The abort is still reachable; that is accepted and owned. What I checked is
whether the recorded diagnosis is safe for a later task to act on.

**(a) `resolve_and_run_call` renders every argument unconditionally --
TRUE.** `run.rs:3300` does `let rendered = self.to_text(argument.value())
.to_vec();` then `self.trace_argument(...)`, and `trace_argument`
(`trace.rs:565`) returns immediately unless `trace_mode().intermediates`. A
`tracing_intermediates()` helper already exists two functions away. The
assignment `>>>` site (`run.rs:976`) has the identical shape.

**(b) The experiment returns 300000000 and 400000000 to rc 0 -- TRUE, but
"no size aborts at all" is false.** I re-applied exactly the described change
in an extracted copy and rebuilt. At `ulimit -v 1048576`:

```text
N          oracle    HEAD        experiment
300000000  rc 0      SIGABRT     rc 0
400000000  rc 0      SIGABRT     rc 0
450000000  rc 0      SIGABRT     rc 0
500000000  rc 0      Error 5     Error 5 (rc 251)
```

But four one-line neighbours **still SIGABRT with the experiment applied**,
at the same 400 MB, where the oracle returns rc 0 -- see F-N3.

**(c) The early Error 5 is the 512 MiB stack reservation -- TRUE.** Raising
the limit by exactly 512 MiB (1048576 -> 1572864 KB) with the *unmodified*
HEAD binary:

```text
N           ours @ 1 GiB    ours @ 1.5 GiB
400000000   SIGABRT         rc 0
500000000   Error 5 rc 251  rc 0
600000000   Error 5 rc 251  SIGABRT
700000000   Error 5 rc 251  SIGABRT
```

The threshold moves, and it moves by the reservation. The arithmetic is
consistent throughout: with ~500 MiB of usable address space at 1 GiB, one
400 MB buffer fits and two do not (SIGABRT), and a single 500 MB buffer does
not (Error 5); with ~1012 MiB at 1.5 GiB, 500 MB twice fits and 600 MB twice
does not. Nothing about an allocator threshold is involved, as the row says.

---

## 3. New findings

### F-N1 (Important) -- `verify` is now wrong for an empty reference with a `0x00` option

The round correctly found that `optionArgument`'s `strchr(validOptions,
option)` finds the terminating NUL, so a `0x00` option is accepted. It then
changed `verify` to compute one flag, `let nomatch = option == b'N'`, and
used it for **both** branches. The C++ does not test the same thing in both:

```c
// StringUtil::verify, support/StringUtil.cpp:1305-1320
if (referenceLen == 0)
{
    if (opt == RexxString::VERIFY_MATCH) { return IntegerZero; }
    else { return new_integer(startPos); }        // <-- tests M
}
else
{
    if (opt == RexxString::VERIFY_NOMATCH) { ... } // <-- tests N
```

With a NUL option the two tests disagree, and the crate now answers the
empty-reference branch backwards. Measured:

```text
say verify('abcde','','00'x)          oracle 1   ours 0
say verify('abcde','','00'x,3)        oracle 3   ours 0
say verify('abcde','','00'x||'M')     oracle 1   ours 0
```

`verify('abcde','','M')` -> 0 and `verify('abcde','','N')` -> 1 still agree,
so only the NUL option is affected. The parent commit raised 93.915 for all
three, so this is not a regression from working behaviour -- it is the new
code being wrong in half the cases it was written for, while its own doc
comment and its own new test assert only the half that is right.

**Why nothing caught it:** corpus C contains 384 `verify` programs and **not
one** with an empty reference string; `a_null_option_byte_is_accepted_and_
matches_no_letter` tests `verify('abcde','abc',...)` and `verify('abcde',
'xyz',...)` only. The 18,576-at-zero number is real and does not cover the
change it was produced to validate.

Fix: keep the two tests distinct, as the C++ does -- the empty-reference
branch is `option == b'M'`, the other is `option == b'N'` -- and add
`verify(s,'','00'x)` to the test and to the corpus.

### F-N2 (Important) -- the byte rule is missing from the live TRACE channel, which the report says was covered

The report's own diagnosis names `processTraceInfo` as the sink. That sink
serves both the error report *and* every live trace line; only the first was
fixed. Reproducer with no builtin and no error, three lines:

```text
trace r
say 'p<0x02>q'

  oracle stderr:   2 *-* say 'p?q'   /   >>>   "p?q"
  ours   stderr:   2 *-* say 'p\002q' /   >>>   "p\002q"
```

Also under `trace i` for values that never touch source bytes:
`say left(zz,4,'01'x||'02'x)` traces `>L> "?"`, `>O> "||" => "??"`,
`>A> "??"` on the oracle and the raw bytes here -- while the *error report*
printed by the same program is now byte-identical, which is the tell. Three
of my six trace probes mismatch (`rr1/echo/e2`, `e4`, `e6`); all three are
byte-identical to the parent binary on the trace half, so the defect predates
the round -- but the round reported it fixed, and it is one call to
`displayable` away in `trace.rs`'s render. It has no `KNOWN GAP` row and no
owner.

### F-N3 (Important) -- the recorded F1 diagnosis will close a gap that is still open

The exclusions row says, in capitals, "MEASURED AS THE WHOLE OF THIS CAUSE:
with the argument render placed behind `if self.trace_mode().intermediates`
... no size aborts at all". That is measured over one program shape. With
exactly that change applied and rebuilt, at the mandated 1 GiB limit:

```text
program                                        oracle       experiment
say length(strip(copies('a',400000000)))       rc 0         SIGABRT rc 134
say length(reverse(copies('a',400000000)))     rc 0         SIGABRT rc 134
say length(copies('a',400000000)copies('b',1)) rc 0         SIGABRT rc 134
x = copies('a',400000000); say length(x)       rc 0         SIGABRT rc 134
```

`RUST_BACKTRACE=1` names three distinct sites, none of which the row
mentions:

* `rexx_exec::builtin::string::strip` and `::reverse` -- `required_string`
  (`builtin/string.rs:128`) does `interp.to_text(value).into_owned()`, an
  **infallible** full-size copy of every string argument, on exactly the path
  the gap is about. Every one of the 23 builtins takes this copy.
* `<rexx_exec::Interp>::concat` -- the same shape in the operator.
* the assignment `>>>` render, which the row does name but the experiment
  did not patch, so "no size aborts" was never true of it either.

The hazard is concrete: Task 13 fixes the two trace renders the row names,
re-runs the row's own probe `say length(copies('a',N))`, sees rc 0 at
300000000 and 400000000, and closes a gap that three other one-line programs
still hit. The row needs the sentence narrowed to the probe it was measured
on, and `required_string`/`concat` named.

### F-N4 (Important) -- the reason for keeping `TRANSLATE` at `implemented` is not the harness limitation it is described as

The row states, as "a fact about the harness and not a judgement", that
`builtin-status.txt` "cannot hold a divergence its own probe does not
reproduce". The probe is a choice, not a property. In an extracted copy I
changed `builtin-probes.txt`'s TRANSLATE row to a program that *does*
reproduce it and committed `divergent`:

```text
TRANSLATE  zz = left('abc',0); say '['translate('abcdef','123',zz)']'
TRANSLATE  divergent
```

`cargo test -p rexx-exec --test builtin_status` -> **11 passed, 1 failed**.
`the_status_file_matches_a_live_differential_run` **passes** (the derived
status is now `divergent` and matches), and the `KNOWN GAP: TRANSLATE`
marker requirement (`builtin_status.rs:649`) **passes** on the row this round
added -- exactly as designed. The single failure is
`every_string_builtin_is_implemented`, a hard-coded list the report never
mentions.

So the honest recording is available and costs two edits plus a carve-out in
`STRING_FAMILY`; what is committed instead is a `builtin-status.txt` row
saying `implemented` for a builtin measured divergent, defended by a reason
that is false as written. This is the shape this project has paid for before:
a harness limitation asserted rather than tested, standing in for a decision.

### F-N5 (Minor) -- corpus D's reported scope is wider than the corpus

The table row says D covers "thirteen producers of a null string in every
optional-string position of all 23 names". The file has 18 distinct call
shapes covering **13** names: `abbrev changestr compare countstr insert
lastpos length overlay pos space strip translate verify`. Absent entirely:
`center`, `centre`, `copies`, `delstr`, `left`, `right`, `reverse`, `substr`,
`lower`, `upper`, and `translate`'s own pad position. The 20/234 result and
its attribution are correct -- I re-ran it and every one of the 20 is
`translate` with the singleton in `tablein`, ten producer expressions in two
shapes each -- but the sentence beside it is the licence for "and nothing
else", and it overstates.

### F-N6 (Minor) -- "nine producers" and "ten measured producers" in the same report

F3's heading says nine, the corpus table says "the ten measured producers".
Both are true -- nine *names*, ten *expressions*, because `COPIES` produces
the singleton two ways -- but neither passage says which it is counting.

---

## 4. Things checked and found sound

* **`Interp::text_owned` introduced nothing.** Its only caller outside
  `builtin/string.rs` is `text` itself (`value.rs:47`), which now routes
  through it. All 17 call sites hand over a locally built `Vec`; Rust's move
  semantics make retaining or aliasing the buffer impossible, and no site
  keeps a slice of it. Rooting is unchanged: the value is still created by
  `alloc_with(BehaviourId::STRING, ...)`, and the bytes live on the Rust heap
  either way, so a collection during the allocation cannot invalidate them.
  No `unsafe`, no `Heap::alloc`, no `alloc_with_uncollected`. The one
  behavioural difference is that `changestr` and `delstr` build with
  geometric growth, so the stored buffer can retain spare capacity where
  `text` used to allocate exactly `len` -- RSS only, no correctness effect.
* **`substitute`'s rewrite from `chars` to bytes is behaviour-preserving.**
  `&0` still renders `&0`, an unmatched `&N` still renders `&N`, a bare `&`
  still passes through; the catalogue template is ASCII-safe either way.
* **Every builtin taking an option letter was checked, not just the two
  named.** `builtin-status.txt` lists 23 implemented names; only `STRIP`
  (`BLT`) and `VERIFY` (`MN`) take an option, and both go through
  `option_letter`. `STRIP` is correct for the NUL option in all three
  positions I measured; `VERIFY` is F-N1.
* **The "adds coverage" claim, spot-checked on three of the round's tests
  rather than two.** Each mutation applied alone in an extracted copy, then
  the new test deleted with the mutation kept:

  | Mutation | With the new test | With the test deleted |
  |---|---|---|
  | `substitute` re-introduces `from_utf8_lossy` | FAILED, 326 passed / 1 failed | **ok, 326 passed** |
  | `option_letter` refuses `0x00` | FAILED, 326 passed / 1 failed | **ok, 326 passed** |
  | `text_owned` clones its argument | FAILED, 326 passed / 1 failed | **ok, 326 passed** |

  For the first I also ran the whole workspace with the mutation kept and the
  test deleted: the only failures were four targets that fail identically on
  the *unmutated* extracted copy because the ooTest tree is not in the
  archive. `corpus` still reported `42 of 42 matching`. So each test really
  is the only thing in the suite that catches its defect.
* **4a/4b regression check.** The gate is green at 42/42 in STRICT mode, and
  30 hand-written raised-condition programs with no builtin in them (`1/0`,
  bad `**`, `signal` misses, `IF`/`WHEN`/`WHILE`/`UNTIL` non-logicals,
  `RAISE`, `SIGNAL ON SYNTAX`, `INTERPRET`, `LEAVE`/`ITERATE` outside a loop,
  `PROCEDURE`, `NUMERIC DIGITS 'q'`, traces) are byte-identical on stdout,
  stderr and exit status in 27 of 30. The three that are not are the
  pre-existing loud paths -- `PARSE` (4c), a parenthesised `RAISE ...
  ADDITIONAL` list (Phase 5), routine-not-found (4c) -- and all three are
  byte-identical to the **parent** binary, so nothing in this round moved
  them.
* **Parse-time error reports are unaffected and unchanged** (`rexx-exec: 6.2:
  ...` versus the oracle's full report); identical on the parent binary. Out
  of scope here, but it is the one other place a byte could have been lost.
* **No `from_utf8_lossy` left on a compared path.** The remaining non-test
  sites are `raise_syntax_condition` (used only to parse a number; a
  non-UTF-8 argument yields 33.904 with no substitution), `condition_name`,
  and `Loud` messages, none of which carries arbitrary program bytes into a
  compared byte string.

---

## 5. What has to change

1. **`verify`**: restore the two distinct tests. The empty-reference branch
   is `option == b'M'`; the other is `option == b'N'`. Add
   `verify(s,'','00'x)` (with and without a start position) to the unit test
   and to corpus C, which has no empty-reference `verify` at all.
2. **The trace channel**: either apply `displayable` where trace lines are
   rendered, or add a `KNOWN GAP` row with an owner and correct the report's
   claim that fixing `Raised::report` covered the echo. A three-line
   reproducer is `trace r` / `say 'p<0x02>q'`.
3. **The F1 exclusions row**: narrow "MEASURED AS THE WHOLE OF THIS CAUSE" to
   the single probe it was measured on, and name `required_string`'s
   `into_owned()` and `Interp::concat` as further unguarded full-size copies,
   with the four measured programs. Otherwise the owning task will close it
   early.
4. **`TRANSLATE`'s status row**: either flip it to `divergent` (change the
   probe, carve TRANSLATE out of `STRING_FAMILY` with a reason) or replace
   the recorded justification with the true one -- that the probe was chosen
   and `every_string_builtin_is_implemented` hard-codes the row -- rather
   than "the file cannot hold a divergence its own probe does not reproduce".
5. Correct corpus D's alphabet row to the 13 names and 18 shapes it has, and
   say whether "producers" is counting names or expressions.
