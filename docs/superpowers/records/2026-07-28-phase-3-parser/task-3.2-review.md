## Task 3.2 review: `ProgramSource` and `SOURCELINE`

### Verdicts

* **Spec compliance: PASS.**
  All four interfaces (`new`, `line`, `line_count`, `line_of`) exist with the
  exact signatures the brief specifies. `Vec<u8>` in, `&[u8]` out, everywhere.
  No `Result`, no `unsafe`, no dependency added. `ParseError` is correctly
  left to Task 3.3. The Ctrl-Z and independent-CR/LF behaviours are real
  (already confirmed by the orchestrator) and correctly implemented in the
  one function that owns them.

* **Code quality: APPROVED.**
  I hand-traced the line-splitting state machine and `line_of`'s binary
  search across every boundary class the brief calls out, then stress-tested
  the compiled crate with a throwaway test file (`review_probe_tmp.rs`,
  written to `rust/crates/rexx-parse/tests/`, run via `cargo test --offline
  -p rexx-parse`, and deleted afterward -- `git status` on `rust/` is clean).
  All 7 probes passed. I also ran two oracle probes against
  `build/bin/rexx` for terminator patterns neither the report nor prior
  review had checked (double bare CR, and LF-LF-CR). Both matched the
  implementation exactly. No off-by-one, no panic path, found anywhere.
  Two Minor polish items below, no Critical or Important findings.

### Verification performed

**Index arithmetic at boundaries** (brief's own list), via a temporary test
file exercising the compiled crate directly:

* No trailing terminator (`"ab\ncd"`): `line_of` correct at line-1 start (0),
  mid-line-1 (1), the terminator byte itself (2, attributed to line 1),
  line-2 start (3), mid-line-2 (4), one-past-end (5, clamps to line 2), and
  far-past-end (100, clamps to line 2). No panic.
* With trailing terminator (`"ab\ncd\n"`): terminator byte (5) and exactly
  `len` (6) both clamp to line 2, no panic.
* Empty source: `line_count() == 0`, `line_of(0) == 1`, `line_of(5) == 1`
  (the documented `.max(1)` fallback -- unreachable in practice, but doesn't
  panic).
* File that is only terminators: `"\n"` -> 1 line, empty content;
  `"\r\n"` -> 1 line, empty content; `"\n\n\n"` -> 3 lines, all empty. Traced
  by hand against the state machine and confirmed by direct execution.
* Ctrl-Z truncation boundary: for `"say 1\nsay 2\x1a more\nsay 3\n"`
  (retained text truncated to 11 bytes, 2 lines), `line_of(11)` (exactly the
  truncation point), `line_of(20)`, and `line_of(1000)` all clamp to line 2
  with no panic.

All 7 assertions passed on the first run; I re-read the actual output rather
than assuming green.

**Terminator handling**, traced by hand over the brief's exact list, then
partially re-verified against the running interpreter for the two patterns
nobody had tested yet:

| Input | Hand trace | Oracle (where checked) |
|---|---|---|
| `a\r\nb` | 2 lines: `a`, `b` (CRLF collapses) | matches shipped test |
| `a\n\rb` | 3 lines: `a`, ``, `b` | matches report's probe 9 |
| `a\r\rb` | 3 lines: `a`, ``, `b` | **checked now**: `say sourceline()\r\rsay "x"\r` -> `3`, then `x` |
| `a\n\n\rb` | 4 lines: `a`, ``, ``, `b` | **checked now**: `say sourceline()\n\n\rsay "y"\n` -> `4`, then `y` |
| `\r\n` | 1 line, empty | matches shipped test pattern |
| `\r` | 1 line, empty | matches report's probe D-style trace |
| `a\r` | 1 line: `a` | matches report's probe D exactly |

Every hand trace matched the code's actual branch structure (only `\r`
triggers a look-ahead for a following `\n`; `\n` never triggers a
look-ahead), and both newly-checked patterns matched the oracle exactly.

### Findings

**Critical:** none.

**Important:** none.

**Minor:**

1. `line_of`'s public doc comment does not state its own contract for
   out-of-range input. It says only "The 1-based physical line containing
   byte offset `byte`." The clamp-to-last-line behaviour for a byte past the
   end, and the `.max(1)` fallback for a source with zero lines, are stated
   only in the `//` reasoning comment inside the function body. Per this
   project's convention (contract in the doc comment, reasoning at the
   decision point), that clamp behaviour belongs in the `///` doc comment;
   the reasoning comment should explain *why* `partition_point` produces
   that clamp, not be the only place the clamp is documented at all.
   Contrast with `line`'s doc comment, which does state its out-of-range
   contract (`None` for `n == 0` or past `line_count()`) up front. Behaviour
   itself is correct and verified; this is a documentation-placement gap,
   not a defect.

2. Test-suite coverage/naming gaps, not correctness gaps (I independently
   verified the underlying behaviour is right in both cases):
   * `crlf_pair_is_one_terminator_but_lone_cr_ends_a_line_on_its_own` tests
     only the second half of its own name (`b"say 1\rsay 2\r"`, bare CR).
     The "CRLF pair is one terminator" half is verified by a different,
     differently-named test (`crlf_terminators_are_excluded_from_line_content`).
     A reader relying on test names to map to claims would misread this one
     as self-contained.
   * `line_of_is_one_based` and `empty_source_has_no_lines` don't exercise
     `line_of` at a terminator byte, at exactly `len`, or on an empty
     source at all -- exactly the boundaries the brief calls out as highest
     risk. I confirmed correctness for all of these myself (see above), but
     the shipped suite doesn't, so a future regression in this exact spot
     would not be caught by `cargo test -p rexx-parse`.

### Cannot verify from diff alone

* Whether Task 3.3's scanner will ever call `line_of` with an offset that
  predates Ctrl-Z truncation (i.e., whether the "unreachable in practice"
  claim holds structurally once the scanner exists). The report's reasoning
  is sound given `ProgramSource::new` truncates before any token can be
  produced, but this depends on code that doesn't exist yet.
* Whether Task 3.8 will correctly map every `None` from `line` to the right
  error number (40.14 for `n == 0`, 40.34 for `n` past `line_count()`) --
  this diff only sets up the `Option` return; the mapping is out of scope
  here and unverifiable until 3.8 lands.

### Bottom line

The index arithmetic is correct at every boundary the brief flags as
high-risk, including two terminator patterns nobody had checked against the
oracle before this review. No `String`/`&str` drift, no `unsafe`, no
`Result`, no column anywhere. The two Minor items are documentation
placement and test-coverage polish, not defects -- clean to merge as-is,
polish optional.
