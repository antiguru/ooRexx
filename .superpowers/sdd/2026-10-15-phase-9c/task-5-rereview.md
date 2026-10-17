# Phase 9c Task 5 re-review: fix round 2

Reviewer: c9c-rereview-5. Range `git diff dc16e526e 271a082ef`. Release `rexx` built at `271a082ef`;
oracle runs from a fresh fixture, stdout, stderr and rc compared separately (probe scripts were in
the scratchpad's `9c-rr5/tools/`, not durable).

## Verdict

Approved. Critical 1 and Important 1 are fixed and agree with the oracle on all three descriptors in
every shape run. No open Critical or Important finding. Two minors.

## Checks run

1. **Critical 1, 17 shapes, all identical to the oracle.** The review's: `dé/req.rex`, `dé/ext.rex`,
   `d'ff'x/req.rex`, `d'ff'x/ext.rex`, cwd `d€` with `ext.rex` and `req.rex`, `REXX_PATH=rpé` and
   `rp'ff'x` with `call farcall`. Mine: `REXX_PATH` with both a `rpé` and a `rp'ff'x` entry;
   `REXX_PATH=rpé` from cwd `d€` running `../farmain.rex`; PATH-only resolution of `call pathonly`
   through a `rpé` entry, a `rp'ff'x` entry, and a `rpé` entry from cwd `d€`; a relative `REXX_PATH`
   entry (both rc 213); `::requires "r'ff'xq.rex"` from an ASCII, a `dé` and a `d'ff'x` directory
   (the file exists, both run it); `call (n)` with `n = 'f'||'ff'x||'x'` found through a `rp'ff'x`
   `REXX_PATH` entry (both rc 0) and not found without it (both 213 identically); `call (n)` with a
   `'ff'x` name found in the cwd.
   - No panic. `search` has no `unwrap` or `expect`. `narrow` uses `unwrap_or(b'?')`. The one
     remaining `expect` in the area is `paths::normalize` (`paths.rs:27`), the `&str` wrapper, which
     `search` does not call (it uses `normalize_bytes`).
2. **Side effect confirmed.** On the oracle a `::REQUIRES` or `CALL` name containing `'ff'x` is
   searched and its file run (`in r'ff'xq` printed from cwd, rc 0 for `call (n)`), and not-found
   when absent (rc 213, 43.1). The crate now matches both.
3. **Important 1.** `link.rex -> sub/t.rex`, `lsub/t.rex` with `lsub -> sub`, extensionless `link`
   (found through `.rex`), an extensionless link to an extensionless target, `./link.rex`, the
   absolute `DIR/link.rex` and `DIR/lsub/t.rex`, and a link into a `dé` directory: `PARSE SOURCE`
   and the `Error 42 running` line match the oracle, rc 214 or 0.
4. **Test count.** The tree at `271a082ef` has 1142 lib tests, not the 1141 the report states
   (`cargo test -p rexx-exec --lib`: `1142 passed`). From 1143 at `dc16e526e` exactly one `#[test]`
   is gone: `launcher::widening_a_name_round_trips_every_byte`, deleted because `widen` and `narrow`
   became private to `require.rs`. `git grep -c '#\[test\]'` over `rust/crates` is 2770 then 2769.
   The deletion is justified (no test of `shadow_var` existed). Its property now rides on the `'ff'x`
   directory, `REXX_PATH` and name cases against the oracle.
5. **Minors, comments.** The `interpret` sentence and the interleave sentence are gone; `syscargs`
   and `syscargs_base` are named fields. Added lines contain no em-dash and no set sizes.
6. **Mutation.** In a detached worktree copy, `search` stat-ing `String::from_utf8_lossy(candidate)`
   instead of the bytes: `rexx_command` goes red on `d\xff/req.rex`, `d\xff/ext.rex` and
   `farmain.rex` (the `rp_ff` case). The `é` cases stay green, as expected, because a valid UTF-8
   candidate survives a lossy round trip. Worktree and target dir removed.
   - Method note: my own first HEAD run after the mutation was red because the shared target dir
     handed the mutated test binary back; after `touch` and a `Compiling` line it is `1 passed`.

## Findings

### Minor

1. **`require.rs`, doc on `search`**: the paragraph "`to_ascii_lowercase` leaves the chars above 127
   alone, which is `strlower`'s rule" is a leftover of `widen`'s old doc, now under `search`, where
   it reads as a non sequitur after "carried through the `str` functions above". Move it to `widen`
   or reword.
2. **`task-5-report.md`, Fix round 2 table**: `--lib` says 1141 passed; the tree gives 1142. The
   one deleted test (check 4) is also not mentioned.
