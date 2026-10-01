-- Main busy-waits for, then halts, a started activity spinning inside
-- INTERPRET: the spinning activity's pinned yields run main, whose state is
-- all in its record.
do kind over .array~of('sig', 'call', 'none')
  g = .gate~new
  m = .message~new(.t~new, kind, 'I', g)
  m~start
  do while g~i < 5; end
  say kind 'halt' m~halt
  m~wait
  if m~hasError then say kind 'error' m~errorCondition~code
  else say kind 'result' m~result
end
::class gate
::attribute i unguarded
::method init
  expose i
  i = 0
::class t
::method sig
  use arg g
  signal on halt
  interpret 'do i = 1; g~i = i; end'
halt:
  return 'sigl' sigl
::method call
  use arg g
  call on halt name h
  stop = 0
  interpret 'do i = 1 until stop; g~i = i; end'
  return 'done' stop
h:
  stop = 1
  return
::method none
  use arg g
  interpret 'do i = 1; g~i = i; end'
