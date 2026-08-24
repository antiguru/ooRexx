/* A ::CONSTANT expression has no value while its own class is constructed.

   INIT runs inside the construction and the expressions are a later pass, so
   the accessor exists and the value does not: the oracle reports that as its
   own 97.4 rather than as the 97.1 a name miss gets. The enclosing echo is
   the ::CLASS clause, which is what a failure inside a class-side INIT is
   blamed against.

   The literal constant above it is the discriminating neighbour: read from
   the same INIT, at the same moment, it answers, because its value is fixed
   when the directive is parsed. */
say "main"

::CLASS A
::CONSTANT literal 5
::CONSTANT computed (2+3)
::METHOD init CLASS
  say "the literal is" self~literal
  say "the computed one is" self~computed
