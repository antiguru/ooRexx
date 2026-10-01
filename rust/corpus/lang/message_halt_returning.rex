-- Message~halt reaching a started activity waiting in a RETURN, the last
-- clause of its activation: the condition is raised in that activation, so
-- its own SIGNAL ON HALT takes it, and untrapped the traceback starts there.
do kind over .array~of('trapped', 'untrapped')
  g = .gate~new
  m = .message~new(.t~new, 'outer', 'I', g, kind)
  m~start
  do until g~parked == 1
  end
  call syssleep 0.2
  say kind 'halt' m~halt
  g~q~send
  m~wait
  say kind 'error' m~hasError
  if m~hasError then say kind m~errorCondition~code m~errorCondition~position
  else say kind 'result' m~result
end
::class gate
::attribute parked unguarded
::attribute q unguarded
::method init
  expose parked
  parked = 0
::method release unguarded
  return 'released'
::class t
::method outer
  use arg g, kind
  if kind == 'trapped' then r = self~inner(g)
  else call plain g
  say 'outer after' r
  return 'outer done'
plain:
  use arg g
  g~q = .message~new(g, 'release')
  g~parked = 1
  return g~q~result
::method inner
  use arg g
  signal on halt
  g~q = .message~new(g, 'release')
  g~parked = 1
  return g~q~result
halt:
  return 'inner trapped sigl' sigl
