/* The fields a TraceObject carries of where its line was traced: THREAD,
   the traced frame, the guard entries of an unguarded method with no GUARD
   instruction, and on a >I> line the CALLERSTACKFRAME. That is the frame
   below the traced one, else the one that started the activity: the frame
   that sent ~start, or the replying frame for a REPLY's continuation, either
   with the starting activity's THREAD. */
s = .sink~new
zz = .traceoutput~destination(s)
call rt 'a', 'b'
o = .w~new
m = o~start('work')
m~wait
r = o~rep
do while \o~done
  call SysSleep 0.01
end
zz = .traceoutput~destination(.stderr)
do ln over s~seen
  say ln
end
exit
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
  seen~append(subword(v~traceline, 1, 3) '| thread' v~thread 'invocation' v~invocation~datatype('W'))
  seen~append('  entries' v~allIndexes~sort~makeString('L', ' '))
  f = v~stackframe
  seen~append('  frame' f~type filespec('name', f~name) f~line f~executable~class~id f~target~class~id)
  if v~hasEntry('ISGUARDED') then
    seen~append('  guard' v~isguarded v~scopelockcount v~hasscopelock v~receiver~class~id)
  if v~hasEntry('CALLERSTACKFRAME') then do
    c = v~callerstackframe
    seen~append('  caller' c~allIndexes~sort~makeString('L', ' '))
    if c~hasEntry('THREAD') then seen~append('  spawner thread' c~thread)
    seen~append('  caller frame' c~type filespec('name', c~name) c~line c~executable~class~id c~target~class~id c~arguments~items)
    seen~append('  caller line' c~traceline~strip)
  end
  return 0
::class w
::attribute done unguarded
::method init
  expose done
  done = 0
::method work unguarded
  trace a
  return
::method rep unguarded
  trace a
  reply 5
  self~done = 1
::routine rt
  trace a
  return
