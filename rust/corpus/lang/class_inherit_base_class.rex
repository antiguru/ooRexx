/* A mixin may only be inherited by a class that already has the mixin's own
   base class in scope. M's base is P, and K descends from Object alone, so
   this is 98.943 naming all three classes. */
say 'main ran'

::CLASS P

::CLASS M MIXINCLASS P

::CLASS K INHERIT M
