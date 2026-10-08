# Task 5 review: parse errors as SYNTAX, with the traceback line

Base `f1202acc1`, head `a46465a9f`. Reviewer t5-review, 2026-10-08. Read-only on the tree. Probes ran
from fresh empty directories under the session scratchpad's `t5rev/run/`, using the standard oracle
wrapper and two release builds made from `git archive` copies, each in its own target directory with a
`Compiling rexx-exec` line: HEAD `a46465a9f` and base `f1202acc1`. Comparisons drop the sub-message
line, because under R3 the inserts stay `&1`. Target directories deleted after the run.

### Spec Compliance

- ❌ Issues found: two Important (I1, I2), both about records or expectations rather than the
  mechanism itself. The mechanism matches the brief: `ParseError` carries the end (`token.rs:28-71`),
  `Raised::parse_failure` (`error.rs:1864`) with `Interp::raise_parse_failure` (`run.rs:3205`)
  is called from the main program (`lib.rs:3185`, called at `:3224`), `::REQUIRES` and `Package~new(file)`
  (`install.rs:655`), `Package~new` over lines (`install.rs:954`), `newFile` (`install.rs:1836`),
  external CALL (`lib.rs:2128`), `run_fragment` (`run/interpret.rs:31`) and `Method/Routine~new`
  (`class_protocol.rs:336`). `Loud::required_source` and `method_from_source` are deleted;
  `library_source` stays (b17).
- ⚠️ Cannot verify from the diff:
  - `whole_groups` was not run (OOM; a separate investigation owns it). See I2.
  - The `parse` running total is +0.6090% against `e6af1198b`, which is over the +0.5% budget, and
    +0.6004% of it was already there at `f1202acc1`. `cg2.log` confirms both figures. The budget rule
    ("three rounds, then stop for Moritz's ruling") is for the controller to apply. Task 5 itself adds
    +0.0085%.

### Checks run

1. **Every parse-error path against the oracle.** Rc, stdout, and stderr with the sub line dropped.
   All agree except I1:
   - Main program, untrapped: error at the first clause; at the last clause with no final newline;
     in a continued clause (`, ` joins); in a comment-terminated clause; after a comment that spans
     lines; in a `;`-separated middle clause; in a mid-line continued clause. Also covered:
     - CRLF and tab sources.
     - 13.1 on a 2-byte character, a 4-byte character, and a bare `80`x.
     - 6.1 for an unclosed comment and 6.2 for an unclosed quote on a continuation line.
     - 99.943, 19.x on `::class`, 14.x for an unclosed DO and for THEN at EOF (with and without a
       final newline), 7.x, 8.x, 10.1 after labels, 47.1, 15.x, and an error inside a
       `::routine` body.
     - An 80 KB file with the error at line 20002, a 13.1 at line 20001, and 80 KB and 84 KB single
       clauses (one-line and 12000-way continued). Compared at full length, with the echo up to 80131
       bytes: identical.
   - `::REQUIRES`: nested (main requires mid, mid requires bad) untrapped; through an external call,
     trapped; from a subdirectory; at the first clause; with no final newline.
   - `Package~new(file)` trapped and untrapped, `loadPackage`, and `Package~new` over lines (a
     label line) and over a string.
   - `Method~newFile` untrapped.
   - `Routine~new` over a string, and `Method~new` with a scanner error.
   - External CALL, nested and untrapped; an external function call, trapped.
   - INTERPRET:
     - trapped, multi-line;
     - untrapped, multi-byte;
     - nested INTERPRET;
     - in a procedure;
     - CRLF;
     - a trailing LF (13.1, clause keeps its line end);
     - comments inside the fragment;
     - an unclosed comment;
     - after an earlier condition, inside a handler;
     - `Routine~new` inside an INTERPRET;
     - in a recursive handler chain;
     - `CALL ON ANY` (untrapped on both) and `SIGNAL ON ANY` (trapped on both).
   - INTERPRET after REPLY, trapped and untrapped: 5 runs per engine for each, all identical.
   - Debug-pause typed lines: `say (`, `interpret 'if then'`, and `call 'bad.cls'`. Code, traceback
     and stdout agree. Two divergences are identical at `f1202acc1`, so they predate this task and
     are not counted:
     - The oracle's `+++ "LINUX COMMAND <path>"` line.
     - The oracle not tracing `10 *-* return` after a typed `say (` followed by blank lines.
   - Condition `STACKFRAMES` agree on every trapped probe except I1.
