/* importedRoutines answers a package's merged routines, library routines
   included, in the package that required the library and in one requiring it. */
p = .context~package
say 'main findRoutine' p~findRoutine('rxcalcsqrt')~call(4)
say 'main findPublicRoutine' p~findPublicRoutine('RxCalcSqrt')~call(9)
say 'main routines' p~routines['RXCALCSQRT']
say 'main publicRoutines' p~publicRoutines['RXCALCSQRT']
say 'main importedRoutines' p~importedRoutines['RXCALCSQRT']
say 'main importedRoutines MID' p~importedRoutines['MID']
m = mid()
say 'mid findRoutine' m~findRoutine('RXCALCSQRT')~call(16)
say 'mid findPublicRoutine' m~findPublicRoutine('RXCALCSQRT')~call(25)
say 'mid routines' m~routines['RXCALCSQRT']
say 'mid publicRoutines' m~publicRoutines['RXCALCSQRT']
say 'mid importedRoutines' m~importedRoutines['RXCALCSQRT']
::requires 'mid.cls'
