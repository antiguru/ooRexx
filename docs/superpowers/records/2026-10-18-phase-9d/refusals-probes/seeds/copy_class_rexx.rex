-- COPY of a Rexx-defined class object.
signal on syntax name trapped
c = .zz~copy
say 'copied' c~id (c == .zz) c~new~m
exit 0
trapped: say 'syntax' condition('O')~code
::class zz
::method m
  return 'm'
