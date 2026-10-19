c = .recorder~new
m = .myarr~of(2, 1)
m~sortWith(c)
call show 'sub', c~frames[2], m
m~stableSortWith(c)
call show 'substable', c~frames[2], m
o = .other~of(2, 1)
o~sortWith(c)
call show 'other', c~frames[2], o
s = .sub~new
s~sm('OWN', 'return 2')
say 'instance own' s~instanceMethod('OWN')~source~makeString cnt(s~instanceMethods, 'OWN') cnt(s~instanceMethods(.nil), 'OWN')
.sub~sm('OWN', 'return 2')
say 'class own' .sub~instanceMethod('OWN')~source~makeString cnt(.sub~instanceMethods, 'OWN') cnt(.sub~instanceMethods(.nil), 'OWN')
exit
show: procedure
  use arg tag, f, target
  say tag f~type f~name (f~target == target) f~traceLine
  say tag (f~executable == .Array~method(f~name)) f~executable~scope~id
  return
cnt: procedure
  use arg s, name
  n = 0
  do while s~available; if s~index == name then n = n + 1; s~next; end
  return n
::class recorder
::attribute frames
::method compare
  self~frames = .context~stackFrames
  return 0
::class myarr subclass array
::method sortWith
  forward class (super)
::method stableSortWith
  forward class (super)
::class other subclass array
::method activate class
  self~define('SORTWITH', .array~method('SORTWITH'))
::class sub
::method sm class
  forward message 'SETMETHOD'
::method sm
  forward message 'SETMETHOD'
