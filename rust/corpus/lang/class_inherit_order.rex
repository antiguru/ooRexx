/* INHERIT: several mixins, the diamond, and a forward reference.

   K1 and K2 name the same mixins in opposite orders and answer differently,
   which is what says the rule is "leftmost first" rather than "M1 always
   wins".

   Combo is the diamond: MixinA and MixinB both inherit MixinBase and both
   override methodx, so the first-listed one wins it, while methody -- which
   only the shared ancestor defines -- comes through once and undamaged.

   Combo2 is the diamond arranged so that merging and walking the chain
   answer differently, and it is the only line here that separates them.
   PlainA overrides nothing and PlainB overrides methodx. Merging folds the
   shared ancestor's BASE_X in and then PlainB's own B_ONLY on top of it, so
   B_ONLY answers; walking the chain reaches PlainA first, finds no methodx
   of its own, descends into MixinBase and answers BASE_X without ever
   asking PlainB.

   Fwd names a mixin declared below it, so the install order is not source
   order. */
say .K1~m
say .K2~m
say .Combo~methodx
say .Combo~methody
say .Combo2~methodx
say .Fwd~m

::CLASS M1 MIXINCLASS Object
::METHOD m CLASS
  return 'M1'

::CLASS M2 MIXINCLASS Object
::METHOD m CLASS
  return 'M2'

::CLASS K1 INHERIT M1 M2

::CLASS K2 INHERIT M2 M1

::CLASS MixinBase MIXINCLASS Object
::METHOD methodx CLASS
  return 'BASE_X'
::METHOD methody CLASS
  return 'BASE_Y'

::CLASS MixinA MIXINCLASS Object INHERIT MixinBase
::METHOD methodx CLASS
  return 'A_X'

::CLASS MixinB MIXINCLASS Object INHERIT MixinBase
::METHOD methodx CLASS
  return 'B_X'

::CLASS Combo SUBCLASS Object INHERIT MixinA MixinB

::CLASS PlainA MIXINCLASS Object INHERIT MixinBase

::CLASS PlainB MIXINCLASS Object INHERIT MixinBase
::METHOD methodx CLASS
  return 'B_ONLY'

::CLASS Combo2 SUBCLASS Object INHERIT PlainA PlainB

::CLASS Fwd INHERIT Later

::CLASS Later MIXINCLASS Object
::METHOD m CLASS
  return 'forward'
