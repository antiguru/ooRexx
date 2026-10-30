signal on syntax name failure
k = .key~new
t = .table~new
t~put('value',k)
k~reset
say 'table' t~remove(k) t~items
s = .set~new
k~reset
s~put(k)
k~reset
say 'set' (s~removeItem(k) == k) s~items
r = .relation~new
k~reset
r~put('first',k)
k~reset
r~put('second',k)
k~reset
say 'all' r~allAt(k)~items
k~reset
say 'count' r~items(k)
k~reset
say 'has' r~hasItem('first',k)
k~reset
say 'remove' r~removeItem('first',k) r~items
exit
failure: say 'SYNTAX' condition('o')~code; exit
::class key
::method init
  expose n
  n = 0
::method reset
  expose n
  n = 0
::method hashCode
  expose n
  n = n + 1
  say 'hash' n
  if n > 1 then raise syntax 98.900 array('duplicate hash')
  return 'key'
::method '=='
  say 'equal'
  return 1
