-- A SYNTAX condition trapped after it left the activations that raised it:
-- POSITION, STACKFRAMES and TRACEBACK describe the raising levels first.
o = .t~new
call show o~trap
call show r4()
call show r6()
call show o~viaSend(1, 2)
call show inPlace()
exit

show: procedure
  use arg c
  say 'position' c~position filespec('n', c~program) c~stackframes~items c~traceback~items
  do f over c~stackframes
    say ' ' f~type '|' f~name '|' f~line '|' f~arguments~items '|' (f~target == .nil) '|' (f~invocation == .nil) '|' (f~context == .nil)
  end
  do l over c~traceback; say ' ' l; end
  return

inPlace: procedure
  signal on syntax
  interpret 'x = 1 +' "'a'"
  return .nil
syntax: return condition('O')

::routine r4
  signal on syntax
  call a4
  return .nil
syntax: return condition('O')
a4: call b4; return
b4: interpret 'raise syntax 41.1 array("zz")'

::routine r6
  signal on syntax
  call a6 'x'
  return .nil
syntax: return condition('O')
a6: call b6; return
b6: raise syntax 41.1 array("zz")

::class t
::method trap
  signal on syntax name h
  self~raiser
  return .nil
h: return condition('O')
::method raiser
  x = 1 + 'a'
::method viaSend
  signal on syntax
  .message~new(self, 'n', 'I', 'a')~send
  return .nil
syntax: return condition('O')
::method n
  interpret 'x = 1 +' "'a'"
