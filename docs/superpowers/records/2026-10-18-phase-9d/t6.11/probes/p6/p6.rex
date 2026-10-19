o = .t~new
o~put('PUT2', 'return 123', 'OBJECT')
say o~put2
m = o~instanceMethod('put2')
say m~class~id (m~scope == .t)
o~put('PUT', 'return 5', 'OBJECT')
m = o~instanceMethod('put')
say m~class~id (m~scope == .t) (m == .t~method('PUT')) m~source
s = o~instanceMethods(.nil)
do while s~available; say s~index s~item~class~id (s~item == m) s~item~source; s~next; end
o~unhide
say o~instanceMethod('PUT') == .nil
say .t~new~instanceMethod('PUT') == .t~method('PUT')
::class t
::method put
  use arg n, m, sc
  self~setMethod(n, m, sc)
::method unhide
  self~setMethod('PUT')
