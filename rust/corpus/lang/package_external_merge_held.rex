/* An external routine's public routines, merged into its caller's package,
   keep the routine's package alive after the call returns: calling one and
   finding it through ~findRoutine both reach that package. */
call lineout 'extmerge.rex', 'return 1'
call lineout 'extmerge.rex', '::routine pubr public'
call lineout 'extmerge.rex', '  return .context~package~name~right(12)'
call stream 'extmerge.rex', 'c', 'close'
call sub
do zi = 1 to 1000
  zz = copies('x', 100000)
end
say 'called' pubr()
r = .context~package~findRoutine('PUBR')
say 'found' (r == .nil) r~call
exit
sub: procedure
  x = extmerge()
  return
