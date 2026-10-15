m = .Method~new('m', 'return 1')
say 1 m~copy~isGuarded
x = .ms~new('x', 'return 2')
say 2 x~isPrivate x~copy~isPrivate x~source[1]
r = .rs~new('r', 'return 7')
say 3 r~call r~copy~call
o = .k~new
o~put('Z', m)
say 4 o~z
.k~define('W', x)
say 5 .k~new~w
.k~defineClassMethod('V', x~copy)
say 6 .k~v
y = .ms~new('y', 'return 9')
say 7 y~class~id
say 8 .k~method('FOO')~copy~isGuarded
::class ms subclass Method
::method init
  nop
::class rs subclass Routine
::class k
::method foo
  return 3
::method put
  use arg n, m
  self~setMethod(n, m)
