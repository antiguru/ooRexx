.local~n = 0
.traceOutput~destination(.deep~new)
trace r
x = 1
trace off
.traceOutput~destination
say 'after' .local~n
::class deep
::method lineout unguarded
  .local~n = .local~n + 1
  call spin 800
  return 0
::routine spin
  use arg k
  if k = 0 then return 1/0
  call spin k - 1
