/* A RexxContext read from another activity answers for its own activation
   there; once that activation has ended it is 98.981. */
o = .k~new
gate = .message~new(o, 'open')
m = o~start('hold', gate)
do while o~ctx == .nil
  call SysSleep 0.01
end
c = o~ctx
say 'line' c~line 'name' c~name 'args' c~args~items
say 'thread' c~thread 'main' .context~thread
say 'x' c~variables~x 'digits' c~digits 'form' c~form 'fuzz' c~fuzz
say 'executable' c~executable~class~id 'package' (c~package == .context~package)
say 'rs' c~rs~class~id 'condition' c~condition~class~id 'interpreter' c~interpreter
f = c~stackFrames
say 'frames' f~items f[1]~name f[1]~line f[1]~type
inv = c~invocation
gate~send
m~wait
say 'invocation kept' (inv == o~inv)
signal on syntax
say c~line
exit
syntax:
  say 'ended:' condition('o')~code condition('o')~message
::class k
::attribute ctx get unguarded
::attribute inv get unguarded
::method init
  expose ctx
  ctx = .nil
::method open unguarded
  return 'opened'
::method hold unguarded
  expose ctx inv
  use arg gate
  x = 'in-hold'
  numeric digits 12
  numeric form engineering
  inv = .context~invocation
  ctx = .context
  gate~wait
