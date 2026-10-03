/* A post ends a timed wait with 1 and stays posted. */
e = .EventSemaphore~new
t = .t~new
m = t~start('w', e, 2)
call SysSleep 0.1
e~post
say 'post' m~result e~isPosted
::class t
::method w unguarded
  use arg e, t
  return e~wait(t)
