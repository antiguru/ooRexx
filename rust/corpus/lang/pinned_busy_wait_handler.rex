-- A started activity busy-waiting inside a CALL ON handler, where it is
-- pinned, ends once the activity that sets its flag runs on its stack.
f = .flag~new
a = f~start('waitForFlag')
f~start('setFlag')
say a~result
::class flag
::attribute done unguarded
::method init
  expose done
  done = 0
::method waitForFlag unguarded
  call on error name h
  'exit 1'
  return 'A ended'
h:
  do while \self~done
  end
  say 'handler done'
  return
::method setFlag unguarded
  self~done = 1
