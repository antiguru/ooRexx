# Phase 4c plan review: independent re-verification of every measured claim

Reviewed: `docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md`
Date: 2026-08-04. Reviewer: measurement-verification pass, read-only.

**79 discrete claims checked. 15 REFUTED, 0 COULD-NOT-CHECK, 64 VERIFIED.**
Thirteen of the fifteen refutations trace to **one instrument defect**
(Finding A). The other two are mis-cited locations (`plan.rs:631`,
`phase-4-exclusions.txt:88`). Seven further items are VERIFIED but carry a
caveat that will bite an implementer; each is flagged in place.

---

## Finding A -- the root cause of every `base/bif` refutation

**`grep` in this shell is not GNU grep.** It is a bash *function* that
execs `ugrep` with `--ignore-files -I`:

```
$ type grep
grep is a function
grep () { ... exec -a ugrep "$_cc_bin" -G --ignore-files --hidden -I \
          --exclude-dir=.git ... ; }
$ grep --version | head -1
ugrep 7.5.0 x86_64-pc-linux-gnu +sse2; -P:pcre2jit; ...
```

`-I` means **skip binary files**. Seven `base/bif` test groups contain
non-UTF-8 bytes and are therefore invisible to every `grep` run in this
shell:

```
$ for f in C2X COPIES D2C DATATYPE DELSTR DELWORD INSERT; do
    file -b ootest/ooRexx/base/bif/$f.testGroup; done
a rexx script, ISO-8859 text executable, with very long lines (341)
a rexx script, ASCII text executable, with very long lines (555)
a rexx script, ISO-8859 text executable
a rexx script, Non-ISO extended-ASCII text executable
a rexx script, ISO-8859 text executable, with very long lines (437)
a rexx script executable (binary data)
a rexx script executable (binary data)
```

Every D12 / Task 15 figure in the plan reproduces **exactly** when the same
query is restricted to the 71 files ugrep reads, and **not** when it is run
over all 78. That is the signature: the plan's numbers are not arithmetic
errors, they are a silently truncated population.

What the seven hold:

| file | `assertSame` | `^::method` | lines |
|---|---|---|---|
| C2X.testGroup | 34 | 34 | 180 |
| COPIES.testGroup | 479 | 622 | 2,481 |
| D2C.testGroup | 50 | 43 | 242 |
| DATATYPE.testGroup | 9 | 18 | 430 |
| DELSTR.testGroup | 87 | 71 | 353 |
| DELWORD.testGroup | 69 | 45 | 265 |
| INSERT.testGroup | 124 | 144 | 611 |
| **total** | **852** | **977** | **4,562** |

These are real, in-scope test groups. `COPIES`, `DELSTR`, `INSERT` and
`DELWORD` are Tasks 3 and 4's own builtins; `C2X`, `D2C` are Task 5's;
`DATATYPE` is Task 11's. Task 15's L1 harness will read all seven.

**Method note for anyone re-measuring:** use `/bin/grep -a` (GNU, `-a` for
"treat binary as text"), or count in Python over `latin-1`. Both are used
below and agree. The `wc -l` line count is unaffected because `wc` does not
classify.

Reference implementation used for the authoritative recount
(`/tmp/.../probe-measure/pycount.py`, decodes `latin-1`, no grep involved):

```
--- ootest/ooRexx/base/bif
  files=78  newlines=31162
  ::method  col0-cs=5397  col0-ci=5447  ws-ci=5462  anywhere-ci=5462
  assertSame             6293  files=73
  expectSyntax           1230  files=62
  assertTrue              232  files=18
  assertEquals            116  files=19
  assertFalse              76  files=9
  assertSameList            5  files=1
--- ootest/ooRexx/base/keyword
  files=42  newlines=24524
  ::method  col0-cs=2105  col0-ci=2105  ws-ci=2139  anywhere-ci=2189
  assertSame             1931  files=28
  AssertSame              510  files=28
  assertSameList          120  files=8
```

---

## Claim 1 -- the `base/bif` measurements (D12, Task 15 Step 1)

Directory `/home/moritz/dev/repos/ooRexx-rust-rewrite/ootest/ooRexx/base/bif`
(the C++ tree at `/home/moritz/dev/repos/ooRexx` has no `ootest/`; the only
`base/` in either tree is the one under the Rust-rewrite repo).

### 1a. "78 files" -- **VERIFIED**

```
$ ls -1 ootest/ooRexx/base/bif | wc -l
78
```
76 `.testGroup` + `ARG_TEST.rex` + `lineout`.

### 1b. "31,162 lines" -- **VERIFIED**

```
$ wc -l ootest/ooRexx/base/bif/* | tail -1
  31162 total
```

### 1c. "5,441 `assertSame`" -- **REFUTED. Correct value: 6,293.**

