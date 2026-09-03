/* ::OPTIONS DIGITS against the package's accumulated FUZZ, across two
   directives, and in source order with the rest of the install walk: the
   duplicate ::ROUTINE below is never reached. */
say 'unreached'

::options fuzz 5
::options digits 3

::routine dup
return

::routine dup
return
