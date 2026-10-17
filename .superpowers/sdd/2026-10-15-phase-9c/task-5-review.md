# Phase 9c Task 5 review: `rexx` beside `rexx-run`, and the A/B

Reviewer: c9c-review-5 (opus). Range `6391ead7c..dc16e526e`, read from
`review-6391ead7c..dc16e526e.diff`. Builds at `dc16e526e` in a detached worktree, plus `6391ead7c`'s
`rexx-run` for one regression A/B. Probe scripts and transcripts are in the
session scratchpad's `9c-r5/` (`tools/probe*.sh`, `probe1.txt`, `probe2.txt`, `runs*/`). That is
`/tmp` and not durable. The shapes are listed under Checks run so they can be re-run. Each oracle run used the standard wrapper from a fresh fixture directory, and stdout,
stderr and rc were compared separately.

### Spec Compliance

- ❌ Issues found:
  - The program-path chain widened under ruling (A) regresses resolution from any non-ASCII directory
    (Critical 1).
  - The canonical reported path the brief mandates diverges from the oracle for a program reached
    through a symlink (Important 1, plan-mandated).
  - Everything else the brief asks for is present and matches the oracle. Option scanning is a
    faithful port of `rexx.cpp:81-150`, `.SYSCARGS` uses the `argc_base - 1` index, `-e` runs as
    INSTORE without `SYSCARGS` and with the short status, the usage text gives rc 255, 3.901/3.1 give
    rc 253 with no `running` line, `-o`/`-od` take the count and the colon rule, and
    `interpreter_bin`, `check_path` and `suite.rs:1902,1915` all name `CARGO_BIN_EXE_rexx`.
- ⚠️ Cannot verify from diff: nothing outstanding beyond the findings.

### Strengths

- `launcher::parse` (`launcher.rs:71-124`) is a line-for-line port of `rexx.cpp:81-150` that keeps
  the oracle's `argc`/`i` arithmetic, so `-e` not consuming its text word and `-o` consuming its
  string fall out of the port rather than being re-derived. `options_directive`'s read past the
  string (`launcher.rs:134-151`) reproduces `rexx.cpp:116`'s out-of-bounds look, and it is pinned.
- 41 shapes outside `rexx_command/cases` (list below) agree on all three descriptors except the
  symlink ones. That includes `-o` twice, `-o` then `-od`, `-od STR` with no program, `-e` with a
  `'ff'x` argument and in its text, empty and blank-edged arguments (ARG(1) `a  b  c `,
  `.SYSCARGS` items as hex), absolute and trailing-slash names, a directory as the program,
  `REXX_PATH` and `PATH` with empty entries, `-version`/`-exec`/`-output`/`-odx`, a `'ff'x` program
  in a `'ff'x` REXX_PATH directory, and a malformed `-o` against a missing program and against a
  directory.
- `'ff'x` names reach `PARSE SOURCE` and `Error N running` byte for byte, including through `-o`
  (`-o 'digits 12' p'ff'x`) and through a `'ff'x` REXX_PATH entry.

### Issues

#### Critical (Must Fix)

1. **`resolve_search` mixes widened and unwidened strings, so a program in a non-ASCII directory
   cannot `::REQUIRES` or `CALL` a sibling, and a non-ASCII cwd panics** (`install.rs:932-960`,
   `require.rs:186-192`). `program` is widened byte-per-char (`install.rs:933`). `name`, `cwd`
   (`self.cwd.to_str()`) and `REXX_PATH`/`PATH` (`shadow_var`) stay real UTF-8. Every candidate is
   then `stat`ed as its UTF-8 encoding (`install.rs:954`), and only the hit is narrowed
   (`install.rs:960`). `launcher::resolve_program` does this correctly: it widens every input and
   narrows before each `stat` (`launcher.rs:162-184`). Measured at `dc16e526e`, each against the
   oracle:
   - `dé/req.rex` (`::requires "rq.rex"`, sibling present): oracle rc 0, crate 43.901 rc 213.
     `dé/ext.rex` (`call other`): oracle rc 0, crate 43.1 rc 213. The same holds for `d'ff'x/`.
   - From a cwd `d€` (a char above U+00FF), `rexx ext.rex`: the crate **panics** at
     `require.rs:191:34` "a widened path holds only chars below 256", rc 101. The oracle gives rc 0.
   - `REXX_PATH=.../rpé`, `call farcall`: oracle prints `far`, rc 0. The crate gives 3.1
     `File ".../rp\xE9/farcall.rex" is unreadable`, rc 253, because narrowing a real `é` yields the
     byte `E9`.
   - **Regression**: base `6391ead7c`'s `rexx-run` answers rc 0 with the oracle's output for the
     `dé` and `rpé` shapes. HEAD's `rexx-run` fails `dé/req.rex` the same way, so the defect is in
     the library and not in the launcher.

   Fix: widen `name`, `cwd` and both environment values in `resolve_search`, and narrow each
   candidate before `metadata`, as `resolve_program` does. Better, make both callers share one
   function. Add cases for a UTF-8 directory (requires, external call, REXX_PATH) and for a cwd
   with a char above U+00FF. `rexx_command`'s fixture never has a program outside an ASCII
   directory, so no current test can see this.

