/* The natives that run a Rexx method or routine: ~new into INIT, the
   Message and Object sends, a started message, Routine~call, and REPLY. */
o = .k~new('first-object')
m = .message~new(o, 'SHOUT', 'I', 'hello there')
say m~send
say m~result m~completed m~hasError
say .message~new(o, 'SHOUT', 'I', 'second message')~send
say .message~new(o, 'SHOUT')~sendWith(o, .array~of('sent with'))
say o~send('SHOUT', 'via object send')
say o~sendWith('SHOUT', .array~of('via object sendwith'))
t = o~start('SHOUT', 'started message')
say t~result t~completed
t = .message~new(o, 'BOOM')
signal on syntax name trapped
t~send
trapped:
say 'trapped' condition('C') t~hasError
signal off syntax
r = .context~package~findRoutine('RT')
say r~call('routine call', 2)
say r~callWith(.array~of('routine callwith', 3))
say r['routine brackets', 1]
say .k~new('second-object')~name
say .r~new~m
exit
::class k
::method init
  expose nm
  use arg nm
  self~note('init' nm)
::method note
  say 'note' arg(1)
::method name
  expose nm
  return nm
::method shout
  return arg(1)~upper
::method boom
  return 1 / 0
::routine rt
  use arg s, n
  return copies(s' ', n)
::class r
::method m
  reply 'replied first'
  say 'continuation' self~tag
::method tag
  return 'tagged'
