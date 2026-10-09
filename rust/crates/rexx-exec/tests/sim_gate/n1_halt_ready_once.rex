/* A halt reaching main after its timed wait's deadline readied it leaves it
   ready once (the seeded gate's witness, run under halt@K). Main polls with
   timed waits while a started activity runs a long loop. The gate judges
   this program by the scheduler's invariants only: its stdout is not read. */
call on halt name h
s = .eventsemaphore~new
o = .w~new~start('w')
halted = 0
do until halted
  r = s~wait(0.001)
end
call time 'R'
call SysSleep 1
say 'slept' (time('E') >= 0.9)
exit
h: halted = 1; return
::class w
::method w
  call on halt name wh
  do i = 1 to 100000
  end
  return
wh: return
