-- .Object's COPY run on a StackFrame through a scope override (control: the
-- crate's COPY answers a StackFrame).
signal on syntax name trapped
f = .context~stackFrames[1]
c = .Message~new(f, .array~of('COPY', .Object))~send
say 'copied' c~class~id (c == f) c~type c~line
exit 0
trapped: say 'syntax' condition('O')~code
