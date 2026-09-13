# Survey C — standard streams, monitors, output/input routing, interactive `TRACE ?`

Surveyor C, 2026-09-12, read-only, against oracle `build/` (5.3, `RelWithDebInfo`) and crate snapshot
`bins/h-5bcb28edb`. Claims are marked **Measured** / **Read** / **Inferred**.

Probe root `<P>` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/survey-C`.
Every probe is `<P>/pNN/t.rex`, run from inside `<P>/pNN` by `<P>/run.sh` (oracle under the brief's
wrapper, crate as `timeout 20 $BIN t.rex`, stdin from `/dev/null` unless a file is named, stdout /
stderr / rc into three files). In quoted output the absolute program path is abbreviated to
`<P>/pNN/t.rex`; everything else is byte-exact (`cat -A`, `$` = end of line). "crate" below always
means the snapshot binary; its refusals are all `rc 120`.

---

## 1. Documented surface

Enumerated from `oodocs/rexxref/en-US/oneof.xml`'s `.LOCAL` section (`locenv`, `:146`) and its item
list, the Monitor class table (`utilityclasses.xml:4701`), `iostrms.xml`, `xintdeb.xml`'s chapter
structure, the TRACE instruction (`instrc.xml:3440`) and the TRACE builtin (`funct.xml:5099`).

| item | doc citation | one-line contract (Read) |
|---|---|---|
| `.LOCAL` | `oneof.xml:146-176` | Directory of per-instance objects: the five Monitors, the three Streams, `.STDQUE`, `.SYSCARGS`; `Directory` methods apply; own additions should carry a period in the name |
| `.INPUT` | `oneof.xml:246-262` | Monitor holding the default input stream; source for PARSE LINEIN, LINEIN builtin with no name, and PULL/PARSE PULL when the external queue is empty; initial source `.STDIN` |
| `.OUTPUT` | `oneof.xml:264-281` | Monitor holding the default output stream; destination of SAY, `.OUTPUT~LINEOUT`, LINEOUT builtin with no name; "you can replace this object in the environment"; initial destination `.STDOUT` |
| `.ERROR` | `oneof.xml:232-244` | Monitor holding the error stream; initial destination `.STDERR` |
| `.DEBUGINPUT` | `oneof.xml:216-230` | Monitor; source of *all* interactive-debug input; initial source `.INPUT` |
| `.TRACEOUTPUT` | `oneof.xml:283-295` | Monitor; trace output target; initial destination `.ERROR` |
| `.STDIN` | `oneof.xml:309-319` | Stream on the process's standard input; startup default for `.INPUT` |
| `.STDOUT` | `oneof.xml:321-331` | Stream on standard output; startup default for `.OUTPUT` |
| `.STDERR` | `oneof.xml:297-307` | Stream used for trace and error message output |
| `.STDQUE` | `oneof.xml:333-343` | RexxQueue; PUSH/QUEUE destination and PULL/PARSE PULL source — **Phase 10** per the brief; confirmed a `RexxQueue` and nothing more in §2.2 |
| `.SYSCARGS` | `oneof.xml:345-378` | Array of the individual C command-line arguments (vs. `ARG(1)`'s one joined string); "may not be available in all situations, e.g. `rexx -e`" |
| `Monitor` class | `utilityclasses.xml:4701-4827` | Proxy forwarding messages to a target that can be changed dynamically; methods `current` (`:4739`), `destination` (`:4755`: with an argument, stacks it; without, "the previous destination object becomes the new destination"), `init` (`:4775`), `unknown` (`:4791`, forwards all unknown messages); example `:4809-4826` |
| SAY → `.OUTPUT` | `iostrms.xml:210`, `instrc.xml:3131` | SAY writes to the default output stream object `.OUTPUT` |
| PULL → `.INPUT` | `iostrms.xml:146`, `:298`, `instrc.xml:2301` | PULL/PARSE PULL read `.INPUT` if the queue is empty |
| `LINEOUT(,"x")` ≡ `.Output~lineout("x")` | `iostrms.xml:401-409` | both write to the default output stream; `.INPUT`/`.OUTPUT` default to the OS streams |
| names `STDIN`/`STDOUT`/`STDERR` | `iostrms.xml:411-420` | reserved stream names, case-insensitive; qualify with a directory to reach a file so named |
| NOTREADY on the defaults | `iostrms.xml:681-682` | "SAY .OUTPUT and PULL .INPUT never raise the NOTREADY condition" — **contradicted for PULL by measurement**, §2.6 |
| Interactive debugging | `xintdeb.xml:50-107` | `?` on the TRACE instruction, TRACE function or `::OPTIONS TRACE` turns debug on; further TRACE instructions ignored; pause after nearly all traced instructions; null line continues; `=` re-executes the last traced clause; anything else is interpreted as clauses (INTERPRET rules; syntax error → message and re-prompt; SIGNAL conditions disabled; no tracing except non-zero command RCs; RC and `.RS` not set); off via `TRACE ?` while active, or `TRACE O` / bare `TRACE` |
| numeric TRACE in debug | `xintdeb.xml:109-125`, `instrc.xml:3629-3637` | `TRACE n` skips the next n pauses; `TRACE -n` inhibits all tracing for n clauses; the trace setting is saved and restored across subroutine calls |
| `?` prefix | `instrc.xml:3581-3626` | valid alone or with an option; toggles debug; `TRACE ?` alone turns debug off keeping the option; "use `CALL TRACE '?'` to turn off" since the instruction is ignored in debug |
| TRACE builtin | `funct.xml:5099-5135` | returns the setting (with `?` prefix when debugging); alters the setting **even in interactive debug**; option cannot be a number; `TRACE("O")` returns `?R` while switching off |
| `RXTRACE` | `xintdeb.xml:174-190` | env var `ON` (case-insensitive) starts each new program as if `TRACE '?R'` were its first instruction; checked once at interpreter start |
| trace lines are TraceObjects | `xintdeb.xml:413-414`, `:555`, `utilityclasses.xml:11372`, `:12092` | the interpreter sends a TraceObject to `.traceOutput`, which needs its string value; a `'P'` option stops forwarding |

Documented and **not covered** here: `HANDLE:x` streams (`iostrms.xml:421-437`, stream survey);
TraceObject profiling and `setMakeString` (`xintdeb.xml:536-`, `:792-`) and the multithread trace
additions (`xintdeb.xml:274-`); `::OPTIONS TRACE ?x` (`xintdeb.xml:52-55`) — **never run**, it is
`corpus/oracle-crashes.txt` entry 9; the system exits that sit in front of every route
(`callSayExit`, `callPullExit`, `callDebugInputExit`, …, Phase 8/10); `rexx -e`; Windows.

---

## 2. Oracle behaviour

### 2.1 Which object each route goes through, and with which message (p05, p06)

A recorder class whose `unknown` writes `[tag:MSG <class|value>... (n=argc)]` straight to
`.stderr` (bypassing the monitors) was stacked as the destination of `.output`, `.error`,
`.traceoutput` (p05) and of `.input` (p06).

**Measured**, `<P>/p05/t.rex`:
```rexx
.output~destination(.Rec~new('O'))
.error~destination(.Rec~new('E'))
.traceoutput~destination(.Rec~new('T'))
say 'S1'
say
call lineout , 'L1'
call charout , 'C1'
.output~lineout('O1')
.output~charout('O2')
.output~say('O3')
call lineout 'STDERR', 'L2'
.error~say('E1')
.error~lineout('E2')
.traceoutput~lineout('T1')
trace r
zz = 1 + 1
trace off
call lineout , 'L3', 1
call lineout
say result
exit
::class Rec
::method init
  expose tag
  use arg tag
::method unknown
  expose tag
  use arg msg, args
  line = '['tag':'msg
  do i = 1 to args~size
    if \args~hasIndex(i) then line = line '<omitted>'
    else line = line '<'args[i]~class~id'|'args[i]~makestring'>'
  end
  .stderr~lineout(line '(n='args~size')]')
  return 7