```
$ /bin/grep -aohP 'assertSame(?![A-Za-z0-9_])' ootest/ooRexx/base/bif/* | wc -l
6293
```
(The plan's 5,441 is exactly this count minus the 852 in the seven skipped
files. Token-matched, not prefix-matched: `assertSameList`'s 5 are excluded
by the negative lookahead. All 6,293 occurrences are preceded by `~` and
followed by `(` -- checked, zero comment or definition hits.)

### 1d. "in 66 files" -- **REFUTED. Correct value: 73 files.**

```
$ /bin/grep -laP 'assertSame(?![A-Za-z0-9_])' ootest/ooRexx/base/bif/* | wc -l
73
```

### 1e. "zero capital-`A` `AssertSame`" -- **VERIFIED**

```
$ /bin/grep -aohiP 'assertsame[A-Za-z0-9_]*' ootest/ooRexx/base/bif/* | sort | uniq -c
   6293 assertSame
      5 assertSameList
```
Every occurrence in all 78 files is lowercase-`a`. The claim holds on the
*full* population, not only the truncated one -- the one D12 sub-claim that
the instrument defect did not damage.

### 1f. "1,021 `expectSyntax`, 55 files" -- **REFUTED. Correct: 1,230 in 62 files.**

### 1g. "186 `assertTrue`, 17 files" -- **REFUTED. Correct: 232 in 18 files.**

### 1h. "106 `assertEquals`, 18 files" -- **REFUTED. Correct: 116 in 19 files.**

### 1i. "51 `assertFalse`, 8 files" -- **REFUTED. Correct: 76 in 9 files.**

All four from the same command shape:

```
$ for t in expectSyntax assertTrue assertEquals assertFalse; do
    echo -n "$t: "; /bin/grep -aohiP "${t}(?![A-Za-z0-9_])" ootest/ooRexx/base/bif/* | wc -l; done
expectSyntax: 1230
assertTrue: 232
assertEquals: 116
assertFalse: 76
```

### 1j. "`assertSameList` appears 5 times" -- **VERIFIED** (5, all in one file)

### 1k. "4,420 `::method` bodies" -- **REFUTED. Correct value: 5,447.**

This claim has *two* independent defects.

1. The seven skipped files hold 977 more (`4,420 + 977 = 5,397`).
2. The count is **case-sensitive**, and `DATE.testGroup` and `TIME.testGroup`
   spell 50 of their directives `::METHOD`:

```
$ /bin/grep -ac '::METHOD' ootest/ooRexx/base/bif/*.testGroup | grep -v ':0'
DATE.testGroup:30
TIME.testGroup:20
$ /bin/grep -an '::METHOD' ootest/ooRexx/base/bif/DATE.testGroup | head -2
290:::METHOD 'test_standard'
317:::METHOD 'test_standard_dash'
```

This is exactly the trap D12 asserts is absent: *"The case trap that cost
`base/keyword` 510 rows does not exist here, and this is measured rather than
assumed."* It does not exist for `assertSame` (1e is true). It **does** exist
for `::method`, and the plan's own `::method` figure is its first casualty.

Full spread of conventions over all 78 files (Python, `latin-1`):

