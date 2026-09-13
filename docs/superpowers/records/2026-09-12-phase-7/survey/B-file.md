# Survey B: `.File` class and the file-system layer

Area letter: B. Probe root: `survey-B/pNN` under the shared scratchpad. Oracle checked out at
`cb9563364` (2026-07-27), binary swapped 2026-08-20 (`rust/CLAUDE.md:11`). Crate snapshot binary
`h-5bcb28edb` at branch HEAD `5bcb28edb`.

## 1. Documented surface

Source: `oodocs/rexxref/en-US/utilityclasses.xml`, section `clsFile` (`:3382`), enumerated from its
own class table (`:3400-3475`) and its member sections in document order. `File` is `public inherit
Comparable Orderable` (`interpreter/RexxClasses/StreamClasses.orx:506`).

### Class methods

| item | doc citation | one-line contract |
|---|---|---|
| `isCaseSensitive` | `utilityclasses.xml:3488` | case-sensitivity of `/` (unconditionally `true` on this platform's build, see §3) |
| `listRoots` | `utilityclasses.xml:3509` | array of root strings; `["/"]` on Unix |
| `pathSeparator` | `utilityclasses.xml:3536` | `":"` on Unix |
| `readChars` (5.2) | `utilityclasses.xml:3556` | reads a whole file's content via a `Stream`, returns a string |
| `readLines` (5.2) | `utilityclasses.xml:3579` | reads a whole file via a `Stream`, returns an array of lines |
| `searchPath` (5.0) | `utilityclasses.xml:3604` | finds `name` along `path` (default `PATH` env var), returns a `File` or `.nil` |
| `separator` | `utilityclasses.xml:3640` | `"/"` on Unix; also an instance method |
| `temporaryPath` (5.0) | `utilityclasses.xml:3669` | `TMPDIR` env var or `/tmp`, as a new `File` |
| `writeChars` (5.2) | `utilityclasses.xml:3701` | appends (default) a string to a file via a `Stream` |
| `writeLines` (5.2) | `utilityclasses.xml:3725` | appends (default) an ordered collection's lines to a file via a `Stream` |

### Instance methods (32) and the two mixins

| item | doc citation | one-line contract |
|---|---|---|
| `absoluteFile` | `:3753` | fully-qualified path as a new `File` |
| `absolutePath` | `:3787` | fully-qualified path as a string |
| `canRead` | `:3821` | exists and is readable |
| `canWrite` | `:3843` | exists and is writable |
| `compareTo` | `:3865` | `Comparable`: sorts on absolute path, caseless iff the filesystem is caseless |
| `delete` | `:3896` | deletes a file, or an *empty* directory; boolean |
| `exists` | `:3916` | boolean |
| `extension` (5.0) | `:3936` | text after the last dot in the name; `""` if the only dot is the first character |
| `hashCode` | `:3963` | hash for `MapCollection` keying |
| `init` | `:3981` | `path` (required), optional `dir` (string or `File`, prepended) |
| `isCaseSensitive` | `:4026` | case-sensitivity of this path (or its nearest existing ancestor) |
| `isDirectory` | `:4048` | boolean |
| `isFile` | `:4066` | boolean |
| `isHidden` | `:4084` | Unix: name or an ancestor directory starts with `.` |
| `lastAccessed` (5.0, attribute) | `:4119` | GET: `DateTime` or `.nil`; SET: no-op if the file is missing, argument must be a `DateTime` |
| `lastModified` (attribute) | `:4166` | same shape as `lastAccessed` |
| `length` | `:4231` | size in bytes |
| `list` | `:4249` | array of child names (order is filesystem-defined), `.nil` if not a directory, no `.`/`..` |
| `listFiles` | `:4286` | array of child `File`s, same shape as `list` |
| `makeDir` | `:4341` | leaf directory only, parents must already exist; boolean, `.false` if it exists |
| `makeDirs` | `:4363` | full hierarchy; boolean, `.false` if it exists |
| `makeString` (5.0) | `:4386` | alias of `absolutePath` |
| `name` | `:4401` | text after the last separator |
| `parent` | `:4434` | parent directory as a string, `.nil` at a root |
| `parentFile` | `:4466` | parent directory as a `File`, `.nil` at a root |
| `path` | `:4486` | the normalized constructor argument, unqualified |
| `pathSeparator` | `:4505` | instance form of the class method |
| `renameTo` | `:4525` | `dest` must be a `File`; `rename(2)` on Unix; boolean |
| `separator` | `:4549` | instance form of the class method |
| `setReadOnly` | `:4569` | clears write permission; **no documented return value** |
| `setWritable` (5.0) | `:4594` | sets write permission; **no documented return value** |
| `string` | `:4613` | the raw constructor path (same value `path` answers) |

`Orderable` and `Comparable` mixin methods (`=`, `==`, `<`, `<=`, `>`, `>=`, `<>`, `\=`, `\==`, …)
are inherited, not overridden except `compareTo`; not separately tabulated here — they resolve
through `compareTo`.

**Not in the documented table, but real and load-bearing:** two `PRIVATE` natives,
`deleteFile` (`file_delete_file`) and `deleteDir` (`file_delete_directory`)
(`StreamClasses.orx:686-687`), which `delete` dispatches to by `isDirectory`. The brief's
`deleteDirectory` name does not exist anywhere in the tree — the private method is `deleteDir`
(Measured/Read: `grep -n deleteFile\\|deleteDirectory` over both `utilityclasses.xml` and
`StreamClasses.orx` finds only `deleteDir`, `deleteFile`, and `file_delete_directory` as the
*entry-point* spelling, not a method name).

## 2. Oracle behaviour

All measured from a fresh directory, `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout 20
.../build/bin/rexx FILE )`, all three descriptors read separately (none of these probes produced
stderr or a non-zero exit except where shown).

**Class methods**, `survey-B/p01/t.rex`:
```
say .File~separator        -- /
say .File~pathSeparator    -- :
say .File~isCaseSensitive  -- 1
say .File~listRoots~toString(, " ")   -- /
say .File~temporaryPath    -- /tmp
say .File~searchPath("ls") -- /usr/bin/ls
```
rc 0, stderr empty. `isCaseSensitive` answering `1` says this build's `ext4` (or the container
filesystem) is not running with `FS_CASEFOLD_FL`-style per-directory case folding, or that support
was not compiled in (see §3).

