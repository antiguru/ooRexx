/* RANDOM and SETLOCAL at internal-call depth 2000 act on main's activation:
   the seeded stream continues across the calls and main's ENDLOCAL restores
   what the deepest call saved. One restore in the process
   (oracle-crashes.txt entry 10b). */
say 'main' random(1,1000,5)
call value 'P61D', 'orig', 'ENVIRONMENT'
call down 2000
say 'main' random(1,1000) value('P61D',,'ENVIRONMENT') endlocal() value('P61D',,'ENVIRONMENT')
exit
down: procedure
  use arg n
  if n > 0 then do
    call down n - 1
    if n = 1000 then say 'depth 1000' random(1,1000)
    return
  end
  say 'depth 2000' random(1,1000) setlocal()
  call value 'P61D', 'deep', 'ENVIRONMENT'
  return
