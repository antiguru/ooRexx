p = .context~package
lr = .Routine~loadExternalRoutine('TestRaiseException0', 'LIBRARY orxfunction TestRaiseException0')
signal on syntax name s5
call TestRaiseException0 93900
s5: call show 'call', condition('O'), lr
signal on syntax name s6
lr~call(93900)
s6: call show 'lr', condition('O'), lr
signal on syntax name s7
x = TestRaiseException0(93900)
s7: call show 'fn', condition('O'), lr
exit 0
::requires 'orxfunction' LIBRARY
::routine show
  use arg tag, c, lr
  do f over c~stackFrames
    e = f~executable
    say tag f~type f~name e~class~id (e == lr) (e == .context~package~findRoutine('TESTRAISEEXCEPTION0'))
  end
