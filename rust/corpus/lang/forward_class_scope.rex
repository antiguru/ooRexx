/* FORWARD's CLASS value is validated by the send, not by the instruction, so
   where it lands decides both which error comes out and whether the
   forwarding method can trap it. The first three sends are the adjacent
   successes: a valid ancestor, a CLASS that is no class at all -- raised
   before the send and so trapped, reporting 88 -- and a bad scope under
   CONTINUE, which is an ordinary send and so also trapped, reporting 93 and
   not 97. The last send is the same bad scope without CONTINUE: the
   forwarding activation is already a phantom by the time the send validates,
   so its own trap does not fire, and the object the report names is the TO
   target rather than SELF. */
o = .K~new
t = .Tgt~new
say 'a' o~good
say 'b' o~notaclass
say 'c' o~contbad
say 'd' o~bad(t)
say 'never'

::CLASS Base
::METHOD m
  return 'base-m'

::CLASS Other
::METHOD m
  return 'other-m'

::CLASS Tgt
::METHOD m
  return 'tgt-m'

::CLASS K SUBCLASS Base
::METHOD good
  forward class (.Base) message('M')
::METHOD notaclass
  signal on syntax name t1
  forward class (5) message('M')
t1:
  return 'notaclass' rc
::METHOD contbad
  signal on syntax name t2
  forward continue class (.Other) message('M')
t2:
  return 'contbad' rc
::METHOD bad
  use arg t
  signal on syntax name t3
  forward to (t) class (.Other) message('M')
t3:
  return 'INNER-handler'
