# Phase 9d Task 8.4 review: install order, directives, scanner

Range `4afa97948..9194fef58` (BASE = `f0164456d` plus a ledger line). Binaries built for this review:
BASE from a worktree at `4afa97948`, HEAD from the tree at `9194fef58`, both `--release`,
`CARGO_INCREMENTAL=0`, own target dirs under `t84-review-scratch/`. Every probe below was run
three ways (oracle, base, head) from a fresh directory per side, stdout, stderr and rc compared
separately, run directory normalised out of the text.

**Spec compliance: COMPLIANT.** **Quality: CHANGES REQUESTED** (0 Critical, 1 Important, 4 Minor).

## Checked by running

1. **Closed items.** All nine new corpus witnesses differ from the oracle at BASE and match on all
   three descriptors at HEAD:
   ```
   scan_error_waits_for_its_clause: base[out DIFF ...] head[out same err same rc same]
   translation_checks_in_source_order: base[out DIFF ...] head[same same same]
   translation_error_in_called_file: base[out DIFF err DIFF] head[same same same]
   install_error_in_called_file: base[out DIFF err DIFF] head[same same same]
   install_error_in_routine_new: base[err DIFF] head[same same same]
   constant_expression_level: base[out DIFF err DIFF rc DIFF] rc o/b/h=0/159/0 head[same same same]
   constant_expression_trace / _trace_labels: base[err DIFF] head[same same same]
   install_error_in_required_file: base[err DIFF] head[same same same]
   ```
   The items' own probes too: `queued-probes/libbind/libbind.rex` rc o/b/h=158/230/158,
   `queued-probes/scanlater/scanlater.rex` 221/243/221, `t8.3/probes/constant_frames.rex` base out
   DIFF, head same. The five CLOSED first lines are present with the right shas; inventory columns
   4 and 6 read `closed:<sha>` for the five files and both gate lines, column 5 unchanged (checked in
   the diff of `queued-inventory.tsv`); `t8.4/closed.tsv` holds both gate rows;
   `install-error-names-caller-file.md` stays open with a status line for the `hs9`/`qpull`
   paragraph only.

