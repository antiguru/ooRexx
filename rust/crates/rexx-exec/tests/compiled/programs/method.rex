lines = .stream~new('usese.rxo')~arrayin
signal on syntax name s2
m = .Method~new('d', lines)
say m~package~name m~source~items
.k~define('ZZ', m)
say 'method rxo' .k~new~zz
s2: say 'trap2' condition('o')~code condition('o')~message
do l over condition('o')~traceback; say '['l']'; end
exit
::class k
::routine callerfn
  return 'callerfn'