| convention | count |
|---|---|
| `^::method`, case-sensitive (**the plan's, over 71 files**) | **4,420** |
| `^::method`, case-sensitive, all 78 files | 5,397 |
| `^::method`, **case-insensitive**, all 78 files | **5,447 <- correct** |
| any indent, case-insensitive, all 78 files | 5,462 |

The 15 in the last row are all in `LINES.testGroup`, inside four
`/* disable test, as it fails ... */` blocks (lines 63-85, 87-103, ...-182,
...-192) -- correctly excluded. 5,447 is the right figure.

**Consequence the plan draws from the wrong number:** D12's extrapolation
*"At `base/keyword`'s yield, `base/bif` would emit roughly 1,900 bodies"* is
`4,420 x 43%`. With 5,447 it is **~2,320**. Task 15's budget is ~22% light.

### 1l. "`assertTrue`/`assertEquals`/`assertFalse` total 343 calls in 25 files" -- **REFUTED. Correct: 424 calls in 34 files.**

```
assertTrue+assertEquals+assertFalse total = 424 in 34 files (union)
same over the 71 ugrep-visible files    = 343 in 33 files
```
The **343** reproduces on the truncated population, so it is the same defect.
The **25** reproduces under *no* convention I could find -- not the union
(33 truncated / 34 true), not the sum of per-token file counts (43). Treat it
as an independent error.

### 1m. `base/keyword`: "2,105 `::method` in 39 files" -- **VERIFIED**

```
$ /bin/grep -c '^::method' ootest/ooRexx/base/keyword/*.testGroup | awk -F: '{s+=$2} END{print s}'
2105
```
No keyword file is binary, so ugrep and GNU grep agree, and the count is
case-invariant here (zero `::METHOD` in that group). 39 `.testGroup` files
(the directory holds 42 entries; the other three are `.cls`, `.other`,
`.search_order`).

*Caveat, not a refutation:* 34 further `::method` lines are indented. 22 of
them sit inside `::resource`/`::RESOURCE ... ::end` blocks in
`TRACE.testGroup` and `TRACE_TraceObject.testGroup` and are embedded source,
correctly excluded; the remaining **12, in `LABEL.testGroup:128-150`, are
real two-space-indented directives** and are silently dropped by a column-0
anchor. The like-for-like comparison with `base/bif` is unaffected only
because both sides use the same anchor.

### 1n. `base/keyword`: "1,931 lowercase + 510 capital-`A` = 2,441" -- **VERIFIED**

```
$ /bin/grep -ohiP 'assertsame[A-Za-z0-9_]*' ootest/ooRexx/base/keyword/* | sort | uniq -c
   1931 assertSame
    510 AssertSame
    120 assertSameList
```
1,931 + 510 = 2,441 exactly. (`assertSameList`'s 120 in this group are a
separate method and are not part of the sum -- consistent with the plan.)

### 1o. "The shape is `assertSame`-dominated exactly as `base/keyword` was" -- **VERIFIED**

Survives correction: 6,293 `assertSame` against 1,230 / 232 / 116 / 76. D12's
*conclusion* (reuse `keyword.rs`, write no third extractor) is unaffected by
the refuted magnitudes.

---

## Claim 2 -- QUALIFY (D4) -- **VERIFIED**

```
$ /bin/grep -ain 'qualify' ootest/ooRexx/base/bif/* | sed 's|.*/bif/||' | cut -d: -f1 | sort | uniq -c
     49 QUALIFY.testGroup
$ /bin/grep -ain 'qualify *(' ootest/ooRexx/base/bif/* | /bin/grep -v 'QUALIFY.testGroup'
(no output, exit 1)
```
All 49 mentions of the token in any case (`qualify` 37, `Qualify` 21,
`QUALIFY` 8 -- 66 occurrences across 49 lines) are inside
`QUALIFY.testGroup`. Zero calls from any other group. Checked with GNU grep
`-a`, so the seven binary-ish files are included.

Outside `base/bif` (not claimed, recorded for completeness):
`ootest/ooRexx/API/classic/CLASSIC.testGroup:980` calls `qualify(...)`, and
`ootest/ooRexx/base/class/Stream.testGroup:2039` sends `~qualify("")`.
Neither is in `base/bif`, so D4's condition stands.

---

## Claim 3 -- RANDOM (D11) -- **VERIFIED**

`RANDOM.testGroup` read in full (165 lines).

The file contains exactly **three** `assertSame` calls, all three degenerate:

```
111:  r = random(1,1)
112:  self~assertSame(1, r)
118:  r = random(0)
119:  self~assertSame(0, r)
122:  r = random(,0)
123:  self~assertSame(0, r)
```

Nothing else in the file pins a generator output. The rest is:

* `test_RANDOM01` (:54-100): `reps=100`, six `(min,max)` pairs, each rep
  checked `x<mi | x>ma -> self~fail`, then `self~assertTrue(.true)`; then a
  seeded sequence generated twice **inside one process** (`x.1=random(mi,ma,se)`
  at :78 and `y.1=random(mi,ma,se)` at :86) and compared element-by-element
  (:94-97).
* `testRandom02` (:102-127): 100 unseeded range checks, the three degenerate
  `assertSame`s above, and further `assertTrue` range checks.
* `testRandom11` (:161-162): `r = random(0, 999999999)` with no assertion.

D11's three enumerated bullets match the file exactly, including the specific
degenerate forms named (`random(1,1)`, `random(0)`, `random(,0)`).

**One omission worth carrying into Task 6's brief** (an incompleteness, not a
refutation): the plan's summary of this file does not mention that
`testRandom03`-`testRandom10` (:129-159) are eight `expectSyntax` cases
pinning RANDOM's *argument validation*: **40.12** six times
(`random("abc")`, `random(1,"abc")`, `random(1,2,"abc")`, `random(1.5)`,
`random(1,1.5)`, `random(1,2,1.5)`), **40.13** once (`random(1,2,-1)`), and
**40.33** once (`random(2,1)`). Those are requirements on Task 6 that D11's
"asserts only properties" phrasing does not surface.

---

## Claim 4 -- the 40.x error family (Task 2 Step 1) -- **VERIFIED**

Probes run from `/tmp/.../scratchpad/p40x`, a directory created empty for
this purpose, absolute paths throughout, stdout/stderr/status read
separately.

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx /tmp/.../p40x/a.rex ) \
  > /tmp/.../p40x/a.out 2> /tmp/.../p40x/a.err
```

**`say substr('abc')` -- rc 216, stdout empty:**
```
     1 *-* say substr('abc')
Error 40 running /tmp/.../p40x/a.rex line 1:  Incorrect call to routine.
Error 40.3:  Not enough arguments in invocation of SUBSTR; minimum expected is 2.
```

**`say substr('abc','x')` -- rc 216, stdout empty:**
```
     1 *-* say substr('abc','x')
