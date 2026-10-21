/* StackFrame~executable on the frame of a routine a ::REQUIRES LIBRARY
   merged, from the condition it raises and from a live .context~stackFrames
   while it calls back: the Routine object findRoutine and
   loadExternalRoutine answer. */
p = .context~package
signal on syntax name s1
call TestRaiseException0 93900
s1: call show 'call', condition('O')
signal on syntax name s2
x = TestRaiseException0(93900)
s2: call show 'function', condition('O')
lr = .Routine~loadExternalRoutine('TestRaiseException0', 'LIBRARY orxfunction TestRaiseException0')
say 'loaded' (lr == p~findRoutine('TESTRAISEEXCEPTION0'))
signal on syntax name s3
y = TestNestedAttach(.w~new)
s3: say 'attach' condition('O')~code
exit 0
::requires 'orxfunction' LIBRARY
::routine show
  use arg tag, condition
  do f over condition~stackFrames
    e = f~executable
    name = f~name
    if f~type == 'PROGRAM' then name = ''
    say tag f~type name e~class~id (e == .context~package~findRoutine('TESTRAISEEXCEPTION0'))
  end
::class w
::method string
  do f over .context~stackFrames
    e = f~executable
    name = f~name
    if f~type == 'PROGRAM' then name = ''
    say 'live' f~type name e~class~id (e == .context~package~findRoutine('TESTNESTEDATTACH'))
  end
  return 'w'
