-- .Object's COPY run on a RexxContext through a scope override.
signal on syntax name trapped
c = .Message~new(.context, .array~of('COPY', .Object))~send
say 'copied' c~class~id
exit 0
trapped: say 'syntax' condition('O')~code
