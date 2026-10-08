/* A call of the program's own Routine runs as a subroutine and blames CALL. */
if arg() > 0 then do
  parse source s
  say 'inner' arg() arg(1) word(s, 2) .context~executable~class~id
  if arg(1) = 'err' then return 1/0
  return 'r' arg(1)
end
r = .context~executable
say r~call('x', 'y')
say r~callWith(.array~of('z'))
v = 'kept'
say r~call('w') v
x = r~call('err')
