.c~define('ZZ', .c~e)
.local~div = 0
signal on syntax
.c~new~zz
exit
syntax:
  c = condition('o')
  say c~code c~position
  do l over c~traceback; say l; end
::class c
::constant e (.c~prop(.context~executable))
::method prop class
  signal on syntax name h
  d = .local~div
  if d == .nil then d = 1
  x = 1/d
  return arg(1)
h:
  raise propagate
