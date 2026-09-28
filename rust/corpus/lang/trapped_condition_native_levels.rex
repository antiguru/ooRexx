-- A condition trapped after a native level: the native frame's context, and
-- POSITION and PROGRAM from the first frame with a package -- none for a
-- built-in method, the caller for a condition a native call re-raises, and
-- the native frame itself, which has no line, for a raise its boundary made.
call t1
call t2
call t3
call t4
call t5
call t6
exit

t1: procedure
  signal on syntax
  x = 'abc'~substr('x')
  return
syntax: call show; return
t2: procedure
  signal on syntax
  call SysSleep 'abc'
  return
syntax: call show; return
t3: procedure
  signal on syntax
  x = filespec('N')
  return
syntax: call show; return
t4: procedure
  signal on syntax
  x = tint('abc')
  return
syntax: call show; return
t5: procedure
  signal on syntax
  x = .k~new~m('abc')
  return
syntax: call show; return
t6: procedure
  signal on syntax
  x = .k~new~viaSend
  return
syntax: call show; return

show:
  c = condition('O')
  say c~code c~position c~hasIndex('POSITION') filespec('n', c~program) c~traceback~items
  do l over c~traceback; say ' ' l; end
  do f over c~stackframes
    say ' ' f~type f~name f~line (f~context == .nil) f~arguments~items
  end
  return

::routine tint external "LIBRARY orxfunction TestIntArg"
::class k
::method m external "LIBRARY orxmethod TestIntArg"
::method viaSend
  return .message~new(self, 'inner')~send
::method inner
  return 1 / 0
