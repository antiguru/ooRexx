call t ".Queue~of('p','q','r')~section(1, .array~of(2))~makeArray~toString('L',' ')"
call t ".Queue~of('p','q','r')~section(1, .array~of('099'))~items"
call t ".Queue~of('p','q','r')~section(1, .array~of('1.0'))~items"
call t ".Queue~of('p','q','r')~section(1, .array~of(' 2 '))~items"
call t ".Queue~of('p','q','r')~section(1, .array~new(5)~~put(2,1))~items"
call t ".Queue~of('p','q','r')~section(1, .array~of(1,2))~items"
call t ".Queue~of('p','q','r')~section(1, .array~new)~items"
call t ".Array~of('p','q','r')~section(1, .array~of(2))~class~id"
call t ".Array~of('p','q','r')~section(1, .array~of(,2))~items"
call t ".QS~of('p','q','r')~section(1, .array~of(2))~class~id"
call t "'abcdef'~substr(.array~of(2), .array~of(3))"
call t "'abcdef'~left(.array~of(2))"
call t "'abcdef'~copies(.array~of(2))"
call t ".MutableBuffer~new('abcdef')~substr(.array~of(2), .array~of(3))"
call t ".MutableBuffer~new('abcdef', .array~of(50))~getBufferSize"
call t "'abcdef'~substr(.array~of(1,2))"
call t ".Queue~of('p','q','r')~next(.array~of('099'))"
call t ".Queue~of('p','q','r')~previous(.array~of('099'))"
call t ".Queue~of('p','q','r')~next(.array~of(,2))"
call t ".Queue~of('p','q','r')~previous(.array~of(,2))"
call t ".Queue~of('p','q','r')~next(.array~of(2)~~put(3,3))"
call t ".CircularQueue~of('p','q','r')~next(.array~new(3)~~put(2,3))"
call t ".Array~of('p','q','r')~insert('z', 999999999999)"
call t ".Array~of('p','q','r')~put('z', 999999999999)"
exit
t:
  parse arg e
  signal on syntax name f
  interpret 'r =' e
  say '  ' e '->' r
  return
f:
  c = condition('o'); say '  ' e 'ERR' c~code c~message
  return
::class QS subclass Queue
