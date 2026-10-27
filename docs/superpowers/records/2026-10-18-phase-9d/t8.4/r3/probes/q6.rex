.c~define('ZZ', .c~e)
o = .c~new
.local~div = 0
signal on syntax
o~zz
exit
syntax:
  c = condition('o')
  say c~code c~position
  do f over c~stackframes; say f~type f~name f~line; end
  do l over c~traceback; say l; end
::class c
::constant e (.c~boom(.context~executable))
::method boom class
  d = .local~div
  if d == .nil then d = 1
  x = 1/d
  return arg(1)
