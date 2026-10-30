/* Each USE trace line sends separately, before and after the assignment. */
call bind .once~new(1)
call bind .once~new(2)
exit
bind:
  signal on syntax name caught
  value = 'old'
  trace i
  use arg value
  trace off
  say 'not reached'
  return
caught:
  trace off
  say 'assigned' (value~class == .once) 'code' condition('o')~code
  return
::class once
::method init
  expose n limit
  use arg limit
  n = 0
::method defaultname
  expose n limit
  n = n + 1
  say 'name' n
  if n = limit then raise syntax 98.900 array('boom')
  return 'ONCE'
