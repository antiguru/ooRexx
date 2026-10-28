k = .kmake~new; s = .kstr~new; o = .object~new
say 'kmake'~equals(k) 'KMAKE'~caselessEquals(k) 'kmake'~equals(s) 'kstr'~equals(s)
say 'The NIL object'~equals(.nil) 'the nil object'~caselessEquals(.nil)
say 'an Object'~equals(o) 'AN OBJECT'~caselessEquals(o)
say '2'~equals(.array~of(2)) 'a'~equals(.array~of('a','b')) ('a'||'0a'x||'b')~equals(.array~of('a','b'))
say '12'~equals(12) '1E1'~equals(10) 'The Array class'~equals(.array)
signal on syntax
say 'x'~equals()
exit
syntax: say condition('o')~code condition('o')~message
::class kmake
::method makestring; return 'kmake'
::class kstr
::method string; return 'kstr'
