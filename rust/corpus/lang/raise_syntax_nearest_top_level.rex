-- A RAISE SYNTAX with no RETURN is raised at the nearest activation that is
-- not an internal call, and offered outward from there.
o = .k~new
say o~ta
say o~tr
say o~tm
say o~tself
call r
signal on syntax name hmain
call selfish
say 'main no'
hmain: say 'main trapped' condition('o')~code sigl
exit
selfish:
  signal on syntax name hs
  raise syntax 42.3
hs: say 'selfish trapped'; return
::class k
::method boom
  raise syntax 42.3
::method ta
  signal on syntax name h
  self~boom
  return 'no'
h: return 'ta trapped' condition('o')~code
::method tr
  signal on syntax name h
  call r1
  return 'no'
h: return 'tr trapped' condition('o')~code
::method tm
  signal on syntax name h
  call inner
  return 'no'
h: return 'tm trapped' condition('o')~code sigl
inner:
  call inner2
inner2:
  raise syntax 42.3 exit
::method tself
  signal on any name h
  raise syntax 93.964 array ('m', 'd')
  return 'no'
h: return 'tself trapped' condition('c') condition('o')~code condition('o')~additional[2]
::routine r1
  raise syntax 40.4
::routine r
  signal on syntax name h
  call inner
  say 'r no'; return
h: say 'r trapped' condition('o')~code sigl; return
inner:
  raise syntax 40.4
