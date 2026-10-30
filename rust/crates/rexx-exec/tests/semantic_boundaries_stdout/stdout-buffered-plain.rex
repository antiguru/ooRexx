s = .stream~new('STDOUT')
s~charout('abc')
call lineout 'STDERR', 'held' s~query('position write char')
call lineout 'STDERR', 'sys' s~query('position sys')
s~flush
call lineout 'STDERR', 'flushed' s~query('position write char') 'sys' s~query('position sys')
s~close
