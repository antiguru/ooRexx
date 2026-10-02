x = 'main'
say Outer(.c~new)
say 'end' x
::requires 'outer9b' LIBRARY
::class c
::method run
  x = 'run'
  m = self~start('other')
  say 'started' m~result
  return 'ret'
::method other unguarded
  x = 'other'
  return OGet('X')
