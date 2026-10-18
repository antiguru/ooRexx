/* Array~insert and Array~section take their index as ArrayClass's
   validateIndex reads one: an Array argument stands for the subscripts it
   holds, and a single-dimensional receiver takes exactly one. Section's
   count is a length argument, so a count that is not a non-negative whole
   number is 93.923. A Queue does not spread an Array index: it reads one as
   its items' lines, so only a single item holding a position converts.
   Queue~insert past the last item is 93.966 however far past it is. */

a = .Array~of(1, 2, 3)
say 'insert after .array~of(2)' a~insert('x', .array~of(2)) a~toString('L', ' ')
a = .Array~of(1, 2, 3)
say 'insert after 2' a~insert('x', 2) a~toString('L', ' ')
b = .Array~new(0)
say 'insert past the end' b~insert('y', .array~of(4)) b~size b~items
say 'section .array~of(2)' .Array~of(1, 2, 3)~section(.array~of(2))~toString('L', ' ')
say 'section .array~of(2), 1' .Array~of(1, 2, 3)~section(.array~of(2), 1)~toString('L', ' ')
say 'section past the end' .Array~of(1)~section(.array~of(2))~items
q = .Queue~of('p', 'q')
say 'queue insert after .array~of(1)' q~insert('r', .array~of(1)) q~makeArray~toString('L', ' ')

call try "a~insert('x', .array~of(2, 3))"
call try "a~insert('x', .array~new)"
call try "a~insert('x', .array~of('two'))"
call try "a~section(.array~of(1, 2))"
call try "a~section(.array~new)"
call try "a~section(.array~of(0))"
call try "a~section(1, .nil)"
call try "a~section(1, .string)"
call try "a~section(1, -1)"
call try "a~section(1, 1.5)"
call try "a~section(1, 'x')"
call try ".Queue~new~insert('a', 99)"
call try ".Queue~new~insert('a', 17)"
call try ".Queue~of(1, 2)~insert('a', 3)"
call try ".Queue~of(1, 2)~insert('a', .array~of(5))"
call try ".Queue~new(50)~insert('a', 20)"
call try ".Queue~of(1, 2)~insert('a', .array~of(1, 2))"
call try ".Queue~of(1, 2)~insert('a', .array~new)"
call try ".Queue~of(1, 2)~insert('a', .array~of('x'))"
call try ".Queue~of(1, 2)~section(.array~of(2))"
call try ".Queue~of(1, 2)~section(.array~of(1, 2))"
call try ".Queue~of(1, 2)~section(1, .nil)"
exit

try:
  parse arg expression
  signal on syntax name failed
  interpret 'say' expression '~class~id'
  say '  ' expression 'raised nothing'
  return
failed:
  c = condition('o')
  say '  ' expression c~code c~message
  return
