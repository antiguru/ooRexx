# `DO ... OVER` a user class, and the three `Stem` rows

Base `da02f3f1e`. Goal set 2026-09-07: finish the `DO ... OVER` user-class
implementation and the three `Stem` rows. Together they are eight of the ten
`send-differs` rows left in `corpus/collection-arity.tsv`.

## The rows

| cause | rows |
| --- | --- |
| `DO ... OVER` a user-class instance | `subset` on `Directory`, `IdentityTable`, `Properties`, `StringTable`, `Table` |
| the three `Stem` methods | `Stem request`, `Stem toDirectory`, `Stem unknown` |

The two left after this are `Properties save` (Phase 7 streams) and
`Properties setLogical` (`ARG` option `"A"`), neither in scope.

## Part 1 -- `DO ... OVER`

> **Not landed.** Everything measured below stands and the implementation
> agreed with the oracle, but it was reverted: a converted array's items have
> no root whose lifetime is the loop's. See this plan's close report.

`OverLoop::setup` (`instructions/DoBlockComponents.cpp:233`) tests
`isArray(result)` and calls `makeArray()` directly; otherwise it calls
`result->requestArray()` and raises `Error_Execution_noarray` -- 98.913, rc
158 -- if what comes back is not an array.

`RexxInternalObject::requestArray` (`classes/ObjectClass.cpp:1646`) is a
**two-path protocol keyed on `isBaseClass()`**:

* a base-class object: itself if it is an array, else `makeArray()` called
  directly as a C++ virtual, with **no message send**;
* anything else -- a user class, or a subclass of a built-in --
  `sendMessage(REQUEST, 'ARRAY')`.

That split is observable and was measured, four ways:

* a user class with `makeArray` iterates its result (`made a/b/c`);
* a **subclass of `Table`** overriding `makeArray` answers `OVERRIDDEN`, so
  the override is honoured -- the `REQUEST` path;
* a **subclass of `Array`** overriding `makeArray` answers `ARRAYSUB`, so
  `isArray()` is the primitive test and NOT "an array or a subclass";
* a user class overriding `request` itself answers `via ARRAY`, so `REQUEST`
  is genuinely the message and `'ARRAY'` genuinely the argument.

`makeArray` runs exactly **once** per loop, measured with a counter.

A `Table` iterates its indexes (`k1 k2`) and a `Directory` its own (`a`). A
class with no `makeArray` is 98, and a `makeArray` answering a non-array is
98.

**Work.** `over_target_gap` currently refuses everything outside an `Array`
and the one `Body::Native` `StringTable`. Replace it with the protocol above,
performed where `HeaderRole::Over` is evaluated -- which already returns
`Result`, so the 98.913 has somewhere to go, and which is where the oracle
does it. `hash.rs`'s existing `is_base_class` is `isBaseClass()` already and
moves somewhere both callers can reach it. `over_items` then reads the
converted array's non-empty slots and nothing else.

**What keeps refusing, deliberately:** `.environment` and `.local`. This
crate models them as a subset of the oracle's, so iterating one differs in
MEMBERSHIP and not merely in order -- measured previously at ten entries
against none. That refusal is not this task's to close and its test stays.

## Part 2 -- the three `Stem` rows

All three are small, and two of them stand on work already landed: the tail
tree's post-order walk, and `Directory`.

**`Stem~request(class)`** (`StemClass::request`). Upper-cases its argument.
`'ARRAY'` answers `makeArray` -- which for a `Stem` is its TAILS, already
implemented. Everything else forwards `REQUEST` to the stem's own value.
Measured: on a stem whose value is `hello`, `request('STRING')` is `hello`
and `request('DIRECTORY')` on an empty-valued stem is `.nil`;
`request('array')` lower-case works; `request()` is **93**, positional, since
it is `stringArgument(requestclass, ARG_ONE)`.

**`Stem~toDirectory`** (`StemClass::toDirectory`). A new `Directory`, one
entry per tail that HAS a value, keyed by the tail name, added in
`tails.first()`/`next()` order -- the post-order walk this crate already
ports. Measured: a stem with `X`=9 and `Y`=8 answers a `Directory` of 2 whose
`allIndexes` is `X,Y` and `allItems` `9,8`; after `remove('a')` a two-tail
stem answers 1. The resulting order is the directory store's, not the stem's,
and falls out of Phase 5h's geometry rather than needing anything here.

**`Stem~unknown(message, args)`** (`StemClass::unknownRexx`). Forwards the
message and its argument array to the stem's value. Measured: `q.` valued
`hello` answers `q.~length` as 5, and an empty-valued stem answers 0.

## Witnesses

One per part, filed in the four places: a `do_over_request_array.rex`
carrying the four observable arms of the protocol plus the two 98 shapes, and
a `stem_request_and_directory.rex` carrying the three methods over both a
valued and an unvalued stem, including a removed tail.

## Tables and gates

`collection-arity.tsv` re-derived (10 `send-differs` to 2);
`method-bodies.txt` refreshed. The seven gates over the commit, by the
commit-before-gating protocol.
