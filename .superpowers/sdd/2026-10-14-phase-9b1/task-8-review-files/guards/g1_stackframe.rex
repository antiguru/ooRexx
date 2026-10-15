signal on syntax name s1
f = .context~stackFrames[1]
say f~objectName
f~objectName = 'tagged'
say f~objectName
sub = .stackframe~subclass('sf')
say sub~id
o = sub~new
say o~objectName
s1:
if symbol('RC') = 'VAR' then say 'syntax' condition('O')~code
signal on syntax name s2
x = .stackframe~new
say x~objectName
s2:
if symbol('RC') = 'VAR' then say 'syntax2' condition('O')~code
signal on syntax name s3
y = .context~stackFrames[1]~copy
say y~objectName y~name
s3:
if symbol('RC') = 'VAR' then say 'syntax3' condition('O')~code
say 'end'
