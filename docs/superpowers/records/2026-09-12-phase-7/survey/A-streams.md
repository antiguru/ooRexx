# Survey A — the stream model and the stream builtins

Surveyor A, Phase 7, 2026-09-12. Read-only survey; tree at `5bcb28edb`. Probes live under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/survey-A/pNN/`
(`$P` below). Oracle is `build/bin/rexx` 5.3.0 under the standard wrapper; crate is the snapshot
binary `bins/h-5bcb28edb`. Every claim is marked **Measured** (program and three descriptors given),
**Read** (`file:line`) or **Inferred**.

Section status: all seven written. Probe scripts: `$P/../survey-A/run1.sh`, `run2.sh`, plus the
inline batches for `p08d`, `p08e`, `p10c`, `p10d`, `p16`, `p17`, `p18`, `x1`-`x6`.

## 1. Documented surface

Enumerated from the `<section>` titles of `oodocs/rexxref/en-US/streamclasses.xml` (**Read**, `grep -n '<title>'`),
the chapter sections of `iostrms.xml`, and the `bif*` sections of `funct.xml`. Method signatures in the
reference are SVG images (`images/classes/stream_charin.svg`, `streamclasses.xml:562`) and are not
machine-readable; the argument shapes below are taken from `interpreter/RexxClasses/StreamClasses.orx`
(`use strict arg` lines) and the `RexxMethodN(...)` declarations in `interpreter/streamLibrary/StreamNative.cpp`
(**Read**, cited in §3).

### 1a. The `Stream` class (`streamclasses.xml:389`)

| item | doc | contract (one line) |
|---|---|---|
| `new(name)` | `:483` | initialise for `name` (string or `.File`), does not open |
| `init(name)` | `:1079` | same; the orx body at `StreamClasses.orx:154-178` |
| `arrayIn([type])` | `:503` | Array of remaining lines (`LINES`, default) or chars (`CHARS`) from the read position; first line may be partial after `charIn` |
| `arrayOut(array[,type])` | `:530` | writes each element with `lineOut` (`LINES`, default) or `charOut` (`CHARS`) |
| `charIn([start][,length])` | `:553` | up to `length` (default 1) chars from the read position; `start` repositions; past end → `""` and NOTREADY; implicit open BOTH, falling back to READ |
| `charOut([string][,start])` | `:580` | count of chars NOT written (0 on success); `""` writes nothing, returns 0; **no args closes the stream**; implicit open BOTH, falling back to WRITE |
| `chars` | `:612` | chars from the read position to the end incl. line ends; transient: 1 if data present else 0 |
| `close` | `:639` | `READY:` on success, error message otherwise, `""` if never opened |
| `command(string)` | `:661` | dispatches OPEN / CLOSE / FLUSH / SEEK / POSITION / QUERY; error result is a `description`-shaped string with ERRNO |
| — OPEN options | `:716-789` | READ, WRITE, BOTH (default), APPEND (default), REPLACE, SHARED, SHAREREAD, SHAREWRITE, NOBUFFER, BINARY, RECLENGTH n (BINARY on a nonexistent file opened for write requires RECLENGTH, `:774`) |
| — CLOSE | `:803` | `READY:` or error; `""` if unopened |
| — FLUSH | `:811` | writes buffered output |
| — SEEK offset [READ\|WRITE] [CHAR\|LINE] | `:815-893` | prefixes `=` (default) `<` `+` `-`; both positions move when open BOTH and neither READ nor WRITE given; stream must be open; returns the new position or an error string; LINE on a write-only stream is an error (`:877`) |
| — POSITION | `:895` | synonym for SEEK |
| — QUERY DATETIME | `:909` | US format `MM-DD-YY HH:MM:SS`, `""` if nonexistent |
| — QUERY EXISTS | `:925` | full path or `""` |
| — QUERY HANDLE | `:941` | fd of the open stream (needs open) |
| — QUERY POSITION [READ\|WRITE] [CHAR\|LINE\|SYS] | `:956` | read position by default when open BOTH, else whichever is open; SYS is the OS position |
| — QUERY SEEK | `:999` | synonym for QUERY POSITION |
| — QUERY SIZE | `:1002` | size in bytes of a persistent stream |
| — QUERY STREAMTYPE | `:1015` | `PERSISTENT`, `TRANSIENT` or `UNKNOWN` |
| — QUERY TIMESTAMP | `:1021` | ISO `YYYY-MM-DD HH:MM:SS` |
| `description` | `:1039` | `state` + `:` + extra info for ERROR/NOTREADY |
| `flush` | `:1061` | `READY:`; writes buffered output |
| `lineIn([line][,count])` | `:1098` | next line (`count` 0 or 1); `line` repositions, must not be given for a transient stream; implicit open BOTH → READ |
| `lineOut([string][,line])` | `:1126` | 0 on success, 1 on error; `line` repositions; **both omitted closes**; implicit open BOTH → WRITE |
| `lines([option])` | `:1152` | `Count` (default for the method) = number of lines left, `Normal` = 1/0; a partial first line counts |
| `makeArray([type])` | `:1201` | same as `arrayIn` (orx: `forward message 'ARRAYIN'`, `:211`) |
| `open([options])` | `:1222` | `READY:` or error string; option set identical to the OPEN command |
| `position(spec)` | `:1333` | synonym for `seek` |
| `qualify` | `:1351` | fully qualified name, stream need not be open |
| `query(spec)` | `:1368` | the QUERY subcommands without the word QUERY; `""` for nonexistent streams; HANDLE and POSITION need an open stream |
| `say([string])` | `:1509` | 0 / 1, as `lineOut` |
| `seek(spec)` | `:1528` | as the SEEK command |
| `state` | `:1620` | `ERROR`, `NOTREADY`, `READY`, `UNKNOWN` (closed/never opened) |
| `string` | `:1667` | the stream name as given |
| `supplier` | `:1685` | a `StreamSupplier` over the remaining lines with line numbers as indexes |
| `uninit` | `:1704` | closes on collection; a subclass `uninit` must call `super~uninit` last |

### 1b. The mixins (`streamclasses.xml:57-388`)

| class | doc | contract |
|---|---|---|
| `InputOutputStream` | `:57` | abstract mixin inheriting both; no methods of its own (`:94`) |
| `InputStream` | `:103` | `charIn`/`chars`/`lineIn`/`lines` abstract → 93.965 (`:155,175,195,215`); `charOut`/`lineOut`/`position` → 93.963 (`:165,205,234`); `open`/`close` NOPs (`:185,225`); `arrayIn` default loops `lineIn` until NOTREADY (`:145`, orx `:102-112`) |
| `OutputStream` | `:245` | `charOut`/`lineOut` abstract → 93.965 (`:308,348`); `charIn`/`chars`/`lineIn`/`lines`/`position` → 93.963 (`:298,318,338,358,378`); `open`/`close` NOPs; `arrayOut` default loops `lineOut` (`:288`, orx `:62-67`) |
| `StreamSupplier` | no section of its own in `streamclasses.xml`; the class is `StreamClasses.orx:371-434` | `next` raises 93.937 when exhausted; `index` is the 1-based line number; closes the stream at the end only if it was `UNKNOWN` when the supplier was made (`:385-388,410-411`) |

### 1c. The model chapter (`iostrms.xml`)

| item | doc | contract |
|---|---|---|
| input streams; read position | `:122-199` | persistent streams keep a read position starting at 1; transient streams have none, `CHARS`/`LINES` answer 1 "if data is waiting or a determination cannot be made" (`:189-193`) |
| output streams; write position | `:201-246` | separate from the read position; starts at the end (append) |
| default stream names | `:390-441` | `STDIN`/`STDOUT`/`STDERR` case-insensitive reserved names (`:410`); `HANDLE:n` for an already-open fd (`:420`), cannot be reopened once closed (`:432`); `.INPUT`/`.OUTPUT` used when no name is given (`:401`) |
| line vs character positioning | `:444-501` | binary+RECLENGTH lines are fixed records; non-binary LINE positioning scans; moving the write position flushes buffered output (`:461`) |
| implementation | `:504-531` | first `CHARIN`/`CHAROUT`/`CHARS`/`LINEIN`/`LINEOUT`/`LINES` opens the stream, for BOTH if possible (`:519,527`); `LINEOUT(name)` alone closes (`:522`) |
| errors during I/O | `:623-684` | NOTREADY is trappable, never fatal when untrapped; `CALL ON` delays to clause end and reads return `""` meanwhile; `CONDITION('O')` is the stream object (`:670`); `SAY`/`PULL` on the defaults never raise it (`:681`) |

### 1d. The builtins (`funct.xml`)

| builtin | doc | contract |
|---|---|---|
| `CHARIN([name][,[start][,length]])` | `:1286` | default name STDIN, length 1; `start` not valid for transient; length 0 repositions only; past end → `""` + NOTREADY |
| `CHAROUT([name][,[string][,start]])` | `:1370` | residual count; `""` → 0 written; `start` alone repositions; **neither → close**, returns 0 |
| `CHARS([name])` | `:1458` | as the method; `CHARS(nonfile)` → 0 (`:1497`) |
| `LINEIN([name][,[line][,count]])` | `:3053` | count 0 or 1; `line` repositions; partial line after `CHARIN` |
| `LINEOUT([name][,[string][,line]])` | `:3159` | 0/1; `line` must be within bounds for non-binary (`:3208`); neither → close, 0 |
| `LINES([name][,option])` | `:3276` | **default `Normal`** for the builtin (`:3304`) where the method defaults to `Count`; `C`/`N` |
| `QUALIFY([name])` | `:3545` | full path, file need not exist |
| `STREAM(name[,operation[,command]])` | `:4177` | `S` (default) / `D` / `C`; only the first letter is read (`:4205`); stream state is program-global, not saved across internal calls (`:4230-4234`); OPEN returns `READY:` or e.g. `ERROR:2` (`:4293`) |

Not covered in this survey: `QUEUE:` as a stream name (`funct.xml:3139`; Phase 10 per the brief),
`HANDLE:n` beyond the two probes in §2.11, the `SHARED*` options' cross-process semantics (single
process here), and every mixin abstract-method error (93.963/93.965 are Phase 5 raises already in place).

## 2. Oracle behaviour

All **Measured** 2026-09-12 against `build/bin/rexx` 5.3.0 under the standard wrapper, each program
run from a fresh `$P/pNN/o/` directory holding the fixtures `subdir/` (a directory),
`unreadable.txt` (`secret\n`, `chmod 000`), `fixed.txt` (`l1\nl2\nl3\n`, mtime `2015-11-12 03:29:12`)
and, from batch 2, `two.txt` (`ab`, no newline). Programs are at `$P/pNN/prog.rex`; the three
descriptors at `$P/pNN/o.out`, `o.err`, `o.rc`. stderr is empty and rc is 0 unless stated. The
crate was run beside every probe from `$P/pNN/c/` and refused each one at its first stream call
(§4 has the exact refusal lines); the oracle columns below are therefore the whole of the evidence.

Three probes killed the oracle (rc 139, SIGSEGV). None is in `rust/corpus/oracle-crashes.txt`; §7
has the recommended entries. **Do not re-run them.**

* `p07`: `say 'clause x' || linein(f) || 'y' || linein(f) || 'z'` with `CALL ON NOTREADY` armed and
  `f` at EOF — the same `CALL ON` condition queued twice in one clause, which is crash entry 5's
  shape with a different condition name. stdout up to and including the handler's line was
  written (`clause xyz` then the trap line), then SIGSEGV.
* `x3` (isolated from `p13`): `say lineout('HANDLE:1','via handle 1')` — a `HANDLE:n` stream used
  for output. `stream('HANDLE:1','C','query streamtype')`, `stream('handle:1','S')`,
  `.Stream~new('HANDLE:1')~state`, `~string`, `~qualify` and `stream('HANDLE:99','C','query exists')`
  all answer (`UNKNOWN`, `UNKNOWN`, `UNKNOWN`, `HANDLE:1`, `<cwd>/HANDLE:1`, `""`) without crashing
  (`x1`,`x2`,`x4`,`x5`,`x6`); only the write does.
* `p10b`: `s~uninit` followed by `s~state` on the same `Stream`. `stream_uninit` destroys the
  `StreamInfo` and drops `CSELF` (`StreamNative.cpp:2195-2198`); `stream_state` dereferences the
  now-null `CSELF` without the `checkStreamInfo` guard (`:3609-3610`). Measured safe after
  `uninit`: `~lineout`, `~linein`, `~open`, `~chars` raise **48.1** `Failure in system service:
  Stream not initialized.`; `~close` and a second `~uninit` return no result (91.999 when used as
  a value) (`p10c`). Inferred from the same code shape and **not run**: `~description`, `~qualify`,
  `~query('exists'|'handle'|'size'|'timestamp'|'streamtype')` after `uninit` crash the same way.

### 2.1 Implicit open, read/write positions, EOF, close (`p01`)

```
f = 'a.txt'
say 'lineout' lineout(f, 'first')            -> lineout 0
say 'lineout' lineout(f, 'second')           -> lineout 0
say 'S' stream(f,'S') 'D' stream(f,'D')      -> S READY D READY:
say 'handle' stream(f,'C','query handle')    -> handle 5
say 'pos w' ...write 'r' ...read 'dflt' ...  -> pos w 14 r 1 dflt 1
say 'pos w line' .. 'r line' ..              -> pos w line 3 r line 1
say 'pos sys' .. 'seek' stream(f,'C','query seek') -> pos sys 13 seek 1
say 'size' .. 'type' ..                      -> size 13 type PERSISTENT
say 'exists' ..                              -> exists /…/p01/o/a.txt
say 'lines' lines(f) 'C' lines(f,'C') 'N' lines(f,'N') 'chars' chars(f) -> lines 1 C 2 N 1 chars 13
say 'linein' linein(f)                       -> linein first
say 'chars' chars(f) 'lines C' lines(f,'C')  -> chars 7 lines C 1
say 'charin3' charin(f,,3)                   -> charin3 sec
say 'linein partial' linein(f)               -> linein partial ond
say 'lines' lines(f) 'chars' chars(f)        -> lines 0 chars 0
say 'eof linein [' || linein(f) || ']'       -> eof linein []
say 'S' .. 'D' ..                            -> S NOTREADY D NOTREADY:EOF
say 'eof charin [' || charin(f) || ']'       -> eof charin []
say 'S' .. 'D' ..                            -> S ERROR D ERROR:0
say 'lines at eof' lines(f) lines(f,'C') 'chars' chars(f) -> lines at eof 0 0 chars 0
say 'close via lineout' lineout(f)           -> close via lineout 0
say 'S' .. 'D' ..                            -> S UNKNOWN D UNKNOWN:
say 'close unopened [' || stream(f,'C','close') || ']' -> close unopened []
say 'reopen lineout' lineout(f, 'third')     -> reopen lineout 0
say 'pos w' .. 'r' ..                        -> pos w 20 r 1
say 'close' stream(f,'C','close')            -> close READY:
say 'open read' stream(f,'C','open read')    -> open read READY:
say 'pos w' .. 'r' .. 'dflt' ..              -> pos w 0 r 1 dflt 1
say 'linein' linein(f)                       -> linein first
say 'lineout on read-only' lineout(f,'nope') -> lineout on read-only 1
say 'S' .. 'D' ..                            -> S ERROR D ERROR:13 Permission denied
say 'close' stream(f,'C','close')            -> close READY:
closed queries: exists /…/a.txt ; size 19 type UNKNOWN ; handle [] ; pos [] ; S UNKNOWN D UNKNOWN:
fixed datetime 11-12-15 03:29:12 ; fixed timestamp 2015-11-12 03:29:12 ; fixed size 9
flush READY: ; flush unopened READY: ; S never UNKNOWN D UNKNOWN:
```

So: a first `LINEOUT` opens BOTH (read position 1, write position at end); the write position is
the 1-based index of the *next* byte (14 after 13 bytes); `QUERY POSITION` with neither READ nor
WRITE answers the read position when the stream is open BOTH; `QUERY POSITION SYS` is the 0-based
OS offset; `LINES()` with no option is `Normal` (1/0) for the builtin; `CHARIN` after `LINEIN`
continues mid-line and a following `LINEIN` returns the rest of the line; reading at EOF with
`LINEIN` gives `""` and `NOTREADY:EOF`, with `CHARIN` gives `""` and **`ERROR:0`** (§3 explains the
difference); a read-only stream answers a `LINEOUT` with `1` and `ERROR:13 Permission denied`
(the errno is EACCES chosen by the stream layer, not the OS); `QUERY EXISTS`, `SIZE`, `DATETIME`,
`TIMESTAMP` work on a closed stream, `HANDLE`, `POSITION` and `STREAMTYPE` do not (`""`, `""`,
`UNKNOWN`); `FLUSH` on a never-opened stream answers `READY:` and leaves it `UNKNOWN`.

### 2.2 OPEN modes and options (`p02`, `p02b`, `p16`, `p17`, `p10d`)

| command | answer | notes |
|---|---|---|
| `open write` (new or existing) | `READY:` | write pos 1 on an empty file; **`linein` on a write-only stream still reads** (`[abc]`, state READY — the fd is opened RDWR, §3), but `lines`/`chars` answer 0 and `charin` reads too |
| `open read` | `READY:` | `query position write` → `0`; `charout` → residual count `3`, `ERROR:13 Permission denied`; `open read` on a nonexistent file → `ERROR:2` and the stream stays `UNKNOWN` (not `ERROR`) -- **this row is the builtin form**, whose table entry is dropped on a failed open; the object form leaves `ERROR`, as §2.7's last line says. Both re-measured 2026-09-12: `stream(f,'c','open read')` then `stream(f,'s')`/`(f,'d')` answer `ERROR:2`, `UNKNOWN`, `UNKNOWN:`, while `.Stream~new(g)~open('read')` then `~state`/`~description` answer `ERROR:2`, `ERROR`, `ERROR:2 No such file or directory`. Either way a trappable NOTREADY is raised |
| `open both append` | `READY:` | write pos = size+1, read pos 1 |
| `open both replace` | `READY:` | size 0, write pos 1 |
| `open` (no options) | `READY:` | BOTH; a second `open` on an open stream closes and reopens, `READY:` |
| `open write replace`, `open write append` | `READY:` | |
| `open both shared` / `shareread` / `sharewrite` / `nobuffer` | `READY:` | no observable difference single-process |
| `open both binary` on `hello\nworld\n` | `READY:` | RECLENGTH defaults to the file size: `lines('C')` 1, `linein` returns the whole 12 bytes incl. newlines |
| `open read binary reclength 4` | `READY:` | `lines('C')` 3; `linein` → `hell`, `o\nwo`, `rld\n`; `seek =2 read line` → `2`, read pos `5` |
| `open write binary` on a **nonexistent** file | **93** uncaught (`Error 93 running REXX line 270: Incorrect call to method.`, rc 163, no sub-line) | the doc's "requires RECLENGTH" case; same 93 for `open binary` on an **empty** file (`p16`, `p10d`) |
| `open write binary reclength 4` on a new file | `READY:` | `lineout 'ab'` pads to `ab  `; `lineout 'abcdefg'` (longer than the record) → **93**; `charout 'pq'` then `lineout 'r'` completes the record: bytes `616220207778797A70717220` |
| `open bogus`, `open read write`, `open read read`, `open append replace`, `open read append`, `open read replace`, `open shared shared`, `open nobuffer nobuffer`, `open both reclength 0`, `open both reclength`, `open both reclength abc`, `open reclength 2` (no BINARY), `open read binary reclength 4 binary` | **93.0** | mutually exclusive or repeated options, a RECLENGTH without BINARY, a zero/missing/non-numeric length |
| `open nosuch write` / `both` / `append` / `replace` / bare | `READY:` and the file now exists | `open nosuch read` → `ERROR:2` |
| `open both binary reclength 2` | `READY:` | |
| `  open    both   `, `open<TAB>both` | `READY:` | the command word is parsed by Rexx `PARSE`, the options by the C++ tokenizer (blanks only; a tab is part of the token, yet `open<TAB>both` still opened — `PARSE UPPER VAR command command_word parms` splits on blanks, so `command_word` was `OPEN<TAB>BOTH`, which `' OPEN'` still matched by prefix; the options string was empty and the default BOTH applied) |

Abbreviations (`p16`, `p17`; `f.txt` non-empty): `READ` needs 3 (`r`,`re` → 93; `rea` ok); `WRITE`
1; `BOTH` 2 (`b` → 93); `APPEND` 2 (`a` → 93); `REPLACE` 3 (`re` → 93 — it is READ's prefix and
too short for either); `NOBUFFER` 3; `BINARY` 2 (`bi`, `bin` ok); `RECLENGTH` 3 (`rec 2`, `recl 2`
ok); `SHARED` 6 exactly (`share` → 93); `SHAREREAD` 6 (`sharer` ok); `SHAREWRITE` 6 (`sharew` ok).
Matching is caseless prefix against the *table entry* at the input token's length, first match in
table order (READ, WRITE, BOTH, APPEND, REPLACE, NOBUFFER, BINARY, RECLENGTH, SHARED, SHAREREAD,
SHAREWRITE), then the minimum-length check (`StreamCommandParser.h:98-101`, `.cpp:131-156`).

`~open` answers on an object (`p10d`): `open('read')`, `open('write')`, `open('both')`,
`open('both replace')` each `READY:`; `open('read binary')` on the now-empty file → uncaught 93.
`s~open` on an open stream: `READY:` (closes and reopens, `p10`).

### 2.3 SEEK / POSITION and QUERY POSITION (`p03`, `p03b`, `p15`, `p17`)

File `s.txt` = `line one\nline two\nline three\n` (29 bytes), `open both`:

```
seek =1 read -> 1        seek 3 -> 3 (r 3 w 3: both moved)      seek +2 read -> 5    seek -1 read -> 4
seek <1 read -> 29       seek <0 read -> 30      seek <99 read -> -69   (no bound check; state stays READY)
seek =2 read line -> 2 ; query position read line -> 2 ; linein -> line two
seek +1 read line -> 4 ; linein -> ""            (line 4 is past the last line; state NOTREADY:EOF after the read)
seek 0 read -> 0 ; S READY                       seek 999 read -> 999 ; charin -> "" ; S ERROR D ERROR:0
seek 1 read -> 1 ; S READY                       (a seek resets the state to READY)
position =1 write line -> 1
seek write | seek | position | seek = | seek + | seek <  -> 93.903 Missing argument in method; argument SEEK is required.
seek abc read | seek 1 read bogus | seek 1 read write | seek 1 char line | seek 5 6 | seek 1e1 read | seek =1.5 read | seek 1 read char line | seek 1 line char | seek 1 read read -> 93.0
seek = 5 -> 5 (blank after the prefix is fine)   seek + 3 -> 8   seek +0 read -> 2   seek -0 read -> 2   seek -5 read (from 1) -> -4
seek 0 read line -> 1   seek 99 read line -> 4 (clamped to last line + 1; state READY)
unopened.txt (nonexistent): seek 1 -> 0, then S ERROR D ERROR:2 No such file or directory  (SEEK implicitly opens; failure answers 0 through the builtin)
fixed.txt (exists, unopened): seek 2 -> 2, S READY, read pos 2      (SEEK implicitly opens BOTH)
query position on an unopened stream -> "" ; query handle -> ""
seek 5 write line (file has 3 lines) -> 4 ; S READY      seek 2 write line -> 2, write pos 10
charout 'LINE' at write pos 10 -> read pos 30 / read line 3 unchanged ; write pos 34 / write line 2
query position (no READ/WRITE) after a write -> 30 (the READ position) ; seek 3 (no READ/WRITE) -> both 3
after a linein: seek 5 -> r 5 w 5
open write: seek 2 line -> 2 ; seek 2 write line -> 2 ; seek 2 read -> 2 (all accepted on a write-only stream)
open write: query position -> 31 (write) ; query position read -> 2 ; query position write line -> 2
open read: query position -> 1 ; query position write -> 0 ; query position write line -> 2
binary reclength 10: seek 3 read line -> 3 ; query position read line -> 3 ; seek <1 read line -> 2
/dev/null (transient): open read -> ERROR:2, NOT READY: -- `fileExists` is `S_ISREG` only
  (`platform/unix/SysFileSystem.cpp:198`-`:212`), so the read-only path rejects a character
  device; `open write`/`open both` -> READY: and TRANSIENT, and `seek 1` on that is uncaught
  93.958 "Positioning of transient streams is not valid." Re-measured 2026-09-12 by Task 8,
  both the builtin and the object form.