Error 40 running /tmp/.../p40x/b1.rex line 1:  Incorrect call to routine.
Error 40.12:  SUBSTR argument 2 must be a whole number; found "x".
```

**`say substr('abc',2,3,'pq')` -- rc 216, stdout empty:**
```
     1 *-* say substr('abc',2,3,'pq')
Error 40 running /tmp/.../p40x/c.rex line 1:  Incorrect call to routine.
Error 40.23:  SUBSTR argument 4 must be a single character; found "pq".
```

Sub-codes 40.3 / 40.12 / 40.23, secondary texts, the `*-*` echo, the
`Error 40 running <path> line 1:  Incorrect call to routine.` shape (two
spaces after the colon), and rc **216** in every case: all exactly as the
plan's Step 1 table and stderr block state.

---

## Claim 5 -- the `>.>` trace shape (Task 7 Step 1) -- **VERIFIED**

Program `ti.rex`: `trace i` / `parse value 'a b c' with p . q`. stdout empty,
rc 0, stderr (`cat -A`, `$` = EOL):

```
     2 *-* parse value 'a b c' with p . q$
       >L>   "a b c"$
       >K>   "VALUE" => "a b c"$
       >>>   "a b c"$
       >=>   P <= "a"$
       >.>   "b"$
       >=>   Q <= "c"$
```

Six lines, in exactly the plan's order, byte-identical including the
three-space gap after each prefix and the `>K>   "VALUE" => "a b c"`
continuation form.

Same program under `trace r`, stdout empty, rc 0:

```
     2 *-* parse value 'a b c' with p . q$
       >K>   "VALUE" => "a b c"$
       >>>   "a b c"$
       >>>   "a"$
       >>>   "c"$
