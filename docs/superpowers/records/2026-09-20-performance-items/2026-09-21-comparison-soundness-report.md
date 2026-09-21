# Item 4's precondition: can a comparison operator answer something other than `0` or `1`?

Read-only investigation, 2026-09-21. No file in the repository was edited and no
`cargo` command was run. Tree at `bc26d7936`, `git status --porcelain` empty when
this started (HEAD had already moved past the `4ef6af5bb` the fusion brief names).

**Another agent is editing `ir.rs`, `ir/compile.rs` and `ir/drive.rs` while this
was written** -- by the end of it `git status` showed all three modified, an
`OpIx` type where `target: u32` had been, and line numbers shifted by up to
seventeen. Every citation below was re-resolved against the working tree at the
end; they will drift again. `Op::Condition` and `Op::JumpUnless` both still
exist and neither the check nor the branch has changed, so nothing here is
affected in substance.

**Verdict: item 4's stated precondition is false. Every one of the eighteen
comparison operators can answer an arbitrary value, including a non-string
object. `Op::Condition` is not a no-op and its check cannot be dropped.**

**A second, independent finding kills it again on the measurement side:** in
`rexxcps`'s own rendered op stream, `Binary` is adjacent to `Condition` at
**0 of 19** sites, because `rexxcps` contains `TRACE` and a `TraceOperator` op
sits between them. Item 3's pair is adjacent at all 19. Section 4 has the counts.

---

## 1. The operator set, derived

The set is taken from every place in the crate that names it, each extracted by a
command rather than read off. Three of them are comparison sets proper (B, C, D)
and they agree exactly; A is the operator identity set they are drawn from, E and
the scanner are the spellings.

