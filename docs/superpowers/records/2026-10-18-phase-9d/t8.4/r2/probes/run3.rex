.c~define('ZZ', .c~e)
say 'defined'
.c~new~zz
say 'zz' var('result')
::class c
::constant a (.c~note('a'))
::constant e (.c~keep(.context~executable))
::constant z (.c~note('z'))
::method note class
  say 'note' arg(1)
  return arg(1)
::method keep class
  say 'keep'
  return arg(1)
