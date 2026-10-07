# Scout D: subclass `NEW` on String, Stem, Method, Routine, Message, VariableReference

HEAD `fc04e3857`. Built from a worktree copy (`/tmp/claude-1000/p61/sd/wt`, target
`/tmp/claude-1000/p61/sd/target`, both removed). A throwaway prototype was written in that copy to measure
the design rather than guess it; its diff is `scout-d-prototype.patch` beside this file (not for landing:
no docs, no tests, prototype names).

## Verdict

Each class alone is M or less; the sum is L. Under R2 as written, subclass `NEW` does not land in 6.1 and
Phase 9's row is amended. Two pieces do not follow that rule and need their own disposition (section 4):
VariableReference's 93.967 (S, a wrong answer, one row) and a second route to b14's refusal sites that
does not go through `NEW` at all (class objects).

| Class | Size | With tests (est.) |
|---|---|---|
| VariableReference | S | ~20 lines |
| Message | S/M boundary | ~65 |
| Method + Routine | M | ~130 |
| Stem | M | ~140 |
| String | M, upper end | ~260 |
| **Total** | **L** | **~600** |

Prototype measured: `git diff --stat` in the copy, "8 files changed, 176 insertions(+), 25 deletions(-)",
for String, Stem, Method, Routine and Message, no tests, no doc comments. With it all twenty `b8_*` probes
of those five classes answer the oracle's values on both engines (section 5).

## 1. The oracle's rule

All five are the same shape: allocate the primitive, then `RexxClass::completeNewObject`
(`classes/ClassClass.cpp:1882`): abstract check, `setBehaviour(getInstanceBehaviour())`, UNINIT
registration, `INIT` send with what `processNewArgs` left. So a subclass instance **is** the primitive
(a `RexxString`, `StemClass`, ...) carrying K's behaviour and an ordinary object-variable dictionary.
Because `isOfClass(t, r)` compares the behaviour (`behaviour/RexxBehaviour.hpp:186`-`:194`), every
primitive type test (`isString`, `isStem`, `isMethod`, ...) is **false** for a subclass instance, and
`requestString` takes the non-base path (`REQUEST` send, `classes/ObjectClass.cpp:1235`).

| Class | C++ | Arguments consumed | INIT gets |
|---|---|---|---|
| String | `RexxString::newRexx`, `classes/StringClass.cpp:2352` | 1, required, `stringArgument`; bytes copied | the rest |
| Stem | `StemClass::newRexx`, `classes/StemClass.cpp:92` | 1, optional name | the rest |
| Method | `MethodClass::newRexx`, `classes/MethodClass.cpp:499` | name, source, optional context (`BaseExecutable::processNewExecutableArgs`, `execution/BaseExecutable.cpp:225`) | the rest |
| Routine | `RoutineClass::newRexx`, `classes/RoutineClass.cpp:439` | same | the rest |
| Message | `MessageClass::newRexx`, `classes/MessageClass.cpp:828` | target, name, `A`/`I` arguments | nothing (`completeNewObject(newMessage)`) |
| VariableReference | `VariableReference::newRexx`, `classes/VariableReference.cpp:97`; `memory/Setup.cpp:1294` | none | n/a: raises 93.967 naming the receiver's id |

Observables measured on the oracle (probe texts in Appendix A):

- String subclass (`str_battery`, `str_all`, `str_ident`): string value, arithmetic, comparison,
  concatenation, BIFs, PARSE, tails, DATATYPE and Table keys all behave as the bytes; `~objectName` is
  `a K`; `~makestring`, `~string`, `~request('STRING')` answer class `String`; INIT sees `arg()` 0 after
  `.k~new('12')`. Methods that hit a C++ `return this` answer the **subclass instance**:
  `strip`, `strip('t')`, `changestr('z','y')`, `delstr(9)`, `append('')` answer class `K`; methods that
  build answer `String`. `grep -c 'return this;'` over `StringClass.cpp`, `StringClassSub.cpp`,
  `StringClassMisc.cpp`, `StringClassWord.cpp`: 17, 4, 6, 2.
- Stem subclass (`stem_battery`): `[]`, `put`, `at`, `items`, `allIndexes`, `supplier`, `toDirectory`
  work; `say x` is the stem name `S.`; `s. = x` and `use arg q.` take `x` as the default value (not an
  alias); `x~copy` is a `K` with its own tails; `x~request('STEM')` answers `.nil`.