**Construction, path normalization, existence/attribute reads**, `survey-B/p03/t.rex` (directory
holds `sub/afile.txt` = 12 bytes "hello world\n", and `sub/dangling.lnk` a symlink to a
non-existent target):
```
.File~new('.')~path            -> .
.File~new('.')~absolutePath    -> <probe dir>                      (no trailing separator)
.File~new('sub/afile.txt')~name       -> afile.txt
.File~new('sub/afile.txt')~extension  -> txt
.File~new('.hidden')~extension        -> ""            (dot is the first character)
.File~new('sub/afile.txt')~parent      -> <probe dir>/sub
.File~new('sub/afile.txt')~parentFile~path -> <probe dir>/sub
.File~new('sub/afile.txt')~isDirectory -> 0
.File~new('sub/afile.txt')~isFile      -> 1
.File~new('sub/afile.txt')~isHidden    -> 0
.File~new('sub/.dotfile')~isHidden     -> 0   -- the file does NOT exist; isHidden requires exists() first, see §3
.File~new('sub/afile.txt')~canRead     -> 1
.File~new('sub/afile.txt')~canWrite    -> 1
.File~new('sub/afile.txt')~length      -> 12
.File~new('sub/afile.txt')~string      -> sub/afile.txt      -- the RAW ctor argument, unqualified
.File~new('sub/afile.txt')~makeString  -> <probe dir>/sub/afile.txt
.File~new('sub/afile.txt')~lastModified~class -> The DateTime class
.File~new('sub/doesnotexist')~lastModified -> The NIL object
.File~new('sub/doesnotexist')~length       -> 0                -- NOT nil, NOT an error
.File~new('sub/dangling.lnk')~exists   -> 0
.File~new('sub/dangling.lnk')~isFile   -> 0
.File~new('sub/dangling.lnk')~canRead  -> 0
.File~new('sub/')~path                 -> sub                  -- trailing separator stripped
.File~new('sub/../sub/afile.txt')~absolutePath -> <probe dir>/sub/afile.txt   -- lexically collapsed
.File~new('afile.txt', fileObjectForSub)~absolutePath -> <probe dir>/sub/afile.txt
.File~new('afile.txt', 'sub')~absolutePath            -> <probe dir>/sub/afile.txt
.File~new('/')~name    -> ""            (empty string, matches doc)
.File~new('/')~parent  -> The NIL object
.File~new('')~path     -> ""
.File~new('')~exists   -> 0
.File~new(A)~compareTo(.File~new(A)) -> 0
```
rc 0, stderr empty for the whole batch. Note: this probe directory pre-existed from the earlier,
session-limited survey attempt and its `list` output (also captured) is contaminated by that
attempt's leftover files (`fx.c`, `prog.rex`, …) — not a filesystem-listing defect, a probe-hygiene
one; not relied on for any claim above other than "list runs and returns an array."

**Mutating operations**, `survey-B/p04/t.rex` and `t2.rex` (rc reported per program; a later `say`
of a call with no return value is itself the interesting case, see below):
```
.File~new('newdir')~makeDir        -> 1              (first call)
.File~new('newdir')~makeDir        -> 0              (already exists)
.File~new('a/b/c')~makeDirs        -> 1              (creates a, a/b, a/b/c)
.File~new('a/b/c')~makeDirs        -> 0              (already exists)
say ro~setReadOnly   -- Error 91.999: Message "SETREADONLY" did not return a result. rc 165
```
**`setReadOnly` and `setWritable` answer nothing** (no `RETURN` in
`StreamClasses.orx:660-667`) — using either in an expression context is Error 91.999 on the oracle.
The chmod side effect still runs before the error is raised (measured: the file was read-only on a
later probe run against the same physical file). Re-run without `say`:
```
ro~canWrite            -> 1  (before)
ro~setReadOnly          (no result; side effect applied)
ro~canWrite            -> 0
ro~canRead             -> 1
ro~setWritable           (no result)
ro~canWrite            -> 1  (restored)

.File~new('newdir')~delete          -> 1   (empty dir)
.File~new('a')~delete               -> 0   (non-empty dir, refused, no error)
.File~new('ro.txt')~delete          -> 1   (file)
.File~new('doesnotexist')~delete    -> 0

.File~new('r1.txt')~renameTo(.File~new('r1.txt')) -> 0   (same path -> EEXIST -> false)
.File~new('r1.txt')~renameTo(.File~new('r2.txt')) -> 0   (target exists -> EEXIST -> false)
.File~new('r1.txt')~renameTo(.File~new('r3.txt')) -> 1   (target new -> renamed)
.File~new('r3.txt')~exists -> 1 ;  .File~new('r1.txt')~exists -> 0

.File~new('r3.txt')~renameTo('plainstring')
  -> Error 88.914: Argument destination must be an instance of the File class.
     (raised inside .Validate~classType, StreamClasses.orx:927-929)
```

