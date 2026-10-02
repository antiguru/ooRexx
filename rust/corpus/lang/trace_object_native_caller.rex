/* A >I> line's CALLERSTACKFRAME names the primitive method that entered
   the traced method: Message~send, in main and in a started activity. */
s = .sink~new
zz = .traceoutput~destination(s)
.message~new(.w~new, 'two', 'I', 'x')~send
m = .w~new~start('one')
m~wait
zz = .traceoutput~destination(.stderr)
do ln over s~seen
  say ln
end
exit
::class w
::method two unguarded
  trace a
  return
::method one unguarded
  .message~new(.w~new, 'two', 'I', 'y', 'z')~send
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
  seen~append(subword(v~traceline, 1, 3) '| thread' v~thread)
  if v~hasEntry('CALLERSTACKFRAME') then do
    c = v~callerstackframe
    seen~append('  caller' c~allIndexes~sort~makeString('L', ' '))
    seen~append('  caller frame' c~type c~name c~line c~invocation c~executable~class~id c~target~class~id c~arguments~items)
    seen~append('  caller line' c~traceline~strip)
  end
  return 0
