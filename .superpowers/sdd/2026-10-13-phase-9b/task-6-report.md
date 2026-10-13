# Phase 9b Task 6 report: the RexxUtil message routines

## Status

DONE_WITH_CONCERNS. `base/rexxutil/platform/unix/SysGetMessage` is 9 of 9 started (its other two
tests keep their Phase 10 `Rxapi` rows) and `base/rexxutil/SysFormatMessage` is 5 of 5, both green
under the harness. `base/keyword/ADDRESS` stays red on one test, `TEST_COMMAND_CD`, which this task
unblocked and does not fix (concern 1). Commits:

- `d1185ae85` Phase 9b Task 6: the RexxUtil message routines
- the records commit that carries this report (sha in the reply)

## What changed

New `builtin/rexxutil/messages.rs` (and `messages/tests.rs`). The four rows in
`internal_routines.rs` now run.

- **`SysGetMessage` / `SysGetMessageX`.** Arguments as `positive_wholenumber_t` (88.901, 88.905)
  and `OPTIONAL_CSTRING` (88.909, cut at a NUL), then `getErrorMessage`, then `formatMessage` over
  the argument list from position 3 (4 for `X`). Numbers are truncated to an `int` as the C++
  passes them on: measured, `SysGetMessage(4294967301)` is message 5 and
  `SysGetMessageX(4294967297, 5)` is set 1.
- **The interpreter's own messages** (no catalog, or exactly `rexx.cat`): set 1 only, found by
  message number among the messages whose major code is below 102, which is what
  `Message_map_table` holds (`RexxMessageTable.xsl`: `Message[Code < 102]` and their subcodes).
  The texts come from `rexx_inventory::errors::MESSAGES`. Message numbers in that range are unique
  (checked over the generated table). The witness prints every message found in 1..999 on both sides.
- **Any other catalog: glibc 2.43's `catopen(name, 0)` and `catgets`, ported from the source**
  (`catgets/catgets.c`, `catgets/open_catalog.c`, fetched at tag `glibc-2.43`; this machine runs
  `Debian GLIBC 2.43-7`). A name with a `/` is opened as is, relative to the program's directory.
  Otherwise `NLSPATH` (with glibc's default appended, or the default alone where it is unset or
  empty; the default string was checked in `libc.so.6`) is walked with `LANG` (`C` where unset or
  empty) for `%L`, `%l`, `%t`, `%c`, `%N`, `%%`, an unknown `%` code dropping the element, an empty
  element meaning the bare name. The first element that opens is the file, whatever it is: a
  directory there ends the search with "catalog not found". Then glibc's checks: regular file of
  at least 12 bytes, magic in either byte order, the size test and the terminator scan **with
  glibc's own arithmetic** (it counts table words as bytes), over the file plus the zeros to the end
  of its page, which is what `mmap` gives. `catgets`'s hash, `++set <= 0`, and its do-while over the
  depth are ported as written. An empty message answers "Message not found".
