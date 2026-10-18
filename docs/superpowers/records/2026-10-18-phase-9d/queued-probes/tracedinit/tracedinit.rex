s = .sink~new
zz = .traceoutput~destination(s)
o = .w~new
zz = .traceoutput~destination(.stderr)
say 'after' s~seen~items
do ln over s~seen
  say ln
end
::class w
::method init
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
