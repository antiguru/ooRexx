s = .stream~new('STDOUT')
s~charout('abc')
call lineout 'STDERR', 'held' s~query('position') s~query('position read char') s~query('position write char') s~query('position sys')
call physical 'before-flush'
s~flush
call lineout 'STDERR', 'flushed' s~query('position write char') s~query('position sys')
call physical 'after-flush'
s~charout('D')
call lineout 'STDERR', 'pending' s~query('position write char') s~query('position sys')
call physical 'before-size'
call lineout 'STDERR', 'size' s~query('size')
call physical 'after-size'
s~close
call lineout 'STDERR', 'closed' '['s~query('position')']' '['s~query('size')']'
s~open
call lineout 'STDERR', 'reopen' s~query('position') s~query('position write char') s~query('position sys')
s~charout('E')
s~flush
call lineout 'STDERR', 'rewritten' s~query('position write char') s~query('position sys')
call physical 'reopened-bytes'
exit
physical: procedure
  use arg label
  call lineout 'STDERR', label .stream~new('captured-output.bin')~query('size')
  return
