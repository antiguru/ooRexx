/* A redirect target whose method waits on something other than the command
   while the child keeps writing: a sleep runs to its end, a semaphore wait
   ends at its post, and a guard another activity holds keeps the target's
   method out until it is released. The output that arrived meanwhile reaches
   the target afterwards. */
o = .Sleeper~new
address system 'echo a; sleep 0.2; echo b; sleep 0.2; echo c' with output using (o)
say 'sleep' rc o~calls
.local~sem = .EventSemaphore~new
.local~posted = 0
o = .Waiter~new
address system 'echo a; sleep 0.2; echo b; sleep 0.2; echo c' with output using (o)
say 'semaphore' rc o~calls
.local~guardReady = .EventSemaphore~new
.local~callbackEntered = .EventSemaphore~new
.local~released = 0
o = .Guarded~new
h = o~start('hold')
.guardReady~wait
address system 'echo a; sleep 0.2; echo b; sleep 0.2; echo c' with output using (o)
say 'guard' rc o~calls 'released' .released
h~result
exit

::class Sleeper subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method lineout
  use arg line
  self~calls += 1
  slept = 0
  if line == 'a' then do
    -- The alarm starts in this callback, so command startup cannot satisfy it.
    .local~sleepMarked = 0
    alarm = .Alarm~new(0.5, .Message~new(.SleepMark~new, 'mark'))
    call SysSleep 1
    slept = .sleepMarked
    alarm~cancel
  end
  say 'O:'line 'slept' slept
  return 0

::class Waiter subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method lineout
  use arg line
  self~calls += 1
  waited = 0
  if line == 'a' then do
    -- Start the poster after entry, then observe its post before returning.
    p = .Poster~new~start('post')
    .sem~wait
    waited = .posted
    p~result
  end
  say 'O:'line 'waited' waited
  return 0

::class SleepMark
::method mark
  .local~sleepMarked = 1

::class Poster
::method post
  call SysSleep 1
  .local~posted = 1
  .sem~post

::class Guarded subclass OutputStream
::attribute calls
::method init
  self~calls = 0
::method hold
  -- Readiness is outside this guarded object and is posted while it is held.
  .guardReady~post
  -- Keep the guard until a callback actually starts trying to enter it.
  .callbackEntered~wait
  .local~released = 1
  say 'released'
::method lineout unguarded
  use arg line
  .callbackEntered~post
  return self~append(line)
::method append
  use arg line
  self~calls += 1
  say 'O:'line 'released' .released
  return 0
