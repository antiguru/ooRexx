r = .Routine~loadExternalRoutine('x','LIBRARY rexxutil SysSleep')
signal on syntax
call SysSleep 'abc'
syntax: do f over condition('o')~stackFrames; say f~type f~name (f~executable == r) f~executable~class~id; end
signal on syntax name s2
call ww 'abc'
s2: do f over condition('o')~stackFrames; say f~type f~name (f~executable == r); end
exit
::routine ww external 'LIBRARY rexxutil SysSleep'
