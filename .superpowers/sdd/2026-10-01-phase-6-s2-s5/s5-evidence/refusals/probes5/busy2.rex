g = .gate~new
a = .t~new~start('a', g)
b = .t~new~start('b', g)
say a~result
say b~result
::class gate
::attribute x unguarded
::attribute y unguarded
::method init
  expose x y
  x = 0
  y = 0
::class t
::method a unguarded
  use arg g
  interpret 'do while \g~x; end'
  g~y = 1
  return 'A ended'
::method b unguarded
  use arg g
  interpret 'g~x = 1; do while \g~y; end'
  return 'B ended'
