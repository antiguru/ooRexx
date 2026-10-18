-- .Object's COPY run on .nil through a Message with a scope override.
signal on syntax name trapped
c = .Message~new(.nil, .array~of('COPY', .Object))~send
say 'copied' (c == .nil) c~class~id
exit 0
trapped: say 'syntax' condition('O')~code
