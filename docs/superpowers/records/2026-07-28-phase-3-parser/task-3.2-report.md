## Task 3.2 report: `ProgramSource` and `SOURCELINE`

Status: DONE

Commit: `68a3b553cc7ca2f16a0b883d2c3c422d78112650`

Test summary: 9/9 new tests pass (`cargo test --offline -p rexx-parse`);
`cargo clippy --offline --all-targets -- -D warnings` clean workspace-wide;
`cargo fmt -p rexx-parse -- --check` clean (no `cargo fmt` was run).

### Files

- `rust/crates/rexx-parse/Cargo.toml` (new crate, no dependencies, `[lints] workspace = true`)
- `rust/crates/rexx-parse/src/lib.rs`
- `rust/crates/rexx-parse/src/source.rs` -- `ProgramSource`
- `rust/crates/rexx-parse/tests/sourceline.rs`
- `rust/Cargo.lock` (updated, new package entry only)

### Oracle probes (`build/bin/rexx`)

All probe files were written to the scratchpad directory; paths omitted below,
only content and output shown.

**1. Basic count/content (brief's Step 1 driver):**
```
say sourceline()
say "[" || sourceline(1) || "]"
say "[" || sourceline(2) || "]"
```
Output:
```
3
[say sourceline()]
[say "[" || sourceline(1) || "]"]
```
Confirms: no argument returns the total line count; a line's text has no
trailing newline.

**2. `sourceline(0)`:**
```
Error 40.14:  SOURCELINE argument 1 must be positive; found "0".
```

**3. `sourceline(99)` (file has 1 line):**
```
Error 40.34:  SOURCELINE argument 1 ("99") must be less than or equal to
the number of lines in the program (1).
```
Both confirm the brief: out-of-range is a hard error (40.14 / 40.34), not an
empty string. `ProgramSource::line` returns `None` for both; a later task
maps that to these error numbers.

**4. Final line with no trailing newline** (`say sourceline()\nsay "[" ||
sourceline(1) || "]"`, no final `\n`): output `2` then
`[say sourceline()]`. Confirms the unterminated last line still counts.

**5. CRLF file** (three lines, each ending `\r\n`): output `3`,
`[say sourceline()]END`, `[say "[" ...]"]"...]END` -- the returned line text
contained neither `\r` nor `\n`.

**6. Empty file, and a file containing only `\n`:** both ran with exit 0 and
no output (no executable statement exists to call `SOURCELINE`, so this only
confirms the file is a legal, trivial program -- it does not by itself pin
`line_count()` for an empty buffer; see below).

**7. Comment line + statement + trailing blank line**
(`/* comment line 1 */\nsay sourceline()\n\n`, i.e. ending in `\n\n`):
output `3`. Confirms a trailing `\n` does **not** create an extra empty
line after it (blank line before EOF still counts once), which is the basis
for `new`'s "no terminator found -> that's the last line" / "terminator
consumes the whole remaining buffer -> loop simply ends" logic.

**8. Bare `\r` (no `\n`) as the only line terminator, old Mac style**
(three lines separated by bare `\r`): output `3`,
`[say sourceline()]END`, `[say "[" ...]"]END`. This is **not called out
anywhere in the brief**. A bare `\r` is a full terminator on its own, not
just the first half of a CRLF pair.

**9. `\n` immediately followed by `\r` (LF-then-CR, not CRLF):**
(`say sourceline()\n\rsay "done"\n`): output `3` then `done`. Confirms
LF-then-CR is **two** terminators (unlike CR-then-LF, which is one),
producing an empty line between them. Only one `say` actually executed
(`done`), consistent with the middle "line" being empty, not garbage.

**10. Ctrl-Z (0x1A) mid-stream, at the start of the byte right after a
statement's trailing `\n`:**
(`say sourceline()\n\x1asay "should not run"\nsay "also not"\n`): output `1`,
exit 0. Also **not in the brief.** I found this by reading
`interpreter/parser/ProgramSource.cpp:373`-`377` (this project vendors the
original C++ source), which truncates the buffer at the first Ctrl-Z byte
before any line scanning happens, and confirmed it against the running
interpreter.

**11. Ctrl-Z mid-line (not right after a terminator):**
(`say sourceline() /* comment\x1a still comment */\nsay "x"\n`): the
interpreter reported line 1 as `say sourceline() /* comment` (truncated
exactly at the 0x1A byte, mid-comment) and raised
`Error 6.1: Unmatched comment delimiter ("/*") on line 1`, because
everything from the 0x1A onward -- including the closing `*/` -- was
discarded before parsing. This pins that truncation happens at the exact
byte offset, not at the next line boundary.

**12. A case matching one of my own written tests exactly**
(`say sourceline()\nsay 2\x1a more\nsay 3\n`): output `2` then `2`,
confirming `sourceline()` == 2 and `sourceline(1)`/`sourceline(2)` reflect
only the truncated two lines (`say sourceline()`, `say 2`).

### Design decisions and one deviation from the brief's literal text

- **CRLF/`\r` placement:** implemented exactly per the orchestrator's
  resolution (line content excludes the terminator; CRLF collapses to one
  terminator), but the real rule is broader than "trim a trailing `\r`
  after CRLF" -- `\r` and `\n` are *independently* valid terminators, per
  probes 8 and 9 above. `ProgramSource::new`'s doc comment and two of the
  nine tests (`crlf_pair_is_one_terminator_but_lone_cr_ends_a_line_on_its_own`,
  `lf_then_cr_is_two_terminators_producing_an_empty_line`) cover this.

- **Ctrl-Z truncation (probes 10-12):** not mentioned anywhere in the
  brief, but it lives in the exact function I ported
  (`BufferProgramSource::buildDescriptors`), is fully verified against the
  running interpreter three ways, and per the task's own constraint
  ("Behaviour is defined by what `build/bin/rexx` does... where they
  disagree, the interpreter wins") I implemented it in `ProgramSource::new`
  rather than deferring it. Flagging this prominently since it's scope the
  brief didn't ask for -- worth a second look in case a later task expected
  to own this instead.

- **Empty source, `line_count() == 0`:** the oracle cannot directly pin this
  (a zero-byte program has no statement that could call `SOURCELINE`).
  I read `ProgramSource.cpp:387`'s `while (bufferLength != 0)` loop guard,
  which never executes for an empty buffer, leaving `lineCount` at its
  initialized 0. `line_count()` returns 0 and `line(1)` returns `None` for
  an empty `Vec<u8>`; `line_of` is guarded to return `1` rather than panic
  in this case (unreachable in practice, since no token can point into a
  source with no lines).

- **Brief's Step 2 test snippet is not directly compilable:** it writes
  `assert_eq!(src.line(1), Some("say 1"))`, comparing `Option<&[u8]>` to
  `Option<&str>`, which does not typecheck against the brief's own stated
  signature `line(&self, n: usize) -> Option<&[u8]>`. I used byte-string
  literals (`Some(&b"say 1"[..])`) in the actual test file, matching the
  pattern the brief's own third test already used correctly.

### Concerns

- The Ctrl-Z and bare-CR/LF-then-CR behaviours above are real and verified,
  but they're additions beyond the brief's explicit scope. If Phase 3
  intended `ProgramSource` to stay minimal and push Ctrl-Z handling to a
  later task (e.g. the scanner in 3.3), this needs to move; as written, it's
  load-bearing for every later task since they all hold ranges into this
  crate's truncated `text`.
- No other concerns. `line_of` uses `Vec::partition_point` (binary search),
  satisfying the brief's throughput requirement without a manual
  `binary_search_by`.
