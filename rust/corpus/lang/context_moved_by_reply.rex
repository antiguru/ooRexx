/* A RexxContext taken before a REPLY follows its activation to the activity
   the rest of the method runs on, with the same invocation. */
o = .k~new
gate = .message~new(o, 'open')
c = o~r(gate)
do while o~waiting == 0
  call SysSleep 0.01
end
say 'line' c~line 'name' c~name 'y' c~variables~y
say 'moved' (c~thread \== .context~thread) 'invocation' (c~invocation == o~inv)
f = c~stackFrames
say 'frames' f~items f[1]~name f[1]~line
gate~send
::class k
::attribute waiting get unguarded
::attribute inv get unguarded
::method init
  expose waiting
  waiting = 0
::method open unguarded
  return 'opened'
::method r unguarded
  expose waiting inv
  use arg gate
  inv = .context~invocation
  reply .context
  y = 'after-reply'
  waiting = 1
  gate~wait