- Method/Routine subclass (`exe_battery`, `ex/e*`): `source`, `isGuarded`, `isPrivate`, `package`,
  `scope` (`.nil`), `annotations` (a StringTable), `call`, `callWith`, `setPrivate`, `setUnguarded`,
  `copy` work; `addRoutine('RR', r)` accepts it; `call (r)` is 43.1 and `.zz~define('Q', m)` is 93.974
  (the `isOfClass` tests above).
- Message subclass (`msg_battery`): `send`, `start`, `result`, `wait`, `hasError`, `errorCondition`,
  `target`, `messageName`, `arguments` work; INIT runs with `arg()` 0.

## 2. Why ours refuses

| Class | Refusal site |
|---|---|
| String | `native_string_new`, `dispatch/construct.rs:379`: `class != string` then `unbuilt_new` |
| Stem | `native_stem_new`, `dispatch/construct.rs:401`: same test; `Body::Stem` has no class field (`dispatch/tests.rs:1290` pins this) |
| Method, Routine | `native_executable_new`, `dispatch/construct.rs` (the `args.len() > 2 || !(routine || method)` test); `compile_method_source`/`compile_routine_source` build `native_instance(method_class)` (`dispatch/class_protocol.rs:300`, `:335`) |
| Message | `native_message_new`, `dispatch/construct.rs`: `class != message` after the argument checks |
| VariableReference | no row: `Setup.cpp`'s `NEW` row has no native body, so the generic refusal at `dispatch.rs:2333` answers `method "NEW" of class "VariableReference"` |

