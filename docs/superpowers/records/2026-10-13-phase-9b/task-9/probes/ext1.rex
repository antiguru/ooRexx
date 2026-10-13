say 'go'
r = .Routine~loadExternalRoutine('x','LIBRARY rexxutil SysSleep')
say 'ler nil' (r == .nil)
if r \== .nil then do; say 'ler pkg' r~package~name; say 'ler call' r~call(0); end
r2 = .Routine~loadExternalRoutine('x','LIBRARY rexxutil SysSleepX')
say 'ler bad nil' (r2 == .nil)
r3 = .Routine~loadExternalRoutine('x','LIBRARY REXXUTIL SysSleep')
say 'ler upper nil' (r3 == .nil)
r4 = .Routine~loadExternalRoutine('x','LIBRARY rexxutil syssleep')
say 'ler lower entry nil' (r4 == .nil)
m = .Method~loadExternalMethod('x','LIBRARY rexxutil SysSleep')
say 'lem nil' (m == .nil)
say 'pkg lib' .context~package~loadLibrary('rexxutil') .context~package~loadLibrary('REXX')
say 'after loadLibrary findRoutine nil' (.context~package~findRoutine('SysSleep') == .nil)
call rr 0
say 'rr' result
say 'rr pkg' .context~package~findRoutine('rr')~package~name
say 'same as ler' (.context~package~findRoutine('rr') == r)
say 'done'
::routine rr external 'LIBRARY rexxutil SysSleep'
