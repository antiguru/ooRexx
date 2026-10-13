# Phase 9b Task 6 review: the RexxUtil message routines

Reviewed `b1b3b8425..1bb5e9dc5` (diff `review-b1b3b8425..1bb5e9dc5.diff`). Probes ran from
fresh empty directories under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/t6rev/`.
The oracle ran under the standard wrapper and the crate under `memcap 4G`. The crate binary was
`rust/target/release/rexx-run`, and `cargo build --release --bin rexx-run` reported it fresh at
HEAD.

### Spec Compliance

- ❌ One item. `base/keyword/ADDRESS` is one of the brief's Step 3 groups, and it is red on
  `TEST_COMMAND_CD` (`task-6/table.tsv`: crate 78/79, oracle 79/79). The implementer reported this
  rather than fixing it, which the "mechanical batches" rule allows because `command.rs` is a shared
  path. The controller has to send the fix somewhere before 9b closes (see Q1).
- ✅ The four routines run (`internal_routines.rs`, the `util_run` rows). `TEST_CATALOG_INVALID` and
  `TEST_GENCAT` keep `Phase10(Rxapi)` (`task-6/tests.tsv:89-90`,
  `phase-9-exclusions.txt:339-340`). SysGetMessage is 9/9 and SysFormatMessage is 5/5.
- ⚠️ Not verifiable from the diff: whether ADDRESS's red row needs a whole-group or sim-gate row.
  `sim-gate.tsv` and `concurrency_tests` mention none of the three groups (grep), so probably not.

### Strengths

- The catalog port is faithful where it matters. A differential fuzz against glibc gave **0 differences
  over 327 comparable files and no crate panic** (details under Q2). A strace comparison of the
  candidate paths the search opens matched in **143 of 143** LANG×NLSPATH combinations.
- The `SysGetErrorText` output is byte-identical to the oracle's over -3000..3000 and 11 extreme
  or large values (6012 lines, `cmp` equal).
- The witness is strong. Both mutations I added went red (Q6), on top of the implementer's three.
- Entries 41 and 42 are re-measured and correct.

### Answers to the controller's questions

**Q1. The `cd` rule** (`interpreter/platform/unix/SystemCommands.cpp:494-625`, entered from
`:760` only when the command is exactly `cd` or begins with `cd `, and no `<>|&;` appears outside
quotes):

1. `st` = the command after `cd`, with leading **spaces only** skipped (not tabs). Trailing blanks
   are kept and reach `chdir`.
2. If `st` is empty, the target is `$HOME`. With `HOME` unset the command is **handed to the
   shell** (`return false`).
3. If `st` is `~` followed by NUL, `/` or a space:
   - `~/rest` becomes `$HOME/rest`. With `HOME` unset it goes to the shell.
   - `~` and `~ anything` both become `$HOME/`, and **the rest after the space is ignored**. With
     `HOME` unset the oracle runs `strlen(NULL)` and gets **SIGSEGV, rc 139, 3 of 3 runs**
     (`env -u HOME`, program `'cd ~'`). That is a new oracle crash and is not in
     `oracle-crashes.txt`.
4. If `st` is `~user`, with no `/`, the target is `getpwnam(user)->pw_dir` plus `/`. The user name
   is everything after `~`, quotes and blanks included.
5. If `st` is `~user/rest`, the target is `pw_dir/rest`. The user name is copied into a 256-byte
   stack buffer without a length check (an overflow; not measured). If `getpwnam` fails in either
   form, the command is **handed to the shell**, so `/bin/sh` prints its message on stderr and
   rc is 2.
6. Anything else is used as written.
7. **Unquoting** (`unquote`, `:453`) is applied to the result of every branch above, so it also
   runs over `$HOME`. It **drops every `"`**, and a `\` makes the next byte literal. The `\` itself
   is dropped and the escaped byte is kept, including a `"`. A trailing `\` copies the NUL, so the
   string simply ends there. The `~` test comes before unquoting, so `cd "~"` is a literal `~`.
8. **No normalisation.** `chdir(unquoted)` receives the string as is. `..` is therefore resolved
   physically by the kernel, and the new directory is what `getcwd` reports, with symlinks
   resolved. `chdir("")` gives ENOENT. On failure the oracle raises ERROR with `rc` set to `errno`
   (nothing on stderr). On success `rc` is 0.

The crate's `change_directory` (`rust/crates/rexx-exec/src/command.rs:297`) does none of steps 3-5,
removes only a *wrapping* pair of quotes (the doc on `unquote` at `command.rs:282` is false), and
normalises lexically through `paths::normalize` before a `metadata` check. **The lexical
normalisation is a real bug and not Deviation 34.** Deviation 34 is `normalizePathName` on
qualified names. `sys_process_cd` never calls it. Symlink probe (`src/cdsym.rex`: `real/sub`,
`link -> real/sub`), oracle then crate:

| command | oracle | crate |
|---|---|---|
| `cd link` | 0, `DIRECTORY()` = `<dir>/real/sub` | 0, `<dir>/link` |
| `cd link/..` | 0, `<dir>/real` | 0, `<dir>` (a different directory) |
| `cd link/../sub` | 0, `<dir>/real/sub` | **2**, unmoved |
| `cd ""` | 2 | **0** (moves to cwd) |
| `cd ~ foo` | 0, `$HOME` | 2 |

The report's claim that "the last row is Deviation 34's shape and wants a ruling" is wrong. No
ruling is needed. The crate diverges from a `chdir` that the oracle performs exactly as written.

**Recommended fix** (one function in `command.rs`, plus a witness):

- Port steps 1-7 byte for byte. `None` from `run_internally` already means "hand to the shell", so
  use it for the `getpwnam` failures and for an unset `HOME` in steps 2 and 3(`~/`).
- For the remaining unset-`HOME` shape, `cd ~` / `cd ~ x`, the oracle crashes. Record it as an
  oracle-crashes entry and pin the crate's answer with a crate-only test. Handing it to the shell is
  the natural choice.
- Reuse `platform::passwd_home` for the account lookup. Do **not** reuse `qualify_name` or
  `expand_tilde` as they stand: they normalise, they map `~` to `$HOME` without the trailing `/`,
  they treat `~ foo` as user ` foo`, and they cannot express "hand to the shell".
- Then resolve without lexical reduction. Join the target to `interp.cwd_bytes()` unless it is
  absolute, and treat an empty target as ENOENT. Then `std::fs::canonicalize` the joined path,
  which gives physical `..` and ENOENT/ENOTDIR/ELOOP with their real errno. Require a directory
  (ENOTDIR = 20), check search permission (`rustix::fs::access(X_OK)`, EACCES = 13, which `chdir`
  checks and `canonicalize` does not), and `set_cwd` the canonical path.
- Witness: the symlink rows above, plus `cd ""`, `cd ~ foo`, `cd "a"b`, `cd a\ b`, `cd ~/x`, and
  `cd ~nosuchuser` (stderr compared).

**Q2. Catalog reader.**

- **Fidelity.** I checked `catopen`/`catgets` line by line against the fetched glibc source
  (`scratchpad/t6-glibc/`). The `size_t` `plane_size` in `catalog_info` makes the crate's
  sign-extended hash (`messages.rs:563`) exact.
- The oracle calls `catopen(repository, 0)` (`SysRexxUtil.cpp:1127`), so it is `LANG`, not
  `NL_CAT_LOCALE`/`LC_MESSAGES`, and the crate is right. The default `NLSPATH` matches the string in
  `libc.so.6` (`strings`).
- **Search-path differential.** I ran a C harness (`catopen(path,0)`+`catgets`) and the crate under
  `strace -e openat`, filtered to the candidates. Inputs: LANG ∈ {unset, "", en, en_, en., en_US,
  en_US.UTF-8, en_US.UTF-8@euro, _x, .x, x_y_z, a.b.c, a/b} × NLSPATH ∈ {unset, "",
  `QQ/%l/%t/%c/%N`, `QQ/%L:%N`, `:QQ/%N`, `QQ/%Z/%N:QQ2/%N`, `QQ/%`, `QQ/%:QQ3/%N`,
  `QQ/%%%N`, `QQ/%t%c`, `::QQ/%N::`}. Result: identical open sequences in 143 of 143 cases
  (`np.out`). I read one sample's strace to confirm the comparison was not empty: five candidates,
  `%l/%t/%c` expanded.
- **Bounds.** I generated 357 files from a real `gencat` catalog (`src/gen.py`): every third
  truncation length, 35 header (plane_size, plane_depth) pairs including 0 and 0xffffffff, 150
  random byte-flip files, 80 random bodies with a valid magic, and one byte-swapped magic. Each was
  queried with 10 (set, msg) pairs on both sides.
  - glibc crashed on 30 files: 25 SIGBUS and 5 SIGFPE, the latter on plane_size 0. Note that the
    byte-swapped-magic file is one of the crashers.
  - On the other **327 files the crate's answers equal glibc's, with no difference**.
  - The crate exited 0 with empty stderr over all 357, so no panic.
  - On the crash files the crate answers "Message not found" (plane 0) or "Message catalog not
    found" (the rest).
  - Reading `Catalog::new`: `remaining` cannot underflow (`size > claimed` implies `remaining >= 1`),
    and every read goes through `CatalogImage::byte`, which is checked.

**Q3. Entries 41-43.**

- 41 is accurate: crash41.rex gives SIGSEGV 3/3 and the crate raises 93.900.
- 42 is accurate: crash42.rex gives SIGFPE 3/3 and crash42b.rex SIGBUS 3/3. The crate's answers
  match the text.
- **Entries 43 and 34 are both over-general.** The outcome is deterministic **per string content**
  and identical on both paths. I measured 3 runs each of `.environment~unknown('ARRAY', .k~new)`
  (entry 34's program) and `SysFormatMessage('&1', .k~new)` with `MAKEARRAY` returning:

  | MAKEARRAY returns | entry 34's path | SysFormatMessage |
  |---|---|---|
  | `''`, `'x'`, `'abc'` | 98.913, rc 158 | 98.913, rc 158 |
  | `'not an array'` | **rc 0, prints `The Array class`** | SIGSEGV |
  | `copies('a',40)`, `copies('b',300)`, `1`, `12345` | SIGSEGV | SIGSEGV |

  - Entry 34's "a string answer raises 98.913" holds only for short strings. With entry 43's own
    string, entry 34's program answers rc 0.
  - Entry 43's SIGSEGV is right for its string. "So the answer depends on what the string's memory
    holds" is right in substance, but it should say the answer is deterministic per string and
    give the table.
  - Fix both entries' prose. The crate's 98.913 for every string is defensible, since it is the
    oracle's only non-crash error answer. The short-string rows agree 3/3 on both sides and could be
    witnessed in the corpus instead of pinned crate-only.

**Q4. strerror.**

- The suffix strip is exact. `std::io::Error`'s `Display` for an OS error is
  `"{strerror} (os error {n})"`, and the strip removes exactly that suffix (`messages.rs:101-109`).
- The values outside -3..140 also match: -3000..3000 plus ±2147483647, -2147483648, 65535, 65536,
  99999, -99999, 1000000, ±123456789 and 2147483646 are byte-identical on both sides (6012 lines,
  5879 of them `Unknown error N`).
- Neither side calls `setlocale` (grep over the oracle tree), so the text is untranslated on both.

**Q5. "Invalid message catalog".** The choice is defensible. glibc's `catgets` sets `errno` only to
ENOMSG, when the message is not found, so the oracle's EBADF test reads a stale `errno`. That
happens only for a found *empty* message or for `++set <= 0`, and nothing before it in the
successful `catopen` sets EBADF. The crate's "Message not found" is the oracle's answer under any
non-EBADF stale value. The report records this in concern 2 and in the doc at `messages.rs:278-281`.

**Q6. Mutations.** Built from a `git archive` copy of the tree with its own target directory,
rebuilt, and compared with `witness.oracle.out`. The control (unmutated HEAD binary) was
byte-identical to it.

- M1: `Some(text) if !text.is_empty()` changed to `Some(text)` (`messages.rs:287`). **Red** at line
  916: `SysGetMessage(3, cat) -> []`.
- M2: the `%c` arm disabled (`b'c' if false`, `messages.rs:391`). **Red** at line 939: the `code
  set` lookup becomes "catalog not found".
- The copy was deleted afterwards.

### Issues

#### Critical (Must Fix)

None in this diff.

#### Important (Should Fix)

1. `rust/crates/rexx-exec/src/command.rs:297` (`change_directory`, unblocked by this task):
   - It normalises lexically before resolving, so `cd link/..` lands in a different directory from
     the oracle and `cd link/../sub` fails where the oracle succeeds.
   - It lacks the `~` forms and the C `unquote`, and `cd ""` succeeds.
   - Port as described under Q1. The report's "Deviation 34 shape, wants a ruling" is wrong.
2. `rust/corpus/oracle-crashes.txt` entries 34 and 43 contradict each other, and both are
   over-general. Replace them with the per-string table from Q3.
3. A new oracle crash, `'cd ~'` with `HOME` unset (SIGSEGV 3/3), is not in `oracle-crashes.txt`.
   Whoever ports `sys_process_cd` should record it.

#### Minor (Nice to Have)

1. `command.rs:282`: the `unquote` doc says "a wrapping pair of double quotes is removed and
   everything else is kept". That is false against `SystemCommands.cpp:453`. Delete it with the
   port.
2. `messages.rs:317`: the `open` doc and the report say "relative to the program's directory".
   The code uses the interpreter's current directory (`cwd_bytes`), which can be misread as the
   script's directory.
3. `messages.rs:448-450`: the whole file is read once the magic matches, so a large regular file
   with a valid magic costs its full size in memory, where glibc maps it. This is concern 6 in the
   report. It would be bounded by reading only the bytes `Catalog::new`/`message` touch.
4. The short-string `MAKEARRAY` rows (Q3) could move from crate-only pins into the witness.

### Assessment

**Task quality:** Approved. The message routines are correct, faithful and well-witnessed.
`TEST_COMMAND_CD` needs a routed fix before 9b closes.

**Reasoning:** The catalog port matched glibc on every non-crashing fuzz input and every
search-path combination, and `strerror` matched across 6012 values. The only red item is the `cd`
port in `command.rs`, which this task reported rather than widened, and the evidence shows it is a
genuine crate bug, not an oracle bug.
