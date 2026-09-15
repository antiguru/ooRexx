/* A library routine an imported-routine table holds is one Routine object:
   findRoutine answers it on every ask, importedRoutines answers the same one,
   so does another package importing the same library, and so does
   loadExternalRoutine under either spelling of the entry. A ::ROUTINE a
   required package made public is one object the same way. The hashes are
   compared as strings: a numeric comparison at the default digits can call
   two distinct objects' hashes equal. */
p = .context~package
a = p~findRoutine('RXCALCSQRT')
say 'library findRoutine' (a~identityHash == p~findRoutine('RXCALCSQRT')~identityHash)
say 'library importedRoutines' (p~importedRoutines['RXCALCSQRT']~identityHash == a~identityHash)
say 'library another package' (midpackage()~findRoutine('RXCALCSQRT')~identityHash == a~identityHash)
d = .Routine~loadExternalRoutine('x', 'LIBRARY rxmath RxCalcSqrt')
e = .Routine~loadExternalRoutine('y', 'LIBRARY rxmath rxcalcsqrt')
say 'loadExternalRoutine' (d~identityHash == a~identityHash) (e~identityHash == d~identityHash) d~call(16)
say 'distinct objects' (.object~new~identityHash == .object~new~identityHash)
c = p~findRoutine('PUBR')
say 'routine findRoutine' (c~identityHash == p~findRoutine('PUBR')~identityHash)
say 'routine importedRoutines' (p~importedRoutines['PUBR']~identityHash == c~identityHash)
::requires 'rxmath' LIBRARY
::requires 'mid.cls'
