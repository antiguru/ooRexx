-- A started activity halting its own message inside INTERPRET: the
-- condition is raised at the end of the interpreted clause.
do kind over .array~of('sig', 'call', 'none')
  g = .gate~new
  g~m = .message~new(.t~new, kind, 'I', g)
  g~m~start
  g~m~wait
  if g~m~hasError then say kind 'error' g~m~errorCondition~code g~m~errorCondition~position
  else say kind 'result' g~m~result
end
::class gate
::attribute m unguarded
::class t
::method sig
  use arg g
  signal on halt
  interpret 'do i = 1; if i = 3 then x = g~m~halt; y = i; end'
halt:
  return 'sigl' sigl 'i' i
::method call
  use arg g
  call on halt name h
  stop = 0
  interpret 'do i = 1 until stop; if i = 3 then x = g~m~halt; y = i; end'
  return 'done' stop 'i' i
h:
  stop = 1
  return
::method none
  use arg g
  interpret 'do i = 1; if i = 3 then x = g~m~halt; y = i; end'
