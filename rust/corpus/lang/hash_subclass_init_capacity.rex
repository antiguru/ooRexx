/* A hash collection is sized by INIT, not NEW: a subclass whose INIT does
   not forward keeps the default capacity whatever NEW was given, and NEW
   does not check the capacity argument, so a subclass's INIT may take
   anything; a forwarding INIT sizes as the base does. */
t = .kt~new(100)
do i = 1 to 12; t[i] = i; end
s = ''; do k over t; s = s k; end; say 'kt' s
u = .table~new(100)
do i = 1 to 12; u[i] = i; end
s = ''; do k over u; s = s k; end; say 'table100' s
v = .table~new
do i = 1 to 12; v[i] = i; end
s = ''; do k over v; s = s k; end; say 'table' s
w = .kf~new(100)
do i = 1 to 12; w[i] = i; end
s = ''; do k over w; s = s k; end; say 'kf' s
st = .ks~new('abcdefghij', 'q'); say st~class~id st~items
x = .kt~new('x'); say 'kt x' x~items
d = .kd~new('q'); say d~class~id d~items
a = .set~new(5); a~put(3); say a~items .bag~new(0)~items .relation~new('3')~items .identitytable~new(7)~items
signal on syntax name e1
x = .stringtable~new('x')
e1: say 'e1' condition('O')~code condition('O')~message
signal on syntax name e2
x = .kf~new('x')
e2: say 'e2' condition('O')~code condition('O')~message
signal on syntax name e3
x = .table~new(-1)
e3: say 'e3' condition('O')~code condition('O')~message
signal on syntax name e4
x = .directory~new(1, 2)
e4: say 'e4' condition('O')~code condition('O')~message
exit
::class kt subclass table
::method init
  say 'kt init' arg()
::class kf subclass table
::method init
  forward class (super)
::class ks subclass stringtable
::method init
  say 'ks init' arg()
::class kd subclass directory
::method init
  say 'kd init' arg()