```

The doc's "line positioning in a write-only file gives an error message" (`streamclasses.xml:877`)
does **not** reproduce: `seek 2 write line` on `open write` answered `2`. The character-mode
`SEEK` default (`=`) and the `<` arithmetic are as documented; the result is never range-checked.
`QUERY POSITION ... LINE` costs a scan on a variable-record file and the line counts it caches are
what §2.6's anomaly is about.

Abbreviations (`p16`): `r`/`re`/`rea`/`read`, `w`/`wr`, `c`/`ch`, `l`/`li` all accepted for SEEK and
QUERY POSITION; `s`/`sy`/`sys` accepted by QUERY POSITION (→ `0`, the OS offset) and **rejected by
SEEK** (93.0). QUERY subcommand words match the first entry of `DATETIME EXISTS HANDLE POSITION SEEK
SIZE STREAMTYPE TIMESTAMP` that starts with the abbreviation (`s` → SEEK, `si` → SIZE, `st` →
STREAMTYPE, `t` → TIMESTAMP, `p` → POSITION, `e`,`h`,`d` likewise); an abbreviation matching none
(`sizes`) → 93.0; **`query` with no subcommand answers DATETIME** (`query`, `q`, `qu` all →
`09-12-26 …`), because `parse value ' DATETIME …' with ('') +1` matches at column 1. Command words
match the first of `CLOSE FLUSH OPEN POSITION QUERY SEEK` with the same prefix rule (`c` → CLOSE,
`s` → SEEK, `p` → POSITION, `f` → FLUSH, `o` → OPEN); `opens` → 93.914.

### 2.4 Writes: CHAROUT/LINEOUT positioning, past-end writes, no-string forms (`p04`, `p04b`, `p15`)

```
w.txt: charout 'abcdef' -> 0 ; charout(f,,3) -> 0, w 3 ; charout 'ZZ' -> 0, w 5 ; charout(f,,20) -> 0, w 20, S READY
       charout 'Q' -> 0, w 21 ; query size (open) 20 ; charout(f) -> 0 (close) ; size 20
       bytes 61625A5A6566 + 13 x 00 + 51                       (a hole is zero-filled by the OS)
charout(f,,0) | charout(f,,-1) | charout(f,'z',0) | lineout(f,'q',0) | lineout(f,,0)
       -> 93.907 Method argument 1 must be a positive whole number; found "0".   (note "argument 1")
charin(f,,0) -> "" ; charin(f,2,0) -> "" and read pos 2 ; charin(f,1,-1) -> 88.907 Argument 2 must be in the range 0 to 18446744073709551615; found "-1".
charin(f,1,99) on 6 bytes -> abcdef, S NOTREADY D NOTREADY:EOF (short read = EOF) ; charin(f,7) -> "" NOTREADY:EOF ; charin(f,6) -> f READY
charin(f) at end -> "" ERROR:0 (twice) ; linein(f) at end -> "" NOTREADY:EOF
two.txt ('ab'): charin 1,2 -> ab READY ; charin -> "" ERROR:0 ; linein(f,1) -> ab READY ; linein -> "" NOTREADY:EOF ; lines 0 chars 0 pos 3
l.txt: lineout one/two/three -> 0 0 0 ; lineout('TWO',2) -> 0, w 19, w line 3 ; lineout('X',5) -> 0, S READY
       (write position for line 5 of a 3-line file is clamped to "after the last line": file becomes
        one\nTWO\nthree\n then, because the clamped char position 15 was set by charout-style seek past
        the 14-byte file, 4 NULs then X\n: 6F6E650A54574F0A74687265650A00000000580A)
       lineout(f,,1) -> 0, w 1 ; lineout('end',4) -> 0 ; lineout('a much longer line two',2) overwrites from line 2 to EOF
