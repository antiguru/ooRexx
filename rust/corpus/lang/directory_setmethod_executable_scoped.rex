/* Directory~setMethod installs a .nil-scoped copy of a scoped Method object,
 * which is what .context~executable answers. */
m = .u~method('M')
d = .directory~new
d~setMethod('x', m)
e = d~x
say (e == m) e~scope
::class u
::method m
  return .context~executable
