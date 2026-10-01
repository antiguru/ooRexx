-- Message~halt reaching a started activity parked in a wait: the condition
-- is raised once the wait ends, at the waiting clause's boundary. A SIGNAL ON
-- trap takes it with that clause as SIGL; a CALL ON trap runs its handler
-- after the next clause. Main waits for the activity to park with a busy loop
-- the time slice ends.
do kind over .array~of('signalled', 'called')
  g = .gate~new
  m = .message~new(.t~new, kind, 'I', g)
  say kind 'before start' m~halt
  m~start
  do until g~parked == 1
  end
  call syssleep 0.2
  say kind 'halt' m~halt('stop' kind)
  say kind 'halt again' m~halt
  g~q~send
  m~wait
  say kind 'error' m~hasError 'result' m~hasResult
  say kind 'answered' m~result
  say kind 'after end' m~halt
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
::method signalled
  use arg g
  signal on halt
  g~q = .message~new(g, 'release')
  g~parked = 1
  g~q~wait
  say 'signalled ran on'
  return 'signalled'
halt:
  c = condition('o')
  return c~condition c~description c~instruction 'sigl' sigl
::method called
  use arg g
  call on halt name handler
  g~q = .message~new(g, 'release')
  g~parked = 1
  g~q~wait
  say 'called next'
  say 'called after'
  return 'called'
handler:
  say 'handler sigl' sigl condition('d') condition('i')
  return
