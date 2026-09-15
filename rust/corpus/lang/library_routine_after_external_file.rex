/* A library routine is found before an external file of the same name, from
   the load onwards, at a call site that found the file before it. */
do i = 1 to 2
  say i RxCalcSqrt(16)
  if i = 1 then say 'load' .context~package~loadLibrary('rxmath')
end