Our object model: own methods (`write_object_method`, `dispatch.rs:2082`) and variable pools
(`pool_owner`, `run.rs:1536`) exist only on `Body::Instance`. `Body::Native` dispatch is by exact class
identity (`receiver_kind`, `dispatch.rs:1754`; only StringTable uses descent), with a fixed behaviour per
primitive. So a subclass instance needs to be a `Body::Instance` (K's behaviour, pools, own methods) with
the primitive payload reachable from it.

Precedents already landed, all `Body::Instance`:

| Precedent | Payload | Where |
|---|---|---|
| Array subclass | a `Body::Array` store in a pool variable; array natives redirect through `store_of` | `array_of_class`, `dispatch/array.rs:544`; `instance_over_store`, `dispatch/collection.rs:327` |
| MutableBuffer (base and subclass) | `NativeState::Buffer` in the instance's `native` slot; `to_text` reads it | `native_mutable_buffer_new`, `dispatch/buffer.rs:345`; `value.rs` instance arms |
| WeakReference | a cell in a pool variable under its own scope | `native_weak_reference_new`, `dispatch/construct.rs:520` |
| Directory subclass | plain instance; hash natives read the instance's store | `native_directory_new`, `dispatch/construct.rs` |

`Body::Instance` also gives the oracle's `isOfClass` answers for free: every evaluator site that tests
`Body::Text`/`Body::Stem`/`Body::Native` class identity takes the "not that primitive" path, which is what
the oracle does for a subclass instance (measured: `s. = x`, `call (r)`, `define` all agree in shape).

## 3. Design per class

| Class | Oracle rule | Our gap | Design | Size |
|---|---|---|---|---|
| VariableReference | 93.967 naming the receiver's id, base and subclass | generic Loud, rc 120, names `VariableReference` | `("VariableReference", "NEW", Arity::Counted, native_unsupported_new)` beside Buffer/Pointer (`dispatch.rs:1102`); unit test | S |
| Message | primitive + K behaviour; INIT with no args | refused | `new_instance(class)`; `native_entry`/`set_native_entry`/`remove_native_entry` (`environment.rs:1224`-`:1257`) keep the `MESSAGE_*` entries in a pool under the Message scope for an instance whose class descends from Message (the WeakReference shape); the existing 45 entry reads need no change. Prototyped: `msg_battery` byte-identical to the oracle | S/M |
| Method, Routine | executable + K behaviour; INIT with leftovers | refused; base also refuses the context argument and `~copy` (Phase 9 labels) | compile into `new_instance(class)` instead of `native_instance` (a class parameter on the two compile functions); the per-object side tables (`executable_sources`, `method_flag_writes`, ...) are keyed by `ObjRef` and generation-safe (`lib.rs:2750`), so `source`, flags, `call`, `callWith` work unchanged (prototyped: `e1`, `e4`, `e6` agree). Still to do: `scope` (`.nil`) and `annotations` for an instance (`environment/identities.rs:434`-`:459` read `Body::Native` only), `addRoutine` accepting a descendant (ours 88.914), the INIT send (rows move to the resumable table), `copy` refusing with the base's label, not an unrelated Phase 5 one | M |
| Stem | `StemClass` + K behaviour | refused | Array's shape: `Body::Stem` store in a pool variable; the Stem natives (`dispatch/hash/stem.rs`, 17 functions) redirect their receiver; prototyped, the battery agrees except: string value (`say x` answers `a K`, oracle `S.`; `s. == x` follows), `request('STEM')` (ours Loud, oracle `.nil`), `empty` must answer the instance, and `copy` must duplicate the store (the `COLLECTION_STORES` shape, `dispatch/object_protocol.rs:649`) | M |
| String | `RexxString` + K behaviour | refused | `NativeState::Text` on the instance (MutableBuffer's shape); `to_text`/`text_len`/`try_text` arms; `MAKESTRING` copies the bytes out; String operator natives (33) and `numeric_receiver` unwrap the receiver to its bytes (sending the operator to the instance first is right: K may override `+`, and the oracle sends). Prototyped: all but these agree in `str_all` (91 methods): `sign`; hash (`hashcode`, Table keys: `hash_value`, `dispatch/object_protocol.rs:203`, answers identity for an instance); the `return this` identity answers (an audit of every String row against the C++ sites counted above) | M, upper end |

b14 once `NEW` exists: nothing further for these five. With the prototype, every `b8_{string,stem,method,
routine,message}_*` probe agrees with the oracle and none of b14's refusal sites is touched. So scout A's
"S once NEW exists" is 0 for this route.

## 4. Things this sizing does not cover

- **b14's sites are reachable without `NEW`.** `setMethod`/`unsetMethod` sent to self from a class
  method (receiver: a class object) refuse today with a Phase 5 label; the oracle answers.
  `cls_setm`: ours rc 120 `a receiver with no scope of its own is not implemented (Phase 5)`, oracle
  rc 0 `42` `42`; `cls_setmobj`: oracle `42 5`; `cls_unset`: oracle `ok`. Both engines agree with each
  other. So rehoming b14 to Phase 9 "with the NEW rows" is not a true reason for this route; it needs its
  own disposition (per-object methods on a class object; not sized here). A TraceObject subclass, the
  other non-`Body::Instance`-looking route, already agrees (`tobj_*`).
- **`Class~enhanced` bypasses the receiver's `NEW`** (`native_enhanced`, `dispatch/class_protocol.rs:1029`:
  `new_instance` then `INIT` with all the arguments). Oracle `ClassClass.cpp:1470` sends `NEW` to the
  subclass. Measured: `.array~enhanced(d, 3)`, `.mutablebuffer~enhanced(d, "abc")` and all five classes
  here: ours 93.902 rc 163, oracle rc 0. Independent of this item; once subclass `NEW` exists it is a
  one-line change to send `NEW` instead.
- **`'12'~translate('','')`**: ours `"  "`, oracle `12` (`tr_base`), on a base string. Unrelated silent
  divergence met on the way.
- Base-class gaps the subclass inherits rather than adds: `.method~new(n, s, context)` and
  `Method~copy` refuse on the base with Phase 9 labels (`ex/base_ctx`, `ex/base_copy`).

## 5. Probes at HEAD (starting point, task 4)

Runner `/tmp/claude-1000/p61/sd/run.sh` (Appendix B): each file once on ours (default), once on ours with
`REXX_SWITCH_MODE=every`, once on the oracle, from an empty directory.

`run.sh probes/b8_*.rex` at HEAD: all 24 DIFF, no engine difference. String, Stem, Method, Routine,
Message: ours rc 120 `method "NEW" of class "K" is not implemented (Phase 9)`, oracle rc 0 `42`, `42`,
`3`, `ok` (setm, setmobj, expose, unset). VariableReference: ours rc 120 `method "NEW" of class
"VariableReference" ...`, oracle rc 163 93.967 `NEW method is not supported for the K class.`; also
`.VariableReference~new` itself (`vr_base`): ours the same rc 120, oracle 93.967 naming
`VariableReference`. This matches scout A's Appendix C rows.

With the prototype: the 20 `b8_*` rows of the five classes all `same`; the 4 VariableReference rows
unchanged (not prototyped).

## Appendix A: probes

In `/tmp/claude-1000/p61/sd/probes/` (kept). b8 set as scout A's Appendix B, restricted to the six
classes (constructors `.k~new('abc')`, `.k~new`, `.k~new('m', 'return 1')`, `.k~new('r', 'return 1')`,
`.k~new(.nil, 'X')`, `.k~new`).

```rexx
-- cls_setm.rex
say .k~go
say .k~m2
::class k
::method go class
  self~setMethod('m2', 'return 42')
  return self~m2
-- cls_setmobj.rex: body  self~setMethod('m2', 'expose v; v = 5; return 42 v', 'OBJECT'); return self~m2
-- cls_unset.rex:   body  self~unsetMethod('m2'); return 'ok'

-- str_ident.rex
x = .k~new('abc')
say x~strip~class~id x~upper~class~id x~lower~class~id x~left(3)~class~id x~substr(1)~class~id x~space~class~id x~changestr('z','y')~class~id x~translate~class~id x~delstr(9)~class~id x~overlay('')~class~id
y = .k~new('12'); say (y+0)~class~id y~abs~class~id y~format~class~id (y~'+'())~class~id
::class k subclass string
-- oracle: K String String String String String K String K String / String String String String

-- str_all.rex: for each of 91 calls c (every String row name with plausible arguments),
--   signal on syntax name eN; r = .k~new("12")~<c>; say "<c> ->" r~class~id r; ... eN: say "<c> -> ERR" condition("o")~code
--   then diffed ours against oracle with `diff`.

-- stem_battery.rex
x = .k~new('S.')
say x~class~id
x[1] = 'a'; say x[1] x~items x~hasindex(1)
say x
x~put('b', 2); say x~at(2)
say x~allindexes~makestring(,',')
s. = x; say s.1 s.~class~id (s. == x)
call r x
x2 = .k~new; say '['x2']' x2[7]
x3 = .k~new('T.'); say x3[9]
say x~supplier~class~id x~toDirectory~items
say x~copy~class~id x~copy[1]
y. = 'dflt'; x4 = .k~new; say x4~request('STEM')~class~id
exit
r: use arg q.; say 'r' q.1 q.~class~id; return
::class k subclass stem

-- msg_battery.rex
mm = .kmsg~new(.array~of(1,2), 'items'); say mm~class~id mm~send mm~target~class~id mm~messagename mm~arguments~items
mm2 = .kmsg~new('abc', 'length'); mm2~start; say mm2~result mm2~completed mm2~hasResult mm2~hasError
mm3 = .kmsg~new('abc', 'pos', 'I', 'b'); say mm3~send mm3~result
mm4 = .kmsg~new('abc', 'nosuch'); mm4~start; mm4~wait; say mm4~hasError mm4~errorCondition~code
say mm4
::class kmsg subclass message
::method init
  say 'init' arg()

-- clsenh_string.rex (and stem, method, routine, message)
d = .stringtable~new; d["GO"] = "expose x; x = 3; self~setMethod('m', 'return 42', 'OBJECT'); return x self~m"
x = .string~enhanced(d, 'abc')
say x~go x~class~id
-- oracle rc 0: 3 42 String; ours rc 163 93.902

-- tr_base.rex
say '['||'12'~translate('','')||']'
```

`str_battery.rex`, `exe_battery.rex`, `ex/e1`..`e9`, `lines/s01`..`s22` are the same calls split one per
file; their texts are in the probes directory.

## Appendix B: runner

```bash
#!/usr/bin/env bash
B=/tmp/claude-1000/p61/sd
OURS=$B/target/release/rexx-run
cd $B/empty || exit 1
for f in "$@"; do
  n=$(basename $f .rex)
  ( ulimit -v 4194304; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 $OURS $f >$B/o.out 2>$B/o.err ); rc=$?
  o="rc $rc: $(cat $B/o.out $B/o.err | tr '\n' '|' | head -c 300)"
  ( ulimit -v 4194304; REXX_SWITCH_MODE=every LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 $OURS $f >$B/o.out 2>$B/o.err ); rc=$?
  e="rc $rc: $(cat $B/o.out $B/o.err | tr '\n' '|' | head -c 300)"
  ( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx $f >$B/r.out 2>$B/r.err ); rc=$?
  r="rc $rc: $(cat $B/r.out $B/r.err | tr '\n' '|' | head -c 300)"
  if [ "$o" = "$r" ]; then s=same; else s=DIFF; fi
  if [ "$o" != "$e" ]; then s="$s ENGINES-DIFFER every=[$e]"; fi
  printf '%s\t%s\tours %s\toracle %s\n' "$n" "$s" "$o" "$r"
done
```

The only `ENGINES-DIFFER` flags seen were the prototype's stack-overflow aborts (thread ids differ in
the message), before the operator unwrap; none at HEAD.