lineout('e.txt','') -> 0, size 1 (a bare newline) ; charout('e2.txt','') -> 0, file created, size 0
charout('e3.txt') | lineout('e4.txt') alone -> 0, file created, state UNKNOWN (open-for-write then close)
lines('e5.txt') on a nonexistent file -> 0, not created, S ERROR D ERROR:2 No such file or directory ; chars('e6.txt') the same
charout('e7.txt',,3) on a new file -> 0, size 0, READY ; lineout('e8.txt',,3) -> 0 READY ; lineout('e9.txt',,1) then lineout 'x' -> 780A
```

The `(f,,20)` case shows a write position past the end is accepted for `CHAROUT` (doc `:1405`) and
the hole is filled with NULs by the OS on the next write. `LINEOUT` with `line` past the end does
not raise (doc `funct.xml:3208` says it cannot be specified; the oracle clamps).

### 2.5 Line ends, empty files, CR handling, buffer growth (`p05`, `p17`)

| file bytes | `lines(C)` | `lines(N)` | `chars` | `linein` results (`c2x`), read pos after each, line pos after each |
|---|---|---|---|---|
| `a\nbb\n` | 2 | 1 | 5 | `61` (3, 2), `6262` (6, 3) |
| `a\nbb` (no final LF) | 2 | 1 | 4 | `61` (3, 2), `6262` (5, 3) |
| `a\r\nbb\r\n` | 2 | 1 | 7 | `61` (4, 2), `6262` (8, 3) — the CR before LF is stripped, the position skips it |
| `a\rbb\r` | 1 | 1 | 5 | `610D62620D` (6, 2) — a bare CR is data |
| empty | 0 | 0 | 0 | — ; `query exists` non-empty, size 0 |
| `a\0bb\n\nc` | 3 | 1 | 7 | `61006262` (6,2), `` (7,3), `63` (8,4) — NUL is data |
| `\n\n` | 2 | 1 | 2 | `` (2,2), `` (3,3) |
| `a\r\r\nb\r` | 2 | — | — | `610D`, `620D` — only the CR *immediately* before LF is dropped; `charin(1,4)` sees `610D0D0A` |

After a `do while lines(f)` loop ends, the state is `READY` (the loop stopped on `lines` = 0, so no
EOF read happened). `seek =2 read line` on the CRLF file → `2`. `charout 'abc'`, `lineout 'def'`,
`charout 'ghi'`, close → `abcdef\nghi` (LINEOUT terminates whatever partial line CHAROUT left).
`linein(f,1,0)` → `""` and read pos 1; `linein(f,2,1)` → `bb`; `linein(f,3)` on a 2-line file →
`""` with `NOTREADY:EOF`; `linein(f,1)` afterwards → `a` and `READY` (an explicit position clears
EOF). `LINEOUT` writes `\n` only (`610A`). A 12,000-byte line reads back whole (`length 12000`).
A file ending in `0x1A`: the next `LINEOUT` **overwrites the ctrl-Z** (`ab\x1a` + `lineout 'X'` →
`6162580A`), `StreamNative.cpp:2520` (Windows EOF-marker convention, applied on unix too).

### 2.6 LINES: both modes, the count cache and its off-by-one (`p04b`, `p15`, `p10`)

```
lc.txt = a\nb\nc\n
lines(f,'C') x3            -> 3 3 3
linein -> a ; lines C x2   -> 2 2
charin -> b ; lines C x2   -> 2 1        <- second count after a CHARIN is one less than the first
  query position read line -> 2 ; lines C -> 0   <- and zero after a line-position query
fresh: open read, seek 3 read, lines C x2 -> 3 2
fresh: N then C then C     -> 1 3 3
after lineout 'd' (append): lines C x2 -> 4 4
open both, lines C, lineout 'e', lines C x2 -> 4 5 5
open both, lines C, linein, lineout 'f', lines C x2 -> 5 (a) 5 5
Stream object t.txt with 7 lines (p10): s~chars 29 s~lines 7 s~lines('N') 1 s~lines('Count') 6 s~lines('normal') 1
  -- the same anomaly: a Count after a Count (with a Normal between) answers one less
lc2.txt (3 lines) via an object: s~lines s~lines s~lines s~lines('C') s~chars -> 3 3 3 3 6  (consistent when nothing else intervenes)
```

The method's default is Count (`s~lines` = 7), the builtin's is Normal (`lines(f)` = 1). The
`2 1` and `7 … 6` rows are an oracle defect in `countStreamLines`'s `stream_line_size` cache
(§3.6) — after a CHARIN resets `lineReadPosition` to 0, the cached total is recomputed as
`count + 0 - 1`. A byte-for-byte port either reproduces the cache or documents the divergence;
recommended in §5.

### 2.7 Errors: nonexistent, directory, unreadable, missing parent, devices (`p06`, `p10b`)

```
nosuch.txt: linein -> "" ; S ERROR D ERROR:2 No such file or directory ; lines 0 chars 0 ; charin -> "" (same state)
            query exists [] size [] streamtype UNKNOWN datetime [] timestamp [] handle [] position []
            open read -> ERROR:2, then S UNKNOWN D UNKNOWN: (a failed explicit open leaves UNKNOWN) ; close -> ""
subdir:     linein -> "" ; ERROR:2 No such file or directory (open of a directory fails with ENOENT, SysFile.cpp:141-146)
            query exists -> "" (a directory does not "exist" as a stream) ; streamtype UNKNOWN ; size [0]
            open read -> ERROR:2 ; lineout -> 1 ; S ERROR D ERROR:21 Is a directory ; lines 0 chars 0
unreadable.txt (mode 000): linein -> "" ; ERROR:13 Permission denied ; open read/write/both -> ERROR:13 ; exists 1 size 7 type UNKNOWN
            lines 0 chars 0 ; lineout -> 1 ; ERROR:13 Permission denied
subdir/nosub/x.txt: lineout -> 1, charout -> 1 ; ERROR:2 No such file or directory
/dev/null:  lineout -> 0 ; streamtype TRANSIENT ; query size 0 ; lines 0 chars 0 ; linein -> "" S NOTREADY ; query exists /dev/null
/dev/zero:  chars 1 lines 1 ; TRANSIENT ; charin(,,3) -> 000000
object form (p10b): w=.Stream~new('subdir'); w~open('read') -> ERROR:2 ; w~state ERROR ; w~description ERROR:2 No such file or directory
            u=.Stream~new('unreadable.txt'); u~open -> ERROR:13 ; u~open('write') -> ERROR:13 ; u~lineout('a') -> 1
            n=.Stream~new('nosuch.txt'); n~open('read') -> ERROR:2 ; n~state ERROR   <- the *object* form leaves ERROR where the builtin form left UNKNOWN