2. **End offset.**
   - The 80 KB file probes above put the error past offset 65535: the line number and echo are
     identical.
   - Multi-byte 13.1 echoes end at the end of the UTF-8 sequence, matching the oracle for 2-byte,
     4-byte and invalid bytes.
   - Above 4 GiB, by reasoning only: see M1.
3. **Concern 3 (PROGRAM/PACKAGE).**
   - Measured: `condition('O')~program` and `~package~name` are `x` and `badm.rex` on the oracle,
     and the calling program on ours, for `.package~new('x', ...)`, `Routine~newFile`,
     `Method~newFile`, and an external call (`DIR/badm.rex`).
   - `STACKFRAMES` agree, including `COMPILE 1 x` and `COMPILE 1 badm.rex`.
   - The record at `phase-4-exclusions.txt:1960-1966` is true.
4. **Concern 4 (debug-pause leak).**
   - I removed `self.clear_failure_levels();` (`run/interpret.rs:282`) in a scratch copy of HEAD.
     `a_failing_pause_line_leaves_no_traceback_level` failed at `tests.rs:459` with `left: "3\n"`,
     `right: "1\n"`. With the line restored it passed.
   - Behaviour against `f1202acc1` with the typed lines `zz = 1/0` then `say (`: base 2 items, HEAD 1
     item, oracle 1 item.
5. **`.cls` rename.**
   - `bef52e1b6` renames six fixtures and updates the three `.rex` files and three
     `sourceline_oracle` copies.
   - `git grep -E 'bad(call2?|m|r|u|req)\.rex' a46465a9f` finds only the historical scout report
     and the exclusions example, which is measured true above. Nothing depends on the old names.
   - `.cls` files in `.d` directories are the corpus's existing practice.
   - `call badcall` became `call 'badcall.cls'`. The witness now uses a quoted filename instead of
     the bare-name search. After resolution it reaches the same parse path.
6. **Records.**
   - Sweep: re-ran `task-5-parse-sweep.py` with OURS pointed at my HEAD build: `rows 567 same 565`.
     The two remaining rows are 434 and 753, 18.x against 35.1, pinned at
     `rexx-parse/tests/errors.rs:458-479`.
   - Gate record:
     - The binary sha256s match `cg2/binaries.txt` and the files on disk.
     - `base61` is the `e6af1198b` binary (`2ea19b3e...`).
     - Each build log has its `Compiling rexx-exec` line.
     - The deltas recompute from `cg2.log`: startup r1 against base is 131174/58120046 = +0.2257%, and
       parse is +0.0085%.
     - The wall-clock table matches `wall.log`.
   - `refusal-sites.tsv`: the deleted rows (`required_source`, `method_from_source`, `source_syntax`)
     and the moved `Raised from` row match the diff.
   - `rexx-parse --test errors` on the HEAD copy: 31 passed. `every_sample_program_parses` failed
     only because the archive has no `samples/` directory (`errors.rs:644`), so that failure comes
     from my environment.

### Strengths

