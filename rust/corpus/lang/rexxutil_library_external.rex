/* EXTERNAL 'LIBRARY rexxutil ...' binds the internal rexxutil package's
   routine table, which exports no methods, and loadExternalRoutine finds
   the same rows by exact spelling, then without regard to case. */
call rr 0
say 'directive' result 'package' .context~package~findRoutine('rr')~package~name
r = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleep')
say 'routine' (r \== .nil) 'package' r~package~name 'call' r~call(0)
say 'caseless' (.Routine~loadExternalRoutine('x', 'LIBRARY rexxutil syssleep') \== .nil)
say 'missing' (.Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysSleepX') == .nil)
say 'upper library' (.Routine~loadExternalRoutine('x', 'LIBRARY REXXUTIL SysSleep') == .nil)
say 'method' (.Method~loadExternalMethod('x', 'LIBRARY rexxutil SysSleep') == .nil)

::routine rr external 'LIBRARY rexxutil SysSleep'