**Argument-count and argument-type errors**, `survey-B/p05/*.rex`:
```
say .File~new                    -> Error 93.901: Not enough arguments for method; 1 expected.
say .File~new('a','b','c')       -> Error 93.902: Too many arguments in invocation of method; 2 expected.
say .File~new(.array~new(2))     -> rc 0, prints one blank line (see §7 -- not fully explained)
.File~new('plain.txt')~list       -> The NIL object   (receiver is a file, not a directory)
.File~new('plain.txt')~listFiles  -> The NIL object
.File~searchPath('this-does-not-exist-xyz') -> The NIL object
```
Both the 93.901 and 93.902 cases are **already byte-identical on the crate today** (measured, same
program, same stderr text, same rc 163) — they are Object-level `NEW`/argument-count checking, never
reach a `LIBRARY REXX` entry point.

## 3. C++ mechanism

`File` (`interpreter/RexxClasses/StreamClasses.orx:506-1010`) is ordinary Rexx. Every operation that
touches the filesystem goes through one of 24 `EXTERNAL 'LIBRARY REXX file_*'` (or
`this_file_case_sensitive`) methods, each a thin `RexxMethodN` wrapper in
`interpreter/streamLibrary/FileNative.cpp` (337 lines total) around
`interpreter/platform/unix/SysFileSystem.cpp` (2076 lines). All 24 take/return plain `CSTRING`,
`logical_t`, `int64_t`/`uint64_t`, or a `RexxArrayObject` of strings — no object graph crosses the
native boundary except plain values.

**Construction (`init`, `:516-550`) and qualification.** `path = normalizePathSyntax(path)`
(`:581-613`, pure Rexx, uses `MutableBuffer`: strips a trailing separator except at a root, no
native call) runs first, unconditionally. Then:
- one-argument form, or two-argument form with `dir` a plain string: `self~qualifiedPath` is called
  **inside `init`**, eagerly.
- two-argument form with `dir` a `File`: `dir~absolutePath` is read (forcing `dir`'s own
  qualification if not already done) and `path` is built via string concatenation
  (`createPath`, `:614-631`); *this instance's own* qualification is deferred until something reads
  it later (`qualifiedPath`, `:633-645`, memoizes into the `qualifiedPath` ivar, `.nil` until then).

