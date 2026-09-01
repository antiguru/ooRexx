/* A non-continuing FORWARD leaves its own clause on the traceback when the
   forwarded-to method fails, even though the activation is already on its way
   out: the send happens with this activation still on the stack. Its pair is
   lang/delegate_no_frame.rex, which is the same failure reached through
   DELEGATE and reports one line fewer. */
o = .K~new
say 'a' o~fail

::CLASS Inner
::METHOD fail
  return 1/0

::CLASS K
::ATTRIBUTE d
::METHOD init
  expose d
  d = .Inner~new
::METHOD fail
  expose d
  forward to (d)
