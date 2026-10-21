.traceoutput~destination(.dst~new)
trace r
x = 1
trace off
say 'after' .local~halted
::class dst
::method lineout
  if .local~halted == 1 then return 0
  signal on halt
  do forever; end
halt:
  .local~halted = 1
  return 0
