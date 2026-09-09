/* A message send in a builtin's argument position.

   The shared argument stack used to be cleared on the way into a send and
   overwritten on the way out, so the send's own arguments reached the
   enclosing builtin and the builtin's earlier ones were lost. Each line below
   separates one part of that: a send with arguments, a send without, a send
   that is not the first argument, one whose result the builtin must convert,
   and a user routine in the same position, which never went through that
   path. */
s = 'abcdef'
say '1' length(s~copies(1))
say '2' length(s~upper)
say '3' substr(s~copies(1), 2, 3)
say '4' substr(s, s~length - 4, 3)
say '5' word('a b c'~copies(1), 2)
say '6' date('F', .DateTime~fromIsoDate('2026-09-09T12:34:56.000000')~standardDate, 'S')
say '7' r(s~copies(1), 2)
say '8' length(.string~new('abc'))
exit
r: procedure
  use arg a, b
  return a || '/' || b
