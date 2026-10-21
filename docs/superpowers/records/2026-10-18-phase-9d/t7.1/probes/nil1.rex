signal on syntax name trapped
c = .Message~new(.nil, .array~of('COPY', .Object))~send
say 'copy isNil' c~isNil (c = .nil) (c \= .nil) (c~identityHash = .nil~identityHash) (c~hashCode == .nil~hashCode)
say 'isA' c~isA(.Object) c~isInstanceOf(.Object)
t = .table~new; t[.nil] = 1; t[c] = 2; say 'table' t~items t[.nil]
m = .Message~new(.nil, .array~of('OBJECTNAME=', .Object), 'I', 'zz')
m~send
say 'sent' .nil~objectName '|' .nil~string '|' .nil~defaultName '|' c~objectName
say 'result' m~hasResult
e = .Message~new(.nil, .array~of('COPY', .Object))~send
say 'copy after' e~objectName '|' e~string
a = .array~new(3); say a[1]
say 'in string' 'x'.nil
.Message~new(.nil, .array~of('OBJECTNAME=', .Object), 'I', 12)~send
say .nil~objectName
.Message~new(c, .array~of('OBJECTNAME=', .Object), 'I', 'cc')~send
say c~objectName .nil~objectName
say c~objectName
exit 0
trapped: say 'syntax' condition('O')~code condition('O')~message
