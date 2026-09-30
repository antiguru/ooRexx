/* Sends and calls in argument positions, and sends in expressions. */
o = .k~new('alpha-beta')
say f(o~name, g(2))
say length(o~name) o~twice(o~name)
call f o~name, g(3)
say result
x = f(g(1), o~twice(g(4)))
say x
say o~twice(o~twice('ab'))
say .k~new(g(5))~name
say (o~name, g(6))~items
a = .array~of(o~name, , 'b')
say a~items a[1] a~size
if o~ok(g(1)) then say 'ok' o~count
do i = 1 to o~count
  say i o~twice(i)
end
say f(.true, .nil, .k)
o~note(f(g(7), o~name))
say result
trace i
y = f(o~name, g(8))
say o~twice(g(9)) (1, , 2)~size
trace o
exit
f: return arg(1)'/'arg(2)
g: return 'g' || arg(1)
::class k
::method init
  expose nm
  use arg nm
::method name
  expose nm
  return nm
::method twice
  use arg s
  return s || s
::method ok
  return arg(1) == 'g1'
::method count
  return 2
::method note
  say 'note' arg(1)
  return 'noted'