2. **Ordering hunt.** 107 multi-error main programs (`p1/`, `p6/`): scanner error after or before a
   duplicate, a duplicate before and after a bad `::OPTIONS`, `::RESOURCE` without end marker plus a
   duplicate resource, a resource body holding an invalid character, duplicates against
   `CLASS`-side and `=`-setter keys (`::method "A="` against `::attribute a`, both directions),
   literal and symbol names of different case, the unattached method table across `::CLASS`
   boundaries, `::OPTIONS DIGITS`/`FUZZ` in both orders and across directives, every `::ANNOTATE`
   target kind present and missing, `EXTERNAL` binds before and after duplicates, scanner errors,
   unclosed comments and strings, and a missing superclass. Every one matches the oracle at HEAD
   except three: `a53` (`REGISTERED`, now the loud Phase 10 refusal where BASE gave a wrong 35.1;
   correct by Ruled 8's shape), and `b21`/`b22` (`LIBRARY rxmath`, 98.903 here against 90.998/90.999,
   identical at BASE: the crate does not load `rxmath`, outside this task). INTERPRET strings
   (`i1.rex`, 16 shapes incl. directives in INTERPRET and a scan error after a refused directive),
   `.Routine~new`/`.Method~new`/`.Package~new` sources (`n1.rex`, 19 shapes) and `::REQUIRES`d files
   with a duplicate, a bind, a resource without marker (`q1`-`q4`) all match. `q5` (condition
   `~program` after a compile failure in a called file) differs and is the report's concern 3.
   The changed `tests/errors.rs` expectations are the oracle's: `rexxc` on `say )\n\n'unclosed\n`
   is 37.2 line 1 rc 219; on `say 1\n::resource data\nnever terminated` 99.943 line 2.

3. **Perf shape.** `callgrind.sh -B 0.5 -j 4 -p "startup sayloop interpret" base=<BASE> head=<HEAD>`,
   rc 0, libc subtracted: startup +0.0214%, sayloop +0.0114%, interpret +0.0019% (the report's
   figures exactly). The unchecked library parse skips only the duplicate tables and the
   `::ANNOTATE` target check (`Declared::checked` gates `claim`, the class/routine/resource inserts
   and `require_annotation_target`); 33.1, 99.905, 99.906 and every grammar error still run, and
   `tests::every_library_program_passes_the_declaration_checks` parses every `rexx_lib::PROGRAMS`
   entry checked (passes in `cargo test -p rexx-exec --lib`, 1178 passed). `parse_known_program`
   has one caller (`lib.rs:2598`). INTERPRET of a string with a directive reports the oracle's
   answer (`i1.rex`).

4. **`::CONSTANT` level.** Matching the oracle: traceback text of an error in the expression and in
   a class method it calls (`k2`, `k3`, `t_err`), `.context~executable` and its flags, `~source`,
   `~isGuarded`, `~package`, `~scope`, private `~run` (97.2) (`k1`), `self` and `.context` in the
   expression (`k5`), TRACE under `::OPTIONS TRACE` I, R, A, L, N, O, C, E, F and a main-program
   `trace i` (`t_*`). Differing: the levels' `traceline` (Important 1), invoking the method object
   (Minor 4), and the guard (queue candidate 2).

5. **parse-errors.tsv.** `derive.sh` re-derives the file byte-identically from the oracle (rc 0,
   `diff` empty). The 26 new rows through HEAD's `rexxc`: all 26 match code, line and message; 18
   of them differ at BASE (the other 8 are the adjacent successes the report names). The
   install-row count is 12, the classification test asserts it exactly, and its new form (accept,
   or reject having bound the oracle's line first) passes.

6. **Deviation candidate A is covered by Deviation 67.** `r1.rex` is 67's own shape and its race
   reproduced: oracle 4 of 5 runs `line 3` twice, 1 of 5 `line 8` then `line 3` (this crate's
   answer, 5 of 5). `r2.rex` (HALT, the `hs9` shape) is the same mechanism through the same
   condition object: oracle 5 of 5 `line 2` twice, crate `line 9` then `line 2`. An untrapped HALT is
   reported as an Error 4 report, so 67's title and rule ("A started message's error report names
   the line that raised it", RESULT re-raising) cover it; 67 does not need extending beyond, at
   most, naming the HALT probe as a second witness. `install-error-names-caller-file.md` can close
   under 67.

7. **Memory.** 100000 iterations under `memcap 2G`, peak RSS (`/usr/bin/time -f %M` on the
   interpreter itself), 10000 against 100000 iterations, base and head:
   ```
   m1 .Package~new with class/method/attribute/constants/routine/resource/annotate/options
        base 50192 -> 50240 KB   head 50220 -> 50296 KB
   m2 .Routine~new failing 99.902, trapped   base 39232 -> 38980   head 38412 -> 38996
   m3 .Routine~new failing on a bind (98.903) base 38936 -> 38156   head 38824 -> 39452
   m4 .Routine~new with ::constant (.context~executable)  base 52484 -> 52588  head 52152 -> 51788
   ```
   No growth.

Tests: `cargo test -j 4 --release -p rexx-parse` rc 0; `cargo test -j 4 --release -p rexx-exec
--lib` rc 0, 1178 passed; `cargo fmt --all --check` rc 0. The corpus gate and the suite were not
re-run here.

## Strengths

The parser move is the oracle's shape and it holds under every multi-error program tried, including
the INTERPRET, `~new` and `::REQUIRES` routes; the bind-before-error rule (`Rejected::binds`, not
recorded when the body check read past a scanner error) matches in each direction probed. The perf
fix brought the declaration state to near zero on every hot program.

## Important

**I1. The `::CONSTANT` level and the level below it report the wrong `traceline`.** The live
frames and a trapped condition's `~stackframes` show the main program's first clause (or nothing,
for a `.Routine~new` source) where the oracle shows the directive's clause and the installing
`::CLASS`. Lines, types, names and executables are right, and the error traceback text is right.
Cause: `eval_constant_expression` (`install.rs`, the two `Activation`s it pushes) sets `body = None`
and `enter_directive_clause` sets clause index 0, and `trace_line_text`
(`dispatch/context.rs:1105`-`:1117`) reads `body_of(program, None)`, the main body, at index 0.
The closed item's subject is this frame; no witness prints `traceline`.
```
k4.rex: say .c~k / ::class c / ::constant k (.c~f) / ::method f class
        do f over .context~stackframes; say f~type ... f~line ...; say '   ' f~traceline; end
