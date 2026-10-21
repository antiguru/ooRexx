-- .Object's COPY run on a built-in class object through a scope override.
signal on syntax name trapped
c = .Message~new(.Array, .array~of('COPY', .Object))~send
say 'copied' c~class~id (c == .Array) c~id c~of(1, 2)~items
exit 0
trapped: say 'syntax' condition('O')~code