- The oracle match on the traceback is broad: line, echo (including joined continuations, kept
  comments, and an INTERPRET clause's own line end), `COMPILE` stack frame, and the
  `Compiled method` level. It held across every shape in item 1, including inputs the witnesses do
  not cover (CRLF, 4-byte UTF-8, 80 KB clauses, nested INTERPRET, handlers after earlier
  conditions).
- `Rejected`/`program_from`/`fragment_from` return the source without cloning it. The old
  signatures stay as thin wrappers (`rexx-parse/src/lib.rs:88-140`).
- The perf round that kept `ParseError` at two words, cutting startup from +0.53% to +0.23%, is
  measured and recorded with instruments that can be checked.
- The debug-pause leak was found, fixed, and pinned by a test that I confirmed goes red without the
  fix.

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

**I1. A parse failure in a file that a `::REQUIRES` inside `Routine~newFile`, `Method~newFile` or
`Package~new(file)` loads diverges from the oracle, and nothing records it.**
- It enters through `install.rs:655`, called from the `install_executable` prologue after
  `install.rs:1836`.
- Probe: `r = .routine~newFile('mid.cls')`, where `mid.cls` holds `::requires 'bad.cls'` and
  `bad.cls` holds `x = (`.
- Trapped:

  | | traceback | `STACKFRAMES` |
  |---|---|---|
  | oracle | 4 items: `1 *-* x = (`, `2 *-* ::requires 'bad.cls'`, `*-* Compiled method "NEWFILE" with scope "Routine".`, the call | 4 |
  | ours | 2 items: the first and the last | 2 |

- Untrapped (rc 221 on both): our stderr omits `*-* Compiled method "NEWFILE" ...` and the calling
  clause `2 *-* r = .routine~newFile('mid.cls')`.
- `.package~new("mid.cls")` behaves the same way.
- At `f1202acc1` this shape was the loud rc 120 `does not parse here`. Task 5 turned that refusal
  into a silently divergent answer.
- The runtime analogue (`bad.cls` = `x = 1/0`) diverges identically at both base and HEAD, so the
  level-chain gap predates this task. Its recorded family ("THE TRACEBACK OF A RAISE INSIDE
  .Package~new OR loadPackage LACKS ...", `phase-4-exclusions.txt` near :5466) names neither
  `newFile` nor the `::REQUIRES` level.
- Meanwhile the new row at `phase-4-exclusions.txt:1948-1958` states agreement "for ... a ::REQUIRES
  file, ... Method~newFile, Routine~newFile, Package~new". That is true only for each one on its own.
- Fix: record the nested shape, with the measurement, beside the existing family row, and narrow the
  agreement sentence. Fixing the level chain is the alternative.

**I2. `whole_groups` expectation rows quote a message that HEAD can no longer produce.**
- The rows are in `tests/concurrency_tests.rs`, at :2688, :2695, :2730, :2737, :2758, :2765, :2800,
  :2807, :2842, :2849, :2898, :2905 (+:2899, :2906).
- They expect `rexx-exec: test does not parse here: ... is not implemented (Phase 5), rc 120`. That
  is `Loud::required_source`'s text, which this task deleted. The only remaining producer of
  `does not parse here` is `Loud::library_source` (`lib.rs:605`), which names an embedded `.orx`.
- These rows are therefore known false. The report's "unchanged and unverified" understates that.
- The brief's Step 2 ("update the whole-group expectation lines") is not done. It is blocked on the
  OOM, so the controller has to decide whether this blocks Task 5 or carries to the OOM
  investigation.

#### Minor (Nice to Have)

- **M1.** `token.rs:38-42, 59-71`: `end` saturates at `u32::MAX`, so the echo degrades silently above
  4 GiB:
  - If `byte` is above `u32::MAX`, `clause()` is empty, because of `max(self.byte)`.
  - If only the end is above, the echo is truncated.

  This is reasoned, not run. The field's doc gives the two-word reason but not this consequence. Add
  one sentence, or a `debug_assert!`.
- **M2.** `error.rs:1872`: `"<clause span outside the retained source>"` is invented
  user-visible text on a path that should be unreachable. An `expect`, or a `debug_assert!` plus the
  empty echo, states the invariant instead. The same fallback already exists at `run.rs:3798`, so
  this copies an idiom already in the code. Optional.
- **M3.** Three copies of "`ProgramSource::new(text, SourceKind::Program)`, `program_from`, map
  `Err` to `raise_parse_failure`": `lib.rs:2128-2131`, `install.rs:655-658`, `install.rs:1836-1838`.
  `class_protocol.rs:336` already shows the helper shape (`parse_source_lines`), and a file
  equivalent would remove the copies.
- **M4.** `phase-6-1-gate.md:502`: "the pad prints the same as `base`" is true for `parse` but not
  for `startup`, where base is +0.0388 and pad48 is +0.0395. The band conclusion of about 0 stands.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** The mechanism is right and matches the oracle across every shape I ran, including
large and multi-byte sources. What is left are records: a converted refusal that diverges unrecorded
(I1), and whole-group rows known false that the brief asked to rewrite (I2, blocked on the OOM).
