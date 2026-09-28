x = 'main'
say Outer9(.c~new)
say 'y' y
exit
::requires 'outer9' LIBRARY
::class c
::method run
  x = 'run'
  y = 'run-y'
  return UseOuterVar() UseOuterSet() 'y-in-run' y