oracle:  METHOD ::CONSTANT 4 ...            head:  METHOD ::CONSTANT 4 ...
                  4 *-* ::constant k (.c~f)                   4 *-* say .c~k
         PROGRAM k4.rex 3 ...                      PROGRAM k4.rex 3 ...
                  3 *-* ::class c                             3 *-* say .c~k
tl1.rex (.Routine~new source):  oracle "3 *-* ::constant k (.c~f)" / "2 *-* ::class c"
                                head   "3 *-* " / "2 *-* "
tb1.rex (condition('o')~stackframes inside the class method): the same two lines differ.
```

## Minor

**M1. `parse-errors.tsv`'s header now states something false.** Line 12 says of an `install` row
"This parser must ACCEPT it", and lines 39-40 "The decision to accept these rows is unchanged"; three
of the new install rows are rejected (bound first), which the renamed test
`a_rejection_that_is_not_a_translation_error_is_accepted_or_bound_first` knows. Delete or correct
the sentences (`rust/corpus/errors/parse-errors.tsv:12`, `:39`).

**M2. The `::CONSTANT` `~source` departure is not offered as a Deviation candidate.** The crate
answers each expression method's own clause where the oracle answers the last one's
(`t8.4/constant/probes/f8.rex`, re-run: oracle `[::constant g ...]` three times, head `e`, `f`'s two
lines, `g`). That is a chosen divergence from the oracle (Ruled 13), so it needs Moritz's ruling as a
Deviation, but the report lists it only under "New oracle defects". The comment at
`dispatch/executable.rs` (`directive_clause_lines`) states the oracle's answer and the code does the
other thing without saying why; fine once ruled.

**M3. Two report statements have no record.** "Oracle, 8 of 8 runs identical" for candidate A:
`naming/result-reraise-r1.txt` holds one run, and this review saw 1 of 5 oracle runs give the crate's
answer (it is 67's race, so the sentence overstates the oracle's answer). "A debug build of
`7fe50bedd` ran every new witness ... with no assertion firing": no record in `t8.4/`.

**M4. Invoking the `::CONSTANT` method object is a loud refusal with a false message.**
`.c~define('ZZ', .c~e); .c~new~zz` (also `self~run(e)` and `enhanced`): oracle 91.999 `Message "ZZ"
did not return a result.` rc 165; head `rexx-exec: a Method or Routine object with no executable
record` rc 120 (the object has a record, `ConstantExpression`, and no installed `MethodId`; the
message is `lib.rs:908`'s). Loud, so not a wrong answer; this is the report's concern 2, now
measured. Queue it.

## New oracle defects

None new beyond the report's (the `~source` one reproduces, M2).

## Queue candidates

1. **Report concern 3** (condition `~program`/`~package` after a compile failure names the
   caller): `q5.rex` reproduces it, `99.903 2 dup.cls` on the oracle, `99.903 2 q5.rex` here; same at
   BASE.
2. **A `::CONSTANT` expression takes no guard on its class.** `g2.rex`: the expression calls an
   unguarded class method that starts a guarded class method and waits on `~result`. Oracle rc 158,
   `Error 98.905: Deadlock detected on a guarded method.`; head (and BASE) rc 0 printing `g ran`.
   Predates the task; the report's concern 2 names it unmeasured.
3. `LIBRARY rxmath` is 98.903 here, 90.998/90.999 on the oracle (`b21`, `b22`), same at BASE.

Probe sources and outputs: `.superpowers/sdd/2026-10-18-phase-9d/t84-review-scratch/` (deleted at
the end of the review; the deciding lines are quoted above).
