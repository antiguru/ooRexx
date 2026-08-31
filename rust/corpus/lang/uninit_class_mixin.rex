/* A mixin's class-side UNINIT reaches the class that inherits it, and fires
   for both. K declares none of its own: it fires only if INHERIT propagates
   the finalizer, which is what deleting the INHERIT here takes away. */
say 'main'

::class m mixinclass Object
::method uninit class
  say 'uninit on' self~id

::class k inherit m
