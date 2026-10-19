as = .ArrayStream~new
.TraceOutput~destination(as)
src = .array~of( -
 't=.test~new', -
 't~one', -
 'call syssleep 0.001', -
 'res=t~three', -
 'call syssleep 0.001', -
 'return res', -
 '::class test', -
 '::method one', -
 '  forward message "onetwo"', -
 '  i=i/0', -
 '::method onetwo', -
 '  reply', -
 '::method three', -
 '  reply 42', -
 '  forward message "threefour" continue', -
 '  if result<>42 then i=i/0', -
 '::method threefour', -
 '  return 42', -
 '::options trace labels')
r = .routine~new('fw3', src)
res = r~call
.TraceOutput~destination
say 'res' res
do v over as
  if word(v, 2) <> 'Routine' then say left(v, pos(' in package', v' in package') - 1)
end
::class ArrayStream subclass Array inherit Stream
::method init
  expose line
  line = 0
::method lineOut
  parse arg line
  self~append(line)
  return 0