```
rc 0, stdout empty, stderr:
```
[O:SAY <String|S1> (n=1)]
[O:SAY <String|> (n=1)]
[O:LINEOUT <String|L1> (n=1)]
[O:CHAROUT <String|C1> (n=1)]
[O:LINEOUT <String|O1> (n=1)]
[O:CHAROUT <String|O2> (n=1)]
[O:SAY <String|O3> (n=1)]
[E:LINEOUT <String|L2> (n=1)]
[E:SAY <String|E1> (n=1)]
[E:LINEOUT <String|E2> (n=1)]
[T:LINEOUT <String|T1> (n=1)]
[T:LINEOUT <TraceObject|    16 *-* zz = 1 + 1> (n=1)]
[T:LINEOUT <TraceObject|       >>>   "2"> (n=1)]
[T:LINEOUT <TraceObject|    17 *-* trace off> (n=1)]
[O:LINEOUT <String|L3> <String|1> (n=2)]
[O:LINEOUT (n=0)]
[O:SAY <String|7> (n=1)]
```
So: **SAY sends `SAY` with exactly one String argument** (the null string for a bare `say`) and
ignores the reply; `LINEOUT(,…)` / `CHAROUT(,…)` with the name omitted send `LINEOUT` / `CHAROUT`
to `.OUTPUT` with exactly the arguments given (0, 1 or 2) and return the reply (`result` = 7);
`LINEOUT('STDERR',…)` sends `LINEOUT` to the **`.ERROR` monitor**; every trace line is one `LINEOUT`
to `.TRACEOUTPUT` carrying a **`TraceObject`** whose `makestring` is the line without its newline.

**Measured**, `<P>/p06/t.rex` with stdin `in1\nin2\n` and the same recorder (returning `'r'n`) on
`.input`:
```rexx
.input~destination(.Rec~new('I'))
pull v1          ; say '<'v1'>'
parse pull v2    ; say '<'v2'>'
v3 = linein()    ; say '<'v3'>'
parse linein v4  ; say '<'v4'>'
v5 = linein('STDIN') ; say '<'v5'>'
push 'queued'
pull v6          ; say '<'v6'>'
say lines() chars() lines('STDIN')
v7 = charin()    ; say '<'v7'>'
say .input~linein
say .stdin~linein
say .debuginput~linein
```
(one clause per line in the file) rc 0; stdout `<R1> <r2> <r3> <r4> <r5> <QUEUED> r6 r7 r8 <r9>
r10 in1 r11` one per line; stderr:
```
[I:LINEIN (n=0)]   x5
[I:LINES <NORMAL> (n=1)]
[I:CHARS (n=0)]
[I:LINES <NORMAL> (n=1)]
[I:CHARIN (n=0)]
[I:LINEIN (n=0)]   x2
```
So PULL, PARSE PULL (queue empty), `LINEIN()`, PARSE LINEIN, `LINEIN('STDIN')`, `.debuginput~linein`
all send **`LINEIN` with no arguments** to `.INPUT`; `LINES()` sends `LINES('NORMAL')`, `CHARS()`
sends `CHARS`, `CHARIN()` sends `CHARIN`; PULL uppercases what comes back; a queued line is taken
before `.INPUT` is consulted; `.stdin~linein` reads the real stdin directly (`in1`).

### 2.2 Identities, names, classes (p02, p02b, p38, p38b, p39)

**Measured** `<P>/p02/t.rex` (no arguments), rc 0, stdout, one line per `say`:
```
Monitor 0 1                          -- .output~class~id, isA(.Stream), isA(.Monitor)
Stream 1 0                           -- .stdout
Monitor Stream Monitor Stream Monitor Monitor   -- .input .stdin .error .stderr .traceoutput .debuginput
RexxQueue / The Object class / Class -- .stdque class, superclasses, metaclass
Array 0                              -- .syscargs class, items, makestring
The OUTPUT monitor | The INPUT monitor | The ERROR monitor | The TRACE OUTPUT monitor | The DEBUG INPUT monitor
The OUTPUT monitor | STDOUT | STDERR | STDIN | The ERROR monitor | The TRACE OUTPUT monitor | SESSION
1 1 1 1 1                            -- current: .output==.stdout .error==.stderr .traceoutput==.error .input==.stdin .debuginput==.input
The Local Directory | Directory
STDOUT | STDERR | STDIN              -- ~string
STDOUT | READY | READY | READY | READY:   -- .stdout~qualify, states, .stdout~description
1 1 1 1                              -- hasentry OUTPUT STDQUE SYSCARGS TRACEOUTPUT
The NIL object | The NIL object | STDOUT     -- .monitor~new~current, ~destination, .monitor~new(.stdout)~current
STDERR | The NIL object              -- .monitor~new(.stdout)~destination(.stderr), ~destination (pop)
DEBUGINPUT ERROR INPUT OUTPUT STDERR STDIN STDOUT STDQUE SYSCARGS TRACEOUTPUT   -- .local~allindexes~sort
The InputOutputStream class | The Object class   -- .stdout~class~superclasses, .monitor~superclasses
0 0 1 1 0 1                          -- .output~hasmethod say/lineout/unknown, .stdout~hasmethod('say'), unknown~isGuarded, destination~isGuarded
```
`.local` holds **exactly** the ten names in `ORACLE_LOCAL`. With arguments `1 "2 3" 4` (p02b) the
`.syscargs` line is `Array 3 1,2 3,4`. `.stdque` is class `RexxQueue`, string `SESSION`, direct
subclass of Object (p38b agrees) — nothing beyond the brief's ruling. p39: `.stream~new('STDOUT') ==
.stdout` is `0` (a fresh Stream on the same fd is a different object), and `.local~stdout = .stderr`
moves neither SAY nor `LINEOUT('STDOUT',…)` (both still on stdout) — the monitor holds the Stream
object itself, not the name — while `.stdout~lineout('D')` now writes to stderr.

### 2.3 Byte order per descriptor (p01)

**Measured** `<P>/p01/t.rex`:
```rexx
say 'S1'
.output~lineout('O1')
.stdout~lineout('D1')
.stdout~charout('C1')
say 'S2'
call lineout , 'L1'
call charout , 'C2'
.error~say('E1')
.error~lineout('E2')
.stderr~lineout('D2')
.stderr~charout('C3')
.traceoutput~lineout('T1')
call lineout 'STDERR', 'L2'
call lineout 'stdout', 'L3'
call charout 'STDERR', 'C4'
call lineout 'STDERR:', 'L4'
say 'S3'
say result
.stderr~charout('tail')
```
rc 0; stdout `S1$ O1$ D1$ C1S2$ L1$ C2L3$ S3$ 0$`; stderr `E1$ E2$ D2$ C3T1$ L2$ C4L4$ tail` (no
final newline). Program order within each descriptor; a `charout` leaves the next line appended to
it; `lineout` returns `0`. p25 adds: `LINEOUT(,…)`, `CHAROUT(,…)`, `.stdout~lineout/charout`,
`.output~lineout/charout` and `.output~say` all return `0`.

### 2.4 Redirecting `.OUTPUT` (p03, p03j, p04, p25, p26, p40)

**Measured** `<P>/p03/t.rex`:
```rexx
say 'A'
prev = .output~destination(.stderr)
say 'B'
say (prev == .stderr) (prev == .stdout) prev
.output~lineout('C')
call lineout , 'D'
call charout , 'E'
.stdout~lineout('F')
say (.output~current == .stderr)
r = .output~destination
say 'G'
say (r == .stdout) r
say 'I'
```
rc 0; stdout `A$ F$ G$ 1 STDOUT$ I$`; stderr `B$ 1 0 STDERR$ C$ D$ E1$`. `destination(d)` returns
the **new** top (`prev == .stderr`), the pop returns the new top after popping; SAY, `LINEOUT(,)`,
`CHAROUT(,)` and `.output~lineout` all follow the redirection; `.stdout~lineout` does not. p03j
(push `.stderr` then `.stdout`, say, pop, say, pop, say): stdout `A$ C$`, stderr `B$` — a real stack.
p04 with a file `f = .stream~new('<P>/p04/say.txt')`: after `.output~destination(f)` the SAY /
`LINEOUT(,)` / `.output~lineout` lines are in the file (`B$ C$ D$`), `.stdout~lineout('E')` stays on
stdout, after the pop `F` is on stdout, `f~close` answers `READY:`. p26, destination a stream whose
directory does not exist: `say 'A'`, `say 'B'`, `call lineout , 'C'` and `say 'after' result` all
vanish silently, rc 0, and after the pop `f~state` is `ERROR`, description `ERROR:2 No such file or
directory` — SAY through an unwritable destination raises nothing. p40, `.stdout~close` as the first
clause: **every subsequent SAY is lost, stdout empty, rc 0** (Stream~say swallows the NOTREADY).
p25 chained monitors: `.error~destination(.output)`, `.output~destination(.stderr)`: `.error~say`,
`.error~lineout`, `.traceoutput~lineout` all land on stderr; after `.output~destination` (pop),
`.error~say('E3')` lands on stdout; `.output~current .error~current .traceoutput~current` prints
`STDOUT The OUTPUT monitor The ERROR monitor`.

### 2.5 Replacing, emptying or breaking the route (p03b–p03h, p13, p13b, p14, p18, p18b)

| probe | program (after the setup line) | stdout | stderr / rc |
|---|---|---|---|
| p03b `r = .output~destination` (pop the only entry) then `say 'H'` | | empty | `r=The NIL object$ cur=The NIL object$` then the 97 report below, **rc 159** |
| p03c `.output~destination(.nil)`; `say 'H'` | | empty | same 97 report, rc 159 |
| p03d `.output~destination(.object~new)`; `say 'H'` | | empty | 97.1 `Object "an Object" does not understand message "SAY".`, rc 159 |
| p03e `.local~output = .nil`; `say 'H'`; `call lineout , 'L'` | | `H$` | `     3 *-* call lineout , 'L'$` / `Error 97 running <P>/p03e/t.rex line 3:  Object method not found.$` / `Error 97.1:  Object "The NIL object" does not understand message "LINEOUT".$`, rc 159 |
| p03f `.local~output = .object~new`; `say 'H'` | | empty | `     2 *-* say 'H'$` / `Error 97 running <P>/p03f/t.rex line 2: …` / `97.1 … "an Object" … "SAY"`, rc 159 |
| p03g2 `.local~remove('OUTPUT')`; `say 'H'`; `say 'after'` | | `H$ after$` | empty, rc 0 |
| p03g `.local~remove('OUTPUT')`; `say 'H'`; `call lineout , 'L'` | | `H$` | empty, **rc 139 (SIGSEGV)** — see §2.9 |
| p03h `.local~output = .stderr`; `say 'H'`; `call lineout , 'L'`; `say 'after'` | | empty | `H$ L$ after$`, rc 0 |
| p14 `.local~output = .Sink~new` (class with only `say`); `say 'S1'`; `call lineout , 'L1'` | | empty | `[sink say <S1>]$` then `     3 *-* call lineout , 'L1'$` / `Error 97 running … line 3` / `97.1 … Object "a SINK" does not understand message "LINEOUT".`, rc 159 |
| p13b `say 'A'`; `.output~foo` | | `A$` | Monitor report below with `Object "a Stream" does not understand message "FOO".`, rc 159 |

The 97 report **through the Monitor** (p03b/p03c/p10c/p13/p13b) has this exact shape — note the
program name `REXX`, the line `1457` (`CoreClasses.orx:1457`, the `forward` in `Monitor~unknown`)
and the `(no source available)` traceback line ahead of the program's own clause:
```
  1457 *-* Method UNKNOWN with scope "Monitor" in package "REXX" (no source available).$
     4 *-* say 'H'$
