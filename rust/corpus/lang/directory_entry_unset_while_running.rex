/* A Directory entry that takes itself out of the table while it runs keeps
 * its .context~executable through a forced collection: the activation holds it. */
d = .directory~new
d~setMethod('x', 'self~unsetMethod("X"); call gc "force"; return .context~executable~source[1]')
say d~x
