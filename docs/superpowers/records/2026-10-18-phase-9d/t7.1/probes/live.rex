signal on syntax name done
x = TestNestedAttach(.w~new)
done: say 'raised' condition('O')~code
exit 0
::requires 'orxfunction' LIBRARY
::class w
::method string
  do f over .context~stackFrames
    e = f~executable
    say f~type f~name e~class~id (e == .context~package~findRoutine('TESTNESTEDATTACH'))
  end
  return 'w'
