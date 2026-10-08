/* The Routine standing for the program's main section runs it again on call
 * and callWith. */
if arg(1) <> '' then return 'got' arg(1)
r = .context~executable
say r~class~id
say r~call('x')
say r~callWith(.array~of('y'))
