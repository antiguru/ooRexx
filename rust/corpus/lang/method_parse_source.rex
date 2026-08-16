/* PARSE SOURCE's second word inside a ::METHOD body: METHOD, whatever the
   sending clause's own context was. Only that word is printed -- the third is
   the program's own absolute path, and corpus/lang/parse_sources.rex projects
   it away the same way and for the same reason.

   The pair with that program is the point. It pins COMMAND for a program and
   an internal label, and SUBROUTINE against FUNCTION for one ::routine body
   reached both ways; this pins the word a method activation answers. An
   engine that renders a single fixed word agrees with one of the two programs
   and prints that program's word here, at rc 0, with nothing on stderr to say
   so.

   M2 sends from inside a ::ROUTINE reached as a function, which is the
   context that would be inherited if a method activation inherited one: the
   routine reads FUNCTION at M3 and the method it sends to still reads METHOD.

   No abuttal anywhere: a symbol abutting a preceding string literal can be
   read as its hex or binary suffix, so every join below is an explicit ||. */

say 'M1 ['||.K~context||']'
say 'M2 ['||sender()||']'

::routine sender
  parse source sys ctx .
  say 'M3 ['||ctx||']'
  return .K~context

::class K

::method context class
  parse source sys ctx .
  return ctx
