-- .Object's OBJECTNAME= method run on .nil through a method .nil answers.
signal on syntax name trapped
m = .Object~method('OBJECTNAME=')
.nil~setMethod('ZZSET', m)
.nil~zzset('zz')
say 'sent' .nil~objectName
exit 0
trapped: say 'syntax' condition('O')~code
