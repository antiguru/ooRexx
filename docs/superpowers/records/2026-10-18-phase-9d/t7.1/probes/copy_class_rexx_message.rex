-- .Object's COPY run on a Rexx-defined class object through a scope override.
signal on syntax name trapped
c = .Message~new(.zz, .array~of('COPY', .Object))~send
say 'copied' c~class~id (c == .zz) c~id c~new~m (c~new~class == c)
exit 0
trapped: say 'syntax' condition('O')~code
::class zz
::method m
  return 'm'
