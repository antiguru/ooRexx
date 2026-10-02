/* A REPLY inside a DO, a SELECT branch and either arm of an IF: the rest of
   the construct, and the body after it, run on the new activity. */
o = .k~new
d = .directory~new
say 'do:' o~indo(d)
call wait d
say 'while:' o~inwhile(d)
call wait d
say 'when:' o~insel(d, 2)
call wait d
say 'otherwise:' o~insel(d, 3)
call wait d
say 'then:' o~inif(d, 1)
call wait d
say 'else:' o~inif(d, 0)
call wait d
say 'end main'
exit
wait: procedure
  use arg d
  do until d~hasIndex('done')
    call SysSleep 0.01
  end
  d~remove('done')
  return
::class k
::method indo
  use arg d
  do i = 1 to 3
    if i = 2 then reply i
    say 'loop' i
  end
  say 'after loop' i
  d['done'] = 1
::method inwhile
  use arg d
  n = 0
  do while n < 3
    n = n + 1
    if n = 1 then do
      reply 'n='n
      say 'in the do after the reply'
    end
    say 'pass' n
  end
  d['done'] = 1
::method insel
  use arg d, n
  select
    when n = 1 then say 'one'
    when n = 2 then do
      reply 'two'
      say 'in when after reply'
    end
    otherwise
      reply 'other'
      say 'in otherwise after reply'
  end
  say 'after select'
  d['done'] = 1
::method inif
  use arg d, c
  if c then reply 'yes'
  else reply 'no'
  say 'after if' c
  d['done'] = 1