Error 97 running REXX line 1457:  Object method not found.$
Error 97.1:  Object "The NIL object" does not understand message "SAY".$
```
Without a Monitor in the way (p03e/p03f/p14) the report is the ordinary one naming the program.
The `.nil` fallbacks are asymmetric: `.local~output = .nil` sends SAY **nowhere but stdout**
(`H` printed) yet `LINEOUT(,)` fails 97.1 on `.nil`; a removed entry makes SAY print and
`LINEOUT(,)` crash. p13 (`.monitor~new(.stdout)` used directly): `lineout`, `say`, `destination`,
`current` all behave as for `.output`; `.monitor~new~lineout('x')` (empty stack) is the same
`.nil` 97.1 through line 1457.

**Nobody can change the routing by redefining methods**: p18 `.stream~define('say', …)` → `Error 98
running … line 2:  Execution error.` / `Error 98.985:  User additions are not allowed to the REXX
language classes.`, rc 158, preceded by `       *-* Compiled method "DEFINE" with scope "Class".` (the
crate agrees byte for byte); p18b `.output~setmethod('say', …)` → forwarded through the Monitor to
the Stream and refused `Error 98.991:  Method SETMETHOD may only be invoked from a method of the same
object or one of its classes.` reported at `REXX line 1457`, rc 158. So the only mutable routing
state is `.local`'s entries and each Monitor's destination stack (§5, D-C2).

### 2.6 Input redirection and end of input (p07, p24, p24b, p24c)

**Measured** p07, stdin `stdin1\nstdin2\n`, `in.txt` = `fa\nfb\nfc\n`:
```rexx
f = .stream~new('<P>/p07/in.txt')
.input~destination(f)
pull a1        ; say '<'a1'>'
v = linein()   ; say '<'v'>'
parse linein l ; say '<'l'>'
say .input~destination
pull a2        ; say '<'a2'>'
say .stdin~linein
say lines()
```
stdout `<FA>$ <fb>$ <fc>$ STDIN$ <STDIN1>$ stdin2$ 0$`, rc 0. p24 (stdin `/dev/null`):
`linein()` → `''`, `.stdin~state` → `NOTREADY`, `lines() chars() .stdin~lines .stdin~chars` → `0 0 0
0`, `.stdin~description` → `NOTREADY:EOF`, `parse pull` → `''` with `.input~state` `NOTREADY`.
**p24c, `call on notready name handler` with the handler printing `NOTREADY at <sigl> <D>`:**
```
  NOTREADY at 2 <STDIN>      q = linein()
  NOTREADY at 4 <STDIN>      parse linein r
  NOTREADY at 6 <STDIN>      pull s
  NOTREADY at 8 <STDIN>      c = charin()
                             n = lines()   -> 0, no condition
                             m = chars()   -> 0, no condition
  NOTREADY at 14 <STDIN>     .stdin~linein
  NOTREADY at 16 <STDIN>     .input~linein
```
So at end of input every *read* raises a trappable NOTREADY with description `STDIN` — **PULL
included**, against `iostrms.xml:681` — and the counts do not. Untrapped, NOTREADY is silent (p24's
first line ran with no trap and no message). p24b with `SIGNAL ON NOTREADY`: `linein()` at EOF
transfers to the label with `sigl` 2.

### 2.7 Trace lines and the error report (p03i, p08a–p08c, p15)

**Measured** p08a: recorder on `.error` and `.traceoutput`, then `say 'ok'`, `zz = 1/0`. stdout
`ok$`; stderr:
```
[T:LINEOUT <TraceObject|     4 *-* zz = 1/0> (n=1)]$
[T:LINEOUT <TraceObject|Error 42 running <P>/p08a/t.rex line 4:  Arithmetic overflow/underflow.> (n=1)]$
[T:LINEOUT <TraceObject|Error 42.3:  Arithmetic overflow; divisor must not be zero.> (n=1)]$
```
rc 214. **The traceback echo, the `Error NN running …` line and the secondary line are three
`LINEOUT`s of TraceObjects to `.TRACEOUTPUT`; nothing goes to `.ERROR` directly.** p08b
`.traceoutput~destination(.stdout)`: the whole report is on stdout, stderr empty, rc 214. p08c
`.error~destination(.stdout)` then `trace r` lines and `yy = 1/0`: trace lines and report all on
stdout (through `.TRACEOUTPUT` → `.ERROR` → `.stdout`). p03i `.local~traceoutput = .stdout`
(replacing the monitor by a Stream): trace lines on stdout. p15 `.traceoutput~destination(f)` with
`f` a file: the file holds `     5 *-* zz = 1$ / >>> "1" / 6 *-* trace off$ / 8 *-* yy = 1/0$` and
both `Error 42` lines; stderr holds only `E1$` from `.error~say('E1')`; rc 214. Exit codes are
`256 − N` throughout: 42→214, 97→159, 98→158, 24→232.

### 2.8 Interactive `TRACE ?` transcripts (p09, p09b–p09h, p10, p10b–p10d, p22, p32, p32b, p33, p33b, p36)

**Measured** `<P>/p09/t.rex`, stdin = `""`, `=`, `say 'from debug' zz`, `trace off`, `""`,
`line-for-pull` (six lines):
```rexx
say 'A'
trace ?r
zz = 1
yy = zz + 1
say 'B'
pull v
say '<'v'>'
say 'C'
```
stdout `A$ from debug 1$ B$ <>$ C$`, rc 0, stderr:
```
       +++ "LINUX COMMAND <P>/p09/t.rex"$
     3 *-* zz = 1$
       >>>   "1"$
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++$
     4 *-* yy = zz + 1$
       >>>   "2"$
     4 *-* yy = zz + 1$
       >>>   "2"$