**A. The operator identity set** (the lexer's own enum), 32 entries:

    /bin/sed -n '/^pub enum Operator {$/,/^}$/p' \
      rust/crates/rexx-parse/src/token.rs | /bin/grep -a '^    [A-Z]' | tr -d ' ,'

`rexx-parse/src/token.rs:115`. Output: 32 lines, of which `Backslash` is
prefix-only and the rest are dyadic.

**B. What the grammar treats as a comparison** -- precedence level 3 in
`rexx-parse/src/expr.rs:40`:

    /bin/sed -n '/^fn precedence(op: Operator) -> u8 {$/,/^}$/p' \
      rust/crates/rexx-parse/src/expr.rs \
      | /bin/sed -n '/Operator::Equal$/,/=> 3,$/p' \
      | /bin/grep -ao 'Operator::[A-Za-z]*' | /bin/sed 's/Operator:://' | sort

**C. What `apply_binary` routes to `compare_values`** --
`rexx-exec/src/eval.rs:1468`:

    /bin/sed -n '/^fn is_comparison(op: Operator) -> bool {$/,/^}$/p' \
      rust/crates/rexx-exec/src/eval.rs \
      | /bin/sed -n '/matches!(/,/^    )$/p' | /bin/grep -av 'matches!\|^ *op,' \
      | /bin/grep -ao '[A-Za-z][A-Za-z]*' | sort

**D. What `compare_op` maps to a `CompareOp`** -- `rexx-exec/src/eval.rs:1423`:

    /bin/sed -n '/^fn compare_op(op: Operator) -> CompareOp {$/,/^}$/p' \
      rust/crates/rexx-exec/src/eval.rs \
      | /bin/grep -a '=> CompareOp::' | /bin/sed 's/ *=>.*//' | tr '|' '\n' \
      | /bin/grep -ao '[A-Za-z][A-Za-z]*' | sort

`diff` of B against C and of C against D: **identical, 18 entries each**,
exit status 0 for both diffs. That is the whole point of taking it from three
places: an operator in the grammar's comparison level but missing from
`is_comparison` would fall through `apply_binary`'s last arm to
`Loud::binary_operator`, and an operator in `is_comparison` but missing from
`compare_op` would hit that function's `unreachable!`. Neither gap exists.

**E. The canonical spelling of each**, which is also the method name sent when
the left operand is an object (`eval.rs:1288` passes `op.spelling()`):

    /bin/sed -n '/^    pub fn spelling(self) -> &.static str {$/,/^    }$/p' \
      rust/crates/rexx-parse/src/token.rs | /bin/grep -a '=> "' \
      | /bin/sed -e 's/^ *Operator:://' -e 's/ *=> */\t/' -e 's/,$//' | sort \
      > /tmp/spellings.tsv
    # with command C's output saved as /tmp/set-C.txt:
    join -t $'\t' /tmp/set-C.txt /tmp/spellings.tsv

| variant | spelling |
|---|---|
| `Equal` | `=` |
| `BackslashEqual` | `\=` |
| `GreaterThan` | `>` |
| `BackslashGreaterThan` | `\>` |
| `LessThan` | `<` |
| `BackslashLessThan` | `\<` |
| `GreaterThanEqual` | `>=` |
| `LessThanEqual` | `<=` |
| `StrictEqual` | `==` |
| `StrictBackslashEqual` | `\==` |
| `StrictGreaterThan` | `>>` |
| `StrictBackslashGreaterThan` | `\>>` |
| `StrictLessThan` | `<<` |
| `StrictBackslashLessThan` | `\<<` |
| `StrictGreaterThanEqual` | `>>=` |
| `StrictLessThanEqual` | `<<=` |
| `LessThanGreaterThan` | `<>` |
| `GreaterThanLessThan` | `><` |

**Every spelling of each.** The scanner's byte arms (the `b'='` arm at
`rexx-parse/src/scanner.rs:532` through the `b'\\'` arm) add source spellings
without adding operators: `b'\\' | 0xAA | 0xAC` share one arm
(`scanner.rs:574`), so `¬=`,
`¬==`, `¬>`, `¬>>`, `¬<`, `¬<<` (and the same six with byte `0xAA`) are
alternative spellings of the six `\`-prefixed variants.
They produce the same `Operator` and therefore the same behaviour; probed below.

There is no sixth producer: the only synthetic `Expr::binary` construction sites
are `instruction.rs:465` and `:501`, both for the `+=`-family assignment
shortcuts, and `scanner.rs:407`'s `check_assignment` is reached only from the
`+ - % / * ** & |` bytes, never from a comparison byte.

## 2. Per-operator verdict

**All eighteen: can return a value that is not `0` or `1`. Same mechanism, same
line.**

`Interp::apply_binary` (`rexx-exec/src/eval.rs:1272`) opens with

    rust/crates/rexx-exec/src/eval.rs:1287
        if let Some(target) = self.operator_message_receiver(left) {
    rust/crates/rexx-exec/src/eval.rs:1288
            return self.send_operator(op.spelling(), target, &[Some(right)]);

This is **ahead of the `op` dispatch at `:1304`**, so it applies to every
operator uniformly -- comparison, concatenation and logical alike. It returns
whatever `send_message` returns (`eval.rs:1209`), unvalidated. The two literal
`logical(...)` sites a comparison would otherwise reach
(`eval.rs:1016`, `:1066`, `:1090`) are never consulted on this path.

`Interp::operator_message_receiver` (`eval.rs:1160`) names five receiver shapes:
`ObjRef::NIL`, `Body::Class`, `Body::Instance`, a `Body::Stem` whose default is
one of these, and a `Body::VarRef` that references one. `Body::Instance` is a
user class instance and can define a method for any of the eighteen spellings.

### The probe, against the oracle

Eighteen programs, one per operator, of the shape

    o = .Kustom~new
    say 'ret:' (o <op> 1)
    if o <op> 1 then say 'then'
    say 'unreached'

    ::class Kustom
    ::method '<op>'
      return 'banana'

run as

    ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
      /home/moritz/dev/repos/ooRexx/build/bin/rexx $D/opNN.rex )

from a fresh empty directory. **Result, 18 of 18 identical:**

* exit status **222**
* stdout: `ret: banana`
* stderr: `Error 34.1:  Value of expression following IF keyword must be exactly "0" or "1"; found "banana".`

So the value a comparison answers is `banana`, and it is `Op::Condition`'s
counterpart in the oracle that turns it into 34.1. There is no operator among
the eighteen for which this fails.

### The paths that are not the obvious one, each probed

| route | program | oracle result |
|---|---|---|
| user class instance | `o = .Kustom~new; if o = 1` | rc 222, `34.1 ... found "banana"` |
| `WHEN` rather than `IF` | `select; when o = 1 then ...` | rc 222, **`34.2`** `... following WHEN keyword ... found "banana"` |
| stem default is the object | `st. = .Kustom~new; if st.anytail = 1` | rc 222, `34.1 ... found "banana"` |
| class object via a metaclass | `::class Meta subclass Class` with `::method '='`; `if .Kustom = 1` | rc 222, `34.1 ... found "banana"` |
| alternative spelling, byte `0xAC` | `if o ¬= 1`, method named `\=` | rc 222, `34.1 ... found "banana"` |
| the answer is an **object**, not a string | `::method '=' ; return .Kustom~new` | rc 222, `34.1 ... found "a KUSTOM"` |

The last row matters most for the fusion: the register handed to
`Op::JumpUnless` would hold a heap handle, not text.

A `NaN`-like or uninitialised operand is **not** a route: an unset Rexx variable
reads as its own name, a string, so `if zzz = 1` is an ordinary string
comparison. The routes that exist are all "the operator became a message send".

### The Rust interpreter agrees, measured

`target/release/rexx-run` was already built (mtime `2026-09-21 10:08:47`, sha256
`f9390c5e13436255c7baa38ba5e720d532ca1e6b86c807310a7059764b7735a3`), so running
it needed no `cargo`. The same eighteen programs:

    $R/target/release/rexx-run $D/ops/opNN.rex

**18 of 18 identical to the oracle on all three descriptors**: rc 222, stdout
`ret: banana`, stderr `Error 34.1: ... found "banana".` This is not a reading of
the Rust source, it is a run of it.

### What the Rust side does with it

`Op::Condition`'s driver arm (`rexx-exec/src/ir/drive.rs:1671`) fast-paths
`LOGICAL_TRUE`/`LOGICAL_FALSE`/`SmallInt(0|1)` and otherwise enters
`Interp::condition_value` with `keyword.raiser()`, which is
`raised_if_not_logical` or `raised_when_not_logical` (`ir.rs:300`). That is
the 34.1 / 34.2 above.

`Op::JumpUnless` reads the register through `register_holds`
(`drive.rs:2303`, failing at `:2317`), whose failure arm is

    rust/crates/rexx-exec/src/lib.rs:601
        /// A register a branch op reads holds something that is not a Rexx
        /// logical value -- an internal inconsistency, never a program error.
        fn register_not_logical() -> Loud {

**"an internal inconsistency, never a program error" is exactly the invariant
`Op::Condition` establishes.** Drop the validation and a program that today
reports 34.1 on stderr at rc 222 reports a loud internal failure instead: a
differential divergence on stderr and on status, from ordinary user Rexx.

`Op::Condition` also owes the `>>>` trace line (`ConditionTrace::Result(indent)`
at `drive.rs:1729`), so it is not a no-op in a second, independent way.

## 3. Is item 4 sound?

**No, as written.** Its text says the fusion turns `Binary` + `Condition` +
`JumpUnless` into one op on the premise that "`Op::Condition` is not a no-op and
the fusion is unsound" if any comparison can answer something else. One can, all
eighteen can, so the premise fails and the item as stated is dead.

What survives, stated once and not pursued: the *dispatch* saving does not depend
on the check being dead. A fused op that still validates -- exactly the shape
`op-fusion-brief.md` already specifies for item 3, "validate the value, emit the
`>>>` line it owes, and branch" -- removes two dispatches per site and changes no
behaviour. Item 4 is then "also fold the `Binary` in", and its justification has
to be the dispatch count, never "the check is provably dead". No static
restriction rescues the original premise: the operands' types are not known at
compile time, and the stem-default row above shows a compound read is as capable
of yielding an object receiver as a bare variable is.

## 4. Coverage: how much of `Op::Condition` is comparison-rooted

`Op::Condition` is emitted for **every** `IF` and plain `WHEN` whose condition
`native_shape` accepts (`ir/compile.rs:401` and `:593`), and `native_shape`
(`compile.rs:1191`) accepts `Literal`, `Constant`, `Variable`, `Stem`,
`Compound`, `Call` (when the node has an address), any `is_native_binary`
operator, and `Prefix`. So a comparison root is one case among many: a literal
(`if 1 then`), a bare variable (`when flag then`), a call, an arithmetic or
concatenation or `&`/`|`/`&&` root, and a prefix `\` all reach `Op::Condition`
too, and I checked each by rendering rather than by reading:

    zv = 1
    if zv=1, zv=1 then say 'comma'
    if length(zv)  then say 'call'
    if \zv         then say 'prefix'
    if zv || ''    then say 'concat'
    if zv & zv     then say 'and'

gives `CallArgs`+`TraceFunction`, `Prefix`, `Binary op=||` and `Binary op=&` as
the four producers of a `Condition` register. **The comma list is the exception
and I had it wrong from reading**: `if zv=1, zv=1` compiles to `EvalExpr` then
`JumpUnless` with **no `Condition` op at all**, because the `EvalExpr` path
validates inside `Interp::eval_if_condition`. So `JumpUnless` is the op emitted
for every `IF`/`WHEN` and `Condition` is emitted only for the native ones; their
being exactly equal at 5,900,006 is itself the evidence that `rexxcps` has no
comma-list or other non-native condition, which the `EvalExpr: 0` count below
confirms directly.

Parentheses are **not** a case: `subterm`'s `LeftParen` arm
(`rexx-parse/src/expr.rs:227`, whose body ends `Ok(inner)`) returns the inner
expression with no wrapper node, and `ExprKind` has no `Paren` variant. So
`if (a = b) then` is comparison-rooted.

### The figure for `rexxcps`, from the rendered op stream

`target/release/rexx-ir` was already built (mtime `2026-09-21 10:08:06`, sha256
`3f91e0ac39eda1b203abec64f766ee50fc32295a44bac03dfe9b29a73855fa67`), so the
static side is measured rather than reasoned:

    $R/target/release/rexx-ir rust/bench-rexxcps/rexxcps.rex > rexxcps.ir   # rc 0

    /bin/grep -ac ' Condition '  rexxcps.ir   ->  19
    /bin/grep -ac ' JumpUnless ' rexxcps.ir   ->  19
    /bin/grep -ac ' WhenTest '   rexxcps.ir   ->   0
    /bin/grep -ac ' EvalExpr '   rexxcps.ir   ->   0

Nineteen static sites, no `WhenTest` and no `EvalExpr`: **every `IF` and `WHEN`
in `rexxcps` is native-shaped**, which is why the item's two executed counts are
exactly equal. Walking back from each `Condition` past the trace ops to the op
that wrote its register gives the root of each, measured:

    /bin/grep -an ' Condition ' rexxcps.ir | cut -d: -f1 | while read -r ln; do
      i=$((ln-1))
      while :; do l=$(/bin/sed -n "${i}p" rexxcps.ir)
        case "$l" in *" Trace"*) i=$((i-1));; *) break;; esac; done
      printf '%s | %s\n' "$(/bin/sed -n "${ln}p" rexxcps.ir)" "$l"
    done

**16 of the 19 producers are `Binary op=` with a comparison spelling; 3 are not**
-- op 371 and op 450 are `Load read=Simple` (the two `when flag`), op 405 is
`LoadConstant` (`if 1`). That is the static answer to the second question, and it
matches the table below line for line.

Per-1000-clause-body execution counts, from `rust/bench-rexxcps/rexxcps.rex`
(`count=200`, `averaging=100`, so 20,000 bodies for the 20,000,000-clause
baseline):

| line | condition | root | exec/body |
|---|---|---|---|
| 47 | `if flag=acompound.key1.loop` | comparison `=` | 14 |
| 49 | `if j>acompound.key1.loop` | comparison `>` | 28 |
| 50 | `if 17<length(j)-1` | comparison `<` | 28 |
| 51 | `if j='foobar'` | comparison `=` | 28 |
| 52 | `if substr(1234,1,1)=9` | comparison `=` | 28 |
| 53 | `if word(key1,1)='?'` | comparison `=` | 28 |
| 54 | `if j<5` | comparison `<` | 28 |
| 56 | `if j=2` | comparison `=` | 28, **not in the counted dispatch** |
| 62 | `when flag='string'` | comparison `=` | 14 |
| 63 | `when avar.flag.2=0` | comparison `=` | 14 |
| 64 | `when flag=5+99.7` | comparison `=` | 14 |
| 65 | `when flag` | **Variable** | 14 |
| 66 | `when flag==0` | comparison `==` | 1 |
| 68 | `if 1` | **Literal** | 14 |
| 70 | `when flag=='ring'` | comparison `==` | 14 |
| 71 | `when avar.flag.3=0` | comparison `=` | 14 |
| 72 | `when flag` | **Variable** | 14 |
| 73 | `when flag==0` | comparison `==` | 0 |
| 99 | `if left(tracevar,1)='O'` | comparison `=` | 1 per program |

The counts follow from `do loop=1 to 14`, `do j=1.1 to 2.2 by 1.1` (two passes),
and `flag` being `0` only on the first `loop` pass, which is what makes line 66
reachable once and line 73 never.

**The model reconciles with the measurement, twice.** Summing the table without
line 56 gives **295 per body**; `295 x 20,000 = 5,900,000` against the measured
`Condition` and `JumpUnless` counts of **5,900,006** in
`2026-09-20-instructions-per-op.md`, residual 6, of which line 99 is 1. The same
model applied to `Op::Arith` -- `length(j)-1` 28, `acompound...+1` 28 (line 55),
`5+99.7` 14, `avar.1.2*1.1` 13 and 14 -- gives 97 per body, 69 once line 55 is
dropped for the same reason line 56 is; `69 x 20,000 = 1,380,000` against the
measured **1,380,205**, residual 205.

Lines 55 and 56 are the only two clauses inside the `if j<5 then do ... end`
block, and both are exactly the amount by which the model overshoots. The
rendered stream shows why they are different from their neighbours: the `DO`
block compiles to `275: Clause index=49 ; 54: do` followed by
`276: LoopRun index=49`, so its body (ops 277-300, including the `Arith op=+` at
282 and the `Condition` at 292) is entered through `Op::LoopRun` rather than
walked by the enclosing region. The clauses of the `do j` loop around it are not:
they sit flat in the same stream. **I did not establish why a `LoopRun` body's
dispatches are outside the counted total** -- the count was taken by summing
jump-table addresses in `run_ops_from` instantiations, which is the kind of
method that can miss one. What the two residuals, 6 and 205, do establish is that
the rest of the model is right.

**Coverage:**

* of the **counted** 5,900,006: comparison-rooted is `295 - 14 - 14 - 14 = 253`
  per body, **253/295 = 85.8%**
* of the true total including line 56: **281/323 = 87.0%**

Call it **~86%**. The non-comparison remainder is three sites -- one literal
(`if 1`) and two bare variables (`when flag`) -- at 42 of 295 per body.

### What that is worth

At 20.0 Ir per dispatched op and 21,147,250,696 Ir for the baseline:

* Item 4 removes 2 ops per covered site: `0.86 x 5.9M x 2 = 10.1M` ops,
  `202 MIr`, **~0.96%** -- against the item's stated "up to ~1.1%".
* But item 3 already removes 1 op per site **unconditionally**: 5.9M ops,
  ~0.56%. So item 4's **marginal** gain over item 3 is the `Binary` op alone on
  the covered subset: `0.86 x 5.9M = 5.1M` ops, `101 MIr`, **~0.48%**.

That 0.48% is the ceiling, and the next finding takes it to zero on this program.

### The adjacency item 4 needs does not exist in `rexxcps`

`op-fusion-brief.md` requires a **statically adjacent** pair, and that is what
makes the fusion's soundness question go away. Counting adjacent pairs in the
rendered stream:

    adj () { awk -v a="$1" -v b="$2" '$0 ~ a && p ~ b {n++} {p=$0} END{print n+0}' "$3"; }

| stream | `Condition` sites | `Condition` preceded by `Binary` | `JumpUnless` preceded by `Condition` |
|---|---|---|---|
| `rexxcps.ir` | 19 | **0** | 19 |
| a trace-free control program | 3 | 2 | 3 |

**Zero.** `rexxcps` contains `TRACE` instructions and emission is per chunk, so a
`TraceOperator` op sits between the `Binary` and the `Condition` at every one of
the sixteen comparison-rooted sites (op 270 `Binary op=<`, op 271 `TraceOperator
op=<`, op 272 `Condition`). Item 3's pair is adjacent at all nineteen; item 4's
is adjacent at none. The control -- the same rendering of

    zv = 1
    if zv = 1 then say 'yes'
    select
      when zv > 0 then say 'pos'
      otherwise nop
    end
    if zv then say 'bare'

-- has the pair adjacent at 2 of its 3 sites (ops 6/7/8 and 19/20/21), which is
what shows the zero above is the `TRACE` and not the rendering.

So **item 4 buys nothing measurable on `rexxcps` until item 1 or item 2 lands**,
and item 2 was built and reverted. Fusing across an intervening op is a different
and larger change than the brief describes, because the trace op it would have to
absorb still has to emit.

## 5. Concerns

1. **The 0.48% marginal figure is derived, not measured**, and it rests on the
   20.0 Ir/op constant from the TRACE A/B. Project memory records that per-clause
   conditionals in the driver cost ~0.5% on `rexxcps` whatever they do; an added
   jump-table arm is cheaper than that, but it is not free by assumption.
2. **`rexxcps` is one program.** 86% is its number. A program whose conditions
   are mostly `if \flag` or `if f(x)` would get much less.
3. **The behaviour that kills item 4 is not covered by the suite.**
   `/bin/grep -rlna "method '='\|method \"=\"" rust/crates/ rust/corpus/` returns
   nothing, so no test and no corpus program defines a comparison operator as a
   method. Both interpreters get it right today and neither is pinned to.
4. `rexx-run` and `rexx-ir` were built at 10:08 today, before HEAD reached
   `bc26d7936`, so they are near but not necessarily at HEAD; their sha256s are
   quoted beside the figures. A gate run was said to be in flight, so I did not
   rebuild and did not take the `target/` lock. Nothing measured here is
   plausibly revision-sensitive -- the send path in `apply_binary` and the shape
   of `Op::Condition` are both older than this week -- but it is not pinned.
5. HEAD moved from `4ef6af5bb` to `bc26d7936` during this task. Nothing here
   depends on which of the two it is.
