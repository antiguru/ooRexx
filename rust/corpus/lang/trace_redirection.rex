/* Every trace line reaches .TRACEOUTPUT as one LINEOUT of a TraceObject.
   The monitor's destination is the route a program can change here, so the
   redirection is `~destination` rather than an entry in .local. The traced
   region includes a builtin call with an argument, whose `>A>` line is
   emitted between pushing that argument and consuming it. */
s = .sink~new
zz = .traceoutput~destination(s)
trace i
aa = 1 + 1
zz = length('abcd')
trace off
zz = .traceoutput~destination(.stderr)
say 'delivered' s~count
do ln over s~seen
  say ln
end

::class sink
::method init
  expose seen count
  seen = .array~new
  count = 0
::method count
  expose count
  return count
::method seen
  expose seen
  return seen
::method lineout
  expose seen count
  use arg v
  count = count + 1
  seen~append(v~class~id '[' || v~request('STRING') || ']')
  return 0
