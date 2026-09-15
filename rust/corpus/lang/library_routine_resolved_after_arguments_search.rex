/* A call searches past the package's routines once its arguments have run: a
   library an argument loads answers ahead of an external file of the same
   name, a routine an argument adds answers ahead of a kept external file, and
   a name nothing answered before its arguments ran finds what they added. */
do a over .array~of(16, 25)
  say 'library' RxCalcSqrt(loadit(a))
end
do i = 1 to 3
  say 'file' i ext(addext(i))
end
do i = 1 to 2
  say 'unresolved' i foo(addfoo(i))
end
exit
loadit: procedure
  x = .context~package~loadLibrary('rxmath')
  return arg(1)
addext:
  if arg(1) = 2 then .context~package~addRoutine('EXT', .routine~new('x', 'return "added" arg(1)'))
  return arg(1)
addfoo:
  if arg(1) = 1 then .context~package~addRoutine('FOO', .routine~new('x', 'return "added" arg(1)'))
  return arg(1)
