/* A call site that found a library routine, or an internal routine, finds a
   public routine of the same name that addPackage made callable in front of
   it, on its next pass. */
x = .context~package~loadLibrary('rxmath')
do i = 1 to 2
  say 'library' i RxCalcSqrt(16)
  if i = 1 then .context~package~addPackage(.context~package~loadPackage('sqrt.cls'))
end
do i = 1 to 2
  say 'internal' i filespec('N', '/a/b.c')
  if i = 1 then .context~package~addPackage(.context~package~loadPackage('filespec.cls'))
end