```

**No `>.>`.** Confirms the plan's core point: the scoping document's `trace r`
probe could not have seen the prefix.

*Two extra facts the probe shows that the plan does not state, and Task 7
will need them:* under `trace r` the `>L>` line is also absent, and assigned
targets render as bare `>>>   "a"` / `>>>   "c"` rather than `>=>`. So the
`trace r` shape is not "the `trace i` shape minus `>.>`".

---

## Claim 6 -- `+++` (D-P) -- **VERIFIED, with one caveat that does not change the ruling**

### 6a. Exactly two `TRACE_PREFIX_ERROR` emission sites -- **VERIFIED**

```
$ /bin/grep -arn 'TRACE_PREFIX_ERROR' interpreter/ common/ rexxapi/ api/
interpreter/execution/RexxActivation.hpp:93:        TRACE_PREFIX_ERROR    ,         //  1
interpreter/execution/RexxActivation.cpp:3570:  "+++",                               // TRACE_PREFIX_ERROR
interpreter/execution/RexxActivation.cpp:4024:    buffer->put(PREFIX_OFFSET, trace_prefix_table[TRACE_PREFIX_ERROR], PREFIX_LENGTH);
interpreter/execution/RexxActivation.cpp:4468:            traceValue(rc_trace, TRACE_PREFIX_ERROR);
```

Four hits: the enum declaration, the table's own string, and the **two**
emission sites the plan names. I looked specifically for a third by three
independent routes, all negative:

* **Variable-prefix routes.** `trace_prefix_table[...]` is indexed at
  `:3714`, `:3743`, `:3794`, `:3880`, `:3974` with a *variable*. Every one of
  those is reachable only from `traceValue`/`traceClause`/`traceEntry`
  callers, and `TRACE_PREFIX_ERROR` appears as an argument at exactly one
  call site (`:4468`). `traceIntermediate`, `traceArgument`, `traceResult`,
  `traceResultValue` (`RexxActivation.hpp:339-371`) pass fixed non-ERROR
  prefixes.
* **Numeric back door.** `/bin/grep -arn '(TracePrefix)' interpreter/` finds
  one hit, a *declaration* (`RexxActivation.hpp:238`, `void
  traceEntryOrExit(TracePrefix);`). No int-to-enum cast anywhere.
* **Case-insensitive and accessor searches** over the whole `interpreter/`
  tree for `trace_prefix`, `PREFIX_OFFSET`, `TracePrefix`: nothing else.

### 6b. `:4024`'s only caller is guarded by `inDebug()` -- **VERIFIED, and the plan's `:4305` is exact**

```
$ /bin/grep -arn 'traceSourceString' interpreter/ --include=*.cpp --include=*.hpp
interpreter/execution/RexxActivation.hpp:113:        TRACE_OUTPUT_SOURCE = 30,   // for: void RexxActivation::traceSourceString()
interpreter/execution/RexxActivation.hpp:235:   void              traceSourceString();
interpreter/execution/RexxActivation.cpp:4007:void RexxActivation::traceSourceString()
interpreter/execution/RexxActivation.cpp:4307:            traceSourceString();
```
One declaration, one definition, **one call site**. And at `:4302-4307`:
```cpp
    if (line != OREF_NULL)
    {
        // if we've just dropped into debug mode, we need to put out the extra context line.
        if (inDebug() && !settings.wasSourceTraced())          // <- :4305
        {
            traceSourceString();                                //  <- :4307
```
The guard is on `:4305` exactly as the plan cites.

### 6c. `address sh` + `'exit 3'` under `trace r` prints `+++   "RC(3)"` -- **VERIFIED**

Program: `trace r` / `address sh` / `'exit 3'`. rc 0, stdout empty, stderr
(`cat -A`):
```
     2 *-* address sh$
     3 *-* 'exit 3'$
       >>>   "exit 3"$
       +++   "RC(3)"$
```
Three spaces between `+++` and `"RC(3)"`, exactly as written.

### 6d. CAVEAT -- `+++` has three more producers, none of them `TRACE_PREFIX_ERROR`

The plan's sentence is narrowly true, but a reader of the paragraph Task 1
Step 4 pastes into `phase-4-exclusions.txt` will reasonably infer "there are
only two ways `+++` can appear in oracle output". There are five:

```
interpreter/messages/RexxErrorMessages.h:725:  MESSAGE(Message_Translations_debug_error,  "+++ Interactive trace.  Error")
interpreter/messages/RexxErrorMessages.h:726:  MESSAGE(Message_Translations_debug_prompt, "+++ Interactive trace. \"Trace Off\" to end debug, ENTER to continue. +++")
interpreter/execution/RexxActivation.cpp:4237:  processTraceInfo(activity, Interpreter::getMessageText(Message_Translations_debug_prompt), TRACE_OUTPUT, ...);
interpreter/concurrency/Activity.cpp:1496:      RexxString *text = Interpreter::getMessageText(Message_Translations_debug_error);
interpreter/concurrency/Activity.cpp:1507:      text = Interpreter::getMessageText(Message_Translations_debug_error);
```

This matters for the D-P text specifically because **D-P's own quoted
two-line banner attributes both lines to the `:4024` site**, and only the
first comes from there:

```
+++ "LINUX COMMAND <absolute path>"                                        <- :4024, traceSourceString
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++    <- :4237, Message_Translations_debug_prompt
```

**The D-P ruling is unaffected and in fact strengthened** -- all three extra
producers are interactive debug, which D-P assigns to Phase 7. But the
paragraph should say "`TRACE_PREFIX_ERROR` has two emission sites; the
remaining `+++` output comes from the interactive-debug message table
(`RexxErrorMessages.h:725-726`), also Phase 7's", or a later reader will
find `Activity.cpp:1496` and reopen the question.

### 6e. Downstream arithmetic: "16 of 19, not 17" -- **VERIFIED**

`PREFIX_COVERAGE` (`tests/trace_oracle.rs:527-547`) has 19 rows;
`WITNESSED_PREFIX_COUNT = 13` (`:551`), `OUT_OF_SCOPE_PREFIX_COUNT = 6`
(`:555`). 4c adds `>.>` (`:531`), `>I>` (`:542`), `<I<` (`:546`) -> 16.
Remaining: `+++` (`:529`), `>M>` (`:539`), `>N>` (`:543`) -> 3. 16 + 3 = 19.

`OWNER_PHASES` at `:560` is `&["4c", "Phase 5", "Phase 7"]`, so Task 1
Step 4's "`OWNER_PHASES` already admits `"Phase 7"`" is **VERIFIED**.

---

## Claim 7 -- the code citations

| citation | what the plan says it says | verdict |
|---|---|---|
| `run.rs:3218` | the loud fallback in `resolve_and_run_call` | **VERIFIED** |
| `eval.rs:529` | already delegates to `resolve_and_run_call` | **VERIFIED** |
| `plan.rs:79` | `directive: Option<usize>` | **VERIFIED** |
| `plan.rs:631` | "its **one** construction site", `None` | **REFUTED** |
| `lib.rs:761` | the `instruction_owner` arm carrying `Parse` | **VERIFIED** (caveat) |
| `lib.rs:1340` | the private two-variant `Argument` enum | **VERIFIED** |
| `owners.rs:165` | `Parse => Owner::Phase("4c")` | **VERIFIED** |
| `owners.rs:352` | the split-table row for `Parse` | **VERIFIED** (caveat) |
| `trace_oracle.rs:529` | `("+++", Coverage::Owned("4c"))` | **VERIFIED** |
| `trace_oracle.rs:542` | `(">I>", Coverage::Owned("4c"))` | **VERIFIED** |
| `trace_oracle.rs:546` | `("<I<", Coverage::Owned("4c"))` | **VERIFIED** |

### 7a. `run.rs:3218` -- VERIFIED

```
3217	        };
3218	        let Some(target) = target else {
3219	            return Err(Loud::unresolved_call(name).into());
3220	        };
```
`:3218` is the first line of the three the plan quotes, and the quote is
byte-exact. The enclosing function is `pub(crate) fn resolve_and_run_call`,
opened at `:3192`; the next `fn` is `:3481`. So the citation, the snippet and
the function attribution are all right.

### 7b. `eval.rs:529` -- VERIFIED

```
519	    fn eval_call(
...
529	        match self.resolve_and_run_call(code, name, search_labels, args)? {
```

### 7c. `plan.rs:79` -- VERIFIED

```
79	    pub(crate) directive: Option<usize>,
```

### 7d. `plan.rs:631` -- **REFUTED**

Two separate errors in one sentence (repeated verbatim in D-R and again in
Task 13 Step 1).

**Error 1: `plan.rs:629-631` is inside `#[cfg(test)] mod tests`.**

```
$ /bin/grep -n '#\[cfg(test)\]' rust/crates/rexx-exec/src/plan.rs
615:#[cfg(test)]
616:mod tests {
```
`:631` sits in a test helper (`fn activate(interp: &mut Interp, program:
Program)`, documented "so these tests can drive `slot_of`/`Plan` through a
live activation"). Task 13's Step 1 sends an implementer to a test fixture.

**Error 2: it is not "its one construction site". There are seven.**

```
$ /bin/grep -rn 'BodyKey *{' rust/crates/rexx-exec/src
src/queue.rs:205        (in mod tests, #[cfg(test)] at :136)
src/eval.rs:1019        (in mod tests, #[cfg(test)] at :996)
src/trace.rs:827        (in mod tests, #[cfg(test)] at :644)
src/lib.rs:1422         (NO preceding #[cfg(test)] -- production)
src/stem.rs:539         (in mod tests, #[cfg(test)] at :519)
src/run.rs:6753         (in mod tests, #[cfg(test)] at :6736)
src/plan.rs:629         (in mod tests, #[cfg(test)] at :615)
```

**Correct value: the one *production* construction site is `lib.rs:1422`**,
inside `Interp`, immediately after a comment about `plan_for`'s borrow:

```rust
        let plan = self.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
```
The other six are copies of the same test fixture. **The sub-claim that
`directive` is still `None` everywhere is VERIFIED** -- all seven sites set
`None`. Only the "one construction site (`plan.rs:631`)" locator is wrong,
and Task 13 Step 1 should read `lib.rs:1422` (production) with a note that
six test fixtures carry the same literal and will need the same treatment.

### 7e. `lib.rs:761` -- VERIFIED with a caveat

```
758	        InstructionKind::Parse(_)
759	        | InstructionKind::Arg(_)
760	        | InstructionKind::Pull(_)
761	        | InstructionKind::Address(_) => Some("4c"),
```
`:761` is the *last* line of the arm. The `Parse` pattern that Task 7 Step 4
removes is on `:758`. "`src/lib.rs:761`'s arm loses `Parse`" is true of the
arm; the line number points at `Address`.

### 7f. `lib.rs:1340` -- VERIFIED

```
1339	#[derive(Clone)]
1340	enum Argument {
1341	    Value(ObjRef),
1342	    Reference {
...
1348	    },
1349	}
```
`:1340` is the `enum Argument {` line exactly. Private (no `pub`), two
variants. `Argument::value()` exists at `:1354-1358` and its doc reads
"`USE ARG` without `>` and `ARG()` both want only this" -- so Task 2's
"its own doc names `ARG()` as a caller" is **VERIFIED**.

### 7g. `owners.rs:165` and `:352` -- VERIFIED, with a name caveat on `:352`

```
164	    // ---- 4c's ----
165	    InstructionKind::Parse(_) => ("Parse", Owner::Phase("4c")),
...
352	    ("InstructionKind", "Parse", "4c"),
```
Both lines say what the plan says. The plan calls `:352` "its `SPLIT_TABLE`
row"; the const holding it (`:347`) is named **`EXPECTED_OUT_OF_SCOPE`**.
`SPLIT_TABLE` does not exist as an identifier -- the nearest is
`SPLIT_TABLE_PHASES` at `:378`. Cosmetic, but a Task 7 implementer grepping
for `SPLIT_TABLE` will land on the wrong const.

### 7h. `trace_oracle.rs:529` / `:542` / `:546` -- VERIFIED

```
527	const PREFIX_COVERAGE: &[(&str, Coverage)] = &[
528	    ("*-*", Coverage::Witnessed),
529	    ("+++", Coverage::Owned("4c")),
...
542	    (">I>", Coverage::Owned("4c")),
...
546	    ("<I<", Coverage::Owned("4c")),
547	];
```

---

## Claim 8 -- the builtin arithmetic -- **VERIFIED in every part**

### 8a. `rexx_inventory::builtins::NAMES` has 81 entries -- VERIFIED

`NAMES` is generated by `crates/rexx-inventory/build.rs` from
`interpreter/expression/BuiltinFunctions.cpp`. Checked both ends:

```
$ /bin/grep -oP '"[A-Z0-9]+"' <generated builtins.rs> | wc -l
81
$ awk '/pbuiltin LanguageParser::builtinTable\[\] =/{f=1} f{print} f&&/^};/{exit}' \
    interpreter/expression/BuiltinFunctions.cpp | /bin/grep -oP 'builtin_\w+' | wc -l
81
```
Name-by-name identical modulo the `FUNCTION_` prefix. The generated file is
not stale.

### 8b. `EXCLUDED_BUILTINS` has 18 rows, 3 partial -- VERIFIED

`tests/coverage.rs:701-723`: 15 whole (`CHARIN CHAROUT CHARS LINEIN LINEOUT
LINES STREAM QUALIFY USERID SETLOCAL ENDLOCAL RXQUEUE RXFUNCADD RXFUNCDROP
RXFUNCQUERY`) + 3 under `// Partial: in scope in one form, excluded in
another.` (`VALUE ADDRESS QUEUED`). The file asserts both:
`assert_eq!(EXCLUDED_BUILTINS.len(), 18, ...)` at `:747`.

### 8c. `in_scope` is 66 -- VERIFIED

`coverage.rs:757`: `let in_scope = names.len() - (EXCLUDED_BUILTINS.len() - 3);`
-> `81 - 15 = 66`, asserted at `:758`.

### 8d. The seven family lists partition the in-scope set -- **VERIFIED by symmetric difference**

Not an addition check. The seven lists were transcribed from the plan's File
Structure table (identical to the per-task lists in Tasks 3-6, 10-12),
sorted, and compared to `NAMES` minus the 15 whole exclusions:

```
=== per-family declared counts ===
1: 23   2: 7   3: 12   4: 7   5: 4   6: 2   7: 11
total family entries: 66
unique family entries: 66
=== duplicates across families ===
(none)
in-scope size: 66
=== in-scope names MISSING from the family lists ===
(none)
=== family names NOT in the in-scope set ===
(none)
=== symmetric difference size ===
0
```

The seven lists are a **true partition** of the 66 in-scope builtins: no name
missing, none duplicated across families, none present that is out of scope.
The stated per-family counts (23/7/12/7/4/2/11) each match their own list's
length, and `VALUE`, `ADDRESS`, `QUEUED` -- the three partials -- correctly
appear (in `datatype.rs` and `state.rs`).

---

## Additional claims checked (beyond the eight, all cheap and load-bearing)

### Governing-document line citations

| citation | verdict |
|---|---|
| `phase-4-exclusions.txt:84` "Four of the six -- +++ and >.> (4c), >M> and >N> (Phase 5)" | **VERIFIED** (phrase spans `:83-:84`) |
| `phase-4-exclusions.txt:88` says `::routine` is 4c's | **VERIFIED as to substance**, quote misplaced -- see below |
| `phase-4-exclusions.txt:540` `QualifiedCall` row citing "every directive" | **VERIFIED** |
| `phase-4-exclusions.txt:989-1011` `TRACE ?` row ending "Owner unassigned." | **VERIFIED** exactly -- row opens `:989`, "Owner unassigned." paragraph is `:1009-1011` |
| `2026-07-30-phase-4a-executor-design.md:71` "every directive ... are Phase 5's" | **VERIFIED**, quoted fragment byte-exact |
| `RexxActivation.cpp:3655` = `traceEntry = tracingLabels() && isMethodOrRoutine();` | **VERIFIED** |

**The `:88` quote.** D-R says *"`phase-4-exclusions.txt:88` says `::routine`
is 4c's (**"which 4c will have to meet"**)"*. Line 88 reads
`  >I> / <I<   4c, deferred alongside ::routine dispatch itself.` -- which
does support the proposition. But the parenthetical evidence is at **`:124`**
(`LABEL'S, which 4c will have to meet:`) and there it introduces the
*two-condition trace gate*, not `::routine` ownership. The claim survives;
the quotation attached to it is from a different line and a different
subject. D-R's better citation is `:99-100`, which says in so many words that
4b declines `::routine` "because getting builtin-colliding names right needs
4c's table" -- the exact sentence D-R's reason 1 relies on.

### `keyword-exempt.txt`'s 790 -- VERIFIED

```
$ wc -l < rust/corpus/keyword-exempt.txt        -> 850
$ /bin/grep -vc '^#\|^$' rust/corpus/keyword-exempt.txt  -> 796
$ /bin/grep -v '^#\|^$' ... | awk -F'\t' '{print $NF}' | sort | uniq -c
    790 4c
      6 defect:compound-do-control-variable
```
790 `4c` rows and Task 14's six compound-`DO` bodies, both exact. (A bare
`grep -c '4c'` returns **792** -- two of the hits are header comment lines,
`:24` and `:35`. The plan's number is the population, not the hit count.)

### D-R's three measured facts about a `::routine` activation -- all VERIFIED

Probes from `/tmp/.../scratchpad/pdr`, fresh directory, stdin at `/dev/null`.

* **Own variable pool.** `nn = 5` / `call zorkolo` / `::routine zorkolo` /
  `say nn` -> stdout `NN`, rc 0.
* **Builtins shadow it.** `call max 1, 9` / `say 'RESULT=' result` /
  `::routine max` / `say 'ROUTINE RAN'` / `return 'from-routine'` ->
  stdout `RESULT= 9`, rc 0. `ROUTINE RAN` never printed.
* **Trace does not cross into it.** `trace r` / `call zorkolo` /
  `::routine zorkolo` / `say 'inside'` -> stdout `inside`, stderr exactly
  `     2 *-* call zorkolo`, rc 0. None of the routine's clauses echoed.
* **D-R reason 3.** `call zorkolo` / `say 'after'` / `::routine zorkolo` /
  `nop` -> stdout `after`, rc 0. Runs on the oracle.

### Task 13 Step 3's 43.1 -- VERIFIED

`call nosuchroutinehere` from a clean directory: rc **213**, stdout empty,
```
     1 *-* call nosuchroutinehere
Error 43 running /tmp/.../pdr/r5.rex line 1:  Routine not found.
Error 43.1:  Could not find routine "NOSUCHROUTINEHERE".
```
The plan writes `Error 43.1 rc 213, "Routine not found"` -- accurate, though
"Routine not found" is the Error 43 primary; 43.1's own secondary is
`Could not find routine "NAME".`

### Task 13 Step 5's `>I>`/`<I<` gate and content -- VERIFIED, with one byte-level correction

* `trace l` in the **caller** targeting a `::routine` -> stdout empty, stderr
  **empty**, rc 0. Nothing at all, as claimed.
* A non-dynamic `trace l` as the routine's own first instruction -> both fire:

```
       >I> Routine "ZORKOLO" in package "/tmp/.../pil/c2.rex".
       <I< Routine "ZORKOLO" in package "/tmp/.../pil/c2.rex".
```

**Correction to the plan's "Verbatim" block:** both lines carry a **7-space
leading indent** that the plan's quoted block omits. A `tests/trace_oracle/`
expectation built from the plan's text as printed would be off by seven
bytes on every line. (The plan correctly says the witness must live in the
live corpus because of the absolute path; the indent is separate.)

### Task 9 / Task 10 spot-checks -- VERIFIED

`say address()` -> `sh`; `say gc()` -> `0`; rc 0, stderr empty.

### Task 7's AST claims -- VERIFIED

`rexx_parse::ast::Parse` (`ast.rs:1044-1053`) carries `source`, `upper`,
`lower`, `caseless`, `template: Vec<Option<ParseTrigger>>` with `None`
documented as the comma fence. `ParseTrigger` (`:1071-1079`) carries `kind`,
`value: Option<Expr>`, `targets: Vec<Option<Expr>>` with `None` documented as
the `.` placeholder. `TriggerKind` (`:1083-1100`) has all eight variants:
`End`, `Plus`, `Minus`, `Absolute`, `MinusLength`, `PlusLength`, `String`,
`Mixed`.

---

## One consistency problem in Task 1 that is arithmetic, not measurement

Task 1 Step 2 expects "66 `loud`, 15 `excluded`, 0 `implemented`" and asserts
`implemented + loud == 66`. The three numbers add to 81 and the assertion is
consistent with them. But Step 1's classifier is ordered:

1. loud message -> `loud`
2. **name is in `coverage.rs`'s `EXCLUDED_BUILTINS`** -> `excluded`
3. otherwise -> `implemented`

`EXCLUDED_BUILTINS` has **18** rows, not 15 -- `VALUE`, `ADDRESS` and
`QUEUED` are in it *and* in scope. Because rule 1 fires first they will
classify `loud` (they are unimplemented today), which happens to produce the
expected 66/15/0. But the 15 whole exclusions will *also* hit rule 1 if
`rexx-exec` fails loudly on them, in which case the harness reports 81 `loud`
and 0 `excluded` and Step 2's expectation is unreachable. The classifier
needs to test `EXCLUDED_BUILTINS`-membership-minus-the-three **before** the
loud message, or Step 2 needs to say which of the two it expects. Flagging
because Step 3's falsification exercise will not surface it -- both
directions of the set assertion pass either way.

---

## Appendix: how each figure was obtained

* Probe directories, all created empty for this review, all outside the repo:
  `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/0b337b3a-acaa-4a86-b38a-f4a89668e346/scratchpad/{p40x,ptrace,pplus,pdr,pil,probe-measure}`.
* Every oracle invocation wrapped exactly as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx <abs path> )`,
  with `> <abs>.out 2> <abs>.err < /dev/null` and `$?` read unpiped
  immediately after. No `2>&1` anywhere.
* No file in either repository was modified. `NUMERIC DIGITS` was never set.
  The SF #2018 `select` shape was never run. `.Package~new` was never used.
* Text counts were taken with **`/bin/grep -a`** (GNU 3.12) and independently
  with Python over `latin-1`; the two agree on every figure quoted. Shell
  `grep` (ugrep 7.5.0 via the wrapper function) is quoted only where the
  point is that it *disagrees*.
