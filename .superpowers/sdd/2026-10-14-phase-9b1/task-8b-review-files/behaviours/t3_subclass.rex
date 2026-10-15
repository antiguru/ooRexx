u = .s~new
v = .k~new
.k~define('FOO', 'return "new"')
call filler
say 't3' u~foo v~foo .s~new~foo
exit
::class k
::method foo
  return 'old'
::class s subclass k
::routine filler
  do i = 1 to 300; z = copies('x', 100000); end
  return
