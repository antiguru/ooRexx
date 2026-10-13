# Phase 9b Task 6 re-review, fix round 1

Fix `1bb5e9dc5..1888b1d81`. Probes ran from fresh empty directories under `scratchpad/t6rr/`, oracle and
crate both under `memcap 4G`. The release `rexx-run` was fresh at HEAD (`cargo build` finished in 0.02 s).

## Verdict: approved, with two Minor findings

### Ruled items

- **Port of `sys_process_cd`: resolved.** Checked rule by rule against `SystemCommands.cpp:494-625`:
  - Leading spaces only skipped (`after_keyword`). A blank-only target is the home directory.
  - `~`, `~/rest` and `~ rest` become `HOME` plus `/`, the rest kept only after `~/`.
  - `~user` and `~user/rest` go through `passwd_home` plus `/`. `None` hands the command to the shell, for
    an unknown account and for an unset `HOME` on a bare `cd` and on `~/x`.
  - `unquote` is applied last to every branch, drops every `"`, and a `\` keeps the next byte. A trailing
    `\` ends the text. The NUL cut matches C string semantics.
  - `HOME` set to the null string: `cd ~` lands in `/`, and a bare `cd` is ENOENT (witness rows).
- **Physical resolution: resolved.** `lang/address_cd.rex` run on the oracle and the crate: stdout (34
  lines) and stderr (the two `/bin/sh` messages) byte-identical, rc 0, and the run directory left empty.
  `link`, `link/..`, `link/../sub`, `link/../../ab`, `cd ""` (2), file (20), `file/..` (20) and a
  directory without search permission (13) all agree.
- **Other cwd state: consistent.** `change_directory` has one caller (`run_internally`). The only other
  `set_cwd` caller is `DIRECTORY(new)` (`platform.rs:198`). Extra probe not in the witness: after
  `address system 'cd link'` (link to `real/sub`), `DIRECTORY()`, a relative `lineout`,
  `stream(..,'C','QUERY EXISTS')`, `SysFileTree`, a child `pwd -P; ls`, then `cd ..`, then
  `cd ../.. ; pwd` through the shell. Output was identical between oracle and crate on all of it, and the
  file landed in the physical directory on both sides.
- **Entries 34 and 43: accurate.** The table matches my review's measurements. The short-string rows are
  now in `lang/rexxutil_messages.rex` (98.913 on both sides). I ran that file on both sides: stdout and
  stderr identical, rc 216 on both. One imprecision: both headings say "a string that is not short", but
  entry 34's program answers rc 0 for `'not an array'`, not a crash. The table is right and the heading
  over-claims by one row. Trivial.
- **Entry 44: accurate as to the oracle** (SIGSEGV 3/3 was measured in the original review). Its prose is
  shell-specific, see M1.
- **Minors from the review:** the false `unquote` doc is gone and `unquote` has a unit test. The `open` doc
  names the interpreter's current directory.

### The entry-44 crate-only test

Question: with `HOME` unset, what does the crate do, and is that defensible?

- The crate hands `cd ~` to the shell. Under dash (`/bin/sh` here) that prints `/bin/sh: 1: cd: can't cd
  to ~` and gives rc 2. The message does not reach the test run's stderr (`--nocapture` shows nothing), so
  the test is quiet. It is not noisy, and not flaky on this machine (3 of 3 runs, 0.02 s). `unset HOME`
  inside the program changes the interpreter's own environment table (`env_get` reads `self.env`), so the
  test cannot disturb other tests.
- **Handing it to the shell is defensible.** It is what the oracle itself does for every other
  `HOME`-unset shape (bare `cd`, `cd ~/x`), and the oracle's only alternative is a segfault. Using
  `getpwuid` would be a bigger deviation: the oracle never consults the password file for `~` alone, and
  the shell already may.
- **But the answer is not fixed: it depends on `/bin/sh`.** `env -u HOME bash --posix -c 'cd ~'` gives
  rc 0 here, because bash expands `~` through `getpwuid`. dash gives rc 2. Bare `cd` with `HOME` unset
  gives rc 0 under dash (measured). So the crate's `cd ~` rc is the shell's, not the crate's.

## Findings

### Minor

- **M1. The entry-44 test and prose are dash-specific.**
  `command::tests::cd_home_with_home_unset_goes_to_the_shell` asserts `rc == 2`
  (`command.rs`, the new test). On a platform whose `/bin/sh` is bash (macOS, and Linux distributions that
  link `sh` to bash) the shell expands `~` from the password file and exits 0, so it would go red there.
  CI runs five platforms. The directory-unchanged half is portable, since the child cannot move the
  interpreter. Assert that half and the absence of a crash, and drop `rc == 2` from the test. Also rewrite
  entry 44's "which answers `rc 2` with its own message on stderr" as "the shell's answer" (dash: rc 2).
- **M2. `passwd_home` reads `/etc/passwd` only, where `getpwnam` also consults NSS.**
  This predates the fix (`platform.rs:84`) but `~user` now relies on it. An account that exists only in
  NSS (LDAP, sssd) is handed to the shell, which would resolve it, so the user-visible result is usually
  the same. Not worth action in this task. Worth one line in a later platform pass.

### Out-of-scope observations

- The `~user` user-name buffer overflow noted in the review (`username[256]`) is not reproduced by the
  crate and is not recorded in `oracle-crashes.txt`. It was not measured, so nothing to record yet.

## Fix-diff review

I read the whole `command.rs` change. No new defect found:

- `canonicalize` plus `is_dir` plus `access(X_OK)` reproduces `chdir`'s errno for the rows tested (ENOENT,
  ENOTDIR 20, EACCES 13). ELOOP and ENAMETOOLONG pass through `raw_os_error`.
- The `rest.get(1)` match is `None | '/' | ' '`, which is the C test. `cd ~` followed by a tab is a user
  named `\t` in both.
- `home()` is a closure over `interp` borrowed immutably and dropped before `set_cwd`. It compiles and
  there is no aliasing problem.
- The refusal-sites row moved by location only, as the report says.
