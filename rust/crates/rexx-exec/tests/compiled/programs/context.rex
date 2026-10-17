lines = .stream~new('usese.rxo')~arrayin
src = .array~of('return callerfn()')
say 'routine src' .Routine~new('a', src)~call
say 'method src' .Method~new('b', src)~source~items
signal on syntax name s1
say 'routine rxo' .Routine~new('c', lines)~call
s1: say 'trap1' condition('o')~code condition('o')~message
signal on syntax name s2
m = .Method~new('d', lines)
.object~define('ZZ', m)
say 'method rxo' .object~new~zz
s2: say 'trap2' condition('o')~code condition('o')~message
exit
::routine callerfn
  return 'callerfn'
