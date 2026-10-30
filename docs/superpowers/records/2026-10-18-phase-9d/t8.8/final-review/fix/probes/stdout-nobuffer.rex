s = .stream~new('STDOUT')
s~open('NOBUFFER')
.stdout~charout('uv')
s~charout('abc')
call lineout 'STDERR', 'written' s~query('position') s~query('position write char') s~query('position sys')
call physical 'unbuffered-bytes'
s~close
s~open
call lineout 'STDERR', 'buffered-reopen' s~query('position write char') s~query('position sys')
s~charout('D')
call lineout 'STDERR', 'pending' s~query('position write char') s~query('position sys')
s~flush
call physical 'reopened-bytes'
exit
physical: procedure
  use arg label
  call lineout 'STDERR', label
  address system 'stat -Lc %s /proc/self/fd/1 >&2; od -An -tx1 /proc/self/fd/1 >&2'
  return
