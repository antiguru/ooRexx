do name over .array~of('STDOUT', 'STDOUT:')
  s = .stream~new(name)
  s~charout('abc')
  call lineout 'STDERR', name 'held' s~query('position write char') 'sys' s~query('position sys')
  .stdout~charout('uv')
  call lineout 'STDERR', name 'both' s~query('position write char') 'sys' s~query('position sys')
  s~flush
  call lineout 'STDERR', name 'flushed' s~query('position write char') 'sys' s~query('position sys')
  s~close
end
