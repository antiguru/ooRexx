/* An activity that ends holding a mutex at two levels hands it to the
   activity waiting for it. */
m = .MutexSemaphore~new
e = .EventSemaphore~new
msg = .w~new~start('hold', m, e)
e~wait
say 'main waits'
say 'main acquire' m~acquire
say 'worker ended' msg~completed
say m~release m~release
::class w
::method hold
  use arg m, e
  say 'worker' m~acquire m~acquire
  e~post
  call SysSleep 0.1
