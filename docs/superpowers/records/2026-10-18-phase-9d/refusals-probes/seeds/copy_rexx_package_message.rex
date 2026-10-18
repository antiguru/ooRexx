-- .Object's COPY run on the REXX package through a scope override.
signal on syntax name trapped
p = .Array~package
c = .Message~new(p, .array~of('COPY', .Object))~send
say 'copied' c~class~id (c == p)
exit 0
trapped: say 'syntax' condition('O')~code
