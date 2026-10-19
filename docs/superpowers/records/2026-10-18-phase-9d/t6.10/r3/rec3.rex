.traceOutput~destination(.deep~new)
trace r
x = 1
trace off
.traceOutput~destination
say 'after'
::class deep
::method lineout unguarded
  call spin
  return 0
spin: call spin
