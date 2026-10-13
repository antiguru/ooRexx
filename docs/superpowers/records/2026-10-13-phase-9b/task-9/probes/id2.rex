a = .Routine~loadExternalRoutine('x','LIBRARY rexxutil SysSleep')
f = .Routine~loadExternalRoutine('x','LIBRARY REXX Filespec')
signal on syntax
call SysSleep 'x'
back:
signal on syntax name syn2
call filespec 'Q', 'x'
back2:
say 'directive' (.context~package~findRoutine('rr') == a) (.context~package~findRoutine('ff') == f)
exit
syntax:
e = condition('o')~stackFrames[1]~executable
say 'frame sysSleep' (e == a) e~class~id
signal back
syn2:
e = condition('o')~stackFrames[1]~executable
say 'frame filespec' (e == f) e~class~id
signal back2
::routine rr external 'LIBRARY rexxutil SysSleep'
::routine ff external 'LIBRARY REXX Filespec'
