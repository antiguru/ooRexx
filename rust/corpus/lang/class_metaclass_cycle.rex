/* A METACLASS target is a dependency as much as a SUBCLASS or an INHERIT
   target is, so a pair naming each other cannot be ordered: 98.911, blaming
   the first of them. The directive is otherwise refused here, which is why
   this shape and not an installing one is what witnesses the edge. */
say 'main ran'

::class a metaclass b

::class b metaclass a
