/* A timed wait a post wakes tests the post again: after a post and a reset
   in one clause it waits out its timeout, where an untimed wait answers 1;
   a post with no reset ends a timed wait with 1. */
e = .EventSemaphore~new
t = .t~new
m1 = t~start('w', e, '')
m2 = t~start('w', e, 1)
m3 = t~start('w', e, '')
call SysSleep 0.1
e~~post~reset
say 'pulse' m1~result m2~result m3~result e~isPosted
m4 = t~start('w', e, 2)
call SysSleep 0.1
e~post
say 'post' m4~result e~isPosted
::class t
::method w unguarded
  use arg e, t
  if t == '' then return e~wait
  return e~wait(t)
