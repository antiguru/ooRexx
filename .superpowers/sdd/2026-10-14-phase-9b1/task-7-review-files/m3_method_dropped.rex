.local['M'] = .Method~new('m', 'say "in m"; .local~remove("M"); do 30; z = copies("x",100000); end;' -
  'e = .context~executable; say "exe" e~class~id e~isGuarded e~isPrivate e~source[1]~left(9); return 3')
.local['M']~setUnguarded
o = .r~new
say 'run' o~go(.local['M'])
m = .Method~new('x', 'return 1'); m~setUnguarded; m~setPrivate
.k~define('X', m); m = .nil
do 30; z = copies('x',100000); end
q = .k~method('X')
say 'defined' q~isGuarded q~isPrivate
signal on syntax name s1
say 'private send answered' .k~new~x
s1: say 'private send raised' rc
signal off syntax
n = .Method~new('y', 'return 2'); n~setUnguarded
o~put('Y', n); n = .nil
do 30; z = copies('x',100000); end
say 'setMethod' o~y o~instanceMethod('Y')~isGuarded
::class k
::class r
::method go
  use arg m
  return self~run(m)
::method put
  use arg nm, mm
  self~setMethod(nm, mm)