Either way, **every `File` object that is ever asked for `absolutePath`, `name`, `exists`, … touches
`qualifyImpl` (`file_qualify`) at least once**, because the recursion bottoms out at some
plain-string construction. `file_qualify` (`FileNative.cpp:132-137`) is one line:
`QualifiedName qualifiedName(name); return context->String(qualifiedName);` — `QualifiedName`
wraps `SysFileSystem::qualifyStreamName` → `canonicalizeName` (`SysFileSystem.cpp:172-187`,
`:628-672`), which:
1. Expands a leading `~` via `resolveTilde` (`:556-615`, reads `$HOME` or `getpwnam`).
2. If not absolute, prepends **`getCurrentDirectory`** (`:1393-1411`, a bare `getcwd()` loop) —
   this is the process's real current directory on the oracle, and is where **the crate's shadow
   current directory (architecture fact #1) must be substituted** instead.
3. Runs `normalizePathName` (`:687-776`), a **purely lexical**, single-pass, byte-at-a-time
   collapse of `.`  and `..` segments and duplicate/trailing `/` — it explicitly avoids
   `realpath()`/`canonicalize_file_name()` because both require the path to exist and this must
   work for paths that do not. `..` at the root stays at the root (`previousSlash` floors at 0).
   Trailing `/` is stripped unless the whole result is `/`. A component like `..my.file` is left
   alone (the `..` branch only fires when the next byte is `/` or end-of-string).

Empty input is refused outright (`canonicalizeName` returns `false` for `name.isEmpty()`); on
failure `qualifyStreamName` resets to `""`, which is why `.File~new('')~path` is `""` and
`~exists` is `0` rather than an error.

**Predicates**, all via `stat64` (follows symlinks, so a dangling symlink answers `false`/`0` to
`exists`, `isFile`, `isDirectory`, `canRead`):
- `exists` (`:908-914`): `stat64(name, &st) == 0`.
- `isDirectory`/`isFile` (`:817-823`, `:892-898`): `S_ISDIR`/`S_ISREG||S_ISBLK`.
- `canRead`/`canWrite` (`:846-849`, `:876-879`): **`access(name, R_OK)`/`access(name, W_OK)`** —
  a real process-credential access check, not a permission-bit test (matters under setuid, ACLs, or
  read-only filesystems; irrelevant for this project's test environment but not for correctness).
  `file_can_read`/`file_can_write` (`FileNative.cpp:92-104`) additionally require `exists(name)` to
  be true first (`&&`), so a nonexistent path answers `false` from either without ever calling
  `access()`.
- `isHidden` (`:1115-1133`): **requires `exists(name)` first** (returns `false` immediately if not),
  then scans the **whole qualified path string** for the two-byte pattern `"/."` — not just the
  basename. This is why `.File~new("/tmp/.dir/file")~isHidden` is `1` per the doc's own example: an
  ancestor directory starting with `.` makes every descendant "hidden," and it is why my probe
  against a *non-existent* `sub/.dotfile` measured `0`: the exists-first gate fires before the scan.

**Deletion.** `file_delete_file` → `deleteFile` (`:786-794`) **first checks `canWrite(name)`
(`access(W_OK)` on the file itself)** and returns `EACCES` without calling `unlink()` at all if that
fails — this is stricter than raw POSIX `unlink()` semantics, which only cares about the
*directory's* write permission, not the file's own mode bits. A Rust port that just calls
`std::fs::remove_file` and maps its `Result` will accept deletes the oracle refuses whenever a
read-only file sits in a writable directory. `deleteDirectory` (`:803-806`) is bare `remove()` (=
`rmdir()` for a directory) with no such pre-check, so it fails naturally with `ENOTEMPTY` for a
non-empty directory — matching the doc's "only empty directories."

**Timestamps.** `getLastModifiedDate`/`getLastAccessDate` (`:1025-1074`) read `st_mtime`/`st_atime`
(plus sub-second `st_mtim`/`st_atim` nanoseconds when available), run them through `utcToLocal`
(`:942-980`) — which computes the *local wall-clock* representation of the UTC timestamp and then
re-encodes it *as if it were itself a UTC instant counted from year 1* — and add
`StatEpoch = 62_135_596_800` (seconds from `0001-01-01` to `1970-01-01`,
`SysFileSystem.cpp:79`). Result is microseconds since `0001-01-01T00:00:00`, matching `.DateTime`'s
own "basetime" convention exactly (see §5). Failure (bad `stat64` or `gmtime_r`/`localtime_r`)
answers the sentinel `NoTimeStamp = -999999999999999999` (`:80`), which the Rexx wrapper
(`StreamClasses.orx:830-846`) turns into `.nil`. **`getFileLength` (`:1084-1092`) answers `0` on a
`stat64` failure — not an error, not `.nil`** — this is the one predicate whose "missing" answer is
a plain zero rather than a sentinel.
`setLastModifiedDate`/`setLastAccessDate` (`:1145-1212`) read the *current* `stat64` first to
preserve the *other* timestamp (since `utimes()` sets both atomically), convert the new value back
to UTC via `localToUtc` (`:991-1013`, `mktime()`-based, with an explicit epoch-boundary
special-case), and call `utimes()`.
`setFileReadOnly`/`setFileWritable` (`:1223-1257`) `stat64` for the current mode, then
`chmod()` with `S_IWUSR|S_IWGRP|S_IWOTH` cleared or set — **all three write bits**, not just the
owner's.

**Renaming.** `file_rename` → `moveFile` (`:1759-1815`) is not a bare `rename()`: it explicitly
refuses (`EEXIST`, so `.false`, no exception) when the two paths are the *same file*
(`samePaths`) and, separately, when the *target already exists* (`fileExists(toFile)`) —
**both refusals happen before `rename(2)` is ever called**, so a Rust port using
`std::fs::rename` directly (which silently replaces an existing target, standard POSIX behaviour)
diverges on both cases unless it pre-checks. On `EXDEV` (cross-filesystem) it falls back to a
rename-away/rename-back liveness probe, then copy+unlink; that fallback is unreachable inside this
project's probe directories, which are always one filesystem, so it is not exercised or measured
here.

**Listing.** `file_list` (`FileNative.cpp:261-288`) refuses (`.nil`) unless `isDirectory(name)`,
then iterates via `SysFileIterator` (`readdir(3)` order — **filesystem/inode order, not sorted**,
matching the doc's own "not necessarily alphabetic"), skipping literal `"."`/`".."` entries.

**Class-level items.** `file_case_sensitive`/`this_file_case_sensitive`
(`FileNative.cpp:73-86`, `SysFileSystem.cpp:1266-1336`): on this Linux build, gated behind
`HAVE_PC_CASE_SENSITIVE` (Darwin-only) or `HAVE_FS_CASEFOLD_FL` (`ioctl(FS_IOC_GETFLAGS)`
per-directory ext4 casefold, Linux ≥5.2); absent either, the function falls straight through to
`return true` — measured `1` matches "neither macro compiled in, or this filesystem has no
casefold directories." `getRoots` (`:1345-1350`) is a one-line `"/"`. `getSeparator`/
`getPathSeparator` (`:1358-1372`) are the literals `"/"`/`":"`. `getTemporaryPath`
(`:1852-1858`) is `$TMPDIR` or `/tmp` — a *real process environment* read, not the shadow cwd.
`searchPath`'s default `pathList` (`StreamClasses.orx:554-580`, pure Rexx) is
`value("PATH", , "ENVIRONMENT")` — also real-environment, not shadow-state.

**Errors surfaced above the native layer are all ordinary Rexx**, not native: `93.901`/`93.902`
(argument count, `Object>>NEW`/strict-arg checking, nothing File-specific), `88.914`
(`.Validate~classType`, used by `renameTo`'s `dest` check and `writeLines`'s collection-type check,
`StreamClasses.orx:927-929`, `967-968`), `88.916` (`writeLines`/`writeChars`'s mode-character
validation, `:970-971`, `1000-1001`), and **91.999 "did not return a result"** for any expression
context around `setReadOnly`/`setWritable`, which is a straight consequence of those two methods
having no `RETURN` (`:660-667`) — not an error path the C++ or the Rust side does anything special
for.

## 4. The crate today

`rust/crates/rexx-exec/src/dispatch/native.rs` is the whole `LIBRARY REXX` entry-point registry
(D37). `Family::File` (`:41-42`, doc comment names `streamLibrary/FileNative.cpp` as the body it
owes) covers 24 of the 25 File-facing entry points; `Family::owner` (`:51`) reports `"Phase 7"` for
all of `Timer`/`Stream`/`Queue`/`File` except `Timer` which is `"Phase 6"` and already closed.
**Two are already implemented** (`:154-164`): `file_separator` → `native_file_separator`
(`dispatch.rs:8589-8596`) and `file_path_separator` → `native_file_path_separator`
(`dispatch.rs:8598-8605`), each a four-line function hardcoding `b"/"`/`b":"` — matching the oracle
exactly since this project is Linux-only. **The other 24 are `deferred(name, Family::File)`**
(`native.rs:165-187`): `file_case_sensitive`, `file_list_roots`, `file_qualify`, `file_exists`,
`file_delete_file`, `file_delete_directory`, `file_isDirectory`, `file_isFile`, `file_isHidden`,
`file_get_last_modified`, `file_set_last_modified`, `file_set_read_only`, `file_length`,
`file_list`, `file_make_dir`, `file_can_read`, `file_can_write`, `file_rename`,
`this_file_case_sensitive`, `file_get_last_accessed`, `file_set_last_accessed`,
`file_set_writable`, `file_temporary_path`, `file_search_path_impl`.

A send to any deferred entry point runs `deferred_send` (`native.rs:372-379`): the bind itself
always succeeds (so `::METHOD ... EXTERNAL 'LIBRARY REXX file_qualify'` compiles and installs
fine — this is why `File`'s *class* is installable and `.File~separator`/`.File~pathSeparator`
already answer, measured rc 0), but the *send* raises a `Loud` reading `the LIBRARY REXX entry
point "<name>" is not implemented (Phase 7)`, exit code 120 (measured, both `file_case_sensitive`
and `file_qualify`, byte-identical wording each time). Because `init` unconditionally reaches
`qualifiedPath` → `qualifyImpl` before any `File` instance can answer anything else, **`file_qualify`
alone is the entry point that blocks every instance-side File operation today** — confirmed by
exhaustive case analysis in §3 (both single-arg and string-`dir` two-arg forms qualify inside
`init`; the File-`dir` form defers *this* instance's qualification but recurses into a `dir` that
itself was built the same way, so the recursion bottoms out at a qualify call regardless).

`MutableBuffer`, the class `normalizePathSyntax`/`createPath`/`searchPath` build on, is **fully
built** (5c; `corpus/docs/class-set.txt:92`, `corpus/method-bodies.txt:1082-1132` all `answers`) —
this closes the gap an earlier survey found and the brief flags; **`file_qualify` is now the only
remaining blocker**, not a `MutableBuffer` method.

`corpus/method-bodies.txt:1017-1076` carries `File`'s rows. Class-arm: `pathSeparator`, `separator`
already `answers` (rc 0); `isCaseSensitive`, `listRoots`, `temporaryPath` are `loud` naming their
entry point; `readChars`/`readLines`/`searchPath`/`writeChars`/`writeLines` are `answers rc 163` —
**this is misleading as a status**: rc 163 is "message not understood"-shaped in this table's
convention for a class-arm call taking *required arguments* sent with none (the harness's own
no-argument probe), not evidence the underlying stream path works; `readChars`/`readLines`
additionally route through `.Stream`, entirely `Family::Stream` and equally deferred. Instance-arm:
every one of the 32 documented methods plus the `Comparable`/`Orderable` operators (`=`, `==`, `<`,
… `:1059-1076`) is `unanswered — a bare `~new` raises; the method was never sent`, because
`corpus/docs/class-set.txt:89` records `File`'s construction column as `-` (no committed
construction expression) with the reason **"no construction program is committed; a bare ~new
raises 93.901 on the oracle"** — matching §2's measurement exactly. **Recommendation for that
table**: `.File~new('.')` — `.` always exists (it is the interpreter's own directory), is relative
(so it exercises `file_qualify`'s cwd-resolution path rather than skipping it), and every method
in the instance table above can be sent to it without a precondition (delete/renameTo/makeDir would
misbehave against `.` specifically, but the method-bodies harness sends each method bare with no
arguments, so those simply answer `.false`/error the same way they would against any directory).

**The native call signature** (`dispatch.rs`, e.g. `native_file_separator`/`native_length`/
`native_array_put`): `fn(interp: &mut Interp, _cleared: Cleared, receiver: ObjRef, args: &[Option<ObjRef>]) -> Result<Option<ObjRef>, Failure>`.
A string argument is read via `interp.to_text(objref) -> &[u8]`; a string result is built via
`interp.text_built(Vec<u8>) -> ObjRef`; a boolean/integer result via `interp.counted(usize) ->
ObjRef` (`native_is_nil` does exactly this: `interp.counted(usize::from(receiver == ObjRef::NIL))`).
Arity is declared per entry point as `Arity::Fixed(n)` (`dispatch/executable.rs`), and the C++
`RexxMethodN` macro each entry point uses (`FileNative.cpp`) says exactly which `n`: three are
`Fixed(0)` (`file_case_sensitive`, `file_list_roots`, `file_temporary_path` — `RexxMethod0`,
`:73`, `:110`, `:312`), four are `Fixed(2)` (`file_set_last_modified`, `file_set_last_accessed`,
`file_rename`, `file_search_path_impl` — `RexxMethod2`, `:206`, `:224`, `:303`, `:324`), and the
remaining seventeen (including `this_file_case_sensitive`) are `Fixed(1)`, one `CSTRING` name.
None of the 24 need anything beyond `CSTRING`-shaped string args and boolean/integer/string
returns — no object graph crosses the boundary, matching §3's `RexxMethodN` signatures exactly.

**No shadow current directory exists in the crate today.** `require.rs:135` (`normalize(path, cwd)`)
and `lib.rs:2977` (`std::env::current_dir().ok()?`) are the only two places anything resembling a
"current directory" appears, and both read the **real process cwd**, not a per-interpreter shadow —
consistent with there being no `DIRECTORY()` BIF implementation anywhere in the tree yet (the BIF
that would read/write such a field is itself unassigned — `phase-4-exclusions.txt:116-121` names it
"Phase 7's: it needs a pool such as ENVIRONMENT"). **A Phase 7 `file_qualify` implementation and a
`DIRECTORY()` BIF implementation must share one shadow-cwd field** or they will disagree with each
other under a threaded test harness even if each independently "works."

## 5. Design questions

**D1 — path qualification: hand-port `normalizePathName`, do not reach for `std::fs::canonicalize`
or `Path`'s own component normalization.** `canonicalize` resolves symlinks and requires the path
to exist (§3 explains why the oracle deliberately avoids the POSIX equivalent); `std::path::Path`'s
`Components` iterator normalizes repeated separators and literal `.` but does **not** collapse `..`
against a preceding normal segment (that pass does not exist in `std`). Cost of getting this wrong:
every relative path with a `..` in it, and the `..`-past-root case, silently diverges from the
oracle. Recommend a direct byte-for-byte port of `SysFileSystem::normalizePathName`
(`SysFileSystem.cpp:687-776`) as a pure function of `(cwd: &str, input: &str) -> String`, no
dependency, no `unsafe`. Costs one function and its unit tests against the corpus witnesses in §6;
getting it wrong costs silent wrong answers on every corpus program that uses a relative path.

**D2 — the shadow current directory is not File's to own.** It is a Phase 7 cross-cutting piece
(`DIRECTORY()` BIF, `file_qualify`, and presumably `ADDRESS`'s command-issuance cwd argument all
need the same field — D18/commands territory per `phase-4-exclusions.txt:191-200`). Recommend the
field live on `Interp` (one per interpreter instance, satisfying the "no process-global state"
architecture fact), initialized once from `std::env::current_dir()` at interpreter startup and
never touched again except by whatever implements `DIRECTORY()`. Cost if this survey's
recommendation is ignored and File grows its own private notion of cwd: a corpus program that both
calls `DIRECTORY(newdir)` and constructs a `File` with a relative path afterward will pass File's
own tests and fail the corpus gate.

**D3 — `canRead`/`canWrite` need a real `access(2)`, which `std` does not expose.** `std::fs::
metadata().permissions()` only reads mode bits; it cannot answer `access()`'s question under a
setuid binary, an ACL, or a read-only-remounted filesystem, and the oracle's own `isReadOnly`/
`isWriteOnly`/`canRead`/`canWrite` are all defined as `access()` calls, not mode-bit tests
(`SysFileSystem.cpp:833-879`). Two options, both already in the offline registry and both expose a
safe (no caller-side `unsafe`) wrapper: **`rustix` 1.1.4** (`fs::access`, a purpose-built low-level
syscall crate, smallest surface) or **`nix` 0.31.3** (`unistd::access`, a broader POSIX wrapper this
project would then also have available for anything else Phase 7 needs, e.g. `chmod`/`utimes`
equivalents, though those are also reachable through `std::fs::Permissions`/`filetime`).
Recommend **`rustix`**: smaller dependency surface for a single syscall, already a common transitive
dependency in this ecosystem so unlikely to be a net-new supply-chain addition. Cost if skipped
(implementing `canRead`/`canWrite` from mode bits instead): wrong answers specifically in
adversarial/permission-based tests, which is exactly the shape ooTest's file tests are likely to
take (`canRead`/`canWrite` exist to be tested against a chmod'd fixture).

**D4 — `isHidden` needs no dependency and no `unsafe` at all.** §3 established the oracle's own
algorithm is "exists, and the qualified absolute path contains `/.` anywhere" — a pure two-line
string predicate (`existsImpl(path) && path.contains("/.")`), not a filesystem attribute query.
Getting this right is cheaper than getting it wrong; flagged only because "hidden" sounds like it
should need a platform call and does not.

**D5 — timestamps: reuse `builtin/datetime.rs`'s existing epoch machinery, do not re-derive it.**
`.DateTime` is already built (5c; `corpus/docs/class-set.txt:87`, `~new`/`init` both `answers`).
`crates/rexx-exec/src/builtin/datetime.rs` already carries the exact "microseconds since
`0001-01-01`" convention the oracle's `Ticks`/basetime value uses — `UNIX_BASE_TIME`
(`datetime.rs:65-66`, comment: "since both systems count from the identical proleptic 0001-01-01")
is precisely the constant needed to convert a `SystemTime::duration_since(UNIX_EPOCH)` into the
same basetime `.DateTime~new` already accepts. What is *not* already built anywhere in the crate
(not found by this survey) is the UTC→local *wall-clock-as-if-UTC* re-encoding step
(`utcToLocal`, `SysFileSystem.cpp:942-980`) that `getLastModifiedDate`/`getLastAccessDate` apply —
this needs `chrono::Local` to get the local calendar fields, then the existing `base_days`/day
arithmetic in `datetime.rs` to turn those fields into a basetime value, mirroring `utcToLocal`'s own
approach (get local wall-clock fields, re-encode as if they were UTC, shift by the epoch constant)
rather than reusing `UNIX_BASE_TIME`'s pure UTC-epoch shift directly. `chrono` is already a
workspace dependency (`crates/rexx-exec/Cargo.toml:52`, 0.4.45, no new dependency needed) and its
own doc comment already states its Unix path carries no `libc` call and no `unsafe`. For the file
metadata itself (mtime/atime with nanosecond precision), `std::fs::Metadata::modified()`/
`accessed()` return `SystemTime`, sufficient without `filetime`. **`filetime` (0.2.29, in the
registry) is needed only for the *setters*** (`setLastModifiedDate`/`setLastAccessDate`): `std`
has no stable API to *set* atime/mtime, and `filetime::set_file_times` is the standard safe
(no-`unsafe`-at-the-call-site) wrapper around `utimes`/`utimensat`. Recommend adding it now rather
than reinventing a `libc::utimes` call. Cost of skipping `filetime` and hand-rolling: an `unsafe`
site needing per-site sign-off, for no benefit over a well-audited existing crate.

**D6 — `renameTo` and `delete` both need a pre-check the underlying `std::fs` call does not
perform.** §3: the oracle's `moveFile` refuses same-path and existing-target renames *before*
calling `rename(2)`; `deleteFile` refuses via `access(W_OK)` on the file itself *before* calling
`unlink(2)`. `std::fs::rename` silently replaces an existing target (ordinary POSIX behaviour) and
`std::fs::remove_file` does not pre-check the file's own writability (only the OS's own unlink
permission check, which is directory-based, fires). Recommend both be implemented as the oracle's
own multi-step sequence — same-path/exists checks, then `access`, then the `std::fs` call — rather
than a one-line delegation to `std::fs`. Cost if skipped: `renameTo` silently overwrites an existing
file the oracle would refuse to touch (a real data-loss-shaped divergence, not just a wrong return
value), and `delete` succeeds on a `chmod 444` file inside a writable directory where the oracle
answers `.false`.

**D7 — `list`/`listFiles` ordering is filesystem-order (`readdir(3)`), not sorted, on both sides.**
`std::fs::read_dir` is also `readdir`-ordered and `.`/`..` are never yielded by it (unlike the raw
syscall), so this is a close match already; the risk is a task writer "fixing" it to sort for
determinism and thereby diverging from the oracle. No dependency, no `unsafe`; flagged as a
do-not-fix.

**Platform scope**: everything above is Linux-specific by design (this project's stated platform);
`SysFileSystem.cpp`'s Windows sibling was not read and none of these recommendations claim to cover
it. No `unsafe` is required anywhere in this design if D3's `rustix`/`nix` recommendation is taken
for `access()` and D5's `filetime` for the setters — both expose safe APIs, so no per-site approval
against the workspace's `unsafe_code = "forbid"` should be needed for File specifically.

## 6. Proposed task slices

Ordered; each independently testable against the oracle; corpus witness names are proposed, not
committed.

1. **Shadow current directory on `Interp`, plus `file_qualify`.** Witness:
   `corpus/file/qualify.rex` — relative, absolute, trailing-separator, empty, `~`-containing (or
   documented as out of scope if `$HOME` expansion is deferred), and `..`-past-root names, from a
   fixed starting directory. Negative/adjacent: the existing `.File~new` argument-count cases
   (93.901/93.902, §2) as a regression guard that this task must NOT touch — they already pass.
   This unblocks every other instance method transitively (§3/§4), so it is the only task on the
   critical path; everything below is parallelizable once it lands.
2. **Predicates**: `file_exists`, `file_isDirectory`, `file_isFile`, `file_isHidden`,
   `file_can_read`, `file_can_write`. Witness: `corpus/file/predicates.rex` over a file, a
   directory, a missing path, a dangling symlink, and a chmod'd-unreadable file (probe-only, never
   committed to the repository chmod'd). Negative/adjacent: `file_length` on the same missing path
   must answer `0`, not `.nil` and not an error (§3) — the corpus program should assert this
   explicitly since it is the one predicate whose "missing" convention differs from the others.
3. **`file_length`, `file_list`.** Witness: `corpus/file/list.rex`, asserting `.nil` on a
   non-directory and that `.`/`..` never appear (order itself is not asserted, per D7).
4. **Timestamps**: `file_get_last_modified`, `file_get_last_accessed`,
   `file_set_last_modified`, `file_set_last_accessed`. Witness: `corpus/file/timestamps.rex`,
   round-tripping a `DateTime` through `~lastModified=` and back, and asserting `.nil` on a missing
   file. Negative/adjacent: assert the *type* of a fresh file's `~lastModified` is `.DateTime` (a
   corpus program run without a fixed reference timestamp cannot assert the *value* portably).
5. **Mutation**: `file_make_dir`, `file_delete_file`, `file_delete_directory`,
   `file_set_read_only`, `file_set_writable`. Witness: `corpus/file/mutate.rex`, run in a
   throwaway directory the corpus harness owns, asserting `makeDir` twice (`1` then `.false`),
   `delete` on a non-empty directory (`.false`, no error), and — the one from D6 — `delete` on a
   `chmod 444` file inside a writable directory (`.false`, matching the oracle's own pre-check,
   *not* whatever `std::fs::remove_file` alone would answer). Negative/adjacent: assert
   `setReadOnly`/`setWritable` raise 91.999 under `say` (§2) — a corpus program that only tests the
   side effect and never exercises the "no return value" shape would leave that divergence
   unwitnessed.
6. **`file_rename`.** Witness: `corpus/file/rename.rex`: same-path (`.false`), existing-target
   (`.false`), successful rename (`1`, and the old path stops existing). This is D6's other half and
   is the task most likely to be implemented wrong by a straight `std::fs::rename` delegation, so
   its witness should assert existing-target refusal explicitly, not just the happy path.
7. **Class-only natives**: `file_case_sensitive`, `this_file_case_sensitive`, `file_list_roots`,
   `file_temporary_path`, `file_search_path_impl`. Witness: `corpus/file/class-methods.rex`.
   `isCaseSensitive` should be measured against the oracle at task-implementation time, not assumed
   `true` — this survey measured `1` on this specific machine/filesystem (§2/§3) but the
   `HAVE_FS_CASEFOLD_FL` branch means the honest Rust answer is also "true unless a Linux
   ext4-casefold directory is involved," which needs its own explicit corpus comment rather than a
   hardcoded `true` presented as universal.
8. **`method-bodies.txt` and `class-set.txt` updates**: change `class-set.txt:89`'s construction
   column from `-` to `.File~new('.')` (§4's recommendation) once task 1 lands, and re-derive the 42
   `File` rows in `method-bodies.txt` from the instance-arm harness — this is D-P7-4's stated gate
   mechanism and should be its own task rather than folded into task 1, so the method-body-table
   regeneration is reviewed on its own diff.
9. **`readChars`/`readLines`/`writeChars`/`writeLines`** are explicitly *not* File-native work —
   they are pure Rexx over `.Stream` (`StreamClasses.orx:941-1010`) and are blocked on
   `Family::Stream`'s natives, which are another surveyor's area. Listed here only so a task
   planner does not accidentally schedule them alongside the File-native tasks above under the
   assumption they share a blocker.

## 7. Not done / not established

- **Windows.** `interpreter/platform/windows/SysFileSystem.cpp`/`.hpp` were not read; every
  claim in §3/§5 is Unix-only, matching this project's stated platform scope.
- **`.File~new(.array~new(2))` answering rc 0 with a blank line** (§2) was measured but not
  explained — I did not trace far enough into `MutableBuffer~new`'s handling of a non-string
  constructor argument or `File`'s own `~string` override interacting with an object whose default
  string representation is not obviously empty. Not load-bearing for any recommendation above; flagged
  so a task author does not assume it is unexercised rather than unexplained.
- **`.DateTime~fullDate`**, which `lastModified=`/`lastAccessed=` call
  (`StreamClasses.orx:844`, `:863`) to convert the argument back to a settable value, was not
  independently confirmed to already answer on the crate (5c's `DateTime` rows in
  `corpus/method-bodies.txt` were spot-checked for `init`/`new`/`from*` but `fullDate` specifically
  was not grepped). If it is still `loud`, task 4 above has a second, DateTime-side dependency this
  survey did not surface.
- **The `EXDEV` cross-filesystem fallback inside `moveFile`** (§3, copy-then-unlink path) is
  unreachable from any probe run inside a single-filesystem scratchpad directory and was read, not
  measured; a task implementing `file_rename` should treat it as optional/deferred rather than a
  gate requirement unless the corpus gains a way to force two filesystems.
- **`isCaseSensitive`'s `HAVE_FS_CASEFOLD_FL` compile-time branch** was read from source but this
  build's actual compiled configuration (whether that macro is defined at all) was not confirmed
  from `CMakeCache.txt` or a build log — only the *outcome* (`1`) was measured, which is consistent
  with either "the macro is undefined" or "it is defined and this directory has no casefold
  attribute set."
- **`~` (tilde) home-directory expansion** in a `File` path (`resolveTilde`, §3) was read but not
  measured — no probe used a `~`-prefixed name, since doing so from a sandboxed probe directory
  risks touching `$HOME` outside the probe root, which the brief's file-system rule forbids. A task
  author should decide whether Phase 7 owes this at all before scoping it into task 1's witness set.

<!-- SURVEY COMPLETE -->
