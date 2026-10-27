/* A failing run of a ::CONSTANT expression's Method is a level of its own,
   stopped on the directive: the expression's own failure is raised there,
   one below keeps its line, and RAISE PROPAGATE arrives there. Each class
   holds one ::CONSTANT. */
.a~define('ZZ', .a~e)
.b~define('ZZ', .b~e)
.p~define('ZZ', .p~e)
.local~div = 0
signal on syntax name s1
.a~new~zz
s1: call show
signal on syntax name s2
.b~new~zz
s2: call show
signal on syntax name s3
.p~new~zz
s3: call show
.a~new~zz
exit

show:
  c = condition('o')
  say c~code c~position
  do f over c~stackframes
    say f~type f~name f~line
  end
  do l over c~traceback
    say l
  end
  return

::class a
::constant e (.a~keep(.context~executable, 1 / .a~d))
::method keep class
  return arg(1)
::method d class
  if .local~div == .nil then return 1
  return .local~div

::class b
::constant e (.b~boom(.context~executable))
::method boom class
  d = .local~div
  if d == .nil then d = 1
  x = 1/d
  return arg(1)

::class p
::constant e (.p~prop(.context~executable))
::method prop class
  signal on syntax name h
  d = .local~div
  if d == .nil then d = 1
  x = 1/d
  return arg(1)
h:
  raise propagate
