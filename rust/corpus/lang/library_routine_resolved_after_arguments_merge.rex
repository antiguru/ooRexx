/* A call looks its routine up once its arguments have run: an argument that
   adds a routine of the call's name to the package, or merges one into it
   through loadPackage, is the routine the call runs, from the first pass on. */
.local~stepped = 0
do i = 1 to 3
  say 'addRoutine' i foo(step(16))
end
r = .Routine~newFile('r.rex', .context~package)
r~call
exit
step:
  .local~stepped = .local~stepped + 1
  if .local~stepped = 1 then .context~package~addRoutine('FOO', .routine~new('x', 'return "added" arg(1)'))
  return arg(1)
::routine foo
  return 'main' arg(1)
::requires 'rxmath' LIBRARY
