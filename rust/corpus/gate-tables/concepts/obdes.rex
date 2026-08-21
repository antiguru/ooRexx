/* provide.xml `obdes`: object destruction is implicit, and an object needing
   uninitialization defines UNINIT, which Rexx runs before reclaiming the
   object's storage. A class object is destroyed like any other object, so a
   class-side UNINIT fires with no instance anywhere. */
say 'main'

::class k
::method uninit class
  say 'uninit ran'
