s = .sink~new
zz = .traceoutput~destination(s)
.message~new(.w~new, 'two')~triggered(.nil)
zz = .traceoutput~destination(.stderr)
do ln over s~seen
  say 'sink:' ln
end
exit
::class w
::method two
  trace a
  return
::class sink
::method init
  expose seen
  seen = .array~new
::method seen
  expose seen
  return seen
::method lineout unguarded
  expose seen
  use arg v
  seen~append(subword(v~traceline, 1, 3))
  return 0
