.local~flag = 0
w = .worker~new
w~start('run')
.traceOutput~destination(.waiting~new)
trace r
x = 1
trace off
.traceOutput~destination
say 'done' .local~flag
::class worker
::method run
  do i = 1 to 100; nop; end
  .local~flag = 1
::class waiting
::method lineout unguarded
  do while .local~flag = 0; end
  return 0
