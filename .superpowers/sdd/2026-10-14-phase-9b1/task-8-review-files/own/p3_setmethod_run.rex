o = .k~new
m = .k~method('BODY')~copy
m~setPrivate
o~put('X', m)
drop m
call filler
say o~callx
o~put("Y", .k~method("BODY")~copy, "OBJECT")
call filler
say o~y
say o~go(.k~method('BODY')~copy)
exit
filler: procedure
  do i = 1 to 300; z = copies('x', 100000); end
  return
::class k
::method put
  use arg n, m, s = "FLOAT"
  self~setMethod(n, m, s)
::method callx
  return self~x
::method go
  use arg m
  return self~run(m)
::method body
  call filler2
  return 'body' .context~executable~class~id .context~executable~isPrivate
::routine filler2
  do i = 1 to 300; z = copies('y', 100000); end
  return