#### Important (Should Fix)

1. **Plan-mandated: the canonicalised reported path resolves symlinks, and the oracle does not**
   (`bin/rexx.rs:110-113`; the brief's "the canonical reported path"). Measured:
   - `link.rex -> sub/t.rex`: the oracle says `LINUX COMMAND DIR/link.rex`, the crate says
     `DIR/sub/t.rex`.
   - `lsub/t.rex` with `lsub -> sub`: the oracle says `DIR/lsub/t.rex`, the crate says
     `DIR/sub/t.rex`.
   - `link` (found through `.rex`): same split.

   The comment at `:110-112` ("one step further than the oracle has been measured to go") is now a
   measured divergence on `PARSE SOURCE` and every error report. Fix, tried here:
   `let reported = found;`. `resolve_program` already returns the absolute dot-normalised path.
   With that change `rexx_command` stays green (1 passed) and all three symlink shapes match the
   oracle. Add a symlink case, and delete the comment.

#### Minor (Nice to Have)

1. **The report's explanation of `interpret`'s base move is not supported** (`task-5-report.md`,
   Step 5, last bullet). Between `cg-9c/ab` and `cg-9c/ab-9994f5c1c`, `interpret.base.r1`:
   - libc is identical. `_int_malloc`, `free` and `memcpy` show the same Ir to the instruction.
   - The whole -77.86M Ir sits inside `rexx_parse::token::symbols::SymbolTable::intern`, inclusive
     2,248,508,182 → 2,170,567,335, over the same 3,120,624 calls. The loss is spread across its
     inlined hashbrown, rustc-hash and core lines.
   - `git diff --stat 629f2d79b 9994f5c1c -- rust/crates/rexx-parse` is empty.

   So this is a codegen change in an unchanged function, not removed path copies. No path code runs
   under `intern`. It is not a regression, because `head` equals `base` there. It also does not
   bear on the front end's share. Replace the sentence with this measurement, or delete it.
2. **`bin/rexx.rs:138-139`**: "The two descriptors' interleaving is not observable (D17)" is false
   by this task's own finding. `TEST_PASSINGARGS7` observes the merge through a shared pipe, and the
   ruling drops Deviation 40's interleave clause. The sentence also names a set's size. Delete it.
3. **`launcher.rs:44`**: `syscargs: (Vec<Vec<u8>>, usize)` is a bare tuple whose second half is an
   index base. A named field, or `with_syscargs` taking the `Run`, would read better. This is
   cosmetic only.

### Checks run

1. **Oracle comparison**, beyond the `cases` file, 41 shapes, three descriptors each:
   - All agree except the three symlink shapes (Important 1) and the non-ASCII-directory shapes
     (Critical 1).
   - Shapes: `-o` ×2; `-o`+`-od`; `-od STR` alone (usage, 255); `-o` before `-e`; `-e` with a
     `61FF62` argument (`61FF62`, arg() 1); `-e "say c2x('a'ff'x')"`; `-e` syntax error with
     `'ff'x` (35.1, INSTORE, 221); `showargs '' a '' 'b  c' ''` (items 5, hex per slot); blank-edged
     arguments; `-e` with the same; `./sub/t.rex`; an absolute `DIR/sub/t`; `sub`; `adir`; `sub/`
     (3.901 with the typed name); `REXX_PATH=:DIR/rp`, `DIR/rp::`, empty, `::`; `PATH=:DIR/pp:...`
     and `DIR/pp::...`; `both`/`both.rex` (`.rex` first); `T.REX`; `t.`; `''` (3.901 `""`);
     `-version`; `-exec`; `-output`; `-odx`; `showargs -v -e x`; `-o 'trace all' nonexist`;
     `-o '::options digits 12 ::requires ...'` against a missing program and against `t.rex`
     (25.924 with position, 231); `-o 'digits 12' p'ff'x`; `p'ff'x`; `-e 'exit -70000'` and
     `'exit 32768'` (0); `-e 'say 1/0' -o x` (214); `-e '' a`; `-x -o` and `-o -x` before
     `showargs q` (size 4, slots 1-3 empty); `-o 'bogus 9' sub`; `-o 'digits 12' local.rex`
     (10 entries with SYSCARGS); `-od 'digits 12' locked.rex` (3.1, 253); `qq'ff'x` and
     `qq'ff'x.rex` via a `'ff'x` REXX_PATH.
2. **The two mains**: `rexx.rs` keeps every element of `rexx-run.rs:18-149`. That is
   `install_signal_handlers`, `ProgramInput::Stdin`, the `/proc/self/fd` transient flags, both
   flushing sinks, `REXX_SWITCH_MODE` (every, at:K, sim with its seed line, status 2 on a malformed
   mode), the post-run `rexx-sim:` line under the `SIM_CHILD_ENV` filter, the canonical path, and
   `as u8` truncation. The only textual change is the `rexx:` prefix.
3. **The bytes chain**: no `to_string_lossy`, `from_utf8_lossy` or `String::from` is added on the
   program-path chain (`git diff ... | grep`). The two removed ones are `error.rs`'s package span
   and `execute_on`'s. The remaining `from_utf8_lossy` sites are outside the chain
   (`install.rs:1919` `new_file_executable`, reported as still open, and `run/settings.rs`'s option
   parsing). The live lossy point is `resolve_search`'s `from_utf8(name).ok()?` plus the mixed
   widening in Critical 1.
4. **Concern 4**:
   - The empty-program-first order reproduces the oracle wherever measured: `-o 'bogus 9'` against
     a missing program, against a directory and (in `cases`) against nonexist; `-o '::options ...
     ::requires'` against a missing program; `-o 'trace all' nonexist`, which prints no trace
     output on either side; `-od` against an unreadable file.
   - The empty run's own outcome is dropped when it exits 0, and the not-found path prints no
     `rexx-sim:` stats line either way. I found no other observable.
5. **`rexx_command` group**, from `rust/` with a separate target dir: status 101. The table shows
   oracle 30/30 and crate 29/30, `TEST_PASSINGARGS7` the only red. `tests.tsv` is identical to
   `task-5/suite2/tests.tsv`.
   - Isolation, `merge.rex` with the parent and the child swapped:
     - oracle parent with oracle child: in order;
     - oracle parent with crate child: the `SAY`s first;
     - crate parent with oracle child: in order;
     - crate parent with crate child: the `SAY`s first.

     The child alone decides the order, as the report claims.
6. **A/B**:
   - `head-minus-base.tsv` reproduces from `table.txt` with the report's awk (no diff).
   - Every `table.txt` cell is the median of `summary.tsv`'s three rounds, and every percentage
     recomputes (0 mismatches over 32 programs). There is no `OVER`.
   - The pin, base and head sha256s match `binaries.txt`.
   - The front end's share is +28-30k Ir per run, or 17k with libc subtracted, except `pingmsg`'s
     11k. The four rows that move more (`churn`, `hashnew`, `interpret`, `rexxcps`) are libc, and
     their `-libc` deltas are 17-18k like the rest.
   - `interpret`: see Minor 1. Nothing about the front end needs explaining.
7. **Tests can fail**, as mutants in the worktree, each restored by copy and checked with
   `git diff --stat`:
   - `rexx.rs`: dropping `instore_status` reddens `-e return 70000`.
   - `launcher.rs`: `Some(b'd' | b'D')` → `Some(b'd')` reddens `-oD digits 12 req.rex`.

   Unmutated, `rexx_command` gives 1 passed.
   - Prose: no em-dashes and no ` -- ` in the added `.rs` lines.
   - Set sizes: Minor 2, and "the three descriptors" in `rexx_command.rs:14,155`, which is the
     project's term of art.

   Process note: a second worktree sharing one `CARGO_TARGET_DIR` made cargo link the base
   `rexx-exec` rlib into a later HEAD build, and that build failed to compile. I re-ran every probe
   whose binary was in doubt after touching the sources and confirming a `Compiling` line.

### Assessment

**Spec compliance:** ❌. The widened chain regresses non-ASCII directories (Critical 1), and the
mandated canonical path diverges for symlinks (Important 1). Everything else complies.

**Task quality:** Needs fixes

**Reasoning:** The launcher is a careful, well-tested port, and 41 off-case shapes agree with the
oracle. But `resolve_search`'s half-widened strings break `::REQUIRES` and external calls from any
non-ASCII directory, and panic for a cwd with a char above U+00FF. This worked at base, and no
current test can see it.
