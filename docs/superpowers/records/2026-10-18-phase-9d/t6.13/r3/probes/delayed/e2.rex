call on error name h
o = .G~new
address system o~cmd
say 'after' rc
exit
h: say 'handler' condition('c') '['condition('d')']' 'rc' rc 'sigl' sigl
return
::class G
::method cmd
  raise error 3 return 'exit 5'
  return 'exit 5'
