o = .t~new
o~go('m1')
o~go('m2')
::class t
::method go
  use arg name
  signal on any name h
  .message~new(self, name)~send
  say name 'not raised'
  return
h: say name 'caught' condition('c')
::method m1
  signal on any
  "exit 1"
  return
  any: raise propagate
::method m2
  signal on error
  "exit 1"
  return
  error: raise propagate
