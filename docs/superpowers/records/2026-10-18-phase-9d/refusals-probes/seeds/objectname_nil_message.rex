-- OBJECTNAME= sent to .nil by a Message with a scope override.
signal on syntax name trapped
m = .Message~new(.nil, .array~of('OBJECTNAME=', .Object), 'I', 'zz')
m~send
say 'sent' .nil~objectName
exit 0
trapped: say 'syntax' condition('O')~code
