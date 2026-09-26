/* A redirecting handler that writes, then raises or throws. */
call AddCmd 'xr', 'r'
o.0 = 'untouched'
address xr 'WRAISE ERROR' with output stem o.
say 'wraise' rc .rs o.0 o.1
o. = ''; o.0 = 'untouched'
signal on syntax name s1
address xr 'WSYNTAX' with output stem o.
say 'not here'
s1:
say 'wsyntax' condition('o')~code o.0 o.1
o. = ''; o.0 = 'untouched'
signal on syntax name s2
address xr 'WTHROW' with output stem o.
say 'not here'
s2:
say 'wthrow' condition('o')~code o.0 o.1 Dtors()
o. = ''; o.0 = 'untouched'
call AddCmd 'xd', 'd'
address xd with output stem o.
signal on syntax name s3
'hello'
say 'not here'
s3:
say 'global' condition('o')~code condition('o')~message
::requires 'cmd' LIBRARY
