/* What another activity sees of a redirected command's targets while the
   command runs: an output stem and a collection are emptied before it
   starts and take each output line as it arrives, an error stem is emptied
   and filled once the output has ended, and a stem that is also the input
   is left alone until the end. Each child waits for a file this activity
   writes once it has read the targets. */
do n = 1 to 3
  call SysFileDelete 'go'n
end
call SysFileDelete 'started'
s.0 = 7; s.1 = 'old'
e. = 'E'; e.0 = 3
a = .array~of('x', 'y')
i.0 = 1; i.1 = 'in'
msg = .Job~new~start('run', s., e., a, i.)
do 400 until s.0 = 1
  call SysSleep 0.025
end
say 'output' s.0 s.1 'error' e.0 e.1
call release 1
do 400 until a~items = 1 & a[1] == 'three'
  call SysSleep 0.025
end
say 'array' a~items a[1]
call release 2
do 400 until SysFileExists('started')
  call SysSleep 0.025
end
say 'same' i.0 i.1
call release 3
say msg~result
say 'end output' s.0 s.1 s.2 'error' e.0 e.1 'array' a~items a[1] a[2] -
  'same' i.0 i.1 i.2
do n = 1 to 3
  call SysFileDelete 'go'n
end
call SysFileDelete 'started'
exit

release:
  call lineout 'go'arg(1), 'go'
  call lineout 'go'arg(1)
  return

::class Job
::method run
  use arg s., e., a, i.
  address system 'echo one; echo eone >&2;' self~waits(1) '; echo two' -
    with output stem s. error stem e.
  r1 = rc
  address system 'echo three;' self~waits(2) '; echo four' with output using (a)
  r2 = rc
  address system 'cat; touch started;' self~waits(3) '; echo more' -
    with input stem i. output stem i.
  return r1 r2 rc

::method waits
  use arg n
  return 'n=0; while [ ! -f go'n' ] && [ $n -lt 200 ]; do sleep 0.025; n=$((n+1)); done'
