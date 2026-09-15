/* A loadExternalRoutine answer loads the library, which makes the entry
   callable by name, and the answer itself runs through call, callWith and []. */
r = .Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcSqrt')
say 'load' r~class~id
say 'main' RxCalcSqrt(16)
say 'call' r~call(16)
say 'callWith' r~callWith(.array~of(2, 4))
say 'bracket' r[9]
say 'package' (r~package == .nil)
