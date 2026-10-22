r = .routine~loadExternalRoutine('myname', 'LIBRARY orxfunction TestRaiseException0')
signal on syntax
r~call('x')
exit
syntax:
do l over condition('O')~traceback; say l; end
do f over condition('O')~stackframes; say f~type f~name; end
