o = .r~new
n = .Method~new('y', 'return 2'); n~setUnguarded
o~put('Y', n)
say 'held' o~y (o~instanceMethod('Y') == n) o~instanceMethod('Y')~isGuarded
n = .nil
do 30; z = copies('x',100000); end
say 'dropped' o~y o~instanceMethod('Y')~isGuarded
::class r
::method put
  use arg nm, mm
  self~setMethod(nm, mm)
