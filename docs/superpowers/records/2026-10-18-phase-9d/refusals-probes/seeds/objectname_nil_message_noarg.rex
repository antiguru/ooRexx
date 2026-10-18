-- OBJECTNAME= sent to .nil by a Message with a scope override, no argument.
signal on syntax name trapped
.Message~new(.nil, .array~of('OBJECTNAME=', .Object))~send
say 'sent' .nil~objectName
exit 0
trapped: say 'syntax' condition('O')~code