```

The last line is a real asymmetry: `stream(n,'C','open read')` on a nonexistent file leaves the
stream `UNKNOWN` because the builtin **removes the failed stream from the table**
(`BuiltinFunctions.cpp:2524-2527`, any OPEN answer other than `READY:`), so the next `stream(n,'S')`
resolves a fresh object; `n~open('read')` on a held object leaves that object in `ERROR`.

### 2.8 NOTREADY: when it is raised, what the condition object carries (`p07`, `p07b`, `p14`)

With `SIGNAL ON NOTREADY` (`p07`): `say 'before' linein(f) 'after'` at EOF jumps to the label; the
`say` does not print. In the handler: `condition('C')` = `NOTREADY`, `condition('D')` = `n.txt` (the
stream name **as given**, not qualified), `condition('O')` is a `Directory` (not the stream),
`condition('A')` = `n.txt`, `condition('I')` = `SIGNAL`, `condition('S')` = `OFF`; `rc` is unset
(`RC`), `sigl` is the raising line; `stream(f,'S')` = `NOTREADY`, `D` = `NOTREADY:EOF`.

The directory (`p14`, `o~allIndexes~sort`): `ADDITIONAL` = `nosuch.txt` (**the stream name, a
string**, not the stream object — but see the next line), `CONDITION` = `NOTREADY`,
`DESCRIPTION` = `nosuch.txt`, `INSTRUCTION` = `SIGNAL`, `PACKAGE` = `The REXX Package`,
`PROGRAM` = `REXX`, `PROPAGATED` = `0`, `RESULT` = `""`, `STACKFRAMES` = a List,
`TRACEBACK` = a List — 10 entries; no `RC`, `CODE`, `ERRORTEXT`, `MESSAGE`, `POSITION`. For the
EOF case on a held object, `o~additional == s` is **1**: `ADDITIONAL` *is* the stream object
(`context->RaiseCondition("NOTREADY", String(stream_name), self, result)`,
`StreamNative.cpp:342,406` — description is the name, additional is `self`), and it printed as
`nosuch.txt` above because a `Stream`'s string value is its name. `PACKAGE`/`PROGRAM` are
`REXX` because the raising frame is the `Stream` method inside the `REXX` package.

`RESULT` is the value the failing call answered: `""` for a read, `1` for `lineout('subdir','x')`,
`3` for `charout('subdir','xyz')` (the residual count), the short string `l1\nl2\nl3\n` for
`charin('fixed.txt',1,99)`, `ERROR:21` for `stream('subdir','C','open write')`, `1` for a
`lineout` on a read-only object.

`CALL ON NOTREADY` (`p07b`, one raising call per clause): the handler runs at the end of the
clause and the clause's own `say` prints first with the empty/residual value; `condition('S')` =
`DELAY` inside the handler. Every failing operation below raised it, with the description shown:

```
linein EOF  n.txt NOTREADY:EOF        charin EOF  n.txt ERROR:0          lines/chars at EOF: 0, no raise
linein nosuch  ERROR:2 …              charin nosuch  ERROR:2 …           lines nosuch: 0 AND raises (state ERROR:2)
lineout dir  ERROR:21 Is a directory  charout dir  same
stream open nosuch/dir  -> ERROR:2 answered AND raised; stream state afterwards UNKNOWN (table entry dropped)
stream open unreadable  -> ERROR:13 answered and raised
seek 999 read -> 999, no raise ; charin at 999 -> "" raises, ERROR:0
stream('zz.txt','C','seek 5') -> 0 and raises (the implicit open failed) ; query position/handle on a closed stream -> "" no raise
charin(f,99) / linein(f,99) beyond the end -> "" raises, NOTREADY:EOF
lineout/charout on read-only -> 1 raises ERROR:13 ; close of it -> READY: no raise ; flush closed -> READY: no raise
.Stream~new('zz.txt')~arrayin~items -> 0 and .Stream~new(f)~arrayin~items -> 1: neither reaches the caller's CALL ON (arrayIn's own `signal on notready`, StreamClasses.orx:228/233, swallows it; the trap line that follows in o.out belongs to the next clause's `s~linein` at EOF)
sup = .Stream~new('zz.txt')~supplier -> sup~available 0 (the `~item`/`~index`/`~next` rows of p07b are void: the `try` helper was a PROCEDURE and saw the literal SUP; p10's supplier rows stand)
/dev/null linein -> "" NOTREADY:EOF ; charin -> "" ERROR:0 ; STDIN (empty) linein -> "" NOTREADY:EOF ; charin -> "" ERROR:0 ; lines 0 chars 0
```

Untrapped (`p01` etc.): the program continues, the function answers `""`/`1`/residual, and the
state is visible through `STREAM(...,'S')`. `::OPTIONS NOTREADY SYNTAX` turns an untrapped raise
into a syntax error (`Activity.cpp:619-621`, `condtra.xml:283`). **Probed 2026-09-12 by
Task 7**: `Error 98.974:  Stream "n.txt" is not ready.` at rc 158, substituting the stream
name as given; `Error_Execution_notready_syntax = 98974` (`messages/RexxErrorCodes.h:643`).

### 2.9 The per-activation stream table: sharing across calls (`p12`, `p12b`)

`sh.txt` has six lines; `main` reads `one`.

```
internal CALL inner (no PROCEDURE)   -> reads two ; main after -> three          (shared)
CALL proc_inner (PROCEDURE EXPOSE f) -> reads four ; main after -> ""/five        (shared)
::routine rout                       -> reads the next line ; main after continues (shared; p12b: routine four, main five, routine 2 six)
::routine closes it (stream ...'C','close') -> main after: reopened from line one  (same table entry)
method o~m(f) (a ::class method)     -> reads one ; main after -> two              <- p12b: main one, method one, main two, method 2 one, method 3 one, main three
   so a METHOD activation has its OWN table and its own Stream object for the same name; each method call gets a fresh table (method 2 read `one` again)
   -- BUT p12's earlier read had the method continue main's stream after main had explicitly closed/reopened it; p12b's cleaner sequence shows the separate table. p12's `main after method one` and p12b agree: main's position was not advanced by the method.
INTERPRET "say linein(f)"            -> continues main's stream (interp one after a close/reopen; main two)
names: linein('./sh.txt') continues the same stream as 'sh.txt' (three) ; linein(qualify(f)) too (four) — the table key is the qualified name
       'SH.TXT' -> "" ERROR (case-sensitive fs, a different file) ; 'sh.txt ' -> "" ERROR (not stripped) ; ' sh.txt' -> "" (not stripped)
STATE from a routine or a method after main closed: UNKNOWN everywhere; stream('STDOUT'|'stdout'|'Stdout','S') -> READY READY READY
```

So: one table per **program or method** activation, and internal calls, `PROCEDURE` routines,
`::ROUTINE`s and `INTERPRET` share their caller's table by reference (`RexxActivation.cpp:2049-2079`,
§3.9). `.Stream~new(name)` is always a fresh object outside the table (`p12`: `object vs builtin
one`; `p10d`: two objects on the same file both append, `from a1\nfrom a2\nfrom a1 again\n`).
At program end every table entry gets `CLOSE` (`closeStreams`, `:4691`), which is what flushes
`p09b`/`p09c`'s unclosed files (§2.11).

### 2.10 The standard streams (`p08`, `p08b`, `p08c`, `p08e`)

`STDIN`/`STDOUT`/`STDERR` (any case, with or without a trailing colon) resolve to `.INPUT`,
`.OUTPUT`, `.ERROR` — the **Monitors** — not to the Stream objects (`RexxActivation.cpp:1967-1978`;
the traceback of a seek error shows `Method UNKNOWN with scope "Monitor"` forwarding to the Stream,
`p08d`). `.stdout~class` = `The Stream class`, `.output~class` = `The Monitor class`,
`.input~class` = `The Monitor class`, `.error` = `The ERROR monitor`, `.stdout` = `STDOUT`,
`.stderr` = `STDERR` (`p18`); `.Stream~new('STDOUT') == .stdout` is `0` and the new one is
`UNKNOWN` until used (`p10b`).

With stdin a regular file of `l1\nl2\nl3\nl4\nl5abcdef\nl6\nl7` (`p08`, stdout also a file):
`linein()` → `l1`; `parse pull` → `l2`; `.input~linein` → `l3`; `.stdin~linein` → `l4`;
`charin(,,3)` → `l5a`; `linein()` → `bcdef`; `linein('stdin:')` → `l6`; `lines()` 1 `chars()` 2
`lines(,'C')` 1; the final unterminated `l7` reads as `l7`; then `linein()` → `""` `NOTREADY:EOF`,
`charin()` → `""` `ERROR:0`, `parse pull` → `""`. **One position, shared by every construct.**
`stream('STDIN','C','query streamtype')` was `TRANSIENT` even for a regular file; `query exists`
→ `""`... in `p08` the first line printed `type` — lost to the overwrite artifact below; re-measured
in `p08e` with a pipe: `TRANSIENT`, `chars 1 lines 1 lines C 1 pos 1`. `query handle` 0 (`p08b`),
`STDIN datetime`/`timestamp` answered the *current* time (`p08b`, an artifact of `ctime` on a
non-regular fd: `SysFile.cpp:1145` only fills it for `S_IFREG`, and the orx date arithmetic runs
on `""` → today).

STDOUT to a **regular file** (`p08`): `query streamtype` → `PERSISTENT`, `query position` 1,
`query handle` 1, `S` READY, `query exists` → `STDOUT`; `qualify('STDOUT')` → `<cwd>/STDOUT` (no
special case in QUALIFY). Closing STDOUT (`stream('STDOUT','C','close')` → `READY:`, then
`UNKNOWN`) and continuing to `say` **rewound the output file**: the next writes went to offset 0
and overwrote earlier output (`p08`, `p08b`; `o.out` there is scrambled and only its tail is
trustworthy). Through a **pipe** (`p08c`, `p08e`): STDOUT and STDIN are `TRANSIENT`, STDERR (still a
file) `PERSISTENT`; `say 'one'`, `charout 'STDOUT','two-'`, `say 'three'` → `one\ntwo-three\n`;
close STDOUT → `READY:`/`UNKNOWN`, then `say 'after close'` and `lineout 'STDOUT',…` still appear
in order (`after close\nlineout after close\n`) — closing a std stream does not close fd 1
(`SysFile::close` skips `::close` for `openedHandle == false`, `SysFile.cpp:299`); but the output
**stopped after `charout 'STDOUT'`** (a close through CHAROUT) — `after charout close`, `end`,
`STDERR close`, `err after close` never appeared on stdout, and stderr got only `err line`. So a
`charout('STDOUT')` / `lineout('STDOUT')` close makes the following `SAY`s vanish silently while
`stream(...,'C','close')` does not; **not explained by the code read here** (§7).

Other std facts: `linein('STDOUT')` → `""` `NOTREADY:EOF`; `chars('STDOUT')` 0 `lines` 0 and the
description stays `READY:`; `lineout('STDIN','z')` → `1` `ERROR:13 Permission denied`; `charout`
likewise; `stream('STDIN','C','close')` → `READY:`, `UNKNOWN`, and the next `linein()` reopens and
**continues** (`l2`); `open read`/`open write`/`open nobuffer` on STDIN all `READY:`; `charin('STDIN',1,2)`,
`linein('STDIN',2)`, `seek 1` on STDIN or STDOUT → **93.958 Positioning of transient streams is not
valid.** (uncaught in `p08d`, rc 163); `query position read line` on STDOUT → `1`; `open nobuffer` on
STDOUT/STDERR → `READY:` and output continues; `lineout('STDOUT:',…)`, `lineout('stderr:',…)`,
`linein('STDIN:')` all work; `lineout('/dev/stdout',…)` writes in order with SAY (`p08`).

### 2.11 Output ordering and flushing (`p09`, `p09b`, `p09c`)

`p09` stdout, exact: `say1\nco1lo1\nsd1\nsc1sl1\nout1\nlo2\nsay2\nco2say3\nco3nolf-at-end` (no
final newline); stderr: `err1\nerr2\nerr3\nerr4\n`. So `SAY`, `charout('STDOUT')`,
`lineout('STDOUT')`, `.stdout~say/charout/lineout`, `.output~say`, `lineout(,…)`, `charout(,…)`
are one byte stream in program order; `lineout('STDERR')`, `.stderr~say`, `.error~say`,
`lineout('stderr:')` are one stderr stream. `p09b`: `lineout 'f.txt','buffered line'` + `charout
'f.txt','partial'` left unclosed, `open write`+`charout` on `g.txt`, a closed `h.txt`, then `say
1/0` → rc 214, stderr the usual two-line Error 42 report — **all three files hold their bytes**
(`buffered line\npartial`, `via open write`, `closed properly\n`). `p09c` (`exit 3`): the same,
and stdout `x1x2` (two `.stdout~charout`s, no newline), stderr `e1\n`, rc 3. Buffered stream
output is flushed at program end on every exit path the probes took.

### 2.12 The Stream class surface (`p10`, `p10b`, `p10c`, `p10d`, `p11`)

```
s = .Stream~new('t.txt'): class The Stream class ; isA InputOutputStream/InputStream/OutputStream 1 1 1 ; ~string t.txt ; ~defaultName a Stream ; ~objectName a Stream
  ~state UNKNOWN ~description UNKNOWN: ; ~qualify == qualify('t.txt') ; ~query('exists') "" ; ~close "" ; ~open READY: ; ~open again READY: ; ~open('write') READY:
  ~lineout('one') 0 ~lineout('two') 0 ~say('three') 0 ~say 0 (writes an empty line) ; ~arrayout(.array~of('four','five')) 0 ; ~arrayout(...,'C') 0 ; ~arrayout(.array~of(''),'chars') 0
  ~chars 29 ~lines 7 ~lines('N') 1 ~lines('Count') 6 (§2.6) ; ~position('=1 read') 1 ~seek('+2 read') 3 ~query('position read') 3 ~query('seek read') 3 ~query('POSITION READ LINE') 1
  ~linein e ; ~charin t ; ~charin(1,2) on ; ~linein(2) two ; ~linein(,0) "" ; ~chars 21
  ~arrayin -> three||four|five|six (from the read position; empty line kept) ; then ~lines 0 ~state NOTREADY
  ~makearray('L')~items at EOF 0 ; after ~position('=1 read') ~makearray -> all 7 ; ~arrayin('C') at EOF 0 ; ~position('=1') (both) then ~arrayin('chars')~items 0 (!) ; ~position('=1 read') then ~arrayin('c') -> every byte
     (this stream's last `~open` was `open('write')`, which sets `write_only`; a SEEK with neither READ nor WRITE then moves only the write pointer, `StreamNative.cpp:2718-2721`, so the read pointer stayed at EOF — consistent with p03b, where the same stream opened BOTH moved both)
  do l over s (after ~position('=2 read line')) -> lines 2..7 ; ~supplier: class The StreamSupplier class, index = line number (2..7), ~available 0 when exhausted
  supplier on a CLOSED stream: index 1 item one, the stream is opened (READY) and closed again when exhausted (UNKNOWN) ; supplier on an UNKNOWN-then-read stream: 1 one, state READY afterwards
  ~command('open read') READY: ~command('query size') 29 ~command('CLOSE') READY: ~command('flush') READY: ~command('close') "" (already closed)
  ~query('streamtype') UNKNOWN (closed) ~query('handle') "" ~query('timestamp') 2026-09-12 02:31:39 ~query('datetime') 09-12-26 02:31:39 ; ~flush READY: (closed) ~state UNKNOWN
  ~uninit -> no result (91.999 if used as a value) ; .Stream~new(.File~new('t.txt'))~string -> the absolute path ; .Stream~new(42)~string -> 42 ; .Stream~new('')~string "" ; .Stream~new(.array~new)~string ""
  .Stream~new(.nil) -> 93.938 Method argument 1 must have a string value. ; .Stream~new -> 93.901
  ~makestring -> 97.1 (no MAKESTRING) ; lineout(.Stream~new('viaobj.txt'),'hello') -> 0 and the file exists (the builtin takes the string value) ; lineout(.File~new('viafile.txt'),'hi') likewise
  ~~open('write')~chars -> 0 ; ~seek('=1') on a closed object -> 1 (implicit open) ; ~position('=1 read') -> 1 ; ~query('position') closed -> ""
  ~lineout returns 0, ~charout 0, ~say 0 ; two objects on one file both append (a1/a2 interleave in write order)
  .SubStream (init calls self~init:super('sub_'||n)) -> ~string sub_sub.txt, writes go to that file
```

Argument errors (`p11`, `try` harness printing `condition('O')~code | ~message`):

```
stream('a.txt','X')            40.904 STREAM argument 2 must be one of SDC; found "X".      stream('a.txt','')  same with found ""
stream('a.txt','S','open')     40.4 Too many arguments in invocation of STREAM; maximum expected is 2.   (also stream('a.txt',,'query exists'))
stream('a.txt','C')            40.3 Not enough arguments in invocation of STREAM; minimum expected is 3.
stream()                       40.3 … minimum expected is 1.      stream('')  40.27 STREAM argument 1 must be a valid stream name; found "".
stream(' ')                    UNKNOWN (a blank name is a name)     stream('a.txt','C','')  ""     stream('a.txt','C','query')  "" (a.txt does not exist: DATETIME of a nonexistent file)
stream('a.txt','C','query bogus'|'query size extra'|'query position read write'|'query position bogus'|'close now'|'flush now')  93.0 (message text "The NIL object" — a bare `raise syntax 93` in the orx)
stream('a.txt','C','bogus')    93.914 Method argument 1 must be one of CLOSE FLUSH OPEN POSITION QUERY SEEK; found "bogus".
stream('a.txt','C','open',4)   40.4 … maximum expected is 3.     stream('a.txt','description') UNKNOWN:    stream('a.txt','command','QUERY EXISTS') ""    stream('a.txt','c','Query Streamtype') UNKNOWN   stream('a.txt','s') UNKNOWN
lines('a.txt','X'|'')          40.904 LINES argument 2 must be one of CN; found "X".     lines('a.txt','count'|'normal') 0     lines('a.txt','C','extra') 40.4 … maximum expected is 2.
linein('a.txt',1,2)            93.0     linein('a.txt',0) ""  linein('a.txt',-1) ""  (the *builtin* passes them through; on a nonexistent file the open fails first)
linein('a.txt','a')            40.12 LINEIN argument 2 must be a whole number; found "a".   linein('a.txt',1.5) 40.12   linein('a.txt',,'') 40.12 argument 3
linein('a.txt',,-1)            88.907 Argument 2 must be in the range 0 to 18446744073709551615; found "-1".   (the method's argument 2 = the builtin's argument 3)
charin('a.txt',0|-1) ""        charin('a.txt','q') 40.12 argument 2     charin('a.txt',,-1) 88.907 Argument 2 …    charin('a.txt',,'q') 40.12 argument 3
charout('a.txt','q',0|-1)      93.907 Method argument 1 must be a positive whole number; found "0".   charout('a.txt','q','pos') 40.12 CHAROUT argument 3 …   charout('a.txt',,'pos') 40.12
lineout('a.txt','q',0|-1)      93.907   lineout('a.txt','q','pos') 40.12   lineout('a.txt','q',1,'extra') 40.4 … maximum expected is 3.
charout('a.txt',.array~new) 0  lineout('a.txt',.nil) 0 (nil = omitted)  lineout(.nil,'q') 0 (to STDOUT)  lineout('a.txt','a',1.0) 0
chars('a.txt','x')             40.4 … maximum expected is 1.      qualify() 40.3 … minimum expected is 1.   qualify('') ""   qualify('a.txt','extra') 40.4
.Stream~new('a.txt')~command('close now') 93.0 ; ~command('') "" ; ~command 93.901 ; ~query('bogus') 93.0 ; ~query('size extra') 93.0 ; ~query('') -> the DATETIME (§2.3)
~lines('q')                    93.915 Method option must be one of "CN"; found "q".    ~arrayin('q') 93.915 … "CL" …    ~arrayout(.array~of('a'),'q') 93.0    ~arrayout('notarray') 0 (DO OVER a string yields nothing)
~linein(1,2) 93.0   ~charin(0) 93.907   ~charin(,-1) 88.907   ~lineout('a',0) 93.907   ~say('a','b') 93.902 Too many arguments in invocation of method; 1 expected.
~open('bogus') 93.0   ~open('read','extra') 88.922 Too many arguments in invocation; 1 expected.   ~seek 88.901 Missing argument; argument 1 is required.   ~seek('') 93.903 Missing argument in method; argument SEEK is required.
~position('1','x') 88.922 … 1 expected.   ~query 93.901   ~state('x') 88.922 … 0 expected.   ~string('x') 93.902 … 0 expected.   ~qualify('x') ~chars('x') ~close('x') ~flush('x') ~description('x') 88.922 … 0 expected.   ~supplier('x') 93.902 … 0 expected.
~seek('1') on a closed object 1 ; ~seek('1 read line') 1 ; ~query('position') closed ""
.InputStream~new~linein 93.965 Method LINEIN is ABSTRACT and cannot be directly invoked. ; .InputStream~new~lineout('a') 93.963 Call to unsupported or unimplemented method. ; .OutputStream / .InputOutputStream symmetric
```

The `88.9xx` codes come from the native-method argument checks (a native `RexxMethodN` with a
typed `int64_t`/`size_t` argument, message text `Argument N` counting the method's own arguments)
and the `93.9xx` from the orx `use strict arg` lines and the C++ `raiseException` calls; which one a
given error uses is fixed by which layer sees it first, so a port has to keep the same split.

### 2.13 QUALIFY and stream names (`p13`, `p12`)

```
qualify('a.txt')          /…/p13/o/a.txt          qualify('./sub/../a.txt')  /…/p13/o/a.txt      qualify('sub/./a.txt')  /…/p13/o/sub/a.txt
qualify('a//b')           /…/p13/o/a/b            qualify('/abs/../x/y')     /x/y                 qualify('/') /       qualify('.') /…/p13/o    qualify('..') /…/p13
qualify('a.txt/')         /…/p13/o/a.txt          qualify('~')  /home/moritz   qualify('~/a') /home/moritz/a     qualify('$HOME/a') /…/p13/o/$HOME/a (no env expansion)
qualify('STDIN'|'stdout:'|'STDERR'|'handle:5') -> <cwd>/STDIN etc. (no special names)     qualify(' sp ') -> "<cwd>/ sp " (blanks kept)     qualify('nosuch/deeper/../x') -> <cwd>/nosuch/x (purely lexical)
qualify('a.txt') == .Stream~new('a.txt')~qualify -> 1 ; .Stream~new('./a.txt')~string -> ./a.txt (string is as given)
query exists: nosuch "" ; fixed.txt /…/fixed.txt ; ./fixed.txt the same ; subdir/ "" (a directory) ; subdir/../fixed.txt /…/fixed.txt
```

No symlink resolution and no filesystem access: `..` is collapsed lexically (`/abs/../x/y` →
`/x/y`), trailing `/` and doubled `/` are removed, `~` and `~/` expand to `$HOME` (`~user` not
probed), anything else is joined onto the current directory
(`SysFileSystem::canonicalizeName`, `platform/unix/SysFileSystem.cpp:628-672`).

Not measured in this section: `SHARED*` across processes, `HANDLE:n` beyond the crash and the
five safe answers above, `QUEUE:`, `::OPTIONS NOTREADY SYNTAX`, `~uninit` at collection time
(the `CustomStream` subclass in `p10c` printed ` custom uninit` at process end, after every other
line, so a subclass `uninit` does run at exit — but whether it ran from a collection or from the
shutdown sweep was not separated), and `LINES`/`CHARS` on a TTY.

## 3. C++ mechanism

All **Read**. Paths are under `/home/moritz/dev/repos/ooRexx/`; `StreamNative.cpp` means
`interpreter/streamLibrary/StreamNative.cpp`, `SysFile.cpp` means `common/platform/unix/SysFile.cpp`.

### 3.1 The layers

1. **Builtin** (`interpreter/expression/BuiltinFunctions.cpp:2126-2570, 2988-3000`): argument
   checking with the `40.x` messages, `QUEUE:` detection (`check_queue`), then
   `context->resolveStream(name, input, fullName, &added)` and a **message send** of the same name
   to the object it returns (`LINEIN`, `CHARIN`, `LINEOUT`, `CHAROUT`, `LINES`, `CHARS`, `STATE`,
   `DESCRIPTION`, `COMMAND`). Arguments are passed positionally by `argcount`, so an omitted middle
   argument arrives as omitted (`:2163-2179`). `LINES` defaults `option` to `NORMAL` and squashes
   the method's count to 1/0 itself (`:2348-2391`); `LINEOUT(name)` alone and
   `STREAM(name,'C','... CLOSE ...')` **remove the table entry** after the send (`:2277`, `:2539`);
   `STREAM(name,'C','... OPEN ...')` removes it when the answer is not `READY:` (`:2524-2527`).
   The OPEN/CLOSE/SEEK detection is `wordPos` on the upper-cased command (`:2516,2531,2543`), so it
   fires on any word, e.g. `query open` would take the OPEN branch. `STREAM`'s second argument is
   checked by first character only (`:2460`); `S`/`D` refuse a third argument (`:2469-2471`), `C`
   requires one (`:2502-2505`). `QUALIFY` is `QualifiedName(name)` (`:2998`), i.e.
   `SysFileSystem::qualifyStreamName` → `canonicalizeName`.
2. **Activation stream table** (`interpreter/execution/RexxActivation.cpp:1938-2082`): §3.9.
3. **`Stream` class, Rexx half** (`interpreter/RexxClasses/StreamClasses.orx:121-369`): `init`
   (`:154-178`) requires a string value (93.938), calls `!c_stream_init`, then marks
   `STDIN/STDOUT/STDERR[:]` via `!std_set` and `HANDLE:n` via `!handle_set(substr(name,8))`;
   `command` (`:245-287`) and `query` (`:291-356`) are the `PARSE VALUE ' CLOSE FLUSH OPEN POSITION
   QUERY SEEK' WITH (' 'word) +1 word .` dispatchers whose prefix rule §2.3 measured; `query`
   answers `DATETIME`/`TIMESTAMP` by reformatting the C `ctime` string `!query_time` returns
   (`Thu Nov 12 03:29:12 2015\n` → `parse var c_time . month day time year` → `date('O',…)` /
   `date('S',…)`), and returns `""` for a `TRANSIENT` stream; `arrayout` (`:182-209`) loops
   `lineout`/`charout` under `signal on notready` and `raise propagate return (items - count)`;
   `arrayIn` (`:214-240`) sizes the array with `self~lines` or `self~chars` and fills it with
   `line_arrayin` (native) or a `charin` loop under `signal on notready`; `makearray` forwards to
   `ARRAYIN`; `say` (`:358-365`) is `lineout(line)` with `line = ""` default; `supplier` builds a
   `StreamSupplier` (`:371-434`) that remembers whether the stream was `UNKNOWN` (then closes it
   when exhausted) and reads `linein(position)` for a persistent stream or `linein` for a
   transient one, `next` raising 93.937 once exhausted.
4. **`StreamInfo`, the native half** (`StreamNative.cpp`): one C++ object per `Stream` instance,
   allocated in a `RexxBuffer` stored in the object variable `CSELF` (`:3724-3732`); every native
   method fetches it with `checkStreamInfo` (`:107-119`, raises 48.1 `Stream not initialized` when
   `CSELF` is gone) except the eight query/state methods that dereference it unguarded (the
   post-`uninit` crash, §2).
5. **`SysFile`** (`SysFile.cpp`): fd, a 4096-byte buffer that is either read-buffered or
   write-buffered (`writeBuffered` flips it, flushing on the switch, `:388-394`, `:533-543`), and
   the type flags from `getStreamTypeInfo` (`:1184-1224`): `transient` = `isatty(fd)` or
   `S_IFCHR` or `S_IFIFO`; `device` = tty or char device. `open` refuses a directory with
   `ENOENT` after a successful `open(2)` (`:141-146`), sets `append` and seeks to the end for
   `O_APPEND` (`:158-163`), and buffers unless the fd is a tty (`:171-178`). The std streams are
   **unbuffered** (`setStdIn/Out/Err` call `setBuffering(false, 0)`, `:1237,1254,1272`) and
   `openedHandle = false`, so `close` never closes fd 0/1/2 (`:299`).

### 3.2 State and the answers it produces

`StreamInfo` fields (`StreamNative.hpp:158-192`): `charReadPosition`, `charWritePosition`
(1-based, next byte), `lineReadPosition`, `lineWritePosition`, `lineReadCharPosition`,
`lineWriteCharPosition` (0 = "not tracked"), `stream_line_size` (cached total line count, 0 =
unknown), `state` ∈ {Unknown, Ready, Notready, Eof, Error}, `errorInfo`, `binaryRecordLength`,
`read_only`/`write_only`/`read_write`/`append`/`nobuffer`/`stdstream`/`last_op_was_read`/
`opened_as_handle`/`transient`/`record_based`/`isopen`.

`STATE` (`:3584-3602`): `UNKNOWN`, `NOTREADY` (for both Notready and Eof), `ERROR`, `READY`.
`DESCRIPTION` (`:3621-3685`): `UNKNOWN:`, `NOTREADY:EOF`, `NOTREADY:%d %s` / `NOTREADY:%d`,
`ERROR:%d %s` / `ERROR:%d` (the text is `strerror(errno)` when `errorInfo != 0`, hence `ERROR:0`
bare and `ERROR:13 Permission denied`), `READY:`. The state Notready is never assigned in this
file — only Eof and Error are — so `NOTREADY:%d` is unreachable from the stream library.

The two raise paths (`:318-345`, `:390-411`): `notreadyError(errno, result)` sets `state =
StreamError`, `errorInfo = errno`, clears the SysFile error, raises `NOTREADY` with
`description = stream_name`, `additional = self`, `result`, then `throw this`; `eof(result)` sets
`state = StreamEof` and raises the same condition. `checkEof()` (`:419-432`) picks `eof()` when
`fileInfo.atEof()` (which is `!hasData()`, a real 1-byte read-ahead, `SysFile.cpp:1288-1334`) and
`notreadyError()` otherwise. **`CHARIN` at EOF answers `ERROR:0`** because `readBuffer` (`:1076`)
calls `notreadyError()` directly — with `errorInfo` still 0 — when `SysFile::read` returns false on
a zero-byte read; `LINEIN` goes through `readVariableLine` → `checkEof()` → `eof()` (`:1341`).
A *short* `CHARIN` (fewer bytes than asked) is `eof(string)` (`:1536-1538`), hence `NOTREADY:EOF`
with the partial string as `RESULT`. Every `RexxMethodN` wrapper catches `StreamInfo*` and returns
the method's default result (`""`, `0`, `1` for `lineout`, `False`...), which is what the builtin
answers while the condition is pending (`:1546-1564` and siblings).

Each raise passes through `RexxThreadContext::RaiseCondition` → `NativeActivation::raiseCondition`
(`interpreter/execution/NativeActivation.cpp:2729-2737`) → `Activity::raiseCondition`
(`interpreter/concurrency/Activity.cpp:596-636`): first the `::OPTIONS NOTREADY SYNTAX` escalation
(`:619-621`, error `Error_Execution_notready_syntax`), then `checkCondition` walks the stack for a
frame that `willTrap` it and returns `false` — **no condition object is built at all** — when none
does (`:648-670`); otherwise `createConditionObject` (`:723-750`) builds the `Directory` with
`CONDITION`, `DESCRIPTION` (`""` when null), `PROPAGATED` = false, `RC` only if given (never, for
streams), `ADDITIONAL`, `RESULT`, plus `generateProgramInformation` (PROGRAM, PACKAGE, TRACEBACK,
STACKFRAMES), and `raiseCondition(dir)` delivers it to the first trapping frame. This is the
10-entry directory §2.8 measured.

### 3.3 Open

`implicitOpen(type)` (`:770-885`): std streams and handles reopen their own way; otherwise
`resetFields()`, `resolveStreamName()`, then **try `O_RDWR`** (`| O_CREAT` unless
`operation_nocreate`, which every *read* path passes) with `read_write = true`; on failure clear
the error and fall back to `O_WRONLY` (`write_only`) for a write operation or `O_RDONLY`
(`read_only`) for a read; if that fails too, `defaultResult` becomes `ERROR:%d` and
`notreadyError()` raises (state Error, `errorInfo` = the second `open`'s errno — this is the
`ERROR:2` a `LINEIN` on a nonexistent file shows, and why `LINES`/`CHARS` on one leave `ERROR:2`
after answering 0). Then, unless transient or read-only, the write position is set to `size()+1`
— or to `size()` when the last byte is `0x1A` (`:852-876`, the ctrl-Z overwrite) — and
`lineWritePosition` is zeroed (untracked). `readSetup()` (`:890-913`) implicit-opens with
`nocreate`, sets `state = Ready`, and re-seeks the fd to `charReadPosition` when the OS position
disagrees (this is how one fd serves two positions). `writeSetup()` (`:918-949`) implicit-opens for
write, raises `notreadyError(EACCES)` for a `read_only` stream (the `ERROR:13` on a read-only
write), sets Ready, and re-seeks to `charWritePosition` unless `append`.

`streamOpen(options)` (`:2262-2537`): closes if open, delegates std/handle streams,
`resetFields()`, parses options with the tokenizer (§2.2's table; `parser() != 0` → 93.0),
defaults to `RDWR|CREAT` BOTH; a `BINARY` with `O_TRUNC` and no `RECLENGTH` → 93.0 (`:2410-2413`);
no explicit mode → `RDWR|CREAT`, `read_write` (and `append` if `O_APPEND` was given); `read_only`
→ `fileExists` check, else `ENOENT` → `notreadyError` with `ERROR:2` as the result (`:2430-2444`,
the reason a failed `open read` answers `ERROR:2` and, through the builtin's table removal, leaves
`UNKNOWN`); **`WRITE` is rewritten to `RDWR|CREAT` with `write_only = true` and `read_write =
true`** (`:2447-2458`), which is why `linein` on an `open write` stream reads. `open()` failure
retries `WR_CREAT` for `write_only` or a device, else raises with `ERROR:%d` (`:2459-2484`).
`nobuffer` → `setBuffering(false)`. Then the same end-of-file write positioning as
`implicitOpen`, `state = Ready`, `checkStreamType()` (`:441-478`: transient flag from SysFile;
for `record_based` without `RECLENGTH`, the record length becomes the file size, **0 → 93.0** —
the "BINARY on an empty file" error; transient binary defaults the record to 1).

`handleOpen` (`:607-725`) accepts only READ/WRITE/BOTH/NOBUFFER/BINARY/RECLENGTH; `openStd`
(`:535-596`) maps the three names, honours only `NOBUFFER`, sets `qualified_name = stream_name`
(hence `query exists` → `STDOUT`) and `transient` from `fstat` (a redirected regular file is
`PERSISTENT`, a pipe `TRANSIENT`).

### 3.4 Reading

`charin(setPos, pos, len)` (`:1488-1541`): `readSetup`; `setCharReadPosition` (`:1217-1240`:
93.958 if transient, 93.907 if < 1, `eof()` if `> size()` — so `charin(f,99)` raises before reading;
otherwise `lseek`); `len == 0` → `""`; `readBuffer` (10,000-byte stack buffer or a
`RexxBufferString` above that); `resetLinePositions()` (`:1468-1473`: line positions and the
count cache to 0 — §2.6's anomaly); short read → `eof(string)`.

`linein(setPos, pos, count)` (`:1664-1732`): `count` not 0/1 → 93.0 **before** `readSetup`;
`setLineReadPosition` (`:1247-1263`: 93.958 transient, 93.907 < 1, then `setLinePosition` →
`seekToVariableLine` (`:3299-3314`) which rewinds to line 1 unless already at or before the target
and `readForwardByLine` (`:3257-3285`) counts newlines via `SysFile::seekForwardLines`
(`SysFile.cpp:859-935`, 512-byte blocks); a target beyond the last line stops at EOF and records
`stream_line_size`); `count == 0` → `""`; record-based → a fixed-size `readBuffer` with
`eof(string)` on a short record; else `readVariableLine` (`:1318-1367`): `SysFile::gets` byte by
byte into the 512-byte default buffer, doubling on overflow (`extendBuffer`, `:263-270`), stopping
after `\n`; `gets` (`SysFile.cpp:696-754`) turns `\r\n` into `\n` by peeking one byte and
`ungetc`-ing a non-`\n` — so only the CR immediately before an LF disappears and a bare CR is
data; the returned string drops the `\n`; a final unterminated line is returned when `gets` fails
with bytes in hand (`:1335-1339`). `lineReadIncrement` (`:1439-1461`) refreshes
`charReadPosition` from the fd, bumps `lineReadPosition` if tracked, and sets `last_op_was_read`.

`arrayin` (`:1768-1799`) loops `appendVariableLine` (or fixed records) until the EOF throw, which
`stream_arrayin` swallows (`:1805-1824`) — the array is filled in place, the raise still reaches
Rexx, and `arrayIn`'s `signal on notready` catches it.

`lines(quick)` (`:1834-1913`): implicit open; transient → `hasData() ? 1 : 0` (`FIONREAD` on a
tty or stdin, else a 1-byte read + `lseek(-1)`, `SysFile.cpp:1310-1333`); a stream that is neither
`read_only` nor `read_write` → 0; record-based → arithmetic; else `charReadPosition > size()` → 0,
`quick` → 1, cached `stream_line_size > 0 && lineReadPosition > 0` → `size - pos + 1`, otherwise
`countStreamLines(lineReadPosition, charReadPosition)` (`:3745-3765`: seek to the read position,
count newlines to the end, cache `count + lineReadPosition - 1`). **The anomaly**: after
`CHARIN` set `lineReadPosition = 0`, the cache is stored as `count - 1`, and the *next* call takes
the `cached - 0 + 1` branch only if `lineReadPosition > 0` — it is 0, so it recounts... measured
`2 1`: the second count is one short, consistent with the cache being consulted through
`countStreamLines`'s first line (`if (stream_line_size > 0) return stream_line_size;` — the cached
`count - 1`). A port that keeps this cache reproduces the defect; one that recounts every time
diverges on the second `LINES('C')` after a `CHARIN`.

`chars()` (`:1958-1981`): implicit open; transient → `hasData()`; not readable → 0; else
`size() - (charReadPosition - 1)`, floored at 0. `size()` is `SysFile::getSize`
(`SysFile.cpp:1064`, **not read**); measured (`p15`): `query size` right after a buffered
`lineout` of 4 bytes answers `5`, so whatever it does, unflushed bytes are counted.

### 3.5 Writing

`charout(data, setPos, pos)` (`:1576-1632`): with no string — a `read_only` stream and no
position → `close()`; `writeSetup()`; no position → `close()` and 0; else `setCharWritePosition`
(`:1270-1282`: 93.958 transient, 93.907 < 1, **no upper bound**) and 0. With a string:
`defaultResult = length` (the residual if the write fails), `writeSetup`, optional positioning,
`writeBuffer` (`SysFile::write`, then `charWritePosition = getPosition()+1`, `:1018-1036`), a
short write → residual and `notreadyError`, `resetLinePositions()`, return 0.

`lineout(data, setPos, pos)` (`:2016-2101`): no string → same close/position dance with
`setLineWritePosition` (`:1290-1309`, `setLinePosition` clamps to the last line + 1 through
`readForwardByLine`) and, for record-based, pads the current record (`completeLine`); with a
string: `writeSetup`, positioning, record-based → 93.0 if the string does not fit the rest of the
record, else `writeFixedLine` pads with blanks; variable → adjust `stream_line_size` (`++` when
appending at the end, else invalidate), `writeLine` = `SysFile::putLine` (data then `"\n"`,
`SysFile.cpp:661-678`), bump `lineWritePosition` if tracked. Always returns 0; the `1` a caller
sees is the wrapper's default when the `StreamInfo*` throw unwinds (`:2111`).

`SysFile::write` (`SysFile.cpp:517-629`): switching to write mode seeks the fd to the logical
read position first (`:537-539`); writes larger than the buffer flush and go straight through; an
`O_APPEND` unbuffered write seeks to the end first. `flush` (`:321-344`) is the only place buffered
bytes reach the fd besides `close`, `reset` and the read/write switch.

### 3.6 Positioning and queries

`streamPosition(options)` (`:2611-2844`): tokenizer table `=`/`<`/`+`/`-`, READ 1, WRITE 1, CHAR 1,
LINE 1, with `position_offset` as the fallback token (a number; a second number or a non-number
fails, `:162-181`); 93.958 if transient (**after** parsing); no offset → 93.903 with
substitutions `SEEK`, `offset`; `state = Ready`; neither READ nor WRITE → the stream's only mode,
or both plus a copy of the last-used position onto the other (`:2711-2747`) — the `last_op_was_read`
rule §2.12's `=1` case shows; implicit open; a READ seek clears the count cache; `<n` is
`SEEK_END` with offset `n` (so `<1` = size, `<0` = size+1, no check); `-n` negates; CHAR seeks with
`SysFile::seek` (**no range check**, negative results returned as measured), LINE goes through
`seekLinePosition` (`:2899-2939`, clamps below 1 to 1) and `setLinePosition`. A write-only stream
gets `return 0` for a LINE seek only when it is neither `read_write` nor `read_only`
(`:2809-2812`) — `open write` sets `read_write`, so the documented error never shows.

`queryStreamPosition` (`:3036-3160`): tokens SYS 1, READ 1, WRITE 1, CHAR 1, LINE 1 (`sys` is a
QUERY-only token, hence SEEK rejects it); not open → `""`; transient → `1`; SYS → `getPosition`;
default to WRITE only when `write_only`, else READ; LINE → `getLineReadPosition` /
`getLineWritePosition` (`:3169-3223`, which count lines from the top with `countLines(0, pos-1)`
when untracked and `+1` for the write side).

The other queries: `streamExists` (`:3400-3430`: handle streams `""`; open → `stream_name` for a
device else `qualified_name`; closed → `fileExists(qualified)`, and `fileExists` is false for a
directory), `queryHandle` (`""` unless open), `getStreamType` (`UNKNOWN` unless open),
`getStreamSize` (open → `fstat(fd)`; closed → `stat(name)` or `""`), `getTimeStamp` (`ctime` of
`st_mtime`, regular files only, else `""`), `qualify` (`resolveStreamName`, no open).

### 3.7 Close, flush, uninit

`streamClose` (`:2130-2141`): not open → `state = Unknown`, `""`; else `close()` (`:487-502`:
`SysFile::close` flushes, frees the buffer, closes the fd only if this object opened it, then
`isopen = false`, `state = Unknown`; a failing close raises with the errno as `RESULT`) and
`READY:`. `streamFlush` (`:2219-2231`): `SysFile::flush`, failure → `notreadyError` with
`ERROR:%d`, else `READY:` — on a closed stream `flush` is a no-op (`buffered` false or nothing
pending), hence `READY:`/`UNKNOWN`. `stream_uninit` (`:2177-2210`): close, destroy the
`StreamInfo`, drop `CSELF`, return nothing.

### 3.8 The tokenizer

`StreamCommandParser.cpp:56-99`: tokens are runs of non-blank characters, except that `=`, `+`,
`-`, `<` are single-character tokens and also terminate the run before them (`seek =2` and `seek
= 2` tokenize the same way; `1e1` is one token that `toNumber` rejects). `parser()` (`:121-169`)
finds the first table entry whose leading `len(token)` bytes match caselessly
(`StreamToken::equals`, `.h:98-101`), then requires `token length >= minlength`, then runs the
entry's `ParseAction` list — `MEB`/`ME` (fail if the bool/bit is already set: the "twice" and
"conflicting" 93s), `MIB` (fail unless set: RECLENGTH needs BINARY first), `SetBool`, `SetItem`,
`BitOr`, `CallItem` (`reclength_token` reads the next token as a non-zero number,
`:131-151`). Anything unmatched goes to the `unknown_tr` fallback, which fails (`:192-195`), or to
`position_offset` for SEEK.

### 3.9 The stream table and name resolution

`RexxActivation::resolveStream` (`RexxActivation.cpp:1938-2041`): an empty or omitted name →
`.INPUT` or `.OUTPUT` from the **local environment** (the Monitors); `STDIN[:]`, `STDOUT[:]`,
`STDERR[:]` caselessly → `.INPUT`, `.OUTPUT`, `.ERROR`; otherwise the name is qualified
(`Interpreter::qualifyFileSystemName`, with a per-activation `fileNames` alias table only on
case-insensitive file systems) and looked up in the activation's `StringTable` by the **qualified
name**; a miss asks the security manager (`checkStreamAccess`), else sends `NEW` to the `Stream`
class with the **unqualified** name (so `~string` is as written) and, only when the caller passed
`&added` (every builtin except `STATE`, `DESCRIPTION` and non-OPEN/CLOSE/SEEK commands), stores it.
So `stream(n,'S')` on an unknown name builds a throw-away object each time and never populates the
table — the reason a `query exists` from `STREAM` never leaves a table entry behind.

`getStreams` (`:2049-2082`): a **program or method** activation creates its own table; any other
kind (internal call, `PROCEDURE`, `::ROUTINE`, `INTERPRET`) borrows its caller's table by reference
when the caller is a Rexx frame, else creates one. `closeStreams` (`:4691-4708`), run from the
activation's termination (`:1502`) only for program/method frames, sends `CLOSE` to every entry —
the end-of-program flush. `removeFileName` (`:5117-5125`) is the builtin's table removal.

### 3.10 Errors raised, by number

| number | text | where |
|---|---|---|
| 40.3 / 40.4 | argument count | `fix_args`/`check_args` in the builtins |
| 40.12 | `X argument N must be a whole number; found "…"` | `optional_big_integer` |
| 40.27 | `STREAM argument 1 must be a valid stream name; found ""` | `BuiltinFunctions.cpp:2444` |
| 40.904 | `STREAM argument 2 must be one of SDC` / `LINES argument 2 must be one of CN` | `:2458,2566,2362` |
| 48.1 | `Failure in system service: Stream not initialized.` | `checkStreamInfo`, `StreamNative.cpp:111` |
| 88.901 / 88.907 / 88.922 | native-method argument checks, counting the native method's own args | the `RexxMethodN` marshalling |
| 91.999 | `Message "UNINIT" did not return a result.` | a value-context send of `uninit`/post-uninit `close` |
| 93 (bare, message `.nil`) | option/parse failures | `raiseException(Rexx_Error_Incorrect_method)` and the orx `raise syntax 93` |
| 93.901 / 93.902 / 93.903 | `use strict arg` counts; `Missing argument in method; argument SEEK is required.` | orx; `StreamNative.cpp:2702` |
| 93.907 | `Method argument 1 must be a positive whole number; found "0".` | `:1226,1256,1278,1301` (always "argument 1") |
| 93.914 | `Method argument 1 must be one of CLOSE FLUSH OPEN POSITION QUERY SEEK; found "…"` | `StreamClasses.orx:283` |
| 93.915 | `Method option must be one of "CN"` / `"CL"` | `StreamNative.cpp:1929`; `StreamClasses.orx:222` |
| 93.937 | supplier exhausted | `StreamClasses.orx:400,424,432` |
| 93.938 | `Method argument 1 must have a string value.` | `StreamClasses.orx:164` |
| 93.958 | `Positioning of transient streams is not valid.` | `Rexx_Error_Incorrect_method_stream_type`, `:1221,1251,1274,1295,2696` |
| 93.963 / 93.965 | unsupported / abstract mixin methods | `StreamClasses.orx:56-100` |
| 97.1 | `does not understand message "MAKESTRING"` | no such method on `Stream` |

## 4. The crate today

All **Read** at `5bcb28edb` unless marked Measured (the snapshot binary `bins/h-5bcb28edb`).

**What refuses, and how** (Measured, every probe in §2):

| construct | stderr line | rc |
|---|---|---|
| `lineout(...)`, `linein`, `charout`, `charin`, `chars`, `lines`, `stream`, `qualify` | `rexx-exec: routine "LINEOUT" is not implemented (4c)` (the name varies) | 120 |
| `.Stream~new(...)` | `rexx-exec: the LIBRARY REXX entry point "stream_init" is not implemented (Phase 7)` | 120 |
| `.stdout`, `.output`, `.input` (and by the same table `.stderr`, `.error`, `.stdin`) | `rexx-exec: environment symbol ".STDOUT" is not implemented (Phase 7)` | 120 |
| `.local['STDOUT']` | `rexx-exec: directory entry ... is not implemented (Phase 7)` (asserted by `crates/rexx-exec/src/dispatch.rs:10128-10140`) | 120 |
| `.File~new('x')` (any argument) | `... entry point "file_qualify" is not implemented (Phase 7)` (survey B's area; `p10b` hit it through `.Stream~new(.File~new(...))`) | 120 |

Output written before the refusing clause survives (`p09`: stdout `say1\n`, then the CHAROUT
refusal), which is the buffered-sink design working as intended.

**Where** (Read):

* The excluded builtins: `rust/crates/rexx-exec/src/run.rs:3547` — `None if
  builtin::is_excluded_builtin(name) => return Err(Loud::unresolved_call(name).into())`;
  `is_excluded_builtin` is `rexx_inventory::builtins::wholly_excluded()` (`builtin.rs:552-564`);
  `Loud::unresolved_call` (`lib.rs:273-283`) hardcodes the owner `Some("4c")` — D-P7-5's fix
  point. Nothing asserts that owner (`tests/owners.rs` covers instruction kinds, and lists
  `"Phase 7"` among `SPLIT_TABLE_PHASES` at `:334` with `Command`/`Address::Command` owned by
  Phase 7 at `:65,:170`).
* The native registry: `rust/crates/rexx-exec/src/dispatch/native.rs:33-63` (`Family::{Timer,
  Stream, Queue, File}`, `owner()` answering `"Phase 7"` for `Stream | Queue | File` at `:51` —
  D-P7-3 moves `Queue` off it); the 24 `deferred("stream_*"|"query_*"|"qualify"|"handle_set"|
  "std_set", Family::Stream)` rows at `:119-142`, matching `StreamClasses.orx:127-147,179,180,242,
  289` one for one; `deferred_send` at `:372-379` produces the refusal above; the dispatch site is
  `dispatch.rs:2102-2103` (`ExternalBody::Deferred` → `Err(deferred_send(entry))`,
  `ExternalBody::Implemented { arity, run }` → the seam an implementation plugs into).
* `tests/native_entries.rs:115-171` runs one probe per family from
  `corpus/gate-tables/native-entries/<family>.rex`, asserts stdout `BEFORE_THE_SEND`, the exact
  refusal line and exit 120, **and asserts the entry point named by the probe is still deferred**
  (`:132-137`) — so implementing `stream_init` reddens `stream.rex` there until the probe is
  re-pointed or the test is retired; `:174-190` pins the implemented set to a constant
  `IMPLEMENTED`. `corpus/method-bodies.txt:1235-1243` holds `StreamSupplier`'s rows as
  `unanswered` (a bare `~new` raises), `:1348` `InputOutputStream arrayOut answers rc 163`, and
  `corpus/gate-tables/methods/{stream,streamsupplier,inputstream,outputstream,inputoutputstream}__*.rex`
  are the D-P7-4 gate rows that move `loud` → `answers`.
* `StreamClasses.orx` is embedded by `rust/crates/rexx-lib/build.rs:23-24` from
  `../../../interpreter/RexxClasses/StreamClasses.orx` (1010 lines, sha-checked) and run by the
  bootstrap's `call 'StreamClasses.orx' rexxPackage` (`rexx-lib/src/lib.rs:19-21`), so the Rexx
  halves of `Stream` (`init`, `command`, `query`, `arrayIn`, `arrayout`, `makearray`, `say`,
  `supplier`, `StreamSupplier`) and the three mixins already exist and dispatch — every
  `EXTERNAL 'LIBRARY REXX …'` method binds to a deferred entry (`dispatch.rs:8239`).
* The sinks: `Interp.out` / `Interp.trace` (`lib.rs:1692-1694`), `SAY` appends at `run.rs:2079`,
  `Outcome { stdout, stderr, .. }` (`lib.rs:159-162`) and `rexx-run` writes both at exit
  (`crates/rexx-exec/src/bin/rexx-run.rs:56-58`). There is no third sink; nothing writes to a
  file today.
* Input: `crates/rexx-exec/src/input.rs:35-105` — `Input { source: Nothing | Stdin(std::io::Stdin)
  | Bytes(Cursor<Vec<u8>>) }` from `ProgramInput`, one `read_line` (drops `\n` and a `\r` before
  it, matching §2.5's rule) behind `pull_line` (queue first) and `linein_line`. It is line-only:
  `CHARIN`/`CHARS` on `.STDIN` need a byte-level read on the same cursor.
* Native state: `rust/crates/rexx-core/src/body.rs:114` `Body::Instance { native:
  Option<Box<BufferState>> }`, `BufferState { bytes, capacity, default_size }` at `:152-157`,
  reached through `Interp::buffer`/`buffer_mut` (`crates/rexx-exec/src/value.rs:404-425`, a
  `match` on `Body::Instance { native: Some(state), .. }`) and stored by the `MutableBuffer`
  constructor at `dispatch.rs:6601-6605`. The collector does not look inside it (no `ObjRef`s).
  `UNINIT`: `Heap::collect` resurrects unreachable objects with `has_uninit` into
  `pending_uninit` (`rexx-core/src/heap.rs:153-176`), which `lib.rs:4498` moves to
  `Interp.uninit_ready` for delivery.
* Conditions: `PendingTrap { condition, rc, description, activation, queued_during_delivery,
  fragment_depth }` (`lib.rs:1452-1470`) is the `CALL ON` queue and `deliver_one_pending_trap`
  (`run.rs:2992`) delivers it; there is no `additional`/`result` slot, so a NOTREADY's `O`
  (the stream) and `RESULT` have nowhere to ride yet. `::OPTIONS NOTREADY SYNTAX` already parses
  (`options.rs:26,77,115,149,180,245,290,463`, `escalated[4]`) and `run.rs:2827`'s
  `condition_raises_syntax(b"LOSTDIGITS")` is the escalation pattern to copy.
* No shadow current directory exists: the only `current_dir` use is `lib.rs:2977`
  (`::REQUIRES` resolution through `require::normalize`). Fact 2 of the brief is a requirement,
  not a description of the tree.
* `.local` names: `environment.rs:163-174` lists `ERROR`, `INPUT`, `OUTPUT`, `STDERR`, `STDIN`,
  `STDOUT` (and `DEBUGINPUT`, `STDQUE`, `SYSCARGS`, `TRACEOUTPUT`) as oracle entries this crate
  does not build; each currently resolves to the Phase 7 refusal.

**Seams a Phase 7 implementation plugs into**: (1) `ExternalBody::Implemented { arity, run }` for
each of the 24 entries, with `run` receiving the receiver `ObjRef` and args; (2) the
`Body::Instance.native` slot, widened (§5.1); (3) a stream table on the activation (§5.5) and a
`resolve_stream` helper the eight builtins call before sending the same-named message — which
means the builtins become message sends to `Stream` objects exactly as in C++, and the Rexx halves
in `StreamClasses.orx` do the rest; (4) `Loud::unresolved_call`'s owner; (5) `PendingTrap` plus
whatever `SIGNAL ON` delivery `RAISE` uses, extended with `additional` and `result`; (6) the
`.local` construction for the six std names; (7) `Input` for `.STDIN`.

## 5. Design questions

### 5.1 The Rust representation of a stream, and the `native` slot

Widen `Body::Instance { native: Option<Box<BufferState>> }` to `Option<Box<NativeState>>` with
`enum NativeState { Buffer(BufferState), Stream(StreamState) }` (`rexx-core/src/body.rs:114`).
`Interp::buffer`/`buffer_mut` (`value.rs:404-425`) gain a `NativeState::Buffer` arm and a
`stream`/`stream_mut` pair beside them; the `MutableBuffer` constructor at `dispatch.rs:6601`
wraps its state. `NativeState` holds no `ObjRef`, so `Trace` is unaffected and the collector never
looks inside; `Body::Instance`'s size is unchanged (still one `Box`). Alternative: a second
`Option<Box<StreamState>>` field — costs a word on every instance for the benefit of nothing;
rejected. Alternative: a side table `HashMap<ObjRef, StreamState>` keyed by object — breaks under
compaction/moves and needs its own uninit hook; rejected.

`StreamState` (all `Read` from `StreamNative.hpp:158-192`, keep the oracle's names):

```rust
pub struct StreamState {
    name: Vec<u8>,                       // as given; `~string`
    qualified: Option<PathBuf>,          // lazily `qualify`'d against the shadow cwd
    kind: Kind,                          // Closed | File(std::fs::File) | Std(Std) | Handle(u32)
    state: State,                        // Unknown | Ready | Eof | Error(i32 errno)
    read_only: bool, write_only: bool, read_write: bool, append: bool, nobuffer: bool,
    stdstream: bool, opened_as_handle: bool, transient: bool, record_based: bool,
    last_op_was_read: bool, isopen: bool,
    char_read: i64, char_write: i64, line_read: i64, line_write: i64,
    line_read_char: i64, line_write_char: i64, stream_line_size: i64,
    reclength: usize,
    rd: ReadBuf,                         // read-ahead buffer + one unget byte, for `gets` and `hasData`
    wr: Vec<u8>,                         // pending output (see 5.3)
}
```

`State::Notready` is omitted on purpose (§3.2: never assigned). `DESCRIPTION` is derived exactly
as `getDescription` does, with `strerror` text from
`std::io::Error::from_raw_os_error(errno).to_string()` minus its ` (os error N)` suffix (the text
comes from the same libc as the oracle's on the same machine; other platforms differ and are a
CI-baseline matter).

**`UNINIT` and the collector**: the oracle's `stream_uninit` closes and then destroys the state
and drops `CSELF`, after which the guarded methods raise 48.1 and the unguarded ones crash (§2).
Port: `stream_uninit` closes, then sets `native = None`; every entry point that finds `None`
raises 48.1 `Stream not initialized` — including the eight that crash on the oracle, which is a
documented, licensed divergence (a crash is not an answer). `Heap::collect`'s
resurrect-then-deliver path (`heap.rs:153-176` → `lib.rs:4498`) already sends `UNINIT`, and
`Stream`'s `uninit` is a real method on the class, so the collector reaches the file close through
the ordinary send with no new hook. **Do not rely on `Drop` of `std::fs::File` for correctness**:
a `File` drop closes the fd but never writes `wr`; flush must happen in `close()`, and the
end-of-program sweep (5.5) must run on every exit path including errors (`p09b`) — `Interp`'s
own drop can run a last "flush every open stream" pass as a belt, but the order of those flushes
is a hash order on the oracle and unobservable, so any order is fine.

### 5.2 Std streams and the sinks

`.STDOUT` and `.STDERR` are `Stream` instances whose `kind` is `Std(Stdout)`/`Std(Stderr)` and
whose writes append to `Interp.out` / `Interp.trace` respectively (fact 1); they are never
buffered locally (the oracle's are `setBuffering(false)` too), so `SAY`, `lineout('STDOUT')`,
`.stdout~charout` and `lineout(,…)` interleave in program order by construction — `p09`'s byte
string is the witness. `.STDIN` is `Std(Stdin)` and reads through `Interp.input` (fact 3): extend
`Input` with `read_bytes(n)` and `has_data()` on the same cursor; `Bytes` answers exactly (`p08`'s
`charin(,,3)` → `l5a` then `linein()` → `bcdef`); `Nothing` answers EOF; the real `Stdin` variant
blocks for `read_bytes` and, for `has_data`, cannot use `FIONREAD` without `unsafe` — answer 1
without peeking ("a determination cannot be made", `iostrms.xml:191-193`), which diverges from
the oracle only for an exhausted real stdin, which no harness runs. `query streamtype` answers
`TRANSIENT` for Std always (the oracle says `PERSISTENT` when the descriptor is a regular file,
`p08` — the harness compares a pipe, `p08c`/`p08e`, where both say `TRANSIENT`; on a redirected
file the crate would diverge, recorded here, and `std::io::IsTerminal` plus
`fs::metadata("/dev/stdout")` could close it later without `unsafe`). Positioning on any Std
stream → 93.958 as measured.

**Name routing**: `resolveStream` maps `""`/`STDIN`/`STDOUT`/`STDERR` to the **Monitors**
`.INPUT`/`.OUTPUT`/`.ERROR`, which D-P7-1 leaves loud. Route them straight to `.STDIN`/`.STDOUT`/
`.STDERR` instead. Observable difference: only the traceback of an *uncaught* error inside such a
send, where the oracle prints an extra `Method UNKNOWN with scope "Monitor" in package "REXX" (no
source available).` line (`p08d`, stderr). That is one line of one uncaught-error transcript on
the std streams; record it as an accepted divergence or have survey C's Monitor forward it. Also
`.Stream~new('STDOUT')` must be a distinct object from `.stdout` (`p10b`: `== .stdout` is 0, state
`UNKNOWN` until used) — `std_set` marks it and its writes still go to `Interp.out`.

Closing a std stream: `stream('STDOUT','C','close')` → `READY:`, state `UNKNOWN`, later writes
reopen and continue (`p08c`); the crate's std kind never closes a real fd so this is free. The
`charout('STDOUT')` silence (§2.10) is **not** understood and should not be reproduced until it is.

### 5.3 Buffering and flush points

Files: write-through (`File::write_all` per `charout`/`lineout`) in the first cut, no local
write buffer — every measured observable is identical (`query size` counts unflushed bytes on
the oracle, `query position sys` reports the logical position on a buffered stream, both of
which write-through answers for free), the ctrl-Z/append logic does not depend on it, and the
end-of-program sweep then only has to *close*. Cost: one syscall per write; `bench-programs/` has
no I/O axis and ooTest is not I/O-bound; add a `BufWriter` behind `flush()` later if measured.
Reads: a 4096-byte read-ahead with one unget byte (`ReadBuf`), because `gets`'s CR peek and
`hasData`'s one-byte probe need it and because `linein` byte-by-byte through `read(2)` would be
the real perf hazard. Switching from reading to writing (or seeking) discards the read-ahead and
re-seeks the fd to the logical position, exactly `SysFile::write:537-539` / `readSetup:907-911`.

Flush points that matter for byte-identity: `close` (explicit, `LINEOUT(name)`, `CHAROUT(name)`,
`uninit`), the activation-end sweep (5.5), and interpreter exit on every path. With write-through
none of them can lose bytes; with a later `BufWriter` all three plus `flush` and every
read-after-write switch must drain it.

### 5.4 NOTREADY

Raise from native code with `(description = name as given, additional = the stream object,
result = the answer the call is returning)`; build the 10-entry directory of §2.8 (no `RC`); when
no frame traps it, build nothing and return the default result (`checkCondition`, §3.2); `SIGNAL
ON` unwinds the clause; `CALL ON` queues a `PendingTrap` with two new fields `additional:
Option<ObjRef>` and `result: Option<ObjRef>` (both must be rooted while pending — the trap queue
is already traced for `description`? it holds `Vec<u8>`, not an `ObjRef`, so this is the first
`ObjRef` in that queue; put it on the root set the way `roots.push_temp` does or make
`PendingTrap` `Trace`). `::OPTIONS NOTREADY SYNTAX` → the existing `escalated[4]` gate; the
error number `Error_Execution_notready_syntax` was not measured (§7). Costs if wrong: the
condition-object shape is compared byte for byte by any program printing `condition('O')`
entries, and ooTest's stream tests do.

### 5.5 The stream table and the shadow cwd

Per **program or method** activation: `streams: HashMap<Box<[u8]>, ObjRef>` keyed by the
qualified path, traced (the activation is a root); internal-call, `PROCEDURE`, `::ROUTINE` and
`INTERPRET` frames resolve through their nearest program/method ancestor (store the owning
`ActivationId`, walk up), which is the by-reference sharing `p12b` measured. Builtins that pass
`&added` insert on miss; `STATE`/`DESCRIPTION`/non-OPEN/CLOSE/SEEK commands do not; `LINEOUT(name)`
alone, a `CLOSE` command, and an `OPEN` command answering other than `READY:` remove the entry.
On program/method activation exit send `CLOSE` to every entry (order unobservable). `.Stream~new`
never touches the table.

`Interp.cwd: PathBuf`, initialised from `std::env::current_dir()` once per `Interp`; `qualify`
joins relative names onto it and normalises lexically (`..`, `//`, trailing `/`, `~`/`~/` →
`$HOME`); `~user` unexpanded and recorded (§7). Every `File::open` uses the qualified absolute
path, so the process cwd is never read after startup and never written (fact 2). If a harness
changes the process cwd between runs, nothing here notices, which is the point.

### 5.6 `unsafe`, dependencies, platform

No `unsafe` is needed for the recommended design: file type via `std::os::unix::fs::FileTypeExt`
(`is_char_device`, `is_fifo`) and `std::io::IsTerminal`; size/mtime via `fs::metadata`; errno via
`io::Error::raw_os_error`; directory detection via `metadata.is_dir()` before `open` (the
oracle opens then `fstat`s, same answer `ENOENT`). `HANDLE:n` needs `File::from_raw_fd` (unsafe)
— **refuse it loudly** (`handle_set` deferred with a message); the oracle segfaults on the only
interesting use anyway (§2). `FIONREAD` (unsafe `ioctl`) — not used (5.2).

One new dependency is recommended: **`chrono` 0.4.x** (present in the offline cache: 0.4.41,
0.4.44, 0.4.45) for `query_time`'s local-time `ctime` string (`Thu Nov 12 03:29:12 2015\n`, which
`StreamClasses.orx:307-346` reparses in Rexx — keep that shape so the Rexx half stays untouched);
`std` has no local-time conversion. Alternative: parse `TZ` ourselves — no. `libc` is also present
(0.2.169–0.2.185) but every use of it is `unsafe`. The cache listing I took was cut at 20 lines,
so `nix`/`rustix`/`os_pipe` presence is unverified; none is needed.

Platform: line terminator is `\n` on unix (`SysFile.hpp:89 LINE_TERMINATOR "\n"`); the Windows
CI baseline will differ on `LINEOUT` bytes, CRLF stripping is the same code; `strerror` texts
differ per libc (macOS/BSD say `Permission denied` too; `ERROR:2 No such file or directory` is
universal). The ctrl-Z rule applies on every platform (`StreamNative.cpp:2520`).

### 5.7 The `LINES` count cache

Port `stream_line_size`/`lineReadPosition`/`lineReadCharPosition` and the exact reset points
(`resetLinePositions` on every `charin`/`charout`, the READ-seek clear, the `lineout` adjust);
this reproduces the `2 1` / `7 … 6` off-by-one (§2.6) byte for byte. Cost if the port "fixes"
it: the second `LINES('C')` after a `CHARIN` diverges, and ooTest's stream tests exercise
`lines('C')` in loops. If Moritz prefers a documented divergence here, it is one line in the
accepted-divergence record; the recommendation is parity, because the cache is also what makes
`lines('C')` O(1) on the second call.

### 5.8 Owner message (D-P7-5)

`Loud::unresolved_call` takes its owner from the same inventory `wholly_excluded()` comes from
(a `(name, owner)` pair per excluded builtin); `tests/owners.rs` asserts every excluded name has
an owner in `SPLIT_TABLE_PHASES ∪ {"Phase 10"}` and that the eight stream names say `Phase 7`.
When Phase 7 implements them the refusal disappears with the row, which is the assertion's
negative control.

## 6. Proposed task slices

Each slice is independently testable against the oracle; the witness programs are the `$P/pNN`
probes of §2 with the named lines removed (the crash clauses, the `HANDLE:` lines, the `~`
expansions that depend on `$HOME`, and the STDOUT-to-a-file overwrite artifacts), re-homed under
`rust/corpus/streams/` with their fixtures created by the program itself (a corpus program cannot
rely on a pre-made `subdir/` — it must `lineout` its own files; the unreadable-file case needs a
`chmod`, which needs `ADDRESS`, survey D — so it stays a test-harness fixture rather than a corpus
program). Every witness is compared on all three descriptors.

| # | slice | witnesses (must match byte for byte) | paired negative / adjacent case |
|---|---|---|---|
| 1 | Owner message: `Loud::unresolved_call` reads the owner from the inventory; `owners.rs` asserts it (5.8) | the refusal lines of §4 now say `Phase 7` for the eight names | `RXQUEUE`/`RXFUNC*` say `Phase 10`; `USERID`/`SETLOCAL`/`ENDLOCAL` say `Phase 7` (survey D) |
| 2 | Shadow cwd + `QUALIFY` + `qualify` entry point (5.5) | `p13` minus the `~`, `$HOME` and `HANDLE:` lines; `.Stream~new('a.txt')~qualify == qualify('a.txt')` | `qualify()` 40.3, `qualify('')` `""`, `qualify('a','b')` 40.4; `qualify('/abs/../x/y')` = `/x/y` with no such path |
| 3 | `Stream` skeleton: `stream_init`, `std_set` (mark only), `handle_set` (loud), `~string`, `state`/`description` on a closed stream, `query_exists/size/time/streamtype/handle`, `close` on unopened, `flush` on unopened, `uninit` (5.1) | `p10` lines 1-4, `p01`'s "closed queries" block, `p06`'s `nosuch` query lines, `p10c`'s 48.1 rows | `.Stream~new` 93.901, `.Stream~new(.nil)` 93.938, `~state('x')` 88.922, `~makestring` 97.1; a directory answers `query exists` `""` |
| 4 | `stream_open`/`stream_close` and the option tokenizer (§3.3, §3.8) | `p02`, `p02b`, `p16` (with the file re-created before the BINARY rows), `p17` rows 1-10, `p10d`'s `open answers` | every 93.0 row of `p02b`; `open write binary` on a new file 93; `open read` on nosuch `ERROR:2` and the table drop (`UNKNOWN` after) |
| 5 | `charin`/`charout`/`linein`/`lineout`/`chars`/`lines` on files: positions, EOF vs `ERROR:0`, CR/LF, ctrl-Z, buffer growth (§3.4-3.5, 5.3, 5.7) | `p01`, `p04b`, `p05`, `p15`, `p17` rows 11-20, `p10` lines 5-12 | `p11`'s 40.12/88.907/93.907/93.0 rows; `lineout('e.txt','')` size 1 vs `charout('e2.txt','')` size 0; `lines('e5.txt')` on nosuch → 0 + `ERROR:2`, file not created |
| 6 | NOTREADY: raise, trap, condition object, `CALL ON` delay, `::OPTIONS NOTREADY SYNTAX` (5.4) | `p07b`, `p14`, `p07` up to and excluding the double-raise clause | untrapped continues (`p01`); `lines` at EOF does not raise but `lines` on nosuch does; `~arrayin` never lets it out |
| 7 | SEEK/POSITION/QUERY POSITION incl. LINE mode, `last_op_was_read`, the transient 93.958 (§3.6) | `p03`, `p03b` (minus the `/dev/null` clause, which is uncaught), `p15` line-position rows, `p16`'s seek/query abbreviation blocks | `seek write` 93.903, `seek <99` → -69, `seek 999` then `charin` `ERROR:0`; `query position` on unopened `""` |
| 8 | The activation stream table, the eight builtins as sends, `STREAM(...)`'s S/D/C, `LINEOUT(name)` close, end-of-program close, `LINES` Normal default (§3.1, §3.9, 5.5) | `p12`, `p12b`, `p09b` (file contents + rc 214 + the two-line error), `p09c`, `p11`'s STREAM/LINES rows | `stream('')` 40.27, `stream(' ')` UNKNOWN, `stream('a','C')` 40.3, `stream('a','S','x')` 40.4; a method call does **not** advance the caller's stream, a routine does |
| 9 | `.STDIN`/`.STDOUT`/`.STDERR` objects, name routing, `Input` byte reads, `.local` entries (5.2) | `p18`, `p09`, `p08c` and `p08e` (pipe form: run with `ProgramInput::Bytes`), `p08`'s stdin lines 5-20 | 93.958 on any std positioning; `lineout('STDIN','z')` → 1 `ERROR:13`; `.Stream~new('STDOUT') == .stdout` → 0; `linein('STDOUT')` → `""` `NOTREADY:EOF` |
| 10 | `arrayin`/`makearray`/`arrayout`/`supplier`/`say`/`DO OVER` (`stream_arrayin` native, the rest is the orx) | `p10` lines 13-24, `p10b` lines 1-5 | `~arrayin('q')` 93.915, `~arrayout(.array~of('a'),'q')` 93.0, `~linein(1,2)` 93.0, supplier `~next` past the end 93.937 (`p07b`'s void rows re-run with `expose`) |
| 11 | Error paths: directory, unreadable, missing parent, `/dev/null`, `/dev/zero` (§2.7) | `p06` (unreadable needs a harness `chmod`), `p10b` lines 6-8 | `open read` on a directory leaves `UNKNOWN`, `~open('read')` on a held object leaves `ERROR`; `lineout` to a directory → 1 `ERROR:21` |
| 12 | Gate: `corpus/method-bodies.txt` `Stream`/`StreamSupplier`/mixin rows `loud` → `answers` (D-P7-4); retire or re-point `native_entries.rs`'s `stream.rex` | the D-P7-4 rows | the post-`uninit` crash rows are **not** witnesses — the crate answers 48.1 where the oracle dies, licensed |

Order: 1 and 2 are independent of everything; 3 → 4 → 5 → 6 → 7 are sequential (each needs the
last); 8 needs 5; 9 needs 3 and 8; 10 needs 5 and 6; 11 needs 4 and 6; 12 last. Slices 5 and 7
are the bulk.

## 7. Not done / not established

* **Three oracle crashes found, none filed, none in `rust/corpus/oracle-crashes.txt`** (read-only
  for me). Recommended entries: (10) `lineout('HANDLE:1','x')` — SIGSEGV rc 139, deterministic
  (3 of 3 program shapes with a write; the five read/query shapes answer); (11) `s = .Stream~new(f);
  s~uninit; s~state` — SIGSEGV via the unguarded `CSELF` in `stream_state` (`StreamNative.cpp:3609`;
  `description`, `qualify` and the five `query_*` natives share the shape, inferred, not run); (12)
  an entry-5 variant: two NOTREADY-raising calls in one clause under `CALL ON NOTREADY`
  (`p07`), which is entry 5's "same `CALL ON` condition twice in one clause" with a different
  condition name — worth a sentence there rather than a new entry.
* `charout('STDOUT')` (a close through the no-argument form) silenced every later `SAY` and a
  later `lineout('STDERR')` in the pipe run (`p08c`), while `stream('STDOUT','C','close')` did
  not. The code read here (`SysFile::close` skips `::close` for std handles) does not explain
  it; not reproduced a second time; not to be ported until understood.
* `HANDLE:n` streams beyond the crash and the five safe answers; `QUEUE:` (Phase 10); `SHARED`/
  `SHAREREAD`/`SHAREWRITE` across processes (single process, no observable); `~user` tilde
  expansion; `::OPTIONS NOTREADY SYNTAX`'s error number and text; `LINES`/`CHARS` on a TTY;
  `charin` above the 10,000-byte local buffer; `SysFile::getSize` (only its measured answer);
  Windows/macOS/BSD line-end and `strerror` differences; the security manager's
  `checkStreamAccess` (D12, `RexxActivation.cpp:2016-2022` — a manager can substitute the stream
  object; not measured); whether a subclass `uninit` at process end runs from a collection or the
  shutdown sweep.
* The doc claims not reproduced: "line positioning in a write-only file gives an error message"
  (`streamclasses.xml:877`, measured `2`); `LINEOUT`'s `line` "cannot be beyond the end for
  non-binary streams" (`funct.xml:3208`, measured a clamp, no error); `QUERY DATETIME` "returns
  `""` for a transient stream" holds only through the orx `STREAMTYPE` test — on STDIN (a file
  here) it answered the current time.
* The `LINES('C')` cache off-by-one is measured on three shapes and explained from one code path
  (`countStreamLines:3748-3750` returning a stale `count - 1`); the exact sequence of cache writes
  for the `2 1` case was not stepped through — a port that copies the fields and the reset points
  reproduces it regardless, which is why §5.7 recommends copying rather than understanding.
* Method signatures were taken from the orx `use strict arg` lines and the `RexxMethodN`
  declarations, not from the reference's SVG diagrams, which are not machine-readable here.
* Not measured on the crate beyond the refusals: nothing stream-shaped runs there, so every
  behavioural claim in §2 is oracle-only by construction.

<!-- SURVEY COMPLETE -->
