/* A ::ATTRIBUTE GET written in Rexx is a method body like any other: the one
   ::ATTRIBUTE form that carries a body of its own, against the generated
   accessors, which read an instance variable and are Phase 5's still. */
say .K~a
say .K~a + 1

::class K

::attribute a class get
  return 11
