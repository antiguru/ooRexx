s = .stream~new('STDOUT:')
.stdout~charout('uv')
s~charout('abc')
call lineout 'STDERR', 'pending' s~query('position write char') s~query('position sys')
call physical 'before-alias-flush'
s~flush
call lineout 'STDERR', 'flushed' s~query('position write char') s~query('position sys')
call physical 'after-alias-flush'
s~charout('D')
call lineout 'STDERR', 'continued' s~query('position write char') s~query('position sys')
s~flush
call physical 'continued-bytes'
exit
physical: procedure
  use arg label
  call lineout 'STDERR', label .stream~new('captured-output.bin')~query('size')
  return
