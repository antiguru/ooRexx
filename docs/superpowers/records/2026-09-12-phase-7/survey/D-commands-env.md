# Phase 7 survey D — commands, `ADDRESS`, environment, platform builtins, security manager hooks

Surveyor D. Measured 2026-09-11 against the 5.3 oracle at `/home/moritz/dev/repos/ooRexx/build`
and the crate snapshot `h-5bcb28edb`. Probe root:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/survey-D/pNN/`.

Every claim is marked **Measured** (probe run, three descriptors), **Read** (`file:line`), or
**Inferred**.

## 1. Documented surface

Enumerated from the reference's own section list (`oodocs/rexxref/en-US/`): `instrc.xml`'s
`keyAddress` section, `intro.xml`'s `cmds`, `cmdhost` (`xenvir`, `commnds`) sections,
`condtra.xml`'s ERROR/FAILURE entries and its `spec` (RC) section, `spvard.xml`'s RC entry,
`funct.xml`'s `bifAddress`, `bifDirectory`, `bifEndlocal`, `bifQualify`, `bifSetlocal`, `bifUserid`,
`bifValue` (+ `bifBValue-OSenvironment`, `bifBValue-globalenvironment`) sections, and `secman.xml`.
All rows are **Read**.

| Item | Doc | Contract |
|---|---|---|
| Command clause | `intro.xml:2349`, `:3116-3125` | A clause that is only an expression (and not a message instruction) is evaluated to a string and submitted to the current ADDRESS environment; the environment's return code lands in `RC` |
| ERROR / FAILURE from a command | `intro.xml:3146-3167` | The environment may flag an error or a failure; traced under `TRACE E`/`TRACE F`; `TRACE N` = `TRACE F` is the default |
| `.RS` | `intro.xml:3168-3175` | -1 after a FAILURE, 1 after an ERROR, 0 after neither |
| Default environment and aliases | `intro.xml:3067-3073`, `instrc.xml:189-224` | `sh` on unix; `""`, `COMMAND`, `SYSTEM` are aliases |
| Alternate shells | `intro.xml:3074-3078`, `instrc.xml:198-209` | `bsh`, `bash`, `csh`, `ksh`, `tcsh`, `zsh` run the command through a shell of that name; a missing shell raises FAILURE |
| `PATH` environment | `intro.xml:3080-3089`, `instrc.xml:225-233` | No shell: the command is split and executed from `PATH`; no redirection, piping, substitution or wildcards |
| `ADDRESS env expr` | `instrc.xml:170-188` | Temporary change for one command; `RC` and `.RS` set; errors/failures trapped or traced |
| `ADDRESS env` | `instrc.xml:411-415` | Lasting change; the previous environment is saved |
| `ADDRESS VALUE expr` / `ADDRESS (expr)` | `instrc.xml:430-444` | Lasting change to the evaluated name; VALUE may be omitted when the expression starts with a special character |
| bare `ADDRESS` | `instrc.xml:445-450` | Swaps the current and the saved environment; `""` means the default |
| saved across calls | `instrc.xml:451-454` | Both names are saved and restored across internal and external subroutine and function calls |
| `ADDRESS ... WITH` | `instrc.xml:236-248` | Redirects the command's stdin/stdout/stderr from/to Rexx objects; permanent when no command is given (then re-evaluated per command in the then-current variable context), temporary otherwise; permanent redirection is attached to the environment name |
| `WITH INPUT NORMAL / STEM / STREAM / USING` | `instrc.xml:250-287` | stem: `stem.0` lines; stream: name read with `lineIn`; using: String (one line), Stem, InputStream, Monitor, File, Array (empty items ignored) or anything with `makeArray` |
| `WITH OUTPUT` / `WITH ERROR` `NORMAL / [REPLACE\|APPEND] STEM / STREAM / USING` | `instrc.xml:289-350` | stem REPLACE (default) sets `stem.0` and `stem.i`; APPEND continues from `stem.0`; stream REPLACE truncates; using: Stem, OutputStream, Monitor (no REPLACE/APPEND), RexxQueue (`queue`, no options), File, OrderedCollection (`empty` then `append`) |
| WITH notes | `instrc.xml:354-365` | Repeating INPUT/OUTPUT/ERROR is an error; same object for input and output is buffered; same object for output and error interleaves |
| `ADDRESS()` | `funct.xml:647-654` | The current environment name, trailing whitespace removed |
| ERROR condition | `condtra.xml:149-172` | Raised by a command error, or by a failure when no FAILURE/ANY trap is active; ignored if the trap is in the delayed state; `::OPTIONS ERROR SYNTAX` turns an untrapped one into SYNTAX |
| FAILURE condition | `condtra.xml:183-194` | Raised by a command failure and by an unknown environment; `::OPTIONS FAILURE SYNTAX` likewise |
| condition descriptive string | `condtra.xml:538-553` | The command string for both ERROR and FAILURE |
| `RC` | `spvard.xml:62-70`, `condtra.xml:654-657` | Set to the return code of every command; set before control reaches an ERROR/FAILURE trap; commands issued from interactive trace do not change it |
| `DIRECTORY([newdir])` | `funct.xml:2475-2482` | Returns the current directory, after changing to `newdir` if given and it exists; null on failure |
| `SETLOCAL()` (Linux only) | `funct.xml:4002-4013` | Saves cwd and the process-local environment variables; returns 1/0; an unmatched SETLOCAL is undone when the procedure exits |
| `ENDLOCAL()` (Linux only) | `funct.xml:2532-2539` | Restores the last SETLOCAL's directory and variables; 1 on success, 0 if none saved |
| `USERID()` | `funct.xml:5336` | The active user identification |
| `QUALIFY(name)` | `funct.xml:3559-3562` | The fully qualified path; the file need not exist |
| `VALUE(name, [new], 'ENVIRONMENT')` | `funct.xml:5424-5477` | OS environment variables; names case-sensitive on unix; undefined reads as null; `.nil` deletes, `""` sets empty; a value is truncated at `'00'x`; changes die with the process |
| `VALUE(name, [new], '')` | `funct.xml:5480-5529` | `.environment` directory: read answers the entry or `.NAME`; write sends `NAME=` |
| Security manager: `COMMAND` | `secman.xml:111-145` | Sent for every host command with `COMMAND` and `ADDRESS`; a `1` answer uses `RC` (default 0), raises FAILURE if `FAILURE` is set, else ERROR if `ERROR` is set |
| Security manager: `STREAM` | `secman.xml:213-230` | Sent when CHARIN/CHAROUT/CHARS/LINEIN/LINEOUT/LINES/STREAM resolve a name; a `1` answer uses the `STREAM` entry as the stream object |
| Security manager: `CALL`, `REQUIRES`, `LOCAL`, `ENVIRONMENT`, `METHOD` | `secman.xml:89-108`, `:148-166`, `:169-210`, `:233-255` | The other checkpoints; answers must be exactly 0 or 1 (98.948 is the documented authorization failure number; the interpreter's own check raises 34.903) |

Not in this reference and therefore not enumerated here: `FILESPEC` and `BEEP` share `DIRECTORY`'s
implementation route (3.7) but belong to other surveyors' areas; the `RXCMD` exit and `RXSUBCOM`
registration (`usepcom.xml:104-290`) are the API/RXAPI surface (Phase 8/10).

## 2. Oracle behaviour

All probes below are **Measured** unless marked otherwise, run from
`survey-D/pNN/` with the runner `survey-D/run.sh` (oracle under the standard wrapper, crate binary
`h-5bcb28edb`, stdin `/dev/null`, three descriptors to `o.out`/`o.err`/`o.rc` and `c.out`/`c.err`/
`c.rc`). Probe programs are in each `pNN/t.rex`. "crate:" lines give the crate's answer where it
differs; every crate refusal is rc 120 unless stated.

### 2.1 Commands, `RC`, `.RS` (p01, p02, p03, p06, p41)

```text
say '[' .rs ']'                 -> [ .RS ]        (.RS is the symbol's own name until a command runs)
'echo hi'                       -> hi on stdout; rc= 0 .rs= 0
'true' / ''                     -> rc= 0 .rs= 0   ('' spawns sh -c '' and answers 0)
'exit 3'                        -> rc= 3 .rs= 1   (ERROR; under default TRACE N nothing on stderr)
'false'                         -> rc= 1 .rs= 1
'nosuchcmd_survey_d'            -> rc= 127 .rs= -1 (FAILURE) and, under default TRACE N, stderr:
      /bin/sh: 1: nosuchcmd_survey_d: not found
           1 *-* 'nosuchcmd_survey_d'
             >>>   "nosuchcmd_survey_d"
             +++   "RC(127)"
'sh -c "kill -TERM $$"'         -> rc= -15 .rs= 1  (ERROR: signal n gives rc -n)
'sh -c "kill -KILL $$"'         -> rc= -9 .rs= 1
'exit 127'                      -> rc= 127 .rs= -1 (FAILURE: 127 is the "unknown command" code however produced)
'exit 255'                      -> rc= 255 .rs= 1
'exit 256'                      -> rc= 0 .rs= 0    (WEXITSTATUS wraps)
say 'a'; 'echo b'; say 'c'      -> a b c in that order on stdout (p30, p41); the child's own stderr
                                   ('echo d 1>&2') lands on the interpreter's stderr
```
crate: every command clause is `rexx-exec: a command is not implemented (Phase 7)`; every
`ADDRESS env cmd` / `ADDRESS ... WITH` is `rexx-exec: ADDRESS is not implemented (Phase 7)`.
The crate answers `[ .RS ]` for the first line of p01 and then refuses.

### 2.2 Environment names (p04, p05, p10, p16, p33)

```text
address ksh 'echo hi'  -> rc= 127 .rs= -1 (ksh, zsh, csh not installed here: FAILURE, traced under N)
address bash 'echo hi' -> hi, rc= 0
address nosuchenv 'echo hi' -> rc= 30 .rs= -1 (FAILURE), traced under N as +++ "RC(30)"; address() then
                               reports NOSUCHENV after `address NOSUCHENV`
address SH / 'sh' / Bash / value 'kSh' / ('sy'||'stem') / '' / 'COMMAND'
   -> address() answers SH / sh / BASH / kSh / system / (empty) / COMMAND  -- the name is kept as
      written (symbols upcased by the scanner, literals verbatim); crate identical on all three descriptors
address value copies('e',250) -> address() length 250;  251 -> 29.1 "Environment name exceeds 250
      characters; found "eee..."" (crate raises 29.1 too; its condition('O') is loud)
address path 'echo hello world'   -> hello world, rc 0
address path 'echo "quoted arg" plain' -> quoted arg plain (double quotes group one argv entry)
address path 'echo $HOME'         -> $HOME (no shell, no expansion)
address path 'nosuchcmd_survey_d' -> rc= 127 .rs= -1
address path 'sh -c "exit 4"'     -> rc= 4 .rs= 1
address path ''                   -> rc= 127 .rs= -1 (argv[0] is "" and the spawn fails)
address PATH 'printf %s\n one' with output stem o. -> o.0=1 o.1=one
address value 'sh  '; say '[' address() ']' -> [ sh   ]  (p54: trailing blanks are NOT removed,
      against funct.xml:652-653; crate identical) and 'echo x' then answers rc 30 -- a name with
      whitespace is an unknown environment; '  sh' likewise
```

### 2.3 Conditions ERROR / FAILURE and the condition object (p07, p34, p34b)

`signal on error` + `'exit 5'`: handler sees `rc= 5 sigl= 3 c= ERROR d= exit 5 i= SIGNAL s= OFF`
and `condition('O')` is a Directory with exactly these indexes (sorted): `CONDITION = ERROR`,
`DESCRIPTION = exit 5`, `INSTRUCTION = SIGNAL`, `PACKAGE = a Package`, `POSITION = 3`,
`PROGRAM = <path>`, `PROPAGATED = 0`, `RC = 5`, `RESULT = 5`, `STACKFRAMES = a List`,
`TRACEBACK = a List`. `.rs` is 1 in the handler. `call on failure` + unknown command: `rc= 127
c= FAILURE d= nosuchcmd_survey_d i= CALL`, same index set with `RC = 127`, `RESULT = 127`, and the
`+++ "RC(127)"` trace still appears under TRACE N before the handler runs.

`signal on error` only, command fails (127): the ERROR trap fires with `c= ERROR`, `rc= 127`,
`.rs= -1`, and `condition('O')~condition` is `ERROR`, `~rc` 127, `~result` 127 (the FAILURE
condition object renamed, `RexxActivation.cpp:4513`).

`call on error` + `call on failure`: 127 goes to the FAILURE handler (`i= CALL`), `exit 2` to the
ERROR handler; after each, `rc`/`.rs` are 127/-1 and 2/1.

### 2.4 `::OPTIONS ERROR SYNTAX` / `FAILURE SYNTAX` (p22, p22b, p22c)

```text
::options error syntax   + 'exit 3'            -> rc 158, stderr:
     1 *-* 'exit 3'
Error 98 running <path> line 1:  Execution error.
Error 98.970:  External command "exit 3" ended with return code 3.
::options failure syntax + 'exit 3' then 'nosuchcmd_survey_d' -> "after error rc= 3" printed, then rc 158:
/bin/sh: 1: nosuchcmd_survey_d: not found
     3 *-* 'nosuchcmd_survey_d'
Error 98.971:  External command "nosuchcmd_survey_d" failed with return code 127.
::options failure syntax + signal on syntax -> handler: rc= 98 c= SYNTAX d= (empty)
     a= nosuchcmd_survey_d|127 ; condition('O')~code 98.971, ~rc 98
```
With `::OPTIONS FAILURE SYNTAX` in force the `>>>`/`+++` failure trace lines did **not** appear
(p22b, p22c). Follow-ups (p22d-k):
```text
trace n explicit + failure syntax + 127     -> still no *-*/>>>/+++ before 98.971 (p22d)
::options novalue syntax + 127              -> the normal N failure trace, rc 0 (p22e)
::options error syntax + 127                -> *-* / >>> / +++ "RC(127)" THEN the traceback and 98.970
                                               "ended with return code 127" (p22f: the untrapped
                                               FAILURE was renamed ERROR and then hit ERROR SYNTAX)
signal on failure + failure syntax + 127    -> trapped normally, rc= 127 c= FAILURE, trace lines present (p22g)
call on error + error syntax + exit 3       -> trapped normally, then "after rc= 3" (p22h)
trace a + failure syntax + 127              -> 2 *-* / >>> (from TRACE A, before the command), the shell
                                               message, then the traceback *-* and 98.971 -- NO +++ (p22i)
trace e + error syntax + exit 3             -> no error trace at all, only the traceback and 98.970 (p22j)
say trace() under failure syntax            -> N (p22k)
```
So a condition that the option turns into SYNTAX never reaches the command instruction's trace and
`+++` code at all (3.4 has the site), while a FAILURE converted to ERROR *after* that code is traced
first.

### 2.5 Trace transcripts per setting (p08E/F/C/N/A/R/I/O, p03)

Program: `trace X; 'true'; 'exit 3'; 'nosuchcmd_survey_d'; say 'rc=' rc`. stderr per setting:

```text
N, F : (child's "not found" line) then for the 127 command only:
           4 *-* 'nosuchcmd_survey_d'
             >>>   "nosuchcmd_survey_d"
             +++   "RC(127)"
E    : as N plus, for 'exit 3':   3 *-* 'exit 3' / >>> "exit 3" / +++ "RC(3)"
C    : every command:  2 *-* 'true' / >>> "true" ; 3 *-* 'exit 3' / >>> "exit 3" / +++ "RC(3)" ;
       4 *-* 'nosuchcmd_survey_d' / >>> "nosuchcmd_survey_d" / (child's not-found line) / +++ "RC(127)"
       -- the say clause is NOT traced under C
A, R : as C, and the say clause is traced (R adds its >>> "rc= 127")
I    : as R with a >L> line before each >>> and the say's >L>/>V>/>O>/>>> lines
O    : nothing but the child's own stderr
```
Under C/A/R/I the clause and `>>>` lines are written **before** the command runs (so the child's
stderr comes after `>>>`); under N/E/F they are written after it (child stderr first). The `+++`
line appears only when the numeric rc is non-zero and the clause was traced.

### 2.6 `ADDRESS ... WITH` redirection (p09, p27, p28, p29, p31)

```text
'printf "a\nb\nc\n"' with output stem out.      -> out.0=3, a b c
in.0=2 in.1=first in.2=second; 'cat' with input stem in. output stem got. -> got.0=2 first second
'cat' with input using 'single line' output stem got2.  -> got2.0=1 got2.1=single line
'sh -c "echo out; echo err 1>&2"' with output stem o. error stem e. -> o.0=1 out  e.0=1 err
same command with output stem both. error stem both.     -> both.0=2  out2 err2 (interleaved)
'echo more' with output append stem out.  (out.0 was 3)  -> out.0=4, 4th is more
a = (4,2,3,1); address sh with input using (a) output using (a); 'sort' -> a is 1 2 3 4
                                          (permanent config, USING an Array: emptied then appended)
'printf "a\nb"'         -> o.0=2 [a] [b]        (a trailing partial line is a line)
'printf "a\n\nb\n"'     -> o.0=3 [a] [] [b]
'printf "a\r\nb\r\n"'   -> o.0=2, o.1 length 1 = 'a' (CR LF is one terminator)
'printf ""'             -> o.0=0
'seq 1 3000'            -> o.0=3000, o.1=1 o.1500=1500 o.3000=3000
'printf "%5000s\n" z'   -> o.0=1 length 5000
'printf "a\0b\n"'       -> o.0=1 length 3 c2x 610062 (NUL kept inside a line)
in.0=3 in.1=one in.3=three; 'cat -A' with input stem in.  -> 3 lines: one$ / IN.$ / three$
                                          (a missing tail yields the stem's default value, here the name)
a=.array~new(3); a[1]='x1'; a[3]='x3'; with input using (a) -> 2 lines x1 x3 (empty slots skipped)
with input using 42                       -> one line 42
with input using (.list~of('l1','l2'))    -> 2 lines
in.0='abc'                                -> 26.904 Stem "IN." element 0 is not a whole number; found "abc".
with output using 42        -> 98.996 Object "42" is not a valid ADDRESS WITH OUTPUT or ERROR target.
with output using (.nil)    -> 98.996 Object "The NIL object" is not a valid ... target.
with input using .nil       -> 98.924 Object "The NIL object" is not a valid ADDRESS WITH INPUT source.
with output append using (.array~new) -> ok
with output stem o. after 'echo hi'   -> o.0=1 ; then after 'true' -> o.0=0 (REPLACE resets)
o.0=5; 'true' with output append stem o. -> o.0 stays 5
o.0='abc'; append                          -> 26.904 Stem "O." element 0 is not a whole number; found "abc".
'cat' with input stream 'in.txt' output stream 'out.txt' -> rc 0, file written; 'echo err' with
   error stream 'out.txt' truncates it (REPLACE is the default); append appends; replace truncates
with input stream 'nosuch.txt'  -> 98.999 Unable to open file "<cwd>/nosuch.txt" for reading; open result was "ERROR:2".
with output stream '/nonexistent_survey_d/out.txt' -> 98.920 Unable to open file "..." for writing; open result was "ERROR:2".
o. = 'dflt'; o.7 = 'keep'; 'echo hi' with output stem o. -> o.0=1 o.1=hi, o.7 and o.9 both dflt
   (REPLACE drops the tails and keeps the stem's default value)             (p27b)
drop p.; 'echo hi' with output append stem p.  -> p.0=1 p.1=hi (no p.0: APPEND behaves as REPLACE)
q. = 'q'; append stem q.                        -> q.0=1 q.1=hi (a default is not a stem.0 either)
drop m.; 'cat' with input stem m.              -> 98.998 Stem "M." does not contain a size count in element 0.
permanent `address sh with output stem o.`, then a PROCEDURE callee runs 'echo insub'
   -> the callee's own o. gets o.0=1 o.1=insub and the caller's o.0 stays 'main' (p55: the
      permanent config is inherited and its stem is evaluated in the then-current variable context)
```

### 2.7 `VALUE(name, new, 'ENVIRONMENT')` (p11, p39)

```text
value('SURVEY_D_VAR',,'ENVIRONMENT')          -> '' (unset reads as the null string)
value('SURVEY_D_VAR','one','ENVIRONMENT')     -> '' (the old value), then reads 'one'
'sh -c "echo child sees [$SURVEY_D_VAR]"'     -> child sees [one]
value(v,'','ENVIRONMENT')                      -> child: [] set=yes   (empty is set, not deleted)
value(v,.nil,'ENVIRONMENT')                    -> child: [] set=      (.nil deletes)
value(v,'a'||'00'x||'zz','ENVIRONMENT')        -> reads back 'a' (truncated at the NUL)
value('survey_d_var',,'ENVIRONMENT')           -> '' (names are case-sensitive)
value('HOME',,'environment') == ...'ENVIRONMENT' -> 1 (the selector is caseless)
value(v,,'ENV')                                -> 40.914 Unknown VALUE function variable environment selector; found "ENV". rc 216
value('A=B','x','ENVIRONMENT') / value('','x',...) -> '' and the set silently fails (setenv EINVAL)
value('SURVEY_SP',' a b ',...)                 -> reads back ' a b ' (whitespace kept)
value('SURVEY_N', 12.50, ...)                  -> reads back 12.50 (the string value of the object)
value('SURVEY_O', .object~new, ...)            -> 88.909 Argument 2 must have a string value.
value('NoSuchEntry',,'')                       -> .NOSUCHENTRY ; call value 'SurveyK','v','' ; say .SurveyK -> v  (p38)
```
crate: any third argument is `rexx-exec: VALUE's external-selector form is not implemented` (no
owner named), including the `''` selector.

### 2.8 `DIRECTORY`, `USERID`, `QUALIFY`, arities (p13, p14, p15, p40)

```text
directory()                        -> the probe dir (absolute, as getcwd gives it)
directory('/nonexistent_survey_d') -> '' and the cwd is unchanged
directory('/')                     -> /
directory(d'/')                    -> d (getcwd after the chdir, so no trailing slash)
directory('.')                     -> the absolute cwd
directory('')                      -> ''
directory('/tmp/../tmp')           -> /tmp
directory('/etc/passwd')           -> '' (not a directory)
'echo pwd says $(pwd)'             -> the child's cwd follows directory()
userid()                           -> moritz
qualify('foo.txt')                 -> <cwd>/foo.txt ; qualify('a/../b/./c.txt') -> <cwd>/b/c.txt
qualify('/tmp/x.txt') -> /tmp/x.txt ; qualify('') -> '' ; qualify('~') -> /home/moritz
qualify('.') -> <cwd> ; qualify('..') -> parent ; qualify('foo/') -> <cwd>/foo
userid('x')          -> 40.4 Too many arguments in invocation of USERID; maximum expected is 0.
qualify()            -> 40.3 ; qualify('a','b') -> 40.4
directory('a','b')   -> 88.922 Too many arguments in invocation; 1 expected.   (a native ROUTINE, not a BIF)
setlocal(1)          -> 40.4 ; value() -> 40.3 ; value(a,,b,c) -> 40.4
```
crate: `directory()` is `Error 43.1: Could not find routine "DIRECTORY".` rc 213 (see 4.3);
`userid()`, `qualify()`, `setlocal()`, `endlocal()` are `rexx-exec: routine "NAME" is not
implemented (4c)`.

### 2.9 `SETLOCAL` / `ENDLOCAL` -- two oracle crashes (p12, p18, p19, p40)

```text
say endlocal(); say endlocal()          -> 0 0, rc 0 (p18: unpaired, list never created)
say setlocal() endlocal()               -> 1 1, rc 0 (p40, one pair)
p12: setlocal; value set SURVEY_D_L=inner; directory('/'); endlocal
     -> setlocal 1 ; / [ inner ] ; endlocal 1 ; <probe dir> [ inner ]   -- the cwd is restored,
        the variable ADDED after SETLOCAL is NOT removed
     then a second endlocal()          -> SIGSEGV, rc 139, nothing more on either descriptor
p19: value M=before; setlocal; M=after; N=new; endlocal -> M= before N= new (changed value restored,
        added name kept); internal `sub:` doing setlocal + directory('/') then return
     -> after internal sub 0   (NOT restored at the routine's return; see 3.6)
     then an external sub2.rex doing setlocal + directory('/') -> "sub2 setlocal 1" then
        free(): invalid pointer, SIGABRT rc 134, at sub2's termination (the second restore in the process)
```
**Both crashes are deterministic memory defects in the C++ (3.6 gives the mechanism) and neither
program may be re-run to confirm**; they are recorded here in the terms `corpus/oracle-crashes.txt`
uses and belong in it.

### 2.10 Security manager `COMMAND` (p23, p23b, p23c, p24)

Agent routine: `'echo hi'; say 'rc=' rc '.rs=' .rs; address foo 'exit 9'; say ...`.
Audit manager (`unknown` printing the directory, returning 0):
```text
event COMMAND / ADDRESS = sh / COMMAND = echo hi   then hi, rc= 0 .rs= 0
event LOCAL NAME = RS ; event ENVIRONMENT NAME = RS    (reading .rs asks .local and .environment first)
event COMMAND / ADDRESS = FOO / COMMAND = exit 9   then rc= 30 .rs= -1 with +++ "RC(30)" on stderr
```
Replacing manager (`command` sets `info~rc = 1234`, `info~failure = .true`, returns 1): stderr shows
`1 *-* 'echo hi' / >>> "echo hi" / +++ "RC(1234)"` and no shell runs; the next `.rs` read then sends
`LOCAL` to the manager, which has no such method: `Error 97.1: Object "a REPLACE" does not
understand message "LOCAL".` -- a manager must answer every checkpoint message. A `command` returning
1 with no `RC` entry: RC is 0 (p23b). A `command` returning nothing: `91.999 Message "COMMAND" did
not return a result.` (p23c). An agent that triggers no checkpoint runs normally under a manager (p24).

crate: `r~setSecurityManager(anything)` is `rexx-exec: a security manager is not implemented (D12,
Phase 7)`, before the agent runs (p24).

### 2.11 Same-process `cd` / `export` / `set` / `unset` (p17)

```text
'cd /'                     -> rc= 0, directory() is / afterwards (chdir in the interpreter's process)
'cd /nonexistent_survey_d' -> rc= 2 (ENOENT), .rs= 1 (ERROR), cwd unchanged
'export SURVEY_D_E=exported' -> rc 0, value(...) reads exported, a child sees it
'unset SURVEY_D_E'         -> rc 0, reads ''
'set SURVEY_D_S=setval'    -> rc 0, reads setval (set is handled in-process like export)
'export'                   -> not handled internally; sh prints the whole environment, rc 0
'cd'                       -> rc 0, directory() is $HOME
```

### 2.12 Scoping of the address, `RC` and `.RS` across calls (p20, p21)

```text
address envA; address envB; call sub (no PROCEDURE; sub does address envC; address)
   -> in sub ENVB / in sub after ENVB / after sub ENVB / bare address -> ENVA
   (the callee gets copies of current AND alternate; nothing leaks back)
call sub3 (PROCEDURE) -> in sub3 ENVA (inherited) ; call ext.rex -> in ext ENVA (inherited)
crate: identical up to `call ext` (external routine resolution is 43.1 today)

'exit 3' -> main rc= 3 .rs= 1
call sub (no PROCEDURE, runs 'exit 4') -> in sub rc= 3 .rs= 1 ; after sub rc= 4 .rs= 1
call subp (PROCEDURE, runs 'true')     -> in subp rc= RC .rs= 1 ; in subp after rc= 0 .rs= 0 ;
                                          after subp rc= 4 .rs= 1
call ext.rex ('exit 7')                -> in ext rc= RC .rs= .RS ; after ext rc= 4 .rs= 1
.K~m ('exit 5')                        -> in method rc= RC .rs= .RS ; in method after rc= 5 .rs= 1
interpret "'exit 6'; say ..."          -> in interpret rc= 6 .rs= 1 ; after interpret rc= 6 .rs= 1
```
So `.RS` travels with the activation's settings: internal calls (with or without PROCEDURE) inherit
the caller's value, external routines and methods start unset (`.RS`), and a callee's commands never
change the caller's `.RS`; `RC` is an ordinary variable.

## 3. C++ mechanism

All **Read** unless marked. Paths are under `/home/moritz/dev/repos/ooRexx/interpreter/`.

### 3.1 The two instructions

`RexxInstructionCommand::execute` (`instructions/CommandInstruction.cpp:69-90`): `traceCommand(this)`
(clause line only when `tracingCommands()`), evaluate the expression, `requestString()`, push, then
`traceResultValue(command)` when `tracingCommands()` (the `>>>` line), then
`context->command(context->getAddress(), command, OREF_NULL)`. The parser makes a command out of any
clause that is not an assignment, keyword instruction, label or message instruction
(`parser/InstructionParser.cpp:462`, `commandNew` at `:1241`).

`RexxInstructionAddress::execute` (`instructions/AddressInstruction.cpp:127-191`), three arms:
- neither environment nor expression: `traceInstruction`, `toggleAddress()` (swap current and
  alternate, `execution/RexxActivation.cpp:2120-2125`), `pauseInstruction`.
- constant environment with a command: `traceCommand`, evaluate, `requestString`, `>>>` when tracing
  commands, `SystemInterpreter::validateAddressName` (only a length check, > 250 is 29.1,
  `platform/unix/MiscSystem.cpp:59-73`), `context->command(environment, cmd, getIOConfig())`. The
  address setting is **not** changed by this arm.
- constant environment alone: `traceInstruction`, validate, `setAddress(environment, getIOConfig())`
  (`RexxActivation.cpp:2135-2146`: alternate = current, current = new; a `WITH` config, if any, is
  stored under the name by `addIOConfig`).
- `ADDRESS VALUE`: `traceInstruction`, evaluate, `requestString`, `traceResult` (`>>>`), validate,
  `setAddress`.

Names are stored as written; only the lookup upcases (`InterpreterInstance::resolveCommandHandler`,
`runtime/InterpreterInstance.cpp:946-960`: `name->upper()` against a table filled by
`addCommandHandler` with `new_upper_string(name)`). That is why `ADDRESS()` echoes `kSh` while the
handler for `KSH` still runs (2.2). A miss in the table asks **rxapi** for a registered subcom handler
(`CommandHandler::resolve` → `RexxResolveSubcom`, `concurrency/CommandHandler.cpp:74-85`) and
caches a hit under the upcased name; this crate has no rxapi until Phase 10, so its miss is final.

`RexxInstructionAddressWith` (`instructions/AddressWithInstruction.cpp`) is the same instruction
carrying a `CommandIOConfiguration`; the parser chooses it when `WITH` follows
(`parser/InstructionParser.cpp:563-658`, `parseAddressWith` `:671-773`, `parseRedirectOutputOptions`
`:795-823`, `parseRedirectOptions` `:834-898`). Parse errors: 20.933 nothing after WITH or a
non-symbol; 25.934 option not INPUT/OUTPUT/ERROR; 25.930/25.931/25.932 duplicate INPUT/OUTPUT/
ERROR; 25.933 type not NORMAL/STEM/STREAM/USING; 20.932 STEM not followed by a stem symbol;
35.935 STREAM/USING without an expression; 35.914 ADDRESS VALUE without an expression. `STREAM` and
`USING` take a *constant expression* (literal, constant symbol, environment symbol or a
parenthesised expression, `parseConstantExpression`); `STEM` takes a stem token resolved as a
variable retriever. The crate's parser already agrees on every one of these numbers (2.6, p26a-j).

### 3.2 `RexxActivation::command` (`execution/RexxActivation.cpp:4352-4528`)

1. `instruction_traced = tracingAll() || tracingCommands()`.
2. `resolveAddressIOConfig(address, ioConfig)` (`:4325-4341`): the global config for this name
   (`getIOConfig`, `:1677-1688`, table keyed by `environment->upper()`) merged with the command's
   own; each stream comes from the command's config when it names one, from the global one
   otherwise, and `NORMAL` on the command explicitly turns the global one off for that stream
   (`instructions/CommandIOConfiguration.cpp:139-260`). Sources and targets are **evaluated here,
   per command**, and traced with `traceKeywordResult(INPUT|OUTPUT|ERROR, obj)` (`:282`, `:440`).
   `resolveConflicts` (`CommandIOContext.cpp:96-134`) collapses OUTPUT and ERROR naming the same
   target into one redirector and wraps OUTPUT in a `BufferingOutputTarget` when it is also the
   INPUT.
3. `activity->callCommandExit(...)` (`concurrency/Activity.cpp:2783-2830`): **the security manager
   first** -- `getEffectiveSecurityManager()->checkCommand` (3.8); if it handled the command the
   handler is skipped. Then the `RXCMD` API exit (Phase 8).
4. `activity->resolveCommandHandler(address)`; found → `handler->call(...)`
   (`concurrency/CommandHandler.cpp:98-151`; the unix handlers are all `HandlerType::REDIRECTING`,
   so `ioContext->init()` runs the stem/stream/collection `init`s right before the call and
   `cleanup()` right after -- a `StreamOutputTarget` opens `WRITE REPLACE`/`WRITE APPEND` in
   `init` and closes in `cleanup`, a `StemOutputTarget` empties the stem and sets `stem.0` to 0 in
   `init`, a `CollectionOutputTarget` sends `EMPTY` in `init`); not found → `rc =
   RXSUBCOM_NOTREG` (30, `api/rexxapidefs.h:91`) and a FAILURE condition object.
5. Result processing: if a condition object came back, `RC` is its `RC` entry or else its `RESULT`
   entry copied into `RC`; `FAILURE` sets `returnStatus = RETURN_STATUS_FAILURE` and remembers it
   must be re-raised as ERROR; `ERROR` sets `RETURN_STATUS_ERROR`. No rc at all → `0`.
6. Unless in a debug pause: `RC` is assigned; if `(ERROR && tracingErrors()) || (FAILURE &&
   tracingFailures())` the clause is traced (`*-*`) and the command string echoed (`>>>`), and
   `instruction_traced` becomes true; if `instruction_traced` and `rc->numberValue()` succeeds and
   is non-zero, the line `+++ "RC(n)"` is emitted through `traceValue(..., TRACE_PREFIX_ERROR)` --
   the same formatting and indentation as a `>>>` line (2.5, p46). A non-numeric rc (a manager's
   `'abc'`) gets no `+++` (p23d). Then `setReturnStatus` (`.RS`).
7. Conditions: `FAILURE && isFailureSyntaxEnabled()` → `reportException(98.971, DESCRIPTION, RC)`;
   `!failure && isErrorSyntaxEnabled()` → 98.970; else `activity->raiseCondition(obj)`, and when
   that returns false (nobody trapped it) a FAILURE is renamed to ERROR in the same directory
   (`obj->put(ERRORNAME, CONDITION)`, `:4513`, which is why 2.3's renamed object keeps `RC 127`
   and `RESULT 127`) and raised again, after an `isErrorSyntaxEnabled()` check. Arming a trap
   disables the matching `::OPTIONS ... SYNTAX` (`trapOn`, `:1566-1575`; `trapOff` too,
   `:1620-1629`), which is what makes p22g/p22h trap normally.
8. A debug pause if `instruction_traced && inDebug()` (interactive trace, another survey).

The condition object (`Activity::createConditionObject`, `concurrency/Activity.cpp:723-751`) has
`CONDITION`, `DESCRIPTION` (the command string), `PROPAGATED = 0`, `RC`, and -- because the unix
handler passes the rc as the `result` argument too -- `RESULT`; `generateProgramInformation` adds
`PROGRAM`, `PACKAGE`, `POSITION`, `STACKFRAMES`, `TRACEBACK`; the trap machinery adds
`INSTRUCTION`. 2.3's measured index set is exactly that.

`.RS` (`RexxActivation::rexxVariable`, `:2842-2856`): the integer `settings.returnStatus` once
`isReturnStatusSet()`, else the string `.RS`. It is consulted only after `.local`/`.environment`
(`expression/ExpressionDotVariable.cpp:159-170` calls `resolveDotVariable` first, then
`rexxVariable`), which is why p43's `.local~rs = 'mine'` shadows it and why p23's audit manager saw
`LOCAL`/`ENVIRONMENT` events for `RS`. `settings` (address pair, `.RS`, ioConfigs, trace) is copied
into an internal callee and not copied back; an external routine or method starts from the
defaults (measured, 2.12).

### 3.3 The unix handler (`platform/unix/SystemCommands.cpp`)

`SysInterpreterInstance::registerCommandHandlers` (`:1072-1094`) registers one function,
`ioCommandHandler`, under `SH`, `""`, `COMMAND`, `SYSTEM`, `KSH`, `CSH`, `BSH`, `BASH`, `TCSH`,
`ZSH` and `PATH`, all `REDIRECTING`. `ioCommandHandler` (`:817-1064`):

- `PATH` (caseless): `scan_cmd` (`:638-709`) splits on blanks and tabs, a double-quoted run is one
  argument (quotes removed), at most 400 arguments else FAILURE; an empty command becomes
  `argv[0] = ""` and the spawn fails → FAILURE 127 (2.2). `posix_spawnp` searches `PATH`.
- every other name: `argv = { SYSSHELLPATH "/" + (sh for "", COMMAND, SYSTEM; else the name
  lowercased), "-c", command }` with `SYSSHELLPATH` `/bin` (`PlatformDefinitions.h:68-70`). So
  `ADDRESS Bash` runs `/bin/bash -c ...` and a name whose binary is absent spawns nothing:
  `posix_spawnp` fails → `ErrorFailure` → FAILURE with rc `UNKNOWN_COMMAND` 127 (2.2, ksh/zsh/csh).
- with redirection requested (`:869-1015`): pipes for whichever of stdin/stdout/stderr is
  redirected, `posix_spawn_file_actions_adddup2`, stderr dup'd onto the stdout pipe when both go to
  the same target; the whole input is read first into one buffer
  (`ReadInputBuffer` → `CommandIOContext::readInputBuffered`, lines joined with
  `SysFileSystem::EOL_Marker`, `instructions/InputRedirector.cpp:92-118`) and written by a
  separate thread (EPIPE ignored); stderr is read by a second thread into a growing buffer and
  handed over after the child exits; stdout is read on the calling thread in `PIPE_BUF` chunks and
  fed to `WriteOutputBuffer` as it arrives. Any pipe/read error is 98.923 `Address command
  redirection failed (<strerror>)`.
- without redirection (`:1016-1032`): `handleCommandInternally` (`:721-778`) first -- if the string
  has no unquoted `< > | & ;`, then exactly `cd`, `cd `..., `set `..., `unset `..., `export `... are
  done **in the interpreter's own process**: `sys_process_cd` (`:494-626`) handles `~`, `~/x`,
  `~user`, a double-quoted path, `chdir`s and raises ERROR with rc = errno on failure;
  `sys_process_export` (`:201-451`) parses `NAME=value` with `$VAR` substitution from `environ`,
  `putenv`s (and on the first change copies the whole environment into malloc'd strings, `putflag`),
  returns rc 0; `export` alone is not handled (the shell prints the environment, 2.11). Otherwise
  `posix_spawnp` with the interpreter's `environ`.
- exit status (`:1035-1063`): `WIFEXITED` → `WEXITSTATUS`; else `-(WTERMSIG)` with a stopped child
  mapped to -1. rc 127 → FAILURE; any other non-zero → ERROR; 0 → no condition. The rc reaches
  the interpreter as the condition object's `RC`/`RESULT` (`context->RaiseCondition("ERROR",
  command, NULL, rc)`), or as the handler's return value `False` for zero.

Line splitting of captured output (`instructions/OutputRedirector.cpp:110-345`): `\n` and `\r\n`
end a line, a bare `\r` is data, a `\r` at a buffer end is held until the next buffer decides, and
a trailing unterminated remainder is flushed as a final line at cleanup (2.6, p28). A NUL is data.

Input sources (`InputRedirector.cpp`): `StemInputSource::init` requires `stem.0` (missing → 98.998
`Stem "M." does not contain a size count in element 0.`; non-whole → 26.904) and `read` returns
`stem.i` for i in 1..stem.0 via `getFullElement`, so a missing tail reads the stem's **default
value** (2.6, `IN.`); `ArrayInputSource` walks `items()` and skips empty slots;
`StreamInputSource` opens `READ` (not `OPENREADY` → 98.999 `Unable to open file`) and reads
`LINEIN` until a NOTREADY; `StreamObjectInputSource` sends `LINEIN` to the object, a trapped
condition ending it. `USING` dispatch order for input (`CommandIOConfiguration.cpp:310-376`):
String → one line; Stem; instance of `.InputStream` or `.Monitor` → stream object; `.File` →
`ABSOLUTEPATH` then stream by name; Array → `makeArray`; else `requestArray`, and anything not an
Array is 98.924. For output (`:436-520`): Stem; `.OutputStream`/`.Monitor` (REPLACE/APPEND → 98.997);
`.RexxQueue` (options → 98.922); `.File` → stream by name; `.OrderedCollection`; else 98.996.
Stream names on both sides are qualified with `Interpreter::qualifyFileSystemName` before the
same-target comparison (`:303`, `:453`).

Output targets: `StemOutputTarget::init` (`OutputRedirector.cpp:401-445`) REPLACE/DEFAULT →
`stem->empty()` (tails gone, the stem default kept -- p27b) and `stem.0 = 0`; APPEND → start at
`stem.0 + 1`, a missing `stem.0` treated as REPLACE, a non-whole one 26.904; each line sets
`stem.i` and rewrites `stem.0`. `StreamOutputTarget::init` opens `WRITE REPLACE` / `WRITE APPEND`
(failure → 98.9xx `Error_Execution_file_not_writeable`) and `cleanup` closes; `writeLine` is
`LINEOUT`. `CollectionOutputTarget` → `EMPTY` (REPLACE) then `APPEND` per line. `RexxQueueOutputTarget`
→ `QUEUE`. `BufferingOutputTarget` collects lines and pumps them into the real target at cleanup.

### 3.4 Trace flags per setting

`execution/TraceSetting.hpp:69-85` defines `traceCommands`, `traceErrors`, `traceFailures`;
`defaultTraceFlags` is `traceNormal + traceFailures` (`TraceSetting.cpp:49`). Measured (2.5): `N`
and `F` set failures only; `E` errors and failures; `C` commands (clause + `>>>` for every command,
nothing else traced); `A`, `R`, `I` trace all clauses (so `instruction_traced` is true from
`tracingAll()`); `O` nothing. The `+++` line's producers are enumerated in
`phase-4-exclusions.txt:165-202`; the one command dispatch reaches is `RexxActivation.cpp:4468`.

**Why `::OPTIONS ... SYNTAX` suppresses the trace (2.4, p22d/i/j)**: the handler raises its
condition through the API (`context->RaiseCondition("FAILURE", ...)` → `NativeActivation::
raiseCondition`, `execution/NativeActivation.cpp`, which calls `activity->raiseCondition` and then
throws), and **that** path already checks the package option: `Activity::raiseCondition`
(`concurrency/Activity.cpp:596-610`) raises `Error_Execution_error_syntax` at `:605` and
`Error_Execution_failure_syntax` at `:609` -- "if we have ::OPTIONS condition SYNTAX set on the
package, we raise a SYNTAX error instead of raising the condition" -- before the condition is
ever queued. So under the matching option the 98.97x propagates out of the handler
call and `RexxActivation::command` never reaches its `RC`/trace/`+++` block (step 6). The second
check at `:4478-4493` is reached only for conditions the native side let through, and the
FAILURE→ERROR conversion at `:4497-4515` is what makes an ERROR SYNTAX package trace a failure
first and then raise 98.970 (p22f). A port needs the option check in two places, or one check
before the trace with the conversion folded in; the observable is the same.

### 3.5 `VALUE(name, new, selector)` (`expression/BuiltinFunctions.cpp:1810-1920`)

No selector → local variable (4c's). Empty selector → `TheEnvironment->entry(name)` (case as given;
`.environment` is case-insensitive itself) else `name~upper` prefixed with `.`; a new value is
`TheEnvironment->setEntry`. Selector caselessly `ENVIRONMENT` → `getenv` (missing → `""`,
`platform/unix/SystemInterpreter.cpp:178-195`); a new value `.nil` → `unsetenv`, else
`stringArgument(newvalue, ARG_TWO)` (a non-string object is 88.909) → `setenv(name, value, 1)`
(`:199-213`); `setenv` fails silently for `""` or a name containing `=` (2.7). Any other selector →
`SystemInterpreter::valueFunction` (unix: always false, `platform/unix/ValueFunction.cpp:66-69`) →
the `RXVALUE` exit → 40.914 `Unknown VALUE function variable environment selector; found "X"`.

### 3.6 `SETLOCAL` / `ENDLOCAL` (`platform/unix/ExternalFunctions.cpp:138-342`)

`SETLOCAL` → `buildEnvlist` copies `getcwd` and every `environ` string into one `BufferClass`
(a **GC-managed Rexx object**) and `RexxActivation::pushEnvironment` pushes it on the *top-level*
activation's `environmentList` (`RexxActivation.cpp:4640-4658`: an internal routine or INTERPRET
delegates to its parent). `ENDLOCAL` → `popEnvironment` (`:4666-4684`) then `restoreEnvironment`:
`chdir` to the saved directory (failure → 48.1 `Failure in system service`), and for each saved
`NAME=value` string, find the current `environ` entry with that name, `putenv(saved)`, `free(old)`.
Termination of a top-level activation with a non-empty list restores the **oldest** entry
(`getLastItem`, `:1494-1500`); that is why an external routine's SETLOCAL is undone at its return
(p19b) and an internal routine's is not (p19), contrary to `funct.xml:4011-4013`.

Consequences a port must know:
- restoring never *removes* a name added after SETLOCAL (`:303-341` only re-puts saved names), so
  2.9's `N= new` and `[ inner ]` are the specified behaviour to match, not a defect to fix.
- **Defect 1 (SIGSEGV, p12)**: `popEnvironment` returns `environmentList->pull()`, which is
  `ArrayClass::deleteItem(1)` and answers `OREF_NULL` on an empty queue (`classes/ArrayClass.cpp:
  2577-2590`; `QueueClass` is an `ArrayClass`, `QueueClass.hpp:49`, `:72`); `SystemInterpreter::
  popEnvironment` (`:167-180`) compares that against `TheNilObject`, misses, and calls
  `restoreEnvironment(Current->getData())` on the null pointer. So the *second* unpaired ENDLOCAL
  after any SETLOCAL in the same top-level activation is a null dereference; the first unpaired one
  (list never created, `TheNilObject` returned) answers 0.
- **Defect 2 (SIGABRT `free(): invalid pointer`, p19)**: `restoreEnvironment` `putenv`s pointers
  that live **inside the Rexx buffer** and later `free`s whatever `environ` entry a name resolves
  to. The first restore in a process frees the `putflag` malloc copies and leaves `environ`
  pointing into GC memory; the second restore (a later ENDLOCAL, or a routine's termination) frees
  those and glibc aborts. So exactly one restore per process is survivable, whatever the pairing;
  p12/p18/p19b/p40 are all consistent with that count.
- Neither shape can be run again to confirm; both belong in `corpus/oracle-crashes.txt`.

### 3.7 `DIRECTORY`, `USERID`, `QUALIFY`

`DIRECTORY` is **not a BIF**: it is `sysDirectory`, a native routine of the internal `REXX` package
(`runtime/NativeFunctions.h:48`, with `Filespec` and `Beep` as the whole table;
`runtime/InternalPackage.cpp:64-84`). It is reached through the external-function search
(`platform/unix/ExternalFunctions.cpp:104-135`: macrospace → `PackageManager::callNativeRoutine`
→ external Rexx file → macrospace), which is why its arity error is the native-routine 88.922 and
not 40.4 (2.8), and why this crate answers 43.1 for it (4.3). Body: an argument is qualified
(`RoutineQualifiedName` → `canonicalizeName`, so `''` fails and `~` expands) and `chdir`'d; a
failed `chdir` answers `""`; the answer is always a fresh `getcwd` (so `d/` answers `d`).
`USERID` (`platform/unix/UseridFunction.cpp:58-67`): `getpwuid(geteuid())->pw_name`, at most 255
bytes. `QUALIFY` (`BuiltinFunctions.cpp:2988-3000`): `QualifiedName` → `SysFileSystem::
qualifyStreamName` → `canonicalizeName` (`platform/unix/SysFileSystem.cpp:628-676`): empty →
failure → `""`; leading `~` → `resolveTilde`; relative → prefixed with `getcwd()/`; then
`normalizePathName` removes duplicate and trailing slashes and resolves `.`/`..` textually, so the
path need not exist. The same function backs `ADDRESS WITH STREAM` name comparison and the
security manager's `STREAM` `NAME` (2.10, and `SecurityManager.testGroup:378` expects `directory()`
for the name `.`).

### 3.8 Security manager (`execution/SecurityManager.cpp`)

`callSecurityManager` (`:149-160`): send `methodName` with the info directory; no result → 91.999;
the result's `truthValue` with 34.903 `Authorization return value must be exactly "0" or "1"` for
anything else (p23c, p23d). The checkpoints and their call sites:

| Message | Directory in | Directory out on `1` | Called from |
|---|---|---|---|
| `COMMAND` | `COMMAND`, `ADDRESS` | `RC` (default 0), `FAILURE` present → FAILURE, else `ERROR` present → ERROR | `Activity::callCommandExit`, `concurrency/Activity.cpp:2786-2792`, before the handler |
| `STREAM` | `NAME` (the qualified name) | `STREAM` object used instead of a new `.Stream` | `RexxActivation::resolveStream`, `execution/RexxActivation.cpp:2014-2022`, on a stream-table miss (the stream BIFs only -- `.Stream~new` bypasses it, `SecurityManager.testGroup:431`) |
| `CALL` | `NAME`, `ARGUMENTS` | `RESULT` | `Activity.cpp:2663` (external function calls) |
| `REQUIRES` | `NAME` | `NAME` replaced, `SECURITYMANAGER` for the loaded file | `package/PackageManager.cpp:723`, `:756` (short and resolved name, so twice) |
| `LOCAL` / `ENVIRONMENT` | `NAME` | `RESULT` (absent → not found) | `classes/PackageClass.cpp:1137`, `:1154` |
| `METHOD` | `OBJECT`, `NAME`, `ARGUMENTS` | `RESULT` | `classes/ObjectClass.cpp:982` (PROTECTED methods) |

The `ADDRESS` entry is the name as issued (`sh`, `FOO`, `PATH` -- p23, p23d), and the manager runs
before the handler is resolved, so an unknown environment still reports `COMMAND` (p23). The
`SecurityManager` wrapper is installed per executable (`setSecurityManager` on a Routine, Method
or Package) and reached by `getEffectiveSecurityManager()`; `SecurityManager.testGroup:57-98`
drives the same checks through all three.

## 4. The crate today

All **Read** (paths under `rust/`) with the refusal bytes **Measured** in section 2.

### 4.1 Instructions

- `InstructionKind::Command` is refused in `Interp::exec_instruction`'s catch-all,
  `Loud::instruction` → `rexx-exec: a command is not implemented (Phase 7)` rc 120
  (`crates/rexx-exec/src/lib.rs:213` names it "a command"; owner at `:1328`).
- `InstructionKind::Address` with `command.is_some() || io.is_some()` is refused at
  `crates/rexx-exec/src/run.rs:1439-1445` (`rexx-exec: ADDRESS is not implemented (Phase 7)`);
  the three naming forms run through `exec_address` (`run.rs`, the `fn exec_address` block): the
  toggle, the constant name with the 250-byte check (29.1), and the evaluated name with the `>>>`
  trace. `lib.rs:1293-1299` records the arm-grained owner.
- The parser is complete: `rexx_parse::Address { environment, dynamic, command, io }` with
  `AddressIo { input, output, error, output_option, error_option }` and `Redirection::{Default,
  Normal, Stem(SymbolId), Stream(Expr), Using(Expr)}` (`crates/rexx-parse/src/ast.rs:952-1000`),
  and every parse-error number in 3.1 already agrees (p26a-j).
- Address state: `AddressState { current: Option<Rc<[u8]>>, alternate }` per activation
  (`crates/rexx-exec/src/activation.rs:90-95`), inherited by internal callees
  (`activation.rs:550`) and fresh for methods/external code (`:602`, `:647`), which is exactly the
  oracle's copy discipline (2.12). `ADDRESS()` reads it (`crates/rexx-exec/src/builtin/state.rs:59`,
  default `b"sh"` at `:27`).
- There is no `.RS`: `/bin/grep -rn 'b"RS"' crates/rexx-exec/src` finds nothing, so `.rs`
  falls through the dot-variable path to the string `.RS`, which happens to be the oracle's answer
  before any command. There is no return-status field in `AddressState` or the activation
  settings.
- Conditions: `ERROR` and `FAILURE` are trap slots already (`activation.rs:970-995`,
  `Builtin::Error`, `Builtin::Failure`), so `SIGNAL/CALL ON ERROR|FAILURE` arm and `condition('C')`
  etc. have a home; nothing raises either today. The trap-delivery entry is `run.rs:2992`
  `deliver_one_pending_trap`; the raising side for a *non-syntax* builtin condition (the shape
  HALT/NOTREADY would also use) is what Phase 7 has to add -- I did not find a general "queue
  named condition with this directory" API (`/bin/grep -n "fn raise_" run.rs activation.rs
  lib.rs` finds only `Raised` constructors for syntax errors).
- `::OPTIONS ERROR|FAILURE SYNTAX`: `options.rs:121` `raises(self, condition)` exists (used for
  NOVALUE etc.); `directive_options_novalue_*.rex` are the corpus shape to copy.
- Trace: `+++` is `Coverage::Owned("Phase 7")` in `crates/rexx-exec/tests/trace_oracle.rs:441`;
  the emitters to reuse are `trace_result` (the `>>>` line) and `push_value(out, prefix, indent,
  value)` (`crates/rexx-exec/src/trace.rs:324`, `:452`) -- a `+++` line is `push_value` with
  `"+++"` and the value `RC(n)` quoted, at the clause's value indent (p46).

### 4.2 Builtins

`rexx_inventory::builtins::EXCLUDED` (`crates/rexx-inventory/src/lib.rs:25-46`) lists `QUALIFY`,
`USERID`, `SETLOCAL`, `ENDLOCAL` (and the stream and rxapi names); `PARTIALLY_EXCLUDED` is
`VALUE`, `ADDRESS`, `QUEUED`. A wholly excluded name resolves to `Loud::unresolved_call` at
`run.rs:3547`, whose owner string is the hardcoded `"4c"` (`lib.rs:273-282`) -- the D-P7-5 defect,
still unfixed: `routine "USERID" is not implemented (4c)`. `VALUE` with any third argument is
`Loud::value_selector` (`builtin/datatype.rs:365`, `lib.rs:529`), no owner, and it fires for the
`''` selector too, which is Phase 5's `.environment` and cheap (p38).

### 4.3 Why `DIRECTORY()` is 43.1 and not loud

`DIRECTORY` is not in `NAMES` (the generated BIF table) because the oracle's own
`BuiltinFunctions.cpp` has no `BUILTIN(DIRECTORY)` -- it is a native routine of the internal
package (3.7). So `builtin::resolve` and `is_excluded_builtin` both miss, `installed_routine`
misses, `rexx_lib::lookup` misses, and the resolver answers the oracle's own "no such file"
`Raised::routine_not_found` (`run.rs:3552-3571`). `FILESPEC` and `BEEP` are the other two names in
`runtime/NativeFunctions.h` and behave the same way (`/bin/grep -rn 'b"FILESPEC"\|b"BEEP"'
crates/rexx-exec/src` finds nothing). Any program calling one of the three gets a *wrong*
answer (43.1 at rc 213) rather than a refusal -- the one place in this area where the crate is
not loud. The seam is the resolver at `run.rs:3549`: an internal-package table consulted between
the builtin step and the external search, which is also where `SysFileExists` and the rest of the
`Sys*` library (survey F) will sit.

### 4.4 Security manager (D12)

Phase 5 built the seams and no manager:
- `setSecurityManager` exists on Method/Routine (`crates/rexx-exec/src/dispatch/executable.rs:
  299-311`) and Package (`dispatch/package.rs:390-400`); with an argument both answer
  `Loud::security_manager` = `rexx-exec: a security manager is not implemented (D12, Phase 7)`
  (`lib.rs:394`), with none they answer the oracle's readback. So `r~setSecurityManager(m)` refuses
  before the agent runs (p24), and no manager object is ever held.
- `.local`/`.environment` lookups all pass `env_seam::admit` (`crates/rexx-exec/src/environment.rs
  :50-58`, "D45, site two"), a function whose body is `Ok(Admitted(()))`; `Interp::
  check_protected_method` (`dispatch.rs:1926-1936`) is likewise a no-op. `PROTECTED` is recorded
  (`executable.rs:275-284`) and read only by that stub.
- There is no `CALL`, `REQUIRES`, `COMMAND` or `STREAM` seam, because none of their call sites
  exists: external calls are 43.1, `::REQUIRES` resolution is `Interp::resolve_search` (survey E),
  commands and streams are Phase 7's.
- `ootest/ooRexx/base/security.manager/SecurityManager.testGroup` (572 lines) exercises, through a
  Routine, a Package prolog and a Method (INTERPRET is disabled there): `CALL` ×5 (`SysDropFuncs`,
  `SysSearchPath`, a non-existent `SysDoesNotExist` handled), `COMMAND` ×4 (pass; `RC` 42; `ERROR`
  with rc 0; `FAILURE` with rc 42 under `trace off`), `LOCAL` ×2, `ENVIRONMENT` ×2 (always preceded
  by `LOCAL`), `METHOD` ×4 (`setMethod`, `setSecurityManager`, `superclasses`; an unprotected
  method raises none), `REQUIRES` ×4 (two events per load; a removed `NAME` is 43.901; a
  `SECURITYMANAGER` handed on), `STREAM` ×8 (one per stream BIF plus `stream(...'open read
  shared')` and a replacement stream), each asserting the exact sorted entry list of the info
  directory (e.g. `ADDRESS COMMAND`, `NAME`, `ARGUMENTS NAME OBJECT`). Its `Collector` answers 0
  or 1 from an `unknown` method, so the crate's manager dispatch must reach `UNKNOWN` like any
  send.

### 4.5 Process state the crate touches today

- Output is buffered whole: `Interp.out` / `Interp.trace` (`lib.rs:1692`, `:1694`), returned in
  `Outcome { stdout, stderr }` and written by `rexx-run` at exit (`crates/rexx-exec/src/bin/
  rexx-run.rs`); the harnesses call `run_program` on threads in one process
  (`tests/watchdog/mod.rs:48`, `tests/corpus.rs:70`).
- Environment reads: `Interp::resolve_search` (`lib.rs:2965-2980`) reads `REXX_PATH`, `PATH` and
  `std::env::current_dir()` for `::REQUIRES` and external programs; nothing writes the environment
  or the cwd (`/bin/grep -rn "std::env::" crates/*/src` -- the rest is `rexx-bench` and argv).
- Input: `ProgramInput::{Nothing, Stdin, Bytes}` (`invocation.rs:54-62`), read line by line
  through `Input::read_line` (`input.rs:64`, `stdin().lock().read_until`), so nothing pre-drains
  stdin.
- `Invocation { argument, input, deadline }` (`invocation.rs:39-50`) has no directory or environment
  field.
- The one granted `unsafe` site is `rexx-core/src/bytes.rs` (`crates/rexx-core/src/lib.rs:23`,
  asserted by `tests/unsafe_sites.rs`); `libc 0.2.186` is already in `Cargo.lock` transitively but
  no workspace crate names it.

### 4.6 Corpus and gates touching this area

`corpus/lang/address_env.rex` (the naming forms), `corpus/keyword-exempt.txt:66-76` (the three
`ADDRESS.testGroup` rows exempted as Phase 7: `test_environment_value_with`,
`test_environment_expression_with`, `test_environment_path_null`), `tests/trace_oracle.rs`'s
`PREFIX_COVERAGE` row for `+++`, `tests/refusal_sites.rs` and `tests/owners.rs` (owner strings),
`corpus/gate-tables/` rows for `Package~setSecurityManager` etc. (table C readbacks). The
Phase 4 exclusion rows that this area retires are `phase-4-exclusions.txt:62-91` (QUALIFY,
USERID, SETLOCAL/ENDLOCAL), `:115-117` (VALUE's selector form) and `:165-202` (`+++`).

## 5. Design questions

The controller's four architecture facts are taken as given: A1 output is buffered in
`Interp.out`/`Interp.trace`; A2 the harnesses run interpreters on threads in one process, so no
process-global state may change; A3 `Interp::resolve_search` is the one existing reader of `PATH`/
`REXX_PATH`/cwd; A4 `.input` is `ProgramInput::{Nothing, Stdin, Bytes}`. Every design below is
`unsafe`-free; no new dependency is needed for anything but option (b) of D4.

### D1. Running a command: capture, environment, directory

**Recommendation.** One function `Interp::run_command(env_name, command, io) -> CommandOutcome
{ rc, condition: Option<Error|Failure> }` behind the handler table (D8), implemented with
`std::process::Command`:
- `env_clear().envs(shadow_env)` and `.current_dir(shadow_cwd)` always (A2).
- stdin: `WITH INPUT` present → `Stdio::piped()` fed from the redirector's whole buffer on a
  helper thread (EPIPE ignored, as the oracle does); otherwise by `.input`: `Stdin` →
  `Stdio::inherit()`; `Bytes` → `Stdio::piped()` written with the **unread remainder** of the bytes
  (and the remainder then marked consumed, since the child would have read it); `Nothing` →
  `Stdio::null()`. Measured with a regular file on fd 0 (p49a/b): a child issued before any read
  sees the whole input and the interpreter continues after what the child consumed; a child issued
  after `PARSE PULL` sees nothing, because the interpreter's stream had buffered the rest. Under
  the harness stdin is a pipe (`input_oracle.rs`), where `head -1` would consume everything; the
  `inherit` rule reproduces both because the crate's `stdin().lock()` buffers the same way. The
  `Bytes` rule is best-effort and needs a licence line: the oracle never runs from a byte buffer.
- stdout and stderr: `Stdio::piped()` unless `WITH OUTPUT`/`ERROR` redirects them (then the pipe
  feeds the redirector). Drain both concurrently -- stderr on a helper thread, stdout on the
  interpreter thread, as `ioCommandHandler` does (3.3) -- and after `wait()` append stdout to
  `Interp.out` and stderr to `Interp.trace` **before** the `*-*`/`>>>`/`+++` decision under
  N/E/F and after the `>>>` under C/A/R/I, which is where the oracle's writes fall (2.5). When
  OUTPUT and ERROR name the same target, one pipe with stderr dup'd onto it (`Stdio::from` a
  cloned pipe end) keeps the oracle's interleaving; `os_pipe 1.2.3` (libc-only, in the cache) or
  `std::io::pipe` (stable since 1.87; the toolchain is 1.96/1.97) provides the pair.
- exit status: `ExitStatusExt::signal()` → `-signal`, else `code()`; 127 → FAILURE, other non-zero
  → ERROR (3.3). A spawn error (`ENOENT` for a missing shell or a `PATH` miss, an empty argv[0])
  → FAILURE 127.
- `Command::new(program)` for the `PATH` environment resolves the program against the child's own
  `PATH` when `envs` set one (Rust's documented rule) -- **verify with a test** before relying on
  it, else search `require::search_entries`' split of the shadow `PATH` and pass an absolute path.
  Shell environments use the literal `/bin/<name>` (`SYSSHELLPATH`), never a search.

**Costs.** Memory proportional to the child's output (the oracle streams to fd 1); a child that
wants a terminal sees a pipe (under the harness both interpreters' children already do; `rexx-run`
at a terminal diverges from the oracle for `isatty`-sensitive programs -- accept and note); a child
that never exits blocks the interpreter thread -- the deadline in `Invocation` must `kill()` the
child, since `wait()` cannot be interrupted; per-command cost is two extra threads and two pipes,
irrelevant next to a `fork`. Cross-descriptor interleaving is not observable (D17), so appending
at the command point is exactly as ordered as the oracle within each descriptor (p30/p41).

**If wrong.** Inheriting fd 1/2 instead prints `b a c` for `say 'a'; 'echo b'; say 'c'` -- every
command witness fails; there is no partial credit.

### D2. Shadow environment

**Recommendation.** `Interp.env: Vec<(Vec<u8>, Vec<u8>)>` (insertion order, linear lookup; a few
hundred entries at most), initialised from `std::env::vars_os()` (`OsStrExt::as_bytes`) at
`Interp::new` unless `Invocation::with_environment(...)` supplies one. Writers: `VALUE(...,
'ENVIRONMENT')` (truncate the value at the first NUL; a name that is empty or contains `=` is
silently not stored; `.nil` removes), the in-process `export`/`set`/`unset` (D9), `ENDLOCAL`
(re-put saved names only, D6). Readers -- **every route, and each is a witness**: `VALUE` reads;
child processes (`envs`); `Interp::resolve_search`'s `PATH`/`REXX_PATH` (A3 -- and this is
load-bearing for L2: `ootest/testOORexx.rex:48-77` and `framework/OOREXXUNIT.CLS:280-299` set
`PATH` through `VALUE` so that `worker.rex` and the framework are found -- survey E's area, flagged
here); `SysTempFileName`'s `TMPDIR` (`platform/unix/SysFileSystem.cpp:1854`, survey F); `HOME`
for `cd` with no argument and `~/` (3.3; whether `QUALIFY('~')`/`resolveTilde` reads `HOME` or
`getpwuid` is **not established** -- survey B's `canonicalizeName`). Names and values are bytes,
not `OsString` (unix `OsStrExt` converts losslessly at the spawn boundary).

**Costs.** A child's `environ` order cannot match: `Command` keeps its env map sorted, the oracle
passes `environ` as inherited plus appends. Nothing in the corpus prints the environment, and the
dump is host-dependent anyway (p17's `export` output carries session tokens); license it as
unobservable. `Command::env` with a NUL-bearing value makes the spawn fail -- hence truncation at
store time, which is also the oracle's observable.

**If wrong (a route missed).** The symptom is a stale read after a write: `VALUE` sets `PATH` and
`::REQUIRES` still searches the process `PATH`. The witness is `value('PATH', dir, 'ENVIRONMENT')`
followed by a `CALL` into that directory, which is exactly the ooTest kicker's shape.

### D3. Shadow current directory

**Recommendation.** `Interp.cwd: PathBuf`, initialised from `std::env::current_dir()` or
`Invocation::with_directory(...)`. `DIRECTORY(new)`: qualify `new` against the shadow (the
`canonicalizeName` rules, 3.7), require `metadata(...).is_dir()`, store `fs::canonicalize(...)`
(getcwd answers the symlink-resolved path; `/tmp/../tmp` → `/tmp` measured), answer the stored
path; failure → `""` and no change. `DIRECTORY()` answers the stored path. **Every relative path
in the phase resolves against it**: stream opens (survey A), `.File` (B), `Sys*` (F), `QUALIFY`,
`ADDRESS WITH STREAM` names, `resolve_search` (E), child `current_dir` (D1), `SETLOCAL`'s
snapshot. `std::env::set_current_dir` is safe but process-wide, and A2 rules it out.

**Costs.** One `Path::join` per open; a helper `Interp::resolve_path(&[u8]) -> PathBuf` keeps it
to one call site per consumer. `ENDLOCAL`'s directory restore can no longer fail (no `chdir`), so
the oracle's 48.1 on a vanished directory is unreachable -- a licence line, not a defect.

**If wrong (a consumer missed).** Silent: under `rexx-run` the shadow and the process cwd agree
until the first `DIRECTORY(x)`, so a `std::fs::File::open("rel")` works in every test that never
changes directory. The witness is A6's run directory plus `call directory 'sub'` before a
relative open; every stream/File witness should include one such line.

### D4. `USERID()` without `unsafe`

Options: **(a)** std only -- parse `/proc/self/status` for `Uid:` (second field is the effective
uid), scan `/etc/passwd` for the line whose third field is that uid, answer its first field;
fallback the shadow `USER`/`LOGNAME`, else `""`. **(b)** `nix 0.31.3` with `features = ["user"]`
(`nix::unistd::{geteuid, User::from_uid}`; safe API over `getpwuid_r`; dependencies `libc 0.2.186`
(already locked), `bitflags 2.x`, `cfg-if 1.0.4`, all in the offline cache; `memoffset`/`pin-utils`
are optional and not pulled). **(c)** `whoami` -- larger tree, rejected. **Recommend (a)**: it is
what the sandbox can build offline without a new direct dependency, and the value is host-bound
either way (`phase-4-exclusions.txt:72-78`: no corpus program can carry it). The same passwd scan
gives `pw_dir` for `cd ~user` (3.3). **If wrong:** an NSS-only account (LDAP) answers `""` or the
env name where the oracle answers the account name; no corpus row can show it, so record it.
Reassess to (b) if survey F's `SysGetpwnam`/`SysGeteuid` family lands and wants one implementation.

### D5. `.RS` and `RC`

`return_status: Option<i8>` beside `AddressState` in the activation settings, inherited into
internal callees (with or without PROCEDURE) and fresh for methods and external code (2.12), set
by every command (0 / 1 / -1), read in the dot-variable path **after** `.local`/`.environment` and
before the `.RS` string fallback (p43; the audit manager sees `LOCAL`/`ENVIRONMENT` for `RS`,
p23). `RC` is the ordinary variable the syntax trap already assigns; commands assign it before any
trace and before the condition is raised (3.2 step 6). Cheap; the risk is only the resolution order.

### D6. `SETLOCAL` / `ENDLOCAL` on the shadows

A `Vec<(PathBuf, Vec<(name, value)>)>` on the **top-level** activation (internal routines and
INTERPRET push to their program's; an external routine has its own). `SETLOCAL` → push a clone,
answer 1. `ENDLOCAL` → pop: cwd := saved; for each saved name, set it to the saved value; names not
in the snapshot are left alone (2.9, 3.6); answer 1; empty → 0. Program or external-routine
termination with a non-empty stack restores the **oldest** entry (p19b). **Licensed divergences**
to record: the second unpaired ENDLOCAL after a pair (oracle SIGSEGV) answers 0; the second restore
in a process (oracle SIGABRT) is an ordinary restore; both crash programs go into
`corpus/oracle-crashes.txt` and no differential runs them. Note for the spec's 4a wording: the
abort is not "a deeper list" -- it is the second `restoreEnvironment` in the process, whatever the
nesting (3.6, defect 2).

### D7. ERROR / FAILURE conditions and the `+++` line

Build the condition directory with exactly the measured index set (2.3), queue it through the
trap machinery `deliver_one_pending_trap` serves, rename an untrapped FAILURE to ERROR and re-queue
(3.2 step 7), and implement `::OPTIONS ERROR|FAILURE SYNTAX` in both places 3.4 names (before the
trace when the raise happens, and after the FAILURE→ERROR conversion), with trap arming disabling
the option as `directive_options_novalue_trap_wins.rex` already does for NOVALUE. The `+++` line is
`push_value("+++", indent, b"RC(n)")` with the `>>>` line's indent, only when the clause was traced
and the numeric rc is non-zero (2.5, p46, p53; a non-numeric rc prints nothing). Trace ordering
follows from D1's append point. `PREFIX_COVERAGE`'s `+++` row moves to `Witnessed`.

### D8. Handler table and unknown environments

Static table keyed by the upcased name: `""`, `COMMAND`, `SYSTEM`, `SH` → `/bin/sh -c`; `KSH`,
`CSH`, `BSH`, `BASH`, `TCSH`, `ZSH` → `/bin/<lower> -c`; `PATH` → `scan_cmd` split (blanks/tabs,
double-quoted runs, 400-argument cap → FAILURE); anything else → rc 30 FAILURE with the command as
description. The name is stored as written and never trimmed (p54). The oracle's fallback to a
**registered subcom handler** through rxapi is Phase 10's; until then an unknown name is final,
which only diverges for a program that registered one -- record it beside the Phase 10 row.

### D9. In-process `cd` / `export` / `set` / `unset`

Reproduce `handleCommandInternally` (3.3) on the shadows: only when the command has no unquoted
`< > | & ;`; `cd`/`cd ...` (`~`, `~/x`, `~user`, double-quoted path, unquoted rest) → shadow
`chdir`, ERROR with rc = `raw_os_error` (2 for a missing directory, p17) on failure; `export
NAME=v` / `set NAME=v` → shadow set with the oracle's `$NAME` substitution (a `$` run ends at `/`,
`:`, `$` or end; no braces), no `=` → rc 0 and nothing done unless the text has `|`/`>`; `unset
NAME` → remove; `export` alone → the shell. These four are what make `test_command_cd` and
`test_command_set` in `ADDRESS.testGroup` pass, and they are cheap.

### D10. `ADDRESS ... WITH`

Model the oracle's split: a `CommandIOConfiguration` per instruction (the parser's `AddressIo`),
the permanent table `configs: HashMap<upper name, Rc<AddressIo>>` **in the activation settings**
(inherited by callees, not copied back -- p47/p55), and a per-command `CommandIOContext` built by
evaluating sources/targets in the current variable context at issue time (a stem in a PROCEDURE
callee is the callee's, p55). Redirectors: `Stem` in/out with the REPLACE/APPEND/`stem.0` rules of
3.3 (`empty()` keeps the stem default -- p27b); `Array`/`makeArray` collections in, `OrderedCollection`
out (`EMPTY` then `APPEND` sends, so class dispatch, not a shortcut); String in; buffering when
input and output are one object (the doc's `sort` example); the `STREAM name` forms and `USING` a
stream/File/Monitor object depend on surveys A/B/C and come in a second slice. Error numbers:
98.924, 98.996, 98.997, 98.922, 98.998, 26.904, 98.999, 98.920, 98.923. Ask the `RexxQueue` target
to refuse loudly (Phase 10). Same-target tests: identity for objects, qualified-name string
equality for stream names.

### D11. Security manager (D12's Phase 7 half)

Store the manager on the executable (a table keyed by the Routine/Method/Package `ObjRef`,
rooted); `getEffectiveSecurityManager()` = the running code's, inherited by what it loads via
`REQUIRES`'s `SECURITYMANAGER` entry. One `check(message, entries) -> Option<Directory>` that
sends `message` (must reach `UNKNOWN`), rejects a missing result (91.999) and a non-logical one
(34.903), and hands the directory back on `1`. Hook sites: `COMMAND` in `run_command` before the
handler (D1), reading `RC`/`FAILURE`/`ERROR` back (2.10, 3.8); `STREAM` in the stream-name
resolver (survey A) with the *qualified* name and the `STREAM` entry as the object; `LOCAL`/
`ENVIRONMENT` in `env_seam::admit` and `METHOD` in `check_protected_method`, both already
chokepoints; `CALL`/`REQUIRES` at survey E's sites. `setSecurityManager(m)` on Method/Routine/
Package stops refusing. Zero cost when no manager is installed (an `Option` check); one send per
checkpoint otherwise. The ooTest group asserts exact entry sets and event *sequences*
(`ENVIRONMENT LOCAL` sorted, `LOCAL` before `ENVIRONMENT` in fact), so the lookup order in
`environment.rs` must be `.local` then `.environment`.

### D12. `DIRECTORY`/`FILESPEC`/`BEEP` are routines, not builtins

Add an internal-package table consulted at `run.rs:3549` between the builtin step and the
external search (the same slot the `Sys*` library needs, survey F), so the three stop answering
43.1 and refuse loudly until implemented; their arity errors are the native-routine 88.922 shape,
not 40.3/40.4. Also fix `Loud::unresolved_call`'s `"4c"` owner (D-P7-5) and give
`Loud::value_selector` an owner.

## 6. Proposed task slices

Each slice is independently testable; each witness is a `corpus/lang/*.rex` program compared byte
for byte on all three descriptors under A6's run directory, and each is paired with the negative
or adjacent case that pins the rule. Witnesses avoid host-dependent bytes: no `ksh`/`zsh`/`csh`
(installed or not), no shell "not found" text (`/bin/sh: 1: x: not found` is dash's; use `exit
127` for FAILURE), no `userid()` value, no environment dumps.

1. **Loud before right.** `Loud::unresolved_call` owner from the exclusion table (D-P7-5) with an
   assertion; `Loud::value_selector` names an owner; an internal-routine name table makes
   `DIRECTORY`/`FILESPEC`/`BEEP` refuse loudly (D12). Witnesses: crate-only refusal tests;
   adjacent: `call zorkolo` still 43.1 rc 213.
2. **Shadows and the platform readers.** `Interp.env`, `Interp.cwd`, `Invocation::with_environment/
   with_directory`; `resolve_search` on the shadows (with E); `VALUE(...,'ENVIRONMENT')`,
   `VALUE(...,'')`, `DIRECTORY`, `QUALIFY`, `USERID`, arities. Witnesses: `value_environment.rex`
   (2.7 minus the child lines: old value returned, empty vs `.nil`, NUL truncation, case, `A=B`,
   88.909, 40.914); `value_environment_selector.rex` (`''` selector, `.NOSUCHENTRY`);
   `directory_shadow.rex` (2.8: failing change answers `''` and keeps the cwd, `d/` → `d`,
   `/tmp/../tmp`, a file is not a directory, `''`); `qualify_forms.rex` (2.8 without `~`);
   `platform_arities.rex` (40.3/40.4/88.922). Adjacent: `directory()` twice is stable; `value` of
   an unset name is `''` not `NAME`.
3. **Commands.** The command clause, `ADDRESS env cmd`, the handler table, capture into
   `out`/`trace`, `RC`, `.RS`, in-process `cd`/`export`/`set`/`unset`, `PATH`. Witnesses:
   `command_rc_rs.rex` (2.1: `.RS` before, `echo`, `true`, `''`, `exit 3`, `false`, `exit 127`,
   `exit 255`, `exit 256`, `kill -TERM`); `command_environments.rex` (aliases `''`/`COMMAND`/`SYSTEM`,
   `address nosuchenv` → 30, `address()` after each, `'sh  '` with blanks → 30); `command_path.rex`
   (2.2's `PATH` rows); `command_output_order.rex` (p30/p41); `command_cd_export.rex` (p17 without
   the bare `export`); `command_scoping.rex` (2.12 without the external routine; with it once E
   lands); `command_value_environment_child.rex` (a child printing a variable set by `VALUE`).
   Adjacent: `'exit 0'` sets `.rs` 0 after a prior 1; a command inside `INTERPRET` sets the
   caller's `RC`.
4. **Conditions and trace.** ERROR/FAILURE objects, traps, FAILURE→ERROR, `::OPTIONS ERROR|FAILURE
   SYNTAX` at both sites, `+++`. Witnesses: `command_traps.rex` (p07 with the directory listing,
   p34, p34b); `command_options_error_syntax.rex`, `command_options_failure_syntax.rex`,
   `command_options_syntax_trapped.rex` (p22, p22b, p22c, p22g, p22h, p22i, p22j with `exit 127`
   in place of the missing command); `command_trace_<letter>.rex` for E F C N A R I O (p08 with
   `exit 127`), `command_trace_nested.rex` (p46), `command_trace_signal.rex` (p53). Adjacent: a
   zero rc under `trace c` prints `*-*`/`>>>` and no `+++`.
5. **`ADDRESS WITH`, stems and collections.** Permanent and per-command configs, STEM in/out/error,
   USING String/Stem/Array/OrderedCollection, REPLACE/APPEND, NORMAL, same-target buffering, the
   error numbers. Witnesses: `address_with_stem.rex` (2.6's stem, splitting, sparse, `\r\n`, NUL,
   3000 lines, 5000-byte line), `address_with_stem_options.rex` (p27, p27b), `address_with_global.rex`
   (p47, p55), `address_with_sort_example.rex` (the reference's `sort` example), `address_with_errors.rex`
   (98.924/98.996/98.998/26.904). Adjacent: `with output normal` overriding a global stem.
6. **`ADDRESS WITH` streams** (after A/B/C): STREAM names, USING stream/File/Monitor objects,
   98.999/98.920/98.997, `RexxQueue` loud. Witness: `address_with_stream.rex` (p31 in the run
   directory, p56, p57). Adjacent: a stream name that is the same file spelled two ways is one
   target.
7. **`SETLOCAL`/`ENDLOCAL`.** Witnesses: `setlocal_endlocal.rex` (one pair: changed value restored,
   added name kept, cwd restored, first unpaired `endlocal()` before any `setlocal()` is 0);
   `setlocal_external_restore.rex` (p19b, with E). Licensed and recorded, never run against the
   oracle: the second unpaired ENDLOCAL, the second restore in a process.
8. **Security manager.** Storage, `setSecurityManager` answering, `COMMAND` and `STREAM` hooks,
   `LOCAL`/`ENVIRONMENT`/`METHOD` through the existing seams, `CALL`/`REQUIRES` with E, result
   checks. Witnesses: `security_manager_command.rex` (p23 with a complete `unknown` manager: audit
   entry lists including the `LOCAL`/`ENVIRONMENT` for `.rs`, replace with rc/failure, error with rc
   0, `'abc'` rc and no `+++`, no `RC` → 0), `security_manager_answers.rex` (91.999, 34.903),
   `security_manager_stream.rex` (after A). Adjacent: an agent that triggers no checkpoint (p24).
   Target: `SecurityManager.testGroup` runs (its `SysDropFuncs`/`SysSearchPath` rows need F).
9. **Close-out.** Delete `phase-4-exclusions.txt:62-91`, `:115-117`, `:165-202`; remove
   `corpus/keyword-exempt.txt:74-76`; `PREFIX_COVERAGE` `+++` → `Witnessed`; add the two
   SETLOCAL entries to `corpus/oracle-crashes.txt`; licence lines for D2's environ order, D3's 48.1,
   D6's two crashes, D1's `Bytes` stdin and terminal children.

## 7. Not done / not established

- **Not measured**: `tcsh`/`bsh` environments (not installed; `bash` is, `ksh`/`zsh`/`csh` are
  not, so their FAILURE rows are host-dependent); the `RXCMD` API exit and registered subcom
  handlers through rxapi (Phase 8/10); `RexxQueue` as a WITH target (Phase 10); USING a `.File`,
  `Monitor` or `InputStream`/`OutputStream` object (needs A/B/C; the C++ paths are read in 3.3);
  interactive debug after a command (`TRACE ?` blocks the oracle, `oracle-crashes.txt` entry 9);
  a stopped child (the `-1` branch is unreachable without `WUNTRACED`); stdin sharing with a pipe
  on fd 0 (p49 used a regular file) and with `PARSE LINEIN`; `DIRECTORY` through a symlink
  (`fs::canonicalize` vs `getcwd` agree on symlink resolution by construction, not by measurement);
  `SIGNAL ON ANY` with a command condition; the "ignored if the trap is in the delayed state" rule
  (`condtra.xml:160-161`); `condition('O')~traceback` contents for a command; whether a callee's
  permanent WITH config leaks back to the caller (the settings copy says no, p47 did not isolate it).
- **Not established by reading**: whether `resolveTilde` (QUALIFY `~`, `cd ~/`) reads `HOME` or
  the passwd entry -- `p15` shows `qualify('~')` is `/home/moritz`, consistent with both; whether
  Rust's `Command::new` honours the child's `PATH` from `envs` (documented, not tested here);
  `SysFileSystem::EOL_Marker` (assumed `\n`, the input lines the child received had no `\r`);
  `Error_Execution_file_not_writeable` is 98.920 by measurement (p57) but the symbol was not
  looked up.
- **Not covered by this survey**: the stream side of `STREAM` redirection and the `STREAM`
  security hook's resolver (A), `.File` (B), monitors (C), external routine resolution and the
  `CALL`/`REQUIRES` hooks (E), the `Sys*` table including `SysFileTree`, `SysTempFileName`'s
  `TMPDIR` (F), Windows (`CMD`, `ENDLOCAL` always 0), any platform other than Linux.
- **Crate items seen in passing, outside this area**: parse errors exit 120 with `rexx-exec:
  NN.NNN:` text rather than the oracle's traceback and rc (a Phase 3 scope decision, not a Phase 7
  finding); `condition('O')` is loud (p33), which every witness in slice 4 works around by reading
  `condition('C')`/`('D')`/`rc` -- or waits for it.
- **Probe hygiene**: p17's `'export'` dump contains session secrets and is not quoted anywhere;
  p12 and p19 each crashed the oracle once and were not re-run; no probe touched anything outside
  `survey-D/`.

<!-- SURVEY COMPLETE -->
