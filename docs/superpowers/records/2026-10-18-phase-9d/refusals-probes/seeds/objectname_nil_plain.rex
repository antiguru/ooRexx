-- OBJECTNAME= sent to .nil directly.
signal on syntax name trapped
.nil~objectName = 'zz'
say 'sent' .nil~objectName
exit 0
trapped: say 'syntax' condition('O')~code
