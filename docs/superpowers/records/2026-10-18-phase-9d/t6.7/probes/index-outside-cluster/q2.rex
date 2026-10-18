call try ".Queue~of(1,2)~insert('a', .array~new)"
call try ".Queue~of(1,2)~insert('a', .array~of('x'))"
call try ".Queue~of(1,2)~insert('a', .array~of(1,2,3))"
call try ".Queue~of(1,2)~put('a', .array~new)"
call try ".Queue~of(1,2)~put('a', .array~of(1,2))"
call try ".Queue~of(1,2)~put('a', .array~of(9))"
call try ".Queue~of(1,2)~put('a', .array~of(20))"
call try ".Queue~of(1,2)~at(.array~of(1,2))"
call try ".Queue~of(1,2)~at(.array~new)"
call try ".Queue~of(1,2)~delete(.array~of(1,2))"
call try ".Queue~of(1,2)~section(.array~of(1,2))"
call try ".Queue~of(1,2)~section(.array~of(2))"
call try ".Queue~of(1,2)~section(1, .nil)"
call try ".Queue~new~insert('a', .array~of(1,2))"
call try ".Array~new~insert('a', .array~of(1,2))"
call try ".Array~new~section(.array~of(1,2))"
call try ".Array~new(0)~section(.array~of(1,2))"
exit
try:
  parse arg expression
  signal on syntax name failed
  interpret 'r =' expression
  if var('r') then say '  ' expression '->' r~class~id
  else say '  ' expression 'no result'
  return
failed:
  c = condition('o')
  say '  ' expression c~code c~message
  return
