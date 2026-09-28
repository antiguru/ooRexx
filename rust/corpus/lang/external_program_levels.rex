-- A program called as a routine or function is a ROUTINE level named as it
-- was called: in .context~stackframes, in a trapped condition's frames and
-- traceback (and while its directives install), and in its >I> line.
signal on syntax name s1
call ext 5
s1: call show condition('O'), 'call'
signal on syntax name s2
x = ext(5)
s2: call show condition('O'), 'function'
signal on syntax name s3
call t2
s3: nop
say 'inside' 'kind.rex'()
signal on syntax name s4
x = 'bad.rex'(3)
s4: call show condition('O'), 'install'
call traced
call rr 1
call 'badinit.rex'
exit
traced:
  do 1
    x = tr(1)
  end
  return
t2:
  signal on syntax name s5
  x = extfn(1)
  return
s5: call show condition('O'), 't2'; return
::requires 'lib.cls'
::routine show
  use arg c, tag
  say tag c~code c~position filespec('n', c~program) c~propagated
  do f over c~stackframes; say '  F' f~type filespec('n', f~name) f~line f~arguments~items; end
  do l over c~traceback; say '  T' l; end