```
Reading: the banner is the source string (`PARSE SOURCE`'s value) in double quotes with the `+++`
prefix at the usual 7-space offset, emitted once, **before** the first traced clause; the prompt
line is unindented and emitted at the first pause. Pause 1 read `""` → continue; pause 2 read `=`
→ the clause is re-executed and traced again; pause 3 read `say 'from debug' zz` → ran with no
tracing and printed to stdout, the pause then read `trace off` → debug ended **and the pause ended
immediately**: the fifth line `""` was *not* consumed by debug, it is what `pull v` read (`<>`),
and `line-for-pull` was never read. `say 'B'` was not traced. Crate: no banner, `?` dropped,
plain `trace r` transcript of every clause, `pull` reads `""` (first stdin line) — same stdout by
coincidence of this stdin, different stderr.

| probe | stdin | oracle result (Measured) | crate |
|---|---|---|---|
| p22 = p09 program | `/dev/null` | banner + prompt lines, then every clause traced with no pause; stdout `A B <> C` | identical stdout; stderr lacks the two `+++` lines |
| p09b program without `trace ?r`, `RXTRACE=ON` | as p09 | identical shape from clause 1: banner, `1 *-* say 'A'`, prompt, `2 *-* zz = 1` twice (the `=`), `from debug 1` on stdout, `pull` → `<>` | no banner, nothing traced, `<>` |
| p36 `trace off; say 'A'; say 'B' trace()`, `RXTRACE=ON` | blanks | `1 *-* trace off` traced and ignored, both says traced, stdout `A$ B ?R$` | stdout `A$ B O$`, stderr empty |
| p09c `trace ?r; zz=1; trace ?; yy=2; say trace(); trace off; say trace(); say 'end'` | 8 blanks | **in debug every TRACE instruction is traced and ignored**: stdout `?R$ ?R$ end$`, all eight clauses traced | stdout `O$ O$ end$`, tracing stops after `trace ?` |
| p33 same as p09c's first two lines then `yy=2; say 'end' trace()` | `trace ?` then blanks | `trace ?` **from the prompt** ends debug, keeps `R`: `end R`, later clauses traced, no more pauses | `end R` (no pauses ever) |
| p33b | bare `trace` then blanks | ends debug and sets Normal: stdout `end N`, nothing traced after `zz = 1` | `end R`, all traced |
| p09d `trace ?r; a1=1 … a5=5; say 'end'` | `trace 2` then blanks | all six clauses traced; that a2 and a3 did not pause is Inferred from `doDebugPause`'s skip branch (`:4213-4222`) — a transcript cannot show a pause that read a blank | all traced, no pauses |
| p09d2 | `trace -2` then blanks | a2 and a3 are **not traced at all**; a4 onward traced and paused | all traced |
| p09e `say 'A'; trace 3; say 'B'` | — | `     2 *-* trace 3$` / `Error 24 running … line 2:  Invalid TRACE request.` / `Error 24.901:  Numeric TRACE requests are valid only from interactive debugging.`, rc 232 | byte-identical |
| p09e2 `trace ?r; zz=1; trace 2; yy=2` | blanks | `trace 2` **as an instruction while in debug** is traced twice (the pause's re-trace) and then the same 24.901, rc 232 | byte-identical minus the banner/prompt |
| p09f `trace ?r; zz=1; say trace(); call trace 'O'; say trace(); say 'end'` | blanks | `?R$ O$ end$` — the builtin turns debug off from the program; the `call trace 'O'` clause is traced, nothing after it | `R$ O$ end$` |
| p09g `trace ?r; zz=1; say 'end' zz` | `zz = 1/0`, `say 'still' zz`, blanks | stderr after the prompt: `+++ Interactive trace.  Error 42:   Arithmetic overflow/underflow.$` / `+++ Interactive trace.  Error 42.3:  Arithmetic overflow; divisor must not be zero.$` (three blanks after `42:`, two after `42.3:`), then the pause continues and `still 1` reaches stdout; rc 0 | no pauses; `end 1` only |
| p09h `trace ?r; zz=1; say 'end' rc` | `echo hi`, `false`, blanks | each line is uppercased and run as a command: stderr `/bin/sh: 1: ECHO: not found$` `/bin/sh: 1: FALSE: not found$`; **no `+++ "RC(n)"` line**; `rc` stays unset (`end RC`) | `end RC`, no commands |
| p10 `.debuginput~destination(.stream~new('<P>/p10/dbg.txt'))` then `trace ?r; zz=1; say 'B'; pull v; say '<'v'>'`, `dbg.txt` = `say 'dbg1'`, `trace off`, `""` | `stdin-line` | debug reads the file (`dbg1` on stdout, `trace off` ends debug), PULL reads stdin: stdout `dbg1$ B$ <STDIN-LINE>$` | refuses `.DEBUGINPUT` |
| p10b recorder on `.debuginput` returning `.nil` | — | `[D:LINEIN (n=0)]` once per pause (3), a `.nil` reply counts as the null line | refuses |
| p10c `.debuginput~destination(.nil)` | blanks | first pause: the Monitor 97 report (`1457 *-* Method UNKNOWN …`, then `3 *-* zz = 1`, `Error 97 running REXX line 1457`, `97.1 … "The NIL object" … "LINEIN"`), rc 159 | refuses |
| p10d `.local~remove('DEBUGINPUT')` | blanks | every pause reads the null string and continues; banner and prompt still printed; rc 0 | `Directory~remove` refused (Phase 5) |
| p32 internal `call sub` whose body is `trace ?r; yy = 2; say 'in sub' trace(); return`, then `zz = 1; say 'main end' trace()` | blanks | debug is scoped to the call: `in sub ?R`, `main end N`, only lines 7–9 traced (with the call-depth indent) | `in sub R`, `main end N` |
| p32b same with `::routine sub` | blanks | same, framed by `>I> Routine "SUB" in package "<P>/p32b/t.rex".` / `<I< …` | identical minus `?` and prompt |

### 2.9 Programs that crash the oracle (new; keep out of every corpus)

All deterministic, two runs each, rc 139, no message. Not in `corpus/oracle-crashes.txt`, and
this survey cannot add them (read-only).

1. `.local~remove('OUTPUT')` followed by `LINEOUT(,…)` (p03g; SAY alone is fine, p03g2).
2. `.local~remove('INPUT')` followed by `LINEIN()` (p03g3; `PULL` before it answered `''`).
3. A traced clause while `.TRACEOUTPUT`'s current destination lacks `LINEOUT`:
   `.traceoutput~destination(.nil)` (p08d) or `(.object~new)` (p08e), then `trace r; zz = 1`. The
   trace lines and everything after come out **on stdout** (`ok$ 4 *-* zz = 1$ >>> "1"$ 5 *-* trace
   off$ after$`) and the process dies at exit — even when the destination is restored first
   (p08d2, `restored$` printed, still rc 139). With no traced clause (p08d3) it exits 0.

---

## 3. C++ mechanism

**Read** unless marked. Line numbers are in `/home/moritz/dev/repos/ooRexx/interpreter/`.

### 3.1 Construction of the `.local` objects

`RexxClasses/CoreClasses.orx:975-1006`, class `LocalServer`: `init` runs once per process and opens
`.stream~new('STDIN')~~command('open')`, `'STDOUT'` and `'STDERR'` with `'open nobuffer'`
(`:980-982`); `initInstance` runs per interpreter instance and does
```
.local~objectname = "The Local Directory"
.local~setentry('STDIN', input);      .local~setentry('INPUT', .monitor~new(.stdin));   .input~objectname = "The INPUT monitor"
.local~setentry('DEBUGINPUT', .monitor~new(.input));                                    .debuginput~objectname = "The DEBUG INPUT monitor"
.local~setentry('STDOUT', output);    .local~setentry('OUTPUT', .monitor~new(.stdout)); .output~objectname = "The OUTPUT monitor"
.local~setentry('STDERR', error);     .local~setentry('ERROR', .monitor~new(.stderr));  .error~objectname = "The ERROR monitor"
.local~setentry('TRACEOUTPUT', .monitor~new(.error));                                   .traceoutput~objectname = "The TRACE OUTPUT monitor"
.local~setentry('STDQUE', .RexxQueue~new('SESSION'))
```
(`:988-1006`). `SYSCARGS` is not the interpreter's: the launcher puts the argv array into the
instance's local directory, `utilities/rexx/platform/unix/rexx.cpp:202`
`pgmThrdInst->DirectoryPut(dir, rxcargs, "SYSCARGS")` — which is why `rexx -e` may lack it.

### 3.2 The Monitor class (`CoreClasses.orx:1391-1460`, Rexx)

`init`: `expose destination; use strict arg dest = .nil; destination = .queue~new; if arg(1,'e')
then destination~push(dest)` (`:1402-1406`). `destination`: `if arg(1,'e') then destination~push(dest)
else destination~pull; return destination~peek` (`:1419-1424`) — so the reply is the *new* top, a pop
on an empty queue is harmless and `peek` of an empty queue is `.nil`. `current`: `return
destination~peek` (`:1437`). `unknown` **UNGUARDED**: `use strict arg msgname, arglist; forward to
(destination~peek) message (msgname) arguments (arglist)` (`:1439-1457`; `:1457` is the `forward`,
the line every through-the-monitor 97 report names). Nothing else — no `say`, no `lineout`
(measured `hasmethod` 0 0 1, §2.2).

### 3.3 The routes (`concurrency/Activity.cpp`)

| route | function | what it does |
|---|---|---|
| SAY | `sayOutput` `:3214-3230` | `callSayExit`; `stream = getLocalEnvironment(OUTPUT)`; if not null and not `.nil` → `stream->sendMessage(SAY, line)` (reply dropped); else `lineOut(line)` |
| trace / error lines | `traceOutput` `:3171-3206` | `callTraceExit`; a `'P'` in the trace object's OPTION returns; `stream = getLocalEnvironment(TRACEOUTPUT)`; if not null/`.nil` → `sendMessage(LINEOUT, traceObject)` inside `try { } catch (NativeActivation *)` → `lineOut(traceObject->requestString())`; else `lineOut(...)` |
| debug read | `traceInput` `:3238-3266` | `callDebugInputExit`; `stream = getLocalEnvironment(DEBUGINPUT)`; if not null → `sendMessage(LINEIN)`; `.nil` reply → `""`; missing entry → `""` (**`.nil` entry is not special here** — it gets the LINEIN and fails 97, p10c) |
| PULL | `pullInput` `:3275-3296` | `callPullExit`; `STDQUE~PULL`; `.nil` reply → `lineIn(activation)` |
| PARSE LINEIN, and PULL's fallback | `lineIn` `:3322-3348` | `callTerminalInputExit`; `stream = getLocalEnvironment(INPUT)`; if not null → `sendMessage(LINEIN)`, `.nil` → `""`; missing → `""` |
| fallback writer | `lineOut` `:3306-3312` | `printf("%.*s\n", …)` to the process stdout, returns `IntegerZero` |
| PUSH/QUEUE | `queue` `:3355-3376` | `STDQUE~PUSH` / `~QUEUE` (Phase 10) |

Callers: `instructions/SayInstruction.cpp:74` → `context->sayOutput(evaluateStringExpression(…))`;
`instructions/ParseInstruction.cpp:156` (PULL → `pullInput`) and `:164` (LINEIN → `lineIn`);
`RexxActivation.cpp:4716-4736` are the one-line forwards to the activity.

**Builtins with the name omitted** go through `RexxActivation::resolveStream`
(`execution/RexxActivation.cpp:1938-1977`): a null or empty name answers `.INPUT` (input side) or
`.OUTPUT`; `STDIN`/`STDIN:` → `.INPUT`; `STDOUT`/`STDOUT:` → `.OUTPUT`; `STDERR`/`STDERR:` →
`.ERROR` (all `strCaselessCompare`); anything else is a file (stream survey). The builtins then send
the message named after themselves with exactly the arguments supplied:
`expression/BuiltinFunctions.cpp` LINEIN `:2161-2178` (`LINEIN`, `LINEIN line`, `LINEIN line
count`), CHARIN `:2208-2221`, LINEOUT `:2265-2285` (0/1/2 args; a queue-named stream sends `QUEUE`),
CHAROUT `:2315-2328`, plus LINES/CHARS (`LINES` gets the option, measured `NORMAL`). **When
`getLocalEnvironment` answers null, `resolveStream` returns null and the builtin sends to it** — the
crash in §2.9 items 1 and 2.

### 3.4 The error report

`Activity::display` (`Activity.cpp:1414-1483`): each traceback line, then `Error <rc> running <program>
line <n>:  <errortext>` (`running`/`line` are message texts `Message_Translations_running`/`_line`,
`messages/rexxmsg.xml:6326`/`:6335`; the program part is omitted when the program name is null or
empty), then `Error <code>:  <secondary>` if present — every line via
`currentRexxFrame->displayUsingTraceOutput`, which is `processTraceInfo(…, TRACE_OUTPUT, …)`
(`RexxActivation.cpp:5262-5265`) → `createTraceObject` (`:5160`, a `TraceObject` — `CoreClasses.orx:3993`,
`subclass StringTable` — holding TRACELINE, INTERPRETER, THREAD, INVOCATION, STACKFRAME and,
for values, a VARIABLE table) → `activity->traceOutput` (`:5254`). So every displayed line is one
`LINEOUT` of a TraceObject to `.TRACEOUTPUT` (§2.7 measured it). `Activity::displayDebug`
(`:1492-1515`) is the in-pause variant: `Message_Translations_debug_error` = `+++ Interactive trace.
Error` (`rexxmsg.xml:6344-6346`, two spaces inside) `concatWith(rc, ' ')`, `":  "`, then
`concatWith(errortext, ' ')` — hence the three blanks after `42:` — and the secondary line via
`concat` — two blanks.

### 3.5 Interactive debug

State on the activation (`execution/RexxActivation.hpp`): `TraceSetting` flags `traceDebug`,
`pauseInstructions`/`pauseLabels`/`pauseCommands`, `debugToggle` (`TraceSetting.hpp:70-81`;
`setDebug`, `resetDebug`, `toggleDebug` `:113-140`; `setExternalTrace` `:284-286` = `?R`);
`settings.traceSkip`, `isTraceSuppressed`, `isDebugBypassed`, `wasDebugPromptIssued`,
`wasSourceTraced`; the bool `debugPause` (`:632`). `inDebug()` is `isDebug() && !debugPause`
(`:367`); `noTracing()` includes `debugPause` (`:409-410`), which is why nothing is traced while a
prompt line runs.

- **Entering.** `setTrace` (`RexxActivation.cpp:~1010-1041`): a toggle request flips
  `traceDebug`, otherwise the flags are set; leaving debug resets `debugPromptIssued` (`:1027-1031`);
  **if issued while `debugPause`, `setDebugBypass(true)`** (`:1037-1040`) — the mechanism behind
  "the pause ends immediately after `trace off`" in §2.8. `RXTRACE`: `platform/unix/
  SysInterpreterInstance.cpp:63-70` reads the env var once (`ON`, caseless) and `setupProgram`
  (`:107-113`) calls `activation->enableExternalTrace()` (`RexxActivation.cpp:4121-4127`) for each
  program's top-level activation — before its first instruction, which is why the program's own
  `trace off` is then ignored (p36).
- **TRACE instruction in debug** (`instructions/TraceInstruction.cpp:137-197`): the skip form calls
  `debugSkip` (`RexxActivation.cpp:932-945`: `!debugPause` → `Error_Invalid_trace_debug` 24.901;
  else `traceSkip = |n|`, `setTraceSuppressed(n < 0)`); a non-numeric setting is applied only
  `if (!context->inDebug())`, otherwise the instruction just `pauseInstruction()`s — the "ignored"
  of `xintdeb.xml:58` and p09c. The TRACE **builtin** has no such guard (p09f).
- **The pause** `doDebugPause` (`:4199-4283`): return if already in a pause; if `isDebugBypassed`
  clear it and return; else if `traceSkip > 0` decrement (re-enabling tracing at 0) and return; else
  (needs `code->isTraceable()`) print `Message_Translations_debug_prompt` (`rexxmsg.xml:6354`, the
  `+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++` line, through
  `processTraceInfo` with `TRACE_OUTPUT` — unindented) once per activation, then loop:
  `response = activity->traceInput(this)`; length 0 → break; exactly `=` → `next = current; return
  true` (re-execute); else `debugInterpret(response)` and break if the next instruction changed or
  the bypass flag was set. `debugInterpret` (`:2786-2815`) sets `debugPause = true`, compiles the line
  as an INTERPRET at the current line, runs it in a `DEBUGPAUSE` activation (constructor `:205-209`
  turns that into an INTERPRET with `debugPause` set), clears the flag; a syntax error inside is
  displayed by `displayDebug` and unwinds only the pause activation (`:2476-2486`, `:2601-2607`:
  non-SYNTAX conditions are ignored in a pause). Commands from the pause do not set RC or trace
  `RC(n)` because `RexxActivation::command`'s whole RC block is under `if (!debugPause)` (`:4444`);
  the pause after a command is `if (instruction_traced && inDebug()) doDebugPause()` (`:4523-4526`).
- **Where pauses happen**: `pauseInstruction`/`conditionalPauseInstruction`/`pauseLabel`/
  `pauseCommand` (`RexxActivation.hpp:380-383`), gated on the TraceSetting's pause flags — i.e.
  after every clause that was traced under the letter in force.
- **The banner**: `traceClause` (`:4290-4312`) calls `traceSourceString()` (`:4007-4029`) once when
  `inDebug() && !wasSourceTraced()`: 7 blanks, `+++`, blank, `"` + `sourceString()` + `"`, through
  `processTraceInfo(TRACE_OUTPUT_SOURCE)`.
- **Scoping**: the trace setting lives in the activation's `settings` and an internal `CALL` /
  `::ROUTINE` gets its own copy restored on return (p32/p32b, `xintdeb.xml:118-125`).

### 3.6 Exit code

`256 − N` for an untrapped error N (measured 42/97/98/24; the crate already agrees, p09e/p09e2).

---

## 4. The crate today

All **Measured** with the snapshot binary unless a `file:line` is given (**Read**).

**Refusals** (every one `rc 120`, one line on stderr, program output up to that clause kept):

| construct | message | source |
|---|---|---|
| `.OUTPUT` `.INPUT` `.ERROR` `.TRACEOUTPUT` `.DEBUGINPUT` `.STDOUT` `.STDERR` `.SYSCARGS` `.STDQUE` (p01, p06, p08a, p08b, p10, p03i, p03h, p38, p38b; `.STDIN` not probed alone, same list) | `rexx-exec: environment symbol ".OUTPUT" is not implemented (Phase 7)` | `lib.rs:320` (`owned_message`), owner from `environment.rs:387-395` — every `ORACLE_LOCAL` name (`:164-175`) is `Unbuilt { owner: "Phase 7", scope: Local }`; **`STDQUE` should say Phase 10** under the brief's ruling |
| `LINEOUT(,…)`, `LINEIN()`, `CHAROUT(,…)` (p03e, p24, p25) | `rexx-exec: routine "LINEOUT" is not implemented (4c)` | `lib.rs:281` hardcodes `Some("4c")` — D-P7-5's wrong owner |
| `.stream~new(...)` (p04, p07, p15, p26, p39) | `… the LIBRARY REXX entry point "stream_init" is not implemented (Phase 7)` **and a second line** `… "stream_uninit" …` at exit | `dispatch/native.rs` (stream survey) |
| `.local~remove('X')` (p03g2, p10d) | `rexx-exec: method "REMOVE" of class "Directory" is not implemented (Phase 5)` | a 5c gap this area's probes trip over |

**`TRACE ?` today** (p09, p09b, p09c, p09f, p22, p33, p33b, p36): `trace.rs:211-215` `mode_from_setting`
skips every leading `?` ("a debug-pause toggle this non-interactive runtime has nothing to toggle",
and `trace.rs:27`), so `trace ?r` is `trace r` (no banner, no prompt, no stdin consumed), a bare
`trace ?` is `OFF` (`trace.rs:824` asserts it) where the oracle keeps the letter and toggles debug,
`TRACE()` answers `R` for `?R`, TRACE instructions inside debug take effect instead of being ignored
(`O O` vs `?R ?R`), the TRACE builtin's `O` from inside debug agrees by accident, and `RXTRACE` is
never read. `trace 3` outside debug is already byte-identical 24.901 (`trace.rs:273`
`raised_numeric_trace_interactive_only`, p09e, p09e2). The traced-clause transcript itself is
byte-identical wherever the two differ only by `?` (p09d without the skip, p32b with `>I>`/`<I<`).

**Sinks and sources.** `Interp.out` (`lib.rs:1692`, "SAY writes here and `Outcome::stdout` is what
it becomes") and `Interp.trace` (`:1694`, "becomes `Outcome::stderr`"); `run_program` builds
`Outcome { stdout: interp.out, stderr: interp.trace }` (`lib.rs:4902-4903`). `say_evaluated`
(`run.rs:2069-2081`): `required_string_value`, `trace_result(...)`, then `out.extend_from_slice(&line);
out.push(b'\n')` — the direct write architecture fact 4 protects. Trace lines are built by
`trace.rs:311-400` and pushed by `Interp::trace_clause`/`trace_result` (`trace.rs:424-470`) into
`self.trace`; the error report lands there too (`lib.rs:5176-5200` documents the `Error 34 running
<path> line 3:` shape byte for byte). Input: `input.rs:35-88` `Input { Nothing | Stdin | Bytes }`
with one line cursor, `read_line` drops the `\n` and one `\r` before it; `Interp::pull_line`
(`:93-98`) takes the queue head else `linein_line` (`:101-103`), which is
`read_line().unwrap_or_default()` — **no NOTREADY at end of input** (§2.6), and the crate does know
the condition name (`activation.rs:996`, `options.rs:26`), so `CALL ON NOTREADY; PULL v` at EOF may
already be a silent divergence (not run — see §7). `rexx-run.rs:33` `.with_input(ProgramInput::Stdin)`;
`:56-58` writes `outcome.stdout` then `outcome.stderr` **after** the run returns; `ProgramInput` is
`invocation.rs:54-62`. The in-process harness feeds the oracle's stdin through a pipe
(`tests/support/oracle.rs:160-190`) and reads the crate's `Outcome` buffers.

**What already exists.** `Monitor` is Rexx from `CoreClasses.orx` and its four methods are
`answers` rows (`corpus/method-bodies.txt:1078-1081`), so `.monitor~new(...)~destination/current/
unknown` presumably run today (not probed in isolation: every probe touched a `.local` name first).
`TraceObject`'s class methods are `answers` (`:1315-1326`). `Stream` is `loud` throughout
(`:1379-1394`). `.local` itself is a real `Directory` (`environment.rs:179-188`,
`env_seam::Directories`), and `dot_variable` (`:409`) is where a `.NAME` is resolved.

**Seams a Phase 7 implementation plugs into**: (1) `Interp.out`/`Interp.trace` become the sinks
behind two Stream objects (`.STDOUT`, `.STDERR`) — rename `trace` to `err`, it will carry
`.stderr~lineout` too; (2) `Input` becomes `.STDIN`'s source; (3) the ten `Unbuilt` names leave the
map as objects are installed (`environment.rs:387-395`, and `the_two_oracle_directories_share_no_name`
`:199-200` keeps holding); (4) `say_evaluated` gains the route check (D-C2); (5) `trace_clause`/
`trace_result`/the report writer gain the `.TRACEOUTPUT` route (D-C3); (6) `TraceMode` gains the
debug flag and `mode_from_setting` gains the current mode as input (the exclusion row's own
requirement, `phase-4-exclusions.txt:1934-1940`); (7) `Loud::unresolved_call`'s owner (`lib.rs:281`).

---

## 5. Design questions

**D-C1 — What `.STDOUT`/`.STDERR`/`.STDIN` are.** Stream objects (class `Stream`, `~string`
`STDOUT`/`STDERR`/`STDIN`, `~qualify` the same, `~state` `READY`, `~description` `READY:`,
`.stream~new('STDOUT') == .stdout` is `0`) whose backing is not a file but the interpreter's
buffers: writes append to `Interp.out` / `Interp.err`, reads take `Interp.input`'s next line.
Options: (a) a `StreamBacking::Standard(Which)` variant inside whatever the stream survey designs;
(b) separate classes. (a) — the oracle's are ordinary `Stream` instances (`LocalServer~init`) and
`.stream~new('STDOUT')` must make another. Closed state matters: p40 shows a closed `.STDOUT`
swallows every later SAY with rc 0. No `unsafe`, no dependency, no fd. Cost if wrong: none
observable at the descriptor level as long as bytes land in the right buffer in program order.

**D-C2 — SAY's fast path (architecture fact 4).** Semantics to preserve: SAY sends `SAY(line)` to
whatever `.local` holds under `OUTPUT` and drops the reply; a missing entry or `.nil` writes the line
to stdout directly (p03e, p03g2); any other object gets the message and its errors propagate
(p03d/p03f/p14, and the through-the-Monitor 97 report of §2.5). Default route: the bootstrap Monitor
→ `unknown` → `forward` to the top of its `.queue` → `Stream~say` → `lineout` → stdout.
**What can change the route** — measured to be exactly two things, because `.stream~define` /
`.monitor~define` are 98.985 (p18) and `.output~setmethod` is 98.991 (p18b): (i) the `OUTPUT` entry
of `.local` (replaced, removed, set to `.nil`, p03e–p03h, p14, p18b's last two lines never ran);
(ii) the bootstrap Monitor's destination stack (p03, p03b–p03d, p03j, p04, p25, p26). Not
`.local~stdout` (p39). Not method dictionaries.
Options for the check: **(1) observe per SAY** — `local.get(OUTPUT)` is the bootstrap monitor
(pointer compare; cache the directory slot behind a `.local` mutation generation so this is a
generation compare in the common case), then read that Monitor's `destination` object variable
(a fixed slot in its variable dictionary), peek the queue's head (a native collection read), pointer-
compare with the bootstrap `.STDOUT`, check it is open → direct write; else send. **(2) generation
counters only** — a `.local` generation plus hooking every `Queue` mutation is not possible without
knowing which queue is the monitor's; rejected. **(3) native Monitor** — reimplement the four
methods in Rust with a `Vec<ObjRef>` stack so `destination` can maintain a `route` cache directly;
observable differences (`~source`, the `1457 *-* Method UNKNOWN … (no source available)` traceback
line, `isGuarded`) would have to be faked. Recommend **(1)**: a handful of loads and compares per
SAY, no hooks, and it is exact by construction because it re-derives the route from the same state
the slow path would read. Measure it on `rexxcps` and the bench axes (each bench program has one
`say`, rexxcps eleven — whether any sits in a timed section was not read); if the per-SAY cost shows,
cache `(local_gen, monitor, queue)` and re-read only the queue head. Cost if wrong: a redirected SAY
silently on stdout — the exact hazard D-P7-1 deferred; witnesses p03, p03j, p04, p14, p26, p39, p40.

**D-C3 — Trace lines and the report through `.TRACEOUTPUT`.** Every trace line and every line of
the error report is one `LINEOUT(traceObject)` to `.TRACEOUTPUT` (§2.7, §3.4). The same shape of
check as D-C2 keeps the direct write into `Interp.err` while `.TRACEOUTPUT` → bootstrap `.ERROR`
monitor → bootstrap `.STDERR`, open. When redirected, the crate must construct a `TraceObject`
(class exists as Rexx; the fields `createTraceObject` fills, `RexxActivation.cpp:5160-5215`, at
least TRACELINE so `makestring` answers the line) and send `LINEOUT`. Fallbacks: a missing or `.nil`
`.TRACEOUTPUT` entry writes the line to **stdout** (`lineOut`, p03i is the Stream-replacement case,
the `.nil`-entry case was not probed); a destination lacking `LINEOUT` also ends up on stdout and
then **crashes the oracle at exit** (§2.9 item 3) — the crate should write to stdout and exit
normally, and this is a licensed divergence (rc 139 is not an answer). Cost if wrong: trace
transcripts of redirected programs on the wrong descriptor; ooTest's `NullOutput` case (§5 D-C8).

**D-C4 — Input through `.INPUT`, and NOTREADY.** PULL (queue empty), PARSE PULL, PARSE LINEIN,
`LINEIN()`, `LINEIN('STDIN')`, `CHARIN()`, `LINES()`, `CHARS()` send the message named in §2.1 to
`.INPUT`; `.DEBUGINPUT~LINEIN` for the debug read. At end of input every read raises NOTREADY
(description `STDIN`), silent when untrapped, and PULL is not exempt (p24c) — the crate's
`pull_line` must raise through the condition machinery once `.STDIN` exists. Fast path as D-C2 for
PULL (queue then `Input`); the check is cheaper here because PULL is rare. Cost if wrong: programs
with `CALL ON NOTREADY` diverge silently.

**D-C5 — `rexx-run` streaming versus the buffers (architecture fact 1).** With everything buffered
until exit, an interactive session sees no prompt before its read. Options: (a) stream every write
to fds 1/2 as it happens — changes nothing per descriptor but makes `rexx-run` a different code path
from the harness; (b) **flush-before-read**: when `Input::read_line` is about to block on
`Source::Stdin`, drain `Interp.out` and `Interp.err` to the process's stdout/stderr first (a
callback or `Option<Box<dyn Write>>` pair installed only by `rexx-run`; `Outcome` then carries only
what was produced after the last flush, and `rexx-run`'s existing tail write stays correct). Recommend
(b): per-descriptor bytes are unchanged, the harness never installs the sinks (its `ProgramInput` is
`Bytes`/`Nothing`, which never flush), no process-global state is touched by the interpreter itself
(the sinks belong to the one binary that owns the process). Cost if wrong: only the human at a
terminal; no differential can see it — which is also why it needs a `rexx-run`-level test, e.g.
spawning `rexx-run` with a pipe and checking the prompt arrives before stdin is written.

**D-C6 — Interactive debug state.** Add to the per-activation trace state: `debug: bool`, `pause:
bool` (the C++ `debugPause`), `skip: usize` + `suppressed`, `bypass`, `prompt_issued`,
`source_traced`; inherited by internal CALL and `::ROUTINE` activations and restored on return
(p32, p32b). The pause after a traced clause: read a line via `.DEBUGINPUT~LINEIN` (`.nil` → `""`;
missing entry → `""`; the entry `.nil` → 97 through the monitor, p10c), `""` continues, `=`
re-executes the clause (needs the IR's "re-run this op sequence" — the same shape the tree-walker's
`next = current` had; the DO/block undo the C++ comment mentions was not probed), anything else is
compiled and run as an INTERPRET fragment in a frame flagged `pause` (no tracing, no RC/.RS, SYNTAX
displayed as the two `+++ Interactive trace.  Error` lines and swallowed, other conditions ignored),
after which the pause ends if the fragment changed the flow or **set any trace setting** (the
bypass, p09/p33/p33b), else reads again. `TRACE` instructions while `debug && !pause` only pause;
the builtin always sets; numeric TRACE outside a pause is 24.901; `RXTRACE=ON` sets `?R` on each
program's top-level activation before its first clause. Banner and prompt: §2.8 exact bytes. The
`::OPTIONS TRACE ?x` form must be delivered by the same flag but **never run against the oracle**
(`oracle-crashes.txt` entry 9); with piped stdin the crate's answer can only be checked against
itself. Commands typed at the prompt run through ADDRESS (survey D) with no RC line and no RC
variable (p09h). Cost if wrong: every PULL program under `?` reads the wrong lines (the exclusion
row's argument); witnesses in §6 T5.

**D-C7 — `.SYSCARGS`.** An `Array` of the separate argv words (`Array 3 1,2 3,4` for `1 "2 3" 4`),
built by the launcher, not the interpreter. `rexx-run` has the words (`rexx-run.rs:31`), so
`Invocation` needs a `c_args: Option<Vec<Vec<u8>>>` beside the joined string; `None` means no entry
(the `rexx -e` shape). The in-process harness passes `Some(words)` when it wants parity with the
oracle's `run_with(path, args, …)`. Cheap; no dependency.

**D-C8 — What ooTest needs** (Read): `ootest/worker.rex:129-136` pushes a file stream
(`.stream~new(logFile)~~command("open write" mode)`) on `.output`, prints the report through
`testResult~print`, then pops with `.output~destination`; `framework/OOREXXUNIT.CLS:1538`
`.error~say(...)`, `:1541-1543` and `:1554-1556` push `.stdout`/`.stderr`/`.error` back onto the three
output monitors before and after every test (so the stacks **grow by three per test** and are never
popped — the design must not cap the stack or treat a re-push of the default as a no-op);
`:1635-1650` push `.NullOutput~new` (`:2282-2287`: a class with `lineout` returning 0 and an empty
`say`) on all three and then push the defaults back; `ooTest.frm:2445-2466` `.stdout~charout(msg)`,
`.stdout~lineout(".")`, `.stdout~charout(".")` from an unguarded ticker method (a second thread —
Phase 6 must let two activities append to `Interp.out`). The API exit test programs under
`ootest/ooRexx/API/oo/tests/` redirect `.error` to `self~new` and `.input` to a trap object (e.g.
`ioExitPull.rex:43,55`, `traceExit.rex:43,68`; the grep was truncated, so no count is given);
`TRACE ?` appears in `base/keyword/TRACE.testGroup`, `base/directives/OPTIONS.testGroup`,
`base/runtime.objects/environmentEntries.testGroup`, `API/oo/tests/ioExitDebug.rex` (not read).

**D-C9 — The through-the-Monitor 97 report.** `Error 97 running REXX line 1457` with the
`  1457 *-* Method UNKNOWN with scope "Monitor" in package "REXX" (no source available).` traceback
line first (§2.5). Whether the crate renders `(no source available)` lines for `CoreClasses.orx`
frames and names the package `REXX` with the `.orx` line number was not checked; if not, it is
Phase 5 debt that T2's negative witnesses will expose.

**D-C10 — `.local` writes are rare and the Monitor stack is Rexx-visible state.** The route caches
in D-C2/D-C3 must be invalidated by *any* mutation of `.local` (`put`, `setentry`, `[]=`, `remove`,
`setmethod` on the directory) — one generation counter on that one Directory — and must re-read the
Monitor's queue head each time (or after a queue-identity check). Nothing else can move the route
(§2.5). Thread safety belongs to Phase 6, but the state is all heap objects, no globals.

---

## 6. Proposed task slices

Each names its oracle witnesses (programs in `<P>/…`, to be copied into `corpus/` with fresh names)
and the paired negative or adjacent case. Order matters: T1 unblocks everything.

**T1 — The ten `.local` objects exist.** `.STDIN`/`.STDOUT`/`.STDERR` as Streams over `Input` /
`Interp.out` / `Interp.err`; the five Monitors built by running the `LocalServer~initInstance`
equivalent (object names included); `.SYSCARGS` from `Invocation`; `.STDQUE` stays `Unbuilt` with
owner **Phase 10**; the excluded-builtin owner message fixed with an assertion (D-P7-5); the
`stream_uninit` second refusal line gone. Witnesses: p02 and p02b (identities, names, `hasmethod`,
`.local~allindexes` — the crate's `.local` must list exactly those ten), p38, p13 (a Monitor used
directly, including the `.nil` 97 through line 1457), p39 (`.local~stdout = .stderr`). Negative:
p13b (`.output~foo` → 97.1 naming `a Stream`), p18 (98.985, already green).

**T2 — Output routes through `.OUTPUT` and `.ERROR`, with the SAY fast path (D-C2).** SAY,
`LINEOUT(,…)`, `CHAROUT(,…)`, `LINEOUT('STDOUT'|'STDERR'|'STDIN'…)`, `.output~say/lineout/charout`,
`.error~say/lineout`. Witnesses: p01 (byte order and returns), p03, p03j, p25 (chained monitors,
returns), p05 (the recorder — the message names and argument counts, byte for byte), p14, p03h,
p03e. Negative/adjacent: p03b, p03c, p03d, p03f (97 reports, exit 159), p04 and p26 (file
destinations — need T-stream from the stream survey; p26 needs the silent-loss behaviour), p40
(closed `.STDOUT` swallows SAY, rc 0), p03g2 (removed entry: SAY prints). **Not** p03g (crashes the
oracle; the crate's own answer for `LINEOUT(,)` with no entry is a decision — recommend 97.1 on
`.nil` as for p03e, recorded as licensed).

**T3 — Trace and the error report through `.TRACEOUTPUT` (D-C3).** Witnesses: p05's `[T:…]`
lines (TraceObject class and `makestring`), p08a, p08b, p08c, p03i, p15 (needs T-stream).
Negative: p08d/p08e are oracle crashes — pair the crate's own expectation (lines on stdout, exit 0)
as a self-witness with the divergence licensed in the record.

**T4 — Input routes through `.INPUT`, NOTREADY at end of input (D-C4).** Witnesses: p06 (the
recorder — `LINEIN` with no arguments everywhere, `LINES('NORMAL')`, `CHARS`, `CHARIN`; queue
first), p07 (file source, then back to stdin; needs T-stream), p24, p24b, p24c (every read raises,
the counts do not; `SIGNAL` and `CALL` forms). Negative: p03g3 is an oracle crash; pair p03g2's
shape for `.local~remove('INPUT'); pull v` (answers `''`, measured before the crash line).

**T5 — Interactive debug (D-C6).** T5a the flag: `mode_from_setting` takes the current mode,
`TRACE ?` toggles, `TRACE()` reports `?X`, TRACE instructions ignored in debug, the builtin not,
RXTRACE — witnesses p09c, p09f, p36 (stdout is enough for these), p09e2. T5b the pause: banner,
prompt, `""`, `=`, fragments, bypass, skip — p09, p09b, p22, p09d, p09d2, p09g, p33, p33b, p32,
p32b, p10, p10b, p10d. T5c commands at the prompt — p09h, after survey D's ADDRESS lands (no RC
line, no RC variable). Negative: p10c (97 through the monitor at the first pause), p09e.
Never `::OPTIONS TRACE ?x` against the oracle; the harness rows that feed stdin must keep clear of
`?` until T5b is in (the exclusion row's warning at `phase-4-exclusions.txt:1999-2006`).

**T6 — `rexx-run` flush-before-read (D-C5).** A `rexx-run`-level test with a pipe; no corpus
witness can see it.

---

## 7. Not done / not established

- **Three new oracle crashes** (§2.9) are recorded only here; `corpus/oracle-crashes.txt` is
  read-only for this survey and needs entries 10–12 (the third, `.traceoutput~destination(.nil)` +
  a traced clause, crashes at exit even after the destination is restored).
- `.STDIN` was not probed as the *first* `.local` name in a program; `.local~traceoutput = .nil`
  (entry `.nil`, not the destination) and `.local~error = .nil` were not probed — the C++ says
  `traceOutput` treats a `.nil` entry as missing (stdout fallback) and `sayOutput` likewise, but
  `traceInput`/`lineIn` do not (§3.3); only the SAY case is measured (p03e).
- Whether the crate today, with `CALL ON NOTREADY` and stdin at EOF, runs `PULL v` silently where the
  oracle raises (§2.6) — not run; the crate side of p24c stopped at `LINEIN()`.
- Whether the crate renders the `(no source available)` traceback line and `running REXX line 1457`
  for a `CoreClasses.orx` frame (D-C9) — not checked.
- Whether SAY sits inside a timed section of `rexxcps` or any bench axis (D-C2's cost) — counted, not read.
- The `=` re-execute over a block instruction (`DO`, `SELECT`, `IF`) and the "undo side effects" the
  C++ comment at `RexxActivation.cpp:4251-4253` alludes to — not probed.
- A debug fragment that changes flow (`SIGNAL`, `LEAVE`, `EXIT`, `RETURN`) — not probed; the C++
  ends the pause on `currentInst != next` (`:4266`).
- `TRACE ?L` / `?C` / `?E` / `?F` pause points (labels and commands only), `?A` versus `?I`
  pausing on the same clauses — Read (`RexxActivation.hpp:380-383`), not measured.
- `.output~destination(.output)` (a monitor forwarding to itself) — not probed; presumably a
  recursion the oracle may not survive.
- The exact field set a crate-built `TraceObject` must carry beyond TRACELINE for user code that
  inspects it (`xintdeb.xml:536-`) — the C++ list is at `RexxActivation.cpp:5160-5215`, not
  compared against the documented table.
- `setTrace`'s start line is approximate (`RexxActivation.cpp:~1010`); the bypass and prompt-reset
  lines (`:1027-1040`) were read.
- The two oracle divergences that are *not* this area's: `Directory~remove` (Phase 5) and
  `.stream~new` (stream survey), both hit repeatedly here.
- `HANDLE:x`, system exits, `rexx -e`, Windows — out of scope, listed in §1.

<!-- SURVEY COMPLETE -->
