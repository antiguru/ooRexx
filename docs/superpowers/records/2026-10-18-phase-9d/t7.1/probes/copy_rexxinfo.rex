-- COPY of .RexxInfo, and .Object's COPY run on it through a scope override.
signal on syntax name trapped
c = .Message~new(.RexxInfo, .array~of('COPY', .Object))~send
say 'copied' c~class~id (c == .RexxInfo)
exit 0
trapped: say 'syntax' condition('O')~code
