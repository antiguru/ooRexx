/* A Queue does not spread an Array index into subscripts, as an Array
   does: QueueClass's validateIndex reads the Array as its items' lines, so
   a single item holding a position converts and any other Array is 93.907
   naming it. put past the queue's extent is 93.918 naming the index as
   given, and past its last item 93.966. CircularQueue is a Queue. */

do clz over .Queue, .CircularQueue
  say clz~id
  q = clz~of('p', 'q', 'r')
  say '  at .array~of(2)' q~at(.array~of(2)) q[.array~of(3)]
  say '  hasIndex .array~of(2)' q~hasIndex(.array~of(2))
  q~put('z', .array~of(2))
  say '  put .array~of(2)' q~makeArray~toString('L', ' ')
  say '  delete .array~of(1)' q~delete(.array~of(1)) q~makeArray~toString('L', ' ')
  say '  remove .array~of(1)' q~remove(.array~of(1)) q~makeArray~toString('L', ' ')
  q = clz~of('p', 'q', 'r')
  say '  next .array~of(1)' q~next(.array~of(1)) 'previous .array~of(3)' q~previous(.array~of(3))
  call try clz, "q~put('a', .array~of(9))"
  call try clz, "q~put('a', .array~of(20))"
  call try clz, "q~put('a', .array~of(1, 2))"
  call try clz, "q~put('a', .array~new)"
  call try clz, "q~put('a', 99)"
  call try clz, "q~put('a', '099')"
  call try clz, "q~at(.array~of(1, 2))"
  call try clz, "q~at(.array~new)"
  call try clz, "q~at(.array~of('x'))"
  call try clz, "q~at(.array~of(0))"
  call try clz, "q~delete(.array~of(1, 2))"
  call try clz, "q~remove(.array~new)"
  call try clz, "q~hasIndex(.array~of(1, 2))"
end
exit

try:
  use arg clz, expression
  q = clz~of('p', 'q', 'r')
  signal on syntax name failed
  interpret 'r =' expression
  if var('r') then say '  ' expression '->' r
  else say '  ' expression 'no result'
  return
failed:
  c = condition('o')
  say '  ' expression c~code c~message
  return
