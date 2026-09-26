t = .T~new
p = t~loadpkg('/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-5/pkgdir/pk1.cls')
say p~class p~name~right(7)
p2 = t~loadpkg('/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-5/pkgdir/pk1.cls')
say p2~name~right(7)
say count(t~routines(p)) count(t~pubroutines(p)) count(t~classes(p)) count(t~pubclasses(p)) count(t~methods(p))
say t~findpkgclass(p, 'pk1class') t~findpkgclass(p, 'PK1CLASS')~new~hi
d = t~loadpkgdata('inline', 'say "inline prolog"' || '0a'x || '::routine ir public' || '0a'x || '  return "ir!"')
say d~class d~name t~routines(d)['IR']~call
r = t~newroutine('Testing', 'use arg x' || '0d0a'x || 'return x*2')
say r~class r~call(21) t~callroutine(r, .array~of(5)) t~isroutine(r) t~ismethod(r)
m = t~newmethod('Meth', 'return "m:" arg(1)')
say m~class t~ismethod(m) (t~getmethodpkg(m)~class) (t~getroutinepkg(r)~class)
say t~callprogram('/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/surface-5/pkgdir/prog.rex', .array~of('a', 'b'))
exit
count: procedure
  n = 0
  do i over arg(1); n = n + 1; end
  return n
::class T
::method loadpkg external "LIBRARY orxmethod TestLoadPackage"
::method loadpkgdata external "LIBRARY orxmethod TestLoadPackageFromData"
::method routines external "LIBRARY orxmethod TestGetPackageRoutines"
::method pubroutines external "LIBRARY orxmethod TestGetPackagePublicRoutines"
::method classes external "LIBRARY orxmethod TestGetPackageClasses"
::method pubclasses external "LIBRARY orxmethod TestGetPackagePublicClasses"
::method methods external "LIBRARY orxmethod TestGetPackageMethods"
::method findpkgclass external "LIBRARY orxmethod TestFindPackageClass"
::method newroutine external "LIBRARY orxmethod TestNewRoutine"
::method newmethod external "LIBRARY orxmethod TestNewMethod"
::method callroutine external "LIBRARY orxmethod TestCallRoutine"
::method callprogram external "LIBRARY orxmethod TestCallProgram"
::method isroutine external "LIBRARY orxmethod TestIsRoutine"
::method ismethod external "LIBRARY orxmethod TestIsMethod"
::method getmethodpkg external "LIBRARY orxmethod TestGetMethodPackage"
::method getroutinepkg external "LIBRARY orxmethod TestGetRoutinePackage"
