/* An untrapped condition in the rest of a replied method is reported on its
   own activity, through .traceOutput (RAISE.testGroup's raise_insert_crlf). */
t = .traceOutput~destination(.arrayStream~new)
d = .directory~new
say .k~new~m(d)
do until d~hasIndex('done')
  call SysSleep 0.01
end
do i = 1 to 50 while t~items < 3
  call SysSleep 0.01
end
.traceOutput~destination(t)
say 'lines' t~items
say t[3]
::class k
::method m
  use arg d
  reply 'replied'
  d['done'] = 1
  raise syntax 98.900 array('from the continuation')
::class arrayStream subclass Array inherit Stream
::method init
::method lineOut
  self~append(arg(1)~makeString)
  return 0
