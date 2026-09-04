/* Required by package_requires.rex, twice, and by nothing else. Named
   without an extension there, so the search's own .cls step is what finds
   it. */
say 'lib prologue'
say 'lib sees shared as:' shared()

::requires 'package_requires_dep'

::routine pubr public
  return 'lib public routine'

::routine privr
  return 'lib private routine'

::routine shared public
  return 'shared from lib'

::class Widget public
::method describe
  return 'lib public class'
::method viaDep
  return .Helper~new~tag

::class Hidden
::method describe
  return 'lib private class'
