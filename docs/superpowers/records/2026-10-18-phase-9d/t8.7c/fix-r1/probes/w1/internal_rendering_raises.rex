/* A condition DEFAULTNAME raises inside an internal rendering is raised by the rendering */
call joined
call traced
call inserted
call answered
exit

joined:
  signal on syntax name joinedCaught
  say .array~of(1, .bad~new('join'))~makestring
  say 'joined not reached'
  return
joinedCaught:
  say 'joined' condition('o')~code condition('o')~message
  return

traced:
  signal on syntax name tracedCaught
  o = .bad~new('trace')
  trace i
  x = o
  trace off
  say 'traced not reached'
  return
tracedCaught:
  trace off
  say 'traced' condition('o')~code condition('o')~message
  return

inserted:
  signal on syntax name insertedCaught
  o = .bad~new('insert')
  o~nosuch
  say 'inserted not reached'
  return
insertedCaught:
  say 'inserted' condition('o')~code condition('o')~message
  return

answered:
  signal on syntax name answeredCaught
  o = .dn~new
  trace i
  x = o
  trace off
  o~nosuch
  return
answeredCaught:
  say 'answered' condition('o')~code condition('o')~message
  return

::class bad
::method init
  expose where
  use arg where
::method defaultname
  expose where
  say 'defaultname ran for' where
  if where == 'join' then return 1 + 'a'
  raise syntax 98.900 array(where)

::class dn
::method defaultname
  return .array~of('p', 'q')
