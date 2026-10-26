.local~foo = 7
signal on syntax
call 'cyc.rex'
exit
syntax: o = condition('o'); say o~code o~position
do f over o~stackframes; say f~type filespec('name', f~name) f~line; end
