/* A non-continuing FORWARD makes the forwarding activation a phantom for
   condition delivery as well as for its result, so a trap armed in that
   method does not see a condition the send raises and the caller's trap
   does. The first four sends are the adjacent successes that bound the rule
   to that one send: a trap in the caller alone, the same failure under
   CONTINUE, the same failure through DELEGATE, and a NOMETHOD trap in the
   caller of a forwarding method. The last send is the rule itself, and its
   report shows the phantom still on the traceback. */
o = .K~new
say 'a' o~outer
say 'b' o~cont
say 'c' o~viadelegate
say 'd' o~outernomethod
say 'e' o~m
say 'never'

::CLASS Inner
::METHOD fail
  return 1/0

::CLASS K
::ATTRIBUTE d
::METHOD init
  expose d
  d = .Inner~new
::METHOD outer
  signal on syntax name otrap
  return self~m
otrap:
  return 'OUTER-handler'
::METHOD cont
  expose d
  signal on syntax name ctrap
  forward continue to (d) message('FAIL')
ctrap:
  return 'trapped-cont'
::METHOD viadelegate
  signal on syntax name dtrap
  return self~fail
dtrap:
  return 'trapped-delegate'
::METHOD fail DELEGATE d
::METHOD outernomethod
  signal on nomethod name onm
  return self~p
onm:
  return 'outer-nomethod'
::METHOD p
  forward message('NOSUCHNAME')
::METHOD m
  expose d
  signal on syntax name itrap
  forward to (d) message('FAIL')
itrap:
  return 'INNER-handler'
