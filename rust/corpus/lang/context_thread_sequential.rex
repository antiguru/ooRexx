/* Two starts one after the other take the same number; main is 1 though it
   asks last. */
o = .t~new
say 'first' o~start('n')~result
say 'second' o~start('n')~result
say 'main' .context~thread
::class t
::method n
  return .context~thread