- **Which catalog the oracle opens.** For no catalog or `rexx.cat`: none, it never calls
  `catopen`. For `doesnt_exist.cat` (the test group's name) under `LANG=en_US.UTF-8`, `strace`
  shows exactly the four default paths (`probes/st.strace-cat.txt`), and this machine has no
  `.cat` file anywhere (`find / -name '*.cat'` found none). The test group's catalog test builds one
  with `gencat` and opens it by an absolute path.
- **`SysFormatMessage`.** `CSTRING` then `RexxArrayObject` (`requestArray`, 98.913 for no
  single-dimensional array, 88.922, 88.901). `formatMessage`: the value count is the array's
  **size**, not its items (measured: `.array~new(10)` is 40.1, `~remove(10)` keeps size 10 and is
  40.1, `~delete(10)` shrinks it); more than nine is 40.1 naming `SysFormatMessage` even from
  `SysGetMessage`. Every value is converted before the scan, item by item from the live array (a
  `STRING` method that changes a later item is seen, witnessed), an omitted one as the null string,
  each as a C string. A raising conversion does not stop the others and the **last** raise is the
  answer (measured with two raisers). A `STRING` answering `.nil` reads as the null string
  (measured), `.nil` itself as `The NIL object`.
- **`SysGetErrorText`.** `int32_t` through the existing `int_argument` (88.907), and the text is
  `std::io::Error::from_raw_os_error(n)`'s display with its ` (os error n)` suffix stripped. The
  standard library uses glibc's `__xpg_strerror_r`, which gives `strerror`'s text, untranslated
  on both sides since neither calls `setlocale`. The suffix is asserted present for -300..300 and
  both extremes (unit test), and the witness compares -3..140 and both extremes with the oracle.
- `stems::positive` is `pub(super)` for reuse. `rexx-exec`'s `rustix` gains the `param` feature
  for `page_size()` (no new dependency, `Cargo.lock` unchanged).

## Tests and results

Commands from `rust/`, foreground, statuses unpiped. Raw outputs in
`docs/superpowers/records/2026-10-13-phase-9b/task-6/`, each with its commit on the first line.

| check | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 at `d1185ae85`'s tree |
| clippy | `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0 |
| lib + tables | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib --test closed_phases --test internal_routines --test owners --test phase_ten_reasons --test refusal_dispositions --test refusal_sites --test no_process_state` | exit 0: lib 1108, then 8, 2, 1, 6, 5, 3, 5 passed |
| refusal sites | `REXX_REFUSAL_SITES_REFRESH=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --test refusal_sites` | exit 0, one row changed: `native_argument_not_an_array` `send` to `body+send` |
| sourceline | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | exit 0 |
| crate-only | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib rexxutil::messages` | 12 passed |
| suite groups | `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/rexxutil/platform/unix/SysGetMessage,base/rexxutil/SysFormatMessage,base/keyword/ADDRESS REXX_SUITE_TABLE=<records>/table.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | exit 101 at `d1185ae85`, wall 82 s (`run-output.txt`): SysGetMessage green 9/9, SysFormatMessage green 5/5, ADDRESS red 78/79 on `TEST_COMMAND_CD` (crate failed, oracle passed) |
| corpus | `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | exit 0, `1038 of 1038 matching` |
| collect_stress | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test collect_stress` | exit 0, 37 passed |
| others | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test ir_recorded --test trace_oracle --test coverage` | exit 0: coverage 21, ir_recorded 28, trace_oracle 41 |

Not run, per the dispatch: the full suite and callgrind. The change adds four routine bodies and
touches no dispatch, allocation or shared per-call path.

### Before and after

RED: at Task 2's baseline the SysGetMessage group was 9 blocked plus 2 not started, SysFormatMessage
5 blocked, and ADDRESS's `TEST_COMMAND_CD` blocked on `SYSGETERRORTEXT` (`task-2-baseline/tests.tsv`
line 70). GREEN: as in the table.

### Witness

`rust/corpus/lang/rexxutil_messages.rex`, in `phase-9.txt`, sourceline file generated by the test's
driver from an empty directory (left empty). Run on both sides from fresh directories: stdout,
stderr and rc 216 identical (the oracle's are `witness.oracle.out`/`.err`). It makes `messages.d`,
`w.cat` and `100%` in its run directory and removes them. Its oracle stdout was read: every line
prints what its label claims, among them every message found in 1..999, the truncations, the size rule,
conversion order (`string of one`, `string of two`), the live item (`g grown`), first and last
raise, a gencat-layout catalog by absolute, relative and `./`, `../` paths and after `DIRECTORY`,
each search-path expansion answering from a distinct file (`locale`, `language`, `territory`,
`code set`, `C`, `current directory`, `percent`), the directory that stops the search, and
non-catalogs (short, bad magic, a directory, a cut table, an unterminated last message that opens,
a swapped header). It ends with one untrapped 40.1 so the rendering is compared.

Mutations, each built in release and the witness's stdout compared with the oracle's, reversed by
restoring a copy and confirmed with `cmp` and `git diff --stat`:

- array count by items instead of size: red, `('abc', , 'def')` loses `def` and `.array~new(10)`
  answers.
- first raise kept instead of last: red, `raised first`.
- a directory skipped in the search: red, `behind a directory` found.

Probes (`probes/`): `p1.rex` (argument forms, its oracle output `p1.oracle.out`), `p4.rex`/`p5.rex`
(conversion protocol), `p6.rex` over a real `gencat` catalog (`p6-catalog.sh`), all byte-identical
on both sides.

## Concerns

1. **`TEST_COMMAND_CD` is now red, and this task owns it but does not fix it.** It was blocked on
   `SysGetErrorText` and now runs. It fails at `"cd ~"`: the crate's `command.rs`
   `change_directory` is not `sys_process_cd` (`platform/unix/SystemCommands.cpp:494`). Measured
   (`probes/cd2.rex`, outputs `cd2.oracle.out`, `cd2.crate.out`), oracle then crate:

   | command | oracle rc | crate rc |
   |---|---|---|
   | `cd ~` | 0, to `HOME` | 2 |
   | `cd ~/ab` | 0 | 2 |
   | `cd "a"b` | 0, to `ab` (`unquote` drops every `"`) | 2 |
   | `cd a\ b` | 0, to `a b` (`\` escapes) | 2 |
   | `cd ~nosuchuser.x` | handed to `/bin/sh`, its message on stderr, rc 2 | 2, nothing on stderr |
   | `cd ab/x/..` with no `ab/x` | 2 (`chdir`) | 0, to `ab` (lexical normalisation) |

   This is the shared command path, so per "mechanical batches" I report it rather than widen the
   task. A fix ports `sys_process_cd` (blank skip, the `~`, `~/`, `~user`, `~user/` forms with the
   hand-back to the shell where `getpwnam` fails, `unquote`, and `chdir` of the unnormalised
   string); the last row is Deviation 34's shape and wants a ruling.
2. **`"Error: Invalid message catalog"` is never answered.** The C++ answers it where `catgets`
   returns its default and `errno` is `EBADF`. glibc's `catgets` sets `ENOMSG` or nothing, so that
   text depends on a stale `errno` from before the call. This crate answers "Message not found".
3. **Oracle crashes found, entries 41 to 43 in `oracle-crashes.txt`, each with a crate-only pin**:
   a raising value the message refers to (SIGSEGV), a catalog of no buckets (SIGFPE) or with more
   table than file (SIGBUS), and a `MAKEARRAY` answering a string (SIGSEGV 5 of 5 standalone, but
   98.913 once when trapped inside `INTERPRET` in `p4.rex`, so undefined). Entry 34 says a string
   answer is 98.913 on its path, which entry 43 qualifies for this one.
4. **Catalog arithmetic that overflows** (a header whose buckets times depth overflows `size_t`):
   glibc wraps and then reads wild memory. This crate treats overflow as a fault and answers
   "catalog not found". Not measured on the oracle.
5. **`__libc_enable_secure`** (a set-uid `rexx` ignoring a `LANG` with a `/`) is not modelled.
6. **The whole catalog file is read into memory** after its magic matches, where glibc maps it.

## Fix round 1

Commit `a516009f5`. Records in `docs/superpowers/records/2026-10-13-phase-9b/task-6/fix-1/`.

### What changed

- **`cd` ports `sys_process_cd`** (`command.rs` `change_directory`, now answering
  `Option<CommandOutcome>` so `None` hands the command to the shell):
  - leading spaces only are skipped, and trailing blanks reach the directory change;
  - a bare `cd` goes to `HOME`, and `~`, `~/rest` and `~ rest` go to `HOME` with a `/` after it,
    keeping the rest only after `~/`;
  - `~user` and `~user/rest` go through `platform::passwd_home` (now `pub(crate)`), and an unknown
    account goes to the shell;
  - an unset `HOME` goes to the shell, including `cd ~`, where the oracle dies (entry 44);
  - `unquote` drops every `"` and a backslash keeps the next byte, applied last to every branch
    (the false doc on the old `unquote` is gone);
  - no lexical normalisation: the target is joined to the interpreter's directory when relative,
    resolved with `canonicalize` (its errno is the rc), must be a directory (20) with search
    permission (`access(X_OK)`, 13), and the interpreter's directory becomes the physical path.
- **Other cwd state checked.** `change_directory` has one caller (`run_internally`). The only other
  `set_cwd` caller is `DIRECTORY(new)` (`platform.rs`), which already canonicalises, and the
  witness shows `DIRECTORY(link)` answering `<base>/real/sub` on both sides. The starting directory
  is `std::env::current_dir`, which is `getcwd`.
- **Entry 44** (`cd ~` with `HOME` unset): SIGSEGV 3 of 3 runs (`crash44.rex` under `env -u HOME`).
  Pinned by `command::tests::cd_home_with_home_unset_goes_to_the_shell` (`rc 2`, directory
  unchanged). That test lets the shell's message reach the test run's stderr.
- **Entries 34 and 43** rewritten. 43 carries the review's per-string table, and 34 points to it.
  The short strings (`''`, `'x'`, `'abc'`) through both programs are now in
  `lang/rexxutil_messages.rex` (98.913 on both sides, the oracle 3 runs of 3 byte-identical).
- `messages.rs` `open` doc names the interpreter's current directory.
- `unquote` has a unit test of its byte rules.

### Witness

`rust/corpus/lang/address_cd.rex`, in `phase-9.txt`, sourceline file generated from an empty
directory. Run from fresh directories on both sides: stdout, stderr and rc 0 identical
(`address-cd.oracle.out`/`.err`), and the run directory is left empty. The oracle's stdout was read
and each row prints what its label claims:
- the home forms, including `cd ~ real` landing in `HOME`;
- `~<me>` and `~<me>/` landing in the account's home;
- `~nosuchuser.cd` and its `/x` form going to `/bin/sh` (its message on stderr, rc 2);
- `"~"`, quotes and escapes, a trailing backslash, a trailing blank, and `cd ""` (rc 2);
- `link`, `link/..`, `link/../sub` and `link/../../ab` resolved physically, `nowhere/..` (2);
- a file (20), `file/..` (20), and a directory without search permission (13);
- absolute paths, and `DIRECTORY(link)`;
- `HOME` set to the null string: `cd ~` lands in `/`, and a bare `cd` is rc 2.

Mutation: resolving without `canonicalize` (the joined path kept as is) turned the witness red at
the first home row. Reversed from a copy, confirmed with `cmp` and `git diff --stat`, and the
release binary rebuilt.

### Checks

| check | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 |
| clippy | `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0 |
| refusal sites | `REXX_REFUSAL_SITES_REFRESH=1 ... --test refusal_sites` | one row moved location only (`escalated`, `command.rs:409` to `:463`) |
| lib + tables | same command as above | exit 0: 1110, 8, 2, 1, 6, 5, 3, 5 passed |
| sourceline | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | exit 0 |
| suite groups | the three groups, as above | **exit 0** at `a516009f5`, wall 77 s: ADDRESS 79/79, SysFormatMessage 5/5, SysGetMessage 9/9, all green |
| corpus | `REXX_CORPUS_GATE=1 ... --test corpus` | exit 0, `1039 of 1039 matching` |
| collect_stress | `... --test collect_stress` | exit 0, 37 passed |

## Fix round 2

- `command::tests::cd_home_with_home_unset_goes_to_the_shell` no longer asserts the shell's rc,
  which depends on what `/bin/sh` is. It asserts that the program ends normally (exit 0) and that the directory is
  unchanged (`stayed 1`). Entry 44 now says the shell's answer is the command's.
- Headings of entries 34 and 43 match their tables: 34 "a List, or a string of some contents"
  (its program answers rc 0 for `'not an array'` and crashes on longer strings), 43 "a string other
  than a short one".
- Checks: `cargo fmt --all --check` exit 0; clippy `-p rexx-exec --all-targets -D warnings` exit 0;
  `cargo test -p rexx-exec --lib command::tests` 6 passed; `--test refusal_sites` 5 passed.
