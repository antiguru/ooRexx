/* A message sent from the rest of a replied method fails it: the condition
   settles on the message as it does on any activity. */
d = .directory~new
say .k~new~m(d)
do until d~hasIndex('m')
  call SysSleep 0.01
end
m = d['m']
do until m~completed
  call SysSleep 0.01
end
say 'hasError' m~hasError
c = m~errorCondition
say 'condition' c~class~id c~code c~message
::class k
::method m
  use arg d
  reply 'replied'
  m = .message~new(self, 'boom')
  d['m'] = m
  m~send
::method boom
  raise syntax 93.900 array('boom failed')
