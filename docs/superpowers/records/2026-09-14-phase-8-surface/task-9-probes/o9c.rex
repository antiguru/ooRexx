x = 'main'
say Outer(.c~new)
say 'after' symbol('NEWV') symbol('T.')
exit
::requires 'outer9b' LIBRARY
::class c
::method run
  say 'get' OGet('NEWV') OGet('T.1')
  say OSet('NEWV', 'made')
  return 'ret'
