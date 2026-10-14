/* The package whose ::ROUTINE first bound an external library procedure is
   that procedure's package for the run, so it outlives every reference a
   program holds, while a package nothing reaches is collected. */
p = .Package~new('lp', ('::routine sq public external "LIBRARY rxmath RxCalcSqrt"',))
u = .Routine~loadExternalRoutine('lx', 'LIBRARY rxmath RxCalcSqrt')
say 'bound' (u~package == p) (p~findRoutine('SQ')~package == p)
h = p~identityHash
w = .WeakReference~new(p)
s = .WeakReference~new(.Package~new('sentinel', 'nop'))
drop p
i = 0
loop until s~value == .nil | i > 5000
  z = copies('y', 100000)
  i += 1
end
say 'sentinel collected' (s~value == .nil)
say 'binder kept' (w~value \== .nil) (u~package~identityHash == h)
say 'later' .Routine~loadExternalRoutine('lx', 'LIBRARY rxmath RxCalcSqrt')~package~name
