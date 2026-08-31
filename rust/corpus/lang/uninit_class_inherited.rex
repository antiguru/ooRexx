/* A subclass declaring no UNINIT of its own inherits the class-side one and
   fires it, before the class that declares it. */
say 'main'

::class p
::method uninit class
  say 'uninit on' self~id

::class k subclass p
