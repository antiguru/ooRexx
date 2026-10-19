o = .k~new
b. = "bd"
call t o, 'refExposed', >a
call t o, 'refExposedSet', >a
call t o, 'refAutoExposed', >a
call t o, 'refLocal', >a
call t o, 'refExposedStem', >a.
call t o, 'refAutoExposedStem', >a.
call t o, 'refLocalStem', >a.
call t o, 'doubleArgStem', b., >a.
call t o, 'doubleArg', 7, >a
call t o, 'refTwice', >a
call t o, 'refTwiceUnset', >zz
call t o, 'refAutoTwice', >a
call t o, 'kindFirst', >a.
say 'a =' a 'a.1 =' a.1 symbol('a') symbol('a.')
call r1 >a
call r2 >a
return
t: procedure
  use arg o, msg, x, y
  signal on syntax name caught
  if arg() = 3 then r = o~send(msg, x)
  else r = o~send(msg, x, y)
  say msg': ok' r
  return
caught:
  say msg':' condition('o')~code '['condition('o')~message']'
  return
r1: use arg >a
  say 'r1 ok' a
  return
r2: procedure
  a = 5
  signal on syntax name c2
  use arg >a
  say 'r2 not raised'
  return
c2: say 'r2:' condition('o')~code '['condition('o')~message']'
  return
::class k
::method refExposed
  expose abc
  use arg >abc
  return abc
::method refExposedSet
  expose abc
  abc = 'objval'
  use arg >abc
  return abc
::method refAutoExposed
  use local
  use arg >abc
  return abc
::method refLocal
  use local abc
  use arg >abc
  return abc
::method refExposedStem
  expose abc.
  use arg >abc.
  return abc.1
::method refAutoExposedStem
  use local
  use arg >abc.
  return abc.1
::method refLocalStem
  use local abc.
  use arg >abc.
  return abc.1
::method doubleArgStem
  use arg abc., >abc.
  return abc.1
::method doubleArg
  use arg abc, >abc
  return abc
::method refTwice
  use arg >abc
  use arg >abc
  return abc
::method refTwiceUnset
  use arg >abc
  use arg >abc
  return 'twice'
::method refAutoTwice
  use local abc
  use arg >abc
  use arg >abc
  return abc
::method kindFirst
  expose abc
  use arg >abc
