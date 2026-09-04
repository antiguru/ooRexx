/* Required transitively, by package_requires_lib.cls. Not itself a corpus
   program: the differential runs the .rex files of this directory, and this
   is reached only through a ::REQUIRES. */
say 'dep prologue'

::routine depr public
  return 'dep public routine'

::routine shared public
  return 'shared from dep'

::class Helper public
::method tag
  return 'dep public class'
