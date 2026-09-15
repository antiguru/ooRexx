/* Routine~call runs a library routine under the entry's own spelling, below
   the CALL method's own traceback line. */
r = .Routine~loadExternalRoutine('rr', 'LIBRARY rxmath RxCalcSqrt')
say 'start'
say r~call(1, 2, 3)
